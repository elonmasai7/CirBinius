use std::process::Output;

pub struct CommandResult {
    pub output: Output,
}

impl CommandResult {
    pub fn success(&self) -> bool {
        self.output.status.success()
    }

    pub fn stdout_string(&self) -> String {
        String::from_utf8_lossy(&self.output.stdout).to_string()
    }

    pub fn stderr_string(&self) -> String {
        String::from_utf8_lossy(&self.output.stderr).to_string()
    }
}

pub fn run_command(program: &str, args: &[&str]) -> Result<CommandResult, anyhow::Error> {
    let cmd = duct::cmd(program, args);
    let output = cmd.unchecked().stdout_capture().stderr_capture().run()?;
    Ok(CommandResult { output })
}

pub fn run_command_with_stdin(
    program: &str,
    args: &[&str],
    stdin: &[u8],
) -> Result<CommandResult, anyhow::Error> {
    let cmd = duct::cmd(program, args).stdin_bytes(stdin.to_vec());
    let output = cmd.unchecked().stdout_capture().stderr_capture().run()?;
    Ok(CommandResult { output })
}

pub fn find_binary(name: &str) -> Option<std::path::PathBuf> {
    which::which(name).ok()
}

pub struct CommandExt;

impl CommandExt {
    pub fn cmd(program: &str) -> duct::Expression {
        duct::cmd(program, &[] as &[&str])
    }

    pub fn with_args(program: &str, args: &[&str]) -> duct::Expression {
        duct::cmd(program, args)
    }
}
