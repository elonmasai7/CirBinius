use cirbinius_cbir::CbirConstraint;

use super::{OptimizationContext, OptimizationPass, OptimizedConstraint};

pub struct SelectorPass;

impl OptimizationPass for SelectorPass {
    fn name(&self) -> &'static str {
        "selector"
    }
    fn description(&self) -> &'static str {
        "Optimizes sel*(1-sel)==0 selector constraints"
    }

    fn optimize(
        &self,
        _constraints: &[CbirConstraint],
        context: &OptimizationContext,
    ) -> Vec<OptimizedConstraint> {
        vec![OptimizedConstraint {
            original_ids: context.related_ids.clone(),
            gate_kind: "selector".to_string(),
            limb_width: Some("1".to_string()),
        }]
    }
}
