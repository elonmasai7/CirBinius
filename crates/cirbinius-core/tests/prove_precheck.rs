use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use cirbinius_core::{CommandAction, CommandContext, ProveArgs, dispatch};

#[test]
fn prove_precheck_generates_witness_and_passes_with_matching_binius_witness() {
    let workspace_root = workspace_root();
    let temp_dir = temp_dir("prove-ok");
    fs::create_dir_all(&temp_dir).expect("should create temp directory");

    let wasm_path = temp_dir.join("circuit.wasm");
    fs::write(&wasm_path, b"placeholder wasm").expect("should write wasm placeholder");

    let script_path = temp_dir.join("fake-snarkjs.sh");
    write_fake_snarkjs(
        &script_path,
        &workspace_root.join("tests/circuits/simple_mul.wtns"),
    );

    let out_dir = temp_dir.join("out");
    let report_path = temp_dir.join("report.json");
    let outcome = dispatch(
        CommandAction::Prove(ProveArgs {
            r1cs_path: PathBuf::from("tests/circuits/simple_mul.r1cs"),
            sym_path: Some(PathBuf::from("tests/circuits/simple_mul.sym")),
            wasm_path,
            input_json_path: PathBuf::from("tests/circuits/simple_mul_input.json"),
            out_dir,
            snarkjs_bin: script_path.to_string_lossy().to_string(),
            binius_witness_path: Some(PathBuf::from(
                "tests/circuits/simple_mul_binius_witness_ok.json",
            )),
            precheck_report_path: Some(report_path.clone()),
            precheck_only: true,
            backend_capabilities_path: None,
        }),
        &CommandContext {
            project_root: workspace_root,
        },
    )
    .expect("prove precheck should pass");

    assert!(
        outcome.message.contains("precheck completed"),
        "expected prove precheck completion message"
    );

    let report_json = fs::read_to_string(report_path).expect("report should exist");
    let report: serde_json::Value =
        serde_json::from_str(&report_json).expect("report should parse as json");
    assert_eq!(report["schema_version"], "prove-precheck-bundle/v1");
    assert!(
        report["bundle_hash"].as_str().is_some(),
        "bundle hash should be present"
    );
    assert!(
        report["hashes"]["witness_hash"]
            .as_str()
            .map(|hash| hash.starts_with("sha256:"))
            .unwrap_or(false),
        "witness hash should be a sha256-prefixed string"
    );
    assert!(
        report["hashes"]["circuit_hash"]
            .as_str()
            .map(|hash| hash.starts_with("sha256:"))
            .unwrap_or(false),
        "circuit hash should be a sha256-prefixed string"
    );
    assert_eq!(report["report"]["precheck_passed"], true);
    assert_eq!(report["report"]["value_mismatch_count"], 0);
    assert_eq!(report["report"]["constraint_failure_count"], 0);
}

#[test]
fn prove_precheck_fails_when_binius_witness_mismatches() {
    let workspace_root = workspace_root();
    let temp_dir = temp_dir("prove-bad");
    fs::create_dir_all(&temp_dir).expect("should create temp directory");

    let wasm_path = temp_dir.join("circuit.wasm");
    fs::write(&wasm_path, b"placeholder wasm").expect("should write wasm placeholder");

    let script_path = temp_dir.join("fake-snarkjs.sh");
    write_fake_snarkjs(
        &script_path,
        &workspace_root.join("tests/circuits/simple_mul.wtns"),
    );

    let out_dir = temp_dir.join("out");
    let report_path = temp_dir.join("report.json");
    let err = dispatch(
        CommandAction::Prove(ProveArgs {
            r1cs_path: PathBuf::from("tests/circuits/simple_mul.r1cs"),
            sym_path: Some(PathBuf::from("tests/circuits/simple_mul.sym")),
            wasm_path,
            input_json_path: PathBuf::from("tests/circuits/simple_mul_input.json"),
            out_dir,
            snarkjs_bin: script_path.to_string_lossy().to_string(),
            binius_witness_path: Some(PathBuf::from(
                "tests/circuits/simple_mul_binius_witness_bad.json",
            )),
            precheck_report_path: Some(report_path.clone()),
            precheck_only: true,
            backend_capabilities_path: None,
        }),
        &CommandContext {
            project_root: workspace_root,
        },
    )
    .expect_err("prove precheck should fail with mismatched witness");

    assert!(
        err.to_string().contains("Prove precheck failed"),
        "expected prove precheck failure error"
    );

    let report_json = fs::read_to_string(report_path).expect("report should exist on failure");
    let report: serde_json::Value =
        serde_json::from_str(&report_json).expect("report should parse as json");
    assert_eq!(report["report"]["precheck_passed"], false);
    assert_eq!(report["report"]["value_mismatch_count"], 1);
    assert_eq!(report["report"]["constraint_failure_count"], 1);
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

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn temp_dir(tag: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be after unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("cirbinius-{tag}-{unique}"))
}
