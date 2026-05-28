use cirbinius_cbir::CbirConstraint;

use super::{OptimizationContext, OptimizationPass, OptimizedConstraint};

pub struct RangeCheckPass;

impl OptimizationPass for RangeCheckPass {
    fn name(&self) -> &'static str {
        "range-check"
    }
    fn description(&self) -> &'static str {
        "Optimizes range check constraints"
    }

    fn optimize(
        &self,
        _constraints: &[CbirConstraint],
        context: &OptimizationContext,
    ) -> Vec<OptimizedConstraint> {
        vec![OptimizedConstraint {
            original_ids: context.related_ids.clone(),
            gate_kind: "range_check".to_string(),
            limb_width: Some("32".to_string()),
        }]
    }
}
