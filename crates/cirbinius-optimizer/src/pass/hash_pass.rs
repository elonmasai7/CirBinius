use cirbinius_cbir::CbirConstraint;

use super::{OptimizationContext, OptimizationPass, OptimizedConstraint};

pub struct HashPass;

impl OptimizationPass for HashPass {
    fn name(&self) -> &'static str {
        "hash-preimage"
    }
    fn description(&self) -> &'static str {
        "Optimizes hash preimage constraint groups"
    }

    fn optimize(
        &self,
        _constraints: &[CbirConstraint],
        context: &OptimizationContext,
    ) -> Vec<OptimizedConstraint> {
        vec![OptimizedConstraint {
            original_ids: context.related_ids.clone(),
            gate_kind: "hash".to_string(),
            limb_width: Some("field".to_string()),
        }]
    }
}
