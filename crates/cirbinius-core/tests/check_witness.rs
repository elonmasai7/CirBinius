use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use cirbinius_core::{CheckWitnessArgs, CommandAction, CommandContext, dispatch};
use cirbinius_witness::WitnessCheckReport;

#[test]
fn check_witness_passes_for_matching_witnesses() {
    let workspace_root = workspace_root();
    let out_path = temp_report_path("ok");

    let outcome = dispatch(
        CommandAction::CheckWitness(CheckWitnessArgs {
            r1cs_path: PathBuf::from("tests/circuits/simple_mul.r1cs"),
            sym_path: Some(PathBuf::from("tests/circuits/simple_mul.sym")),
            circom_witness_path: PathBuf::from("tests/circuits/simple_mul.wtns"),
            binius_witness_path: PathBuf::from("tests/circuits/simple_mul_binius_witness_ok.json"),
            out_path: out_path.clone(),
        }),
        &CommandContext {
            project_root: workspace_root,
        },
    )
    .expect("check-witness should succeed for matching witnesses");

    assert!(
        outcome.message.contains("passed"),
        "expected pass message from check-witness"
    );
    let report_json = fs::read_to_string(out_path).expect("report should be written");
    let report: WitnessCheckReport =
        serde_json::from_str(&report_json).expect("report json should parse");
    assert!(report.equivalent);
    assert_eq!(report.value_mismatch_count, 0);
    assert_eq!(report.constraint_failure_count, 0);
}

#[test]
fn check_witness_fails_with_constraint_diagnostics() {
    let workspace_root = workspace_root();
    let out_path = temp_report_path("bad");

    let err = dispatch(
        CommandAction::CheckWitness(CheckWitnessArgs {
            r1cs_path: PathBuf::from("tests/circuits/simple_mul.r1cs"),
            sym_path: Some(PathBuf::from("tests/circuits/simple_mul.sym")),
            circom_witness_path: PathBuf::from("tests/circuits/simple_mul.wtns"),
            binius_witness_path: PathBuf::from("tests/circuits/simple_mul_binius_witness_bad.json"),
            out_path: out_path.clone(),
        }),
        &CommandContext {
            project_root: workspace_root,
        },
    )
    .expect_err("check-witness should fail for mismatched witnesses");

    let err_message = err.to_string();
    assert!(
        err_message.contains("Witness equivalence check failed"),
        "error should indicate witness equivalence failure"
    );

    let report_json = fs::read_to_string(out_path).expect("failure report should be written");
    let report: WitnessCheckReport =
        serde_json::from_str(&report_json).expect("report json should parse");

    assert!(!report.equivalent);
    assert_eq!(report.value_mismatch_count, 1);
    assert_eq!(report.constraint_failure_count, 1);
    assert_eq!(report.value_mismatches[0].wire_id, 3);
    assert_eq!(report.constraint_failures[0].constraint_id, 1);
    assert_eq!(
        report.constraint_failures[0].signal_path.as_deref(),
        Some("main.c")
    );
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn temp_report_path(tag: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be after unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("cirbinius-check-witness-{tag}-{unique}.json"))
}
