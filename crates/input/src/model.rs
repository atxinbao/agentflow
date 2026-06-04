use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::{
    issue::InputIssue,
    project::InputProject,
    relations::InputIssueRelationsFile,
    spec_gate::{InputIntakeResult, InputSpecDescriptor},
};

pub const INPUT_VERSION: &str = "input.v1";
pub const INPUT_MANIFEST_VERSION: &str = "input-manifest.v1";
pub const INPUT_INDEX_VERSION: &str = "input-index.v1";
pub const INPUT_STATUS_VERSION: &str = "input-status.v1";
pub const INPUT_SNAPSHOT_VERSION: &str = "input-snapshot.v1";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum InputWorkspaceStatus {
    Missing,
    Ready,
    Degraded,
    Failed,
    Blocked,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InputSummary {
    pub intake: usize,
    pub draft_specs: usize,
    pub approved_specs: usize,
    pub projects: usize,
    pub issues: usize,
    pub blocked_issues: usize,
    pub high_risk_issues: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InputManifest {
    pub version: String,
    pub project_root: String,
    pub status: InputWorkspaceStatus,
    pub paths: BTreeMap<String, String>,
    pub legacy_paths: BTreeMap<String, String>,
    pub summary: InputSummary,
}

impl InputManifest {
    pub fn new(project_root: impl Into<String>, summary: InputSummary) -> Self {
        Self {
            version: INPUT_MANIFEST_VERSION.to_string(),
            project_root: project_root.into(),
            status: InputWorkspaceStatus::Ready,
            paths: input_paths(),
            legacy_paths: legacy_paths(),
            summary,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InputIndexEntry {
    pub id: String,
    pub title: String,
    pub path: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InputIndex {
    pub version: String,
    pub updated_at: u64,
    pub specs: Vec<InputIndexEntry>,
    pub projects: Vec<InputIndexEntry>,
    pub issues: Vec<InputIndexEntry>,
}

impl Default for InputIndex {
    fn default() -> Self {
        Self {
            version: INPUT_INDEX_VERSION.to_string(),
            updated_at: 0,
            specs: Vec::new(),
            projects: Vec::new(),
            issues: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InputStatusSnapshot {
    pub version: String,
    pub project_root: String,
    pub status: InputWorkspaceStatus,
    pub ready: bool,
    pub manifest_exists: bool,
    pub index_exists: bool,
    pub summary: InputSummary,
    pub missing_paths: Vec<String>,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InputSnapshot {
    pub version: String,
    pub project_root: String,
    pub ready: bool,
    pub status: InputStatusSnapshot,
    pub manifest: InputManifest,
    pub index: InputIndex,
    pub intake: Vec<InputIntakeResult>,
    pub specs: Vec<InputSpecDescriptor>,
    pub projects: Vec<InputProject>,
    pub issues: Vec<InputIssue>,
    pub relations: InputIssueRelationsFile,
}

pub fn input_paths() -> BTreeMap<String, String> {
    BTreeMap::from([
        ("root".to_string(), ".agentflow/input".to_string()),
        ("intake".to_string(), ".agentflow/input/intake".to_string()),
        ("specs".to_string(), ".agentflow/input/specs".to_string()),
        (
            "draftSpecs".to_string(),
            ".agentflow/input/specs/drafts".to_string(),
        ),
        (
            "approvedSpecs".to_string(),
            ".agentflow/input/specs/approved".to_string(),
        ),
        (
            "archivedSpecs".to_string(),
            ".agentflow/input/specs/archive".to_string(),
        ),
        (
            "projects".to_string(),
            ".agentflow/input/projects".to_string(),
        ),
        ("issues".to_string(), ".agentflow/input/issues".to_string()),
        (
            "relations".to_string(),
            ".agentflow/input/relations".to_string(),
        ),
        ("views".to_string(), ".agentflow/input/views".to_string()),
    ])
}

pub fn legacy_paths() -> BTreeMap<String, String> {
    BTreeMap::from([
        ("legacySpec".to_string(), ".agentflow/spec".to_string()),
        (
            "legacyGoalTree".to_string(),
            ".agentflow/goal-tree".to_string(),
        ),
    ])
}
