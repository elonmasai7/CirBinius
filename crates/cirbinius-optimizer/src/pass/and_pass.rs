use cirbinius_cbir::CbirConstraint;

use super::{OptimizationContext, OptimizationPass, OptimizedConstraint};

pub struct AndPass;

impl OptimizationPass for AndPass {
    fn name(&self) -> &'static str {
        "and"
    }
    fn description(&self) -> &'static str {
        "Optimizes AND gates"
    }

    fn optimize(
        &self,
        _constraints: &[CbirConstraint],
        context: &OptimizationContext,
    ) -> Vec<OptimizedConstraint> {
        vec![OptimizedConstraint {
            original_ids: context.related_ids.clone(),
            gate_kind: "and".to_string(),
            limb_width: Some("1".to_string()),
        }]
    }
}
