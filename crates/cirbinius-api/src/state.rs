use crate::uuid::Uuid;
use std::collections::HashMap;

use std::sync::Arc;

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Upload {
    pub id: Uuid,
    pub project_id: Uuid,
    pub filename: String,
    pub file_size: i64,
    pub content_type: String,
    pub storage_path: String,
    pub file_hash: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: Uuid,
    pub project_id: Uuid,
    pub job_type: String,
    pub status: String,
    pub params: serde_json::Value,
    pub result: Option<serde_json::Value>,
    pub error_message: Option<String>,
    pub progress_pct: Option<i32>,
    pub created_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    pub id: Uuid,
    pub job_id: Uuid,
    pub artifact_type: String,
    pub filename: String,
    pub file_size: i64,
    pub storage_path: String,
    pub content_hash: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobLog {
    pub id: Uuid,
    pub job_id: Uuid,
    pub level: String,
    pub message: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub id: Uuid,
    pub key_prefix: String,
    pub key_hash: String,
    pub name: String,
    pub project_id: Option<Uuid>,
    pub permissions: Vec<String>,
    pub expires_at: Option<String>,
    pub created_at: String,
}

fn now_iso() -> String {
    // Simple ISO 8601 timestamp without chrono
    let dur = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = dur.as_secs();
    let nanos = dur.subsec_nanos();
    // Format: YYYY-MM-DDTHH:MM:SS.fffffffffZ
    let days = secs / 86400;
    let time_secs = secs % 86400;
    let hours = time_secs / 3600;
    let minutes = (time_secs % 3600) / 60;
    let seconds = time_secs % 60;

    // Calculate date from days since epoch (1970-01-01)
    let (year, month, day) = days_to_date(days as i64);

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:09}Z",
        year, month, day, hours, minutes, seconds, nanos
    )
}

fn days_to_date(mut days: i64) -> (i64, i64, i64) {
    days += 719468;
    let era = if days >= 0 { days } else { days - 146096 };
    let era_days = era.rem_euclid(146097);
    let year_era = (era_days - era_days / 1460 + era_days / 36524 - era_days / 146096) / 365;
    let y = year_era + era / 146097 * 400;
    let doy = era_days - (365 * year_era + year_era / 4 - year_era / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    let y = if y <= 0 { y - 1 } else { y };
    (y, m, d)
}

fn compute_hash(data: &[u8]) -> String {
    hex::encode(Sha256::digest(data))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Store {
    projects: HashMap<Uuid, Project>,
    uploads: HashMap<Uuid, Upload>,
    uploads_by_project: HashMap<Uuid, Vec<Uuid>>,
    jobs: HashMap<Uuid, Job>,
    jobs_by_project: HashMap<Uuid, Vec<Uuid>>,
    artifacts: HashMap<Uuid, Artifact>,
    artifacts_by_job: HashMap<Uuid, Vec<Uuid>>,
    job_logs: HashMap<Uuid, Vec<JobLog>>,
    api_keys: HashMap<String, ApiKey>,
    api_keys_by_id: HashMap<Uuid, ApiKey>,
}

impl Store {
    pub fn new() -> Self {
        Self {
            projects: HashMap::new(),
            uploads: HashMap::new(),
            uploads_by_project: HashMap::new(),
            jobs: HashMap::new(),
            jobs_by_project: HashMap::new(),
            artifacts: HashMap::new(),
            artifacts_by_job: HashMap::new(),
            job_logs: HashMap::new(),
            api_keys: HashMap::new(),
            api_keys_by_id: HashMap::new(),
        }
    }
}

pub type SharedStore = Arc<RwLock<Store>>;

impl Store {
    pub fn save_snapshot(&self, path: &std::path::Path) -> Result<(), String> {
        let data = serde_json::to_string(self).map_err(|e| format!("serialize: {e}"))?;
        std::fs::write(path, &data).map_err(|e| format!("write: {e}"))
    }

    pub fn load_snapshot(path: &std::path::Path) -> Option<Self> {
        let data = std::fs::read(path).ok()?;
        serde_json::from_slice(&data).ok()
    }
}

pub fn create_store() -> SharedStore {
    let snapshot_path = std::path::Path::new("cirbinius_state.json");
    let store = if let Some(s) = Store::load_snapshot(snapshot_path) {
        eprintln!("cirbinius-api: loaded state from snapshot");
        s
    } else {
        Store::new()
    };
    Arc::new(RwLock::new(store))
}

pub fn spawn_snapshot_task(store: SharedStore) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
        loop {
            interval.tick().await;
            let s = store.read();
            let _ = s.save_snapshot(std::path::Path::new("cirbinius_state.json"));
        }
    });
}

// Projects

pub fn create_project(store: &SharedStore, name: &str, description: Option<&str>) -> Project {
    let mut s = store.write();
    let now = now_iso();
    let project = Project {
        id: Uuid::new_v4(),
        name: name.to_string(),
        description: description.map(|s| s.to_string()),
        status: "active".into(),
        created_at: now.clone(),
        updated_at: now,
    };
    s.projects.insert(project.id, project.clone());
    project
}

pub fn list_projects(store: &SharedStore) -> Vec<Project> {
    let s = store.read();
    let mut items: Vec<_> = s.projects.values().cloned().collect();
    items.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    items
}

pub fn get_project(store: &SharedStore, id: Uuid) -> Option<Project> {
    store.read().projects.get(&id).cloned()
}

pub fn update_project(store: &SharedStore, id: Uuid, name: Option<&str>, description: Option<&str>, status: Option<&str>) -> Option<Project> {
    let mut s = store.write();
    let project = s.projects.get_mut(&id)?;
    if let Some(n) = name { project.name = n.to_string(); }
    if let Some(d) = description { project.description = Some(d.to_string()); }
    if let Some(st) = status { project.status = st.to_string(); }
    project.updated_at = now_iso();
    Some(project.clone())
}

pub fn delete_project(store: &SharedStore, id: Uuid) -> bool {
    let mut s = store.write();
    s.projects.remove(&id).is_some()
}

// Uploads

pub fn create_upload(store: &SharedStore, project_id: Uuid, filename: &str, file_size: i64, content_type: &str, storage_path: &str, file_hash: &str) -> Upload {
    let mut s = store.write();
    let now = now_iso();
    let upload = Upload {
        id: Uuid::new_v4(),
        project_id,
        filename: filename.to_string(),
        file_size,
        content_type: content_type.to_string(),
        storage_path: storage_path.to_string(),
        file_hash: file_hash.to_string(),
        created_at: now,
    };
    s.uploads.insert(upload.id, upload.clone());
    s.uploads_by_project.entry(project_id).or_default().push(upload.id);
    upload
}

pub fn list_uploads(store: &SharedStore, project_id: Uuid) -> Vec<Upload> {
    let s = store.read();
    let mut items: Vec<Upload> = s.uploads_by_project.get(&project_id)
        .map(|ids| ids.iter().filter_map(|id| s.uploads.get(id).cloned()).collect())
        .unwrap_or_default();
    items.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    items
}

pub fn get_upload(store: &SharedStore, id: Uuid) -> Option<Upload> {
    store.read().uploads.get(&id).cloned()
}

pub fn delete_upload(store: &SharedStore, id: Uuid) -> bool {
    let mut s = store.write();
    let upload = s.uploads.remove(&id);
    if let Some(ref u) = upload {
        if let Some(ids) = s.uploads_by_project.get_mut(&u.project_id) {
            ids.retain(|i| *i != id);
        }
    }
    upload.is_some()
}

// Jobs

pub fn create_job(store: &SharedStore, project_id: Uuid, job_type: &str, params: &serde_json::Value) -> Job {
    let mut s = store.write();
    let now = now_iso();
    let job = Job {
        id: Uuid::new_v4(),
        project_id,
        job_type: job_type.to_string(),
        status: "queued".into(),
        params: params.clone(),
        result: None,
        error_message: None,
        progress_pct: None,
        created_at: now,
        started_at: None,
        completed_at: None,
    };
    s.jobs.insert(job.id, job.clone());
    s.jobs_by_project.entry(project_id).or_default().push(job.id);
    job
}

pub fn list_jobs(store: &SharedStore, project_id: Uuid) -> Vec<Job> {
    let s = store.read();
    let mut items: Vec<Job> = s.jobs_by_project.get(&project_id)
        .map(|ids| ids.iter().filter_map(|id| s.jobs.get(id).cloned()).collect())
        .unwrap_or_default();
    items.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    items
}

pub fn get_job(store: &SharedStore, id: Uuid) -> Option<Job> {
    store.read().jobs.get(&id).cloned()
}

pub fn update_job_status(
    store: &SharedStore,
    id: Uuid,
    status: &str,
    error_message: Option<&str>,
    result: Option<&serde_json::Value>,
    progress_pct: Option<i32>,
) -> Option<Job> {
    let mut s = store.write();
    let job = s.jobs.get_mut(&id)?;
    job.status = status.to_string();
    if let Some(msg) = error_message { job.error_message = Some(msg.to_string()); }
    if let Some(res) = result { job.result = Some(res.clone()); }
    if let Some(pct) = progress_pct { job.progress_pct = Some(pct); }
    if status == "running" && job.started_at.is_none() {
        job.started_at = Some(now_iso());
    }
    if matches!(status, "succeeded" | "failed" | "cancelled") {
        job.completed_at = Some(now_iso());
    }
    Some(job.clone())
}

// Artifacts

pub fn create_artifact(store: &SharedStore, job_id: Uuid, artifact_type: &str, filename: &str, file_size: i64, storage_path: &str, content_hash: &str) -> Artifact {
    let mut s = store.write();
    let now = now_iso();
    let artifact = Artifact {
        id: Uuid::new_v4(),
        job_id,
        artifact_type: artifact_type.to_string(),
        filename: filename.to_string(),
        file_size,
        storage_path: storage_path.to_string(),
        content_hash: content_hash.to_string(),
        created_at: now,
    };
    s.artifacts.insert(artifact.id, artifact.clone());
    s.artifacts_by_job.entry(job_id).or_default().push(artifact.id);
    artifact
}

pub fn list_artifacts(store: &SharedStore, job_id: Uuid) -> Vec<Artifact> {
    let s = store.read();
    let mut items: Vec<Artifact> = s.artifacts_by_job.get(&job_id)
        .map(|ids| ids.iter().filter_map(|id| s.artifacts.get(id).cloned()).collect())
        .unwrap_or_default();
    items.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    items
}

pub fn get_artifact(store: &SharedStore, id: Uuid) -> Option<Artifact> {
    store.read().artifacts.get(&id).cloned()
}

// Job logs

pub fn append_job_log(store: &SharedStore, job_id: Uuid, level: &str, message: &str) -> JobLog {
    let mut s = store.write();
    let now = now_iso();
    let log = JobLog {
        id: Uuid::new_v4(),
        job_id,
        level: level.to_string(),
        message: message.to_string(),
        created_at: now,
    };
    s.job_logs.entry(job_id).or_default().push(log.clone());
    log
}

pub fn get_job_logs(store: &SharedStore, job_id: Uuid) -> Vec<JobLog> {
    store.read().job_logs.get(&job_id).cloned().unwrap_or_default()
}

// API keys

pub fn create_api_key(store: &SharedStore, key_prefix: &str, key_hash: &str, name: &str, project_id: Option<Uuid>, permissions: &[String], expires_at: Option<String>) -> ApiKey {
    let mut s = store.write();
    let now = now_iso();
    let key = ApiKey {
        id: Uuid::new_v4(),
        key_prefix: key_prefix.to_string(),
        key_hash: key_hash.to_string(),
        name: name.to_string(),
        project_id,
        permissions: permissions.to_vec(),
        expires_at,
        created_at: now,
    };
    s.api_keys.insert(key.key_prefix.clone(), key.clone());
    s.api_keys_by_id.insert(key.id, key.clone());
    key
}

pub fn get_api_key_by_prefix(store: &SharedStore, prefix: &str) -> Option<ApiKey> {
    store.read().api_keys.get(prefix).cloned()
}

pub fn list_api_keys(store: &SharedStore) -> Vec<ApiKey> {
    let s = store.read();
    let mut keys: Vec<_> = s.api_keys.values().cloned().collect();
    keys.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    keys
}

pub fn delete_api_key(store: &SharedStore, id: Uuid) -> bool {
    let mut s = store.write();
    let key = s.api_keys_by_id.remove(&id);
    if let Some(ref k) = key {
        s.api_keys.remove(&k.key_prefix);
    }
    key.is_some()
}

// Stats

pub fn get_stats(store: &SharedStore) -> serde_json::Value {
    let s = store.read();
    let total_projects = s.projects.len() as i64;
    let total_jobs = s.jobs.len() as i64;
    let total_uploads = s.uploads.len() as i64;

    let mut jobs_by_status: HashMap<String, i64> = HashMap::new();
    let mut jobs_by_type: HashMap<String, i64> = HashMap::new();
    for job in s.jobs.values() {
        *jobs_by_status.entry(job.status.clone()).or_default() += 1;
        *jobs_by_type.entry(job.job_type.clone()).or_default() += 1;
    }

    serde_json::json!({
        "total_projects": total_projects,
        "total_jobs": total_jobs,
        "total_uploads": total_uploads,
        "jobs_by_status": jobs_by_status,
        "jobs_by_type": jobs_by_type,
    })
}
