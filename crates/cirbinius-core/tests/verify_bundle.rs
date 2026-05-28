use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use cirbinius_core::{CommandAction, CommandContext, ProveArgs, VerifyArgs, dispatch};

#[test]
fn verify_accepts_valid_proof_bundle_and_rejects_tampered_bundle() {
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let temp_dir = temp_dir("verify-bundle");
    fs::create_dir_all(&temp_dir).expect("should create temp directory");

    let wasm_path = temp_dir.join("circuit.wasm");
    fs::write(&wasm_path, b"placeholder wasm").expect("should write wasm placeholder");
    let script_path = temp_dir.join("fake-snarkjs.sh");
    write_fake_snarkjs(
        &script_path,
        &workspace_root.join("tests/circuits/simple_mul.wtns"),
    );

    let context = CommandContext {
        project_root: workspace_root,
    };

    let prove_outcome = dispatch(
        CommandAction::Prove(ProveArgs {
            r1cs_path: PathBuf::from("tests/circuits/simple_mul.r1cs"),
            sym_path: Some(PathBuf::from("tests/circuits/simple_mul.sym")),
            wasm_path,
            input_json_path: PathBuf::from("tests/circuits/simple_mul_input.json"),
            out_dir: temp_dir.join("out"),
            snarkjs_bin: script_path.to_string_lossy().to_string(),
            binius_witness_path: Some(PathBuf::from(
                "tests/circuits/simple_mul_binius_witness_ok.json",
            )),
            precheck_report_path: Some(temp_dir.join("precheck_report.json")),
            precheck_only: true,
            backend_capabilities_path: None,
        }),
        &context,
    )
    .expect("prove precheck-only should succeed");

    let bundle_path = prove_outcome
        .artifact_path
        .expect("prove should return proof bundle path");

    dispatch(
        CommandAction::Verify(VerifyArgs {
            bundle_path: bundle_path.clone(),
        }),
        &context,
    )
    .expect("verify should accept untampered bundle");

    let mut bundle_json: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&bundle_path).expect("bundle should be readable"))
            .expect("bundle should parse as json");
    bundle_json["status"] = serde_json::Value::String("tampered".to_string());
    fs::write(
        &bundle_path,
        serde_json::to_string_pretty(&bundle_json).expect("tampered bundle should serialize"),
    )
    .expect("tampered bundle should write");

    let err = dispatch(CommandAction::Verify(VerifyArgs { bundle_path }), &context)
        .expect_err("verify should reject tampered bundle");
    assert!(
        err.to_string().contains("hash validation failed"),
        "verify error should indicate bundle hash mismatch"
    );
}

fn write_fake_snarkjs(script_path: &Path, witness_fixture_path: &Path) {
    let script = format!(
        "#!/bin/sh\ncp '{}' \"$5\"\n",
        witness_fixture_path.display()
    );
    fs::write(script_path, script).expect("should write fake snarkjs script");

    use std::os::unix::fs::PermissionsExt;
    let mut perms = fs::metadata(script_path)
        .expect("should read script metadata")
        .permissions();
    perms.set_mode(0o755);
    fs::set_permissions(script_path, perms).expect("should set executable permissions");
}

fn temp_dir(tag: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be after unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("cirbinius-{tag}-{unique}"))
}
