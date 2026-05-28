use std::fs;
use std::path::PathBuf;

use cirbinius_core::{CommandAction, CommandContext, DoctorArgs, ProveArgs, dispatch};

mod common;

#[test]
fn prove_requires_manifest_in_full_mode_and_honors_capabilities() {
    let workspace_root = common::workspace_root();
    let temp_dir = common::temp_dir("prove-gating");
    fs::create_dir_all(&temp_dir).expect("should create temp dir");

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

    let err_without_manifest = dispatch(
        CommandAction::Prove(ProveArgs {
            r1cs_path: PathBuf::from("tests/circuits/simple_mul.r1cs"),
            sym_path: Some(PathBuf::from("tests/circuits/simple_mul.sym")),
            wasm_path: wasm_path.clone(),
            input_json_path: PathBuf::from("tests/circuits/simple_mul_input.json"),
            out_dir: temp_dir.join("out-no-manifest"),
            snarkjs_bin: script_path.to_string_lossy().to_string(),
            binius_witness_path: Some(PathBuf::from(
                "tests/circuits/simple_mul_binius_witness_ok.json",
            )),
            precheck_report_path: Some(temp_dir.join("precheck-no-manifest.json")),
            precheck_only: false,
            backend_capabilities_path: None,
        }),
        &context,
    )
    .expect_err("full prove mode should require backend capabilities manifest");
    assert!(
        err_without_manifest
            .to_string()
            .contains("requires backend capabilities manifest"),
        "expected explicit missing-manifest error"
    );

    let manifest_path = temp_dir.join("backend_capabilities.json");
    dispatch(
        CommandAction::Doctor(DoctorArgs {
            out_path: Some(manifest_path.clone()),
        }),
        &context,
    )
    .expect("doctor should emit backend capabilities manifest");

    let err_with_manifest = dispatch(
        CommandAction::Prove(ProveArgs {
            r1cs_path: PathBuf::from("tests/circuits/simple_mul.r1cs"),
            sym_path: Some(PathBuf::from("tests/circuits/simple_mul.sym")),
            wasm_path,
            input_json_path: PathBuf::from("tests/circuits/simple_mul_input.json"),
            out_dir: temp_dir.join("out-with-manifest"),
            snarkjs_bin: script_path.to_string_lossy().to_string(),
            binius_witness_path: Some(PathBuf::from(
                "tests/circuits/simple_mul_binius_witness_ok.json",
            )),
            precheck_report_path: Some(temp_dir.join("precheck-with-manifest.json")),
            precheck_only: false,
            backend_capabilities_path: Some(manifest_path),
        }),
        &context,
    )
    .expect_err("manifest should gate and refuse unsupported proof generation");
    assert!(
        err_with_manifest
            .to_string()
            .contains("proof generation is not supported"),
        "expected capability-gated proof generation error"
    );
}
