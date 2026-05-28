use cirbinius_cbir::CbirConstraint;

use super::{Confidence, PatternDetector, has_hint_token};

pub struct PoseidonDetector;

impl PatternDetector for PoseidonDetector {
    fn name(&self) -> &'static str {
        "poseidon"
    }
    fn description(&self) -> &'static str {
        "Detects Poseidon hash round constraints"
    }
    fn default_confidence(&self) -> Confidence {
        Confidence::Strong
    }

    fn detect(&self, constraint: &CbirConstraint) -> Option<Confidence> {
        if has_hint_token(constraint, &["poseidon"]) {
            if has_hint_token(constraint, &["round", "sbox", "mix"]) {
                Some(Confidence::Strong)
            } else {
                Some(Confidence::Heuristic)
            }
        } else {
            None
        }
    }

    fn detect_group(&self, constraints: &[CbirConstraint]) -> Vec<Vec<u64>> {
        // Poseidon round = ~3 constraints per round (sbox + mix + add_const)
        let poseidon_ids: Vec<u64> = constraints
            .iter()
            .filter(|c| has_hint_token(c, &["poseidon"]))
            .map(|c| c.id)
            .collect();
        if poseidon_ids.len() >= 3 {
            vec![poseidon_ids]
        } else {
            vec![]
        }
    }
}
