use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const PUBLIC_RELEASE_SUMMARY_VERSION: &str = "public-release-summary.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicReleaseEntry {
    pub issue_id: String,
    pub project_id: Option<String>,
    pub title: String,
    pub current_state: String,
    pub pr_url: Option<String>,
    pub merge_commit: Option<String>,
    pub evidence_path: Option<String>,
    pub changelog_path: Option<String>,
    pub release_notes_url: Option<String>,
    pub updated_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicReleaseSummary {
    pub version: String,
    pub generated_at: u64,
    pub entries: Vec<PublicReleaseEntry>,
    pub changelog_markdown: String,
    pub release_notes_markdown: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublicReleaseDocumentTarget {
    pub changelog_path: PathBuf,
    pub release_notes_path: PathBuf,
}

impl Default for PublicReleaseDocumentTarget {
    fn default() -> Self {
        Self {
            changelog_path: PathBuf::from("CHANGELOG.md"),
            release_notes_path: PathBuf::from("docs/release-notes/agentflow-release-notes.md"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicReleaseDocumentPaths {
    pub changelog_path: String,
    pub release_notes_path: String,
}
