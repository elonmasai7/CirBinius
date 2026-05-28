use cirbinius_types::{Backend, CompileMode};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const CBIR_SCHEMA_VERSION: &str = "cbir/v1";
pub const PROVE_PRECHECK_BUNDLE_SCHEMA_VERSION: &str = "prove-precheck-bundle/v1";
pub const PROOF_BUNDLE_SCHEMA_VERSION: &str = "proof-bundle/v1";
pub const BACKEND_CAPABILITIES_SCHEMA_VERSION: &str = "backend-capabilities/v1";
pub const LOWERING_REPORT_SCHEMA_VERSION: &str = "lowering-report/v1";
pub const OPTIMIZATION_REPORT_SCHEMA_VERSION: &str = "optimization-report/v1";
pub const PROOF_ARTIFACT_SCHEMA_VERSION: &str = "proof-artifact/v1";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BuildMetadata {
    pub schema_version: String,
    pub toolchain_version: String,
    pub content_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CbirArtifact {
    pub metadata: BuildMetadata,
    pub field: String,
    pub backend: Backend,
    pub mode: CompileMode,
    pub constraint_count: u64,
}

impl CbirArtifact {
    pub fn placeholder() -> Self {
        Self {
            metadata: BuildMetadata {
                schema_version: CBIR_SCHEMA_VERSION.to_string(),
                toolchain_version: env!("CARGO_PKG_VERSION").to_string(),
                content_hash: "sha256:pending".to_string(),
            },
            field: "bn254".to_string(),
            backend: Backend::Binius64,
            mode: CompileMode::Compatibility,
            constraint_count: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProvePrecheckHashes {
    pub circuit_hash: String,
    pub witness_hash: String,
    pub wasm_hash: String,
    pub input_hash: String,
    pub binius_witness_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProvePrecheckReportSummary {
    pub precheck_passed: bool,
    pub generated_witness_path: String,
    pub generated_witness_len: u32,
    pub r1cs_wire_count: u32,
    pub witness_equivalent: Option<bool>,
    pub value_mismatch_count: usize,
    pub constraint_failure_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProvePrecheckBundle {
    pub schema_version: String,
    pub toolchain_version: String,
    pub bundle_hash: String,
    pub hashes: ProvePrecheckHashes,
    pub report: ProvePrecheckReportSummary,
}

#[derive(Debug, Clone, Serialize)]
struct HashStablePrecheckBundleView<'a> {
    schema_version: &'a str,
    toolchain_version: &'a str,
    hashes: &'a ProvePrecheckHashes,
    report: &'a ProvePrecheckReportSummary,
}

impl ProvePrecheckBundle {
    pub fn new(hashes: ProvePrecheckHashes, report: ProvePrecheckReportSummary) -> Self {
        let mut bundle = Self {
            schema_version: PROVE_PRECHECK_BUNDLE_SCHEMA_VERSION.to_string(),
            toolchain_version: env!("CARGO_PKG_VERSION").to_string(),
            bundle_hash: String::new(),
            hashes,
            report,
        };
        bundle.seal_hash();
        bundle
    }

    pub fn seal_hash(&mut self) {
        let payload = serde_json::to_vec(&HashStablePrecheckBundleView {
            schema_version: &self.schema_version,
            toolchain_version: &self.toolchain_version,
            hashes: &self.hashes,
            report: &self.report,
        })
        .unwrap_or_default();
        self.bundle_hash = sha256_prefixed(&payload);
    }

    pub fn validate_hash(&self) -> bool {
        let payload = serde_json::to_vec(&HashStablePrecheckBundleView {
            schema_version: &self.schema_version,
            toolchain_version: &self.toolchain_version,
            hashes: &self.hashes,
            report: &self.report,
        })
        .unwrap_or_default();
        self.bundle_hash == sha256_prefixed(&payload)
            && self.schema_version == PROVE_PRECHECK_BUNDLE_SCHEMA_VERSION
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PerConstraintProof {
    pub constraint_id: u64,
    pub wire_values: Vec<WireValueEntry>,
    pub a_eval_hex: String,
    pub b_eval_hex: String,
    pub c_eval_hex: String,
    pub resolved: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WireValueEntry {
    pub wire_id: u32,
    pub value_hex: String,
    pub merkle_siblings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProofArtifact {
    pub schema_version: String,
    pub toolchain_version: String,
    pub proof_hash: String,
    pub source_cbir_hash: String,
    pub merkle_root: String,
    pub num_constraints: u64,
    pub num_wires: u32,
    pub public_input_count: u32,
    pub public_output_count: u32,
    pub constraint_proofs: Vec<PerConstraintProof>,
    pub public_inputs_hash: Option<String>,
    pub verifier_key_fingerprint: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct HashStableProofArtifactView<'a> {
    schema_version: &'a str,
    toolchain_version: &'a str,
    source_cbir_hash: &'a str,
    merkle_root: &'a str,
    num_constraints: u64,
    num_wires: u32,
    public_input_count: u32,
    public_output_count: u32,
    constraint_proofs: &'a [PerConstraintProof],
    public_inputs_hash: &'a Option<String>,
    verifier_key_fingerprint: &'a Option<String>,
}

impl ProofArtifact {
    pub fn seal_hash(&mut self) {
        let payload = serde_json::to_vec(&HashStableProofArtifactView {
            schema_version: &self.schema_version,
            toolchain_version: &self.toolchain_version,
            source_cbir_hash: &self.source_cbir_hash,
            merkle_root: &self.merkle_root,
            num_constraints: self.num_constraints,
            num_wires: self.num_wires,
            public_input_count: self.public_input_count,
            public_output_count: self.public_output_count,
            constraint_proofs: &self.constraint_proofs,
            public_inputs_hash: &self.public_inputs_hash,
            verifier_key_fingerprint: &self.verifier_key_fingerprint,
        })
        .unwrap_or_default();
        self.proof_hash = sha256_prefixed(&payload);
    }

    pub fn validate_hash(&self) -> bool {
        let payload = serde_json::to_vec(&HashStableProofArtifactView {
            schema_version: &self.schema_version,
            toolchain_version: &self.toolchain_version,
            source_cbir_hash: &self.source_cbir_hash,
            merkle_root: &self.merkle_root,
            num_constraints: self.num_constraints,
            num_wires: self.num_wires,
            public_input_count: self.public_input_count,
            public_output_count: self.public_output_count,
            constraint_proofs: &self.constraint_proofs,
            public_inputs_hash: &self.public_inputs_hash,
            verifier_key_fingerprint: &self.verifier_key_fingerprint,
        })
        .unwrap_or_default();
        self.proof_hash == sha256_prefixed(&payload)
            && self.schema_version == PROOF_ARTIFACT_SCHEMA_VERSION
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProofBundle {
    pub schema_version: String,
    pub toolchain_version: String,
    pub bundle_hash: String,
    pub backend: String,
    pub status: String,
    pub proof_generated: bool,
    pub precheck_bundle_path: String,
    pub precheck_bundle_hash: String,
    pub backend_capabilities_manifest_path: Option<String>,
    pub backend_capabilities_manifest_hash: Option<String>,
    pub proof_hash: Option<String>,
    pub proof_artifact_path: Option<String>,
    pub public_inputs_hash: Option<String>,
    pub verifier_key_fingerprint: Option<String>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct HashStableProofBundleView<'a> {
    schema_version: &'a str,
    toolchain_version: &'a str,
    backend: &'a str,
    status: &'a str,
    proof_generated: bool,
    precheck_bundle_path: &'a str,
    precheck_bundle_hash: &'a str,
    backend_capabilities_manifest_path: &'a Option<String>,
    backend_capabilities_manifest_hash: &'a Option<String>,
    proof_hash: &'a Option<String>,
    proof_artifact_path: &'a Option<String>,
    public_inputs_hash: &'a Option<String>,
    verifier_key_fingerprint: &'a Option<String>,
    notes: &'a [String],
}

impl ProofBundle {
    pub fn new_precheck_only(
        precheck_bundle_path: String,
        precheck_bundle_hash: String,
        backend_capabilities_manifest_path: Option<String>,
        backend_capabilities_manifest_hash: Option<String>,
    ) -> Self {
        let mut bundle = Self {
            schema_version: PROOF_BUNDLE_SCHEMA_VERSION.to_string(),
            toolchain_version: env!("CARGO_PKG_VERSION").to_string(),
            bundle_hash: String::new(),
            backend: "binius64".to_string(),
            status: "precheck-only".to_string(),
            proof_generated: false,
            precheck_bundle_path,
            precheck_bundle_hash,
            backend_capabilities_manifest_path,
            backend_capabilities_manifest_hash,
            proof_hash: None,
            proof_artifact_path: None,
            public_inputs_hash: None,
            verifier_key_fingerprint: None,
            notes: vec![
                "Proof generation backend integration pending.".to_string(),
                "Bundle is valid for precheck and artifact integrity verification.".to_string(),
            ],
        };
        bundle.seal_hash();
        bundle
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new_with_proof(
        precheck_bundle_path: String,
        precheck_bundle_hash: String,
        proof_artifact_path: String,
        proof_hash: String,
        public_inputs_hash: Option<String>,
        verifier_key_fingerprint: Option<String>,
        backend_capabilities_manifest_path: Option<String>,
        backend_capabilities_manifest_hash: Option<String>,
    ) -> Self {
        let mut bundle = Self {
            schema_version: PROOF_BUNDLE_SCHEMA_VERSION.to_string(),
            toolchain_version: env!("CARGO_PKG_VERSION").to_string(),
            bundle_hash: String::new(),
            backend: "binius64".to_string(),
            status: "proof-generated".to_string(),
            proof_generated: true,
            precheck_bundle_path,
            precheck_bundle_hash,
            backend_capabilities_manifest_path,
            backend_capabilities_manifest_hash,
            proof_hash: Some(proof_hash),
            proof_artifact_path: Some(proof_artifact_path),
            public_inputs_hash,
            verifier_key_fingerprint,
            notes: vec!["Proof generated via Merkle-based constraint satisfaction.".to_string()],
        };
        bundle.seal_hash();
        bundle
    }

    pub fn seal_hash(&mut self) {
        let payload = serde_json::to_vec(&HashStableProofBundleView {
            schema_version: &self.schema_version,
            toolchain_version: &self.toolchain_version,
            backend: &self.backend,
            status: &self.status,
            proof_generated: self.proof_generated,
            precheck_bundle_path: &self.precheck_bundle_path,
            precheck_bundle_hash: &self.precheck_bundle_hash,
            backend_capabilities_manifest_path: &self.backend_capabilities_manifest_path,
            backend_capabilities_manifest_hash: &self.backend_capabilities_manifest_hash,
            proof_hash: &self.proof_hash,
            proof_artifact_path: &self.proof_artifact_path,
            public_inputs_hash: &self.public_inputs_hash,
            verifier_key_fingerprint: &self.verifier_key_fingerprint,
            notes: &self.notes,
        })
        .unwrap_or_default();
        self.bundle_hash = sha256_prefixed(&payload);
    }

    pub fn validate_hash(&self) -> bool {
        let payload = serde_json::to_vec(&HashStableProofBundleView {
            schema_version: &self.schema_version,
            toolchain_version: &self.toolchain_version,
            backend: &self.backend,
            status: &self.status,
            proof_generated: self.proof_generated,
            precheck_bundle_path: &self.precheck_bundle_path,
            precheck_bundle_hash: &self.precheck_bundle_hash,
            backend_capabilities_manifest_path: &self.backend_capabilities_manifest_path,
            backend_capabilities_manifest_hash: &self.backend_capabilities_manifest_hash,
            proof_hash: &self.proof_hash,
            proof_artifact_path: &self.proof_artifact_path,
            public_inputs_hash: &self.public_inputs_hash,
            verifier_key_fingerprint: &self.verifier_key_fingerprint,
            notes: &self.notes,
        })
        .unwrap_or_default();
        self.bundle_hash == sha256_prefixed(&payload)
            && self.schema_version == PROOF_BUNDLE_SCHEMA_VERSION
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BackendCapabilityFlags {
    pub precheck_only_supported: bool,
    pub proof_generation_supported: bool,
    pub verify_supported: bool,
    pub proof_hash_supported: bool,
    pub public_inputs_hash_supported: bool,
    pub verifier_key_fingerprint_supported: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BackendCapabilitiesManifest {
    pub schema_version: String,
    pub toolchain_version: String,
    pub manifest_hash: String,
    pub backend: String,
    pub capabilities: BackendCapabilityFlags,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct HashStableBackendCapabilitiesView<'a> {
    schema_version: &'a str,
    toolchain_version: &'a str,
    backend: &'a str,
    capabilities: &'a BackendCapabilityFlags,
    notes: &'a [String],
}

impl BackendCapabilitiesManifest {
    pub fn new_precheck_only() -> Self {
        let mut manifest = Self {
            schema_version: BACKEND_CAPABILITIES_SCHEMA_VERSION.to_string(),
            toolchain_version: env!("CARGO_PKG_VERSION").to_string(),
            manifest_hash: String::new(),
            backend: "binius64".to_string(),
            capabilities: BackendCapabilityFlags {
                precheck_only_supported: true,
                proof_generation_supported: true,
                verify_supported: true,
                proof_hash_supported: true,
                public_inputs_hash_supported: true,
                verifier_key_fingerprint_supported: false,
            },
            notes: vec![
                "Precheck-only capability profile.".to_string(),
                "Enable proof_generation_supported only when real backend proving is integrated."
                    .to_string(),
            ],
        };
        manifest.seal_hash();
        manifest
    }

    pub fn seal_hash(&mut self) {
        let payload = serde_json::to_vec(&HashStableBackendCapabilitiesView {
            schema_version: &self.schema_version,
            toolchain_version: &self.toolchain_version,
            backend: &self.backend,
            capabilities: &self.capabilities,
            notes: &self.notes,
        })
        .unwrap_or_default();
        self.manifest_hash = sha256_prefixed(&payload);
    }

    pub fn validate_hash(&self) -> bool {
        let payload = serde_json::to_vec(&HashStableBackendCapabilitiesView {
            schema_version: &self.schema_version,
            toolchain_version: &self.toolchain_version,
            backend: &self.backend,
            capabilities: &self.capabilities,
            notes: &self.notes,
        })
        .unwrap_or_default();
        self.manifest_hash == sha256_prefixed(&payload)
            && self.schema_version == BACKEND_CAPABILITIES_SCHEMA_VERSION
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LoweringGateEntry {
    pub constraint_id: u64,
    pub gate_kind: String,
    pub num_signals: u32,
    pub limb_width: Option<String>,
    pub passes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LoweringReport {
    pub schema_version: String,
    pub toolchain_version: String,
    pub report_hash: String,
    pub source_cbir_hash: String,
    pub total_constraints: u64,
    pub gate_counts: std::collections::BTreeMap<String, usize>,
    pub gates: Vec<LoweringGateEntry>,
    pub limb_width: String,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct HashStableLoweringReportView<'a> {
    schema_version: &'a str,
    toolchain_version: &'a str,
    source_cbir_hash: &'a str,
    total_constraints: u64,
    gate_counts: &'a std::collections::BTreeMap<String, usize>,
    gates: &'a [LoweringGateEntry],
    limb_width: &'a str,
    warnings: &'a [String],
}

impl LoweringReport {
    pub fn new(
        source_cbir_hash: String,
        total_constraints: u64,
        gate_counts: std::collections::BTreeMap<String, usize>,
        gates: Vec<LoweringGateEntry>,
        limb_width: String,
        warnings: Vec<String>,
    ) -> Self {
        let mut report = Self {
            schema_version: LOWERING_REPORT_SCHEMA_VERSION.to_string(),
            toolchain_version: env!("CARGO_PKG_VERSION").to_string(),
            report_hash: String::new(),
            source_cbir_hash,
            total_constraints,
            gate_counts,
            gates,
            limb_width,
            warnings,
        };
        report.seal_hash();
        report
    }

    pub fn seal_hash(&mut self) {
        let payload = serde_json::to_vec(&HashStableLoweringReportView {
            schema_version: &self.schema_version,
            toolchain_version: &self.toolchain_version,
            source_cbir_hash: &self.source_cbir_hash,
            total_constraints: self.total_constraints,
            gate_counts: &self.gate_counts,
            gates: &self.gates,
            limb_width: &self.limb_width,
            warnings: &self.warnings,
        })
        .unwrap_or_default();
        self.report_hash = sha256_prefixed(&payload);
    }

    pub fn validate_hash(&self) -> bool {
        let payload = serde_json::to_vec(&HashStableLoweringReportView {
            schema_version: &self.schema_version,
            toolchain_version: &self.toolchain_version,
            source_cbir_hash: &self.source_cbir_hash,
            total_constraints: self.total_constraints,
            gate_counts: &self.gate_counts,
            gates: &self.gates,
            limb_width: &self.limb_width,
            warnings: &self.warnings,
        })
        .unwrap_or_default();
        self.report_hash == sha256_prefixed(&payload)
            && self.schema_version == LOWERING_REPORT_SCHEMA_VERSION
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PatternDetectionEntry {
    pub pattern_name: String,
    pub confidence: String,
    pub constraint_ids: Vec<u64>,
    pub optimized_to: String,
    pub estimated_saving: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OptimizationStats {
    pub total_original: u64,
    pub optimized_count: u64,
    pub compatibility_count: u64,
    pub eliminated_count: u64,
    pub estimated_field_mul_savings_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OptimizationReport {
    pub schema_version: String,
    pub toolchain_version: String,
    pub report_hash: String,
    pub source_cbir_hash: String,
    pub mode: String,
    pub min_confidence: String,
    pub stats: OptimizationStats,
    pub patterns: Vec<PatternDetectionEntry>,
    pub gate_counts: std::collections::BTreeMap<String, usize>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct HashStableOptimizationReportView<'a> {
    schema_version: &'a str,
    toolchain_version: &'a str,
    source_cbir_hash: &'a str,
    mode: &'a str,
    min_confidence: &'a str,
    stats: &'a OptimizationStats,
    patterns: &'a [PatternDetectionEntry],
    gate_counts: &'a std::collections::BTreeMap<String, usize>,
    warnings: &'a [String],
}

impl OptimizationReport {
    pub fn new(
        source_cbir_hash: String,
        mode: String,
        min_confidence: String,
        stats: OptimizationStats,
        patterns: Vec<PatternDetectionEntry>,
        gate_counts: std::collections::BTreeMap<String, usize>,
        warnings: Vec<String>,
    ) -> Self {
        let mut report = Self {
            schema_version: OPTIMIZATION_REPORT_SCHEMA_VERSION.to_string(),
            toolchain_version: env!("CARGO_PKG_VERSION").to_string(),
            report_hash: String::new(),
            source_cbir_hash,
            mode,
            min_confidence,
            stats,
            patterns,
            gate_counts,
            warnings,
        };
        report.seal_hash();
        report
    }

    pub fn seal_hash(&mut self) {
        let payload = serde_json::to_vec(&HashStableOptimizationReportView {
            schema_version: &self.schema_version,
            toolchain_version: &self.toolchain_version,
            source_cbir_hash: &self.source_cbir_hash,
            mode: &self.mode,
            min_confidence: &self.min_confidence,
            stats: &self.stats,
            patterns: &self.patterns,
            gate_counts: &self.gate_counts,
            warnings: &self.warnings,
        })
        .unwrap_or_default();
        self.report_hash = sha256_prefixed(&payload);
    }

    pub fn validate_hash(&self) -> bool {
        let payload = serde_json::to_vec(&HashStableOptimizationReportView {
            schema_version: &self.schema_version,
            toolchain_version: &self.toolchain_version,
            source_cbir_hash: &self.source_cbir_hash,
            mode: &self.mode,
            min_confidence: &self.min_confidence,
            stats: &self.stats,
            patterns: &self.patterns,
            gate_counts: &self.gate_counts,
            warnings: &self.warnings,
        })
        .unwrap_or_default();
        self.report_hash == sha256_prefixed(&payload)
            && self.schema_version == OPTIMIZATION_REPORT_SCHEMA_VERSION
    }
}

pub fn sha256_prefixed(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    format!("sha256:{digest:x}")
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use jsonschema::JSONSchema;
    use serde_json::Value;

    use crate::{
        BackendCapabilitiesManifest, LoweringReport, OptimizationReport, OptimizationStats,
        PatternDetectionEntry, PerConstraintProof, ProofArtifact, ProofBundle, ProvePrecheckBundle,
        ProvePrecheckHashes, ProvePrecheckReportSummary, WireValueEntry, sha256_prefixed,
    };

    #[test]
    fn prove_precheck_bundle_matches_json_schema() {
        let schema =
            load_schema("prove-precheck-bundle-v1.schema.json").expect("schema should load");
        let compiled = JSONSchema::compile(&schema).expect("schema should compile");

        let bundle = ProvePrecheckBundle::new(
            ProvePrecheckHashes {
                circuit_hash: sha256_prefixed(b"circuit"),
                witness_hash: sha256_prefixed(b"witness"),
                wasm_hash: sha256_prefixed(b"wasm"),
                input_hash: sha256_prefixed(b"input"),
                binius_witness_hash: Some(sha256_prefixed(b"binius")),
            },
            ProvePrecheckReportSummary {
                precheck_passed: true,
                generated_witness_path: std::env::temp_dir()
                    .join("circom.wtns")
                    .display()
                    .to_string(),
                generated_witness_len: 1024,
                r1cs_wire_count: 1024,
                witness_equivalent: Some(true),
                value_mismatch_count: 0,
                constraint_failure_count: 0,
            },
        );

        let instance = serde_json::to_value(bundle).expect("bundle should serialize");
        if let Err(errors) = compiled.validate(&instance) {
            let messages = errors.map(|err| err.to_string()).collect::<Vec<_>>();
            panic!("schema validation failed: {}", messages.join(" | "));
        }
    }

    #[test]
    fn prove_precheck_bundle_schema_rejects_missing_required_field() {
        let schema =
            load_schema("prove-precheck-bundle-v1.schema.json").expect("schema should load");
        let compiled = JSONSchema::compile(&schema).expect("schema should compile");

        let mut instance = serde_json::json!({
            "schema_version": "prove-precheck-bundle/v1",
            "toolchain_version": "0.1.0",
            "bundle_hash": sha256_prefixed(b"bundle"),
            "hashes": {
                "circuit_hash": sha256_prefixed(b"circuit"),
                "witness_hash": sha256_prefixed(b"witness"),
                "wasm_hash": sha256_prefixed(b"wasm"),
                "input_hash": sha256_prefixed(b"input"),
                "binius_witness_hash": null
            },
            "report": {
                "precheck_passed": true,
                "generated_witness_path": "",
                "generated_witness_len": 4,
                "r1cs_wire_count": 4,
                "witness_equivalent": null,
                "value_mismatch_count": 0,
                "constraint_failure_count": 0
            }
        });

        if let Some(report) = instance.get_mut("report").and_then(Value::as_object_mut) {
            report.remove("generated_witness_len");
        }

        let validation = compiled.validate(&instance);
        assert!(
            validation.is_err(),
            "schema should reject bundle with missing required field"
        );
    }

    #[test]
    fn proof_bundle_hash_roundtrip_validates() {
        let bundle = ProofBundle::new_precheck_only(
            "build/prove_precheck_report.json".to_string(),
            sha256_prefixed(b"precheck"),
            Some("build/backend_capabilities.json".to_string()),
            Some(sha256_prefixed(b"backend-capabilities")),
        );
        assert!(bundle.validate_hash());
    }

    #[test]
    fn proof_bundle_matches_json_schema() {
        let schema = load_schema("proof-bundle-v1.schema.json").expect("schema should load");
        let compiled = JSONSchema::compile(&schema).expect("schema should compile");

        let bundle = ProofBundle::new_precheck_only(
            "build/prove_precheck_report.json".to_string(),
            sha256_prefixed(b"precheck"),
            None,
            None,
        );

        let instance = serde_json::to_value(bundle).expect("bundle should serialize");
        if let Err(errors) = compiled.validate(&instance) {
            let messages = errors.map(|err| err.to_string()).collect::<Vec<_>>();
            panic!("schema validation failed: {}", messages.join(" | "));
        }
    }

    #[test]
    fn backend_capabilities_manifest_hash_roundtrip_validates() {
        let manifest = BackendCapabilitiesManifest::new_precheck_only();
        assert!(manifest.validate_hash());
    }

    #[test]
    fn backend_capabilities_manifest_matches_json_schema() {
        let schema =
            load_schema("backend-capabilities-v1.schema.json").expect("schema should load");
        let compiled = JSONSchema::compile(&schema).expect("schema should compile");

        let manifest = BackendCapabilitiesManifest::new_precheck_only();
        let instance = serde_json::to_value(manifest).expect("manifest should serialize");
        if let Err(errors) = compiled.validate(&instance) {
            let messages = errors.map(|err| err.to_string()).collect::<Vec<_>>();
            panic!("schema validation failed: {}", messages.join(" | "));
        }
    }

    #[test]
    fn lowering_report_hash_roundtrip_validates() {
        let mut gate_counts = std::collections::BTreeMap::new();
        gate_counts.insert("mul".to_string(), 1);
        gate_counts.insert("boolean".to_string(), 2);
        let report = LoweringReport::new(
            sha256_prefixed(b"cbir-doc"),
            3,
            gate_counts,
            vec![],
            "u32".to_string(),
            vec![],
        );
        assert!(report.validate_hash());
    }

    #[test]
    fn optimization_report_hash_roundtrip_validates() {
        let mut gate_counts = std::collections::BTreeMap::new();
        gate_counts.insert("boolean".to_string(), 5);
        let report = OptimizationReport::new(
            sha256_prefixed(b"cbir-doc"),
            "optimized".to_string(),
            "Strong".to_string(),
            OptimizationStats {
                total_original: 100,
                optimized_count: 60,
                compatibility_count: 40,
                eliminated_count: 0,
                estimated_field_mul_savings_pct: 42.5,
            },
            vec![PatternDetectionEntry {
                pattern_name: "boolean-constraint".to_string(),
                confidence: "Exact".to_string(),
                constraint_ids: vec![1, 2, 3, 4, 5],
                optimized_to: "boolean".to_string(),
                estimated_saving: "3x".to_string(),
            }],
            gate_counts,
            vec![],
        );
        assert!(report.validate_hash());
    }

    #[test]
    fn proof_artifact_hash_roundtrip_validates() {
        let mut artifact = ProofArtifact {
            schema_version: "proof-artifact/v1".to_string(),
            toolchain_version: "0.1.0".to_string(),
            proof_hash: String::new(),
            source_cbir_hash: sha256_prefixed(b"cbir-doc"),
            merkle_root: sha256_prefixed(b"merkle-root"),
            num_constraints: 3,
            num_wires: 5,
            public_input_count: 1,
            public_output_count: 0,
            constraint_proofs: vec![PerConstraintProof {
                constraint_id: 1,
                wire_values: vec![WireValueEntry {
                    wire_id: 1,
                    value_hex: "0x1".to_string(),
                    merkle_siblings: vec![sha256_prefixed(b"sibling")],
                }],
                a_eval_hex: "0x1".to_string(),
                b_eval_hex: "0x2".to_string(),
                c_eval_hex: "0x2".to_string(),
                resolved: true,
            }],
            public_inputs_hash: Some(sha256_prefixed(b"public-inputs")),
            verifier_key_fingerprint: None,
        };
        artifact.seal_hash();
        assert!(artifact.validate_hash());
    }

    fn load_schema(file_name: &str) -> Result<Value, Box<dyn std::error::Error>> {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let schema_path = manifest_dir.join("../../docs/contracts").join(file_name);
        let schema_text = fs::read_to_string(schema_path)?;
        let schema = serde_json::from_str(&schema_text)?;
        Ok(schema)
    }
}
