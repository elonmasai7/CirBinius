use cirbinius_cbir::CbirConstraint;

use super::{OptimizationContext, OptimizationPass, OptimizedConstraint};

pub struct GenericCompatPass;

impl OptimizationPass for GenericCompatPass {
    fn name(&self) -> &'static str {
        "generic_compat"
    }
    fn description(&self) -> &'static str {
        "Fallback: routes constraints to compatibility lowering"
    }

    fn optimize(
        &self,
        constraints: &[CbirConstraint],
        _context: &OptimizationContext,
    ) -> Vec<OptimizedConstraint> {
        constraints
            .iter()
            .map(|c| OptimizedConstraint {
                original_ids: vec![c.id],
                gate_kind: "generic_compat".to_string(),
                limb_width: None,
            })
            .collect()
    }
}
