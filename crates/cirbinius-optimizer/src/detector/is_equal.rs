use cirbinius_cbir::CbirConstraint;

use super::{Confidence, PatternDetector, is_one, is_zero_linear};

pub struct IsEqualDetector;

impl PatternDetector for IsEqualDetector {
    fn name(&self) -> &'static str {
        "is-equal"
    }
    fn description(&self) -> &'static str {
        "Detects IsEqual gadget (IsZero on a-b)"
    }
    fn default_confidence(&self) -> Confidence {
        Confidence::Exact
    }

    fn detect(&self, constraint: &CbirConstraint) -> Option<Confidence> {
        // IsEqual uses IsZero on (a - b)
        // The subtraction is typically in a linear constraint: diff = a - b
        // Then IsZero constraints on diff
        if constraint.kind == "mul"
            && constraint.a.terms.len() == 1
            && constraint.b.terms.len() == 1
            && is_one(&constraint.a.terms[0].coeff_hex)
            && constraint.c.terms.len() <= 1
            && constraint
                .signal_hints
                .iter()
                .any(|h| h.to_lowercase().contains("isequal"))
        {
            Some(Confidence::Exact)
        } else {
            None
        }
    }

    fn detect_group(&self, constraints: &[CbirConstraint]) -> Vec<Vec<u64>> {
        // IsEqual = IsZero applied to (a - b)
        // Look for: diff = a - b (linear), then IsZero on diff (3 constraints: diff*inv=1-out, diff*out=0)
        if constraints.len() < 3 {
            return vec![];
        }
        for window in constraints.windows(3) {
            let (c1, c2, c3) = (&window[0], &window[1], &window[2]);
            // c1: diff = a - b (linear constraint, kind might be "add")
            if c1.kind != "add" {
                continue;
            }
            if c2.kind == "mul" && c3.kind == "mul" {
                let diff_wire = c2.a.terms[0].wire_id;
                if c3.a.terms[0].wire_id == diff_wire
                    && is_zero_linear(&c3.c)
                    && is_one(&c2.a.terms[0].coeff_hex)
                    && is_one(&c3.a.terms[0].coeff_hex)
                {
                    return vec![vec![c1.id, c2.id, c3.id]];
                }
            }
        }
        vec![]
    }
}
