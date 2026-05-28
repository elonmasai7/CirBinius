pub mod detector;
pub mod pass;

use std::collections::BTreeMap;

use cirbinius_cbir::{CbirConstraint, CbirDocument, CbirLinearCombination};
use cirbinius_types::CompileMode;
use serde::{Deserialize, Serialize};

// -- Backward-compat exports from old optimizer --
pub use detector::{Confidence, PatternDetector, registry::DetectorRegistry};

const RULE_BOOLEAN: &str = "boolean_constraints";
const RULE_BIT_DECOMP: &str = "bit_decomposition_candidates";
const RULE_RANGE: &str = "range_check_candidates";
const RULE_XOR: &str = "xor_chain_candidates";
const RULE_AND: &str = "and_chain_candidates";
const RULE_MUX: &str = "mux_selector_candidates";
const RULE_MERKLE: &str = "merkle_path_candidates";
const RULE_HASH: &str = "hash_motif_candidates";
pub const LOWERING_RULES_INDEX_SCHEMA_VERSION: &str = "lowering-rules-index/v1";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OptimizationSummary {
    pub mode: CompileMode,
    pub total_constraints: usize,
    pub pass_counts: BTreeMap<String, usize>,
    pub detected_constraints: BTreeMap<String, Vec<u64>>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LoweringRuleEntry {
    pub rule: String,
    pub count: usize,
    pub constraint_ids: Vec<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LoweringRulesIndex {
    pub schema_version: String,
    pub mode: CompileMode,
    pub total_constraints: usize,
    pub rules: Vec<LoweringRuleEntry>,
}

pub fn analyze(document: &CbirDocument, mode: CompileMode) -> OptimizationSummary {
    let mut detected_constraints: BTreeMap<String, Vec<u64>> = BTreeMap::new();
    detected_constraints.insert(RULE_BOOLEAN.to_string(), Vec::new());
    detected_constraints.insert(RULE_BIT_DECOMP.to_string(), Vec::new());
    detected_constraints.insert(RULE_RANGE.to_string(), Vec::new());
    detected_constraints.insert(RULE_XOR.to_string(), Vec::new());
    detected_constraints.insert(RULE_AND.to_string(), Vec::new());
    detected_constraints.insert(RULE_MUX.to_string(), Vec::new());
    detected_constraints.insert(RULE_MERKLE.to_string(), Vec::new());
    detected_constraints.insert(RULE_HASH.to_string(), Vec::new());

    for constraint in &document.constraints {
        if matches_boolean_constraint(constraint) {
            push_detection(&mut detected_constraints, RULE_BOOLEAN, constraint.id);
        }
        if matches_bit_decomposition_candidate(constraint) {
            push_detection(&mut detected_constraints, RULE_BIT_DECOMP, constraint.id);
        }
        if matches_range_check_candidate(constraint) {
            push_detection(&mut detected_constraints, RULE_RANGE, constraint.id);
        }
        if matches_xor_chain_candidate(constraint) {
            push_detection(&mut detected_constraints, RULE_XOR, constraint.id);
        }
        if matches_and_chain_candidate(constraint) {
            push_detection(&mut detected_constraints, RULE_AND, constraint.id);
        }
        if matches_mux_selector_candidate(constraint) {
            push_detection(&mut detected_constraints, RULE_MUX, constraint.id);
        }
        if matches_merkle_path_candidate(constraint) {
            push_detection(&mut detected_constraints, RULE_MERKLE, constraint.id);
        }
        if matches_hash_motif_candidate(constraint) {
            push_detection(&mut detected_constraints, RULE_HASH, constraint.id);
        }
    }

    let total_constraints = document.constraints.len();
    let mut pass_counts = BTreeMap::new();
    pass_counts.insert("total_constraints".to_string(), total_constraints);
    for (rule, ids) in &detected_constraints {
        pass_counts.insert(rule.clone(), ids.len());
    }

    let mut notes = Vec::new();
    if mode == CompileMode::OptimizedBinary {
        notes.push("Optimized binary mode enabled: pattern-aware lowering candidates detected for boolean, bitwise, range, mux, Merkle, and hash motifs.".to_string());
    } else {
        notes.push("Compatibility mode enabled: optimization analysis only, no semantic shortcuts applied.".to_string());
    }

    OptimizationSummary {
        mode,
        total_constraints,
        pass_counts,
        detected_constraints,
        notes,
    }
}

pub fn optimize(document: &CbirDocument, mode: CompileMode) -> (CbirDocument, OptimizationSummary) {
    let summary = analyze(document, mode);
    (document.clone(), summary)
}

pub fn build_lowering_rules_index(summary: &OptimizationSummary) -> LoweringRulesIndex {
    let mut rules = summary
        .detected_constraints
        .iter()
        .map(|(rule, ids)| LoweringRuleEntry {
            rule: rule.clone(),
            count: ids.len(),
            constraint_ids: ids.clone(),
        })
        .collect::<Vec<_>>();
    rules.sort_by(|a, b| a.rule.cmp(&b.rule));

    LoweringRulesIndex {
        schema_version: LOWERING_RULES_INDEX_SCHEMA_VERSION.to_string(),
        mode: summary.mode,
        total_constraints: summary.total_constraints,
        rules,
    }
}

fn push_detection(map: &mut BTreeMap<String, Vec<u64>>, rule: &str, constraint_id: u64) {
    if let Some(ids) = map.get_mut(rule) {
        ids.push(constraint_id);
    }
}

fn matches_boolean_constraint(constraint: &CbirConstraint) -> bool {
    is_wire_term_pattern(&constraint.a, 1)
        && is_boolean_rhs(&constraint.b, &constraint.a.terms)
        && is_zero_linear(&constraint.c)
}

fn matches_bit_decomposition_candidate(constraint: &CbirConstraint) -> bool {
    has_hint_token(constraint, &["bit", "bits", "num2bits"])
}

fn matches_range_check_candidate(constraint: &CbirConstraint) -> bool {
    has_hint_token(
        constraint,
        &["range", "lt", "leq", "less", "greater", "cmp", "compare"],
    )
}

fn matches_xor_chain_candidate(constraint: &CbirConstraint) -> bool {
    has_hint_token(constraint, &["xor"])
}

fn matches_and_chain_candidate(constraint: &CbirConstraint) -> bool {
    has_hint_token(constraint, &["and"]) || matches_structural_and_gate(constraint)
}

fn matches_mux_selector_candidate(constraint: &CbirConstraint) -> bool {
    has_hint_token(
        constraint,
        &["mux", "selector", "select", "ifelse", "ternary"],
    )
}

fn matches_merkle_path_candidate(constraint: &CbirConstraint) -> bool {
    has_hint_token(constraint, &["merkle", "path", "sibling", "root", "branch"])
}

fn matches_hash_motif_candidate(constraint: &CbirConstraint) -> bool {
    has_hint_token(
        constraint,
        &[
            "hash", "poseidon", "sha", "keccak", "mimc", "pedersen", "blake",
        ],
    )
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
        && is_wire_term_pattern(&constraint.a, 1)
        && is_wire_term_pattern(&constraint.b, 1)
        && is_wire_term_pattern(&constraint.c, 1)
}

fn is_wire_term_pattern(linear: &CbirLinearCombination, expected_terms: usize) -> bool {
    linear.terms.len() == expected_terms && is_one(&linear.terms[0].coeff_hex)
}

fn is_boolean_rhs(rhs: &CbirLinearCombination, lhs_terms: &[cirbinius_cbir::CbirTerm]) -> bool {
    if rhs.terms.len() != 2 || lhs_terms.len() != 1 {
        return false;
    }

    let lhs_wire = lhs_terms[0].wire_id;
    let mut has_neg = false;
    let mut has_pos = false;
    for term in &rhs.terms {
        if term.wire_id == lhs_wire && is_one(&term.coeff_hex) {
            has_pos = true;
        }
        if term.wire_id == 0 && is_negative_one(&term.coeff_hex) {
            has_neg = true;
        }
    }
    has_neg && has_pos
}

fn is_zero_linear(linear: &CbirLinearCombination) -> bool {
    linear.terms.is_empty() || linear.terms.iter().all(|term| term.coeff_hex == "0x0")
}

fn is_negative_one(coeff_hex: &str) -> bool {
    let body = coeff_hex.trim_start_matches("0x");
    !body.is_empty() && body.chars().all(|ch| ch == 'f')
}

fn is_one(coeff_hex: &str) -> bool {
    coeff_hex.trim_start_matches("0x").trim_start_matches('0') == "1"
}

#[cfg(test)]
mod tests {
    use super::{
        LOWERING_RULES_INDEX_SCHEMA_VERSION, RULE_AND, RULE_HASH, RULE_MERKLE, RULE_MUX,
        RULE_RANGE, RULE_XOR, analyze, build_lowering_rules_index, optimize,
    };
    use cirbinius_cbir::{
        CbirConstraint, CbirDocument, CbirLinearCombination, CbirMetadata, CbirSignal, CbirTerm,
    };
    use cirbinius_types::{Backend, CompileMode};

    fn base_doc(constraints: Vec<CbirConstraint>) -> CbirDocument {
        CbirDocument {
            metadata: CbirMetadata {
                schema_version: "cbir/v1".to_string(),
                toolchain_version: "0.1.0".to_string(),
                content_hash: "sha256:abc".to_string(),
            },
            backend: Backend::Binius64,
            field_modulus_hex: "0x07".to_string(),
            wire_count: 8,
            public_output_count: 0,
            public_input_count: 0,
            private_input_count: 2,
            constraints,
            signals: vec![CbirSignal {
                wire_id: 1,
                name: "main.bits[0]".to_string(),
            }],
        }
    }

    fn make_mul_constraint(id: u64, hint: &str) -> CbirConstraint {
        CbirConstraint {
            id,
            kind: "mul".to_string(),
            a: CbirLinearCombination {
                terms: vec![CbirTerm {
                    wire_id: 1,
                    coeff_hex: "0x01".to_string(),
                }],
            },
            b: CbirLinearCombination {
                terms: vec![
                    CbirTerm {
                        wire_id: 1,
                        coeff_hex: "0x01".to_string(),
                    },
                    CbirTerm {
                        wire_id: 0,
                        coeff_hex: "0xffff".to_string(),
                    },
                ],
            },
            c: CbirLinearCombination { terms: vec![] },
            signal_hints: vec![hint.to_string()],
        }
    }

    fn make_structural_and_constraint(id: u64, hint: &str) -> CbirConstraint {
        CbirConstraint {
            id,
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
    fn analyze_detects_boolean_and_bit_candidates() {
        let summary = analyze(
            &base_doc(vec![make_mul_constraint(1, "main.bits[0]")]),
            CompileMode::OptimizedBinary,
        );
        assert_eq!(summary.total_constraints, 1);
        assert_eq!(summary.pass_counts.get("boolean_constraints"), Some(&1));
        assert_eq!(
            summary.pass_counts.get("bit_decomposition_candidates"),
            Some(&1)
        );
    }

    #[test]
    fn analyze_detects_range_rule() {
        let summary = analyze(
            &base_doc(vec![make_mul_constraint(1, "main.rangeCheck")]),
            CompileMode::OptimizedBinary,
        );
        assert_eq!(summary.pass_counts.get(RULE_RANGE), Some(&1));
    }

    #[test]
    fn analyze_detects_xor_rule() {
        let summary = analyze(
            &base_doc(vec![make_mul_constraint(1, "main.xor_chain")]),
            CompileMode::OptimizedBinary,
        );
        assert_eq!(summary.pass_counts.get(RULE_XOR), Some(&1));
    }

    #[test]
    fn analyze_detects_and_rule_from_structure() {
        let summary = analyze(
            &base_doc(vec![make_structural_and_constraint(1, "main.boolAnd")]),
            CompileMode::OptimizedBinary,
        );
        assert_eq!(summary.pass_counts.get(RULE_AND), Some(&1));
    }

    #[test]
    fn analyze_detects_mux_rule() {
        let summary = analyze(
            &base_doc(vec![make_mul_constraint(1, "main.mux.select")]),
            CompileMode::OptimizedBinary,
        );
        assert_eq!(summary.pass_counts.get(RULE_MUX), Some(&1));
    }

    #[test]
    fn analyze_detects_merkle_rule() {
        let summary = analyze(
            &base_doc(vec![make_mul_constraint(1, "main.merkle.path")]),
            CompileMode::OptimizedBinary,
        );
        assert_eq!(summary.pass_counts.get(RULE_MERKLE), Some(&1));
    }

    #[test]
    fn analyze_detects_hash_rule() {
        let summary = analyze(
            &base_doc(vec![make_mul_constraint(1, "main.poseidon.hash")]),
            CompileMode::OptimizedBinary,
        );
        assert_eq!(summary.pass_counts.get(RULE_HASH), Some(&1));
    }

    #[test]
    fn optimize_preserves_document_in_compatibility_mode() {
        let doc = base_doc(vec![make_mul_constraint(1, "main.bits[0]")]);
        let (optimized, summary) = optimize(&doc, CompileMode::Compatibility);
        assert_eq!(doc.constraints.len(), optimized.constraints.len());
        assert_eq!(summary.total_constraints, 1);
    }

    #[test]
    fn builds_machine_readable_lowering_rules_index() {
        let summary = analyze(
            &base_doc(vec![make_mul_constraint(1, "main.rangeCheck")]),
            CompileMode::OptimizedBinary,
        );
        let index = build_lowering_rules_index(&summary);

        assert_eq!(index.schema_version, LOWERING_RULES_INDEX_SCHEMA_VERSION);
        assert_eq!(index.total_constraints, 1);

        let range_entry = index
            .rules
            .iter()
            .find(|entry| entry.rule == RULE_RANGE)
            .expect("range rule should be present in index");
        assert_eq!(range_entry.count, 1);
        assert_eq!(range_entry.constraint_ids, vec![1]);
    }
}
