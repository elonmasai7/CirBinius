use cirbinius_cbir::CbirConstraint;

use super::{Confidence, PatternDetector, is_one, is_zero_linear};

pub struct IsZeroDetector;

impl PatternDetector for IsZeroDetector {
    fn name(&self) -> &'static str {
        "is-zero"
    }
    fn description(&self) -> &'static str {
        "Detects IsZero gadget (in*inv==1-out, in*out==0)"
    }
    fn default_confidence(&self) -> Confidence {
        Confidence::Exact
    }

    fn detect(&self, constraint: &CbirConstraint) -> Option<Confidence> {
        // First constraint of IsZero: in * inv = 1 - out
        // => A = {wire_in}, B = {wire_inv}, C = {wire_one, wire_out}
        if constraint.kind != "mul" {
            return None;
        }
        if constraint.a.terms.len() != 1 || constraint.b.terms.len() != 1 {
            return None;
        }
        if !is_one(&constraint.a.terms[0].coeff_hex) {
            return None;
        }
        if !is_one(&constraint.b.terms[0].coeff_hex) {
            return None;
        }
        let wire_a = constraint.a.terms[0].wire_id;
        let wire_b = constraint.b.terms[0].wire_id;
        if wire_a == 0 || wire_b == 0 {
            return None;
        }
        // C should be: 1*wire_one + (-1)*wire_out OR just wire_out
        if constraint.c.terms.is_empty() {
            return None;
        }
        let has_one = constraint.c.terms.iter().any(|t| t.wire_id == 0);
        let has_out = !constraint.c.terms.is_empty();
        if (has_one || has_out)
            && constraint
                .signal_hints
                .iter()
                .any(|h| h.to_lowercase().contains("iszero") || h.to_lowercase().contains("inv"))
        {
            Some(Confidence::Exact)
        } else {
            None
        }
    }

    fn detect_group(&self, constraints: &[CbirConstraint]) -> Vec<Vec<u64>> {
        if constraints.len() < 2 {
            return vec![];
        }
        for pair in constraints.windows(2) {
            let (c1, c2) = (&pair[0], &pair[1]);
            // c1: in * inv == 1 - out
            // c2: in * out == 0
            if c1.kind == "mul"
                && c2.kind == "mul"
                && is_zero_linear(&c2.c)
                && c2.a.terms.len() == 1
                && c2.b.terms.len() == 1
            {
                // Check c2 is: same wire_in * wire_out == 0
                let _inv_wire = c1.b.terms[0].wire_id;
                let out_wire = if c1.c.terms.len() == 1 {
                    c1.c.terms[0].wire_id
                } else {
                    0
                };
                if c2.a.terms[0].wire_id == c1.a.terms[0].wire_id
                    && c2.b.terms[0].wire_id == out_wire
                    && is_one(&c2.a.terms[0].coeff_hex)
                    && is_one(&c2.b.terms[0].coeff_hex)
                {
                    return vec![vec![c1.id, c2.id]];
                }
            }
        }
        vec![]
    }
}
