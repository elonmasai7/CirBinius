use cirbinius_cbir::CbirConstraint;

use super::{Confidence, PatternDetector, is_one, is_zero_linear};

pub struct BooleanConstraintDetector;

impl PatternDetector for BooleanConstraintDetector {
    fn name(&self) -> &'static str {
        "boolean-constraint"
    }
    fn description(&self) -> &'static str {
        "Detects x * (x - 1) == 0 boolean constraints"
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
        let wire_id = constraint.a.terms[0].wire_id;
        if wire_id == 0 || !is_one(&constraint.a.terms[0].coeff_hex) {
            return None;
        }
        if !is_zero_linear(&constraint.c) {
            return None;
        }
        let mut has_wire = false;
        let mut has_neg_one = false;
        for term in &constraint.b.terms {
            if term.wire_id == wire_id && is_one(&term.coeff_hex) {
                has_wire = true;
            }
            if term.wire_id == 0
                && term
                    .coeff_hex
                    .trim_start_matches("0x")
                    .chars()
                    .all(|c| c == 'f')
            {
                has_neg_one = true;
            }
        }
        if has_wire && has_neg_one && constraint.b.terms.len() == 2 {
            Some(Confidence::Exact)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cirbinius_cbir::{CbirConstraint, CbirLinearCombination, CbirTerm};

    fn bool_constraint(wire_id: u32) -> CbirConstraint {
        CbirConstraint {
            id: 1,
            kind: "mul".to_string(),
            a: CbirLinearCombination {
                terms: vec![CbirTerm {
                    wire_id,
                    coeff_hex: "0x01".to_string(),
                }],
            },
            b: CbirLinearCombination {
                terms: vec![
                    CbirTerm {
                        wire_id,
                        coeff_hex: "0x01".to_string(),
                    },
                    CbirTerm {
                        wire_id: 0,
                        coeff_hex: "0xffffffff".to_string(),
                    },
                ],
            },
            c: CbirLinearCombination { terms: vec![] },
            signal_hints: vec!["main.bool[0]".to_string()],
        }
    }

    fn non_bool_mul() -> CbirConstraint {
        CbirConstraint {
            id: 2,
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
            signal_hints: vec![],
        }
    }

    #[test]
    fn boolean_detector_matches() {
        let d = BooleanConstraintDetector;
        assert!(d.detect(&bool_constraint(5)).is_some());
        assert_eq!(d.detect(&bool_constraint(5)), Some(Confidence::Exact));
    }

    #[test]
    fn boolean_detector_rejects_non_boolean() {
        let d = BooleanConstraintDetector;
        assert!(d.detect(&non_bool_mul()).is_none());
    }
}
