use cirbinius_cbir::CbirConstraint;

use super::{Confidence, PatternDetector, is_one};

pub struct OrDetector;

impl PatternDetector for OrDetector {
    fn name(&self) -> &'static str {
        "or"
    }
    fn description(&self) -> &'static str {
        "Detects a + b - a*b = c OR gate"
    }
    fn default_confidence(&self) -> Confidence {
        Confidence::Exact
    }

    fn detect(&self, constraint: &CbirConstraint) -> Option<Confidence> {
        // OR is 2 constraints: a*b == m AND a + b - m == c
        if constraint.kind != "mul" {
            return None;
        }
        if constraint.a.terms.len() != 1 {
            return None;
        }
        if !is_one(&constraint.a.terms[0].coeff_hex) {
            return None;
        }
        if constraint.b.terms.len() != 1 {
            return None;
        }
        if !is_one(&constraint.b.terms[0].coeff_hex) {
            return None;
        }
        // a*wire1 + b*wire2 should produce c with wire1+wire2-m
        // For the mul constraint: check c is non-zero (intermediate wire m)
        if constraint.c.terms.len() == 1
            && constraint.c.terms[0].wire_id > 0
            && is_one(&constraint.c.terms[0].coeff_hex)
        {
            if constraint
                .signal_hints
                .iter()
                .any(|h| h.to_lowercase().contains("or"))
            {
                return Some(Confidence::Exact);
            }
            return Some(Confidence::Strong);
        }
        None
    }
}
