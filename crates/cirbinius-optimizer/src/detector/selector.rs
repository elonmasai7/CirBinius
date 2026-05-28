use cirbinius_cbir::CbirConstraint;

use super::{Confidence, PatternDetector, is_one};

pub struct SelectorDetector;

impl PatternDetector for SelectorDetector {
    fn name(&self) -> &'static str {
        "selector"
    }
    fn description(&self) -> &'static str {
        "Detects sel*(1-sel)==0 selector constraint"
    }
    fn default_confidence(&self) -> Confidence {
        Confidence::Exact
    }

    fn detect(&self, constraint: &CbirConstraint) -> Option<Confidence> {
        if constraint.kind != "mul" {
            return None;
        }
        if constraint.a.terms.len() != 1 {
            return None;
        }
        let wire = constraint.a.terms[0].wire_id;
        if wire == 0 || !is_one(&constraint.a.terms[0].coeff_hex) {
            return None;
        }
        if constraint.b.terms.len() != 2 {
            return None;
        }
        // Check: b = wire*1 + ONE*(-1)
        let has_sel = constraint
            .b
            .terms
            .iter()
            .any(|t| t.wire_id == wire && is_one(&t.coeff_hex));
        let has_neg = constraint.b.terms.iter().any(|t| t.wire_id == 0);
        if has_sel && has_neg && constraint.c.terms.is_empty() {
            Some(Confidence::Exact)
        } else {
            None
        }
    }
}
