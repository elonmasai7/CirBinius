use cirbinius_cbir::CbirConstraint;

use super::{OptimizationContext, OptimizationPass, OptimizedConstraint};

pub struct CircomlibPass;

impl OptimizationPass for CircomlibPass {
    fn name(&self) -> &'static str {
        "circomlib-gadget"
    }
    fn description(&self) -> &'static str {
        "Optimizes known circomlib gadgets"
    }

    fn optimize(
        &self,
        _constraints: &[CbirConstraint],
        context: &OptimizationContext,
    ) -> Vec<OptimizedConstraint> {
        vec![OptimizedConstraint {
            original_ids: context.related_ids.clone(),
            gate_kind: format!("circomlib/{}", context.pattern_name),
            limb_width: Some("field".to_string()),
        }]
    }
}
