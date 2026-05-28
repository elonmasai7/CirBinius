use std::fs;

use cirbinius_artifacts::BackendCapabilitiesManifest;
use cirbinius_core::{CommandAction, CommandContext, DoctorArgs, dispatch};

mod common;

#[test]
fn doctor_emits_backend_capabilities_manifest() {
    let workspace_root = common::workspace_root();
    let out_path = common::temp_dir("doctor").join("backend_capabilities.json");

    let outcome = dispatch(
        CommandAction::Doctor(DoctorArgs {
            out_path: Some(out_path.clone()),
        }),
        &CommandContext {
            project_root: workspace_root,
        },
    )
    .expect("doctor should succeed");

    assert_eq!(outcome.artifact_path, Some(out_path.clone()));
    let manifest_json = fs::read_to_string(out_path).expect("manifest should exist");
    let manifest: BackendCapabilitiesManifest =
        serde_json::from_str(&manifest_json).expect("manifest should parse as json");

    assert_eq!(manifest.schema_version, "backend-capabilities/v1");
    assert!(manifest.validate_hash());
    assert!(manifest.capabilities.precheck_only_supported);
    assert!(!manifest.capabilities.proof_generation_supported);
}
