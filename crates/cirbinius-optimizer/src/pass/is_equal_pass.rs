use cirbinius_cbir::CbirConstraint;

use super::{OptimizationContext, OptimizationPass, OptimizedConstraint};

pub struct IsEqualPass;

impl OptimizationPass for IsEqualPass {
    fn name(&self) -> &'static str {
        "is-equal"
    }
    fn description(&self) -> &'static str {
        "Optimizes IsEqual gadget"
    }

    fn optimize(
        &self,
        _constraints: &[CbirConstraint],
        context: &OptimizationContext,
    ) -> Vec<OptimizedConstraint> {
        vec![OptimizedConstraint {
            original_ids: context.related_ids.clone(),
            gate_kind: "is_equal".to_string(),
            limb_width: Some("field".to_string()),
        }]
    }
}
