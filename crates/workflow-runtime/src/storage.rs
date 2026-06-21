use crate::records::{
    RuntimeAcceptedActionFact, RuntimeCommandFact, RuntimeCommandFactBundle, RuntimeDecisionFact,
    RuntimeProposalFact,
};
use anyhow::{Context, Result};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub fn prepare_runtime_workspace(project_root: impl AsRef<Path>) -> Result<()> {
    let root = canonical_project_root(project_root)?;
    ensure_directory(&root.join(".agentflow/runtime/commands"))?;
    ensure_directory(&root.join(".agentflow/runtime/proposals"))?;
    ensure_directory(&root.join(".agentflow/runtime/decisions"))?;
    ensure_directory(&root.join(".agentflow/runtime/actions"))?;
    Ok(())
}

pub fn write_runtime_command_fact(
    project_root: impl AsRef<Path>,
    fact: &RuntimeCommandFact,
) -> Result<PathBuf> {
    let root = canonical_project_root(project_root)?;
    prepare_runtime_workspace(&root)?;
    let path = runtime_command_fact_path(&root, &fact.command_id);
    write_json(&path, fact)?;
    Ok(path)
}

pub fn write_runtime_proposal_fact(
    project_root: impl AsRef<Path>,
    fact: &RuntimeProposalFact,
) -> Result<PathBuf> {
    let root = canonical_project_root(project_root)?;
    prepare_runtime_workspace(&root)?;
    let path = runtime_proposal_fact_path(&root, &fact.proposal_id);
    write_json(&path, fact)?;
    Ok(path)
}

pub fn write_runtime_decision_fact(
    project_root: impl AsRef<Path>,
    fact: &RuntimeDecisionFact,
) -> Result<PathBuf> {
    let root = canonical_project_root(project_root)?;
    prepare_runtime_workspace(&root)?;
    let path = runtime_decision_fact_path(&root, &fact.proposal_id);
    write_json(&path, fact)?;
    Ok(path)
}

pub fn write_runtime_accepted_action_fact(
    project_root: impl AsRef<Path>,
    fact: &RuntimeAcceptedActionFact,
) -> Result<PathBuf> {
    let root = canonical_project_root(project_root)?;
    prepare_runtime_workspace(&root)?;
    let path = runtime_accepted_action_fact_path(&root, &fact.accepted_action_id);
    write_json(&path, fact)?;
    Ok(path)
}

pub fn load_runtime_command_fact(
    project_root: impl AsRef<Path>,
    command_id: &str,
) -> Result<RuntimeCommandFact> {
    let root = canonical_project_root(project_root)?;
    read_json(&runtime_command_fact_path(&root, command_id))
}

pub fn load_runtime_proposal_fact(
    project_root: impl AsRef<Path>,
    proposal_id: &str,
) -> Result<RuntimeProposalFact> {
    let root = canonical_project_root(project_root)?;
    read_json(&runtime_proposal_fact_path(&root, proposal_id))
}

pub fn load_runtime_proposal_facts(
    project_root: impl AsRef<Path>,
) -> Result<Vec<RuntimeProposalFact>> {
    let root = canonical_project_root(project_root)?;
    load_runtime_json_dir(root.join(".agentflow/runtime/proposals"))
}

pub fn load_runtime_decision_fact(
    project_root: impl AsRef<Path>,
    proposal_id: &str,
) -> Result<RuntimeDecisionFact> {
    let root = canonical_project_root(project_root)?;
    read_json(&runtime_decision_fact_path(&root, proposal_id))
}

pub fn load_runtime_decision_facts(
    project_root: impl AsRef<Path>,
) -> Result<Vec<RuntimeDecisionFact>> {
    let root = canonical_project_root(project_root)?;
    load_runtime_json_dir(root.join(".agentflow/runtime/decisions"))
}

pub fn load_runtime_accepted_action_fact(
    project_root: impl AsRef<Path>,
    accepted_action_id: &str,
) -> Result<RuntimeAcceptedActionFact> {
    let root = canonical_project_root(project_root)?;
    read_json(&runtime_accepted_action_fact_path(
        &root,
        accepted_action_id,
    ))
}

pub fn load_runtime_accepted_action_facts(
    project_root: impl AsRef<Path>,
) -> Result<Vec<RuntimeAcceptedActionFact>> {
    let root = canonical_project_root(project_root)?;
    let actions_dir = root.join(".agentflow/runtime/actions");
    if !actions_dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut action_paths = fs::read_dir(&actions_dir)
        .with_context(|| format!("read {}", actions_dir.display()))?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            path.is_file() && path.extension().and_then(|value| value.to_str()) == Some("json")
        })
        .collect::<Vec<_>>();
    action_paths.sort();
    action_paths
        .into_iter()
        .map(|path| read_json(&path))
        .collect()
}

pub fn load_runtime_command_bundle(
    project_root: impl AsRef<Path>,
    command_id: &str,
) -> Result<RuntimeCommandFactBundle> {
    let root = canonical_project_root(project_root)?;
    let command = load_runtime_command_fact(&root, command_id)?;
    let proposal = load_runtime_proposal_fact(&root, &format!("proposal-{command_id}")).ok();
    let decision = proposal
        .as_ref()
        .and_then(|record| load_runtime_decision_fact(&root, &record.proposal_id).ok());
    let accepted_action = decision
        .as_ref()
        .and_then(|record| record.accepted_action_id.as_deref())
        .and_then(|accepted_action_id| {
            load_runtime_accepted_action_fact(&root, accepted_action_id).ok()
        });
    Ok(RuntimeCommandFactBundle {
        command,
        proposal,
        decision,
        accepted_action,
    })
}

pub fn runtime_command_fact_path(project_root: impl AsRef<Path>, command_id: &str) -> PathBuf {
    project_root
        .as_ref()
        .join(".agentflow/runtime/commands")
        .join(format!("{}.json", sanitize_id(command_id)))
}

pub fn runtime_proposal_fact_path(project_root: impl AsRef<Path>, proposal_id: &str) -> PathBuf {
    project_root
        .as_ref()
        .join(".agentflow/runtime/proposals")
        .join(format!("{}.json", sanitize_id(proposal_id)))
}

pub fn runtime_decision_fact_path(project_root: impl AsRef<Path>, proposal_id: &str) -> PathBuf {
    project_root
        .as_ref()
        .join(".agentflow/runtime/decisions")
        .join(format!("{}.json", sanitize_id(proposal_id)))
}

pub fn runtime_accepted_action_fact_path(
    project_root: impl AsRef<Path>,
    accepted_action_id: &str,
) -> PathBuf {
    project_root
        .as_ref()
        .join(".agentflow/runtime/actions")
        .join(format!("{}.json", sanitize_id(accepted_action_id)))
}

fn sanitize_id(id: &str) -> String {
    id.chars()
        .map(|ch| match ch {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => ch,
        })
        .collect()
}

fn load_runtime_json_dir<T: serde::de::DeserializeOwned>(dir: PathBuf) -> Result<Vec<T>> {
    if !dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut paths = fs::read_dir(&dir)
        .with_context(|| format!("read {}", dir.display()))?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            path.is_file() && path.extension().and_then(|value| value.to_str()) == Some("json")
        })
        .collect::<Vec<_>>();
    paths.sort();
    paths.into_iter().map(|path| read_json(&path)).collect()
}

fn canonical_project_root(project_root: impl AsRef<Path>) -> Result<PathBuf> {
    let root = project_root.as_ref();
    if root.exists() {
        return root
            .canonicalize()
            .with_context(|| format!("canonicalize {}", root.display()));
    }
    Ok(root.to_path_buf())
}

fn ensure_directory(path: &Path) -> Result<()> {
    fs::create_dir_all(path).with_context(|| format!("create {}", path.display()))
}

fn write_json<T: serde::Serialize>(path: &Path, value: &T) -> Result<()> {
    if let Some(parent) = path.parent() {
        ensure_directory(parent)?;
    }
    fs::write(path, serde_json::to_string_pretty(value)? + "\n")
        .with_context(|| format!("write {}", path.display()))
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", path.display()))
}
