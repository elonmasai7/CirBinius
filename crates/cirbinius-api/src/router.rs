use crate::uuid::Uuid;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use bytes::Bytes;
use http::Method;
use http_body_util::Full;
use hyper::body::Incoming;

use crate::auth;
use crate::config::Config;
use crate::error::{ApiError, ApiResult};
use crate::handlers;
use crate::ratelimit::RateLimiter;
use crate::state::SharedStore;
use crate::telemetry;

pub type Response = http::Response<Full<Bytes>>;
pub type Request = http::Request<hyper::body::Incoming>;

type HandlerFn = Arc<dyn Fn(RequestContext) -> Pin<Box<dyn Future<Output = Response> + Send>> + Send + Sync>;

pub struct RequestContext {
    pub state: SharedStore,
    pub config: Config,
    pub store: SharedStore,
    pub params: HashMap<String, String>,
    pub body: Option<Vec<u8>>,
    pub auth_user: Option<(Uuid, String, Vec<String>)>,
    pub method: Method,
    pub path: String,
    pub query: HashMap<String, String>,
}

struct Route {
    method: Method,
    path_pattern: String,
    handler: HandlerFn,
}

pub struct HttpRouter {
    routes: Vec<Route>,
    state: SharedStore,
    config: Config,
    rate_limiter: RateLimiter,
}

impl HttpRouter {
    pub fn new(state: SharedStore, config: Config) -> Self {
        let mut router = Self {
            routes: Vec::new(),
            state,
            config,
            rate_limiter: RateLimiter::new(60, 1000),
        };

        router.register_static("/", "web/dashboard/index.html", "text/html");
        router.register(Method::GET, "/health", handlers::health);
        router.register(Method::GET, "/api/v1/health", handlers::health);

        // Projects CRUD
        router.register(Method::GET, "/api/v1/projects", handlers::list_projects);
        router.register(Method::POST, "/api/v1/projects", handlers::create_project);
        router.register(Method::GET, "/api/v1/projects/{id}", handlers::get_project);
        router.register(Method::PATCH, "/api/v1/projects/{id}", handlers::update_project);
        router.register(Method::DELETE, "/api/v1/projects/{id}", handlers::delete_project);

        // Uploads
        router.register(Method::GET, "/api/v1/projects/{project_id}/uploads", handlers::list_uploads);
        router.register(Method::POST, "/api/v1/projects/{project_id}/uploads", handlers::upload_file);
        router.register(Method::GET, "/api/v1/projects/{project_id}/uploads/{id}", handlers::get_upload);
        router.register(Method::DELETE, "/api/v1/projects/{project_id}/uploads/{id}", handlers::delete_upload);

        // Jobs
        router.register(Method::GET, "/api/v1/projects/{project_id}/jobs", handlers::list_jobs);
        router.register(Method::POST, "/api/v1/projects/{project_id}/compile", handlers::create_compile_job);
        router.register(Method::POST, "/api/v1/projects/{project_id}/prove", handlers::create_prove_job);
        router.register(Method::POST, "/api/v1/projects/{project_id}/verify", handlers::create_verify_job);
        router.register(Method::POST, "/api/v1/projects/{project_id}/analyze", handlers::create_analyze_job);
        router.register(Method::POST, "/api/v1/projects/{project_id}/conformance", handlers::create_conformance_job);
        router.register(Method::GET, "/api/v1/projects/{project_id}/jobs/{id}", handlers::get_job);
        router.register(Method::POST, "/api/v1/projects/{project_id}/jobs/{id}/cancel", handlers::cancel_job);
        router.register(Method::GET, "/api/v1/projects/{project_id}/jobs/{id}/logs", handlers::get_job_logs);

        // Artifacts
        router.register(Method::GET, "/api/v1/jobs/{job_id}/artifacts", handlers::list_artifacts);
        router.register(Method::GET, "/api/v1/jobs/{job_id}/artifacts/{id}", handlers::get_artifact);
        router.register(Method::GET, "/api/v1/jobs/{job_id}/artifacts/{id}/download", handlers::download_artifact);

        // Admin
        router.register(Method::GET, "/api/v1/admin/stats", handlers::get_stats);
        router.register(Method::GET, "/api/v1/admin/api-keys", handlers::list_api_keys);
        router.register(Method::POST, "/api/v1/admin/api-keys", handlers::create_api_key);
        router.register(Method::DELETE, "/api/v1/admin/api-keys/{id}", handlers::delete_api_key);

        // Auth route
        router.register(Method::POST, "/api/v1/admin/auth", handlers::auth_check);

        router
    }

    fn register_static(&mut self, path_pattern: &str, file_path: &'static str, content_type: &'static str) {
        let data = std::fs::read_to_string(file_path).unwrap_or_default();
        let body = Full::new(Bytes::from(data));
        let resp = http::Response::builder()
            .status(200)
            .header("content-type", content_type)
            .body(body)
            .unwrap();
        self.routes.push(Route {
            method: Method::GET,
            path_pattern: path_pattern.to_string(),
            handler: Arc::new(move |_ctx| {
                let resp = resp.clone();
                Box::pin(async move { resp })
            }),
        });
    }

    fn register<H, F>(&mut self, method: Method, path_pattern: &str, handler: H)
    where
        H: Fn(RequestContext) -> F + Send + Sync + 'static,
        F: Future<Output = Response> + Send + 'static,
    {
        self.routes.push(Route {
            method,
            path_pattern: path_pattern.to_string(),
            handler: Arc::new(move |ctx| Box::pin(handler(ctx))),
        });
    }

    fn match_route<'a>(&'a self, method: &Method, path: &str) -> Option<(&'a Route, HashMap<String, String>)> {
        for route in &self.routes {
            if *method != route.method {
                continue;
            }
            if let Some(params) = match_path(path, &route.path_pattern) {
                return Some((route, params));
            }
        }
        None
    }

    pub async fn handle(&self, req: http::Request<Incoming>) -> Response {
        let method = req.method().clone();
        let path = req.uri().path().to_string();
        let _timing = telemetry::TimingScope::start("handle_request");
        telemetry::record_request();

        // Rate limit by IP
        let client_ip = req.headers()
            .get("x-forwarded-for")
            .and_then(|v| v.to_str().ok())
            .or_else(|| req.headers().get("x-real-ip").and_then(|v| v.to_str().ok()))
            .unwrap_or("unknown");
        if let Err(msg) = self.rate_limiter.check(client_ip) {
            return error_response(429, msg);
        }

        // Parse query params
        let query = req.uri().query().unwrap_or("").split('&')
            .filter_map(|s| {
                let mut parts = s.splitn(2, '=');
                let key = parts.next()?.to_string();
                let val = parts.next().unwrap_or("").to_string();
                Some((key, val))
            })
            .collect::<HashMap<_, _>>();

        // Extract auth header before consuming body
        let auth_header = req.headers()
            .get(http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_owned());

        // Read body
        let body = match read_body(req.into_body()).await {
            Ok(b) => b,
            Err(e) => return error_response(400, &format!("failed to read body: {e}")),
        };

        // Authenticate
        let auth_user = auth::authenticate(&self.state, auth_header.as_deref()).ok();

        let (route, params) = match self.match_route(&method, &path) {
            Some(r) => r,
            None => return error_response(404, &format!("not found: {method} {path}")),
        };

        let ctx = RequestContext {
            state: self.state.clone(),
            store: self.state.clone(),
            config: self.config.clone(),
            params,
            body: if body.is_empty() { None } else { Some(body) },
            auth_user,
            method,
            path,
            query,
        };

        (route.handler)(ctx).await
    }
}

fn match_path(path: &str, pattern: &str) -> Option<HashMap<String, String>> {
    let path_parts: Vec<&str> = path.trim_matches('/').split('/').collect();
    let pattern_parts: Vec<&str> = pattern.trim_matches('/').split('/').collect();

    if path_parts.len() != pattern_parts.len() {
        return None;
    }

    let mut params = HashMap::new();
    for (pp, pat) in path_parts.iter().zip(pattern_parts.iter()) {
        if pat.starts_with('{') && pat.ends_with('}') {
            let key = &pat[1..pat.len() - 1];
            params.insert(key.to_string(), pp.to_string());
        } else if pp != pat {
            return None;
        }
    }

    Some(params)
}

pub async fn read_body(body: Incoming) -> Result<Vec<u8>, String> {
    use http_body_util::BodyExt;
    let collected = body.collect().await.map_err(|e| format!("body error: {e}"))?;
    Ok(collected.to_bytes().to_vec())
}

pub fn error_response(status: u16, message: &str) -> Response {
    let body = serde_json::json!({ "error": message });
    let bytes = serde_json::to_vec(&body).unwrap_or_default();
    http::Response::builder()
        .status(status)
        .header("content-type", "application/json")
        .body(Full::new(Bytes::from(bytes)))
        .unwrap()
}

pub fn json_response<T: serde::Serialize>(value: &T) -> Response {
    let bytes = serde_json::to_vec(value).unwrap_or_default();
    http::Response::builder()
        .status(200)
        .header("content-type", "application/json")
        .body(Full::new(Bytes::from(bytes)))
        .unwrap()
}

pub fn json_response_with_status<T: serde::Serialize>(value: &T, status: u16) -> Response {
    let bytes = serde_json::to_vec(value).unwrap_or_default();
    http::Response::builder()
        .status(status)
        .header("content-type", "application/json")
        .body(Full::new(Bytes::from(bytes)))
        .unwrap()
}

pub fn parse_body<T: serde::de::DeserializeOwned>(ctx: &RequestContext) -> ApiResult<T> {
    let body = ctx.body.as_deref().unwrap_or_default();
    serde_json::from_slice(body).map_err(|e| ApiError::BadRequest(format!("invalid JSON: {e}")))
}

pub fn require_auth(ctx: &RequestContext) -> ApiResult<&(Uuid, String, Vec<String>)> {
    ctx.auth_user.as_ref().ok_or_else(|| ApiError::Unauthorized("authentication required".into()))
}
