use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

pub fn temp_dir(tag: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be after unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("cirbinius-{tag}-{unique}"))
}

#[allow(dead_code)]
pub fn write_fake_snarkjs(script_path: &Path, witness_fixture_path: &Path) {
    let content = if cfg!(windows) {
        format!(
            "@echo off\ncopy /Y \"{}\" \"%5\" >nul\n",
            witness_fixture_path.display()
        )
    } else {
        format!(
            "#!/bin/sh\ncp '{}' \"$5\"\n",
            witness_fixture_path.display()
        )
    };
    fs::write(script_path, &content).expect("should write fake snarkjs script");

    // Mark executable on Unix
    #[cfg(unix)]
    cirbinius_platform::fs::set_executable(script_path).expect("should set executable permission");
}
