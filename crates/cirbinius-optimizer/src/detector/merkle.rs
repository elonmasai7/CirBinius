use cirbinius_cbir::CbirConstraint;

use super::{Confidence, PatternDetector, has_hint_token};

pub struct MerklePathDetector;

impl PatternDetector for MerklePathDetector {
    fn name(&self) -> &'static str {
        "merkle-path"
    }
    fn description(&self) -> &'static str {
        "Detects Merkle tree path verification constraints"
    }
    fn default_confidence(&self) -> Confidence {
        Confidence::Heuristic
    }

    fn detect(&self, constraint: &CbirConstraint) -> Option<Confidence> {
        if has_hint_token(constraint, &["merkle", "path", "sibling"]) {
            Some(Confidence::Heuristic)
        } else if has_hint_token(constraint, &["root", "leaf"]) {
            Some(Confidence::Experimental)
        } else {
            None
        }
    }

    fn detect_group(&self, constraints: &[CbirConstraint]) -> Vec<Vec<u64>> {
        // Merkle path = repeated hash calls with sibling/root hints
        let merkle_ids: Vec<u64> = constraints
            .iter()
            .filter(|c| has_hint_token(c, &["merkle", "path", "sibling", "root"]))
            .map(|c| c.id)
            .collect();
        if merkle_ids.len() >= 3 {
            vec![merkle_ids]
        } else {
            vec![]
        }
    }
}
