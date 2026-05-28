use cirbinius_cbir::CbirConstraint;

use super::{OptimizationContext, OptimizationPass, OptimizedConstraint};

pub struct IsZeroPass;

impl OptimizationPass for IsZeroPass {
    fn name(&self) -> &'static str {
        "is-zero"
    }
    fn description(&self) -> &'static str {
        "Optimizes IsZero gadget"
    }

    fn optimize(
        &self,
        _constraints: &[CbirConstraint],
        context: &OptimizationContext,
    ) -> Vec<OptimizedConstraint> {
        vec![OptimizedConstraint {
            original_ids: context.related_ids.clone(),
            gate_kind: "is_zero".to_string(),
            limb_width: Some("field".to_string()),
        }]
    }
}
