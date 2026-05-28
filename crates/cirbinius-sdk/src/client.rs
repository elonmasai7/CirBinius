use std::collections::HashMap;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

use crate::error::{SdkError, SdkResult};
use crate::types::*;

#[derive(Debug, Clone)]
pub struct CirbiniusClient {
    host: String,
    port: u16,
    api_key: Option<String>,
}

impl CirbiniusClient {
    pub fn new(host: impl Into<String>, port: u16) -> Self {
        Self { host: host.into(), port, api_key: None }
    }

    pub fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    async fn request(
        &self,
        method: &str,
        path: &str,
        query: Option<&str>,
        body: Option<&[u8]>,
        extra_headers: &[(&str, &str)],
    ) -> SdkResult<(u16, HashMap<String, String>, Vec<u8>)> {
        let addr = format!("{}:{}", self.host, self.port);
        let mut stream = TcpStream::connect(&addr)
            .await
            .map_err(|e| SdkError::ConnectionRefused(format!("{addr}: {e}")))?;

        let body_bytes = body.unwrap_or(b"");
        let body_len = body_bytes.len();

        let mut req = format!(
            "{method} {path} HTTP/1.1\r\nHost: {host}:{port}\r\nContent-Length: {body_len}\r\n",
            host = self.host,
            port = self.port,
        );

        if let Some(q) = query {
            req = format!("{method} {path}?{q} HTTP/1.1\r\nHost: {host}:{port}\r\nContent-Length: {body_len}\r\n",
                host = self.host,
                port = self.port,
            );
        }

        if body_len > 0 {
            req.push_str("Content-Type: application/json\r\n");
        }

        for (key, val) in extra_headers {
            req.push_str(&format!("{key}: {val}\r\n"));
        }

        if let Some(ref key) = self.api_key {
            req.push_str(&format!("Authorization: Bearer {key}\r\n"));
        }

        req.push_str("\r\n");

        stream.write_all(req.as_bytes()).await?;
        if !body_bytes.is_empty() {
            stream.write_all(body_bytes).await?;
        }
        stream.flush().await?;

        let mut reader = BufReader::new(&mut stream);
        let mut resp_buf = Vec::new();
        reader.read_until(b'\n', &mut resp_buf).await?;

        let mut raw_headers = vec![httparse::EMPTY_HEADER; 64];
        let mut response = httparse::Response::new(&mut raw_headers);

        let mut header_bytes = Vec::new();
        loop {
            let mut line = Vec::new();
            let n = reader.read_until(b'\n', &mut line).await?;
            if n == 0 {
                break;
            }
            header_bytes.extend_from_slice(&line);
            if line == b"\r\n" || line == b"\n" {
                break;
            }
        }

        let mut all_bytes = Vec::new();
        all_bytes.extend_from_slice(&resp_buf);
        all_bytes.extend_from_slice(&header_bytes);

        let _consumed = response.parse(&all_bytes)
            .map_err(|e| SdkError::HttpParse(format!("parse error: {e}")))?;

        let status_code = response.code.unwrap_or(500);

        let mut headers = HashMap::new();
        for h in response.headers.iter() {
            if let Ok(val) = std::str::from_utf8(h.value) {
                headers.insert(h.name.to_lowercase(), val.to_string());
            }
        }

        let content_length = headers
            .get("content-length")
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(0);

        let mut body_data = Vec::new();
        if content_length > 0 {
            body_data.resize(content_length, 0);
            reader.read_exact(&mut body_data).await?;
        }

        Ok((status_code as u16, headers, body_data))
    }

    pub async fn request_typed(
        &self,
        method: &str,
        path: &str,
        body: Option<&serde_json::Value>,
    ) -> SdkResult<serde_json::Value> {
        let body_bytes = body.map(|b| serde_json::to_vec(b).unwrap_or_default());
        let body_slice = body_bytes.as_deref();
        let (status, _headers, data) = self.request(method, path, None, body_slice, &[]).await?;

        if status >= 400 {
            let err_body = String::from_utf8_lossy(&data);
            return Err(SdkError::HttpError { status, body: err_body.to_string() });
        }

        if data.is_empty() {
            return Ok(serde_json::Value::Null);
        }

        serde_json::from_slice(&data).map_err(SdkError::Json)
    }

    async fn json_request<T: serde::de::DeserializeOwned>(
        &self,
        method: &str,
        path: &str,
        body: Option<&serde_json::Value>,
    ) -> SdkResult<T> {
        let body_bytes = body.map(|b| serde_json::to_vec(b).unwrap_or_default());
        let body_slice = body_bytes.as_deref();
        let (status, _headers, data) = self.request(method, path, None, body_slice, &[]).await?;

        if status >= 400 {
            let err_body = String::from_utf8_lossy(&data);
            return Err(SdkError::HttpError { status, body: err_body.to_string() });
        }

        serde_json::from_slice(&data).map_err(SdkError::Json)
    }

    async fn delete_request(&self, path: &str) -> SdkResult<DeleteResult> {
        self.json_request("DELETE", path, None).await
    }

    // ---- Health ----

    pub async fn health(&self) -> SdkResult<Health> {
        self.json_request("GET", "/health", None).await
    }

    // ---- Projects ----

    pub async fn list_projects(&self) -> SdkResult<Vec<Project>> {
        self.json_request("GET", "/api/v1/projects", None).await
    }

    pub async fn create_project(&self, req: &CreateProjectRequest) -> SdkResult<Project> {
        let body = serde_json::to_value(req)?;
        self.json_request("POST", "/api/v1/projects", Some(&body)).await
    }

    pub async fn get_project(&self, id: &str) -> SdkResult<Project> {
        self.json_request("GET", &format!("/api/v1/projects/{id}"), None).await
    }

    pub async fn update_project(&self, id: &str, req: &UpdateProjectRequest) -> SdkResult<Project> {
        let body = serde_json::to_value(req)?;
        self.json_request("PATCH", &format!("/api/v1/projects/{id}"), Some(&body)).await
    }

    pub async fn delete_project(&self, id: &str) -> SdkResult<DeleteResult> {
        self.delete_request(&format!("/api/v1/projects/{id}")).await
    }

    // ---- Uploads ----

    pub async fn list_uploads(&self, project_id: &str) -> SdkResult<Vec<Upload>> {
        self.json_request("GET", &format!("/api/v1/projects/{project_id}/uploads"), None).await
    }

    pub async fn upload_file(&self, project_id: &str, filename: &str, content_type: &str, data: &[u8]) -> SdkResult<Upload> {
        let query = format!("filename={filename}&content_type={content_type}");
        let (status, _headers, resp_data) = self.request("POST", &format!("/api/v1/projects/{project_id}/uploads"),
            Some(&query), Some(data), &[]).await?;
        if status >= 400 {
            let err_body = String::from_utf8_lossy(&resp_data);
            return Err(SdkError::HttpError { status, body: err_body.to_string() });
        }
        serde_json::from_slice(&resp_data).map_err(SdkError::Json)
    }

    pub async fn get_upload(&self, project_id: &str, upload_id: &str) -> SdkResult<Upload> {
        self.json_request("GET", &format!("/api/v1/projects/{project_id}/uploads/{upload_id}"), None).await
    }

    pub async fn delete_upload(&self, project_id: &str, upload_id: &str) -> SdkResult<DeleteResult> {
        self.delete_request(&format!("/api/v1/projects/{project_id}/uploads/{upload_id}")).await
    }

    // ---- Jobs ----

    pub async fn list_jobs(&self, project_id: &str) -> SdkResult<Vec<Job>> {
        self.json_request("GET", &format!("/api/v1/projects/{project_id}/jobs"), None).await
    }

    async fn create_job_type(&self, project_id: &str, job_type: &str, params: &serde_json::Value) -> SdkResult<JobCreated> {
        self.json_request("POST", &format!("/api/v1/projects/{project_id}/{job_type}"), Some(params)).await
    }

    pub async fn create_compile_job(&self, project_id: &str, params: &serde_json::Value) -> SdkResult<JobCreated> {
        self.create_job_type(project_id, "compile", params).await
    }

    pub async fn create_prove_job(&self, project_id: &str, params: &serde_json::Value) -> SdkResult<JobCreated> {
        self.create_job_type(project_id, "prove", params).await
    }

    pub async fn create_verify_job(&self, project_id: &str, params: &serde_json::Value) -> SdkResult<JobCreated> {
        self.create_job_type(project_id, "verify", params).await
    }

    pub async fn create_analyze_job(&self, project_id: &str, params: &serde_json::Value) -> SdkResult<JobCreated> {
        self.create_job_type(project_id, "analyze", params).await
    }

    pub async fn create_conformance_job(&self, project_id: &str, params: &serde_json::Value) -> SdkResult<JobCreated> {
        self.create_job_type(project_id, "conformance", params).await
    }

    pub async fn get_job(&self, project_id: &str, job_id: &str) -> SdkResult<Job> {
        self.json_request("GET", &format!("/api/v1/projects/{project_id}/jobs/{job_id}"), None).await
    }

    pub async fn cancel_job(&self, project_id: &str, job_id: &str) -> SdkResult<Job> {
        self.json_request("POST", &format!("/api/v1/projects/{project_id}/jobs/{job_id}/cancel"), None).await
    }

    pub async fn get_job_logs(&self, project_id: &str, job_id: &str) -> SdkResult<Vec<JobLogEntry>> {
        self.json_request("GET", &format!("/api/v1/projects/{project_id}/jobs/{job_id}/logs"), None).await
    }

    // ---- Artifacts ----

    pub async fn list_artifacts(&self, job_id: &str) -> SdkResult<Vec<Artifact>> {
        self.json_request("GET", &format!("/api/v1/jobs/{job_id}/artifacts"), None).await
    }

    pub async fn get_artifact(&self, job_id: &str, artifact_id: &str) -> SdkResult<Artifact> {
        self.json_request("GET", &format!("/api/v1/jobs/{job_id}/artifacts/{artifact_id}"), None).await
    }

    pub async fn download_artifact(&self, job_id: &str, artifact_id: &str) -> SdkResult<Vec<u8>> {
        let (status, _headers, data) = self.request("GET",
            &format!("/api/v1/jobs/{job_id}/artifacts/{artifact_id}/download"),
            None, None, &[]).await?;
        if status >= 400 {
            let err_body = String::from_utf8_lossy(&data);
            return Err(SdkError::HttpError { status, body: err_body.to_string() });
        }
        Ok(data)
    }

    // ---- Admin ----

    pub async fn get_stats(&self) -> SdkResult<Stats> {
        self.json_request("GET", "/api/v1/admin/stats", None).await
    }

    pub async fn list_api_keys(&self) -> SdkResult<Vec<ApiKey>> {
        self.json_request("GET", "/api/v1/admin/api-keys", None).await
    }

    pub async fn create_api_key(&self, req: &CreateApiKeyRequest) -> SdkResult<CreatedApiKey> {
        let body = serde_json::to_value(req)?;
        self.json_request("POST", "/api/v1/admin/api-keys", Some(&body)).await
    }

    pub async fn delete_api_key(&self, key_id: &str) -> SdkResult<DeleteResult> {
        self.delete_request(&format!("/api/v1/admin/api-keys/{key_id}")).await
    }

    // ---- Auth ----

    pub async fn check_auth(&self) -> SdkResult<AuthCheckResult> {
        self.json_request("POST", "/api/v1/admin/auth", None).await
    }
}
