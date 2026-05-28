use std::path::Path;

pub fn set_executable(path: &Path) -> Result<(), anyhow::Error> {
    let mut perms = std::fs::metadata(path)
        .map_err(|e| anyhow::anyhow!("failed to read metadata for {}: {}", path.display(), e))?
        .permissions();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        perms.set_mode(0o755);
    }
    std::fs::set_permissions(path, perms)
        .map_err(|e| anyhow::anyhow!("failed to set permissions on {}: {}", path.display(), e))?;
    Ok(())
}

pub fn write_script(path: &Path, content: &str) -> Result<(), anyhow::Error> {
    std::fs::write(path, content)?;
    set_executable(path)?;
    Ok(())
}

pub fn create_temp_dir() -> Result<tempfile::TempDir, anyhow::Error> {
    Ok(tempfile::tempdir()?)
}

pub fn create_temp_file() -> Result<tempfile::NamedTempFile, anyhow::Error> {
    Ok(tempfile::NamedTempFile::new()?)
}

pub fn canonicalize(path: &Path) -> Result<std::path::PathBuf, anyhow::Error> {
    Ok(path.canonicalize()?)
}
