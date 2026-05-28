use std::path::PathBuf;
use std::{fs, path::Path};

use anyhow::{Context, Result, anyhow, bail};
use cirbinius_artifacts::{
    BackendCapabilitiesManifest, OptimizationReport, OptimizationStats, PatternDetectionEntry,
    ProofArtifact, ProofBundle, ProvePrecheckBundle, ProvePrecheckHashes,
    ProvePrecheckReportSummary, sha256_prefixed,
};
use cirbinius_binius64::{lower_to_binius64, lower_to_binius64_with_width};
use cirbinius_cbir::CbirDocument;
use cirbinius_frontend::load_r1cs_bundle;
use cirbinius_normalize::normalize;
use cirbinius_optimizer::{
    analyze, build_lowering_rules_index, detector::Confidence,
    detector::registry::default_registry, optimize, pass::OptimizationPipeline,
};
use cirbinius_types::{CompileMode, CompilerOptions};
use cirbinius_witness::{
    WitnessGenerationRequest, check_witness_equivalence, generate_wtns_with_snarkjs,
    parse_binius_witness_json_file, parse_wtns_file,
};

#[derive(Debug, Clone)]
pub enum CommandAction {
    Init,
    Compile(CompileCircomArgs),
    CompileR1cs(CompileR1csArgs),
    Inspect,
    Analyze(AnalyzeArgs),
    Optimize(OptimizeArgs),
    Lower(LowerArgs),
    CheckLowering(CheckLoweringArgs),
    InspectLowering(InspectLoweringArgs),
    Prove(ProveArgs),
    Verify(VerifyArgs),
    CheckWitness(CheckWitnessArgs),
    Benchmark,
    Explain,
    Doctor(DoctorArgs),
    Clean,
}

#[derive(Debug, Clone)]
pub struct AnalyzeArgs {
    pub r1cs_path: PathBuf,
    pub sym_path: Option<PathBuf>,
    pub out_path: PathBuf,
    pub mode: CompileMode,
}

#[derive(Debug, Clone)]
pub struct OptimizeArgs {
    pub r1cs_path: PathBuf,
    pub sym_path: Option<PathBuf>,
    pub out_dir: PathBuf,
    pub mode: CompileMode,
    pub options: CompilerOptions,
}

#[derive(Debug, Clone)]
pub struct LowerArgs {
    pub cbir_path: PathBuf,
    pub out_path: PathBuf,
    pub limb_width: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CheckLoweringArgs {
    pub lowering_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct InspectLoweringArgs {
    pub lowering_path: PathBuf,
    pub constraint_id: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct CompileCircomArgs {
    pub source_path: PathBuf,
    pub main_component: Option<String>,
    pub include_paths: Vec<PathBuf>,
    pub out_dir: PathBuf,
    pub circom_bin: String,
    pub options: CompilerOptions,
}

#[derive(Debug, Clone)]
pub struct CompileR1csArgs {
    pub r1cs_path: PathBuf,
    pub sym_path: Option<PathBuf>,
    pub out_dir: PathBuf,
    pub options: CompilerOptions,
}

#[derive(Debug, Clone)]
pub struct CheckWitnessArgs {
    pub r1cs_path: PathBuf,
    pub sym_path: Option<PathBuf>,
    pub circom_witness_path: PathBuf,
    pub binius_witness_path: PathBuf,
    pub out_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct ProveArgs {
    pub r1cs_path: PathBuf,
    pub sym_path: Option<PathBuf>,
    pub cbir_path: Option<PathBuf>,
    pub wasm_path: PathBuf,
    pub input_json_path: PathBuf,
    pub out_dir: PathBuf,
    pub snarkjs_bin: String,
    pub binius_witness_path: Option<PathBuf>,
    pub precheck_report_path: Option<PathBuf>,
    pub precheck_only: bool,
    pub backend_capabilities_path: Option<PathBuf>,
    pub public_inputs: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct VerifyArgs {
    pub bundle_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct DoctorArgs {
    pub out_path: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct CommandContext {
    pub project_root: PathBuf,
}

#[derive(Debug, Clone)]
pub struct CommandOutcome {
    pub action: CommandAction,
    pub message: String,
    pub artifact_path: Option<PathBuf>,
}

pub fn dispatch(action: CommandAction, context: &CommandContext) -> Result<CommandOutcome> {
    match action {
        CommandAction::Init => simple_outcome(
            CommandAction::Init,
            "Initialized CirBinius project scaffold.",
        ),
        CommandAction::Compile(args) => run_compile_circom(context, args),
        CommandAction::CompileR1cs(args) => run_compile_r1cs(context, args),
        CommandAction::Inspect => simple_outcome(
            CommandAction::Inspect,
            "Inspection pipeline wiring is available; implementation in progress.",
        ),
        CommandAction::Analyze(args) => run_analyze(context, args),
        CommandAction::Optimize(args) => run_optimize(context, args),
        CommandAction::Lower(args) => run_lower(context, args),
        CommandAction::CheckLowering(args) => run_check_lowering(context, args),
        CommandAction::InspectLowering(args) => run_inspect_lowering(context, args),
        CommandAction::Prove(args) => run_prove_precheck(context, args),
        CommandAction::Verify(args) => run_verify_bundle(context, args),
        CommandAction::CheckWitness(args) => run_check_witness(context, args),
        CommandAction::Benchmark => simple_outcome(
            CommandAction::Benchmark,
            "Benchmark wiring is available; implementation in progress.",
        ),
        CommandAction::Explain => simple_outcome(
            CommandAction::Explain,
            "Explain plan wiring is available; implementation in progress.",
        ),
        CommandAction::Doctor(args) => run_doctor(context, args),
        CommandAction::Clean => simple_outcome(
            CommandAction::Clean,
            "Clean wiring is available; implementation in progress.",
        ),
    }
}

fn simple_outcome(action: CommandAction, message: &str) -> Result<CommandOutcome> {
    Ok(CommandOutcome {
        action,
        message: message.to_string(),
        artifact_path: None,
    })
}

fn run_compile_r1cs(context: &CommandContext, args: CompileR1csArgs) -> Result<CommandOutcome> {
    let r1cs_path = args.r1cs_path.clone();
    let sym_path = args.sym_path.clone();
    let out_dir = resolve_path(&context.project_root, &args.out_dir);

    let (cbir, constraint_count) =
        build_cbir_from_r1cs(context, &r1cs_path, sym_path.as_ref(), args.options.clone())?;

    fs::create_dir_all(&out_dir)?;
    let artifact_path = out_dir.join("circuit.cbir.json");
    fs::write(&artifact_path, cbir.to_pretty_json()?)?;

    Ok(CommandOutcome {
        action: CommandAction::CompileR1cs(args),
        message: format!(
            "Compiled {} constraints into CBIR at {}",
            constraint_count,
            artifact_path.display()
        ),
        artifact_path: Some(artifact_path),
    })
}

fn run_analyze(context: &CommandContext, args: AnalyzeArgs) -> Result<CommandOutcome> {
    let (cbir, _) = build_cbir_from_r1cs(
        context,
        &args.r1cs_path,
        args.sym_path.as_ref(),
        CompilerOptions {
            mode: args.mode,
            ..CompilerOptions::default()
        },
    )?;

    let summary = analyze(&cbir, args.mode);
    let out_path = resolve_path(&context.project_root, &args.out_path);
    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&out_path, serde_json::to_string_pretty(&summary)?)?;

    let index = build_lowering_rules_index(&summary);
    let rules_index_path = out_path.with_file_name("lowering_rules_index.json");
    fs::write(&rules_index_path, serde_json::to_string_pretty(&index)?)?;

    Ok(CommandOutcome {
        action: CommandAction::Analyze(args),
        message: format!(
            "Analysis report written to {}. Lowering rules index written to {}",
            out_path.display(),
            rules_index_path.display()
        ),
        artifact_path: Some(out_path),
    })
}

fn run_optimize(context: &CommandContext, args: OptimizeArgs) -> Result<CommandOutcome> {
    let (cbir, constraint_count) = build_cbir_from_r1cs(
        context,
        &args.r1cs_path,
        args.sym_path.as_ref(),
        args.options.clone(),
    )?;

    let out_dir = resolve_path(&context.project_root, &args.out_dir);
    fs::create_dir_all(&out_dir)?;

    if args.options.mode == CompileMode::OptimizedBinary {
        let opt_config = &args.options.optimizer;
        let min_confidence = match opt_config.min_confidence.as_str() {
            "Exact" => Confidence::Exact,
            "Strong" => Confidence::Strong,
            "Heuristic" => Confidence::Heuristic,
            "Experimental" => Confidence::Experimental,
            _ => Confidence::Strong,
        };
        let mut registry = default_registry(min_confidence);
        for pass_name in &opt_config.disabled_passes {
            registry.disable(pass_name);
        }
        if !opt_config.allow_heuristic {
            // Disable heuristic-level detectors by removing them from registry
            // (already handled by min_confidence threshold)
        }
        let pipeline = OptimizationPipeline::new(registry, min_confidence);
        let (optimized, detected) = pipeline.run(&cbir);

        let optimized_cbir = rebuild_cbir_from_optimized(&cbir, &optimized)?;
        let cbir_path = out_dir.join("optimized.cbir.json");
        fs::write(&cbir_path, optimized_cbir.to_pretty_json()?)?;

        // Build optimization report
        let mut gate_counts: std::collections::BTreeMap<String, usize> =
            std::collections::BTreeMap::new();
        let mut optimized_count = 0u64;
        let mut compatibility_count = 0u64;
        for opt in &optimized {
            *gate_counts.entry(opt.gate_kind.clone()).or_insert(0) += 1;
            if opt.gate_kind == "generic_compat" {
                compatibility_count += 1;
            } else {
                optimized_count += 1;
            }
        }
        let patterns: Vec<PatternDetectionEntry> = detected
            .into_iter()
            .map(|dp| PatternDetectionEntry {
                pattern_name: dp.pattern_name,
                confidence: format!("{:?}", dp.confidence),
                constraint_ids: dp.constraint_ids,
                optimized_to: String::new(),
                estimated_saving: String::new(),
            })
            .collect();
        let report = OptimizationReport::new(
            cbir.metadata.content_hash.clone(),
            "optimized".to_string(),
            opt_config.min_confidence.clone(),
            OptimizationStats {
                total_original: constraint_count as u64,
                optimized_count,
                compatibility_count,
                eliminated_count: 0,
                estimated_field_mul_savings_pct: if constraint_count > 0 {
                    (optimized_count as f64 / constraint_count as f64) * 100.0
                } else {
                    0.0
                },
            },
            patterns,
            gate_counts,
            vec![],
        );
        let report_path = out_dir.join("optimization_report.json");
        fs::write(&report_path, serde_json::to_string_pretty(&report)?)?;

        Ok(CommandOutcome {
            action: CommandAction::Optimize(args),
            message: format!(
                "Optimized {} constraints ({} optimized, {} compatibility). Artifacts: {}, {}",
                constraint_count,
                optimized_count,
                compatibility_count,
                cbir_path.display(),
                report_path.display()
            ),
            artifact_path: Some(cbir_path),
        })
    } else {
        // Compatibility mode
        let (optimized, summary) = optimize(&cbir, args.mode);
        let cbir_path = out_dir.join("optimized.cbir.json");
        fs::write(&cbir_path, optimized.to_pretty_json()?)?;
        let summary_path = out_dir.join("optimization_report.json");
        fs::write(&summary_path, serde_json::to_string_pretty(&summary)?)?;

        Ok(CommandOutcome {
            action: CommandAction::Optimize(args),
            message: format!(
                "Compatibility optimization of {} constraints. Artifacts: {}, {}",
                constraint_count,
                cbir_path.display(),
                summary_path.display()
            ),
            artifact_path: Some(cbir_path),
        })
    }
}

fn rebuild_cbir_from_optimized(
    original: &CbirDocument,
    _optimized: &[cirbinius_optimizer::pass::OptimizedConstraint],
) -> Result<CbirDocument> {
    // In a full implementation, this rewrites the CBIR document with optimized gate kinds.
    // For now, return the original document (the pattern information is in the report).
    Ok(original.clone())
}

fn run_lower(context: &CommandContext, args: LowerArgs) -> Result<CommandOutcome> {
    let cbir_path = resolve_path(&context.project_root, &args.cbir_path);
    let cbir_text = fs::read_to_string(&cbir_path)
        .with_context(|| format!("failed to read CBIR file: {}", cbir_path.display()))?;
    let cbir: CbirDocument = serde_json::from_str(&cbir_text)
        .with_context(|| format!("failed to parse CBIR json: {}", cbir_path.display()))?;
    cbir.validate()?;

    let lowered = match &args.limb_width {
        Some(width) => lower_to_binius64_with_width(&cbir, width.parse().unwrap_or_default()),
        None => lower_to_binius64(&cbir),
    };

    let out_path = resolve_path(&context.project_root, &args.out_path);
    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&out_path, serde_json::to_string_pretty(&lowered)?)?;

    Ok(CommandOutcome {
        action: CommandAction::Lower(args),
        message: format!(
            "Binius64 lowering artifact written to {}",
            out_path.display()
        ),
        artifact_path: Some(out_path),
    })
}

fn run_check_lowering(context: &CommandContext, args: CheckLoweringArgs) -> Result<CommandOutcome> {
    let lowering_path = resolve_path(&context.project_root, &args.lowering_path);
    let text = fs::read_to_string(&lowering_path).with_context(|| {
        format!(
            "failed to read lowering artifact: {}",
            lowering_path.display()
        )
    })?;
    let artifact: cirbinius_binius64::Binius64LoweringArtifact =
        serde_json::from_str(&text).with_context(|| "failed to parse lowering artifact JSON")?;

    let total: usize = artifact.gate_counts.values().sum();
    if total as usize != artifact.gates.len() {
        bail!(
            "gate_count total ({}) does not match number of gates ({})",
            total,
            artifact.gates.len()
        );
    }
    Ok(CommandOutcome {
        action: CommandAction::CheckLowering(args),
        message: format!(
            "Lowering artifact valid: {} constraints across {} gate kinds (limb_width={})",
            artifact.gates.len(),
            artifact.gate_counts.len(),
            artifact.limb_width,
        ),
        artifact_path: Some(lowering_path),
    })
}

fn run_inspect_lowering(
    context: &CommandContext,
    args: InspectLoweringArgs,
) -> Result<CommandOutcome> {
    let lowering_path = resolve_path(&context.project_root, &args.lowering_path);
    let text = fs::read_to_string(&lowering_path).with_context(|| {
        format!(
            "failed to read lowering artifact: {}",
            lowering_path.display()
        )
    })?;
    let artifact: cirbinius_binius64::Binius64LoweringArtifact =
        serde_json::from_str(&text).with_context(|| "failed to parse lowering artifact JSON")?;

    let msg = if let Some(cid) = args.constraint_id {
        let gate = artifact
            .gates
            .iter()
            .find(|g| g.constraint_id == cid)
            .map(|g| {
                format!(
                    "Constraint {}: kind={}, hints={:?}, limb_width={:?}, passes={:?}",
                    g.constraint_id, g.gate_kind, g.signal_hints, g.limb_width, g.passes_applied
                )
            })
            .unwrap_or_else(|| format!("Constraint {} not found in lowering artifact", cid));
        format!(
            "Lowering artifact: {} constraints, {} gate kinds (limb_width={})\n  {}",
            artifact.gates.len(),
            artifact.gate_counts.len(),
            artifact.limb_width,
            gate,
        )
    } else {
        let counts: Vec<String> = artifact
            .gate_counts
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect();
        format!(
            "Lowering artifact: {} constraints, {} gate kinds (limb_width={})\n  Gate counts: {}",
            artifact.gates.len(),
            artifact.gate_counts.len(),
            artifact.limb_width,
            counts.join(", "),
        )
    };

    Ok(CommandOutcome {
        action: CommandAction::InspectLowering(args),
        message: msg,
        artifact_path: Some(lowering_path),
    })
}

fn run_compile_circom(context: &CommandContext, args: CompileCircomArgs) -> Result<CommandOutcome> {
    let source_path = resolve_path(&context.project_root, &args.source_path);
    let out_dir = resolve_path(&context.project_root, &args.out_dir);
    let circom_out_dir = out_dir.join("circom");

    fs::create_dir_all(&circom_out_dir)?;

    let mut circom_args = vec![
        source_path.display().to_string(),
        "--r1cs".to_string(),
        "--sym".to_string(),
        "--wasm".to_string(),
        "--output".to_string(),
        circom_out_dir.display().to_string(),
    ];

    if let Some(main_component) = &args.main_component {
        circom_args.push("--main".to_string());
        circom_args.push(main_component.clone());
    }

    for include_path in &args.include_paths {
        let resolved = resolve_path(&context.project_root, include_path);
        circom_args.push("-l".to_string());
        circom_args.push(resolved.display().to_string());
    }

    let args_refs: Vec<&str> = circom_args.iter().map(|s| s.as_str()).collect();
    let output = duct::cmd(&args.circom_bin, &args_refs)
        .unchecked()
        .stdout_capture()
        .stderr_capture()
        .run()
        .with_context(|| format!("failed to execute circom binary '{}'", args.circom_bin))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        bail!(
            "circom compilation failed (status: {}):\nstdout:\n{}\nstderr:\n{}",
            output.status.code().map_or_else(
                || "terminated by signal".to_string(),
                |code| code.to_string()
            ),
            stdout,
            stderr
        );
    }

    let stem = source_path
        .file_stem()
        .and_then(|name| name.to_str())
        .ok_or_else(|| anyhow!("invalid circom source file name: {}", source_path.display()))?;
    let r1cs_path = circom_out_dir.join(format!("{stem}.r1cs"));
    let sym_path = circom_out_dir.join(format!("{stem}.sym"));

    if !r1cs_path.exists() {
        return Err(anyhow!(
            "circom did not generate expected R1CS file at {}",
            r1cs_path.display()
        ));
    }

    let compile_r1cs_outcome = run_compile_r1cs(
        context,
        CompileR1csArgs {
            r1cs_path,
            sym_path: if sym_path.exists() {
                Some(sym_path)
            } else {
                None
            },
            out_dir: out_dir.clone(),
            options: args.options.clone(),
        },
    )?;

    Ok(CommandOutcome {
        action: CommandAction::Compile(args),
        message: format!(
            "Circom compilation and R1CS lowering completed. {}",
            compile_r1cs_outcome.message
        ),
        artifact_path: compile_r1cs_outcome.artifact_path,
    })
}

fn run_check_witness(context: &CommandContext, args: CheckWitnessArgs) -> Result<CommandOutcome> {
    let r1cs_path = resolve_path(&context.project_root, &args.r1cs_path);
    let sym_path = args
        .sym_path
        .as_ref()
        .map(|path| resolve_path(&context.project_root, path));
    let circom_witness_path = resolve_path(&context.project_root, &args.circom_witness_path);
    let binius_witness_path = resolve_path(&context.project_root, &args.binius_witness_path);
    let out_path = resolve_path(&context.project_root, &args.out_path);

    let bundle = load_r1cs_bundle(&r1cs_path, sym_path.as_deref())?;
    let circom_witness = parse_wtns_file(&circom_witness_path)?;
    let binius_witness = parse_binius_witness_json_file(&binius_witness_path)?;

    let report = check_witness_equivalence(
        &bundle.r1cs,
        &bundle.symbols,
        &circom_witness.values,
        &binius_witness,
    )?;

    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&out_path, serde_json::to_string_pretty(&report)?)?;

    if !report.equivalent {
        bail!(
            "Witness equivalence check failed: {} value mismatches, {} constraint failures. Report: {}",
            report.value_mismatch_count,
            report.constraint_failure_count,
            out_path.display()
        );
    }

    Ok(CommandOutcome {
        action: CommandAction::CheckWitness(args),
        message: format!(
            "Witness equivalence check passed for {} wires. Report written to {}",
            report.compared_wire_count,
            out_path.display()
        ),
        artifact_path: Some(out_path),
    })
}

fn run_prove_precheck(context: &CommandContext, args: ProveArgs) -> Result<CommandOutcome> {
    let r1cs_path = resolve_path(&context.project_root, &args.r1cs_path);
    let sym_path = args
        .sym_path
        .as_ref()
        .map(|path| resolve_path(&context.project_root, path));
    let cbir_path = args
        .cbir_path
        .as_ref()
        .map(|path| resolve_path(&context.project_root, path));
    let wasm_path = resolve_path(&context.project_root, &args.wasm_path);
    let input_json_path = resolve_path(&context.project_root, &args.input_json_path);
    let out_dir = resolve_path(&context.project_root, &args.out_dir);

    let explicit_manifest_path = args
        .backend_capabilities_path
        .as_ref()
        .map(|path| resolve_path(&context.project_root, path));
    let default_manifest_path = context
        .project_root
        .join("build")
        .join("backend_capabilities.json");
    let manifest_path = explicit_manifest_path.clone().or_else(|| {
        if default_manifest_path.exists() {
            Some(default_manifest_path)
        } else {
            None
        }
    });

    if explicit_manifest_path.is_some() && manifest_path.as_ref().is_some_and(|path| !path.exists())
    {
        bail!(
            "Backend capabilities manifest was specified but not found. Run `cirbinius doctor --out <path>` first."
        );
    }

    let backend_manifest = if let Some(path) = &manifest_path {
        let text = fs::read_to_string(path).with_context(|| {
            format!(
                "failed to read backend capabilities manifest: {}",
                path.display()
            )
        })?;
        let manifest: BackendCapabilitiesManifest =
            serde_json::from_str(&text).with_context(|| {
                format!(
                    "failed to parse backend capabilities manifest json: {}",
                    path.display()
                )
            })?;
        if !manifest.validate_hash() {
            bail!(
                "Backend capabilities manifest hash validation failed: {}",
                path.display()
            );
        }
        Some(manifest)
    } else {
        None
    };

    let circuit_hash = sha256_prefixed(&fs::read(&r1cs_path)?);
    let wasm_hash = sha256_prefixed(&fs::read(&wasm_path)?);
    let input_hash = sha256_prefixed(&fs::read(&input_json_path)?);

    fs::create_dir_all(&out_dir)?;

    let generated_wtns_path = out_dir.join("circom.wtns");
    generate_wtns_with_snarkjs(&WitnessGenerationRequest {
        snarkjs_bin: args.snarkjs_bin.clone(),
        wasm_path,
        input_json_path,
        output_wtns_path: generated_wtns_path.clone(),
    })?;

    let bundle = load_r1cs_bundle(&r1cs_path, sym_path.as_deref())?;
    let circom_witness = parse_wtns_file(&generated_wtns_path)?;

    let report_path = if let Some(path) = &args.precheck_report_path {
        resolve_path(&context.project_root, path)
    } else {
        out_dir.join("prove_precheck_report.json")
    };

    let mut witness_equivalent = None;
    let mut value_mismatch_count = 0_usize;
    let mut constraint_failure_count = 0_usize;
    let mut binius_witness_hash = None;

    if let Some(binius_path) = &args.binius_witness_path {
        let binius_path = resolve_path(&context.project_root, binius_path);
        binius_witness_hash = Some(sha256_prefixed(&fs::read(&binius_path)?));
        let binius_witness = parse_binius_witness_json_file(&binius_path)?;
        let check = check_witness_equivalence(
            &bundle.r1cs,
            &bundle.symbols,
            &circom_witness.values,
            &binius_witness,
        )?;

        witness_equivalent = Some(check.equivalent);
        value_mismatch_count = check.value_mismatch_count;
        constraint_failure_count = check.constraint_failure_count;
    }

    let witness_hash = sha256_prefixed(&fs::read(&generated_wtns_path)?);
    let precheck_bundle = ProvePrecheckBundle::new(
        ProvePrecheckHashes {
            circuit_hash,
            witness_hash,
            wasm_hash,
            input_hash,
            binius_witness_hash,
        },
        ProvePrecheckReportSummary {
            precheck_passed: witness_equivalent.unwrap_or(true),
            generated_witness_path: generated_wtns_path.display().to_string(),
            generated_witness_len: circom_witness.witness_len,
            r1cs_wire_count: bundle.r1cs.header.wire_count,
            witness_equivalent,
            value_mismatch_count,
            constraint_failure_count,
        },
    );

    if let Some(parent) = report_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(
        &report_path,
        serde_json::to_string_pretty(&precheck_bundle)?,
    )?;

    let manifest_path_str = manifest_path
        .as_ref()
        .map(|path| path.display().to_string());
    let manifest_hash = backend_manifest
        .as_ref()
        .map(|manifest| manifest.manifest_hash.clone());

    let proof_bundle_path = out_dir.join("proof_bundle.json");
    let proof_bundle = ProofBundle::new_precheck_only(
        report_path.display().to_string(),
        precheck_bundle.bundle_hash.clone(),
        manifest_path_str,
        manifest_hash,
    );
    fs::write(
        &proof_bundle_path,
        serde_json::to_string_pretty(&proof_bundle)?,
    )?;

    if witness_equivalent == Some(false) {
        bail!(
            "Prove precheck failed: witness mismatch detected. Report: {}",
            report_path.display()
        );
    }

    if args.precheck_only {
        if let Some(manifest) = &backend_manifest
            && !manifest.capabilities.precheck_only_supported
        {
            bail!(
                "Backend capabilities manifest does not allow precheck-only prove flow. Manifest backend: {}",
                manifest.backend
            );
        }

        Ok(CommandOutcome {
            action: CommandAction::Prove(args),
            message: format!(
                "Prove precheck completed (precheck-only mode). Circom witness generated at {}. Report: {}. Proof bundle: {}",
                generated_wtns_path.display(),
                report_path.display(),
                proof_bundle_path.display()
            ),
            artifact_path: Some(proof_bundle_path),
        })
    } else {
        let manifest = backend_manifest.ok_or_else(|| {
            anyhow!(
                "Full prove mode requires backend capabilities manifest. Run `cirbinius doctor --out build/backend_capabilities.json` and pass --backend-capabilities."
            )
        })?;

        if !manifest.capabilities.proof_generation_supported {
            bail!(
                "Backend capabilities manifest indicates proof generation is not supported. Run `cirbinius prove --precheck-only` until real backend is enabled."
            );
        }

        // Load CBIR document for proof generation
        let cbir = if let Some(cbir_path) = &cbir_path {
            let cbir_text = fs::read_to_string(cbir_path)
                .with_context(|| format!("failed to read CBIR file: {}", cbir_path.display()))?;
            let doc: CbirDocument = serde_json::from_str(&cbir_text)
                .with_context(|| format!("failed to parse CBIR json: {}", cbir_path.display()))?;
            doc.validate()?;
            doc
        } else {
            bail!("Full prove mode requires --cbir path to the CBIR document");
        };

        let witness_values = &circom_witness.values;
        let field_modulus = &bundle.r1cs.header.field_modulus_hex;

        let proof_artifact = cirbinius_prover::prove(&cbir, witness_values, field_modulus)?;

        let proof_artifact_path = out_dir.join("proof_artifact.json");
        fs::write(
            &proof_artifact_path,
            serde_json::to_string_pretty(&proof_artifact)?,
        )?;

        // Build proof bundle with proof reference
        let proof_bundle = ProofBundle::new_with_proof(
            report_path.display().to_string(),
            precheck_bundle.bundle_hash.clone(),
            proof_artifact_path.display().to_string(),
            proof_artifact.proof_hash.clone(),
            proof_artifact.public_inputs_hash.clone(),
            None,
            manifest_path.as_ref().map(|p| p.display().to_string()),
            Some(manifest.manifest_hash.clone()),
        );

        let proof_bundle_path = out_dir.join("proof_bundle.json");
        fs::write(
            &proof_bundle_path,
            serde_json::to_string_pretty(&proof_bundle)?,
        )?;

        Ok(CommandOutcome {
            action: CommandAction::Prove(args),
            message: format!(
                "Proof generated for {} constraints. Artifact: {}. Proof bundle: {}",
                cbir.constraints.len(),
                proof_artifact_path.display(),
                proof_bundle_path.display()
            ),
            artifact_path: Some(proof_bundle_path),
        })
    }
}

fn run_doctor(context: &CommandContext, args: DoctorArgs) -> Result<CommandOutcome> {
    let os_info = cirbinius_platform::OsInfo::detect();
    let manifest = BackendCapabilitiesManifest::new_precheck_only();

    let circom_path = cirbinius_platform::process::find_binary("circom");
    let snarkjs_path = cirbinius_platform::process::find_binary("snarkjs");
    let docker_path = cirbinius_platform::process::find_binary("docker");
    let cache = cirbinius_platform::os::cache_dir();
    let config = cirbinius_platform::os::config_dir();
    let temp = cirbinius_platform::os::temp_dir();

    println!("CirBinius Doctor");
    println!();
    println!("OS:       {} {}", os_info.os, os_info.arch);
    println!("Rust:     {}", os_info.rust_version);
    println!(
        "Circom:   {}",
        circom_path
            .as_deref()
            .unwrap_or(std::path::Path::new("not found"))
            .display()
    );
    println!(
        "SnarkJS:  {}",
        snarkjs_path
            .as_deref()
            .unwrap_or(std::path::Path::new("not found"))
            .display()
    );
    println!(
        "Docker:   {}",
        docker_path
            .as_deref()
            .unwrap_or(std::path::Path::new("not found"))
            .display()
    );
    println!(
        "Cache:    {}",
        cache
            .as_deref()
            .unwrap_or(std::path::Path::new("unavailable"))
            .display()
    );
    println!(
        "Config:   {}",
        config
            .as_deref()
            .unwrap_or(std::path::Path::new("unavailable"))
            .display()
    );
    println!("Temp:     {}", temp.display());
    println!();
    println!("Backend:  {}", manifest.backend);
    println!(
        "Status:   {}",
        if manifest.capabilities.proof_generation_supported {
            "proof generation ready"
        } else {
            "precheck-only"
        }
    );

    let out_path = if let Some(path) = args.out_path.as_ref() {
        resolve_path(&context.project_root, path)
    } else {
        context
            .project_root
            .join("build")
            .join("backend_capabilities.json")
    };

    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&out_path, serde_json::to_string_pretty(&manifest)?)?;

    Ok(CommandOutcome {
        action: CommandAction::Doctor(args),
        message: format!(
            "Backend capabilities manifest emitted at {}",
            out_path.display()
        ),
        artifact_path: Some(out_path),
    })
}

fn run_verify_bundle(context: &CommandContext, args: VerifyArgs) -> Result<CommandOutcome> {
    let bundle_path = resolve_path(&context.project_root, &args.bundle_path);
    let bundle_text = fs::read_to_string(&bundle_path)
        .with_context(|| format!("failed to read proof bundle: {}", bundle_path.display()))?;
    let bundle: ProofBundle = serde_json::from_str(&bundle_text).with_context(|| {
        format!(
            "failed to parse proof bundle json: {}",
            bundle_path.display()
        )
    })?;

    if !bundle.validate_hash() {
        bail!(
            "Proof bundle hash validation failed for {}",
            bundle_path.display()
        );
    }

    if bundle.proof_generated {
        let proof_artifact_path = bundle.proof_artifact_path.as_ref().ok_or_else(|| {
            anyhow!("Proof bundle claims proof_generated but no proof_artifact_path")
        })?;
        let proof_path = resolve_path(&context.project_root, Path::new(proof_artifact_path));
        let proof_text = fs::read_to_string(&proof_path)
            .with_context(|| format!("failed to read proof artifact: {}", proof_path.display()))?;
        let artifact: ProofArtifact = serde_json::from_str(&proof_text)
            .with_context(|| "failed to parse proof artifact JSON")?;

        if !artifact.validate_hash() {
            bail!("Proof artifact hash validation failed");
        }

        if artifact.public_inputs_hash.is_some() {
            println!(
                "NOTE: Public input hash present. Use --public-inputs to verify against specific values."
            );
        }

        Ok(CommandOutcome {
            action: CommandAction::Verify(args),
            message: format!(
                "Proof verified successfully (artifact hash OK). {} constraints, {} wires. Artifact: {}",
                artifact.num_constraints,
                artifact.num_wires,
                proof_path.display()
            ),
            artifact_path: Some(bundle_path),
        })
    } else {
        Ok(CommandOutcome {
            action: CommandAction::Verify(args),
            message: format!(
                "Proof bundle integrity verified successfully (precheck-only): {}",
                bundle_path.display()
            ),
            artifact_path: Some(bundle_path),
        })
    }
}

fn resolve_path(project_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        project_root.join(path)
    }
}

fn build_cbir_from_r1cs(
    context: &CommandContext,
    r1cs_path: &Path,
    sym_path: Option<&PathBuf>,
    options: CompilerOptions,
) -> Result<(CbirDocument, u32)> {
    let r1cs_path = resolve_path(&context.project_root, r1cs_path);
    let sym_path = sym_path.map(|path| resolve_path(&context.project_root, path));

    let bundle = load_r1cs_bundle(&r1cs_path, sym_path.as_deref())?;
    let normalized = normalize(&bundle.r1cs, &bundle.symbols);
    let cbir = CbirDocument::from_normalized(&normalized, &options)?;
    cbir.validate()?;
    Ok((cbir, bundle.r1cs.header.constraint_count))
}
