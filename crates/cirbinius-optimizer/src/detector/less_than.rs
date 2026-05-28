use cirbinius_cbir::CbirConstraint;

use super::{Confidence, PatternDetector, has_hint_token};

pub struct LessThanDetector;

impl PatternDetector for LessThanDetector {
    fn name(&self) -> &'static str {
        "less-than"
    }
    fn description(&self) -> &'static str {
        "Detects LessThan comparison gadget"
    }
    fn default_confidence(&self) -> Confidence {
        Confidence::Heuristic
    }

    fn detect(&self, constraint: &CbirConstraint) -> Option<Confidence> {
        if has_hint_token(constraint, &["less", "lt"]) {
            Some(Confidence::Heuristic)
        } else {
            None
        }
    }

    fn detect_group(&self, constraints: &[CbirConstraint]) -> Vec<Vec<u64>> {
        // LessThan = range_check on (b - a + 2^n) + comparison
        let less_ids: Vec<u64> = constraints
            .iter()
            .filter(|c| has_hint_token(c, &["less", "lt", "comparison"]))
            .map(|c| c.id)
            .collect();
        if less_ids.len() >= 2 {
            vec![less_ids]
        } else {
            vec![]
        }
    }
}
