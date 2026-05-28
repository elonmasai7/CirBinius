use std::fmt;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OsInfo {
    pub os: String,
    pub arch: String,
    pub rust_version: String,
}

impl OsInfo {
    pub fn detect() -> Self {
        Self {
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
            rust_version: rustc_version().unwrap_or_else(|| "unknown".to_string()),
        }
    }

    pub fn target_triple(&self) -> String {
        format!("{}-{}", self.arch, self.os)
    }

    pub fn is_linux(&self) -> bool {
        self.os == "linux"
    }

    pub fn is_macos(&self) -> bool {
        self.os == "macos"
    }

    pub fn is_windows(&self) -> bool {
        self.os == "windows"
    }
}

impl fmt::Display for OsInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.os, self.arch)
    }
}

pub fn temp_dir() -> std::path::PathBuf {
    std::env::temp_dir()
}

pub fn cache_dir() -> Option<std::path::PathBuf> {
    directories::ProjectDirs::from("dev", "cirbinius", "CirBinius")
        .map(|d| d.cache_dir().to_path_buf())
}

pub fn config_dir() -> Option<std::path::PathBuf> {
    directories::ProjectDirs::from("dev", "cirbinius", "CirBinius")
        .map(|d| d.config_dir().to_path_buf())
}

pub fn data_dir() -> Option<std::path::PathBuf> {
    directories::ProjectDirs::from("dev", "cirbinius", "CirBinius")
        .map(|d| d.data_dir().to_path_buf())
}

fn rustc_version() -> Option<String> {
    let version = std::process::Command::new("rustc")
        .arg("--version")
        .output()
        .ok()?;
    if version.status.success() {
        Some(String::from_utf8_lossy(&version.stdout).trim().to_string())
    } else {
        None
    }
}
