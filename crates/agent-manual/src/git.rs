use std::{
    path::Path,
    process::{Command, Stdio},
};

pub fn is_git_tracked(project_root: &Path, relative_path: &str) -> bool {
    Command::new("git")
        .arg("ls-files")
        .arg("--error-unmatch")
        .arg(relative_path)
        .current_dir(project_root)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}
