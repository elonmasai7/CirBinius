use anyhow::{Result, ensure};
use cirbinius_normalize::{NormalizedCircuit, NormalizedConstraint, NormalizedLinearCombination};
use cirbinius_types::{Backend, CompilerOptions};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const CBIR_SCHEMA_VERSION: &str = "cbir/v1";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CbirMetadata {
    pub schema_version: String,
    pub toolchain_version: String,
    pub content_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CbirTerm {
    pub wire_id: u32,
    pub coeff_hex: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CbirLinearCombination {
    pub terms: Vec<CbirTerm>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CbirConstraint {
    pub id: u64,
    pub kind: String,
    pub a: CbirLinearCombination,
    pub b: CbirLinearCombination,
    pub c: CbirLinearCombination,
    pub signal_hints: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CbirSignal {
    pub wire_id: u32,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CbirDocument {
    pub metadata: CbirMetadata,
    pub backend: Backend,
    pub field_modulus_hex: String,
    pub wire_count: u32,
    pub public_output_count: u32,
    pub public_input_count: u32,
    pub private_input_count: u32,
    pub constraints: Vec<CbirConstraint>,
    pub signals: Vec<CbirSignal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CbirHashStableView {
    pub schema_version: String,
    pub toolchain_version: String,
    pub backend: Backend,
    pub field_modulus_hex: String,
    pub wire_count: u32,
    pub public_output_count: u32,
    pub public_input_count: u32,
    pub private_input_count: u32,
    pub constraints: Vec<CbirConstraint>,
    pub signals: Vec<CbirSignal>,
}

impl CbirDocument {
    pub fn from_normalized(
        normalized: &NormalizedCircuit,
        options: &CompilerOptions,
    ) -> Result<Self> {
        let constraints = normalized
            .constraints
            .iter()
            .map(convert_constraint)
            .collect::<Vec<_>>();

        let signals = normalized
            .signals
            .iter()
            .map(|signal| CbirSignal {
                wire_id: signal.wire_id,
                name: signal.name.clone(),
            })
            .collect::<Vec<_>>();

        let mut document = Self {
            metadata: CbirMetadata {
                schema_version: CBIR_SCHEMA_VERSION.to_string(),
                toolchain_version: env!("CARGO_PKG_VERSION").to_string(),
                content_hash: String::new(),
            },
            backend: options.backend,
            field_modulus_hex: normalized.field_modulus_hex.clone(),
            wire_count: normalized.wire_count,
            public_output_count: normalized.public_output_count,
            public_input_count: normalized.public_input_count,
            private_input_count: normalized.private_input_count,
            constraints,
            signals,
        };
        document.seal_hash()?;
        Ok(document)
    }

    pub fn seal_hash(&mut self) -> Result<()> {
        let view = self.hash_stable_view();
        let payload = serde_json::to_vec(&view)?;
        let digest = Sha256::digest(payload);
        self.metadata.content_hash = format!("sha256:{digest:x}");
        Ok(())
    }

    pub fn validate(&self) -> Result<()> {
        ensure!(
            self.metadata.schema_version == CBIR_SCHEMA_VERSION,
            "invalid CBIR schema version: {}",
            self.metadata.schema_version
        );
        ensure!(
            self.metadata.content_hash.starts_with("sha256:"),
            "invalid CBIR content hash prefix"
        );

        let expected_hash = {
            let payload = serde_json::to_vec(&self.hash_stable_view())?;
            format!("sha256:{:x}", Sha256::digest(payload))
        };
        ensure!(
            self.metadata.content_hash == expected_hash,
            "CBIR content hash mismatch"
        );
        Ok(())
    }

    pub fn to_pretty_json(&self) -> Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    fn hash_stable_view(&self) -> CbirHashStableView {
        CbirHashStableView {
            schema_version: self.metadata.schema_version.clone(),
            toolchain_version: self.metadata.toolchain_version.clone(),
            backend: self.backend,
            field_modulus_hex: self.field_modulus_hex.clone(),
            wire_count: self.wire_count,
            public_output_count: self.public_output_count,
            public_input_count: self.public_input_count,
            private_input_count: self.private_input_count,
            constraints: self.constraints.clone(),
            signals: self.signals.clone(),
        }
    }
}

fn convert_constraint(constraint: &NormalizedConstraint) -> CbirConstraint {
    CbirConstraint {
        id: constraint.id,
        kind: "mul".to_string(),
        a: convert_linear_combination(&constraint.a),
        b: convert_linear_combination(&constraint.b),
        c: convert_linear_combination(&constraint.c),
        signal_hints: constraint.signal_hints.clone(),
    }
}

fn convert_linear_combination(linear: &NormalizedLinearCombination) -> CbirLinearCombination {
    CbirLinearCombination {
        terms: linear
            .terms
            .iter()
            .map(|term| CbirTerm {
                wire_id: term.wire_id,
                coeff_hex: term.coeff_hex.clone(),
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::CbirDocument;
    use cirbinius_normalize::{
        NormalizedCircuit, NormalizedConstraint, NormalizedLinearCombination, NormalizedSignal,
        NormalizedTerm,
    };
    use cirbinius_types::CompilerOptions;

    #[test]
    fn hash_is_deterministic_and_validates() {
        let normalized = NormalizedCircuit {
            field_modulus_hex: "0x07".to_string(),
            wire_count: 3,
            public_output_count: 0,
            public_input_count: 1,
            private_input_count: 1,
            constraints: vec![NormalizedConstraint {
                id: 1,
                a: NormalizedLinearCombination {
                    terms: vec![NormalizedTerm {
                        wire_id: 1,
                        coeff_hex: "0x01".to_string(),
                    }],
                },
                b: NormalizedLinearCombination {
                    terms: vec![NormalizedTerm {
                        wire_id: 2,
                        coeff_hex: "0x01".to_string(),
                    }],
                },
                c: NormalizedLinearCombination {
                    terms: vec![NormalizedTerm {
                        wire_id: 3,
                        coeff_hex: "0x01".to_string(),
                    }],
                },
                signal_hints: vec![
                    "main.a".to_string(),
                    "main.b".to_string(),
                    "main.c".to_string(),
                ],
            }],
            signals: vec![NormalizedSignal {
                wire_id: 1,
                name: "main.a".to_string(),
            }],
        };

        let first = CbirDocument::from_normalized(&normalized, &CompilerOptions::default())
            .expect("first CBIR build should work");
        let second = CbirDocument::from_normalized(&normalized, &CompilerOptions::default())
            .expect("second CBIR build should work");
        assert_eq!(first.metadata.content_hash, second.metadata.content_hash);
        first.validate().expect("CBIR should validate");
    }
}
