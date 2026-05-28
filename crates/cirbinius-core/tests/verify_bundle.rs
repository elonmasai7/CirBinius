use std::fs;
use std::path::PathBuf;

use cirbinius_core::{CommandAction, CommandContext, ProveArgs, VerifyArgs, dispatch};

mod common;

#[test]
fn verify_accepts_valid_proof_bundle_and_rejects_tampered_bundle() {
    let workspace_root = common::workspace_root();
    let temp_dir = common::temp_dir("verify-bundle");
    fs::create_dir_all(&temp_dir).expect("should create temp directory");

    let wasm_path = temp_dir.join("circuit.wasm");
    fs::write(&wasm_path, b"placeholder wasm").expect("should write wasm placeholder");
    let script_name = if cfg!(windows) {
        "fake-snarkjs.bat"
    } else {
        "fake-snarkjs.sh"
    };
    let script_path = temp_dir.join(script_name);
    common::write_fake_snarkjs(
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
