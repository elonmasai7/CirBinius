use cirbinius_cbir::{CbirConstraint, CbirDocument};
use cirbinius_limb_engine::LimbWidth;

use crate::detector::registry::DetectorRegistry;
use crate::detector::{Confidence, DetectedPattern};
use crate::pass::{OptimizationContext, OptimizationPass, OptimizedConstraint};

pub struct OptimizationPipeline {
    passes: Vec<Box<dyn OptimizationPass>>,
    registry: DetectorRegistry,
    #[allow(dead_code)]
    min_confidence: Confidence,
}

impl OptimizationPipeline {
    pub fn new(registry: DetectorRegistry, min_confidence: Confidence) -> Self {
        let passes: Vec<Box<dyn OptimizationPass>> = vec![
            Box::new(super::BooleanPass),
            Box::new(super::AndPass),
            Box::new(super::OrPass),
            Box::new(super::XorPass),
            Box::new(super::IsZeroPass),
            Box::new(super::IsEqualPass),
            Box::new(super::BitDecompositionPass),
            Box::new(super::RangeCheckPass),
            Box::new(super::MuxPass),
            Box::new(super::MerklePass),
            Box::new(super::HashPass),
            Box::new(super::PoseidonPass),
            Box::new(super::MiMCPass),
            Box::new(super::CircomlibPass),
            Box::new(super::SelectorPass),
            Box::new(super::GenericCompatPass),
        ];
        Self {
            passes,
            registry,
            min_confidence,
        }
    }

    pub fn run(&self, document: &CbirDocument) -> (Vec<OptimizedConstraint>, Vec<DetectedPattern>) {
        let groups = self.registry.detect_groups(&document.constraints);
        let mut detected_patterns = groups.clone();
        let mut used_ids: std::collections::HashSet<u64> = std::collections::HashSet::new();
        let mut optimized = Vec::new();

        // Process groups first
        for group in &groups {
            let group_constraints: Vec<&CbirConstraint> = document
                .constraints
                .iter()
                .filter(|c| group.constraint_ids.contains(&c.id))
                .collect();
            let group_owned: Vec<CbirConstraint> = group_constraints.into_iter().cloned().collect();

            for pass in &self.passes {
                if pass.name() == group.pattern_name {
                    let ctx = OptimizationContext {
                        confidence: group.confidence,
                        pattern_name: group.pattern_name.clone(),
                        related_ids: group.constraint_ids.clone(),
                    };
                    let result = pass.optimize(&group_owned, &ctx);
                    for id in &group.constraint_ids {
                        used_ids.insert(*id);
                    }
                    optimized.extend(result);
                    break;
                }
            }
        }

        // Process individual constraints not in any group
        for constraint in &document.constraints {
            if used_ids.contains(&constraint.id) {
                continue;
            }
            let matches = self.registry.detect_all(constraint);
            let best = matches.first();
            let ctx = match best {
                Some((name, conf)) => {
                    detected_patterns.push(DetectedPattern {
                        pattern_name: name.clone(),
                        confidence: *conf,
                        constraint_ids: vec![constraint.id],
                    });
                    OptimizationContext {
                        confidence: *conf,
                        pattern_name: name.clone(),
                        related_ids: vec![constraint.id],
                    }
                }
                None => OptimizationContext {
                    confidence: Confidence::Rejected,
                    pattern_name: "generic_compat".to_string(),
                    related_ids: vec![constraint.id],
                },
            };

            let mut applied = false;
            for pass in &self.passes {
                if pass.name() == ctx.pattern_name {
                    let result = pass.optimize(std::slice::from_ref(constraint), &ctx);
                    optimized.extend(result);
                    applied = true;
                    break;
                }
            }
            if !applied {
                optimized.push(OptimizedConstraint {
                    original_ids: vec![constraint.id],
                    gate_kind: "generic_compat".to_string(),
                    limb_width: Some(LimbWidth::Auto.name().to_string()),
                });
            }
        }

        (optimized, detected_patterns)
    }
}
