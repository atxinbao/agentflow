use crate::{
    health::unix_timestamp_seconds,
    model::{McpProviderStatus, McpRegistry},
    registry::build_registry,
};
use anyhow::{Context, Result};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub fn prepare_mcp_workspace(project_root: impl AsRef<Path>) -> Result<()> {
    let root = canonical_project_root(project_root)?;
    ensure_directory(&root.join(".agentflow/state/mcp/providers"))?;
    Ok(())
}

pub fn write_provider_status(
    project_root: impl AsRef<Path>,
    status: &McpProviderStatus,
) -> Result<PathBuf> {
    let root = canonical_project_root(project_root)?;
    prepare_mcp_workspace(&root)?;
    let path = provider_status_path(&root, &status.provider);
    write_json(&path, status)?;
    Ok(path)
}

pub fn read_provider_status(
    project_root: impl AsRef<Path>,
    provider: &str,
) -> Result<McpProviderStatus> {
    let root = canonical_project_root(project_root)?;
    read_json(&provider_status_path(&root, provider))
}

pub fn write_registry(project_root: impl AsRef<Path>, registry: &McpRegistry) -> Result<PathBuf> {
    let root = canonical_project_root(project_root)?;
    prepare_mcp_workspace(&root)?;
    let path = root.join(".agentflow/state/mcp/registry.json");
    write_json(&path, registry)?;
    Ok(path)
}

pub fn read_registry(project_root: impl AsRef<Path>) -> Result<McpRegistry> {
    let root = canonical_project_root(project_root)?;
    read_json(&root.join(".agentflow/state/mcp/registry.json"))
}

pub fn write_registry_for_statuses(
    project_root: impl AsRef<Path>,
    statuses: &[McpProviderStatus],
) -> Result<McpRegistry> {
    let root = canonical_project_root(project_root)?;
    let registry = build_registry(statuses, unix_timestamp_seconds());
    write_registry(&root, &registry)?;
    Ok(registry)
}

fn provider_status_path(root: &Path, provider: &str) -> PathBuf {
    root.join(".agentflow/state/mcp/providers")
        .join(format!("{}.json", sanitize_id(provider)))
}

fn sanitize_id(id: &str) -> String {
    id.chars()
        .map(|ch| match ch {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => ch,
        })
        .collect()
}

fn canonical_project_root(project_root: impl AsRef<Path>) -> Result<PathBuf> {
    project_root
        .as_ref()
        .canonicalize()
        .with_context(|| format!("canonicalize {}", project_root.as_ref().display()))
}

fn ensure_directory(path: &Path) -> Result<()> {
    fs::create_dir_all(path).with_context(|| format!("create {}", path.display()))
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", path.display()))
}

fn write_json<T: serde::Serialize>(path: &Path, value: &T) -> Result<()> {
    if let Some(parent) = path.parent() {
        ensure_directory(parent)?;
    }
    fs::write(path, serde_json::to_string_pretty(value)?)
        .with_context(|| format!("write {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{McpProviderKind, McpProviderStatus, McpProviderStatusCode};
    use tempfile::tempdir;

    #[test]
    fn writes_and_reads_provider_status() {
        let dir = tempdir().unwrap();
        let mut status = McpProviderStatus::new(McpProviderKind::Github, 100);
        status.status = McpProviderStatusCode::Ready;

        let path = write_provider_status(dir.path(), &status).unwrap();
        assert!(path.ends_with(".agentflow/state/mcp/providers/github.json"));

        let loaded = read_provider_status(dir.path(), "github").unwrap();
        assert_eq!(loaded, status);
    }

    #[test]
    fn writes_registry_for_provider_statuses() {
        let dir = tempdir().unwrap();
        let status = McpProviderStatus::new(McpProviderKind::Codex, 101);

        write_provider_status(dir.path(), &status).unwrap();
        let registry = write_registry_for_statuses(dir.path(), &[status]).unwrap();
        let loaded = read_registry(dir.path()).unwrap();

        assert_eq!(loaded, registry);
        assert_eq!(loaded.providers[0].provider, "codex");
    }
}
