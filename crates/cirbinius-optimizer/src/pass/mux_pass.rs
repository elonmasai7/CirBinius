use cirbinius_cbir::CbirConstraint;

use super::{OptimizationContext, OptimizationPass, OptimizedConstraint};

pub struct MuxPass;

impl OptimizationPass for MuxPass {
    fn name(&self) -> &'static str {
        "mux-selector"
    }
    fn description(&self) -> &'static str {
        "Optimizes MUX/selector patterns"
    }

    fn optimize(
        &self,
        _constraints: &[CbirConstraint],
        context: &OptimizationContext,
    ) -> Vec<OptimizedConstraint> {
        vec![OptimizedConstraint {
            original_ids: context.related_ids.clone(),
            gate_kind: "mux".to_string(),
            limb_width: Some("1".to_string()),
        }]
    }
}
