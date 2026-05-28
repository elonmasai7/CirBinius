use std::collections::BTreeMap;

use cirbinius_cbir::{CbirConstraint, CbirDocument, CbirLinearCombination};
use serde::{Deserialize, Serialize};

pub const BINIUS64_LOWERING_SCHEMA_VERSION: &str = "binius64-lowering/v1";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LoweredGate {
    pub constraint_id: u64,
    pub gate_kind: String,
    pub signal_hints: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Binius64LoweringArtifact {
    pub schema_version: String,
    pub toolchain_version: String,
    pub source_cbir_hash: String,
    pub gate_counts: BTreeMap<String, usize>,
    pub gates: Vec<LoweredGate>,
}

pub fn lower_to_binius64(document: &CbirDocument) -> Binius64LoweringArtifact {
    let mut gate_counts = BTreeMap::new();
    let mut gates = Vec::with_capacity(document.constraints.len());

    for constraint in &document.constraints {
        let gate_kind = classify_gate(constraint);
        *gate_counts.entry(gate_kind.clone()).or_insert(0) += 1;
        gates.push(LoweredGate {
            constraint_id: constraint.id,
            gate_kind,
            signal_hints: constraint.signal_hints.clone(),
        });
    }

    Binius64LoweringArtifact {
        schema_version: BINIUS64_LOWERING_SCHEMA_VERSION.to_string(),
        toolchain_version: env!("CARGO_PKG_VERSION").to_string(),
        source_cbir_hash: document.metadata.content_hash.clone(),
        gate_counts,
        gates,
    }
}

fn classify_gate(constraint: &CbirConstraint) -> String {
    if has_hint_token(
        constraint,
        &["poseidon", "sha", "hash", "keccak", "mimc", "pedersen"],
    ) {
        return "hash".to_string();
    }
    if has_hint_token(constraint, &["merkle", "path", "sibling", "root"]) {
        return "merkle_path".to_string();
    }
    if has_hint_token(
        constraint,
        &["mux", "selector", "select", "ifelse", "ternary"],
    ) {
        return "mux_selector".to_string();
    }
    if has_hint_token(constraint, &["xor"]) {
        return "xor".to_string();
    }
    if has_hint_token(constraint, &["and"]) || matches_structural_and_gate(constraint) {
        return "and".to_string();
    }
    if has_hint_token(
        constraint,
        &["range", "lt", "leq", "less", "greater", "cmp", "compare"],
    ) {
        return "range_check".to_string();
    }
    if has_hint_token(constraint, &["bit", "bool"]) {
        return "boolean".to_string();
    }

    if constraint.kind == "mul" {
        "mul".to_string()
    } else {
        "generic".to_string()
    }
}

fn has_hint_token(constraint: &CbirConstraint, tokens: &[&str]) -> bool {
    constraint.signal_hints.iter().any(|hint| {
        let lowered = hint.to_lowercase();
        tokens.iter().any(|token| lowered.contains(token))
    })
}

fn matches_structural_and_gate(constraint: &CbirConstraint) -> bool {
    constraint.kind == "mul"
        && has_hint_token(constraint, &["and", "bool", "bit"])
        && is_single_unit_term(&constraint.a)
        && is_single_unit_term(&constraint.b)
        && is_single_unit_term(&constraint.c)
}

fn is_single_unit_term(linear: &CbirLinearCombination) -> bool {
    linear.terms.len() == 1
        && linear.terms[0]
            .coeff_hex
            .trim_start_matches("0x")
            .trim_start_matches('0')
            == "1"
}

#[cfg(test)]
mod tests {
    use super::{BINIUS64_LOWERING_SCHEMA_VERSION, lower_to_binius64};
    use cirbinius_cbir::{
        CbirConstraint, CbirDocument, CbirLinearCombination, CbirMetadata, CbirSignal, CbirTerm,
    };
    use cirbinius_types::Backend;

    #[test]
    fn lowering_classifies_extended_gate_families() {
        let document = CbirDocument {
            metadata: CbirMetadata {
                schema_version: "cbir/v1".to_string(),
                toolchain_version: "0.1.0".to_string(),
                content_hash: "sha256:abc".to_string(),
            },
            backend: Backend::Binius64,
            field_modulus_hex: "0x07".to_string(),
            wire_count: 10,
            public_output_count: 0,
            public_input_count: 0,
            private_input_count: 1,
            constraints: vec![
                CbirConstraint {
                    id: 1,
                    kind: "mul".to_string(),
                    a: CbirLinearCombination {
                        terms: vec![CbirTerm {
                            wire_id: 1,
                            coeff_hex: "0x01".to_string(),
                        }],
                    },
                    b: CbirLinearCombination { terms: vec![] },
                    c: CbirLinearCombination { terms: vec![] },
                    signal_hints: vec!["main.bool[0]".to_string()],
                },
                CbirConstraint {
                    id: 2,
                    kind: "mul".to_string(),
                    a: CbirLinearCombination { terms: vec![] },
                    b: CbirLinearCombination { terms: vec![] },
                    c: CbirLinearCombination { terms: vec![] },
                    signal_hints: vec!["main.rangeCheck".to_string()],
                },
                CbirConstraint {
                    id: 3,
                    kind: "mul".to_string(),
                    a: CbirLinearCombination { terms: vec![] },
                    b: CbirLinearCombination { terms: vec![] },
                    c: CbirLinearCombination { terms: vec![] },
                    signal_hints: vec!["main.xor_chain".to_string()],
                },
                CbirConstraint {
                    id: 4,
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
                },
                CbirConstraint {
                    id: 5,
                    kind: "mul".to_string(),
                    a: CbirLinearCombination { terms: vec![] },
                    b: CbirLinearCombination { terms: vec![] },
                    c: CbirLinearCombination { terms: vec![] },
                    signal_hints: vec!["main.mux.select".to_string()],
                },
                CbirConstraint {
                    id: 6,
                    kind: "mul".to_string(),
                    a: CbirLinearCombination { terms: vec![] },
                    b: CbirLinearCombination { terms: vec![] },
                    c: CbirLinearCombination { terms: vec![] },
                    signal_hints: vec!["main.merkle.path".to_string()],
                },
                CbirConstraint {
                    id: 7,
                    kind: "mul".to_string(),
                    a: CbirLinearCombination { terms: vec![] },
                    b: CbirLinearCombination { terms: vec![] },
                    c: CbirLinearCombination { terms: vec![] },
                    signal_hints: vec!["main.poseidon.hash".to_string()],
                },
            ],
            signals: vec![CbirSignal {
                wire_id: 1,
                name: "main.bool[0]".to_string(),
            }],
        };

        let lowered = lower_to_binius64(&document);
        assert_eq!(lowered.schema_version, BINIUS64_LOWERING_SCHEMA_VERSION);
        assert_eq!(lowered.gate_counts.get("boolean"), Some(&1));
        assert_eq!(lowered.gate_counts.get("range_check"), Some(&1));
        assert_eq!(lowered.gate_counts.get("xor"), Some(&1));
        assert_eq!(lowered.gate_counts.get("and"), Some(&1));
        assert_eq!(lowered.gate_counts.get("mux_selector"), Some(&1));
        assert_eq!(lowered.gate_counts.get("merkle_path"), Some(&1));
        assert_eq!(lowered.gate_counts.get("hash"), Some(&1));
    }
}
