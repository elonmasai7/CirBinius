use cirbinius_cbir::CbirConstraint;

use super::{OptimizationContext, OptimizationPass, OptimizedConstraint};

pub struct BitDecompositionPass;

impl OptimizationPass for BitDecompositionPass {
    fn name(&self) -> &'static str {
        "bit-decomposition"
    }
    fn description(&self) -> &'static str {
        "Optimizes Num2Bits/BitDecomposition patterns"
    }

    fn optimize(
        &self,
        _constraints: &[CbirConstraint],
        context: &OptimizationContext,
    ) -> Vec<OptimizedConstraint> {
        vec![OptimizedConstraint {
            original_ids: context.related_ids.clone(),
            gate_kind: "num2bits".to_string(),
            limb_width: Some("1".to_string()),
        }]
    }
}
