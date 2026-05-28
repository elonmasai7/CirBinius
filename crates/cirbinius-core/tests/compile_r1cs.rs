use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use cirbinius_core::{CommandAction, CommandContext, CompileR1csArgs, dispatch};
use cirbinius_types::CompilerOptions;

#[test]
fn compile_r1cs_emits_expected_cbir_golden() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir.join("../..");

    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be after unix epoch")
        .as_nanos();
    let out_dir = std::env::temp_dir().join(format!("cirbinius-core-test-{unique}"));

    let action = CommandAction::CompileR1cs(CompileR1csArgs {
        r1cs_path: PathBuf::from("tests/circuits/simple_mul.r1cs"),
        sym_path: Some(PathBuf::from("tests/circuits/simple_mul.sym")),
        out_dir: out_dir.clone(),
        options: CompilerOptions::default(),
    });

    let context = CommandContext {
        project_root: workspace_root.clone(),
    };

    let outcome = dispatch(action, &context).expect("compile-r1cs should succeed");
    let artifact_path = outcome
        .artifact_path
        .expect("compile-r1cs should return artifact path");
    let actual = fs::read_to_string(artifact_path).expect("artifact file should be readable");

    let golden_path = workspace_root.join("tests/golden/simple_mul.cbir.json");
    let expected = fs::read_to_string(golden_path).expect("golden file should be readable");

    assert_eq!(
        actual.trim_end(),
        expected.trim_end(),
        "compiled CBIR does not match golden output"
    );
}
