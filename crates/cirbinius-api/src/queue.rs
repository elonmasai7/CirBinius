use sha2::{Digest, Sha256};

use crate::uuid::Uuid;

use tokio::sync::broadcast;

use crate::config::Config;
use crate::sandbox;
use crate::state::{self as st, SharedStore};

static JOB_SENDER: once_cell::sync::OnceCell<broadcast::Sender<JobMessage>> = once_cell::sync::OnceCell::new();

#[derive(Debug, Clone)]
pub struct JobMessage {
    pub job_id: Uuid,
    pub project_id: Uuid,
    pub job_type: String,
    pub params: serde_json::Value,
}

pub fn enqueue_job(job_id: Uuid, project_id: Uuid, job_type: &str, params: &serde_json::Value) {
    if let Some(sender) = JOB_SENDER.get() {
        let msg = JobMessage {
            job_id,
            project_id,
            job_type: job_type.to_string(),
            params: params.clone(),
        };
        let _ = sender.send(msg);
    }
}

pub fn init_queue(store: SharedStore, config: Config) {
    let (tx, _rx) = broadcast::channel::<JobMessage>(1024);
    JOB_SENDER.set(tx).ok();

    let worker_count = config.worker_count;
    for id in 0..worker_count {
        let store = store.clone();
        let config = config.clone();
        tokio::spawn(async move {
            let rx = JOB_SENDER.get().unwrap().subscribe();
            let worker = Worker::new(id, store, config, rx);
            worker.run().await;
        });
    }
}

struct Worker {
    id: usize,
    store: SharedStore,
    config: Config,
    rx: broadcast::Receiver<JobMessage>,
}

impl Worker {
    fn new(id: usize, store: SharedStore, config: Config, rx: broadcast::Receiver<JobMessage>) -> Self {
        Self { id, store, config, rx }
    }

    async fn run(mut self) {
        while let Ok(msg) = self.rx.recv().await {
            self.process_job(msg).await;
        }
    }

    async fn process_job(&self, msg: JobMessage) {
        st::append_job_log(&self.store, msg.job_id, "info", &format!("worker {} processing job", self.id));
        st::update_job_status(&self.store, msg.job_id, "running", None, None, None);

        let result = match msg.job_type.as_str() {
            "compile" => handle_compile(&msg, &self.store, &self.config).await,
            "prove" => handle_prove(&msg, &self.store, &self.config).await,
            "verify" => handle_verify(&msg, &self.store).await,
            "analyze" => handle_analyze(&msg, &self.store, &self.config).await,
            "conformance" => handle_conformance(&msg, &self.store, &self.config).await,
            other => Err(format!("unknown job type: {other}")),
        };

        match result {
            Ok(output) => {
                st::update_job_status(
                    &self.store,
                    msg.job_id,
                    "succeeded",
                    None,
                    Some(&output),
                    Some(100),
                );
                st::append_job_log(&self.store, msg.job_id, "info", &format!("job completed: {}", output["message"]));
            }
            Err(e) => {
                st::update_job_status(&self.store, msg.job_id, "failed", Some(&e), None, None);
                st::append_job_log(&self.store, msg.job_id, "error", &e);
            }
        }
    }
}

async fn handle_compile(msg: &JobMessage, store: &SharedStore, config: &Config) -> Result<serde_json::Value, String> {
    st::append_job_log(store, msg.job_id, "info", "starting compile job");

    let r1cs_upload_id = msg.params.get("r1cs_upload_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or_else(|| "missing r1cs_upload_id".to_string())?;

    let r1cs_upload = st::get_upload(store, r1cs_upload_id)
        .ok_or_else(|| format!("upload {r1cs_upload_id} not found"))?;
    let r1cs_data = std::fs::read(&r1cs_upload.storage_path)
        .map_err(|e| format!("failed to read r1cs: {e}"))?;

    let sym_upload_id = msg.params.get("sym_upload_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok());
    let sym_data = if let Some(id) = sym_upload_id {
        let upload = st::get_upload(store, id)
            .ok_or_else(|| format!("upload {id} not found"))?;
        Some(std::fs::read(&upload.storage_path).map_err(|e| format!("failed to read sym: {e}"))?)
    } else {
        None
    };

    // Create sandboxed temp dir for work
    let tmp_dir = sandbox::create_sandboxed_work_dir(config).map_err(|e| format!("sandbox tempdir: {e}"))?;
    let work_dir = tmp_dir.path().to_path_buf();

    // Write inputs
    let r1cs_path = work_dir.join("input.r1cs");
    std::fs::write(&r1cs_path, &r1cs_data).map_err(|e| format!("write: {e}"))?;
    let sym_path = if let Some(ref data) = sym_data {
        let p = work_dir.join("input.sym");
        std::fs::write(&p, data).map_err(|e| format!("write: {e}"))?;
        Some(p)
    } else {
        None
    };

    let ctx = cirbinius_core::CommandContext { project_root: work_dir.clone() };
    let args = cirbinius_core::CompileR1csArgs {
        r1cs_path: r1cs_path.clone(),
        sym_path: sym_path.clone(),
        out_dir: work_dir.clone(),
        options: cirbinius_types::CompilerOptions::default(),
    };

    let outcome = cirbinius_core::dispatch(
        cirbinius_core::CommandAction::CompileR1cs(args),
        &ctx,
    ).map_err(|e| format!("compile error: {e}"))?;

    // Read CBIR output
    let cbir_path = outcome.artifact_path.as_ref()
        .ok_or_else(|| "no artifact output".to_string())?;
    let cbir_data = std::fs::read(cbir_path)
        .map_err(|e| format!("read cbir: {e}"))?;
    let cbir_hash = hex::encode(Sha256::digest(&cbir_data));

    // Store artifact
    let artifact_dir = &config.artifact_dir;
    std::fs::create_dir_all(artifact_dir).map_err(|e| format!("artifact dir: {e}"))?;
    let artifact_filename = format!("compile_{}.cbir.json", msg.job_id);
    let artifact_path = artifact_dir.join(&artifact_filename);
    std::fs::write(&artifact_path, &cbir_data).map_err(|e| format!("artifact write: {e}"))?;

    st::create_artifact(
        store,
        msg.job_id,
        "cbir",
        &artifact_filename,
        cbir_data.len() as i64,
        &artifact_path.to_string_lossy(),
        &cbir_hash,
    );

    Ok(serde_json::json!({
        "message": outcome.message,
        "artifact": artifact_filename,
    }))
}

async fn handle_prove(msg: &JobMessage, store: &SharedStore, config: &Config) -> Result<serde_json::Value, String> {
    st::append_job_log(store, msg.job_id, "info", "starting prove job");

    let r1cs_upload_id = msg.params.get("r1cs_upload_id")
        .and_then(|v| v.as_str()).and_then(|s| Uuid::parse_str(s).ok())
        .ok_or_else(|| "missing r1cs_upload_id".to_string())?;
    let wasm_upload_id = msg.params.get("wasm_upload_id")
        .and_then(|v| v.as_str()).and_then(|s| Uuid::parse_str(s).ok())
        .ok_or_else(|| "missing wasm_upload_id".to_string())?;
    let input_upload_id = msg.params.get("input_upload_id")
        .and_then(|v| v.as_str()).and_then(|s| Uuid::parse_str(s).ok())
        .ok_or_else(|| "missing input_upload_id".to_string())?;

    let r1cs = st::get_upload(store, r1cs_upload_id).ok_or_else(|| "r1cs not found".to_string())?;
    let wasm = st::get_upload(store, wasm_upload_id).ok_or_else(|| "wasm not found".to_string())?;
    let input = st::get_upload(store, input_upload_id).ok_or_else(|| "input not found".to_string())?;

    let r1cs_data = std::fs::read(&r1cs.storage_path).map_err(|e| format!("read: {e}"))?;
    let wasm_data = std::fs::read(&wasm.storage_path).map_err(|e| format!("read: {e}"))?;
    let input_data = std::fs::read(&input.storage_path).map_err(|e| format!("read: {e}"))?;

    let tmp_dir = sandbox::create_sandboxed_work_dir(config).map_err(|e| format!("sandbox tempdir: {e}"))?;
    let work_dir = tmp_dir.path().to_path_buf();

    let r1cs_path = work_dir.join("circuit.r1cs");
    let wasm_path = work_dir.join("circuit.wasm");
    let input_path = work_dir.join("input.json");

    std::fs::write(&r1cs_path, &r1cs_data).map_err(|e| format!("write: {e}"))?;
    std::fs::write(&wasm_path, &wasm_data).map_err(|e| format!("write: {e}"))?;
    std::fs::write(&input_path, &input_data).map_err(|e| format!("write: {e}"))?;

    // Build CBIR from R1CS
    let bundle = cirbinius_frontend::load_r1cs_bundle(&r1cs_path, None)
        .map_err(|e| format!("r1cs load: {e}"))?;
    let normalized = cirbinius_normalize::normalize(&bundle.r1cs, &bundle.symbols);
    let cbir = cirbinius_cbir::CbirDocument::from_normalized(
        &normalized,
        &cirbinius_types::CompilerOptions::default(),
    ).map_err(|e| format!("cbir build: {e}"))?;

    // Generate witness using snarkjs
    let wtns_path = work_dir.join("proof.wtns");
    let snarkjs_bin = msg.params.get("snarkjs_bin")
        .and_then(|v| v.as_str())
        .unwrap_or(&config.snarkjs_bin);

    let witness_req = cirbinius_witness::WitnessGenerationRequest {
        snarkjs_bin: snarkjs_bin.to_string(),
        wasm_path: wasm_path.clone(),
        input_json_path: input_path.clone(),
        output_wtns_path: wtns_path.clone(),
    };
    cirbinius_witness::generate_wtns_with_snarkjs(&witness_req)
        .map_err(|e| format!("witness gen: {e}"))?;

    // Generate proof
    let circom_witness = cirbinius_witness::parse_wtns_file(&wtns_path)
        .map_err(|e| format!("wtns parse: {e}"))?;

    let proof = cirbinius_prover::prove(
        &cbir,
        &circom_witness.values,
        &bundle.r1cs.header.field_modulus_hex,
    ).map_err(|e| format!("prove: {e}"))?;

    // Store artifacts
    let artifact_dir = &config.artifact_dir;
    std::fs::create_dir_all(artifact_dir).map_err(|e| format!("dir: {e}"))?;

    let proof_filename = format!("proof_{}.json", msg.job_id);
    let proof_path = artifact_dir.join(&proof_filename);
    let proof_data = serde_json::to_string_pretty(&proof).map_err(|e| format!("serialize: {e}"))?;
    std::fs::write(&proof_path, &proof_data).map_err(|e| format!("write: {e}"))?;

    let proof_hash = hex::encode(Sha256::digest(proof_data.as_bytes()));
    st::create_artifact(
        store, msg.job_id, "proof", &proof_filename,
        proof_data.len() as i64, &proof_path.to_string_lossy(), &proof_hash,
    );

    Ok(serde_json::json!({
        "message": format!("Proof generated for {} constraints", cbir.constraints.len()),
        "proof": proof_filename,
    }))
}

async fn handle_verify(msg: &JobMessage, store: &SharedStore) -> Result<serde_json::Value, String> {
    st::append_job_log(store, msg.job_id, "info", "starting verify job");

    let bundle_upload_id = msg.params.get("bundle_upload_id")
        .and_then(|v| v.as_str()).and_then(|s| Uuid::parse_str(s).ok())
        .ok_or_else(|| "missing bundle_upload_id".to_string())?;

    let upload = st::get_upload(store, bundle_upload_id)
        .ok_or_else(|| format!("upload {bundle_upload_id} not found"))?;
    let data = std::fs::read(&upload.storage_path).map_err(|e| format!("read: {e}"))?;
    let bundle: cirbinius_artifacts::ProofBundle = serde_json::from_slice(&data)
        .map_err(|e| format!("parse: {e}"))?;

    if !bundle.validate_hash() {
        return Err("proof bundle hash validation failed".to_string());
    }

    let outcome = cirbinius_core::dispatch(
        cirbinius_core::CommandAction::Verify(cirbinius_core::VerifyArgs {
            bundle_path: std::path::PathBuf::from(&upload.storage_path),
        }),
        &cirbinius_core::CommandContext { project_root: std::path::PathBuf::from("/tmp") },
    ).map_err(|e| format!("verify: {e}"))?;

    Ok(serde_json::json!({
        "message": outcome.message,
    }))
}

async fn handle_analyze(msg: &JobMessage, store: &SharedStore, config: &Config) -> Result<serde_json::Value, String> {
    st::append_job_log(store, msg.job_id, "info", "starting analyze job");

    let r1cs_upload_id = msg.params.get("r1cs_upload_id")
        .and_then(|v| v.as_str()).and_then(|s| Uuid::parse_str(s).ok())
        .ok_or_else(|| "missing r1cs_upload_id".to_string())?;

    let r1cs = st::get_upload(store, r1cs_upload_id).ok_or_else(|| "r1cs not found".to_string())?;
    let r1cs_data = std::fs::read(&r1cs.storage_path).map_err(|e| format!("read: {e}"))?;

    let tmp_dir = sandbox::create_sandboxed_work_dir(config).map_err(|e| format!("sandbox tempdir: {e}"))?;
    let work_dir = tmp_dir.path().to_path_buf();
    let r1cs_path = work_dir.join("input.r1cs");
    std::fs::write(&r1cs_path, &r1cs_data).map_err(|e| format!("write: {e}"))?;

    let out_path = work_dir.join("analysis.json");
    let ctx = cirbinius_core::CommandContext { project_root: work_dir.clone() };
    let outcome = cirbinius_core::dispatch(
        cirbinius_core::CommandAction::Analyze(cirbinius_core::AnalyzeArgs {
            r1cs_path: r1cs_path.clone(),
            sym_path: None,
            out_path: out_path.clone(),
            mode: cirbinius_types::CompileMode::Compatibility,
        }),
        &ctx,
    ).map_err(|e| format!("analyze: {e}"))?;

    let analysis_data = std::fs::read(&out_path).map_err(|e| format!("read: {e}"))?;
    let artifact_dir = &config.artifact_dir;
    std::fs::create_dir_all(artifact_dir).map_err(|e| format!("dir: {e}"))?;
    let filename = format!("analysis_{}.json", msg.job_id);
    let store_path = artifact_dir.join(&filename);
    std::fs::write(&store_path, &analysis_data).map_err(|e| format!("write: {e}"))?;

    let hash = hex::encode(Sha256::digest(&analysis_data));
    st::create_artifact(store, msg.job_id, "analysis", &filename, analysis_data.len() as i64, &store_path.to_string_lossy(), &hash);

    Ok(serde_json::json!({
        "message": outcome.message,
    }))
}

async fn handle_conformance(msg: &JobMessage, store: &SharedStore, config: &Config) -> Result<serde_json::Value, String> {
    st::append_job_log(store, msg.job_id, "info", "starting conformance run");

    let test_categories: Vec<String> = msg.params.get("test_categories")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_else(|| vec!["compile".to_string(), "analyze".to_string()]);

    let fixtures_dir = &config.conformance_fixtures_dir;
    if !fixtures_dir.exists() {
        return Err(format!("fixtures directory not found: {}", fixtures_dir.display()));
    }

    let mut results = Vec::new();
    let mut passed = 0u32;
    let mut failed = 0u32;

    for category in &test_categories {
        let cat_dir = fixtures_dir.join(category);
        if !cat_dir.is_dir() {
            st::append_job_log(store, msg.job_id, "warn", &format!("category dir not found: {cat_dir:?}"));
            continue;
        }

        let entries = match std::fs::read_dir(&cat_dir) {
            Ok(e) => e,
            Err(e) => {
                st::append_job_log(store, msg.job_id, "error", &format!("read dir: {e}"));
                continue;
            }
        };

        for entry in entries {
            let entry = match entry {
                Ok(e) => e,
                _ => continue,
            };
            let path = entry.path();
            if path.extension().map_or(true, |ext| ext != "r1cs") {
                continue;
            }

            let result = run_conformance_test(&path, category).await;
            passed += result.passed as u32;
            failed += result.failed as u32;
            results.push(result.report);
        }
    }

    let report = serde_json::json!({
        "summary": { "passed": passed, "failed": failed, "total": passed + failed },
        "results": results,
    });

    let report_data = serde_json::to_string_pretty(&report).map_err(|e| format!("serialize: {e}"))?;
    let artifact_dir = &config.artifact_dir;
    std::fs::create_dir_all(artifact_dir).map_err(|e| format!("dir: {e}"))?;
    let filename = format!("conformance_{}.json", msg.job_id);
    let store_path = artifact_dir.join(&filename);
    std::fs::write(&store_path, &report_data).map_err(|e| format!("write: {e}"))?;

    let hash = hex::encode(Sha256::digest(report_data.as_bytes()));
    st::create_artifact(store, msg.job_id, "conformance_report", &filename, report_data.len() as i64, &store_path.to_string_lossy(), &hash);

    Ok(serde_json::json!({
        "message": format!("Conformance: {passed} passed, {failed} failed"),
        "report": filename,
    }))
}

struct TestResult {
    passed: bool,
    failed: bool,
    report: serde_json::Value,
}

async fn run_conformance_test(fixture_path: &std::path::Path, category: &str) -> TestResult {
    let ctx = cirbinius_core::CommandContext {
        project_root: fixture_path.parent().unwrap().to_path_buf(),
    };

    let outcome = match category {
        "compile" => {
            cirbinius_core::dispatch(
                cirbinius_core::CommandAction::CompileR1cs(cirbinius_core::CompileR1csArgs {
                    r1cs_path: fixture_path.to_path_buf(),
                    sym_path: None,
                    out_dir: std::path::PathBuf::from("/tmp/cirbinius-conformance"),
                    options: cirbinius_types::CompilerOptions::default(),
                }),
                &ctx,
            )
        }
        "analyze" => {
            let out_path = std::path::PathBuf::from("/tmp/cirbinius-conformance-analysis.json");
            cirbinius_core::dispatch(
                cirbinius_core::CommandAction::Analyze(cirbinius_core::AnalyzeArgs {
                    r1cs_path: fixture_path.to_path_buf(),
                    sym_path: None,
                    out_path,
                    mode: cirbinius_types::CompileMode::Compatibility,
                }),
                &ctx,
            )
        }
        _ => return TestResult {
            passed: false,
            failed: true,
            report: serde_json::json!({
                "fixture": fixture_path.to_string_lossy(),
                "category": category,
                "passed": false,
                "error": format!("unknown category: {category}"),
            }),
        },
    };

    match outcome {
        Ok(o) => TestResult {
            passed: true,
            failed: false,
            report: serde_json::json!({
                "fixture": fixture_path.to_string_lossy(),
                "category": category,
                "passed": true,
                "message": o.message,
            }),
        },
        Err(e) => TestResult {
            passed: false,
            failed: true,
            report: serde_json::json!({
                "fixture": fixture_path.to_string_lossy(),
                "category": category,
                "passed": false,
                "error": e.to_string(),
            }),
        },
    }
}
