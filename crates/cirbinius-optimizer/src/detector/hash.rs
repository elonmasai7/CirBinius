use cirbinius_cbir::CbirConstraint;

use super::{Confidence, PatternDetector, has_hint_token};

pub struct HashPreimageDetector;

impl PatternDetector for HashPreimageDetector {
    fn name(&self) -> &'static str {
        "hash-preimage"
    }
    fn description(&self) -> &'static str {
        "Detects hash preimage constraint groups"
    }
    fn default_confidence(&self) -> Confidence {
        Confidence::Heuristic
    }

    fn detect(&self, constraint: &CbirConstraint) -> Option<Confidence> {
        if has_hint_token(constraint, &["hash", "preimage"]) {
            Some(Confidence::Heuristic)
        } else {
            None
        }
    }

    fn detect_group(&self, constraints: &[CbirConstraint]) -> Vec<Vec<u64>> {
        HashPreimageDetector::find_hash_clusters(constraints)
    }
}

impl HashPreimageDetector {
    pub fn find_hash_clusters(constraints: &[CbirConstraint]) -> Vec<Vec<u64>> {
        let hash_ids: Vec<u64> = constraints
            .iter()
            .filter(|c| {
                has_hint_token(
                    c,
                    &["poseidon", "sha", "keccak", "mimc", "pedersen", "blake"],
                )
            })
            .map(|c| c.id)
            .collect();
        if hash_ids.len() >= 4 {
            vec![hash_ids]
        } else {
            vec![]
        }
    }
}
