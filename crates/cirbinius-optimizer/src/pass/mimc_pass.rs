use cirbinius_cbir::CbirConstraint;

use super::{OptimizationContext, OptimizationPass, OptimizedConstraint};

pub struct MiMCPass;

impl OptimizationPass for MiMCPass {
    fn name(&self) -> &'static str {
        "mimc"
    }
    fn description(&self) -> &'static str {
        "Optimizes MiMC Feistel round constraints"
    }

    fn optimize(
        &self,
        _constraints: &[CbirConstraint],
        context: &OptimizationContext,
    ) -> Vec<OptimizedConstraint> {
        vec![OptimizedConstraint {
            original_ids: context.related_ids.clone(),
            gate_kind: "mimc_round".to_string(),
            limb_width: Some("field".to_string()),
        }]
    }
}
