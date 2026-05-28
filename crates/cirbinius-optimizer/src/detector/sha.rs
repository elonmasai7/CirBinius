use cirbinius_cbir::CbirConstraint;

use super::{Confidence, PatternDetector, has_hint_token};

pub struct ShaDetector;

impl PatternDetector for ShaDetector {
    fn name(&self) -> &'static str {
        "sha"
    }
    fn description(&self) -> &'static str {
        "Detects SHA-256 round constraints"
    }
    fn default_confidence(&self) -> Confidence {
        Confidence::Heuristic
    }

    fn detect(&self, constraint: &CbirConstraint) -> Option<Confidence> {
        if has_hint_token(constraint, &["sha", "sha256"]) {
            if has_hint_token(constraint, &["round", "sigma", "ch", "maj"]) {
                Some(Confidence::Strong)
            } else {
                Some(Confidence::Heuristic)
            }
        } else {
            None
        }
    }

    fn detect_group(&self, constraints: &[CbirConstraint]) -> Vec<Vec<u64>> {
        let sha_ids: Vec<u64> = constraints
            .iter()
            .filter(|c| has_hint_token(c, &["sha", "sha256"]))
            .map(|c| c.id)
            .collect();
        if sha_ids.len() >= 4 {
            vec![sha_ids]
        } else {
            vec![]
        }
    }
}
