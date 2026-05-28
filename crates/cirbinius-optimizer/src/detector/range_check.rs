use cirbinius_cbir::CbirConstraint;

use super::{Confidence, PatternDetector, has_hint_token};

pub struct RangeCheckDetector;

impl PatternDetector for RangeCheckDetector {
    fn name(&self) -> &'static str {
        "range-check"
    }
    fn description(&self) -> &'static str {
        "Detects range check constraints"
    }
    fn default_confidence(&self) -> Confidence {
        Confidence::Strong
    }

    fn detect(&self, constraint: &CbirConstraint) -> Option<Confidence> {
        if has_hint_token(constraint, &["range", "check", "bound"]) {
            Some(Confidence::Strong)
        } else if has_hint_token(
            constraint,
            &["lt", "leq", "less", "greater", "cmp", "compare"],
        ) {
            Some(Confidence::Heuristic)
        } else {
            None
        }
    }

    fn detect_group(&self, constraints: &[CbirConstraint]) -> Vec<Vec<u64>> {
        // A range check is a bit decomposition + comparison
        let bit_ids: Vec<u64> = constraints
            .iter()
            .filter(|c| has_hint_token(c, &["range"]))
            .map(|c| c.id)
            .collect();

        if bit_ids.len() >= 2 {
            vec![bit_ids]
        } else {
            vec![]
        }
    }
}
