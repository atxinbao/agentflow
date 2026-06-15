use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const OUTPUT_AUDIT_VERSION: &str = "output-audit.v1";
pub const AUDIT_MANIFEST_VERSION: &str = "audit-manifest.v1";
pub const AUDIT_INDEX_VERSION: &str = "audit-index.v1";
pub const AUDIT_REQUEST_VERSION: &str = "audit-request.v1";
pub const AUDIT_FINDINGS_VERSION: &str = "audit-findings.v1";
pub const AUDIT_EVIDENCE_MAP_VERSION: &str = "audit-evidence-map.v1";
pub const AUDIT_TRACEABILITY_VERSION: &str = "audit-traceability.v1";

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
    pub run_exists: AuditCheckStatus,
    pub changed_files_recorded: AuditCheckStatus,
    pub allowed_write_paths_only: AuditCheckStatus,
    pub commands_recorded: AuditCheckStatus,
    pub high_risk_confirmed_if_needed: AuditCheckStatus,
    pub evidence_complete: AuditCheckStatus,
    pub public_delivery_complete: AuditCheckStatus,
}

impl AuditChecks {
    pub fn values(&self) -> [&AuditCheckStatus; 7] {
        [
            &self.run_exists,
            &self.changed_files_recorded,
            &self.allowed_write_paths_only,
            &self.commands_recorded,
            &self.high_risk_confirmed_if_needed,
            &self.evidence_complete,
            &self.public_delivery_complete,
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
