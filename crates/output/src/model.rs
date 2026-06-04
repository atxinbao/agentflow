use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const OUTPUT_MANIFEST_VERSION: &str = "output-manifest.v1";
pub const OUTPUT_INDEX_VERSION: &str = "output-index.v1";
pub const OUTPUT_STATUS_VERSION: &str = "output-status.v1";
pub const OUTPUT_SNAPSHOT_VERSION: &str = "output-snapshot.v1";
pub const OUTPUT_EVIDENCE_VERSION: &str = "output-evidence.v1";
pub const OUTPUT_RELEASE_DELIVERY_VERSION: &str = "output-release-delivery.v1";
pub const OUTPUT_PR_METADATA_VERSION: &str = "output-pr-metadata.v1";
pub const OUTPUT_AUDIT_VERSION: &str = "output-audit.v1";

pub const OUTPUT_DIRECTORIES: &[&str] = &[
    ".agentflow/output",
    ".agentflow/output/evidence",
    ".agentflow/output/release",
    ".agentflow/output/audit",
    ".agentflow/output/logs",
    ".agentflow/output/backup",
    ".agentflow/output/cache",
    ".agentflow/output/tmp",
];

pub const OUTPUT_REQUIRED_FILES: &[&str] = &[
    ".agentflow/output/manifest.json",
    ".agentflow/output/index.json",
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OutputWorkspaceStatus {
    Missing,
    Ready,
    Degraded,
    Failed,
    Blocked,
}

impl Default for OutputWorkspaceStatus {
    fn default() -> Self {
        Self::Missing
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputSummary {
    pub evidence: usize,
    pub release_deliveries: usize,
    pub audits: usize,
    pub logs: usize,
    pub backups: usize,
    pub incomplete_evidence: usize,
    pub incomplete_deliveries: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputManifest {
    pub version: String,
    pub project_root: String,
    pub status: OutputWorkspaceStatus,
    pub paths: BTreeMap<String, String>,
    pub summary: OutputSummary,
    pub updated_at: u64,
}

impl OutputManifest {
    pub fn new(project_root: impl Into<String>, summary: OutputSummary, updated_at: u64) -> Self {
        Self {
            version: OUTPUT_MANIFEST_VERSION.to_string(),
            project_root: project_root.into(),
            status: OutputWorkspaceStatus::Ready,
            paths: output_paths(),
            summary,
            updated_at,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputIndexEntry {
    pub run_id: String,
    pub issue_id: String,
    pub source_spec_id: String,
    pub path: String,
    pub status: String,
    pub updated_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputIndex {
    pub version: String,
    pub updated_at: u64,
    pub evidence: Vec<OutputIndexEntry>,
    pub release_deliveries: Vec<OutputIndexEntry>,
    pub audits: Vec<OutputIndexEntry>,
}

impl Default for OutputIndex {
    fn default() -> Self {
        Self {
            version: OUTPUT_INDEX_VERSION.to_string(),
            updated_at: 0,
            evidence: Vec::new(),
            release_deliveries: Vec::new(),
            audits: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputStatusSnapshot {
    pub version: String,
    pub project_root: String,
    pub status: OutputWorkspaceStatus,
    pub ready: bool,
    pub manifest_exists: bool,
    pub index_exists: bool,
    pub summary: OutputSummary,
    pub missing_paths: Vec<String>,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputSnapshot {
    pub version: String,
    pub project_root: String,
    pub ready: bool,
    pub status: OutputStatusSnapshot,
    pub manifest: OutputManifest,
    pub index: OutputIndex,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputEvidenceInput {
    pub issue_path: String,
    pub spec_path: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputEvidencePanel {
    pub snapshot_id: Option<String>,
    pub context_pack_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputEvidenceExecuteArtifacts {
    pub run: String,
    pub preflight: String,
    pub plan: String,
    pub result: String,
    pub checkpoint: Option<String>,
    pub diff: Option<String>,
    pub changed_files: Option<String>,
    pub diff_summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputCommandEvidence {
    pub command_id: String,
    pub label: String,
    pub exit_code: Option<i32>,
    pub record_path: String,
    pub stdout_path: Option<String>,
    pub stderr_path: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputValidationSummary {
    pub passed: bool,
    pub failed_commands: Vec<String>,
    pub skipped: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputManualProof {
    pub required: bool,
    pub notes: Vec<String>,
    pub screenshots: Vec<String>,
    pub recordings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputEvidence {
    pub version: String,
    pub run_id: String,
    pub issue_id: String,
    pub source_spec_id: String,
    pub risk_level: String,
    pub completed_at: u64,
    pub summary: String,
    pub input: OutputEvidenceInput,
    pub panel: OutputEvidencePanel,
    pub execute: OutputEvidenceExecuteArtifacts,
    pub commands: Vec<OutputCommandEvidence>,
    pub validation: OutputValidationSummary,
    pub manual_proof: OutputManualProof,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputReleaseDeliveryArtifacts {
    pub pr_draft: String,
    pub pr_metadata: String,
    pub review_checklist: String,
    pub changelog: String,
    pub release_note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputReleaseDelivery {
    pub version: String,
    pub run_id: String,
    pub issue_id: String,
    pub source_spec_id: String,
    pub risk_level: String,
    pub status: String,
    pub created_by: String,
    pub created_at: u64,
    pub evidence_path: String,
    pub execute_result_path: String,
    pub diff_summary_path: Option<String>,
    pub artifacts: OutputReleaseDeliveryArtifacts,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputPrMetadata {
    pub version: String,
    pub run_id: String,
    pub issue_id: String,
    pub source_spec_id: String,
    pub title: String,
    pub branch_name: Option<String>,
    pub remote_pr_url: Option<String>,
    pub status: String,
    pub created_remote_pr: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputAuditChecks {
    pub spec_aligned: Option<bool>,
    pub issue_acceptance_covered: Option<bool>,
    pub allowed_paths_only: Option<bool>,
    pub evidence_complete: Option<bool>,
    pub release_delivery_complete: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputAudit {
    pub version: String,
    pub run_id: String,
    pub issue_id: String,
    pub source_spec_id: String,
    pub status: String,
    pub created_by: String,
    pub created_at: u64,
    pub checks: OutputAuditChecks,
    pub findings: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputValidationReport {
    pub valid: bool,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

pub fn output_paths() -> BTreeMap<String, String> {
    BTreeMap::from([
        ("root".to_string(), ".agentflow/output".to_string()),
        (
            "evidence".to_string(),
            ".agentflow/output/evidence".to_string(),
        ),
        (
            "release".to_string(),
            ".agentflow/output/release".to_string(),
        ),
        ("audit".to_string(), ".agentflow/output/audit".to_string()),
        ("logs".to_string(), ".agentflow/output/logs".to_string()),
        ("backup".to_string(), ".agentflow/output/backup".to_string()),
        ("cache".to_string(), ".agentflow/output/cache".to_string()),
        ("tmp".to_string(), ".agentflow/output/tmp".to_string()),
    ])
}
