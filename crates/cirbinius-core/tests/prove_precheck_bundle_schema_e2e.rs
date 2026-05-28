use std::fs;
use std::path::PathBuf;

use cirbinius_core::{CommandAction, CommandContext, ProveArgs, dispatch};
use jsonschema::JSONSchema;
use serde_json::Value;

mod common;

#[test]
fn emitted_prove_precheck_bundle_matches_v1_schema() {
    let workspace_root = common::workspace_root();
    let temp_dir = common::temp_dir("precheck-bundle-schema-e2e");
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

    let precheck_bundle_path = temp_dir.join("prove_precheck_report.json");
    dispatch(
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
            precheck_report_path: Some(precheck_bundle_path.clone()),
            precheck_only: true,
            backend_capabilities_path: None,
        }),
        &CommandContext {
            project_root: workspace_root.clone(),
        },
    )
    .expect("prove precheck-only should succeed");

    let bundle_json =
        fs::read_to_string(precheck_bundle_path).expect("precheck bundle should be readable");
    let bundle_value: Value =
        serde_json::from_str(&bundle_json).expect("precheck bundle should parse as json");

    let schema_path = workspace_root.join("docs/contracts/prove-precheck-bundle-v1.schema.json");
    let schema_text = fs::read_to_string(schema_path).expect("schema file should exist");
    let schema_value: Value = serde_json::from_str(&schema_text).expect("schema should parse");
    let compiled = JSONSchema::compile(&schema_value).expect("schema should compile");

    if let Err(errors) = compiled.validate(&bundle_value) {
        let messages = errors.map(|err| err.to_string()).collect::<Vec<_>>();
        panic!(
            "emitted prove-precheck bundle violates v1 schema: {}",
            messages.join(" | ")
        );
    }
}
