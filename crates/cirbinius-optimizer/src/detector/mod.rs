pub mod registry;

mod and;
mod bit_decomposition;
mod bits2num;
mod boolean;
mod circomlib;
mod greater_than;
mod hash;
mod is_equal;
mod is_zero;
mod less_than;
mod merkle;
mod mimc;
mod mux;
mod not;
mod or;
mod poseidon;
mod range_check;
mod selector;
mod sha;
mod xor;

use cirbinius_cbir::CbirConstraint;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Confidence {
    Exact,
    Strong,
    Heuristic,
    Experimental,
    Rejected,
}

impl Confidence {
    pub fn meets_threshold(&self, threshold: Confidence) -> bool {
        self <= &threshold
    }
}

#[derive(Debug, Clone)]
pub struct DetectedPattern {
    pub pattern_name: String,
    pub confidence: Confidence,
    pub constraint_ids: Vec<u64>,
}

pub trait PatternDetector: Send + Sync {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn default_confidence(&self) -> Confidence;
    fn detect(&self, constraint: &CbirConstraint) -> Option<Confidence>;
    fn detect_group(&self, _constraints: &[CbirConstraint]) -> Vec<Vec<u64>> {
        Vec::new()
    }
}

pub use and::AndDetector;
pub use bit_decomposition::BitDecompositionDetector;
pub use bits2num::Bits2NumDetector;
pub use boolean::BooleanConstraintDetector;
pub use circomlib::CircomlibGadgetDetector;
pub use greater_than::GreaterThanDetector;
pub use hash::HashPreimageDetector;
pub use is_equal::IsEqualDetector;
pub use is_zero::IsZeroDetector;
pub use less_than::LessThanDetector;
pub use merkle::MerklePathDetector;
pub use mimc::MiMCDetector;
pub use mux::MuxSelectorDetector;
pub use not::NotDetector;
pub use or::OrDetector;
pub use poseidon::PoseidonDetector;
pub use range_check::RangeCheckDetector;
pub use selector::SelectorDetector;
pub use sha::ShaDetector;
pub use xor::XorDetector;

fn has_hint_token(constraint: &CbirConstraint, tokens: &[&str]) -> bool {
    constraint.signal_hints.iter().any(|hint| {
        let lowered = hint.to_lowercase();
        tokens.iter().any(|token| lowered.contains(token))
    })
}

fn is_zero_linear(linear: &cirbinius_cbir::CbirLinearCombination) -> bool {
    linear.terms.is_empty() || linear.terms.iter().all(|t| t.coeff_hex == "0x0")
}

fn is_one(coeff_hex: &str) -> bool {
    coeff_hex.trim_start_matches("0x").trim_start_matches('0') == "1"
}
