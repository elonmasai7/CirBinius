use cirbinius_cbir::CbirConstraint;

use super::{OptimizationContext, OptimizationPass, OptimizedConstraint};

pub struct XorPass;

impl OptimizationPass for XorPass {
    fn name(&self) -> &'static str {
        "xor"
    }
    fn description(&self) -> &'static str {
        "Optimizes XOR gates"
    }

    fn optimize(
        &self,
        _constraints: &[CbirConstraint],
        context: &OptimizationContext,
    ) -> Vec<OptimizedConstraint> {
        vec![OptimizedConstraint {
            original_ids: context.related_ids.clone(),
            gate_kind: "xor".to_string(),
            limb_width: Some("1".to_string()),
        }]
    }
}
