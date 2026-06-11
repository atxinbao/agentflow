use anyhow::{Context, Result};
use std::{
    path::Path,
    process::{Command, Output},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandProbe {
    pub status_success: bool,
    pub stdout: String,
    pub stderr: String,
}

impl CommandProbe {
    pub fn from_output(output: Output) -> Self {
        Self {
            status_success: output.status.success(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        }
    }

    pub fn combined_output(&self) -> String {
        format!("{}{}", self.stdout, self.stderr)
    }
}

pub fn run_command(
    project_root: impl AsRef<Path>,
    program: &str,
    args: &[&str],
) -> Result<CommandProbe> {
    let output = Command::new(program)
        .args(args)
        .current_dir(project_root.as_ref())
        .output()
        .with_context(|| format!("run {} {}", program, args.join(" ")))?;
    Ok(CommandProbe::from_output(output))
}
