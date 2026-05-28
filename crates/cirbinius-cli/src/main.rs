use std::path::PathBuf;

use anyhow::Result;
use cirbinius_core::{
    AnalyzeArgs, CheckWitnessArgs, CommandAction, CommandContext, CompileCircomArgs,
    CompileR1csArgs, DoctorArgs, LowerArgs, OptimizeArgs, ProveArgs, VerifyArgs, dispatch,
};
use cirbinius_types::{CompileMode, CompilerOptions};
use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "cirbinius")]
#[command(about = "Compile Circom circuits into Binius64 proof circuits automatically.")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(long, global = true, default_value = ".")]
    project_root: PathBuf,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Init,
    Compile(CompileCommand),
    CompileR1cs(CompileR1csCommand),
    Inspect,
    Analyze(AnalyzeCommand),
    Optimize(OptimizeCommand),
    Lower(LowerCommand),
    Prove(ProveCommand),
    Verify(VerifyCommand),
    CheckWitness(CheckWitnessCommand),
    Benchmark,
    Explain,
    Doctor(DoctorCommand),
    Clean,
}

#[derive(Debug, Args)]
struct CompileCommand {
    source: PathBuf,

    #[arg(long)]
    main: Option<String>,

    #[arg(long, value_name = "PATH")]
    include: Vec<PathBuf>,

    #[arg(long)]
    out: PathBuf,

    #[arg(long, default_value = "circom")]
    circom_bin: String,
}

#[derive(Debug, Args)]
struct CompileR1csCommand {
    #[arg(long)]
    r1cs: PathBuf,

    #[arg(long)]
    sym: Option<PathBuf>,

    #[arg(long)]
    out: PathBuf,
}

#[derive(Debug, Args)]
struct CheckWitnessCommand {
    #[arg(long)]
    r1cs: PathBuf,

    #[arg(long)]
    sym: Option<PathBuf>,

    #[arg(long)]
    circom_witness: PathBuf,

    #[arg(long)]
    binius_witness: PathBuf,

    #[arg(long)]
    out: PathBuf,
}

#[derive(Debug, Args)]
struct ProveCommand {
    #[arg(long)]
    r1cs: PathBuf,

    #[arg(long)]
    sym: Option<PathBuf>,

    #[arg(long)]
    wasm: PathBuf,

    #[arg(long)]
    input: PathBuf,

    #[arg(long)]
    out: PathBuf,

    #[arg(long, default_value = "snarkjs")]
    snarkjs_bin: String,

    #[arg(long)]
    binius_witness: Option<PathBuf>,

    #[arg(long)]
    precheck_report: Option<PathBuf>,

    #[arg(long)]
    precheck_only: bool,

    #[arg(long)]
    backend_capabilities: Option<PathBuf>,
}

#[derive(Debug, Args)]
struct VerifyCommand {
    #[arg(long)]
    bundle: PathBuf,
}

#[derive(Debug, Args)]
struct DoctorCommand {
    #[arg(long)]
    out: Option<PathBuf>,
}

#[derive(Debug, Args)]
struct AnalyzeCommand {
    #[arg(long)]
    r1cs: PathBuf,

    #[arg(long)]
    sym: Option<PathBuf>,

    #[arg(long)]
    out: PathBuf,

    #[arg(long)]
    optimized_binary: bool,
}

#[derive(Debug, Args)]
struct OptimizeCommand {
    #[arg(long)]
    r1cs: PathBuf,

    #[arg(long)]
    sym: Option<PathBuf>,

    #[arg(long)]
    out: PathBuf,

    #[arg(long)]
    optimized_binary: bool,
}

#[derive(Debug, Args)]
struct LowerCommand {
    #[arg(long)]
    cbir: PathBuf,

    #[arg(long)]
    out: PathBuf,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let action = match cli.command {
        Commands::Init => CommandAction::Init,
        Commands::Compile(cmd) => CommandAction::Compile(CompileCircomArgs {
            source_path: cmd.source,
            main_component: cmd.main,
            include_paths: cmd.include,
            out_dir: cmd.out,
            circom_bin: cmd.circom_bin,
            options: CompilerOptions::default(),
        }),
        Commands::CompileR1cs(cmd) => CommandAction::CompileR1cs(CompileR1csArgs {
            r1cs_path: cmd.r1cs,
            sym_path: cmd.sym,
            out_dir: cmd.out,
            options: CompilerOptions::default(),
        }),
        Commands::Inspect => CommandAction::Inspect,
        Commands::Analyze(cmd) => CommandAction::Analyze(AnalyzeArgs {
            r1cs_path: cmd.r1cs,
            sym_path: cmd.sym,
            out_path: cmd.out,
            mode: mode_from_flag(cmd.optimized_binary),
        }),
        Commands::Optimize(cmd) => CommandAction::Optimize(OptimizeArgs {
            r1cs_path: cmd.r1cs,
            sym_path: cmd.sym,
            out_dir: cmd.out,
            mode: mode_from_flag(cmd.optimized_binary),
            options: CompilerOptions {
                mode: mode_from_flag(cmd.optimized_binary),
                ..CompilerOptions::default()
            },
        }),
        Commands::Lower(cmd) => CommandAction::Lower(LowerArgs {
            cbir_path: cmd.cbir,
            out_path: cmd.out,
        }),
        Commands::Prove(cmd) => CommandAction::Prove(ProveArgs {
            r1cs_path: cmd.r1cs,
            sym_path: cmd.sym,
            wasm_path: cmd.wasm,
            input_json_path: cmd.input,
            out_dir: cmd.out,
            snarkjs_bin: cmd.snarkjs_bin,
            binius_witness_path: cmd.binius_witness,
            precheck_report_path: cmd.precheck_report,
            precheck_only: cmd.precheck_only,
            backend_capabilities_path: cmd.backend_capabilities,
        }),
        Commands::Verify(cmd) => CommandAction::Verify(VerifyArgs {
            bundle_path: cmd.bundle,
        }),
        Commands::CheckWitness(cmd) => CommandAction::CheckWitness(CheckWitnessArgs {
            r1cs_path: cmd.r1cs,
            sym_path: cmd.sym,
            circom_witness_path: cmd.circom_witness,
            binius_witness_path: cmd.binius_witness,
            out_path: cmd.out,
        }),
        Commands::Benchmark => CommandAction::Benchmark,
        Commands::Explain => CommandAction::Explain,
        Commands::Doctor(cmd) => CommandAction::Doctor(DoctorArgs { out_path: cmd.out }),
        Commands::Clean => CommandAction::Clean,
    };

    let outcome = dispatch(
        action,
        &CommandContext {
            project_root: cli.project_root,
        },
    )?;

    println!("{}", outcome.message);
    Ok(())
}

fn mode_from_flag(optimized_binary: bool) -> CompileMode {
    if optimized_binary {
        CompileMode::OptimizedBinary
    } else {
        CompileMode::Compatibility
    }
}
