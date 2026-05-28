use cirbinius_cbir::CbirConstraint;

use super::{Confidence, PatternDetector, has_hint_token, is_one, is_zero_linear};

pub struct AndDetector;

impl PatternDetector for AndDetector {
    fn name(&self) -> &'static str {
        "and"
    }
    fn description(&self) -> &'static str {
        "Detects a * b = c AND gate"
    }
    fn default_confidence(&self) -> Confidence {
        Confidence::Strong
    }

    fn detect(&self, constraint: &CbirConstraint) -> Option<Confidence> {
        if constraint.kind != "mul" {
            return None;
        }
        if constraint.a.terms.len() != 1
            || constraint.b.terms.len() != 1
            || constraint.c.terms.len() != 1
        {
            return None;
        }
        let all_unit = is_one(&constraint.a.terms[0].coeff_hex)
            && is_one(&constraint.b.terms[0].coeff_hex)
            && is_one(&constraint.c.terms[0].coeff_hex);
        if !all_unit {
            return None;
        }
        if has_hint_token(constraint, &["and", "bool"]) {
            Some(Confidence::Strong)
        } else if is_zero_linear(&constraint.a) && is_zero_linear(&constraint.b) {
            None
        } else {
            Some(Confidence::Heuristic)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cirbinius_cbir::{CbirConstraint, CbirLinearCombination, CbirTerm};

    fn and_constraint() -> CbirConstraint {
        CbirConstraint {
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
                    wire_id: 2,
                    coeff_hex: "0x01".to_string(),
                }],
            },
            c: CbirLinearCombination {
                terms: vec![CbirTerm {
                    wire_id: 3,
                    coeff_hex: "0x01".to_string(),
                }],
            },
            signal_hints: vec!["main.andGate".to_string()],
        }
    }

    #[test]
    fn and_detects_with_hint() {
        let d = AndDetector;
        assert_eq!(d.detect(&and_constraint()), Some(Confidence::Strong));
    }
}
