use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub upload_dir: PathBuf,
    pub artifact_dir: PathBuf,
    pub worker_count: usize,
    pub worker_poll_interval_ms: u64,
    pub conformance_fixtures_dir: PathBuf,
    pub max_upload_size_bytes: u64,
    pub allowed_upload_extensions: Vec<String>,
    pub circom_bin: String,
    pub snarkjs_bin: String,
    pub sandbox_timeout: Duration,
    pub sandbox_memory_limit_mb: u64,
    pub log_level: String,
}

impl Config {
    pub fn from_env() -> Self {
        Config {
            host: env("CIRBINIUS_HOST", "0.0.0.0"),
            port: env_parse("CIRBINIUS_PORT", 3000),
            upload_dir: PathBuf::from(env("CIRBINIUS_UPLOAD_DIR", "/tmp/cirbinius/uploads")),
            artifact_dir: PathBuf::from(env("CIRBINIUS_ARTIFACT_DIR", "/tmp/cirbinius/artifacts")),
            worker_count: env_parse("CIRBINIUS_WORKER_COUNT", 4),
            worker_poll_interval_ms: env_parse("CIRBINIUS_WORKER_POLL_INTERVAL_MS", 1000),
            conformance_fixtures_dir: PathBuf::from(env("CIRBINIUS_CONFORMANCE_FIXTURES_DIR", "/tmp/cirbinius/fixtures")),
            max_upload_size_bytes: env_parse("CIRBINIUS_MAX_UPLOAD_SIZE_BYTES", 100 * 1024 * 1024),
            allowed_upload_extensions: env("CIRBINIUS_ALLOWED_UPLOAD_EXTENSIONS", "r1cs,sym,json,wasm,cbir")
                .split(',').map(|s| s.trim().to_lowercase()).collect(),
            circom_bin: env("CIRBINIUS_CIRCOM_BIN", "circom"),
            snarkjs_bin: env("CIRBINIUS_SNARKJS_BIN", "snarkjs"),
            sandbox_timeout: Duration::from_secs(env_parse("CIRBINIUS_SANDBOX_TIMEOUT_SECS", 300)),
            sandbox_memory_limit_mb: env_parse("CIRBINIUS_SANDBOX_MEMORY_LIMIT_MB", 2048),
            log_level: env("CIRBINIUS_LOG_LEVEL", "info"),
        }
    }

    pub fn addr(&self) -> std::net::SocketAddr {
        format!("{}:{}", self.host, self.port).parse().expect("valid socket addr")
    }
}

fn env(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

fn env_parse<T: std::str::FromStr>(key: &str, default: T) -> T {
    std::env::var(key).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
}
