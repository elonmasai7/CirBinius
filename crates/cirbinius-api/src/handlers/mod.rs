use crate::uuid::Uuid;

use std::path::Path;

use sha2::{Digest, Sha256};

use crate::auth;
use crate::queue::enqueue_job;
use crate::router::{error_response, json_response, json_response_with_status, parse_body, require_auth, RequestContext, Response};
use crate::state::{self as st};
use crate::telemetry::startup_msg;

macro_rules! require_param {
    ($params:expr, $key:expr) => {
        match $params.get($key) {
            Some(v) => v.clone(),
            None => return $crate::router::error_response(400, &format!("missing path param: {}", $key)),
        }
    };
}

macro_rules! require_auth_or_401 {
    ($ctx:expr) => {
        match $crate::router::require_auth($ctx) {
            Ok(_) => {},
            Err(e) => return $crate::router::error_response(401, &e.to_string()),
        }
    };
}

// ---- Health ----

pub async fn health(_ctx: RequestContext) -> Response {
    json_response(&serde_json::json!({
        "status": "healthy",
        "version": env!("CARGO_PKG_VERSION"),
        "uptime_secs": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs(),
    }))
}

// ---- Auth ----

pub async fn auth_check(ctx: RequestContext) -> Response {
    match require_auth(&ctx) {
        Ok((id, prefix, perms)) => json_response(&serde_json::json!({
            "authenticated": true,
            "key_id": id,
            "key_prefix": prefix,
            "permissions": perms,
        })),
        Err(e) => error_response(401, &e.to_string()),
    }
}

// ---- Projects ----

pub async fn list_projects(ctx: RequestContext) -> Response {
    require_auth_or_401!(&ctx);
    let projects = st::list_projects(&ctx.store);
    json_response(&projects)
}

pub async fn create_project(ctx: RequestContext) -> Response {
    require_auth_or_401!(&ctx);
    match parse_body::<serde_json::Value>(&ctx) {
        Ok(body) => {
            let name = body.get("name").and_then(|v| v.as_str()).unwrap_or("unnamed");
            let description = body.get("description").and_then(|v| v.as_str());
            let project = st::create_project(&ctx.store, name, description);
            json_response_with_status(&project, 201)
        }
        Err(e) => error_response(400, &e.to_string()),
    }
}

pub async fn get_project(ctx: RequestContext) -> Response {
    require_auth_or_401!(&ctx);
    let id = require_param!(ctx.params, "id");
    let id = match Uuid::parse_str(&id) {
        Ok(id) => id,
        Err(_) => return error_response(400, "invalid project id"),
    };
    match st::get_project(&ctx.store, id) {
        Some(p) => json_response(&p),
        None => error_response(404, &format!("project {id} not found")),
    }
}

pub async fn update_project(ctx: RequestContext) -> Response {
    require_auth_or_401!(&ctx);
    let id = require_param!(ctx.params, "id");
    let id = match Uuid::parse_str(&id) {
        Ok(id) => id,
        Err(_) => return error_response(400, "invalid project id"),
    };
    match parse_body::<serde_json::Value>(&ctx) {
        Ok(body) => {
            let name = body.get("name").and_then(|v| v.as_str());
            let description = body.get("description").and_then(|v| v.as_str());
            let status = body.get("status").and_then(|v| v.as_str());
            match st::update_project(&ctx.store, id, name, description, status) {
                Some(p) => json_response(&p),
                None => error_response(404, &format!("project {id} not found")),
            }
        }
        Err(e) => error_response(400, &e.to_string()),
    }
}

pub async fn delete_project(ctx: RequestContext) -> Response {
    require_auth_or_401!(&ctx);
    let id = require_param!(ctx.params, "id");
    let id = match Uuid::parse_str(&id) {
        Ok(id) => id,
        Err(_) => return error_response(400, "invalid project id"),
    };
    if st::delete_project(&ctx.store, id) {
        json_response(&serde_json::json!({"deleted": true}))
    } else {
        error_response(404, &format!("project {id} not found"))
    }
}

// ---- Uploads ----

pub async fn list_uploads(ctx: RequestContext) -> Response {
    require_auth_or_401!(&ctx);
    let project_id = require_param!(ctx.params, "project_id");
    let project_id = match Uuid::parse_str(&project_id) {
        Ok(id) => id,
        Err(_) => return error_response(400, "invalid project id"),
    };
    if st::get_project(&ctx.store, project_id).is_none() {
        return error_response(404, "project not found");
    }
    let uploads = st::list_uploads(&ctx.store, project_id);
    json_response(&uploads)
}

pub async fn upload_file(ctx: RequestContext) -> Response {
    require_auth_or_401!(&ctx);
    let project_id = require_param!(ctx.params, "project_id");
    let project_id = match Uuid::parse_str(&project_id) {
        Ok(id) => id,
        Err(_) => return error_response(400, "invalid project id"),
    };
    if st::get_project(&ctx.store, project_id).is_none() {
        return error_response(404, "project not found");
    }

    // Parse multipart form manually (simple delimiter-based)
    let body = match ctx.body {
        Some(ref b) if !b.is_empty() => b.clone(),
        _ => return error_response(400, "no body provided"),
    };

    // Try to extract filename and content from the body
    // For simplicity, we accept application/octet-stream with filename in query
    let filename = ctx.query.get("filename").cloned()
        .unwrap_or_else(|| format!("upload_{}", Uuid::new_v4()));
    let content_type = ctx.query.get("content_type").cloned()
        .unwrap_or_else(|| "application/octet-stream".to_string());

    // Validate extension
    let ext = Path::new(&filename)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();
    if !ctx.config.allowed_upload_extensions.contains(&ext) {
        return error_response(415, &format!("extension '{ext}' not allowed"));
    }

    if body.len() as u64 > ctx.config.max_upload_size_bytes {
        return error_response(413, "upload too large");
    }

    let hash = hex::encode(Sha256::digest(&body));

    // Store to disk
    let upload_dir = &ctx.config.upload_dir;
    if let Err(e) = std::fs::create_dir_all(upload_dir) {
        return error_response(500, &format!("failed to create upload dir: {e}"));
    }
    let store_path = upload_dir.join(format!("{}_{}", Uuid::new_v4(), sanitize(&filename)));
    if let Err(e) = std::fs::write(&store_path, &body) {
        return error_response(500, &format!("failed to write upload: {e}"));
    }

    let upload = st::create_upload(
        &ctx.store,
        project_id,
        &filename,
        body.len() as i64,
        &content_type,
        &store_path.to_string_lossy(),
        &hash,
    );
    json_response_with_status(&upload, 201)
}

pub async fn get_upload(ctx: RequestContext) -> Response {
    require_auth_or_401!(&ctx);
    let id = require_param!(ctx.params, "id");
    let id = match Uuid::parse_str(&id) {
        Ok(id) => id,
        Err(_) => return error_response(400, "invalid upload id"),
    };
    match st::get_upload(&ctx.store, id) {
        Some(u) => json_response(&u),
        None => error_response(404, &format!("upload {id} not found")),
    }
}

pub async fn delete_upload(ctx: RequestContext) -> Response {
    require_auth_or_401!(&ctx);
    let id = require_param!(ctx.params, "id");
    let id = match Uuid::parse_str(&id) {
        Ok(id) => id,
        Err(_) => return error_response(400, "invalid upload id"),
    };
    // Try to delete the file
    if let Some(upload) = st::get_upload(&ctx.store, id) {
        std::fs::remove_file(&upload.storage_path).ok();
    }
    if st::delete_upload(&ctx.store, id) {
        json_response(&serde_json::json!({"deleted": true}))
    } else {
        error_response(404, &format!("upload {id} not found"))
    }
}

// ---- Jobs ----

async fn create_job_internal(ctx: &RequestContext, job_type: &str) -> Response {
    let project_id = match ctx.params.get("project_id").and_then(|s| Uuid::parse_str(s).ok()) {
        Some(id) => id,
        None => return error_response(400, "invalid project id"),
    };
    if st::get_project(&ctx.store, project_id).is_none() {
        return error_response(404, "project not found");
    }
    let params = match ctx.body.as_deref() {
        Some(body) => match serde_json::from_slice(body) {
            Ok(v) => v,
            Err(e) => return error_response(400, &format!("invalid JSON: {e}")),
        },
        None => serde_json::json!({}),
    };

    let job = st::create_job(&ctx.store, project_id, job_type, &params);
    startup_msg(&format!("job created: {} {} for project {}", job.id, job_type, project_id));

    // Enqueue the job for async processing
    enqueue_job(job.id, project_id, job_type, &params);

    json_response_with_status(&serde_json::json!({
        "job_id": job.id,
        "project_id": project_id,
        "status": "queued",
        "job_type": job_type,
    }), 201)
}

pub async fn list_jobs(ctx: RequestContext) -> Response {
    require_auth_or_401!(&ctx);
    let project_id = match ctx.params.get("project_id").and_then(|s| Uuid::parse_str(s).ok()) {
        Some(id) => id,
        None => return error_response(400, "invalid project id"),
    };
    let jobs = st::list_jobs(&ctx.store, project_id);
    json_response(&jobs)
}

pub async fn create_compile_job(ctx: RequestContext) -> Response {
    require_auth_or_401!(&ctx);
    create_job_internal(&ctx, "compile").await
}

pub async fn create_prove_job(ctx: RequestContext) -> Response {
    require_auth_or_401!(&ctx);
    create_job_internal(&ctx, "prove").await
}

pub async fn create_verify_job(ctx: RequestContext) -> Response {
    require_auth_or_401!(&ctx);
    create_job_internal(&ctx, "verify").await
}

pub async fn create_analyze_job(ctx: RequestContext) -> Response {
    require_auth_or_401!(&ctx);
    create_job_internal(&ctx, "analyze").await
}

pub async fn create_conformance_job(ctx: RequestContext) -> Response {
    require_auth_or_401!(&ctx);
    create_job_internal(&ctx, "conformance").await
}

pub async fn get_job(ctx: RequestContext) -> Response {
    require_auth_or_401!(&ctx);
    let id = match ctx.params.get("id").and_then(|s| Uuid::parse_str(s).ok()) {
        Some(id) => id,
        None => return error_response(400, "invalid job id"),
    };
    match st::get_job(&ctx.store, id) {
        Some(j) => json_response(&j),
        None => error_response(404, &format!("job {id} not found")),
    }
}

pub async fn cancel_job(ctx: RequestContext) -> Response {
    require_auth_or_401!(&ctx);
    let id = match ctx.params.get("id").and_then(|s| Uuid::parse_str(s).ok()) {
        Some(id) => id,
        None => return error_response(400, "invalid job id"),
    };
    match st::get_job(&ctx.store, id) {
        Some(job) if job.status == "queued" || job.status == "running" => {
            st::update_job_status(&ctx.store, id, "cancelled", None, None, None);
            json_response(&st::get_job(&ctx.store, id).unwrap())
        }
        Some(_) => error_response(400, &format!("job {id} cannot be cancelled in its current state")),
        None => error_response(404, &format!("job {id} not found")),
    }
}

pub async fn get_job_logs(ctx: RequestContext) -> Response {
    require_auth_or_401!(&ctx);
    let id = match ctx.params.get("id").and_then(|s| Uuid::parse_str(s).ok()) {
        Some(id) => id,
        None => return error_response(400, "invalid job id"),
    };
    let logs = st::get_job_logs(&ctx.store, id);
    json_response(&logs)
}

// ---- Artifacts ----

pub async fn list_artifacts(ctx: RequestContext) -> Response {
    require_auth_or_401!(&ctx);
    let job_id = match ctx.params.get("job_id").and_then(|s| Uuid::parse_str(s).ok()) {
        Some(id) => id,
        None => return error_response(400, "invalid job id"),
    };
    let artifacts = st::list_artifacts(&ctx.store, job_id);
    json_response(&artifacts)
}

pub async fn get_artifact(ctx: RequestContext) -> Response {
    require_auth_or_401!(&ctx);
    let id = match ctx.params.get("id").and_then(|s| Uuid::parse_str(s).ok()) {
        Some(id) => id,
        None => return error_response(400, "invalid artifact id"),
    };
    match st::get_artifact(&ctx.store, id) {
        Some(a) => json_response(&a),
        None => error_response(404, &format!("artifact {id} not found")),
    }
}

pub async fn download_artifact(ctx: RequestContext) -> Response {
    require_auth_or_401!(&ctx);
    let id = match ctx.params.get("id").and_then(|s| Uuid::parse_str(s).ok()) {
        Some(id) => id,
        None => return error_response(400, "invalid artifact id"),
    };
    let artifact = match st::get_artifact(&ctx.store, id) {
        Some(a) => a,
        None => return error_response(404, &format!("artifact {id} not found")),
    };
    let data = match std::fs::read(&artifact.storage_path) {
        Ok(d) => d,
        Err(e) => return error_response(500, &format!("failed to read artifact: {e}")),
    };
    http::Response::builder()
        .status(200)
        .header("content-type", "application/octet-stream")
        .header("content-disposition", format!("attachment; filename=\"{}\"", artifact.filename))
        .header("content-length", data.len().to_string())
        .body(http_body_util::Full::new(bytes::Bytes::from(data)))
        .unwrap()
}

// ---- Admin ----

pub async fn get_stats(ctx: RequestContext) -> Response {
    require_auth_or_401!(&ctx);
    let stats = st::get_stats(&ctx.store);
    json_response(&stats)
}

pub async fn list_api_keys(ctx: RequestContext) -> Response {
    require_auth_or_401!(&ctx);
    let keys = st::list_api_keys(&ctx.store);
    let masked: Vec<serde_json::Value> = keys.into_iter().map(|k| {
        serde_json::json!({
            "id": k.id,
            "key_prefix": k.key_prefix,
            "name": k.name,
            "project_id": k.project_id,
            "permissions": k.permissions,
            "expires_at": k.expires_at,
            "created_at": k.created_at,
            "key_hash": "[REDACTED]",
        })
    }).collect();
    json_response(&masked)
}

pub async fn create_api_key(ctx: RequestContext) -> Response {
    require_auth_or_401!(&ctx);
    let body: serde_json::Value = match parse_body(&ctx) {
        Ok(b) => b,
        Err(e) => return error_response(400, &e.to_string()),
    };
    let name = body.get("name").and_then(|v| v.as_str()).unwrap_or("default");
    let project_id = body.get("project_id").and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok());
    let permissions: Vec<String> = body.get("permissions")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_else(|| vec!["read".to_string()]);
    let expires_days = body.get("expires_in_days").and_then(|v| v.as_i64());

    let (api_key, prefix, id) = auth::generate_api_key(
        &ctx.store,
        name,
        project_id,
        &permissions,
        expires_days,
    );

    json_response_with_status(&serde_json::json!({
        "api_key_id": id,
        "api_key": api_key,
        "key_prefix": prefix,
    }), 201)
}

pub async fn delete_api_key(ctx: RequestContext) -> Response {
    require_auth_or_401!(&ctx);
    let id = match ctx.params.get("id").and_then(|s| Uuid::parse_str(s).ok()) {
        Some(id) => id,
        None => return error_response(400, "invalid api key id"),
    };
    if st::delete_api_key(&ctx.store, id) {
        json_response(&serde_json::json!({"deleted": true}))
    } else {
        error_response(404, &format!("api key {id} not found"))
    }
}

fn sanitize(name: &str) -> String {
    name.chars().map(|c| if c.is_alphanumeric() || c == '.' || c == '-' || c == '_' { c } else { '_' }).collect()
}
