use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use cirbinius_core::{
    AnalyzeArgs, CommandAction, CommandContext, LowerArgs, OptimizeArgs, dispatch,
};
use cirbinius_types::{CompileMode, CompilerOptions};

#[test]
fn analyze_optimize_lower_pipeline_emits_phase4_artifacts() {
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be after unix epoch")
        .as_nanos();
    let temp_root = std::env::temp_dir().join(format!("cirbinius-phase4-{unique}"));

    let context = CommandContext {
        project_root: workspace_root,
    };

    let analyze_path = temp_root.join("analyze.json");
    dispatch(
        CommandAction::Analyze(AnalyzeArgs {
            r1cs_path: PathBuf::from("tests/circuits/simple_mul.r1cs"),
            sym_path: Some(PathBuf::from("tests/circuits/simple_mul.sym")),
            out_path: analyze_path.clone(),
            mode: CompileMode::OptimizedBinary,
        }),
        &context,
    )
    .expect("analyze command should succeed");

    let analyze_json = fs::read_to_string(&analyze_path).expect("analyze report should exist");
    let analyze_report: serde_json::Value =
        serde_json::from_str(&analyze_json).expect("analyze report should be valid json");
    assert!(
        analyze_report["pass_counts"]["total_constraints"].is_number(),
        "analysis report should include total constraint count"
    );

    let rules_index_path = analyze_path.with_file_name("lowering_rules_index.json");
    assert!(
        rules_index_path.exists(),
        "lowering rules index should be emitted by analyze"
    );
    let rules_index_json =
        fs::read_to_string(&rules_index_path).expect("rules index should be readable");
    let rules_index: serde_json::Value =
        serde_json::from_str(&rules_index_json).expect("rules index should be valid json");
    assert_eq!(rules_index["schema_version"], "lowering-rules-index/v1");
    assert!(
        rules_index["rules"].is_array(),
        "rules index should contain machine-readable rules array"
    );

    let optimize_dir = temp_root.join("opt");
    let optimize_outcome = dispatch(
        CommandAction::Optimize(OptimizeArgs {
            r1cs_path: PathBuf::from("tests/circuits/simple_mul.r1cs"),
            sym_path: Some(PathBuf::from("tests/circuits/simple_mul.sym")),
            out_dir: optimize_dir.clone(),
            mode: CompileMode::Compatibility,
            options: CompilerOptions::default(),
        }),
        &context,
    )
    .expect("optimize command should succeed");

    let optimized_cbir_path = optimize_outcome
        .artifact_path
        .expect("optimize should emit optimized cbir path");
    assert!(
        optimized_cbir_path.exists(),
        "optimized cbir should be written"
    );
    assert!(
        optimize_dir.join("optimization_report.json").exists(),
        "optimization report should be written"
    );

    let lowered_path = temp_root.join("lowered.json");
    dispatch(
        CommandAction::Lower(LowerArgs {
            cbir_path: optimized_cbir_path,
            out_path: lowered_path.clone(),
            limb_width: None,
        }),
        &context,
    )
    .expect("lower command should succeed");

    let lowered_json = fs::read_to_string(lowered_path).expect("lowered artifact should exist");
    let lowered: serde_json::Value =
        serde_json::from_str(&lowered_json).expect("lowered artifact should be valid json");
    assert_eq!(lowered["schema_version"], "binius64-lowering/v1");
    assert_eq!(lowered["gate_counts"]["mul"], 1);
}
