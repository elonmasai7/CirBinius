use cirbinius_cbir::CbirConstraint;

use super::{OptimizationContext, OptimizationPass, OptimizedConstraint};

pub struct BooleanPass;

impl OptimizationPass for BooleanPass {
    fn name(&self) -> &'static str {
        "boolean-constraint"
    }
    fn description(&self) -> &'static str {
        "Optimizes boolean constraints to native binary"
    }

    fn optimize(
        &self,
        _constraints: &[CbirConstraint],
        context: &OptimizationContext,
    ) -> Vec<OptimizedConstraint> {
        vec![OptimizedConstraint {
            original_ids: context.related_ids.clone(),
            gate_kind: "boolean".to_string(),
            limb_width: Some("1".to_string()),
        }]
    }
}
