# SDK Reference

CirBinius provides SDKs for Rust, Python, and TypeScript.

## Rust SDK (`cirbinius-sdk`)

### Add to Cargo.toml

```toml
[dependencies]
cirbinius-sdk = { git = "https://github.com/cirbinius/cirbinius" }
```

### Usage

```rust
use cirbinius_sdk::CirbiniusClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = CirbiniusClient::new("127.0.0.1", 8080)
        .with_api_key("my-api-key");

    // Health
    let health = client.health().await?;
    println!("Status: {}", health.status);

    // Projects
    let projects = client.list_projects().await?;
    let project = client.create_project(&CreateProjectRequest {
        name: "my-circuit".into(),
        description: None,
    }).await?;
    let p = client.get_project(&project.id).await?;
    client.delete_project(&project.id).await?;

    // Jobs
    let result = client.create_compile_job(&project.id, &json!({
        "r1cs_upload_id": "...",
    })).await?;
    let job = client.get_job(&project.id, &result.job_id).await?;
    let logs = client.get_job_logs(&project.id, &result.job_id).await?;

    // Artifacts
    let artifacts = client.list_artifacts(&result.job_id).await?;
    let data = client.download_artifact(&result.job_id, &artifacts[0].id).await?;

    // Admin
    let stats = client.get_stats().await?;
    let key = client.create_api_key(&CreateApiKeyRequest {
        name: "dev".into(),
        permissions: Some(vec!["read".into(), "write".into()]),
        expires_in_days: Some(30),
        project_id: None,
    }).await?;

    Ok(())
}
```

### Features

- Async/await throughout
- Typed request/response structs
- Bearer token auth
- Per-request error handling
- Minimal dependencies (tokio, serde, httparse)

## Python SDK (`cirbinius-py`)

### Installation

```bash
# From release (once published)
pip install cirbinius

# Or from source
cp target/release/libcirbinius_py.so crates/cirbinius-py/python/cirbinius/
pip install -e crates/cirbinius-py/python/
```

### Usage

```python
from cirbinius import CirbiniusClient

client = CirbiniusClient(host="127.0.0.1", port=8080, api_key="my-api-key")

# Health
print(client.health())

# Projects
projects = client.list_projects()
project = client.create_project("my-circuit")
client.delete_project(project["id"])

# Jobs
result = client.create_compile_job(project["id"], {
    "r1cs_upload_id": "...",
})
job = client.get_job(project["id"], result["job_id"])
logs = client.get_job_logs(project["id"], result["job_id"])

# Admin
stats = client.get_stats()
key = client.create_api_key(name="dev", permissions=["read", "write"])

client.close()
```

### How it works

The Python SDK uses `ctypes` to load `libcirbinius_py.so` (a Rust cdylib) which wraps the Rust SDK via C FFI. All API calls go through a tokio runtime managed by the shared library.

## TypeScript SDK (`@cirbinius/sdk`)

### Installation

```bash
npm install @cirbinius/sdk
```

### Usage

```typescript
import { CirbiniusClient } from '@cirbinius/sdk';

const client = new CirbiniusClient('127.0.0.1', 8080, 'my-api-key');

// Health
console.log(await client.health());

// Projects
const projects = await client.listProjects();
const project = await client.createProject('my-circuit');
await client.deleteProject(project.id);

// Jobs
const result = await client.createCompileJob(project.id, {
    r1cs_upload_id: '...',
});
const logs = await client.getJobLogs(project.id, result.job_id);

// Admin
const stats = await client.getStats();
```
