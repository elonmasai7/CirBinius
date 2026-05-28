use cirbinius_cbir::CbirConstraint;

use super::{OptimizationContext, OptimizationPass, OptimizedConstraint};

pub struct OrPass;

impl OptimizationPass for OrPass {
    fn name(&self) -> &'static str {
        "or"
    }
    fn description(&self) -> &'static str {
        "Optimizes OR gates"
    }

    fn optimize(
        &self,
        _constraints: &[CbirConstraint],
        context: &OptimizationContext,
    ) -> Vec<OptimizedConstraint> {
        vec![OptimizedConstraint {
            original_ids: context.related_ids.clone(),
            gate_kind: "or".to_string(),
            limb_width: Some("1".to_string()),
        }]
    }
}
