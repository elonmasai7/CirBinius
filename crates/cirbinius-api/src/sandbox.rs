use std::path::Path;
use std::time::Duration;

use crate::config::Config;
use crate::error::{ApiError, ApiResult};

pub enum SandboxKind {
    Linux,
    MacOs,
    Windows,
    Dummy,
}

impl SandboxKind {
    pub fn detect() -> Self {
        #[cfg(target_os = "linux")]
        { SandboxKind::Linux }
        #[cfg(target_os = "macos")]
        { SandboxKind::MacOs }
        #[cfg(target_os = "windows")]
        { SandboxKind::Windows }
        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        { SandboxKind::Dummy }
    }
}

#[derive(Debug, Clone)]
pub struct SandboxOutput {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub timed_out: bool,
}

fn apply_resource_limits(config: &Config) {
    #[cfg(target_os = "linux")]
    {
        use libc::{rlim_t, RLIMIT_AS, RLIMIT_CPU, RLIMIT_DATA, setrlimit};

        let mem_mb = config.sandbox_memory_limit_mb.max(64);
        let mem_bytes = (mem_mb as u64) * 1024 * 1024;
        let rlim = libc::rlimit { rlim_cur: mem_bytes as rlim_t, rlim_max: mem_bytes as rlim_t };
        unsafe { setrlimit(RLIMIT_AS, &rlim); }
        unsafe { setrlimit(RLIMIT_DATA, &rlim); }

        let cpu_secs = config.sandbox_timeout.as_secs().max(1);
        let cpu_rlim = libc::rlimit { rlim_cur: cpu_secs as rlim_t, rlim_max: cpu_secs as rlim_t };
        unsafe { setrlimit(RLIMIT_CPU, &cpu_rlim); }
    }
    let _ = config;
}

pub fn execute(
    command: &str,
    args: &[&str],
    work_dir: &Path,
    config: &Config,
) -> impl std::future::Future<Output = ApiResult<SandboxOutput>> {
    let work_dir = work_dir.to_path_buf();
    let config = config.clone();
    let command = command.to_string();
    let args: Vec<String> = args.iter().map(|s| s.to_string()).collect();

    async move {
        apply_resource_limits(&config);
        let timeout = config.sandbox_timeout;
        let kind = SandboxKind::detect();

        match kind {
            SandboxKind::Linux => execute_linux(&command, &args, &work_dir, timeout).await,
            _ => execute_default(&command, &args, &work_dir, timeout).await,
        }
    }
}

async fn execute_linux(command: &str, args: &[String], work_dir: &Path, timeout: Duration) -> ApiResult<SandboxOutput> {
    let mut cmd = tokio::process::Command::new(command);
    cmd.args(args)
        .current_dir(work_dir)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());

    apply_resource_limits_for_child(&mut cmd);

    let output = match tokio::time::timeout(timeout, cmd.output()).await {
        Ok(Ok(output)) => output,
        Ok(Err(e)) => return Err(ApiError::Internal(format!("sandbox exec: {e}"))),
        Err(_) => {
            return Ok(SandboxOutput {
                stdout: String::new(),
                stderr: "timed out".into(),
                exit_code: -1,
                timed_out: true,
            });
        }
    };

    Ok(SandboxOutput {
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        exit_code: output.status.code().unwrap_or(-1),
        timed_out: false,
    })
}

#[cfg(target_os = "linux")]
fn apply_resource_limits_for_child(cmd: &mut tokio::process::Command) {
    use std::os::unix::process::CommandExt;
    let mem_mb = 2048u64;
    let mem_bytes = mem_mb * 1024 * 1024;
    let cpu_secs = 300u64;
    unsafe {
        cmd.pre_exec(move || {
            let rlim_as = libc::rlimit {
                rlim_cur: mem_bytes as libc::rlim_t,
                rlim_max: mem_bytes as libc::rlim_t,
            };
            libc::setrlimit(libc::RLIMIT_AS, &rlim_as);
            let rlim_cpu = libc::rlimit {
                rlim_cur: cpu_secs as libc::rlim_t,
                rlim_max: cpu_secs as libc::rlim_t,
            };
            libc::setrlimit(libc::RLIMIT_CPU, &rlim_cpu);
            Ok(())
        });
    }
}

#[cfg(not(target_os = "linux"))]
fn apply_resource_limits_for_child(_cmd: &mut tokio::process::Command) {}

async fn execute_default(command: &str, args: &[String], work_dir: &Path, _timeout: Duration) -> ApiResult<SandboxOutput> {
    let mut cmd = tokio::process::Command::new(command);
    cmd.args(args)
        .current_dir(work_dir)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());

    let output = cmd.output().await
        .map_err(|e| ApiError::Internal(format!("sandbox exec: {e}")))?;

    Ok(SandboxOutput {
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        exit_code: output.status.code().unwrap_or(-1),
        timed_out: false,
    })
}

pub fn create_sandboxed_work_dir(config: &Config) -> ApiResult<tempfile::TempDir> {
    apply_resource_limits(config);
    tempfile::TempDir::new().map_err(|e| ApiError::Internal(format!("tempdir: {e}")))
}
