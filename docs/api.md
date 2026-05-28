# API Reference

The CirBinius API server provides RESTful access to compile, prove, verify, analyze, and conformance jobs.

## Running

```bash
# Default: port 8080
cargo run --bin cirbinius-api

# Custom config
CIRBINIUS_HOST=0.0.0.0 CIRBINIUS_PORT=9090 \
  CIRBINIUS_API_KEY=my-key \
  cargo run --bin cirbinius-api
```

## Authentication

Authenticate via `Authorization: Bearer <api_key>` header.

### Create an API Key

```bash
curl -X POST http://localhost:8080/api/v1/admin/api-keys \
  -H "Content-Type: application/json" \
  -d '{"name":"dev","permissions":["read","write"],"expires_in_days":30}'
```

Response includes the raw API key (shown once).

## Endpoints

### Health

```
GET /health
GET /api/v1/health
```

```json
{"status":"healthy","version":"0.1.0","uptime_secs":12345}
```

### Projects

```
GET    /api/v1/projects
POST   /api/v1/projects          {"name":"my-circuit","description":"..."}
GET    /api/v1/projects/{id}
PATCH  /api/v1/projects/{id}     {"name":"...","status":"active"}
DELETE /api/v1/projects/{id}
```

### Uploads

```
GET    /api/v1/projects/{project_id}/uploads
POST   /api/v1/projects/{project_id}/uploads?filename=circuit.r1cs&content_type=application/octet-stream
       Body: raw file bytes
GET    /api/v1/projects/{project_id}/uploads/{id}
DELETE /api/v1/projects/{project_id}/uploads/{id}
```

### Jobs

```
GET    /api/v1/projects/{project_id}/jobs
POST   /api/v1/projects/{project_id}/compile      {"r1cs_upload_id":"...","sym_upload_id":"..."}
POST   /api/v1/projects/{project_id}/prove        {"r1cs_upload_id":"...","wasm_upload_id":"...","input_upload_id":"..."}
POST   /api/v1/projects/{project_id}/verify       {"bundle_upload_id":"..."}
POST   /api/v1/projects/{project_id}/analyze      {"r1cs_upload_id":"...","sym_upload_id":"..."}
POST   /api/v1/projects/{project_id}/conformance  {"test_categories":["compile","analyze"]}
GET    /api/v1/projects/{project_id}/jobs/{id}
POST   /api/v1/projects/{project_id}/jobs/{id}/cancel
GET    /api/v1/projects/{project_id}/jobs/{id}/logs
```

Job status: `queued` → `running` → `succeeded` / `failed` / `cancelled`

### Artifacts

```
GET    /api/v1/jobs/{job_id}/artifacts
GET    /api/v1/jobs/{job_id}/artifacts/{id}
GET    /api/v1/jobs/{job_id}/artifacts/{id}/download
```

### Admin

```
GET    /api/v1/admin/stats
GET    /api/v1/admin/api-keys
POST   /api/v1/admin/api-keys   {"name":"...","permissions":["read","write"],"expires_in_days":30}
DELETE /api/v1/admin/api-keys/{id}
POST   /api/v1/admin/auth
```

## Rate Limiting

- 60 requests/minute per IP
- 1000 requests/hour per IP
- Returns `429 Too Many Requests` when exceeded

## Dashboard

The developer dashboard is served at `http://localhost:8080/`.
