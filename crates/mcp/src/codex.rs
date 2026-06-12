use crate::{
    health::unix_timestamp_seconds,
    model::{McpCapability, McpProviderKind, McpProviderStatus, McpProviderStatusCode},
    provider::run_command,
};
use std::{
    fs,
    path::{Path, PathBuf},
    time::SystemTime,
};

const CLI_FRESHNESS_PATHS: [&str; 9] = [
    "Cargo.toml",
    "Cargo.lock",
    "crates/agentflow-cli/src",
    "crates/execute/src",
    "crates/input/src",
    "crates/state/src",
    "crates/panel/src",
    "crates/agent-manual/src",
    "crates/loop/src",
];

pub fn check_codex_provider(project_root: impl AsRef<Path>) -> McpProviderStatus {
    let root = project_root.as_ref();
    let mut status = McpProviderStatus::new(McpProviderKind::Codex, unix_timestamp_seconds());
    let newest_source = newest_source_mtime(root).ok().flatten();

    for candidate in agentflow_cli_candidates(root) {
        if let Some((source_path, source_modified)) = newest_source.as_ref() {
            if is_local_target_binary(root, &candidate)
                && file_modified(&candidate)
                    .map(|binary_modified| binary_modified < *source_modified)
                    .unwrap_or(false)
            {
                status.warnings.push(format!(
                    "{} is stale; rebuild before use because {} is newer",
                    candidate.display(),
                    source_path.display()
                ));
                continue;
            }
        }
        let program = candidate.to_string_lossy().to_string();
        match run_command(root, &program, &["build-agent", "complete", "--help"]) {
            Ok(help) if help.status_success => {
                status.cli = Some(program);
                status.installed = true;
                status.status = McpProviderStatusCode::Ready;
                status.capabilities = vec![McpCapability::with_detail(
                    "build_agent.complete",
                    true,
                    "agentflow build-agent complete is available",
                )];
                return status;
            }
            Ok(help) => {
                status.warnings.push(format!(
                    "{} does not support build-agent complete: {}",
                    program,
                    help.combined_output().trim()
                ));
            }
            Err(error) => {
                status.warnings.push(format!("{program}: {error}"));
            }
        }
    }

    status.status = McpProviderStatusCode::Unavailable;
    status.capabilities = vec![McpCapability::with_detail(
        "build_agent.complete",
        false,
        "agentflow build-agent complete is unavailable",
    )];
    status
}

fn agentflow_cli_candidates(project_root: &Path) -> Vec<PathBuf> {
    vec![
        project_root.join("target/debug/agentflow"),
        project_root.join("target/release/agentflow"),
        PathBuf::from("agentflow"),
    ]
}

fn newest_source_mtime(root: &Path) -> std::io::Result<Option<(PathBuf, SystemTime)>> {
    let mut newest = None;
    for relative in CLI_FRESHNESS_PATHS {
        collect_newest_mtime(&root.join(relative), &mut newest)?;
    }
    Ok(newest)
}

fn collect_newest_mtime(
    path: &Path,
    newest: &mut Option<(PathBuf, SystemTime)>,
) -> std::io::Result<()> {
    if !path.exists() {
        return Ok(());
    }
    let metadata = fs::metadata(path)?;
    if metadata.is_file() {
        let modified = metadata.modified()?;
        let replace = newest
            .as_ref()
            .map(|(_, current)| modified > *current)
            .unwrap_or(true);
        if replace {
            *newest = Some((path.to_path_buf(), modified));
        }
        return Ok(());
    }
    if metadata.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            collect_newest_mtime(&entry.path(), newest)?;
        }
    }
    Ok(())
}

fn file_modified(path: &Path) -> std::io::Result<SystemTime> {
    Ok(fs::metadata(path)?.modified()?)
}

fn is_local_target_binary(root: &Path, executable: &Path) -> bool {
    executable.starts_with(root.join("target"))
        && executable
            .file_name()
            .and_then(|name| name.to_str())
            .map(|name| name == "agentflow")
            .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::{agentflow_cli_candidates, is_local_target_binary};
    use std::path::Path;

    #[test]
    fn prefers_debug_agentflow_before_release_binary() {
        let candidates = agentflow_cli_candidates(Path::new("/repo"));
        assert_eq!(candidates[0], Path::new("/repo/target/debug/agentflow"));
        assert_eq!(candidates[1], Path::new("/repo/target/release/agentflow"));
    }

    #[test]
    fn local_target_binary_detection_matches_workspace_targets() {
        assert!(is_local_target_binary(
            Path::new("/repo"),
            Path::new("/repo/target/debug/agentflow")
        ));
        assert!(!is_local_target_binary(
            Path::new("/repo"),
            Path::new("/usr/local/bin/agentflow")
        ));
    }
}
