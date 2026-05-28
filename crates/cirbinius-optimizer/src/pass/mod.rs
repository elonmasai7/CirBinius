pub mod pipeline;

mod and_pass;
mod bc_pass;
mod bit_decomposition_pass;
mod circomlib_pass;
mod generic_pass;
mod hash_pass;
mod is_equal_pass;
mod is_zero_pass;
mod merkle_pass;
mod mimc_pass;
mod mux_pass;
mod or_pass;
mod poseidon_pass;
mod range_check_pass;
mod selector_pass;
mod xor_pass;

use cirbinius_cbir::CbirConstraint;

use crate::detector::Confidence;

#[derive(Debug, Clone)]
pub struct OptimizationContext {
    pub confidence: Confidence,
    pub pattern_name: String,
    pub related_ids: Vec<u64>,
}

#[derive(Debug, Clone)]
pub struct OptimizedConstraint {
    pub original_ids: Vec<u64>,
    pub gate_kind: String,
    pub limb_width: Option<String>,
}

pub trait OptimizationPass: Send + Sync {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn optimize(
        &self,
        constraints: &[CbirConstraint],
        context: &OptimizationContext,
    ) -> Vec<OptimizedConstraint>;
}

pub use and_pass::AndPass;
pub use bc_pass::BooleanPass;
pub use bit_decomposition_pass::BitDecompositionPass;
pub use circomlib_pass::CircomlibPass;
pub use generic_pass::GenericCompatPass;
pub use hash_pass::HashPass;
pub use is_equal_pass::IsEqualPass;
pub use is_zero_pass::IsZeroPass;
pub use merkle_pass::MerklePass;
pub use mimc_pass::MiMCPass;
pub use mux_pass::MuxPass;
pub use or_pass::OrPass;
pub use pipeline::OptimizationPipeline;
pub use poseidon_pass::PoseidonPass;
pub use range_check_pass::RangeCheckPass;
pub use selector_pass::SelectorPass;
pub use xor_pass::XorPass;
