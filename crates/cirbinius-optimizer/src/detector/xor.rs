use cirbinius_cbir::CbirConstraint;

use super::{Confidence, PatternDetector, has_hint_token, is_one, is_zero_linear};

pub struct XorDetector;

impl PatternDetector for XorDetector {
    fn name(&self) -> &'static str {
        "xor"
    }
    fn description(&self) -> &'static str {
        "Detects a + b - 2*a*b = c XOR gate"
    }
    fn default_confidence(&self) -> Confidence {
        Confidence::Exact
    }

    fn detect(&self, constraint: &CbirConstraint) -> Option<Confidence> {
        if constraint.kind != "mul" {
            return None;
        }
        // XOR is 2 constraints: a*b == m AND a + b - 2*m == c
        // First constraint: mul where c=0, b has -2 coefficient for some wire
        if constraint.a.terms.len() != 1 {
            return None;
        }
        if !is_one(&constraint.a.terms[0].coeff_hex) {
            return None;
        }
        // Check b has the same wire with -2 coefficient
        let wire = constraint.a.terms[0].wire_id;
        if constraint.b.terms.len() != 1 {
            return None;
        }
        if constraint.b.terms[0].wire_id != wire {
            return None;
        }
        // coeff should be -2 mod field => lots of f's
        let coeff = constraint.b.terms[0].coeff_hex.trim_start_matches("0x");
        if coeff.chars().all(|c| c == 'f') && coeff.len() >= 2 {
            // This is the mul constraint where a*b with wire appears as -2*wire
            // Check c is zero wire (the intermediate variable)
            if constraint.c.terms.len() == 1 && !is_zero_linear(&constraint.c) {
                if has_hint_token(constraint, &["xor"]) {
                    Some(Confidence::Exact)
                } else {
                    Some(Confidence::Strong)
                }
            } else {
                None
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cirbinius_cbir::{CbirConstraint, CbirLinearCombination, CbirTerm};

    #[test]
    fn xor_requires_hint_or_structural_match() {
        let d = XorDetector;
        let c = CbirConstraint {
            id: 1,
            kind: "mul".to_string(),
            a: CbirLinearCombination {
                terms: vec![CbirTerm {
                    wire_id: 1,
                    coeff_hex: "0x01".to_string(),
                }],
            },
            b: CbirLinearCombination {
                terms: vec![CbirTerm {
                    wire_id: 1,
                    coeff_hex: "0xfffffffffffffffe".to_string(),
                }],
            },
            c: CbirLinearCombination {
                terms: vec![CbirTerm {
                    wire_id: 5,
                    coeff_hex: "0x01".to_string(),
                }],
            },
            signal_hints: vec!["xor".to_string()],
        };
        assert_eq!(d.detect(&c), Some(Confidence::Exact));
    }
}
