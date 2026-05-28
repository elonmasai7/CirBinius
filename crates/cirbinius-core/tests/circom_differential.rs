use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use cirbinius_core::{CommandAction, CommandContext, CompileCircomArgs, CompileR1csArgs, dispatch};
use cirbinius_types::CompilerOptions;

#[test]
fn circom_compile_matches_compile_r1cs_path() {
    if !circom_available() {
        eprintln!("circom binary not found; skipping differential test");
        return;
    }

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir.join("../..");
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be after unix epoch")
        .as_nanos();

    let temp_root = std::env::temp_dir().join(format!("cirbinius-diff-{unique}"));
    let direct_circom_out = temp_root.join("direct-circom");
    let compile_out = temp_root.join("compile-path");
    let compile_r1cs_out = temp_root.join("compile-r1cs-path");

    fs::create_dir_all(&direct_circom_out).expect("should create direct circom output directory");

    let circom_source = workspace_root.join("tests/circuits/simple_mul.circom");
    let direct_status = Command::new("circom")
        .arg(&circom_source)
        .arg("--r1cs")
        .arg("--sym")
        .arg("--wasm")
        .arg("--output")
        .arg(&direct_circom_out)
        .status()
        .expect("failed to invoke circom for differential fixture generation");
    assert!(direct_status.success(), "circom direct compilation failed");

    let context = CommandContext {
        project_root: workspace_root,
    };

    let compile_outcome = dispatch(
        CommandAction::Compile(CompileCircomArgs {
            source_path: PathBuf::from("tests/circuits/simple_mul.circom"),
            main_component: None,
            include_paths: Vec::new(),
            out_dir: compile_out,
            circom_bin: "circom".to_string(),
            options: CompilerOptions::default(),
        }),
        &context,
    )
    .expect("compile command should succeed when circom is installed");

    let compile_r1cs_outcome = dispatch(
        CommandAction::CompileR1cs(CompileR1csArgs {
            r1cs_path: direct_circom_out.join("simple_mul.r1cs"),
            sym_path: Some(direct_circom_out.join("simple_mul.sym")),
            out_dir: compile_r1cs_out,
            options: CompilerOptions::default(),
        }),
        &context,
    )
    .expect("compile-r1cs command should succeed on circom-generated artifacts");

    let compile_cbir = fs::read_to_string(
        compile_outcome
            .artifact_path
            .expect("compile should emit a CBIR artifact"),
    )
    .expect("compile path CBIR should be readable");
    let compile_r1cs_cbir = fs::read_to_string(
        compile_r1cs_outcome
            .artifact_path
            .expect("compile-r1cs should emit a CBIR artifact"),
    )
    .expect("compile-r1cs path CBIR should be readable");

    assert_eq!(
        compile_cbir.trim_end(),
        compile_r1cs_cbir.trim_end(),
        "compile and compile-r1cs paths produced different CBIR output"
    );
}

fn circom_available() -> bool {
    Command::new("circom")
        .arg("--help")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}
