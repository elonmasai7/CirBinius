use std::collections::BTreeMap;

use cirbinius_cbir::{CbirConstraint, CbirDocument, CbirLinearCombination};
use cirbinius_limb_engine::LimbWidth;
use serde::{Deserialize, Serialize};

pub const BINIUS64_LOWERING_SCHEMA_VERSION: &str = "binius64-lowering/v1";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LoweredGate {
    pub constraint_id: u64,
    pub gate_kind: String,
    pub signal_hints: Vec<String>,
    pub limb_width: Option<String>,
    pub passes_applied: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct LoweredDocument {
    pub constraints: Vec<LoweredGate>,
    pub gate_counts: BTreeMap<String, usize>,
}

pub trait LoweringPass: Send + Sync {
    fn name(&self) -> &'static str;
    fn can_handle(&self, constraint: &CbirConstraint) -> bool;
    fn lower(&self, constraint: &CbirConstraint) -> LoweredGate;
}

pub struct LoweringPipeline {
    passes: Vec<Box<dyn LoweringPass>>,
    #[allow(dead_code)]
    limb_width: LimbWidth,
}

impl LoweringPipeline {
    pub fn new(limb_width: LimbWidth) -> Self {
        let mut passes: Vec<Box<dyn LoweringPass>> = vec![
            Box::new(HashPass),
            Box::new(MerklePathPass),
            Box::new(MuxSelectorPass),
            Box::new(XorPass),
            Box::new(AndPass),
            Box::new(RangeCheckPass),
            Box::new(BooleanPass),
            Box::new(MulPass),
        ];
        passes.push(Box::new(GenericPass));
        Self { passes, limb_width }
    }

    pub fn lower(&self, constraint: &CbirConstraint) -> LoweredGate {
        for pass in &self.passes {
            if pass.can_handle(constraint) {
                let mut gate = pass.lower(constraint);
                gate.passes_applied.push(pass.name().to_string());
                return gate;
            }
        }
        GenericPass.lower(constraint)
    }

    pub fn lower_document(&self, document: &CbirDocument) -> Vec<LoweredGate> {
        let mut gates = Vec::with_capacity(document.constraints.len());
        for constraint in &document.constraints {
            gates.push(self.lower(constraint));
        }
        gates
    }
}

pub fn lower_to_binius64(document: &CbirDocument) -> Binius64LoweringArtifact {
    lower_to_binius64_with_width(document, LimbWidth::Auto)
}

pub fn lower_to_binius64_with_width(
    document: &CbirDocument,
    limb_width: LimbWidth,
) -> Binius64LoweringArtifact {
    let pipeline = LoweringPipeline::new(limb_width);
    let mut gate_counts = BTreeMap::new();
    let mut gates = Vec::with_capacity(document.constraints.len());

    for constraint in &document.constraints {
        let gate = pipeline.lower(constraint);
        *gate_counts.entry(gate.gate_kind.clone()).or_insert(0) += 1;
        gates.push(gate);
    }

    Binius64LoweringArtifact {
        schema_version: BINIUS64_LOWERING_SCHEMA_VERSION.to_string(),
        toolchain_version: env!("CARGO_PKG_VERSION").to_string(),
        source_cbir_hash: document.metadata.content_hash.clone(),
        gate_counts,
        gates,
        limb_width: limb_width.name().to_string(),
    }
}

fn has_hint_token(constraint: &CbirConstraint, tokens: &[&str]) -> bool {
    constraint.signal_hints.iter().any(|hint| {
        let lowered = hint.to_lowercase();
        tokens.iter().any(|token| lowered.contains(token))
    })
}

fn is_single_unit_term(linear: &CbirLinearCombination) -> bool {
    linear.terms.len() == 1
        && linear.terms[0]
            .coeff_hex
            .trim_start_matches("0x")
            .trim_start_matches('0')
            == "1"
}

fn matches_structural_and_gate(constraint: &CbirConstraint) -> bool {
    constraint.kind == "mul"
        && has_hint_token(constraint, &["and", "bool", "bit"])
        && is_single_unit_term(&constraint.a)
        && is_single_unit_term(&constraint.b)
        && is_single_unit_term(&constraint.c)
}

fn constraint_base_gate(constraint: &CbirConstraint, kind: &str) -> LoweredGate {
    LoweredGate {
        constraint_id: constraint.id,
        gate_kind: kind.to_string(),
        signal_hints: constraint.signal_hints.clone(),
        limb_width: None,
        passes_applied: vec![],
    }
}

// ---------- Pass implementations ----------

pub struct HashPass;
impl LoweringPass for HashPass {
    fn name(&self) -> &'static str {
        "hash"
    }
    fn can_handle(&self, constraint: &CbirConstraint) -> bool {
        has_hint_token(
            constraint,
            &["poseidon", "sha", "hash", "keccak", "mimc", "pedersen"],
        )
    }
    fn lower(&self, constraint: &CbirConstraint) -> LoweredGate {
        let mut gate = constraint_base_gate(constraint, "hash");
        gate.limb_width = Some("field".to_string());
        gate
    }
}

pub struct MerklePathPass;
impl LoweringPass for MerklePathPass {
    fn name(&self) -> &'static str {
        "merkle_path"
    }
    fn can_handle(&self, constraint: &CbirConstraint) -> bool {
        has_hint_token(constraint, &["merkle", "path", "sibling", "root"])
    }
    fn lower(&self, constraint: &CbirConstraint) -> LoweredGate {
        constraint_base_gate(constraint, "merkle_path")
    }
}

pub struct MuxSelectorPass;
impl LoweringPass for MuxSelectorPass {
    fn name(&self) -> &'static str {
        "mux_selector"
    }
    fn can_handle(&self, constraint: &CbirConstraint) -> bool {
        has_hint_token(
            constraint,
            &["mux", "selector", "select", "ifelse", "ternary"],
        )
    }
    fn lower(&self, constraint: &CbirConstraint) -> LoweredGate {
        constraint_base_gate(constraint, "mux_selector")
    }
}

pub struct XorPass;
impl LoweringPass for XorPass {
    fn name(&self) -> &'static str {
        "xor"
    }
    fn can_handle(&self, constraint: &CbirConstraint) -> bool {
        has_hint_token(constraint, &["xor"])
    }
    fn lower(&self, constraint: &CbirConstraint) -> LoweredGate {
        constraint_base_gate(constraint, "xor")
    }
}

pub struct AndPass;
impl LoweringPass for AndPass {
    fn name(&self) -> &'static str {
        "and"
    }
    fn can_handle(&self, constraint: &CbirConstraint) -> bool {
        has_hint_token(constraint, &["and"]) || matches_structural_and_gate(constraint)
    }
    fn lower(&self, constraint: &CbirConstraint) -> LoweredGate {
        constraint_base_gate(constraint, "and")
    }
}

pub struct RangeCheckPass;
impl LoweringPass for RangeCheckPass {
    fn name(&self) -> &'static str {
        "range_check"
    }
    fn can_handle(&self, constraint: &CbirConstraint) -> bool {
        has_hint_token(
            constraint,
            &["range", "lt", "leq", "less", "greater", "cmp", "compare"],
        )
    }
    fn lower(&self, constraint: &CbirConstraint) -> LoweredGate {
        constraint_base_gate(constraint, "range_check")
    }
}

pub struct BooleanPass;
impl LoweringPass for BooleanPass {
    fn name(&self) -> &'static str {
        "boolean"
    }
    fn can_handle(&self, constraint: &CbirConstraint) -> bool {
        has_hint_token(constraint, &["bit", "bool"])
    }
    fn lower(&self, constraint: &CbirConstraint) -> LoweredGate {
        constraint_base_gate(constraint, "boolean")
    }
}

pub struct MulPass;
impl LoweringPass for MulPass {
    fn name(&self) -> &'static str {
        "mul"
    }
    fn can_handle(&self, constraint: &CbirConstraint) -> bool {
        constraint.kind == "mul"
    }
    fn lower(&self, constraint: &CbirConstraint) -> LoweredGate {
        constraint_base_gate(constraint, "mul")
    }
}

pub struct GenericPass;
impl LoweringPass for GenericPass {
    fn name(&self) -> &'static str {
        "generic_compat"
    }
    fn can_handle(&self, _constraint: &CbirConstraint) -> bool {
        true
    }
    fn lower(&self, constraint: &CbirConstraint) -> LoweredGate {
        constraint_base_gate(constraint, "generic_compat")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Binius64LoweringArtifact {
    pub schema_version: String,
    pub toolchain_version: String,
    pub source_cbir_hash: String,
    pub gate_counts: BTreeMap<String, usize>,
    pub gates: Vec<LoweredGate>,
    pub limb_width: String,
}

#[cfg(test)]
mod tests {
    use super::*;
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

    #[test]
    fn lower_uses_limb_width() {
        let document = CbirDocument {
            metadata: CbirMetadata {
                schema_version: "cbir/v1".to_string(),
                toolchain_version: "0.1.0".to_string(),
                content_hash: "sha256:abc".to_string(),
            },
            backend: Backend::Binius64,
            field_modulus_hex: "0x07".to_string(),
            wire_count: 1,
            public_output_count: 0,
            public_input_count: 0,
            private_input_count: 0,
            constraints: vec![],
            signals: vec![],
        };
        let artifact = lower_to_binius64_with_width(&document, LimbWidth::U16);
        assert_eq!(artifact.limb_width, "u16");
    }

    #[test]
    fn pipeline_applies_passes_in_order() {
        let pipeline = LoweringPipeline::new(LimbWidth::U32);
        let constraint = CbirConstraint {
            id: 1,
            kind: "add".to_string(),
            a: CbirLinearCombination { terms: vec![] },
            b: CbirLinearCombination { terms: vec![] },
            c: CbirLinearCombination { terms: vec![] },
            signal_hints: vec!["main.sig".to_string()],
        };
        let gate = pipeline.lower(&constraint);
        assert_eq!(gate.gate_kind, "generic_compat");
        assert_eq!(gate.passes_applied, vec!["generic_compat"]);
    }

    #[test]
    fn hash_pass_sets_field_limb_width() {
        let constraint = CbirConstraint {
            id: 1,
            kind: "mul".to_string(),
            a: CbirLinearCombination { terms: vec![] },
            b: CbirLinearCombination { terms: vec![] },
            c: CbirLinearCombination { terms: vec![] },
            signal_hints: vec!["poseidon.round".to_string()],
        };
        let gate = HashPass.lower(&constraint);
        assert_eq!(gate.gate_kind, "hash");
        assert_eq!(gate.limb_width, Some("field".to_string()));
    }
}
