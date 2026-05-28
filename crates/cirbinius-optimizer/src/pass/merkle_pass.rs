use cirbinius_cbir::CbirConstraint;

use super::{OptimizationContext, OptimizationPass, OptimizedConstraint};

pub struct MerklePass;

impl OptimizationPass for MerklePass {
    fn name(&self) -> &'static str {
        "merkle-path"
    }
    fn description(&self) -> &'static str {
        "Optimizes Merkle path verification"
    }

    fn optimize(
        &self,
        _constraints: &[CbirConstraint],
        context: &OptimizationContext,
    ) -> Vec<OptimizedConstraint> {
        vec![OptimizedConstraint {
            original_ids: context.related_ids.clone(),
            gate_kind: "merkle_path".to_string(),
            limb_width: Some("field".to_string()),
        }]
    }
}
