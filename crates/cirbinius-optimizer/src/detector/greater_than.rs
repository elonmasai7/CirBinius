use cirbinius_cbir::CbirConstraint;

use super::{Confidence, PatternDetector, has_hint_token};

pub struct GreaterThanDetector;

impl PatternDetector for GreaterThanDetector {
    fn name(&self) -> &'static str {
        "greater-than"
    }
    fn description(&self) -> &'static str {
        "Detects GreaterThan comparison gadget"
    }
    fn default_confidence(&self) -> Confidence {
        Confidence::Heuristic
    }

    fn detect(&self, constraint: &CbirConstraint) -> Option<Confidence> {
        if has_hint_token(constraint, &["greater", "gt"]) {
            Some(Confidence::Heuristic)
        } else if has_hint_token(constraint, &["compare"]) {
            Some(Confidence::Experimental)
        } else {
            None
        }
    }
}
