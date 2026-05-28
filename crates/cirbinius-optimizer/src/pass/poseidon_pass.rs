use cirbinius_cbir::CbirConstraint;

use super::{OptimizationContext, OptimizationPass, OptimizedConstraint};

pub struct PoseidonPass;

impl OptimizationPass for PoseidonPass {
    fn name(&self) -> &'static str {
        "poseidon"
    }
    fn description(&self) -> &'static str {
        "Optimizes Poseidon hash round constraints"
    }

    fn optimize(
        &self,
        _constraints: &[CbirConstraint],
        context: &OptimizationContext,
    ) -> Vec<OptimizedConstraint> {
        vec![OptimizedConstraint {
            original_ids: context.related_ids.clone(),
            gate_kind: "poseidon_round".to_string(),
            limb_width: Some("field".to_string()),
        }]
    }
}
