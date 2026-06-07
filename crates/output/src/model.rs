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
pub const AUDIT_MANIFEST_VERSION: &str = "audit-manifest.v1";
pub const AUDIT_INDEX_VERSION: &str = "audit-index.v1";
pub const AUDIT_REQUEST_VERSION: &str = "audit-request.v1";
pub const AUDIT_FINDINGS_VERSION: &str = "audit-findings.v1";
pub const AUDIT_EVIDENCE_MAP_VERSION: &str = "audit-evidence-map.v1";
pub const AUDIT_TRACEABILITY_VERSION: &str = "audit-traceability.v1";

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
    ".agentflow/output/audit/manifest.json",
    ".agentflow/output/audit/index.json",
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context_pack_path: Option<String>,
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
pub struct OutputValidationReport {
    pub valid: bool,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AuditStatus {
    Requested,
    Running,
    Passed,
    PassedWithWarnings,
    Failed,
    Cancelled,
}

impl AuditStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Running => "running",
            Self::Passed => "passed",
            Self::PassedWithWarnings => "passed-with-warnings",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AuditTrigger {
    HumanViaAgent,
    ReleaseAuto,
}

impl Default for AuditTrigger {
    fn default() -> Self {
        Self::HumanViaAgent
    }
}

impl AuditTrigger {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::HumanViaAgent => "human-via-agent",
            Self::ReleaseAuto => "release-auto",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AuditCheckStatus {
    Passed,
    Warning,
    Failed,
}

impl AuditCheckStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Passed => "passed",
            Self::Warning => "warning",
            Self::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AuditFindingSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl AuditFindingSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Critical => "critical",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditPaths {
    pub audit_root: String,
    pub index: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditManifestSummary {
    pub audits: usize,
    pub requested: usize,
    pub running: usize,
    pub passed: usize,
    pub passed_with_warnings: usize,
    pub failed: usize,
    pub cancelled: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditManifest {
    pub version: String,
    pub project_root: String,
    pub status: String,
    pub paths: AuditPaths,
    pub summary: AuditManifestSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditIndexEntry {
    pub audit_id: String,
    pub status: AuditStatus,
    #[serde(default)]
    pub trigger: AuditTrigger,
    pub requested_by: String,
    pub requested_at: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_delivery_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_run_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_issue_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_spec_id: Option<String>,
    pub report_path: String,
    pub audit_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditIndex {
    pub version: String,
    pub updated_at: u64,
    pub audits: Vec<AuditIndexEntry>,
}

impl Default for AuditIndex {
    fn default() -> Self {
        Self {
            version: AUDIT_INDEX_VERSION.to_string(),
            updated_at: 0,
            audits: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditScopeRef {
    pub kind: String,
    pub id: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditScope {
    pub description: String,
    pub refs: Vec<AuditScopeRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HumanAuditRequestDraft {
    pub reason: String,
    pub scope: AuditScope,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditRequestSource {
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delivery_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub issue_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spec_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditRequest {
    pub version: String,
    pub audit_id: String,
    #[serde(default)]
    pub trigger: AuditTrigger,
    pub requested_by: String,
    pub requested_at: u64,
    pub reason: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<AuditRequestSource>,
    pub scope: AuditScope,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditSummary {
    pub checks: usize,
    pub passed: usize,
    pub warnings: usize,
    pub failed: usize,
    pub findings: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditChecks {
    pub checkpoint_exists: AuditCheckStatus,
    pub changed_files_recorded: AuditCheckStatus,
    pub allowed_write_paths_only: AuditCheckStatus,
    pub commands_recorded: AuditCheckStatus,
    pub high_risk_confirmed_if_needed: AuditCheckStatus,
    pub evidence_complete: AuditCheckStatus,
    pub release_delivery_complete: AuditCheckStatus,
}

impl AuditChecks {
    pub fn values(&self) -> [&AuditCheckStatus; 7] {
        [
            &self.checkpoint_exists,
            &self.changed_files_recorded,
            &self.allowed_write_paths_only,
            &self.commands_recorded,
            &self.high_risk_confirmed_if_needed,
            &self.evidence_complete,
            &self.release_delivery_complete,
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HumanAudit {
    pub version: String,
    pub audit_id: String,
    #[serde(default)]
    pub trigger: AuditTrigger,
    pub requested_by: String,
    pub requested_at: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_delivery_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_run_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_issue_id: Option<String>,
    pub status: AuditStatus,
    pub summary: AuditSummary,
    pub checks: AuditChecks,
    pub paths: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditFinding {
    pub finding_id: String,
    pub severity: AuditFindingSeverity,
    pub category: String,
    pub title: String,
    pub detail: String,
    pub evidence_path: String,
    pub recommendation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditFindings {
    pub version: String,
    pub audit_id: String,
    pub findings: Vec<AuditFinding>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditEvidenceMap {
    pub version: String,
    pub audit_id: String,
    pub inputs: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditTraceabilityItem {
    pub layer: String,
    pub id: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditTraceability {
    pub version: String,
    pub audit_id: String,
    pub chain: Vec<AuditTraceabilityItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HumanAuditReport {
    pub request: AuditRequest,
    pub audit: HumanAudit,
    pub report_markdown: String,
    pub findings: AuditFindings,
    pub checklist_markdown: String,
    pub evidence_map: AuditEvidenceMap,
    pub traceability: AuditTraceability,
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
