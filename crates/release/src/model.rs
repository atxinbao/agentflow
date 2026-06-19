use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const PUBLIC_RELEASE_SUMMARY_VERSION: &str = "public-release-summary.v1";
pub const DELIVERY_SUMMARY_VERSION: &str = "delivery-summary.v1";
pub const PROJECT_DELIVERY_SUMMARY_VERSION: &str = "project-delivery-summary.v1";
pub const TASK_PUBLIC_RECORD_TEMPLATE_VERSION: &str = "task-public-record-template.v1";
pub const CHANGELOG_TEMPLATE_VERSION: &str = "changelog-template.v1";
pub const RELEASE_NOTES_TEMPLATE_VERSION: &str = "release-notes-template.v1";
pub const PROJECT_RELEASE_FACTS_VERSION: &str = "project-release-facts.v1";
pub const PROJECT_RELEASE_INDEX_VERSION: &str = "project-release-index.v1";
pub const PROJECT_EXTERNAL_REVIEW_SURFACE_VERSION: &str = "project-external-review-surface.v1";
pub const PROJECT_EXTERNAL_REVIEW_INDEX_VERSION: &str = "project-external-review-index.v1";
pub const RELEASE_TAG_PROOF_VERSION: &str = "release-tag-proof.v1";
pub const REMOTE_RELEASE_PROOF_VERSION: &str = "remote-release-proof.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeliverySummary {
    pub version: String,
    pub public_record_template_version: String,
    pub issue_id: String,
    pub project_id: Option<String>,
    pub source_requirement_id: String,
    pub source_requirement_path: String,
    pub title: String,
    pub summary: String,
    pub status: String,
    pub evidence_status: String,
    pub evidence_path: Option<String>,
    pub pr_url: Option<String>,
    pub merge_commit: Option<String>,
    pub public_record_path: Option<String>,
    pub release_notes_url: Option<String>,
    pub validation_command_count: usize,
    pub public_record_targets: Vec<String>,
    pub public_record_markdown: String,
    pub summary_line: String,
    pub public_record_items: Vec<String>,
    pub missing_public_records: Vec<String>,
    pub updated_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectDeliverySummary {
    pub version: String,
    pub project_id: String,
    pub status: String,
    pub current_issue_id: Option<String>,
    pub published_count: usize,
    pub ready_count: usize,
    pub missing_count: usize,
    pub summary_line: String,
    pub public_record_items: Vec<String>,
    pub missing_public_records: Vec<String>,
    pub updated_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicReleaseEntry {
    pub issue_id: String,
    pub project_id: Option<String>,
    pub source_requirement_id: String,
    pub source_requirement_path: String,
    pub title: String,
    pub summary: String,
    pub evidence_status: String,
    pub current_state: String,
    pub pr_url: Option<String>,
    pub merge_commit: Option<String>,
    pub evidence_path: Option<String>,
    pub changelog_path: Option<String>,
    pub release_notes_url: Option<String>,
    pub validation_command_count: usize,
    pub public_record_targets: Vec<String>,
    pub updated_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicReleaseSummary {
    pub version: String,
    pub changelog_template_version: String,
    pub release_notes_template_version: String,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectReleaseFacts {
    pub version: String,
    pub project_id: String,
    pub project_title: String,
    pub current_state: String,
    #[serde(default = "default_publication_stage")]
    pub publication_stage: String,
    pub gate_status: String,
    pub gate_reason: String,
    pub completion_state: String,
    pub completion_outcome: Option<String>,
    pub delivery_status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub public_record_written_at: Option<u64>,
    pub changelog_path: String,
    pub release_notes_path: String,
    pub entry_count: usize,
    pub summary_line: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tag_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tag_commit_sha: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tag_proof_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remote_provider: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remote_release_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remote_release_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remote_release_commit_sha: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remote_release_proof_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artifact_manifest_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artifact_manifest_sha256: Option<String>,
    pub latest_event_id: Option<String>,
    pub published_at: Option<u64>,
    pub updated_at: u64,
}

fn default_publication_stage() -> String {
    "pending".to_string()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectReleaseIndexEntry {
    pub project_id: String,
    pub current_state: String,
    #[serde(default = "default_publication_stage")]
    pub publication_stage: String,
    pub gate_status: String,
    pub changelog_path: String,
    pub release_notes_path: String,
    pub published_at: Option<u64>,
    pub updated_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectReleaseIndex {
    pub version: String,
    pub updated_at: u64,
    pub releases: Vec<ProjectReleaseIndexEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReleaseTagProof {
    pub version: String,
    pub project_id: String,
    pub tag_name: String,
    pub tag_commit_sha: String,
    pub actor: String,
    pub recorded_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteReleaseProof {
    pub version: String,
    pub project_id: String,
    pub provider: String,
    pub release_id: String,
    pub release_url: String,
    pub tag_name: String,
    pub release_commit_sha: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artifact_manifest_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artifact_manifest_sha256: Option<String>,
    pub actor: String,
    pub recorded_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalReviewEvidenceEntry {
    pub issue_id: String,
    pub title: String,
    pub summary: String,
    pub evidence_status: String,
    pub evidence_path: Option<String>,
    pub validation_command_count: usize,
    pub public_record_targets: Vec<String>,
    pub pr_url: Option<String>,
    pub merge_commit: Option<String>,
    pub updated_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalReviewAuditSummary {
    pub latest_audit_id: Option<String>,
    pub latest_status: Option<String>,
    pub latest_report_path: Option<String>,
    pub total_count: usize,
    pub findings_count: usize,
    pub summary_line: String,
    #[serde(default)]
    pub findings: Vec<String>,
    #[serde(default)]
    pub evidence_gaps: Vec<String>,
    #[serde(default)]
    pub repair_recommendations: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectExternalReviewSurface {
    pub version: String,
    pub project_id: String,
    pub project_title: String,
    pub source_requirement_id: String,
    pub source_requirement_path: String,
    pub objective: String,
    pub review_status: String,
    pub release_state: String,
    pub release_summary_line: String,
    pub handoff_path: String,
    pub total_entries: usize,
    pub evidence_entries: Vec<ExternalReviewEvidenceEntry>,
    pub audit_summary: Option<ExternalReviewAuditSummary>,
    #[serde(default)]
    pub risk_items: Vec<String>,
    pub summary_line: String,
    pub generated_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectExternalReviewIndexEntry {
    pub project_id: String,
    pub review_status: String,
    pub handoff_path: String,
    pub generated_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectExternalReviewIndex {
    pub version: String,
    pub updated_at: u64,
    pub reviews: Vec<ProjectExternalReviewIndexEntry>,
}
