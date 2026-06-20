use crate::model::WorkflowDefinition;
use anyhow::{Context, Result};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub fn parse_workflow_yaml(raw: &str) -> Result<WorkflowDefinition> {
    serde_yaml::from_str(raw).context("parse workflow yaml")
}

pub fn load_workflow_yaml(path: impl AsRef<Path>) -> Result<WorkflowDefinition> {
    let path = path.as_ref();
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    parse_workflow_yaml(&raw).with_context(|| format!("load {}", path.display()))
}

fn canonical_workflow_name(name: &str) -> &str {
    match name.trim() {
        "build-agent.issue-loop" => "work-agent.issue-loop",
        other => other,
    }
}

pub fn workflow_name_from_ref(workflow_ref: &str) -> Result<String> {
    let Some((name, version)) = workflow_ref.rsplit_once('@') else {
        anyhow::bail!("workflowRef must use <workflow-name>@<version>, found {workflow_ref}");
    };
    if name.trim().is_empty() || version.trim().is_empty() {
        anyhow::bail!("workflowRef must use <workflow-name>@<version>, found {workflow_ref}");
    }
    Ok(canonical_workflow_name(name).to_string())
}

pub fn workflow_path_for_ref(
    project_root: impl AsRef<Path>,
    workflow_ref: &str,
) -> Result<PathBuf> {
    let name = workflow_name_from_ref(workflow_ref)?;
    Ok(project_root
        .as_ref()
        .join(".agentflow/workflows")
        .join(format!("{name}.yaml")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workflow_ref_maps_to_workflow_yaml_path() {
        let path = workflow_path_for_ref("/project", "build-agent.issue-loop@v1").unwrap();

        assert_eq!(
            path,
            PathBuf::from("/project/.agentflow/workflows/work-agent.issue-loop.yaml")
        );
    }

    #[test]
    fn workflow_ref_preserves_canonical_work_agent_name() {
        let path = workflow_path_for_ref("/project", "work-agent.issue-loop@v1").unwrap();

        assert_eq!(
            path,
            PathBuf::from("/project/.agentflow/workflows/work-agent.issue-loop.yaml")
        );
    }

    #[test]
    fn workflow_ref_requires_name_and_version() {
        let err = workflow_name_from_ref("build-agent.issue-loop")
            .unwrap_err()
            .to_string();

        assert!(err.contains("<workflow-name>@<version>"));
    }
}
