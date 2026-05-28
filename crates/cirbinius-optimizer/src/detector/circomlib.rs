use cirbinius_cbir::CbirConstraint;

use super::{Confidence, PatternDetector, has_hint_token};

pub struct CircomlibGadgetDetector;

impl PatternDetector for CircomlibGadgetDetector {
    fn name(&self) -> &'static str {
        "circomlib-gadget"
    }
    fn description(&self) -> &'static str {
        "Detects known circomlib gadgets by structural signature"
    }
    fn default_confidence(&self) -> Confidence {
        Confidence::Strong
    }

    fn detect(&self, constraint: &CbirConstraint) -> Option<Confidence> {
        // Check known gadget signatures
        for (_name, tokens, confidence) in GADGET_SIGNATURES.iter() {
            if has_hint_token(constraint, tokens) {
                return Some(*confidence);
            }
        }
        None
    }

    fn detect_group(&self, constraints: &[CbirConstraint]) -> Vec<Vec<u64>> {
        let mut groups = Vec::new();
        for (_name, tokens, _confidence) in GADGET_SIGNATURES.iter() {
            let ids: Vec<u64> = constraints
                .iter()
                .filter(|c| has_hint_token(c, tokens))
                .map(|c| c.id)
                .collect();
            if !ids.is_empty() {
                groups.push(ids);
            }
        }
        groups
    }
}

static GADGET_SIGNATURES: &[(&str, &[&str], Confidence)] = &[
    ("alias-check", &["alias", "aliascheck"], Confidence::Strong),
    (
        "baby-add",
        &["babyadd", "baby_add", "babyjub"],
        Confidence::Heuristic,
    ),
    (
        "baby-dbl",
        &["babydbl", "baby_dbl", "pointdouble"],
        Confidence::Heuristic,
    ),
    (
        "eddsa",
        &["eddsa", "eddsa_verify", "eddsa_mimc"],
        Confidence::Heuristic,
    ),
    ("poseidon-hash", &["poseidon"], Confidence::Strong),
    ("mimc-hash", &["mimc"], Confidence::Strong),
    (
        "pedersen-hash",
        &["pedersen", "pedersenhash"],
        Confidence::Heuristic,
    ),
    (
        "smt-verifier",
        &["smt", "sparsemerkletree"],
        Confidence::Heuristic,
    ),
    (
        "comparator",
        &["comparator", "lesseq", "greaterthan"],
        Confidence::Heuristic,
    ),
    (
        "bitify",
        &["bitify", "num2bits", "bits2num"],
        Confidence::Strong,
    ),
    ("multiplexer", &["mux", "multiplexer"], Confidence::Strong),
    ("sigma", &["sigma", "sha256sigma"], Confidence::Strong),
    ("feistel", &["feistel", "mimcfeistel"], Confidence::Strong),
];

#[cfg(test)]
mod tests {
    use super::*;
    use cirbinius_cbir::{CbirConstraint, CbirLinearCombination, CbirTerm};

    fn hint_constraint(hint: &str) -> CbirConstraint {
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
            signal_hints: vec![hint.to_string()],
        }
    }

    #[test]
    fn detects_poseidon() {
        let d = CircomlibGadgetDetector;
        assert!(d.detect(&hint_constraint("poseidon")).is_some());
    }

    #[test]
    fn detects_mimc() {
        let d = CircomlibGadgetDetector;
        assert!(d.detect(&hint_constraint("mimc")).is_some());
    }

    #[test]
    fn detects_eddsa() {
        let d = CircomlibGadgetDetector;
        assert!(d.detect(&hint_constraint("eddsa_verify")).is_some());
    }
}
