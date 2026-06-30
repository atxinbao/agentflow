use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use agentflow_audit::{load_audit_index, load_audit_report, load_audit_result_summary};
use agentflow_event_store::{
    classify_task_event, map_task_event_to_runtime_event, replay_runtime_events,
    replay_task_events, ReplayFilter, TaskEvent,
};
use agentflow_ontology::{
    core_missing_evidence_reports_for_completeness_policy,
    evaluate_core_evidence_completeness_policy,
    software_dev_reference_evidence_completeness_policy,
    software_dev_reference_evidence_fixture_packs, CoreEvidenceCompletenessEvaluation,
    CoreEvidenceCompletenessPolicy, CoreEvidencePack, CoreEvidenceTraceRefs,
};
use agentflow_pack::{
    software_dev_connector_definition, software_dev_domain_definition,
    software_dev_surface_definition, ui_design_connector_definition, ui_design_domain_definition,
    ui_design_surface_definition, validate_connector_definition, validate_domain_definition,
    validate_surface_definition, PackConnector, PackConnectorDefinition, PackDomainDefinition,
    PackRegistryEntry, PackSurfaceDefinition, PackValidationStatus,
};
use agentflow_spec::{
    read_requirement_preview_runtime, read_spec_issue, read_spec_project, SpecIssue, SpecPriority,
    SpecRequiredAgentRole,
};
use agentflow_task_artifacts::{
    load_task_evidence, load_task_run, load_task_session_evidence,
    load_task_session_history_record, load_task_session_recovery_summary,
};

use crate::model::{
    ProjectIssueLanes, ProjectionDeliverySummary, ProjectionPublicDelivery, TaskProjection,
    TaskTimelineItem,
};
use crate::storage::{
    load_issue_status_index, load_project_projection, load_requirement_preview_index,
    load_requirement_preview_projection, load_spec_loop_projection, load_task_projection,
};

pub const PROJECTION_QUERY_SURFACE_VERSION: &str = "projection-query-surface.v1";
pub const EVIDENCE_KERNEL_READ_MODEL_VERSION: &str = "evidence-kernel-read-model.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectionDefinitionVersions {
    pub ontology_version: String,
    pub action_contract_version: String,
    pub role_policy_version: String,
    pub state_machine_version: String,
}

pub const PROJECTION_FRESHNESS_RECEIPT_VERSION: &str = "projection-freshness-receipt.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectionFreshnessReceipt {
    pub version: String,
    pub receipt_id: String,
    pub projection_ref: String,
    #[serde(default)]
    pub source_refs: Vec<String>,
    pub source_digest: String,
    pub rebuild_receipt_ref: String,
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stale_reason: Option<String>,
    pub generated_at: u64,
    pub writes_authority: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectionFreshness {
    pub projection_version: String,
    pub query_surface_version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_event_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_event_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_event_timestamp: Option<u64>,
    pub last_rebuilt_at: u64,
    pub staleness: String,
    pub definition_versions: ProjectionDefinitionVersions,
    pub receipt: ProjectionFreshnessReceipt,
    #[serde(default)]
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectionFeedbackRoute {
    pub status: String,
    pub route: String,
    pub reason: String,
    pub source_surface_key: String,
    pub target_authority: String,
    pub proposal_kind: String,
    pub requires_confirmation: bool,
    pub confirmation_boundary: String,
    pub writes_authority: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ViewActionHint {
    pub key: String,
    pub label: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeEventRow {
    pub event_id: String,
    pub event_type: String,
    pub timestamp: u64,
    pub actor_role: String,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkLoopEventView {
    pub event_id: String,
    pub event_type: String,
    pub category: String,
    pub stage_key: String,
    pub stage_label: String,
    pub timestamp: u64,
    pub actor_role: String,
    pub actor_kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from_state: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to_state: Option<String>,
    pub summary: String,
    #[serde(default)]
    pub evidence_refs: Vec<String>,
    #[serde(default)]
    pub artifact_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkLoopEvidenceSummaryView {
    pub status: String,
    pub summary: String,
    #[serde(default)]
    pub verification_refs: Vec<String>,
    #[serde(default)]
    pub session_refs: Vec<String>,
    #[serde(default)]
    pub delivery_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceSourceSummaryView {
    pub evidence_id: String,
    pub source_type: String,
    pub status: String,
    pub subject_ref: String,
    pub producer_role: String,
    pub artifact_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceMissingReasonView {
    pub report_id: String,
    pub source_type: String,
    pub outcome: String,
    pub current_state: String,
    pub expected_proof: String,
    pub remediation_hint: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence_ref: Option<String>,
    #[serde(default)]
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceCompletenessReadModelView {
    pub policy_id: String,
    pub outcome: String,
    #[serde(default)]
    pub reasons: Vec<String>,
    #[serde(default)]
    pub satisfied_groups: Vec<String>,
    #[serde(default)]
    pub missing_groups: Vec<String>,
    #[serde(default)]
    pub deferred_groups: Vec<String>,
    #[serde(default)]
    pub invalid_evidence_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceKernelReadModelView {
    pub version: String,
    pub status: String,
    pub policy_id: String,
    pub authority: bool,
    pub readonly: bool,
    #[serde(default)]
    pub source_summaries: Vec<EvidenceSourceSummaryView>,
    #[serde(default)]
    pub trace_refs: Vec<String>,
    #[serde(default)]
    pub missing_reasons: Vec<EvidenceMissingReasonView>,
    pub completeness: EvidenceCompletenessReadModelView,
    pub freshness: ProjectionFreshness,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssuePreviewItem {
    pub issue_id: String,
    pub title: String,
    pub summary: String,
    pub priority: String,
    pub required_agent_role: String,
    #[serde(default)]
    pub blocked_by: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectDependencyEdge {
    pub issue_id: String,
    pub depends_on_issue_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectRunSummary {
    pub issue_id: String,
    pub run_id: String,
    pub run_status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub branch_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequirementIntakeView {
    pub requirement_id: String,
    pub state: String,
    pub classification: String,
    #[serde(default)]
    pub ambiguities: Vec<String>,
    #[serde(default)]
    pub boundary_notes: Vec<String>,
    #[serde(default)]
    pub allowed_actions: Vec<ViewActionHint>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_event_id: Option<String>,
    pub freshness: ProjectionFreshness,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpecPreviewView {
    pub spec_id: String,
    pub state: String,
    pub requirement_ref: String,
    pub preview_summary: String,
    #[serde(default)]
    pub acceptance_criteria: Vec<String>,
    #[serde(default)]
    pub issue_preview: Vec<IssuePreviewItem>,
    pub confirmation_state: String,
    #[serde(default)]
    pub allowed_actions: Vec<ViewActionHint>,
    pub freshness: ProjectionFreshness,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpecLoopStageView {
    pub stage: String,
    pub path: String,
    pub status: String,
    pub authority: String,
    pub authority_layer: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_state: Option<String>,
    #[serde(default)]
    pub input_refs: Vec<String>,
    #[serde(default)]
    pub output_refs: Vec<String>,
    #[serde(default)]
    pub evidence_refs: Vec<String>,
    pub summary: String,
    pub updated_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpecLoopAuthorityLayerView {
    pub authority_layer: String,
    pub path: String,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpecLoopTraceabilityView {
    pub from_ref: String,
    pub to_ref: String,
    pub relation: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpecLoopActionProposalView {
    pub proposal_ref: String,
    pub action_type: String,
    pub target_object_type: String,
    pub target_object_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_object_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_object_id: Option<String>,
    pub actor_role: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub handoff_rule: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command_status: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decision_status: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accepted_action_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proposal_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decision_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accepted_action_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpecLoopView {
    pub requirement_id: String,
    pub requirement_path: String,
    pub project_id: String,
    pub project_title: String,
    pub lifecycle: String,
    pub current_state: String,
    pub manifest_path: String,
    pub runtime_path: String,
    pub next_recommended_action: String,
    pub next_recommended_action_label: String,
    pub next_recommended_action_reason: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub materialized_project_id: Option<String>,
    #[serde(default)]
    pub materialized_issue_ids: Vec<String>,
    #[serde(default)]
    pub stages: Vec<SpecLoopStageView>,
    #[serde(default)]
    pub authority_layers: Vec<SpecLoopAuthorityLayerView>,
    #[serde(default)]
    pub traceability: Vec<SpecLoopTraceabilityView>,
    #[serde(default)]
    pub runtime_action_proposals: Vec<SpecLoopActionProposalView>,
    #[serde(default)]
    pub allowed_actions: Vec<ViewActionHint>,
    pub freshness: ProjectionFreshness,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectHomeView {
    pub project_id: String,
    pub title: String,
    pub objective: String,
    pub state_summary: String,
    pub issue_groups: ProjectIssueLanes,
    #[serde(default)]
    pub dependency_graph: Vec<ProjectDependencyEdge>,
    #[serde(default)]
    pub active_runs: Vec<ProjectRunSummary>,
    #[serde(default)]
    pub blocked_items: Vec<String>,
    #[serde(default)]
    pub recent_events: Vec<RuntimeEventRow>,
    #[serde(default)]
    pub allowed_actions: Vec<ViewActionHint>,
    pub freshness: ProjectionFreshness,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskWorkbenchView {
    pub issue_id: String,
    pub title: String,
    pub summary: String,
    pub issue_state: String,
    pub run_state: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_run: Option<String>,
    #[serde(default)]
    pub evidence_refs: Vec<String>,
    #[serde(default)]
    pub artifact_refs: Vec<String>,
    #[serde(default)]
    pub acceptance_mapping: Vec<String>,
    #[serde(default)]
    pub allowed_actions: Vec<ViewActionHint>,
    #[serde(default)]
    pub blocked_reasons: Vec<String>,
    pub state_explanation: String,
    pub evidence_summary: WorkLoopEvidenceSummaryView,
    #[serde(default)]
    pub event_stream: Vec<WorkLoopEventView>,
    #[serde(default)]
    pub timeline: Vec<TaskTimelineItem>,
    pub freshness: ProjectionFreshness,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkLoopRunView {
    pub issue_id: String,
    pub run_id: String,
    pub issue_state: String,
    pub run_state: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub branch_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_status: Option<String>,
    pub state_explanation: String,
    pub evidence_summary: WorkLoopEvidenceSummaryView,
    #[serde(default)]
    pub event_stream: Vec<WorkLoopEventView>,
    pub freshness: ProjectionFreshness,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkLoopSessionView {
    pub issue_id: String,
    pub run_id: String,
    pub session_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_owner: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_status: Option<String>,
    #[serde(default)]
    pub attempt_count: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub started_at: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_heartbeat_at: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recovery_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resumed_from_attempt: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retry_policy: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retryable: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub terminal_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_error: Option<String>,
    pub state_explanation: String,
    pub evidence_summary: WorkLoopEvidenceSummaryView,
    #[serde(default)]
    pub event_stream: Vec<WorkLoopEventView>,
    pub freshness: ProjectionFreshness,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditSurfaceView {
    pub audit_id: String,
    pub audit_state: String,
    pub scope: String,
    #[serde(default)]
    pub evidence_map: Vec<String>,
    #[serde(default)]
    pub findings: Vec<String>,
    #[serde(default)]
    pub traceability: Vec<String>,
    #[serde(default)]
    pub allowed_actions: Vec<ViewActionHint>,
    pub freshness: ProjectionFreshness,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeliveryPackageView {
    pub issue_id: String,
    pub delivery_state: String,
    #[serde(default)]
    pub artifact_refs: Vec<String>,
    #[serde(default)]
    pub verification_logs: Vec<String>,
    #[serde(default)]
    pub acceptance_mapping: Vec<String>,
    pub build_agent_summary: String,
    #[serde(default)]
    pub allowed_actions: Vec<ViewActionHint>,
    pub freshness: ProjectionFreshness,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeHealthView {
    pub project_id: String,
    pub project_status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_issue_id: Option<String>,
    pub active_issue_count: usize,
    pub blocked_issue_count: usize,
    #[serde(default)]
    pub warnings: Vec<String>,
    #[serde(default)]
    pub allowed_actions: Vec<ViewActionHint>,
    pub freshness: ProjectionFreshness,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackListItemView {
    pub pack_id: String,
    pub name: String,
    pub pack_type: String,
    pub pack_version: String,
    pub registered: bool,
    pub validation_status: String,
    pub manifest_path: String,
    #[serde(default)]
    pub source_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackReadinessView {
    pub pack_id: String,
    pub status: String,
    pub manifest_valid: bool,
    pub domain_valid: bool,
    pub surface_valid: bool,
    pub connector_valid: bool,
    #[serde(default)]
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackDefinitionStatusIndexItem {
    pub pack_id: String,
    pub definition_kind: String,
    pub status: String,
    pub reason: String,
    pub command_execution_allowed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackDomainObjectIndexItem {
    pub pack_id: String,
    pub object_type_id: String,
    pub label: String,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackSurfacePageIndexItem {
    pub pack_id: String,
    pub page_id: String,
    pub label: String,
    pub page_kind: String,
    pub view_model_ref: String,
    #[serde(default)]
    pub command_entry_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackViewModelMappingIndexItem {
    pub pack_id: String,
    pub mapping_id: String,
    pub page_id: String,
    pub projection_ref: String,
    pub view_model_ref: String,
    pub status: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackConnectorCapabilityIndexItem {
    pub pack_id: String,
    pub connector_id: String,
    pub provider_type: String,
    pub action_id: String,
    pub command_type: String,
    pub required_capability: String,
    pub writes_external: bool,
    pub evidence_output: String,
    pub status: String,
    pub disabled_reason: String,
    pub command_execution_allowed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackIndustryWorkbenchItem {
    pub pack_id: String,
    pub workbench_id: String,
    pub page_id: String,
    pub label: String,
    pub primary_object_type: String,
    pub timeline_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackIndustryWorkbenchView {
    pub version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_pack_id: Option<String>,
    #[serde(default)]
    pub pack_list: Vec<PackListItemView>,
    #[serde(default)]
    pub pack_readiness: Vec<PackReadinessView>,
    #[serde(default)]
    pub definition_status_index: Vec<PackDefinitionStatusIndexItem>,
    #[serde(default)]
    pub domain_object_index: Vec<PackDomainObjectIndexItem>,
    #[serde(default)]
    pub surface_page_index: Vec<PackSurfacePageIndexItem>,
    #[serde(default)]
    pub view_model_mapping_index: Vec<PackViewModelMappingIndexItem>,
    #[serde(default)]
    pub connector_capability_index: Vec<PackConnectorCapabilityIndexItem>,
    #[serde(default)]
    pub industry_workbenches: Vec<PackIndustryWorkbenchItem>,
    #[serde(default)]
    pub source_refs: Vec<String>,
    pub authority: bool,
    pub freshness: ProjectionFreshness,
    #[serde(default)]
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectionSurfaceQueryView {
    pub name: String,
    #[serde(default)]
    pub args: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectionSurfaceReadModelView {
    pub key: String,
    pub kind: String,
    pub object_type: String,
    pub object_id: String,
    pub title: String,
    pub status: String,
    pub query: ProjectionSurfaceQueryView,
    pub projection_path: String,
    #[serde(default)]
    pub source_refs: Vec<String>,
    pub authority: bool,
    pub freshness: ProjectionFreshness,
    pub feedback: ProjectionFeedbackRoute,
    #[serde(default)]
    pub missing_facts: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectionSurfaceCatalogView {
    pub version: String,
    pub query_surface_version: String,
    #[serde(default)]
    pub read_models: Vec<ProjectionSurfaceReadModelView>,
    pub freshness: ProjectionFreshness,
    #[serde(default)]
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ProjectionScope {
    RequirementPreview { project_id: String },
    Project { project_id: String },
    Issue { issue_id: String },
    Audit { source_issue_id: Option<String> },
}

impl ProjectionScope {
    fn key(&self) -> String {
        match self {
            Self::RequirementPreview { project_id } => {
                format!("requirement-preview:{project_id}")
            }
            Self::Project { project_id } => format!("project:{project_id}"),
            Self::Issue { issue_id } => format!("issue:{issue_id}"),
            Self::Audit {
                source_issue_id: Some(issue_id),
            } => format!("audit:issue:{issue_id}"),
            Self::Audit {
                source_issue_id: None,
            } => "audit:unscoped".to_string(),
        }
    }

    fn source_refs(&self) -> Vec<String> {
        match self {
            Self::RequirementPreview { project_id } => vec![
                format!(".agentflow/spec/requirements/{project_id}/**"),
                format!(".agentflow/events/task-events/**?projectId={project_id}"),
            ],
            Self::Project { project_id } => vec![
                format!(".agentflow/spec/projects/{project_id}.json"),
                format!(".agentflow/events/task-events/**?projectId={project_id}"),
            ],
            Self::Issue { issue_id } => vec![
                format!(".agentflow/spec/issues/{issue_id}.json"),
                format!(".agentflow/events/task-events/**?issueId={issue_id}"),
            ],
            Self::Audit {
                source_issue_id: Some(issue_id),
            } => vec![
                format!(".agentflow/audit/**?sourceIssueId={issue_id}"),
                format!(".agentflow/events/task-events/**?issueId={issue_id}"),
            ],
            Self::Audit {
                source_issue_id: None,
            } => vec![".agentflow/audit/**".to_string()],
        }
    }
}

pub fn get_projection_surface_catalog(
    project_root: impl AsRef<Path>,
) -> Result<ProjectionSurfaceCatalogView> {
    let project_root = project_root.as_ref();
    let mut warnings = Vec::new();
    let mut read_models = Vec::new();
    let mut project_ids = BTreeSet::new();

    match load_requirement_preview_index(project_root) {
        Ok(index) => {
            for preview in index.previews {
                project_ids.insert(preview.project_id.clone());
                let freshness = explain_projection_staleness(
                    project_root,
                    ProjectionScope::RequirementPreview {
                        project_id: preview.project_id.clone(),
                    },
                    "requirement-preview-projection.v1",
                    preview.updated_at,
                    None,
                )?;
                read_models.push(surface_read_model(
                    "requirement-intake",
                    "requirement",
                    &preview.requirement_id,
                    &format!("Requirement {}", preview.requirement_id),
                    &preview.current_state,
                    "get_requirement_intake_view",
                    vec![preview.requirement_id.clone()],
                    &preview.projection_path,
                    vec![format!("docs/requirements/{}.md", preview.requirement_id)],
                    freshness.clone(),
                ));
                read_models.push(surface_read_model(
                    "spec-preview",
                    "requirement",
                    &preview.requirement_id,
                    &format!("Spec Preview {}", preview.requirement_id),
                    &preview.current_state,
                    "get_spec_preview_view",
                    vec![preview.requirement_id.clone()],
                    &preview.projection_path,
                    vec![preview.projection_path.clone()],
                    freshness.clone(),
                ));
                read_models.push(surface_read_model(
                    "spec-loop",
                    "requirement",
                    &preview.requirement_id,
                    &format!("Spec Loop {}", preview.requirement_id),
                    &preview.current_state,
                    "get_spec_loop_view",
                    vec![preview.requirement_id.clone()],
                    &format!(
                        ".agentflow/projections/spec-loops/{}.json",
                        preview.requirement_id
                    ),
                    vec![preview.projection_path],
                    freshness,
                ));
            }
        }
        Err(error) => warnings.push(format!("requirement-preview-index-missing: {error}")),
    }

    match load_issue_status_index(project_root) {
        Ok(index) => {
            for issue in index.issues {
                if let Some(project_id) = issue.project_id.clone() {
                    project_ids.insert(project_id);
                }
                let task_projection = load_task_projection(project_root, &issue.issue_id).ok();
                let freshness = if let Some(task) = task_projection.as_ref() {
                    let projection_cursor = task
                        .timeline
                        .iter()
                        .flat_map(|item| item.events.iter().map(|event| event.event_id.clone()))
                        .last();
                    explain_projection_staleness(
                        project_root,
                        ProjectionScope::Issue {
                            issue_id: issue.issue_id.clone(),
                        },
                        &task.version,
                        task.updated_at,
                        projection_cursor,
                    )?
                } else {
                    missing_freshness("task-projection-missing")
                };
                let mut missing_facts = Vec::new();
                if task_projection.is_none() {
                    missing_facts.push(issue.projection_path.clone());
                }
                read_models.push(surface_read_model_with_missing(
                    "task-workbench",
                    "issue",
                    &issue.issue_id,
                    &issue.title,
                    &issue.display_status,
                    "get_task_workbench_view",
                    vec![issue.issue_id.clone()],
                    &issue.projection_path,
                    vec![format!(".agentflow/spec/issues/{}.json", issue.issue_id)],
                    freshness.clone(),
                    missing_facts.clone(),
                ));
                read_models.push(surface_read_model_with_missing(
                    "delivery-package",
                    "issue",
                    &issue.issue_id,
                    &format!("Delivery {}", issue.title),
                    task_projection
                        .as_ref()
                        .map(|task| task.delivery.status.as_str())
                        .unwrap_or("missing"),
                    "get_delivery_package_view",
                    vec![issue.issue_id.clone()],
                    &issue.projection_path,
                    vec![format!(".agentflow/spec/issues/{}.json", issue.issue_id)],
                    freshness,
                    missing_facts,
                ));
            }
        }
        Err(error) => warnings.push(format!("issue-status-index-missing: {error}")),
    }

    for project_id in project_ids {
        let project_projection = load_project_projection(project_root, &project_id).ok();
        let (title, status, projection_path, freshness, missing_facts) =
            if let Some(project) = project_projection.as_ref() {
                (
                    project.title.clone(),
                    project.status.clone(),
                    format!(
                        ".agentflow/projections/projects/{}.json",
                        project.project_id
                    ),
                    explain_projection_staleness(
                        project_root,
                        ProjectionScope::Project {
                            project_id: project.project_id.clone(),
                        },
                        &project.version,
                        project.updated_at,
                        None,
                    )?,
                    Vec::new(),
                )
            } else {
                (
                    project_id.clone(),
                    "missing".to_string(),
                    format!(".agentflow/projections/projects/{}.json", project_id),
                    missing_freshness("project-projection-missing"),
                    vec![format!(
                        ".agentflow/projections/projects/{}.json",
                        project_id
                    )],
                )
            };
        read_models.push(surface_read_model_with_missing(
            "project-home",
            "project",
            &project_id,
            &title,
            &status,
            "get_project_home_view",
            vec![project_id.clone()],
            &projection_path,
            vec![format!(".agentflow/spec/projects/{}.json", project_id)],
            freshness.clone(),
            missing_facts.clone(),
        ));
        read_models.push(surface_read_model_with_missing(
            "runtime-health",
            "project",
            &project_id,
            &format!("Runtime Health {title}"),
            &status,
            "get_runtime_health_view",
            vec![project_id.clone()],
            &projection_path,
            vec![projection_path.clone()],
            freshness,
            missing_facts,
        ));
    }

    match load_audit_index(project_root) {
        Ok(index) => {
            for audit in index.audits {
                let freshness = explain_projection_staleness(
                    project_root,
                    ProjectionScope::Audit {
                        source_issue_id: audit.source_issue_id.clone(),
                    },
                    "audit-result-summary.v1",
                    audit.requested_at,
                    None,
                )?;
                read_models.push(surface_read_model(
                    "audit-surface",
                    "audit",
                    &audit.audit_id,
                    &format!("Audit {}", audit.audit_id),
                    audit.status.as_str(),
                    "get_audit_surface_view",
                    vec![audit.audit_id.clone()],
                    &audit.audit_path,
                    vec![audit.report_path],
                    freshness,
                ));
            }
        }
        Err(error) => warnings.push(format!("audit-index-missing: {error}")),
    }

    match build_pack_bundles(project_root) {
        Ok(bundles) => {
            for bundle in bundles {
                read_models.push(surface_read_model(
                    "pack-industry-workbench",
                    "pack",
                    &bundle.pack_id,
                    &format!("{} Pack Workbench", bundle.name),
                    &bundle.readiness_status(),
                    "get_pack_industry_workbench_view",
                    vec![bundle.pack_id.clone()],
                    &format!(".agentflow/projections/packs/{}.json", bundle.pack_id),
                    bundle.source_refs(),
                    missing_freshness("pack-projection-readonly-derived"),
                ));
            }
        }
        Err(error) => warnings.push(format!("pack-registry-unreadable: {error}")),
    }

    read_models.push(surface_read_model(
        "evidence-kernel",
        "core",
        "core-evidence",
        "Evidence Kernel",
        "readonly",
        "get_evidence_kernel_view",
        Vec::new(),
        ".agentflow/projections/evidence/core-evidence.json",
        vec![
            "crates/ontology/src/evidence.rs".to_string(),
            "docs/architecture/068-evidence-projection-read-model-v1.md".to_string(),
        ],
        evidence_kernel_freshness(),
    ));

    read_models.sort_by(|left, right| left.key.cmp(&right.key));
    let freshness = catalog_freshness(&read_models, warnings.clone());

    Ok(ProjectionSurfaceCatalogView {
        version: "projection-surface-catalog.v1".to_string(),
        query_surface_version: PROJECTION_QUERY_SURFACE_VERSION.to_string(),
        read_models,
        freshness,
        warnings,
    })
}

pub fn get_evidence_kernel_view(
    project_root: impl AsRef<Path>,
) -> Result<EvidenceKernelReadModelView> {
    let _ = project_root.as_ref();
    let policy = software_dev_reference_evidence_completeness_policy();
    let packs = software_dev_reference_evidence_fixture_packs();
    Ok(project_evidence_kernel_read_model(&policy, &packs))
}

pub fn project_evidence_kernel_read_model(
    policy: &CoreEvidenceCompletenessPolicy,
    packs: &[CoreEvidencePack],
) -> EvidenceKernelReadModelView {
    let evaluation = evaluate_core_evidence_completeness_policy(policy, packs);
    let missing_reports = core_missing_evidence_reports_for_completeness_policy(policy, packs);
    let status = evidence_projection_status(&evaluation);
    let source_summaries = packs
        .iter()
        .map(|pack| EvidenceSourceSummaryView {
            evidence_id: pack.evidence_id.clone(),
            source_type: pack.source_type.clone(),
            status: pack.status.clone(),
            subject_ref: pack.subject.subject_ref.clone(),
            producer_role: pack.producer.role_ref.clone(),
            artifact_count: pack.artifact_refs.len(),
        })
        .collect::<Vec<_>>();
    let trace_refs = packs
        .iter()
        .flat_map(|pack| evidence_trace_refs(&pack.trace_refs))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let missing_reasons = missing_reports
        .into_iter()
        .map(|report| EvidenceMissingReasonView {
            report_id: report.report_id,
            source_type: report.source_type,
            outcome: report.outcome,
            current_state: report.current_state,
            expected_proof: report.expected_proof,
            remediation_hint: report.remediation_hint,
            evidence_ref: report.evidence_ref,
            reasons: report.reasons,
        })
        .collect::<Vec<_>>();

    EvidenceKernelReadModelView {
        version: EVIDENCE_KERNEL_READ_MODEL_VERSION.to_string(),
        status,
        policy_id: policy.policy_id.clone(),
        authority: false,
        readonly: true,
        source_summaries,
        trace_refs,
        missing_reasons,
        completeness: EvidenceCompletenessReadModelView {
            policy_id: evaluation.policy_id,
            outcome: evaluation.outcome,
            reasons: evaluation.reasons,
            satisfied_groups: evaluation.satisfied_groups,
            missing_groups: evaluation.missing_groups,
            deferred_groups: evaluation.deferred_groups,
            invalid_evidence_ids: evaluation.invalid_evidence_ids,
        },
        freshness: evidence_kernel_freshness(),
    }
}

pub fn evidence_kernel_invalid_missing_projection_fixtures() -> Vec<EvidenceKernelReadModelView> {
    let policy = software_dev_reference_evidence_completeness_policy();
    let mut invalid_pack = software_dev_reference_evidence_fixture_packs()
        .into_iter()
        .next()
        .expect("software dev reference fixture must exist");
    invalid_pack.evidence_id = "evidence-reference-invalid-digest".to_string();
    invalid_pack.digest.value.clear();

    vec![
        project_evidence_kernel_read_model(&policy, &[]),
        project_evidence_kernel_read_model(&policy, &[invalid_pack]),
    ]
}

pub fn get_requirement_intake_view(
    project_root: impl AsRef<Path>,
    requirement_id: &str,
) -> Result<RequirementIntakeView> {
    let runtime = read_requirement_preview_runtime(&project_root, requirement_id)?;
    let projection = load_requirement_preview_projection(&project_root, requirement_id)?;
    let freshness = explain_projection_staleness(
        &project_root,
        ProjectionScope::RequirementPreview {
            project_id: runtime.project_id.clone(),
        },
        &projection.version,
        projection.updated_at,
        None,
    )?;
    let boundary_notes = runtime
        .goal_draft
        .non_goals
        .iter()
        .chain(runtime.goal_draft.constraints.iter())
        .cloned()
        .collect::<Vec<_>>();
    let ambiguities = runtime
        .intake
        .missing_information
        .iter()
        .chain(runtime.intake.clarification_questions.iter())
        .cloned()
        .collect::<Vec<_>>();

    Ok(RequirementIntakeView {
        requirement_id: runtime.requirement_id,
        state: projection.current_state,
        classification: runtime.intake.detected_intent.as_str().to_string(),
        ambiguities,
        boundary_notes,
        allowed_actions: next_action_hints(
            &projection.next_recommended_action,
            &projection.next_recommended_action_label,
            &projection.next_recommended_action_reason,
        ),
        last_event_id: freshness.last_event_id.clone(),
        freshness,
    })
}

pub fn get_spec_preview_view(
    project_root: impl AsRef<Path>,
    requirement_id: &str,
) -> Result<SpecPreviewView> {
    let runtime = read_requirement_preview_runtime(&project_root, requirement_id)?;
    let projection = load_requirement_preview_projection(&project_root, requirement_id)?;
    let freshness = explain_projection_staleness(
        &project_root,
        ProjectionScope::RequirementPreview {
            project_id: runtime.project_id.clone(),
        },
        &projection.version,
        projection.updated_at,
        None,
    )?;

    let mut acceptance_criteria = runtime.goal_draft.success_criteria.clone();
    let mut issue_preview = Vec::new();
    if let Some(plan_draft) = runtime.plan_draft.as_ref() {
        for draft in &plan_draft.issue_contract_drafts {
            acceptance_criteria.extend(draft.acceptance_criteria.clone());
            issue_preview.push(IssuePreviewItem {
                issue_id: draft.issue_draft_id.clone(),
                title: draft.title.clone(),
                summary: draft.goal.clone(),
                priority: priority_label(&draft.priority),
                required_agent_role: required_role_label(&draft.suggested_agent_role),
                blocked_by: draft.dependencies.clone(),
            });
        }
    } else {
        for issue_id in &runtime.materialized_issue_ids {
            let issue = read_spec_issue(&project_root, issue_id)?;
            acceptance_criteria.extend(issue.validation_commands.clone());
            issue_preview.push(spec_issue_preview_item(&issue));
        }
    }
    acceptance_criteria.sort();
    acceptance_criteria.dedup();

    let confirmation_state = runtime
        .confirmation_records
        .last()
        .map(|record| record.decision.clone())
        .unwrap_or_else(|| runtime.current_state.clone());

    Ok(SpecPreviewView {
        spec_id: runtime.project_id.clone(),
        state: projection.current_state,
        requirement_ref: runtime.requirement_id,
        preview_summary: runtime.goal_draft.outcome,
        acceptance_criteria,
        issue_preview,
        confirmation_state,
        allowed_actions: next_action_hints(
            &projection.next_recommended_action,
            &projection.next_recommended_action_label,
            &projection.next_recommended_action_reason,
        ),
        freshness,
    })
}

pub fn get_spec_loop_view(
    project_root: impl AsRef<Path>,
    requirement_id: &str,
) -> Result<SpecLoopView> {
    let runtime = read_requirement_preview_runtime(&project_root, requirement_id)?;
    let projection = load_spec_loop_projection(&project_root, requirement_id)?;
    let freshness = explain_projection_staleness(
        &project_root,
        ProjectionScope::RequirementPreview {
            project_id: runtime.project_id.clone(),
        },
        &projection.version,
        projection.updated_at,
        None,
    )?;

    Ok(SpecLoopView {
        requirement_id: projection.requirement_id,
        requirement_path: projection.requirement_path,
        project_id: projection.project_id,
        project_title: projection.project_title,
        lifecycle: projection.lifecycle,
        current_state: projection.current_state,
        manifest_path: projection.manifest_path,
        runtime_path: projection.runtime_path,
        next_recommended_action: projection.next_recommended_action.clone(),
        next_recommended_action_label: projection.next_recommended_action_label.clone(),
        next_recommended_action_reason: projection.next_recommended_action_reason.clone(),
        materialized_project_id: projection.materialized_project_id,
        materialized_issue_ids: projection.materialized_issue_ids,
        stages: projection
            .stages
            .into_iter()
            .map(|stage| SpecLoopStageView {
                stage: stage.stage,
                path: stage.path,
                status: stage.status,
                authority: stage.authority,
                authority_layer: stage.authority_layer,
                current_state: stage.current_state,
                input_refs: stage.input_refs,
                output_refs: stage.output_refs,
                evidence_refs: stage.evidence_refs,
                summary: stage.summary,
                updated_at: stage.updated_at,
            })
            .collect(),
        authority_layers: projection
            .authority_layers
            .into_iter()
            .map(|entry| SpecLoopAuthorityLayerView {
                authority_layer: entry.authority_layer,
                path: entry.path,
                summary: entry.summary,
            })
            .collect(),
        traceability: projection
            .traceability
            .into_iter()
            .map(|edge| SpecLoopTraceabilityView {
                from_ref: edge.from_ref,
                to_ref: edge.to_ref,
                relation: edge.relation,
            })
            .collect(),
        runtime_action_proposals: projection
            .runtime_action_proposals
            .into_iter()
            .map(|proposal| SpecLoopActionProposalView {
                proposal_ref: proposal.proposal_ref,
                action_type: proposal.action_type,
                target_object_type: proposal.target_object_type,
                target_object_id: proposal.target_object_id,
                created_object_type: proposal.created_object_type,
                created_object_id: proposal.created_object_id,
                actor_role: proposal.actor_role,
                handoff_rule: proposal.handoff_rule,
                command_status: proposal.command_status,
                decision_status: proposal.decision_status,
                accepted_action_id: proposal.accepted_action_id,
                command_path: proposal.command_path,
                proposal_path: proposal.proposal_path,
                decision_path: proposal.decision_path,
                accepted_action_path: proposal.accepted_action_path,
            })
            .collect(),
        allowed_actions: next_action_hints(
            &projection.next_recommended_action,
            &projection.next_recommended_action_label,
            &projection.next_recommended_action_reason,
        ),
        freshness,
    })
}

pub fn get_project_home_view(
    project_root: impl AsRef<Path>,
    project_id: &str,
) -> Result<ProjectHomeView> {
    let spec_project = read_spec_project(&project_root, project_id)?;
    let projection = load_project_projection(&project_root, project_id)?;
    let freshness = explain_projection_staleness(
        &project_root,
        ProjectionScope::Project {
            project_id: project_id.to_string(),
        },
        &projection.version,
        projection.updated_at,
        None,
    )?;

    let mut dependency_graph = Vec::new();
    let mut active_runs = Vec::new();
    for issue_id in &spec_project.issue_ids {
        let issue = read_spec_issue(&project_root, issue_id)?;
        for dependency in &issue.blocked_by {
            dependency_graph.push(ProjectDependencyEdge {
                issue_id: issue.issue_id.clone(),
                depends_on_issue_id: dependency.clone(),
            });
        }
        let task = load_task_projection(&project_root, issue_id).ok();
        if let Some(task) = task {
            if let Some(run_id) = task.latest_run_id.clone() {
                active_runs.push(ProjectRunSummary {
                    issue_id: issue.issue_id.clone(),
                    run_id,
                    run_status: task.runtime.run_status.clone(),
                    branch_name: task.branch_name.clone(),
                });
            }
        }
    }

    Ok(ProjectHomeView {
        project_id: projection.project_id.clone(),
        title: projection.title.clone(),
        objective: projection.objective.clone(),
        state_summary: format!("{} / {}", projection.status, projection.stage_label),
        issue_groups: projection.lanes.clone(),
        dependency_graph,
        active_runs,
        blocked_items: projection
            .blockers
            .iter()
            .map(|blocker| format!("{}: {}", blocker.issue_id, blocker.reason))
            .collect(),
        recent_events: recent_events(
            &project_root,
            ReplayFilter {
                project_id: Some(project_id.to_string()),
                ..ReplayFilter::default()
            },
            8,
        )?,
        allowed_actions: next_action_hints(
            &projection.next_action,
            &projection.next_action_label,
            &projection.next_action_reason,
        ),
        freshness,
    })
}

pub fn get_task_workbench_view(
    project_root: impl AsRef<Path>,
    issue_id: &str,
) -> Result<TaskWorkbenchView> {
    let issue = read_spec_issue(&project_root, issue_id)?;
    let projection = load_task_projection(&project_root, issue_id)?;
    let projection_cursor = projection
        .timeline
        .iter()
        .flat_map(|item| item.events.iter().map(|event| event.event_id.clone()))
        .last();
    let freshness = explain_projection_staleness(
        &project_root,
        ProjectionScope::Issue {
            issue_id: issue_id.to_string(),
        },
        &projection.version,
        projection.updated_at,
        projection_cursor,
    )?;

    let runtime_events = replay_runtime_events(&project_root, ReplayFilter::issue(issue_id))?;
    let event_stream = collect_work_loop_events(
        &project_root,
        ReplayFilter::issue(issue_id),
        EventStreamScope::Issue,
    )?;
    let mut evidence_refs = BTreeSet::new();
    let mut artifact_refs = BTreeSet::new();
    for event in runtime_events {
        for evidence in event.envelope.evidence_refs {
            evidence_refs.insert(evidence);
        }
        for artifact in event.envelope.artifact_refs {
            artifact_refs.insert(artifact);
        }
    }
    if let Ok(evidence) = load_task_evidence(&project_root, issue_id) {
        evidence_refs.insert(evidence.run_path);
        evidence_refs.insert(evidence.validation_path);
        if let Some(changed_files_path) = evidence.changed_files_path {
            evidence_refs.insert(changed_files_path);
        }
        for path in evidence.command_paths {
            evidence_refs.insert(path);
        }
    }
    let evidence_summary = build_work_loop_evidence_summary(
        &project_root,
        issue_id,
        projection.latest_run_id.as_deref(),
        projection.session.session_id.as_deref(),
        &projection.public_delivery,
    );
    let state_explanation =
        explain_issue_state(&projection.current_state, &event_stream, &projection);

    Ok(TaskWorkbenchView {
        issue_id: issue.issue_id.clone(),
        title: issue.title.clone(),
        summary: issue.summary.clone(),
        issue_state: projection.current_state.clone(),
        run_state: projection.runtime.run_status.clone(),
        active_run: projection.latest_run_id.clone(),
        evidence_refs: evidence_refs.into_iter().collect(),
        artifact_refs: artifact_refs.into_iter().collect(),
        acceptance_mapping: issue_acceptance_mapping(&issue, &projection.delivery),
        allowed_actions: task_allowed_actions(&projection),
        blocked_reasons: task_blocked_reasons(&issue, &projection),
        state_explanation,
        evidence_summary,
        event_stream,
        timeline: projection.timeline.clone(),
        freshness,
    })
}

pub fn get_work_loop_run_view(
    project_root: impl AsRef<Path>,
    issue_id: &str,
    run_id: &str,
) -> Result<WorkLoopRunView> {
    let projection = load_task_projection(&project_root, issue_id)?;
    let task_run = load_task_run(&project_root, issue_id, run_id)?;
    let event_stream = collect_work_loop_events(
        &project_root,
        ReplayFilter::run(issue_id.to_string(), run_id.to_string()),
        EventStreamScope::Run {
            run_id: run_id.to_string(),
        },
    )?;
    let evidence_summary = build_work_loop_evidence_summary(
        &project_root,
        issue_id,
        Some(run_id),
        task_run.session_id.as_deref(),
        &projection.public_delivery,
    );
    let run_state = task_run_status_label(&task_run.status).to_string();
    let state_explanation = explain_run_state(&run_state, &event_stream, &task_run);
    let freshness = explain_projection_staleness(
        &project_root,
        ProjectionScope::Issue {
            issue_id: issue_id.to_string(),
        },
        &projection.version,
        projection.updated_at,
        projection
            .timeline
            .iter()
            .flat_map(|item| item.events.iter().map(|event| event.event_id.clone()))
            .last(),
    )?;

    Ok(WorkLoopRunView {
        issue_id: issue_id.to_string(),
        run_id: run_id.to_string(),
        issue_state: projection.current_state,
        run_state,
        branch_name: task_run.branch_name,
        session_id: task_run.session_id,
        session_status: task_run.session_status,
        state_explanation,
        evidence_summary,
        event_stream,
        freshness,
    })
}

pub fn get_work_loop_session_view(
    project_root: impl AsRef<Path>,
    session_id: &str,
) -> Result<WorkLoopSessionView> {
    let (issue_id, run_id, projection) =
        find_session_projection_context(&project_root, session_id)?;
    let session_record =
        load_task_session_history_record(&project_root, &issue_id, &run_id, session_id)?;
    let recovery_summary =
        load_task_session_recovery_summary(&project_root, &issue_id, &run_id).ok();
    let event_stream = collect_work_loop_events(
        &project_root,
        ReplayFilter::run(issue_id.clone(), run_id.clone()),
        EventStreamScope::Session {
            run_id: run_id.clone(),
            session_id: session_id.to_string(),
        },
    )?;
    let evidence_summary = build_work_loop_evidence_summary(
        &project_root,
        &issue_id,
        Some(&run_id),
        Some(session_id),
        &projection.public_delivery,
    );
    let state_explanation =
        explain_session_state(&session_record, recovery_summary.as_ref(), &event_stream);
    let freshness = explain_projection_staleness(
        &project_root,
        ProjectionScope::Issue {
            issue_id: issue_id.clone(),
        },
        &projection.version,
        projection.updated_at,
        projection
            .timeline
            .iter()
            .flat_map(|item| item.events.iter().map(|event| event.event_id.clone()))
            .last(),
    )?;

    Ok(WorkLoopSessionView {
        issue_id,
        run_id,
        session_id: session_id.to_string(),
        provider: Some(session_record.provider),
        session_owner: Some(session_record.session_owner),
        session_status: Some(session_record.status.as_str().to_string()),
        attempt_count: session_record.attempt_count,
        started_at: Some(session_record.started_at),
        last_heartbeat_at: Some(session_record.last_heartbeat_at),
        recovery_reason: recovery_summary
            .as_ref()
            .and_then(|summary| summary.recovery_reason.clone())
            .or(session_record.recovery_reason),
        resumed_from_attempt: recovery_summary
            .as_ref()
            .and_then(|summary| summary.resumed_from_attempt)
            .or(session_record.resumed_from_attempt),
        retry_policy: session_record.retry_policy,
        retryable: Some(session_record.retryable),
        terminal_reason: session_record.terminal_reason,
        last_error: session_record.last_error,
        state_explanation,
        evidence_summary,
        event_stream,
        freshness,
    })
}

pub fn get_audit_surface_view(
    project_root: impl AsRef<Path>,
    audit_id: &str,
) -> Result<AuditSurfaceView> {
    let report = load_audit_report(&project_root, audit_id.to_string())?;
    let summary = load_audit_result_summary(&project_root, audit_id.to_string())?;
    let allowed_actions = audit_allowed_actions(&summary);
    let freshness = explain_projection_staleness(
        &project_root,
        ProjectionScope::Audit {
            source_issue_id: report.audit.source_issue_id.clone(),
        },
        &summary.version,
        summary.requested_at,
        None,
    )?;

    Ok(AuditSurfaceView {
        audit_id: summary.audit_id,
        audit_state: summary.status,
        scope: format!(
            "issue={} / run={}",
            report
                .audit
                .source_issue_id
                .unwrap_or_else(|| "none".to_string()),
            report
                .audit
                .source_run_id
                .unwrap_or_else(|| "none".to_string())
        ),
        evidence_map: report.evidence_map.inputs.values().cloned().collect(),
        findings: report
            .findings
            .findings
            .iter()
            .map(|finding| format!("{}: {}", finding.severity.as_str(), finding.title))
            .collect(),
        traceability: report
            .traceability
            .chain
            .iter()
            .map(|item| format!("{}:{} -> {}", item.layer, item.id, item.path))
            .collect(),
        allowed_actions,
        freshness,
    })
}

pub fn get_delivery_package_view(
    project_root: impl AsRef<Path>,
    issue_id: &str,
) -> Result<DeliveryPackageView> {
    let issue = read_spec_issue(&project_root, issue_id)?;
    let projection = load_task_projection(&project_root, issue_id)?;
    let projection_cursor = projection
        .timeline
        .iter()
        .flat_map(|item| item.events.iter().map(|event| event.event_id.clone()))
        .last();
    let freshness = explain_projection_staleness(
        &project_root,
        ProjectionScope::Issue {
            issue_id: issue_id.to_string(),
        },
        &projection.version,
        projection.updated_at,
        projection_cursor,
    )?;
    let runtime_events = replay_runtime_events(&project_root, ReplayFilter::issue(issue_id))?;
    let mut artifact_refs = BTreeSet::new();
    for event in runtime_events {
        for artifact in event.envelope.artifact_refs {
            artifact_refs.insert(artifact);
        }
    }
    if let Some(pr_url) = projection.public_delivery.pr_url.clone() {
        artifact_refs.insert(pr_url);
    }
    if let Some(changelog_path) = projection.public_delivery.changelog_path.clone() {
        artifact_refs.insert(changelog_path);
    }
    if let Some(release_notes_url) = projection.public_delivery.release_notes_url.clone() {
        artifact_refs.insert(release_notes_url);
    }

    let mut verification_logs = Vec::new();
    if let Ok(evidence) = load_task_evidence(&project_root, issue_id) {
        verification_logs.extend(evidence.command_paths.clone());
        verification_logs.push(evidence.validation_path.clone());
        verification_logs.push(evidence.run_path.clone());
    }

    Ok(DeliveryPackageView {
        issue_id: issue.issue_id.clone(),
        delivery_state: projection.delivery.status.clone(),
        artifact_refs: artifact_refs.into_iter().collect(),
        verification_logs,
        acceptance_mapping: issue_acceptance_mapping(&issue, &projection.delivery),
        build_agent_summary: delivery_summary_line(
            &projection.delivery,
            &projection.public_delivery,
        ),
        allowed_actions: delivery_allowed_actions(&projection),
        freshness,
    })
}

pub fn get_runtime_health_view(
    project_root: impl AsRef<Path>,
    project_id: &str,
) -> Result<RuntimeHealthView> {
    let projection = load_project_projection(&project_root, project_id)?;
    let freshness = explain_projection_staleness(
        &project_root,
        ProjectionScope::Project {
            project_id: project_id.to_string(),
        },
        &projection.version,
        projection.updated_at,
        None,
    )?;

    let mut warnings = Vec::new();
    if !projection.blockers.is_empty() {
        warnings.push("project-blockers-present".to_string());
    }
    if projection
        .audit
        .as_ref()
        .is_some_and(|audit| audit.status == "failed")
    {
        warnings.push("audit-failed".to_string());
    }

    Ok(RuntimeHealthView {
        project_id: projection.project_id.clone(),
        project_status: projection.status.clone(),
        current_issue_id: projection.current_issue_id.clone(),
        active_issue_count: projection.lanes.current.len(),
        blocked_issue_count: projection.lanes.blocked.len(),
        warnings,
        allowed_actions: next_action_hints(
            &projection.next_action,
            &projection.next_action_label,
            &projection.next_action_reason,
        ),
        freshness,
    })
}

pub fn get_pack_industry_workbench_view(
    project_root: impl AsRef<Path>,
    pack_id: Option<&str>,
) -> Result<PackIndustryWorkbenchView> {
    let project_root = project_root.as_ref();
    let bundles = build_pack_bundles(project_root)?;
    let active_pack_id = match pack_id {
        Some(pack_id) => Some(pack_id.to_string()),
        None => bundles.first().map(|bundle| bundle.pack_id.clone()),
    };

    let mut warnings = Vec::new();
    if bundles.is_empty() {
        warnings.push("pack-not-found".to_string());
    }

    let mut pack_list = Vec::new();
    let mut pack_readiness = Vec::new();
    let mut definition_status_index = Vec::new();
    let mut domain_object_index = Vec::new();
    let mut surface_page_index = Vec::new();
    let mut view_model_mapping_index = Vec::new();
    let mut connector_capability_index = Vec::new();
    let mut industry_workbenches = Vec::new();
    let mut source_refs = BTreeSet::new();

    for bundle in &bundles {
        for source_ref in bundle.source_refs() {
            source_refs.insert(source_ref);
        }
        pack_list.push(PackListItemView {
            pack_id: bundle.pack_id.clone(),
            name: bundle.name.clone(),
            pack_type: bundle.pack_type.clone(),
            pack_version: bundle.pack_version.clone(),
            registered: bundle.registered,
            validation_status: bundle.readiness_status(),
            manifest_path: bundle.manifest_path.clone(),
            source_refs: bundle.source_refs(),
        });
        pack_readiness.push(PackReadinessView {
            pack_id: bundle.pack_id.clone(),
            status: bundle.readiness_status(),
            manifest_valid: bundle.manifest_valid,
            domain_valid: bundle.domain_valid,
            surface_valid: bundle.surface_valid,
            connector_valid: bundle.connector_valid,
            warnings: bundle.warnings(),
        });
        definition_status_index.extend(bundle.definition_status_index());
        if active_pack_id.as_ref() != Some(&bundle.pack_id) {
            continue;
        }
        if let Some(domain) = bundle.domain.as_ref() {
            domain_object_index.extend(domain.object_types.iter().map(|object| {
                PackDomainObjectIndexItem {
                    pack_id: bundle.pack_id.clone(),
                    object_type_id: object.object_type_id.clone(),
                    label: object.label.clone(),
                    description: object.description.clone(),
                }
            }));
        }
        if let Some(surface) = bundle.surface.as_ref() {
            let mapped_page_ids = surface
                .view_model_mappings
                .iter()
                .map(|mapping| mapping.page_id.as_str())
                .collect::<BTreeSet<_>>();
            surface_page_index.extend(surface.pages.iter().map(|page| PackSurfacePageIndexItem {
                pack_id: bundle.pack_id.clone(),
                page_id: page.page_id.clone(),
                label: page.label.clone(),
                page_kind: enum_label(&page.kind),
                view_model_ref: page.view_model_ref.clone(),
                command_entry_ids: page.command_entry_ids.clone(),
            }));
            view_model_mapping_index.extend(surface.view_model_mappings.iter().map(|mapping| {
                PackViewModelMappingIndexItem {
                    pack_id: bundle.pack_id.clone(),
                    mapping_id: mapping.mapping_id.clone(),
                    page_id: mapping.page_id.clone(),
                    projection_ref: mapping.projection_ref.clone(),
                    view_model_ref: mapping.view_model_ref.clone(),
                    status: if bundle.surface_valid {
                        "ready".to_string()
                    } else {
                        "invalid".to_string()
                    },
                    reason: if bundle.surface_valid {
                        "pack-surface-mapping-ready".to_string()
                    } else {
                        "pack-surface-invalid".to_string()
                    },
                }
            }));
            view_model_mapping_index.extend(surface.pages.iter().filter_map(|page| {
                if mapped_page_ids.contains(page.page_id.as_str()) {
                    None
                } else {
                    Some(PackViewModelMappingIndexItem {
                        pack_id: bundle.pack_id.clone(),
                        mapping_id: format!("missing:{}", page.page_id),
                        page_id: page.page_id.clone(),
                        projection_ref: String::new(),
                        view_model_ref: page.view_model_ref.clone(),
                        status: "deferred".to_string(),
                        reason: "pack-surface-view-model-mapping-missing".to_string(),
                    })
                }
            }));
            industry_workbenches.extend(surface.workbenches.iter().map(|workbench| {
                PackIndustryWorkbenchItem {
                    pack_id: bundle.pack_id.clone(),
                    workbench_id: workbench.workbench_id.clone(),
                    page_id: workbench.page_id.clone(),
                    label: workbench.label.clone(),
                    primary_object_type: workbench.primary_object_type.clone(),
                    timeline_ref: workbench.timeline_ref.clone(),
                }
            }));
        }
        if let Some(connector_definition) = bundle.connector.as_ref() {
            connector_capability_index.extend(connector_definition.connectors.iter().flat_map(
                |connector| {
                    connector.supported_actions.iter().map(|action| {
                        PackConnectorCapabilityIndexItem {
                            pack_id: bundle.pack_id.clone(),
                            connector_id: connector.connector_id.clone(),
                            provider_type: enum_label(&connector.provider_type),
                            action_id: action.action_id.clone(),
                            command_type: action.command_type.clone(),
                            required_capability: action.required_capability.clone(),
                            writes_external: action.writes_external,
                            evidence_output: action.evidence_output.clone(),
                            status: connector_projection_status(connector, bundle.connector_valid),
                            disabled_reason: connector_disabled_reason(connector),
                            command_execution_allowed: connector_command_execution_allowed(
                                connector,
                                bundle.connector_valid,
                            ),
                        }
                    })
                },
            ));
        }
    }

    let mut freshness = missing_freshness("pack-projection-readonly-derived");
    freshness.warnings.extend(warnings.clone());

    Ok(PackIndustryWorkbenchView {
        version: "pack-industry-workbench-view.v1".to_string(),
        active_pack_id,
        pack_list,
        pack_readiness,
        definition_status_index,
        domain_object_index,
        surface_page_index,
        view_model_mapping_index,
        connector_capability_index,
        industry_workbenches,
        source_refs: source_refs.into_iter().collect(),
        authority: false,
        freshness,
        warnings,
    })
}

#[derive(Debug, Clone)]
struct PackWorkbenchBundle {
    pack_id: String,
    name: String,
    pack_type: String,
    pack_version: String,
    validation_status: PackValidationStatus,
    manifest_path: String,
    registered: bool,
    manifest_valid: bool,
    domain_valid: bool,
    surface_valid: bool,
    connector_valid: bool,
    domain: Option<PackDomainDefinition>,
    surface: Option<PackSurfaceDefinition>,
    connector: Option<PackConnectorDefinition>,
    definition_warnings: Vec<String>,
}

impl PackWorkbenchBundle {
    fn readiness_status(&self) -> String {
        if self.manifest_valid
            && self.validation_status == PackValidationStatus::Valid
            && self.domain_valid
            && self.surface_valid
            && self.connector_valid
        {
            "ready".to_string()
        } else {
            "invalid".to_string()
        }
    }

    fn command_execution_allowed(&self) -> bool {
        self.readiness_status() == "ready"
    }

    fn definition_status_index(&self) -> Vec<PackDefinitionStatusIndexItem> {
        vec![
            self.definition_status_item(
                "app",
                self.manifest_valid && self.validation_status == PackValidationStatus::Valid,
                if self.manifest_valid {
                    "pack-app-definition-ready"
                } else {
                    "pack-manifest-invalid"
                },
            ),
            self.definition_status_item(
                "domain",
                self.domain_valid,
                if self.domain_valid {
                    "pack-domain-ready"
                } else {
                    "pack-domain-invalid-or-missing"
                },
            ),
            self.definition_status_item(
                "surface",
                self.surface_valid,
                if self.surface_valid {
                    "pack-surface-ready"
                } else {
                    "pack-surface-invalid-or-missing"
                },
            ),
            self.definition_status_item(
                "connector",
                self.connector_valid,
                if self.connector_valid {
                    "pack-connector-ready"
                } else {
                    "pack-connector-invalid-or-missing"
                },
            ),
        ]
    }

    fn definition_status_item(
        &self,
        definition_kind: &str,
        valid: bool,
        ready_or_invalid_reason: &str,
    ) -> PackDefinitionStatusIndexItem {
        let (status, reason) = if definition_kind == "app"
            && self.manifest_valid
            && self.validation_status != PackValidationStatus::Valid
        {
            (
                "stale".to_string(),
                format!(
                    "pack-app-definition-validation-status-{}",
                    enum_label(&self.validation_status)
                ),
            )
        } else if valid {
            ("ready".to_string(), ready_or_invalid_reason.to_string())
        } else {
            ("invalid".to_string(), ready_or_invalid_reason.to_string())
        };
        PackDefinitionStatusIndexItem {
            pack_id: self.pack_id.clone(),
            definition_kind: definition_kind.to_string(),
            status,
            reason,
            command_execution_allowed: valid && self.command_execution_allowed(),
        }
    }

    fn source_refs(&self) -> Vec<String> {
        let mut refs = vec![format!("pack-builtin:{}", self.pack_id)];
        if self.registered {
            refs.push(self.manifest_path.clone());
        }
        if let Some(domain) = self.domain.as_ref() {
            refs.push(format!("pack-domain:{}", domain.domain_id));
        }
        if let Some(surface) = self.surface.as_ref() {
            refs.push(format!("pack-surface:{}", surface.surface_id));
        }
        if let Some(connector) = self.connector.as_ref() {
            refs.push(format!("pack-connector:{}", connector.connector_id));
        }
        refs
    }

    fn warnings(&self) -> Vec<String> {
        let mut warnings = Vec::new();
        if !self.registered {
            warnings.push("pack-manifest-not-registered".to_string());
        }
        if !self.manifest_valid {
            warnings.push("pack-manifest-invalid".to_string());
        }
        if !self.domain_valid {
            warnings.push("pack-domain-invalid".to_string());
        }
        if !self.surface_valid {
            warnings.push("pack-surface-invalid".to_string());
        }
        if !self.connector_valid {
            warnings.push("pack-connector-invalid".to_string());
        }
        warnings.extend(self.definition_warnings.clone());
        warnings
    }
}

fn connector_projection_status(connector: &PackConnector, connector_valid: bool) -> String {
    if !connector_valid {
        "invalid".to_string()
    } else if !connector_command_execution_allowed(connector, connector_valid) {
        "deferred".to_string()
    } else {
        "ready".to_string()
    }
}

fn connector_disabled_reason(connector: &PackConnector) -> String {
    let disabled_reason = connector.disabled_reason.trim();
    if disabled_reason.is_empty() || disabled_reason == "capability-registry.disabled-reason" {
        String::new()
    } else {
        disabled_reason.to_string()
    }
}

fn connector_command_execution_allowed(connector: &PackConnector, connector_valid: bool) -> bool {
    connector_valid && connector_disabled_reason(connector).is_empty()
}

fn build_pack_bundles(project_root: &Path) -> Result<Vec<PackWorkbenchBundle>> {
    let registry = agentflow_pack::load_pack_registry(project_root)?;
    let mut bundles = Vec::new();
    if registry.entries.is_empty() {
        bundles.push(pack_bundle_from_definitions(
            "software-dev",
            "Software Dev",
            "software-dev",
            "0.8.0",
            software_dev_domain_definition(),
            software_dev_surface_definition(),
            software_dev_connector_definition(),
        ));
        bundles.push(pack_bundle_from_definitions(
            "ui-design",
            "UI Design",
            "ui-design",
            "0.8.0",
            ui_design_domain_definition(),
            ui_design_surface_definition(),
            ui_design_connector_definition(),
        ));
    } else {
        for entry in registry.entries {
            bundles.push(pack_bundle_from_registry_entry(entry));
        }
    }

    bundles.sort_by(|left, right| left.pack_id.cmp(&right.pack_id));
    Ok(bundles)
}

fn pack_bundle_from_definitions(
    pack_id: &str,
    name: &str,
    pack_type: &str,
    pack_version: &str,
    domain: PackDomainDefinition,
    surface: PackSurfaceDefinition,
    connector: PackConnectorDefinition,
) -> PackWorkbenchBundle {
    let domain_report = validate_domain_definition(&domain);
    let surface_report = validate_surface_definition(&surface);
    let connector_report = validate_connector_definition(&connector);
    PackWorkbenchBundle {
        pack_id: pack_id.to_string(),
        name: name.to_string(),
        pack_type: pack_type.to_string(),
        pack_version: pack_version.to_string(),
        validation_status: PackValidationStatus::Valid,
        manifest_path: format!(".agentflow/packs/{pack_id}/pack.json"),
        registered: false,
        manifest_valid: true,
        domain_valid: domain_report.valid,
        surface_valid: surface_report.valid,
        connector_valid: connector_report.valid,
        domain: Some(domain),
        surface: Some(surface),
        connector: Some(connector),
        definition_warnings: Vec::new(),
    }
}

fn pack_bundle_from_registry_entry(entry: PackRegistryEntry) -> PackWorkbenchBundle {
    let mut definition_warnings = Vec::new();
    let domain = match load_pack_domain_definition(&entry) {
        Ok(domain) => Some(domain),
        Err(error) => {
            definition_warnings.push(format!("pack-domain-unreadable: {error}"));
            None
        }
    };
    let surface = match load_pack_surface_definition(&entry) {
        Ok(surface) => Some(surface),
        Err(error) => {
            definition_warnings.push(format!("pack-surface-unreadable: {error}"));
            None
        }
    };
    let connector = match load_pack_connector_definition(&entry) {
        Ok(connector) => Some(connector),
        Err(error) => {
            definition_warnings.push(format!("pack-connector-unreadable: {error}"));
            None
        }
    };
    let domain_valid = domain
        .as_ref()
        .map(validate_domain_definition)
        .is_some_and(|report| report.valid);
    let surface_valid = surface
        .as_ref()
        .map(validate_surface_definition)
        .is_some_and(|report| report.valid);
    let connector_valid = connector
        .as_ref()
        .map(validate_connector_definition)
        .is_some_and(|report| report.valid);

    PackWorkbenchBundle {
        pack_id: entry.pack_id,
        name: entry.name,
        pack_type: entry.pack_type.as_str().to_string(),
        pack_version: entry.pack_version,
        validation_status: entry.validation_status,
        manifest_path: entry.manifest_path,
        registered: true,
        manifest_valid: entry.validation.valid,
        domain_valid,
        surface_valid,
        connector_valid,
        domain,
        surface,
        connector,
        definition_warnings,
    }
}

fn load_pack_domain_definition(entry: &PackRegistryEntry) -> Result<PackDomainDefinition> {
    load_pack_definition(definition_path_for_entry(entry, &entry.domain_path))
}

fn load_pack_surface_definition(entry: &PackRegistryEntry) -> Result<PackSurfaceDefinition> {
    load_pack_definition(definition_path_for_entry(entry, &entry.surface_path))
}

fn load_pack_connector_definition(entry: &PackRegistryEntry) -> Result<PackConnectorDefinition> {
    load_pack_definition(definition_path_for_entry(entry, &entry.connector_path))
}

fn load_pack_definition<T: DeserializeOwned>(path: PathBuf) -> Result<T> {
    let payload = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str::<T>(&payload).with_context(|| format!("parse {}", path.display()))
}

fn definition_path_for_entry(entry: &PackRegistryEntry, relative_path: &str) -> PathBuf {
    let path = PathBuf::from(&entry.pack_root).join(relative_path);
    if path.extension().is_some() {
        path
    } else {
        path.join("definition.json")
    }
}

fn enum_label<T: Serialize>(value: &T) -> String {
    serde_json::to_value(value)
        .ok()
        .and_then(|value| value.as_str().map(ToString::to_string))
        .unwrap_or_else(|| "unknown".to_string())
}

#[derive(Debug, Clone)]
enum EventStreamScope {
    Issue,
    Run { run_id: String },
    Session { run_id: String, session_id: String },
}

fn collect_work_loop_events(
    project_root: impl AsRef<Path>,
    filter: ReplayFilter,
    scope: EventStreamScope,
) -> Result<Vec<WorkLoopEventView>> {
    let events = replay_task_events(project_root, filter)?;
    Ok(events
        .into_iter()
        .filter(|event| match &scope {
            EventStreamScope::Issue => true,
            EventStreamScope::Run { run_id } => event.run_id.as_deref() == Some(run_id.as_str()),
            EventStreamScope::Session { run_id, session_id } => {
                if event.run_id.as_deref() != Some(run_id.as_str()) {
                    return false;
                }
                match payload_string(&event.payload, "sessionId") {
                    Some(value) => value == *session_id,
                    None => true,
                }
            }
        })
        .map(work_loop_event_view)
        .collect())
}

fn work_loop_event_view(event: TaskEvent) -> WorkLoopEventView {
    let compatibility = map_task_event_to_runtime_event(&event).ok();
    let mut evidence_refs = BTreeSet::new();
    let mut artifact_refs = BTreeSet::new();
    if let Some(runtime) = compatibility.as_ref() {
        evidence_refs.extend(runtime.envelope.evidence_refs.iter().cloned());
        artifact_refs.extend(runtime.envelope.artifact_refs.iter().cloned());
    }
    artifact_refs.extend(event.artifact_refs.iter().cloned());

    let (stage_key, stage_label) = work_loop_stage(event.event_type.as_str());
    WorkLoopEventView {
        event_id: event.event_id.clone(),
        event_type: event.event_type.clone(),
        category: classify_task_event(event.event_type.as_str())
            .as_str()
            .to_string(),
        stage_key: stage_key.to_string(),
        stage_label: stage_label.to_string(),
        timestamp: event.timestamp,
        actor_role: event.actor.role.clone(),
        actor_kind: event.actor.kind.clone(),
        run_id: event.run_id.clone(),
        session_id: payload_string(&event.payload, "sessionId"),
        from_state: event.state.as_ref().map(|state| state.from_state.clone()),
        to_state: event.state.as_ref().map(|state| state.to_state.clone()),
        summary: work_loop_event_summary(&event),
        evidence_refs: evidence_refs.into_iter().collect(),
        artifact_refs: artifact_refs.into_iter().collect(),
    }
}

fn work_loop_stage(event_type: &str) -> (&'static str, &'static str) {
    match event_type {
        "issue.scheduled" => ("todo", "准备开工"),
        "issue.preflight.passed"
        | "issue.preflight.failed"
        | "panel.context-pack.ready"
        | "panel.context-pack.failed" => ("preflight", "前置检测"),
        "agent.launch.requested" | "agent.launch.claimed" => ("launch", "启动会话"),
        value if value.starts_with("agent.session.") => ("session", "执行会话"),
        "issue.validation.passed" | "issue.validation.failed" => ("verification", "沙箱验证"),
        "issue.review.requested"
        | "issue.pr.created"
        | "issue.closeout.proof.recorded"
        | "issue.pr.merged" => ("review", "评审收口"),
        value if value.starts_with("issue.acceptance.") => ("acceptance", "验收判定"),
        value if value.starts_with("issue.completion.") => ("completion", "完成提交"),
        value if value.starts_with("issue.audit.") => ("audit-evaluation", "审计判断"),
        "issue.completed" => ("done", "Done 写回"),
        _ => ("event", "事件记录"),
    }
}

fn work_loop_event_summary(event: &TaskEvent) -> String {
    match event.event_type.as_str() {
        "issue.scheduled" => "任务进入待执行队列。".to_string(),
        "agent.launch.requested" => "已生成 Work Agent 启动请求。".to_string(),
        "agent.launch.claimed" => "执行会话已认领启动请求。".to_string(),
        "agent.session.created" => "外部执行会话已创建。".to_string(),
        "agent.session.resumed" => "外部执行会话已恢复。".to_string(),
        "agent.session.running" => "外部执行会话正在运行。".to_string(),
        "agent.session.interrupted" => "外部执行会话已中断，等待恢复。".to_string(),
        "agent.session.in_review" => "外部执行会话已进入评审。".to_string(),
        "agent.session.completed" => "外部执行会话已完成。".to_string(),
        "agent.session.failed" => "外部执行会话失败。".to_string(),
        "agent.session.cancelled" => "外部执行会话已取消。".to_string(),
        "issue.validation.passed" => "本地沙箱验证已通过。".to_string(),
        "issue.validation.failed" => "本地沙箱验证失败。".to_string(),
        "issue.review.requested" => "任务已请求评审。".to_string(),
        "issue.pr.created" => "PR/MR 已创建。".to_string(),
        "issue.closeout.proof.recorded" => "收口证明已写入，等待 Done 写回。".to_string(),
        "issue.pr.merged" => "PR/MR 已合并，等待关单与收口证明。".to_string(),
        "issue.acceptance.accepted" => "验收判定已通过。".to_string(),
        "issue.acceptance.rejected" => "验收判定被拒绝，需先修复失败原因。".to_string(),
        "issue.acceptance.human-review-required" => {
            "验收判定需要人工判断，不能伪装成自动通过。".to_string()
        }
        "issue.completion.committed" => {
            "Completion Commit 已写入，Done 写回只能由该事件触发。".to_string()
        }
        "issue.audit.evaluated" => {
            "Done 后的可选审计触发已评估；该判断不改变任务完成事实。".to_string()
        }
        "issue.completed" => "任务 Done 写回完成。".to_string(),
        "issue.blocked" => "任务进入阻断状态。".to_string(),
        "issue.cancelled" => "任务已取消。".to_string(),
        other => format!("记录事件：{other}。"),
    }
}

fn payload_string(payload: &Value, key: &str) -> Option<String> {
    payload
        .get(key)
        .and_then(Value::as_str)
        .map(ToString::to_string)
}

fn build_work_loop_evidence_summary(
    project_root: impl AsRef<Path>,
    issue_id: &str,
    run_id: Option<&str>,
    session_id: Option<&str>,
    public_delivery: &ProjectionPublicDelivery,
) -> WorkLoopEvidenceSummaryView {
    let mut status = "missing".to_string();
    let mut summary_parts = Vec::new();
    let mut verification_refs = BTreeSet::new();
    let mut session_refs = BTreeSet::new();
    let mut delivery_refs = BTreeSet::new();

    if let Ok(evidence) = load_task_evidence(&project_root, issue_id) {
        status = evidence.status.clone();
        if !evidence.summary.trim().is_empty() {
            summary_parts.push(evidence.summary);
        }
        verification_refs.insert(evidence.run_path);
        verification_refs.insert(evidence.validation_path);
        verification_refs.extend(evidence.command_paths);
        if let Some(changed_files_path) = evidence.changed_files_path {
            verification_refs.insert(changed_files_path);
        }
    }

    if let (Some(run_id), Some(_session_id)) = (run_id, session_id) {
        if let Ok(session_evidence) = load_task_session_evidence(&project_root, issue_id, run_id) {
            status = session_evidence.status.as_str().to_string();
            if !session_evidence.summary.trim().is_empty() {
                summary_parts.push(session_evidence.summary);
            }
            session_refs.extend(session_evidence.refs);
        }
    }

    if let Some(pr_url) = public_delivery.pr_url.clone() {
        delivery_refs.insert(pr_url);
    }
    if let Some(evidence_path) = public_delivery.evidence_path.clone() {
        delivery_refs.insert(evidence_path);
    }
    if let Some(changelog_path) = public_delivery.changelog_path.clone() {
        delivery_refs.insert(changelog_path);
    }
    if let Some(release_notes_url) = public_delivery.release_notes_url.clone() {
        delivery_refs.insert(release_notes_url);
    }

    if summary_parts.is_empty() {
        summary_parts.push("当前还没有可展示的验证或交付证据。".to_string());
    }

    WorkLoopEvidenceSummaryView {
        status,
        summary: summary_parts.join(" "),
        verification_refs: verification_refs.into_iter().collect(),
        session_refs: session_refs.into_iter().collect(),
        delivery_refs: delivery_refs.into_iter().collect(),
    }
}

fn explain_issue_state(
    current_state: &str,
    event_stream: &[WorkLoopEventView],
    projection: &TaskProjection,
) -> String {
    if let Some(last_event) = event_stream.last() {
        return match current_state {
            "todo" => format!("任务已满足开工前置条件，当前停在 {}。", last_event.summary),
            "in_progress" => format!("任务正在执行，当前事实是：{}。", last_event.summary),
            "in_review" => format!("任务已进入评审，当前事实是：{}。", last_event.summary),
            "done" => format!("任务已完成，最终写回事实是：{}。", last_event.summary),
            "blocked" => format!("任务已阻断，最近事实是：{}。", last_event.summary),
            "cancel" => format!("任务已取消，最近事实是：{}。", last_event.summary),
            _ => format!("当前状态由最新事件驱动：{}。", last_event.summary),
        };
    }

    match current_state {
        "backlog" => "任务还未进入执行，等待进入调度。".to_string(),
        "todo" => "任务已准备开工，等待拉起执行会话。".to_string(),
        "in_progress" => "任务正在执行，但还没有生成可展示事件。".to_string(),
        "in_review" => "任务已进入评审，但还没有生成可展示事件。".to_string(),
        "done" => "任务已完成。".to_string(),
        "blocked" => projection
            .audit
            .findings
            .first()
            .cloned()
            .unwrap_or_else(|| "任务被阻断。".to_string()),
        "cancel" => "任务已取消。".to_string(),
        _ => "当前状态没有额外解释。".to_string(),
    }
}

fn explain_run_state(
    run_state: &str,
    event_stream: &[WorkLoopEventView],
    task_run: &agentflow_task_artifacts::TaskRun,
) -> String {
    if let Some(last_event) = event_stream.last() {
        return format!(
            "当前 run 状态是 {run_state}，最近事件是：{}。",
            last_event.summary
        );
    }
    match run_state {
        "queued" => "当前 run 已创建，等待真正进入执行。".to_string(),
        "in_progress" => "当前 run 正在执行。".to_string(),
        "validating" => "当前 run 正在收集验证结果。".to_string(),
        "completed" => "当前 run 已完成，等待或已经进入后续写回。".to_string(),
        "failed" => task_run
            .last_error
            .clone()
            .map(|error| format!("当前 run 已失败：{error}。"))
            .unwrap_or_else(|| "当前 run 已失败。".to_string()),
        "cancelled" => "当前 run 已取消。".to_string(),
        _ => "当前 run 状态未知。".to_string(),
    }
}

fn explain_session_state(
    session_record: &agentflow_task_artifacts::TaskWorkSessionRecord,
    recovery_summary: Option<&agentflow_task_artifacts::TaskWorkSessionRecoverySummary>,
    event_stream: &[WorkLoopEventView],
) -> String {
    if let Some(last_event) = event_stream.last() {
        return format!(
            "当前会话状态是 {}，最近事件是：{}。",
            session_record.status.as_str(),
            last_event.summary
        );
    }
    if let Some(summary) = recovery_summary {
        return format!(
            "当前会话处于 {}，恢复原因：{}。",
            session_record.status.as_str(),
            summary
                .recovery_reason
                .clone()
                .unwrap_or_else(|| "未记录".to_string())
        );
    }
    format!("当前会话状态是 {}。", session_record.status.as_str())
}

fn find_session_projection_context(
    project_root: impl AsRef<Path>,
    session_id: &str,
) -> Result<(String, String, TaskProjection)> {
    let index = load_issue_status_index(&project_root)?;
    for entry in index.issues {
        let projection = load_task_projection(&project_root, &entry.issue_id)?;
        let Some(run_id) = projection.latest_run_id.clone() else {
            continue;
        };
        if projection.session.session_id.as_deref() == Some(session_id)
            || load_task_session_history_record(&project_root, &entry.issue_id, &run_id, session_id)
                .is_ok()
        {
            return Ok((entry.issue_id, run_id, projection));
        }
    }
    anyhow::bail!("failed to locate work session `{session_id}` in task projections")
}

fn task_run_status_label(status: &agentflow_task_artifacts::TaskRunStatus) -> &'static str {
    match status {
        agentflow_task_artifacts::TaskRunStatus::Queued => "queued",
        agentflow_task_artifacts::TaskRunStatus::InProgress => "in_progress",
        agentflow_task_artifacts::TaskRunStatus::Validating => "validating",
        agentflow_task_artifacts::TaskRunStatus::Completed => "completed",
        agentflow_task_artifacts::TaskRunStatus::Failed => "failed",
        agentflow_task_artifacts::TaskRunStatus::Cancelled => "cancelled",
    }
}

fn explain_projection_staleness(
    project_root: impl AsRef<Path>,
    scope: ProjectionScope,
    projection_version: &str,
    last_rebuilt_at: u64,
    projection_cursor: Option<String>,
) -> Result<ProjectionFreshness> {
    let latest = latest_event_summary(project_root, &scope)?;
    let mut warnings = latest.warnings;
    if latest.last_event_id.is_none() {
        warnings.push("no-runtime-event-yet".to_string());
    }

    let staleness = if latest.last_event_id.is_none() {
        "empty".to_string()
    } else if projection_cursor
        .as_ref()
        .zip(latest.last_event_id.as_ref())
        .is_some_and(|(projection_event_id, latest_event_id)| {
            projection_event_id != latest_event_id
        })
    {
        "stale".to_string()
    } else if latest
        .last_event_timestamp
        .is_some_and(|timestamp| timestamp > last_rebuilt_at)
    {
        "stale".to_string()
    } else {
        "current".to_string()
    };
    let source_refs = scope.source_refs();
    let projection_ref = format!("{}:{projection_version}", scope.key());
    let receipt = projection_freshness_receipt(
        &projection_ref,
        source_refs,
        latest.last_event_id.as_deref(),
        latest.last_event_type.as_deref(),
        latest.last_event_timestamp,
        last_rebuilt_at,
        &staleness,
        &warnings,
    );

    Ok(ProjectionFreshness {
        projection_version: projection_version.to_string(),
        query_surface_version: PROJECTION_QUERY_SURFACE_VERSION.to_string(),
        last_event_id: latest.last_event_id,
        last_event_type: latest.last_event_type,
        last_event_timestamp: latest.last_event_timestamp,
        last_rebuilt_at,
        staleness,
        definition_versions: latest.definition_versions,
        receipt,
        warnings,
    })
}

#[derive(Debug, Clone)]
struct LatestEventSummary {
    last_event_id: Option<String>,
    last_event_type: Option<String>,
    last_event_timestamp: Option<u64>,
    definition_versions: ProjectionDefinitionVersions,
    warnings: Vec<String>,
}

fn latest_event_summary(
    project_root: impl AsRef<Path>,
    scope: &ProjectionScope,
) -> Result<LatestEventSummary> {
    let filter = match scope {
        ProjectionScope::RequirementPreview { project_id }
        | ProjectionScope::Project { project_id } => ReplayFilter {
            project_id: Some(project_id.clone()),
            ..ReplayFilter::default()
        },
        ProjectionScope::Issue { issue_id } => ReplayFilter::issue(issue_id.clone()),
        ProjectionScope::Audit { source_issue_id } => source_issue_id
            .as_ref()
            .map(|issue_id| ReplayFilter::issue(issue_id.clone()))
            .unwrap_or_default(),
    };
    let events = replay_task_events(project_root, filter)?;
    let Some(last_event) = events.last() else {
        return Ok(LatestEventSummary {
            last_event_id: None,
            last_event_type: None,
            last_event_timestamp: None,
            definition_versions: ProjectionDefinitionVersions {
                ontology_version: "unavailable".to_string(),
                action_contract_version: "unavailable".to_string(),
                role_policy_version: "unavailable".to_string(),
                state_machine_version: "unavailable".to_string(),
            },
            warnings: Vec::new(),
        });
    };
    let compatibility = map_task_event_to_runtime_event(last_event)?;
    Ok(LatestEventSummary {
        last_event_id: Some(last_event.event_id.clone()),
        last_event_type: Some(last_event.event_type.clone()),
        last_event_timestamp: Some(last_event.timestamp),
        definition_versions: ProjectionDefinitionVersions {
            ontology_version: compatibility.envelope.ontology_version,
            action_contract_version: compatibility.envelope.action_contract_version,
            role_policy_version: compatibility.envelope.role_policy_version,
            state_machine_version: compatibility.envelope.state_machine_version,
        },
        warnings: compatibility.warnings,
    })
}

fn recent_events(
    project_root: impl AsRef<Path>,
    filter: ReplayFilter,
    limit: usize,
) -> Result<Vec<RuntimeEventRow>> {
    let events = replay_task_events(project_root, filter)?;
    Ok(events
        .into_iter()
        .rev()
        .take(limit)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .map(runtime_event_row)
        .collect())
}

fn runtime_event_row(event: TaskEvent) -> RuntimeEventRow {
    let summary = work_loop_event_summary(&event);
    RuntimeEventRow {
        event_id: event.event_id.clone(),
        event_type: event.event_type.clone(),
        timestamp: event.timestamp,
        actor_role: event.actor.role.clone(),
        summary,
    }
}

fn next_action_hints(key: &str, label: &str, reason: &str) -> Vec<ViewActionHint> {
    if key.is_empty() {
        return Vec::new();
    }
    vec![ViewActionHint {
        key: key.to_string(),
        label: label.to_string(),
        reason: reason.to_string(),
    }]
}

fn task_allowed_actions(projection: &TaskProjection) -> Vec<ViewActionHint> {
    match projection.display_status.as_str() {
        "backlog" => vec![hint(
            "schedule",
            "等待调度",
            "当前任务还在 backlog，等待 Task Loop 排入执行。",
        )],
        "todo" => vec![hint(
            "launch",
            "准备启动",
            "当前任务已经满足开工前置条件，下一步是拉起执行会话。",
        )],
        "in_progress" => vec![hint(
            "observe-runtime",
            "查看运行态",
            "当前任务正在执行，优先观察实时事件和验证输出。",
        )],
        "in_review" => vec![hint(
            "review-closeout",
            "检查交付",
            "当前任务已进入 review，下一步核对 PR、验证证据和合并证明。",
        )],
        "done" => vec![hint(
            "view-delivery",
            "查看交付",
            "当前任务已经完成，优先查看公开交付和验证证据。",
        )],
        "blocked" => vec![hint(
            "inspect-blocker",
            "查看阻断",
            "当前任务存在阻断，先处理依赖、证据或工作区问题。",
        )],
        "cancel" => vec![hint(
            "inspect-cancel",
            "查看取消原因",
            "当前任务已取消，只保留历史事实和关闭原因。",
        )],
        _ => Vec::new(),
    }
}

fn evidence_projection_status(evaluation: &CoreEvidenceCompletenessEvaluation) -> String {
    match evaluation.outcome.as_str() {
        "complete" => "passed",
        "invalid" => "invalid",
        "deferred" | "incomplete" => "deferred",
        _ => "invalid",
    }
    .to_string()
}

fn evidence_trace_refs(trace_refs: &CoreEvidenceTraceRefs) -> Vec<String> {
    trace_refs
        .spec_refs
        .iter()
        .chain(trace_refs.goal_refs.iter())
        .chain(trace_refs.task_refs.iter())
        .chain(trace_refs.run_refs.iter())
        .chain(trace_refs.action_refs.iter())
        .chain(trace_refs.decision_refs.iter())
        .cloned()
        .collect()
}

fn evidence_kernel_freshness() -> ProjectionFreshness {
    let receipt = projection_freshness_receipt(
        "core:evidence-kernel:evidence-kernel-read-model.v1",
        vec![
            "crates/ontology/src/evidence.rs".to_string(),
            "docs/architecture/068-evidence-projection-read-model-v1.md".to_string(),
        ],
        None,
        None,
        None,
        0,
        "readonly-derived",
        &[],
    );
    ProjectionFreshness {
        projection_version: EVIDENCE_KERNEL_READ_MODEL_VERSION.to_string(),
        query_surface_version: PROJECTION_QUERY_SURFACE_VERSION.to_string(),
        last_event_id: None,
        last_event_type: None,
        last_event_timestamp: None,
        last_rebuilt_at: 0,
        staleness: "readonly-derived".to_string(),
        definition_versions: ProjectionDefinitionVersions {
            ontology_version: "agentflow-core-evidence-pack.v1".to_string(),
            action_contract_version: "agentflow-core-evidence-completeness-policy.v1".to_string(),
            role_policy_version: "not-applicable".to_string(),
            state_machine_version: "not-applicable".to_string(),
        },
        receipt,
        warnings: Vec::new(),
    }
}

fn delivery_allowed_actions(projection: &TaskProjection) -> Vec<ViewActionHint> {
    match projection.delivery.status.as_str() {
        "ready" | "published" => vec![hint(
            "view-public-delivery",
            "查看公开交付",
            "公开交付记录已经可读，优先核对 PR、CHANGELOG 或 release notes。",
        )],
        "drafted" => vec![hint(
            "review-public-delivery",
            "检查交付草稿",
            "公开交付草稿已生成，但还没有进入最终公开状态。",
        )],
        _ => vec![hint(
            "wait-closeout",
            "等待收口",
            "交付记录还没有就绪，先完成验证、合并证明和 Done 写回。",
        )],
    }
}

fn audit_allowed_actions(summary: &agentflow_audit::AuditResultSummary) -> Vec<ViewActionHint> {
    match summary.status.as_str() {
        "requested" | "in_progress" => vec![hint(
            "follow-audit",
            "跟踪审计",
            "审计正在进行，先查看检查点、证据映射和 findings。",
        )],
        "passed" => vec![hint(
            "accept-audit",
            "查看通过结论",
            "审计已经通过，优先核对最终结论与 traceability。",
        )],
        "failed" => vec![hint(
            "repair-from-finding",
            "处理 findings",
            "审计失败，下一步根据 findings 创建修复任务。",
        )],
        _ => vec![hint(
            "view-audit",
            "查看审计记录",
            "当前审计记录已存在，可以直接查看事实和结论。",
        )],
    }
}

fn task_blocked_reasons(issue: &SpecIssue, projection: &TaskProjection) -> Vec<String> {
    if projection.display_status != "blocked" {
        return Vec::new();
    }
    if !issue.blocked_by.is_empty() {
        return issue
            .blocked_by
            .iter()
            .map(|dependency| format!("依赖未完成: {dependency}"))
            .collect();
    }
    projection
        .timeline
        .iter()
        .filter(|item| item.phase.as_str() == "exception")
        .map(|item| item.summary.clone())
        .collect()
}

fn issue_acceptance_mapping(
    issue: &SpecIssue,
    delivery: &ProjectionDeliverySummary,
) -> Vec<String> {
    let mut mapping = issue
        .validation_commands
        .iter()
        .map(|command| format!("验证命令: {command}"))
        .collect::<Vec<_>>();
    mapping.push(format!(
        "证据输出: {}",
        issue.expected_outputs.evidence_path
    ));
    mapping.push(format!(
        "任务运行目录: {}",
        issue.expected_outputs.task_run_dir
    ));
    mapping.push(format!(
        "公开交付: {}",
        issue
            .expected_outputs
            .public_delivery_record
            .changelog_or_release_notes
    ));
    if !delivery.summary_line.is_empty() {
        mapping.push(format!("当前交付总结: {}", delivery.summary_line));
    }
    mapping
}

fn delivery_summary_line(
    delivery: &ProjectionDeliverySummary,
    public_delivery: &ProjectionPublicDelivery,
) -> String {
    if !delivery.summary_line.is_empty() {
        return delivery.summary_line.clone();
    }
    if let Some(pr_url) = public_delivery.pr_url.as_ref() {
        return format!("PR/MR: {pr_url}");
    }
    "公开交付待生成".to_string()
}

fn spec_issue_preview_item(issue: &SpecIssue) -> IssuePreviewItem {
    IssuePreviewItem {
        issue_id: issue.issue_id.clone(),
        title: issue.title.clone(),
        summary: issue.summary.clone(),
        priority: priority_label(&issue.priority),
        required_agent_role: required_role_label(&issue.required_agent_role),
        blocked_by: issue.blocked_by.clone(),
    }
}

fn priority_label(priority: &SpecPriority) -> String {
    match priority {
        SpecPriority::P0 => "P0".to_string(),
        SpecPriority::P1 => "P1".to_string(),
        SpecPriority::P2 => "P2".to_string(),
        SpecPriority::P3 => "P3".to_string(),
    }
}

fn required_role_label(role: &SpecRequiredAgentRole) -> String {
    role.provider_role_alias().to_string()
}

fn hint(key: &str, label: &str, reason: &str) -> ViewActionHint {
    ViewActionHint {
        key: key.to_string(),
        label: label.to_string(),
        reason: reason.to_string(),
    }
}

fn surface_read_model(
    kind: &str,
    object_type: &str,
    object_id: &str,
    title: &str,
    status: &str,
    query_name: &str,
    query_args: Vec<String>,
    projection_path: &str,
    source_refs: Vec<String>,
    freshness: ProjectionFreshness,
) -> ProjectionSurfaceReadModelView {
    surface_read_model_with_missing(
        kind,
        object_type,
        object_id,
        title,
        status,
        query_name,
        query_args,
        projection_path,
        source_refs,
        freshness,
        Vec::new(),
    )
}

fn surface_read_model_with_missing(
    kind: &str,
    object_type: &str,
    object_id: &str,
    title: &str,
    status: &str,
    query_name: &str,
    query_args: Vec<String>,
    projection_path: &str,
    source_refs: Vec<String>,
    freshness: ProjectionFreshness,
    missing_facts: Vec<String>,
) -> ProjectionSurfaceReadModelView {
    let key = format!("{kind}:{object_type}:{object_id}");
    let feedback = feedback_route_for_surface(&key, status, &freshness, &missing_facts);
    ProjectionSurfaceReadModelView {
        key,
        kind: kind.to_string(),
        object_type: object_type.to_string(),
        object_id: object_id.to_string(),
        title: title.to_string(),
        status: status.to_string(),
        query: ProjectionSurfaceQueryView {
            name: query_name.to_string(),
            args: query_args,
        },
        projection_path: projection_path.to_string(),
        source_refs,
        authority: false,
        freshness,
        feedback,
        missing_facts,
    }
}

fn catalog_freshness(
    read_models: &[ProjectionSurfaceReadModelView],
    warnings: Vec<String>,
) -> ProjectionFreshness {
    let last_rebuilt_at = read_models
        .iter()
        .map(|entry| entry.freshness.last_rebuilt_at)
        .max()
        .unwrap_or(0);
    let incomplete = read_models
        .iter()
        .any(|entry| !entry.missing_facts.is_empty());
    let staleness = if incomplete {
        "incomplete".to_string()
    } else if read_models.is_empty() {
        "empty".to_string()
    } else {
        "current".to_string()
    };
    let source_refs = read_models
        .iter()
        .flat_map(|entry| entry.freshness.receipt.source_refs.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let receipt = projection_freshness_receipt(
        "catalog:projection-surface-catalog.v1",
        source_refs,
        read_models
            .iter()
            .filter_map(|entry| entry.freshness.last_event_id.as_deref())
            .last(),
        read_models
            .iter()
            .filter_map(|entry| entry.freshness.last_event_type.as_deref())
            .last(),
        read_models
            .iter()
            .filter_map(|entry| entry.freshness.last_event_timestamp)
            .max(),
        last_rebuilt_at,
        &staleness,
        &warnings,
    );
    ProjectionFreshness {
        projection_version: "projection-surface-catalog.v1".to_string(),
        query_surface_version: PROJECTION_QUERY_SURFACE_VERSION.to_string(),
        last_event_id: read_models
            .iter()
            .filter_map(|entry| entry.freshness.last_event_id.clone())
            .last(),
        last_event_type: read_models
            .iter()
            .filter_map(|entry| entry.freshness.last_event_type.clone())
            .last(),
        last_event_timestamp: read_models
            .iter()
            .filter_map(|entry| entry.freshness.last_event_timestamp)
            .max(),
        last_rebuilt_at,
        staleness,
        definition_versions: read_models
            .iter()
            .find_map(|entry| {
                (entry.freshness.definition_versions.ontology_version != "unavailable")
                    .then(|| entry.freshness.definition_versions.clone())
            })
            .unwrap_or_else(unavailable_definition_versions),
        receipt,
        warnings,
    }
}

fn missing_freshness(reason: &str) -> ProjectionFreshness {
    let warnings = vec![reason.to_string()];
    let receipt = projection_freshness_receipt(
        "missing:projection",
        vec!["projection-source-missing".to_string()],
        None,
        None,
        None,
        0,
        "missing",
        &warnings,
    );
    ProjectionFreshness {
        projection_version: "missing".to_string(),
        query_surface_version: PROJECTION_QUERY_SURFACE_VERSION.to_string(),
        last_event_id: None,
        last_event_type: None,
        last_event_timestamp: None,
        last_rebuilt_at: 0,
        staleness: "missing".to_string(),
        definition_versions: unavailable_definition_versions(),
        receipt,
        warnings,
    }
}

fn feedback_route_for_surface(
    source_surface_key: &str,
    status: &str,
    freshness: &ProjectionFreshness,
    missing_facts: &[String],
) -> ProjectionFeedbackRoute {
    if !missing_facts.is_empty() {
        return ProjectionFeedbackRoute {
            status: "blocked".to_string(),
            route: "repair-projection-inputs".to_string(),
            reason: format!("projection facts missing: {}", missing_facts.join(", ")),
            source_surface_key: source_surface_key.to_string(),
            target_authority: ".agentflow/spec/**".to_string(),
            proposal_kind: "spec-evolution-preview".to_string(),
            requires_confirmation: true,
            confirmation_boundary: "preview-confirmation-materialization-required".to_string(),
            writes_authority: false,
        };
    }
    if freshness.staleness == "stale" || freshness.staleness == "incomplete" {
        return ProjectionFeedbackRoute {
            status: "ready-for-spec-evolution".to_string(),
            route: "open-spec-evolution-preview".to_string(),
            reason: stale_feedback_reason(freshness),
            source_surface_key: source_surface_key.to_string(),
            target_authority: ".agentflow/spec/**".to_string(),
            proposal_kind: "spec-evolution-preview".to_string(),
            requires_confirmation: true,
            confirmation_boundary: "preview-confirmation-materialization-required".to_string(),
            writes_authority: false,
        };
    }
    let status = if status == "blocked" {
        "blocked"
    } else {
        "accepted"
    };
    ProjectionFeedbackRoute {
        status: status.to_string(),
        route: "observe-projection".to_string(),
        reason: "projection is read-only and current enough for display".to_string(),
        source_surface_key: source_surface_key.to_string(),
        target_authority: ".agentflow/spec/**".to_string(),
        proposal_kind: "none".to_string(),
        requires_confirmation: false,
        confirmation_boundary: "not-applicable".to_string(),
        writes_authority: false,
    }
}

fn stale_feedback_reason(freshness: &ProjectionFreshness) -> String {
    freshness
        .receipt
        .stale_reason
        .clone()
        .unwrap_or_else(|| "projection freshness is not current".to_string())
}

fn projection_freshness_receipt(
    projection_ref: &str,
    source_refs: Vec<String>,
    last_event_id: Option<&str>,
    last_event_type: Option<&str>,
    last_event_timestamp: Option<u64>,
    last_rebuilt_at: u64,
    staleness: &str,
    warnings: &[String],
) -> ProjectionFreshnessReceipt {
    let stale_reason = projection_stale_reason(
        staleness,
        last_event_id,
        last_event_type,
        last_event_timestamp,
        last_rebuilt_at,
        warnings,
    );
    let mut parts = Vec::new();
    parts.push(format!("projectionRef={projection_ref}"));
    parts.push(format!("staleness={staleness}"));
    parts.push(format!("lastEventId={}", last_event_id.unwrap_or("none")));
    parts.push(format!(
        "lastEventType={}",
        last_event_type.unwrap_or("none")
    ));
    parts.push(format!(
        "lastEventTimestamp={}",
        last_event_timestamp
            .map(|timestamp| timestamp.to_string())
            .unwrap_or_else(|| "none".to_string())
    ));
    parts.push(format!("lastRebuiltAt={last_rebuilt_at}"));
    parts.extend(source_refs.iter().map(|source| format!("source={source}")));
    let source_digest = stable_query_digest(&parts.join("\n"));
    let receipt_short = source_digest
        .strip_prefix("fnv1a64:")
        .unwrap_or(source_digest.as_str());
    ProjectionFreshnessReceipt {
        version: PROJECTION_FRESHNESS_RECEIPT_VERSION.to_string(),
        receipt_id: format!("projection-freshness-{receipt_short}"),
        projection_ref: projection_ref.to_string(),
        source_refs,
        source_digest,
        rebuild_receipt_ref: ".agentflow/projections/replay-report.json".to_string(),
        status: staleness.to_string(),
        stale_reason,
        generated_at: last_rebuilt_at,
        writes_authority: false,
    }
}

fn projection_stale_reason(
    staleness: &str,
    last_event_id: Option<&str>,
    last_event_type: Option<&str>,
    last_event_timestamp: Option<u64>,
    last_rebuilt_at: u64,
    warnings: &[String],
) -> Option<String> {
    match staleness {
        "current" | "readonly-derived" => None,
        "empty" => Some("no runtime event has been recorded for this projection scope".to_string()),
        "missing" | "incomplete" => warnings.first().cloned().or_else(|| {
            Some("required projection source facts are missing or incomplete".to_string())
        }),
        "stale" => {
            if let Some(event_timestamp) = last_event_timestamp {
                if event_timestamp > last_rebuilt_at {
                    return Some(format!(
                        "latest event {} at {} is newer than projection rebuild {}",
                        last_event_id.unwrap_or("unknown"),
                        event_timestamp,
                        last_rebuilt_at
                    ));
                }
            }
            Some(format!(
                "projection cursor is behind latest event {} ({})",
                last_event_id.unwrap_or("unknown"),
                last_event_type.unwrap_or("unknown")
            ))
        }
        other => Some(format!("projection freshness status is {other}")),
    }
}

fn stable_query_digest(input: &str) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in input.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("fnv1a64:{hash:016x}")
}

fn unavailable_definition_versions() -> ProjectionDefinitionVersions {
    ProjectionDefinitionVersions {
        ontology_version: "unavailable".to_string(),
        action_contract_version: "unavailable".to_string(),
        role_policy_version: "unavailable".to_string(),
        state_machine_version: "unavailable".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs;

    use agentflow_event_store::{append_task_event_once, EventActor, TaskEventDraft};
    use agentflow_spec::{
        confirm_goal_draft_preview, confirm_plan_draft_preview, issue_from_requirement,
        materialize_spec_from_requirement_preview, project_from_requirement,
        requirement_preview_from_requirement, write_spec_issue, write_spec_project, SpecIssueDraft,
        SpecIssueStatus, SpecProjectDraft,
    };
    use agentflow_task_artifacts::{
        create_task_run, sync_task_session, update_task_run_status, TaskRunStatus,
        TaskSessionMirror,
    };
    use agentflow_workflow_core::{WorkflowAgentRole, WorkflowFlowType};
    use serde_json::json;
    use tempfile::tempdir;

    use crate::projector::rebuild_projections;

    fn write_fixture(root: &Path) {
        let requirement = root.join("docs/requirements/034-test.md");
        fs::create_dir_all(requirement.parent().unwrap()).unwrap();
        fs::write(&requirement, "# 测试需求\n\n用于 projection query 测试。\n").unwrap();
        let project_docs = root.join("docs/projects/project-projection");
        fs::create_dir_all(&project_docs).unwrap();
        fs::write(project_docs.join("GOAL.md"), "# Goal\n\n确认目标。\n").unwrap();
        fs::write(project_docs.join("PLAN.md"), "# Plan\n\n确认计划。\n").unwrap();
        fs::write(
            project_docs.join("DECISIONS.md"),
            "# Decisions\n\n## Decision Log\n\n### 2026-06-18 - Goal confirmation\n",
        )
        .unwrap();

        let mut issue = SpecIssueDraft::new("AF-PROJ-001");
        issue.project_id = Some("project-projection".to_string());
        issue.validation_commands = vec!["cargo test -p agentflow-projection".to_string()];
        let issue = issue_from_requirement(root, &requirement, issue).unwrap();
        write_spec_issue(root, &issue).unwrap();

        let mut project = SpecProjectDraft::new("project-projection");
        project.issue_ids = vec!["AF-PROJ-001".to_string()];
        let project = project_from_requirement(root, &requirement, project).unwrap();
        write_spec_project(root, &project).unwrap();
    }

    fn write_completion_ready_artifacts(root: &Path, issue_id: &str, run_id: &str) {
        let task_root = root.join(".agentflow/tasks").join(issue_id);
        let evidence_dir = task_root.join("evidence");
        fs::create_dir_all(&evidence_dir).unwrap();
        fs::write(
            evidence_dir.join("evidence.json"),
            serde_json::to_string_pretty(&json!({
                "version": "task-evidence.v1",
                "issueId": issue_id,
                "runId": run_id,
                "status": "ready",
                "summary": "本地验证通过。",
                "runPath": format!(".agentflow/tasks/{issue_id}/runs/{run_id}/run.json"),
                "commandPaths": [format!(".agentflow/tasks/{issue_id}/runs/{run_id}/verify/local.log")],
                "validationPath": format!(".agentflow/tasks/{issue_id}/runs/{run_id}/validation.json"),
                "createdAt": 1
            }))
            .unwrap(),
        )
        .unwrap();
    }

    fn sync_running_session(root: &Path, issue_id: &str, run_id: &str, session_id: &str) {
        sync_task_session(
            root,
            issue_id,
            run_id,
            &TaskSessionMirror {
                provider: "codex".to_string(),
                session_owner: "work-agent".to_string(),
                session_id: session_id.to_string(),
                status: agentflow_task_artifacts::TaskWorkSessionStatus::Running,
                branch_name: Some(format!("agentflow/{issue_id}/{run_id}")),
                working_directory: root.display().to_string(),
                workspace_root: Some(root.display().to_string()),
                worktree_root: Some(root.display().to_string()),
                runtime_root: Some(
                    root.join(format!(".agentflow/tasks/{issue_id}/runs/{run_id}/runtime"))
                        .display()
                        .to_string(),
                ),
                temp_root: None,
                cache_root: None,
                evidence_root: Some(
                    root.join(format!(".agentflow/tasks/{issue_id}/evidence"))
                        .display()
                        .to_string(),
                ),
                launch_request_path: format!(
                    ".agentflow/tasks/{issue_id}/runs/{run_id}/launch/agent-request.json"
                ),
                plan_path: format!(".agentflow/tasks/{issue_id}/runs/{run_id}/plan.json"),
                log_path: Some(format!(
                    ".agentflow/tasks/{issue_id}/runs/{run_id}/runtime.log"
                )),
                last_message_path: None,
                exit_proof_path: None,
                merge_proof_path: None,
                started_at: 10,
                last_heartbeat_at: 40,
                attempt_count: 2,
                retry_policy: Some("reuse-session-or-relaunch".to_string()),
                max_attempts: Some(3),
                resumed_from_attempt: Some(1),
                retryable: true,
                recovery_reason: Some("timeout".to_string()),
                merge_state: None,
                writeback_state: None,
                terminal_reason: None,
                last_error: None,
                exited_at: None,
                exit_code: None,
                updated_at: 40,
            },
        )
        .unwrap();
    }

    fn event(issue_id: &str, event_type: &str, payload: serde_json::Value) -> TaskEventDraft {
        TaskEventDraft {
            flow_type: WorkflowFlowType::Work,
            aggregate_type: "issue".to_string(),
            aggregate_id: issue_id.to_string(),
            project_id: Some("project-projection".to_string()),
            issue_id: Some(issue_id.to_string()),
            run_id: payload
                .get("runId")
                .and_then(serde_json::Value::as_str)
                .map(str::to_string),
            event_type: event_type.to_string(),
            authority_role: Some(WorkflowAgentRole::WorkAgent),
            actor: EventActor {
                role: "test".to_string(),
                kind: "system".to_string(),
            },
            state: None,
            correlation_id: Some(format!("corr-{issue_id}")),
            causation_id: None,
            payload,
            artifact_refs: Vec::new(),
            idempotency_key: Some(format!("{event_type}:{issue_id}")),
        }
    }

    #[test]
    fn task_workbench_view_separates_issue_and_run_state() {
        let dir = tempdir().unwrap();
        write_fixture(dir.path());
        append_task_event_once(
            dir.path(),
            event("AF-PROJ-001", "issue.scheduled", json!({})),
        )
        .unwrap();
        append_task_event_once(
            dir.path(),
            event(
                "AF-PROJ-001",
                "agent.launch.requested",
                json!({"runId":"run-001","branchName":"agentflow/project-projection/AF-PROJ-001"}),
            ),
        )
        .unwrap();
        create_task_run(
            dir.path(),
            "AF-PROJ-001",
            "run-001",
            "work-agent.issue-loop@v1",
            Some("agentflow/project-projection/AF-PROJ-001".to_string()),
        )
        .unwrap();
        update_task_run_status(
            dir.path(),
            "AF-PROJ-001",
            "run-001",
            TaskRunStatus::Validating,
        )
        .unwrap();

        rebuild_projections(dir.path()).unwrap();
        let view = get_task_workbench_view(dir.path(), "AF-PROJ-001").unwrap();

        assert_eq!(view.issue_state, "in_progress");
        assert_eq!(view.run_state, "queued");
        assert_eq!(view.active_run.as_deref(), Some("run-001"));
        assert_ne!(view.issue_state, view.run_state);
        assert_eq!(view.freshness.staleness, "current");
    }

    #[test]
    fn task_workbench_view_exposes_event_stream_and_evidence_summary() {
        let dir = tempdir().unwrap();
        write_fixture(dir.path());
        append_task_event_once(
            dir.path(),
            event("AF-PROJ-001", "issue.scheduled", json!({})),
        )
        .unwrap();
        append_task_event_once(
            dir.path(),
            event(
                "AF-PROJ-001",
                "agent.launch.requested",
                json!({"runId":"run-001","branchName":"agentflow/project-projection/AF-PROJ-001"}),
            ),
        )
        .unwrap();
        append_task_event_once(
            dir.path(),
            event(
                "AF-PROJ-001",
                "agent.session.running",
                json!({"runId":"run-001","sessionId":"codex-run-001","sessionStatus":"running","provider":"codex","ownerId":"work-agent"}),
            ),
        )
        .unwrap();
        append_task_event_once(
            dir.path(),
            event(
                "AF-PROJ-001",
                "issue.validation.passed",
                json!({"runId":"run-001"}),
            ),
        )
        .unwrap();
        create_task_run(
            dir.path(),
            "AF-PROJ-001",
            "run-001",
            "work-agent.issue-loop@v1",
            Some("agentflow/project-projection/AF-PROJ-001".to_string()),
        )
        .unwrap();
        sync_running_session(dir.path(), "AF-PROJ-001", "run-001", "codex-run-001");
        write_completion_ready_artifacts(dir.path(), "AF-PROJ-001", "run-001");

        rebuild_projections(dir.path()).unwrap();
        let view = get_task_workbench_view(dir.path(), "AF-PROJ-001").unwrap();

        assert!(view.state_explanation.contains("当前事实"));
        assert!(view
            .event_stream
            .iter()
            .any(|event| event.stage_key == "session"));
        assert!(view
            .event_stream
            .iter()
            .any(|event| event.event_type == "issue.validation.passed"));
        assert_eq!(view.evidence_summary.status, "running");
        assert!(!view.evidence_summary.verification_refs.is_empty());
        assert!(!view.evidence_summary.session_refs.is_empty());
    }

    #[test]
    fn delivery_view_shows_done_issue_without_audit_side_effect() {
        let dir = tempdir().unwrap();
        write_fixture(dir.path());
        let mut issue = read_spec_issue(dir.path(), "AF-PROJ-001").unwrap();
        issue.status = SpecIssueStatus::Done;
        write_spec_issue(dir.path(), &issue).unwrap();
        append_task_event_once(
            dir.path(),
            event("AF-PROJ-001", "issue.scheduled", json!({})),
        )
        .unwrap();
        append_task_event_once(
            dir.path(),
            event(
                "AF-PROJ-001",
                "agent.launch.requested",
                json!({"runId":"run-001"}),
            ),
        )
        .unwrap();
        append_task_event_once(
            dir.path(),
            event(
                "AF-PROJ-001",
                "issue.completed",
                json!({"runId":"run-001","mergeCommit":"abc123"}),
            ),
        )
        .unwrap();
        write_completion_ready_artifacts(dir.path(), "AF-PROJ-001", "run-001");

        rebuild_projections(dir.path()).unwrap();
        let view = get_delivery_package_view(dir.path(), "AF-PROJ-001").unwrap();
        let task = get_task_workbench_view(dir.path(), "AF-PROJ-001").unwrap();

        assert_eq!(view.issue_id, "AF-PROJ-001");
        assert!(!view.verification_logs.is_empty());
        assert_eq!(task.issue_state, "done");
        assert_eq!(task.freshness.staleness, "current");
    }

    #[test]
    fn work_loop_run_view_filters_events_and_keeps_done_writeback_visible() {
        let dir = tempdir().unwrap();
        write_fixture(dir.path());
        append_task_event_once(
            dir.path(),
            event("AF-PROJ-001", "issue.scheduled", json!({})),
        )
        .unwrap();
        append_task_event_once(
            dir.path(),
            event(
                "AF-PROJ-001",
                "agent.launch.requested",
                json!({"runId":"run-001"}),
            ),
        )
        .unwrap();
        append_task_event_once(
            dir.path(),
            event(
                "AF-PROJ-001",
                "issue.validation.passed",
                json!({"runId":"run-001"}),
            ),
        )
        .unwrap();
        append_task_event_once(
            dir.path(),
            event(
                "AF-PROJ-001",
                "issue.completed",
                json!({"runId":"run-001","mergeCommit":"abc123"}),
            ),
        )
        .unwrap();
        create_task_run(
            dir.path(),
            "AF-PROJ-001",
            "run-001",
            "work-agent.issue-loop@v1",
            Some("agentflow/project-projection/AF-PROJ-001".to_string()),
        )
        .unwrap();
        update_task_run_status(
            dir.path(),
            "AF-PROJ-001",
            "run-001",
            TaskRunStatus::Completed,
        )
        .unwrap();
        write_completion_ready_artifacts(dir.path(), "AF-PROJ-001", "run-001");

        rebuild_projections(dir.path()).unwrap();
        let view = get_work_loop_run_view(dir.path(), "AF-PROJ-001", "run-001").unwrap();

        assert_eq!(view.run_id, "run-001");
        assert_eq!(view.run_state, "completed");
        assert!(view.state_explanation.contains("Done 写回"));
        assert_eq!(
            view.event_stream
                .last()
                .map(|event| event.event_type.as_str()),
            Some("issue.completed")
        );
        assert!(!view.evidence_summary.verification_refs.is_empty());
    }

    #[test]
    fn work_loop_session_view_reads_recovery_and_session_evidence() {
        let dir = tempdir().unwrap();
        write_fixture(dir.path());
        append_task_event_once(
            dir.path(),
            event("AF-PROJ-001", "issue.scheduled", json!({})),
        )
        .unwrap();
        append_task_event_once(
            dir.path(),
            event(
                "AF-PROJ-001",
                "agent.launch.requested",
                json!({"runId":"run-001"}),
            ),
        )
        .unwrap();
        append_task_event_once(
            dir.path(),
            event(
                "AF-PROJ-001",
                "agent.session.interrupted",
                json!({
                    "runId":"run-001",
                    "sessionId":"codex-run-001",
                    "sessionStatus":"interrupted",
                    "provider":"codex",
                    "ownerId":"work-agent",
                    "startedAt":10,
                    "lastHeartbeatAt":40,
                    "recoveryReason":"timeout"
                }),
            ),
        )
        .unwrap();
        create_task_run(
            dir.path(),
            "AF-PROJ-001",
            "run-001",
            "work-agent.issue-loop@v1",
            Some("agentflow/project-projection/AF-PROJ-001".to_string()),
        )
        .unwrap();
        sync_running_session(dir.path(), "AF-PROJ-001", "run-001", "codex-run-001");

        rebuild_projections(dir.path()).unwrap();
        let view = get_work_loop_session_view(dir.path(), "codex-run-001").unwrap();

        assert_eq!(view.issue_id, "AF-PROJ-001");
        assert_eq!(view.run_id, "run-001");
        assert_eq!(view.session_status.as_deref(), Some("running"));
        assert_eq!(view.recovery_reason.as_deref(), Some("timeout"));
        assert_eq!(view.resumed_from_attempt, Some(1));
        assert_eq!(view.attempt_count, 2);
        assert!(view
            .event_stream
            .iter()
            .any(|event| event.event_type == "agent.session.interrupted"));
        assert!(!view.evidence_summary.session_refs.is_empty());
    }

    #[test]
    fn freshness_turns_stale_when_new_issue_event_arrives_after_rebuild() {
        let dir = tempdir().unwrap();
        write_fixture(dir.path());
        append_task_event_once(
            dir.path(),
            event("AF-PROJ-001", "issue.scheduled", json!({})),
        )
        .unwrap();
        rebuild_projections(dir.path()).unwrap();
        let current = get_task_workbench_view(dir.path(), "AF-PROJ-001").unwrap();
        assert_eq!(current.freshness.staleness, "current");
        assert_eq!(
            current.freshness.receipt.version,
            PROJECTION_FRESHNESS_RECEIPT_VERSION
        );
        assert_eq!(current.freshness.receipt.status, "current");
        assert!(current.freshness.receipt.stale_reason.is_none());
        assert!(!current.freshness.receipt.source_refs.is_empty());
        assert!(current
            .freshness
            .receipt
            .source_digest
            .starts_with("fnv1a64:"));
        assert!(!current.freshness.receipt.writes_authority);

        append_task_event_once(
            dir.path(),
            event(
                "AF-PROJ-001",
                "agent.launch.requested",
                json!({"runId":"run-002"}),
            ),
        )
        .unwrap();
        let stale = get_task_workbench_view(dir.path(), "AF-PROJ-001").unwrap();
        assert_eq!(stale.freshness.staleness, "stale");
        assert_eq!(stale.freshness.receipt.status, "stale");
        assert!(stale.freshness.receipt.stale_reason.is_some());
        assert_eq!(
            stale.freshness.receipt.rebuild_receipt_ref,
            ".agentflow/projections/replay-report.json"
        );
        assert_ne!(
            current.freshness.last_event_id,
            stale.freshness.last_event_id
        );
    }

    #[test]
    fn spec_preview_view_uses_requirement_runtime_and_plan_drafts() {
        let dir = tempdir().unwrap();
        let requirement = dir.path().join("docs/requirements/040-preview.md");
        fs::create_dir_all(requirement.parent().unwrap()).unwrap();
        fs::write(&requirement, "# 预览\n\n先做 Goal / Plan Preview。\n").unwrap();

        requirement_preview_from_requirement(dir.path(), &requirement, Some("project-preview"))
            .unwrap();
        confirm_goal_draft_preview(dir.path(), "040-preview", "goal-agent").unwrap();
        confirm_plan_draft_preview(dir.path(), "040-preview", "spec-agent").unwrap();
        rebuild_projections(dir.path()).unwrap();

        let intake = get_requirement_intake_view(dir.path(), "040-preview").unwrap();
        let preview = get_spec_preview_view(dir.path(), "040-preview").unwrap();

        assert_eq!(intake.requirement_id, "040-preview");
        assert_eq!(preview.spec_id, "project-preview");
        assert!(!preview.issue_preview.is_empty());
        assert!(!preview.acceptance_criteria.is_empty());
    }

    #[test]
    fn spec_loop_view_covers_stage_files_traceability_and_action_proposals() {
        let dir = tempdir().unwrap();
        let requirement = dir.path().join("docs/requirements/041-spec-loop-view.md");
        fs::create_dir_all(requirement.parent().unwrap()).unwrap();
        fs::write(
            &requirement,
            "# Spec Loop View\n\n把需求转成 preview、confirmation、materialization 和 runtime action proposal。\n",
        )
        .unwrap();

        requirement_preview_from_requirement(dir.path(), &requirement, Some("project-preview"))
            .unwrap();
        confirm_goal_draft_preview(dir.path(), "041-spec-loop-view", "goal-agent").unwrap();
        confirm_plan_draft_preview(dir.path(), "041-spec-loop-view", "spec-agent").unwrap();
        materialize_spec_from_requirement_preview(dir.path(), "041-spec-loop-view").unwrap();
        rebuild_projections(dir.path()).unwrap();

        let view = get_spec_loop_view(dir.path(), "041-spec-loop-view").unwrap();

        assert_eq!(view.requirement_id, "041-spec-loop-view");
        assert_eq!(
            view.manifest_path,
            ".agentflow/spec/requirements/041-spec-loop-view/manifest.json"
        );
        assert_eq!(view.stages.len(), 8);
        assert!(view
            .stages
            .iter()
            .all(|stage| stage.authority_layer == "preview-artifact"));
        assert_eq!(
            view.stages
                .iter()
                .map(|stage| stage.stage.as_str())
                .collect::<Vec<_>>(),
            vec![
                "intake",
                "classification",
                "context",
                "boundary",
                "route",
                "preview",
                "confirmation",
                "materialization"
            ]
        );
        assert_eq!(
            view.materialized_project_id.as_deref(),
            Some("project-preview")
        );
        assert!(view.authority_layers.iter().any(|entry| {
            entry.authority_layer == "preview-artifact"
                && entry.path == ".agentflow/spec/requirements/041-spec-loop-view"
        }));
        assert!(view.authority_layers.iter().any(|entry| {
            entry.authority_layer == "project-authority"
                && entry.path == ".agentflow/spec/projects/project-preview.json"
        }));
        assert_eq!(
            view.authority_layers
                .iter()
                .filter(|entry| {
                    entry.authority_layer == "issue-authority"
                        && entry.path.starts_with(".agentflow/spec/issues/")
                })
                .count(),
            view.materialized_issue_ids.len()
        );
        assert!(view.authority_layers.iter().any(|entry| {
            entry.authority_layer == "derived-projection"
                && entry.path == ".agentflow/projections/spec-loops/041-spec-loop-view.json"
        }));
        assert_eq!(
            view.runtime_action_proposals.len(),
            1 + view.materialized_issue_ids.len()
        );
        assert!(view.traceability.iter().any(|edge| {
            edge.from_ref == "docs/requirements/041-spec-loop-view.md"
                && edge.to_ref.ends_with("/intake.json")
                && edge.relation == "stage-input"
        }));
        assert!(view.traceability.iter().any(|edge| {
            edge.from_ref.ends_with("/materialization.json")
                && edge
                    .to_ref
                    .starts_with("runtime-action-proposal:createProject:")
                && edge.relation == "runtime-action-proposal"
        }));
        assert_eq!(view.freshness.staleness, "current");
    }

    #[test]
    fn projection_surface_catalog_lists_unified_console_read_models() {
        let dir = tempdir().unwrap();
        write_fixture(dir.path());
        append_task_event_once(
            dir.path(),
            event("AF-PROJ-001", "issue.scheduled", json!({})),
        )
        .unwrap();

        let requirement = dir.path().join("docs/requirements/042-catalog.md");
        fs::create_dir_all(requirement.parent().unwrap()).unwrap();
        fs::write(
            &requirement,
            "# Projection Catalog\n\n统一投影查询面和 Console read model。\n",
        )
        .unwrap();
        requirement_preview_from_requirement(dir.path(), &requirement, Some("project-preview"))
            .unwrap();
        confirm_goal_draft_preview(dir.path(), "042-catalog", "goal-agent").unwrap();
        confirm_plan_draft_preview(dir.path(), "042-catalog", "spec-agent").unwrap();
        materialize_spec_from_requirement_preview(dir.path(), "042-catalog").unwrap();

        rebuild_projections(dir.path()).unwrap();
        let catalog = get_projection_surface_catalog(dir.path()).unwrap();

        assert_eq!(catalog.version, "projection-surface-catalog.v1");
        assert_eq!(
            catalog.query_surface_version,
            PROJECTION_QUERY_SURFACE_VERSION
        );
        for kind in [
            "requirement-intake",
            "spec-preview",
            "spec-loop",
            "project-home",
            "task-workbench",
            "delivery-package",
            "evidence-kernel",
            "runtime-health",
            "pack-industry-workbench",
        ] {
            assert!(
                catalog.read_models.iter().any(|entry| entry.kind == kind),
                "missing read model kind `{kind}`"
            );
        }
        assert!(catalog.read_models.iter().all(|entry| !entry.authority));
        assert!(catalog
            .read_models
            .iter()
            .any(|entry| entry.query.name == "get_task_workbench_view"
                && entry.object_id == "AF-PROJ-001"));
        assert!(catalog
            .read_models
            .iter()
            .any(|entry| entry.query.name == "get_spec_loop_view"
                && entry.object_id == "042-catalog"));
        assert!(
            catalog
                .read_models
                .iter()
                .all(|entry| entry.freshness.query_surface_version
                    == PROJECTION_QUERY_SURFACE_VERSION)
        );
        assert!(catalog.read_models.iter().all(|entry| {
            entry.freshness.receipt.version == PROJECTION_FRESHNESS_RECEIPT_VERSION
                && entry
                    .freshness
                    .receipt
                    .source_digest
                    .starts_with("fnv1a64:")
                && !entry.freshness.receipt.writes_authority
                && !entry.feedback.writes_authority
        }));
    }

    #[test]
    fn projection_surface_catalog_routes_stale_feedback_to_spec_evolution_preview() {
        let dir = tempdir().unwrap();
        write_fixture(dir.path());
        append_task_event_once(
            dir.path(),
            event("AF-PROJ-001", "issue.scheduled", json!({})),
        )
        .unwrap();
        rebuild_projections(dir.path()).unwrap();
        append_task_event_once(
            dir.path(),
            event(
                "AF-PROJ-001",
                "agent.launch.requested",
                json!({"runId":"run-002"}),
            ),
        )
        .unwrap();

        let catalog = get_projection_surface_catalog(dir.path()).unwrap();
        let task = catalog
            .read_models
            .iter()
            .find(|entry| entry.kind == "task-workbench" && entry.object_id == "AF-PROJ-001")
            .expect("task workbench read model should be present");

        assert_eq!(task.freshness.staleness, "stale");
        assert_eq!(task.feedback.status, "ready-for-spec-evolution");
        assert_eq!(task.feedback.route, "open-spec-evolution-preview");
        assert!(task.feedback.requires_confirmation);
        assert_eq!(
            task.feedback.confirmation_boundary,
            "preview-confirmation-materialization-required"
        );
        assert_eq!(task.feedback.target_authority, ".agentflow/spec/**");
        assert_eq!(task.feedback.proposal_kind, "spec-evolution-preview");
        assert!(!task.feedback.writes_authority);
    }

    #[test]
    fn evidence_kernel_read_model_projects_complete_fixture_without_authority() {
        let policy = agentflow_ontology::software_dev_reference_evidence_completeness_policy();
        let packs = agentflow_ontology::software_dev_reference_evidence_fixture_packs();
        let view = project_evidence_kernel_read_model(&policy, &packs);

        assert_eq!(view.version, EVIDENCE_KERNEL_READ_MODEL_VERSION);
        assert_eq!(view.status, "passed");
        assert_eq!(view.completeness.outcome, "complete");
        assert!(!view.authority);
        assert!(view.readonly);
        assert_eq!(view.source_summaries.len(), 6);
        assert!(view
            .source_summaries
            .iter()
            .any(|source| source.source_type == "external-proof"));
        assert!(!view.trace_refs.is_empty());
        assert!(view.missing_reasons.is_empty());
    }

    #[test]
    fn evidence_kernel_read_model_keeps_missing_and_invalid_out_of_passed_state() {
        let fixtures = evidence_kernel_invalid_missing_projection_fixtures();
        let missing = fixtures
            .iter()
            .find(|view| view.status == "deferred")
            .expect("missing evidence fixture should be deferred");
        let invalid = fixtures
            .iter()
            .find(|view| view.status == "invalid")
            .expect("invalid evidence fixture should be invalid");

        assert_ne!(missing.status, "passed");
        assert_ne!(invalid.status, "passed");
        assert_eq!(missing.completeness.outcome, "incomplete");
        assert_eq!(invalid.completeness.outcome, "invalid");
        assert!(missing
            .missing_reasons
            .iter()
            .flat_map(|reason| reason.reasons.iter())
            .any(|reason| reason.starts_with("evidence-required-missing")));
        assert!(invalid
            .missing_reasons
            .iter()
            .flat_map(|reason| reason.reasons.iter())
            .any(|reason| reason.starts_with("evidence-missing-digest")
                || reason.starts_with("evidence-invalid")));
    }

    #[test]
    fn projection_surface_catalog_exposes_evidence_kernel_as_readonly_model() {
        let dir = tempdir().unwrap();
        write_fixture(dir.path());
        rebuild_projections(dir.path()).unwrap();
        let catalog = get_projection_surface_catalog(dir.path()).unwrap();

        let evidence_entry = catalog
            .read_models
            .iter()
            .find(|entry| entry.kind == "evidence-kernel")
            .expect("catalog should include evidence kernel read model");
        assert_eq!(evidence_entry.query.name, "get_evidence_kernel_view");
        assert_eq!(evidence_entry.object_type, "core");
        assert!(!evidence_entry.authority);
        assert!(evidence_entry.missing_facts.is_empty());

        let evidence_view = get_evidence_kernel_view(dir.path()).unwrap();
        assert_eq!(evidence_view.status, "passed");
        assert!(!evidence_view.authority);
    }

    #[test]
    fn pack_industry_workbench_view_exposes_software_and_design_read_models() {
        let dir = tempdir().unwrap();

        let software = get_pack_industry_workbench_view(dir.path(), Some("software-dev")).unwrap();
        let design = get_pack_industry_workbench_view(dir.path(), Some("ui-design")).unwrap();

        assert!(!software.authority);
        assert!(!design.authority);
        assert_eq!(software.active_pack_id.as_deref(), Some("software-dev"));
        assert_eq!(design.active_pack_id.as_deref(), Some("ui-design"));
        assert!(software
            .domain_object_index
            .iter()
            .any(|object| object.object_type_id == "Issue"));
        assert!(software
            .domain_object_index
            .iter()
            .any(|object| object.object_type_id == "Acceptance"));
        assert!(software
            .domain_object_index
            .iter()
            .any(|object| object.object_type_id == "Delivery"));
        assert!(design
            .domain_object_index
            .iter()
            .any(|object| object.object_type_id == "Wireframe"));
        assert!(design
            .domain_object_index
            .iter()
            .any(|object| object.object_type_id == "DesignSystem"));
        assert!(!design
            .domain_object_index
            .iter()
            .any(|object| object.object_type_id == "Issue" || object.object_type_id == "Run"));
        assert!(software
            .surface_page_index
            .iter()
            .any(|page| page.page_id == "task-workbench"));
        assert!(software.view_model_mapping_index.iter().any(|mapping| {
            mapping.page_id == "task-workbench"
                && mapping.projection_ref == "projection.task-workbench"
                && mapping.status == "ready"
        }));
        assert!(software
            .surface_page_index
            .iter()
            .any(|page| page.page_id == "audit-surface" && page.page_kind == "sidecar"));
        assert!(design
            .surface_page_index
            .iter()
            .any(|page| page.page_id == "wireframe-board"));
        assert!(software
            .connector_capability_index
            .iter()
            .any(|capability| capability.connector_id == "github"
                && capability.action_id == "github.pull-request.create"));
        assert!(design
            .connector_capability_index
            .iter()
            .any(|capability| capability.connector_id == "figma"));
        assert!(software
            .pack_readiness
            .iter()
            .all(|readiness| readiness.status == "ready"));
        assert!(design
            .pack_readiness
            .iter()
            .all(|readiness| readiness.status == "ready"));

        let default_view = get_pack_industry_workbench_view(dir.path(), None).unwrap();
        assert_eq!(default_view.pack_list.len(), 2);
        assert_eq!(default_view.pack_readiness.len(), 2);
        assert_eq!(default_view.active_pack_id.as_deref(), Some("software-dev"));
        assert!(default_view
            .domain_object_index
            .iter()
            .all(|object| object.pack_id == "software-dev"));
    }

    #[test]
    fn pack_industry_workbench_uses_custom_pack_definition_files() {
        let dir = tempdir().unwrap();
        write_pack_bundle(
            dir.path(),
            "custom-pack",
            "CustomObject",
            "custom-workbench",
        );

        let view = get_pack_industry_workbench_view(dir.path(), Some("custom-pack")).unwrap();

        assert_eq!(view.active_pack_id.as_deref(), Some("custom-pack"));
        assert_eq!(view.pack_list.len(), 1);
        assert_eq!(view.pack_readiness[0].status, "ready");
        assert!(view
            .domain_object_index
            .iter()
            .any(|object| object.object_type_id == "CustomObject"));
        assert!(!view
            .domain_object_index
            .iter()
            .any(|object| object.object_type_id == "Issue"));
        assert!(view
            .industry_workbenches
            .iter()
            .any(|workbench| workbench.workbench_id == "custom-workbench"));
        assert!(view.view_model_mapping_index.iter().any(|mapping| {
            mapping.pack_id == "custom-pack"
                && mapping.page_id == "custom-page"
                && mapping.projection_ref == "projection.custom"
                && mapping.status == "ready"
        }));
        assert!(view.definition_status_index.iter().all(|status| {
            status.pack_id == "custom-pack"
                && status.status == "ready"
                && status.command_execution_allowed
        }));
        assert!(view.connector_capability_index.iter().all(|capability| {
            capability.pack_id == "custom-pack"
                && capability.status == "ready"
                && capability.command_execution_allowed
        }));
    }

    #[test]
    fn pack_industry_workbench_marks_missing_view_mapping_deferred_without_fallback() {
        let dir = tempdir().unwrap();
        write_pack_bundle_without_view_mapping(
            dir.path(),
            "custom-pack",
            "CustomObject",
            "custom-workbench",
        );

        let view = get_pack_industry_workbench_view(dir.path(), Some("custom-pack")).unwrap();

        assert_eq!(view.active_pack_id.as_deref(), Some("custom-pack"));
        assert!(view
            .domain_object_index
            .iter()
            .any(|object| object.object_type_id == "CustomObject"));
        assert!(!view
            .domain_object_index
            .iter()
            .any(|object| object.object_type_id == "Issue"));
        assert!(view.view_model_mapping_index.iter().any(|mapping| {
            mapping.pack_id == "custom-pack"
                && mapping.page_id == "custom-page"
                && mapping.status == "deferred"
                && mapping.reason == "pack-surface-view-model-mapping-missing"
        }));
        assert!(view
            .pack_readiness
            .iter()
            .any(|readiness| readiness.pack_id == "custom-pack" && readiness.status == "invalid"));
    }

    #[test]
    fn pack_industry_workbench_keeps_invalid_definitions_out_of_command_paths() {
        let dir = tempdir().unwrap();
        write_pack_manifest_only(dir.path(), "custom-pack");

        let view = get_pack_industry_workbench_view(dir.path(), Some("custom-pack")).unwrap();

        for kind in ["domain", "surface", "connector"] {
            let status = view
                .definition_status_index
                .iter()
                .find(|status| status.pack_id == "custom-pack" && status.definition_kind == kind)
                .unwrap();
            assert_eq!(status.status, "invalid");
            assert!(!status.command_execution_allowed);
        }
        assert!(view.connector_capability_index.is_empty());
    }

    #[test]
    fn pack_industry_workbench_projects_disabled_provider_as_deferred() {
        let dir = tempdir().unwrap();
        write_pack_bundle_with_disabled_connector(
            dir.path(),
            "custom-pack",
            "CustomObject",
            "custom-workbench",
            "provider-disabled-by-capability-registry",
        );

        let view = get_pack_industry_workbench_view(dir.path(), Some("custom-pack")).unwrap();

        let capability = view
            .connector_capability_index
            .iter()
            .find(|capability| capability.pack_id == "custom-pack")
            .unwrap();
        assert_eq!(capability.status, "deferred");
        assert_eq!(
            capability.disabled_reason,
            "provider-disabled-by-capability-registry"
        );
        assert!(!capability.command_execution_allowed);
    }

    #[test]
    fn pack_industry_workbench_projects_stale_app_definition_as_non_executable() {
        let dir = tempdir().unwrap();
        write_pack_bundle(
            dir.path(),
            "custom-pack",
            "CustomObject",
            "custom-workbench",
        );
        let manifest_path = dir.path().join(".agentflow/packs/custom-pack/pack.json");
        let mut manifest = serde_json::from_str::<agentflow_pack::PackManifest>(
            &fs::read_to_string(&manifest_path).unwrap(),
        )
        .unwrap();
        manifest.validation_status = agentflow_pack::PackValidationStatus::Draft;
        write_json(&manifest_path, &manifest);

        let view = get_pack_industry_workbench_view(dir.path(), Some("custom-pack")).unwrap();

        let app_status = view
            .definition_status_index
            .iter()
            .find(|status| status.pack_id == "custom-pack" && status.definition_kind == "app")
            .unwrap();
        assert_eq!(app_status.status, "stale");
        assert!(!app_status.command_execution_allowed);
        assert!(view
            .definition_status_index
            .iter()
            .filter(|status| status.pack_id == "custom-pack")
            .any(|status| status.reason == "pack-app-definition-validation-status-draft"));
    }

    #[test]
    fn pack_industry_workbench_marks_missing_custom_definition_invalid_without_fallback() {
        let dir = tempdir().unwrap();
        write_pack_manifest_only(dir.path(), "custom-pack");

        let view = get_pack_industry_workbench_view(dir.path(), Some("custom-pack")).unwrap();

        assert_eq!(view.active_pack_id.as_deref(), Some("custom-pack"));
        assert_eq!(view.pack_list[0].validation_status, "invalid");
        assert!(view.domain_object_index.is_empty());
        let readiness = view
            .pack_readiness
            .iter()
            .find(|readiness| readiness.pack_id == "custom-pack")
            .unwrap();
        assert!(!readiness.domain_valid);
        assert!(!readiness.surface_valid);
        assert!(!readiness.connector_valid);
        assert!(readiness
            .warnings
            .iter()
            .any(|warning| warning.contains("pack-surface-unreadable")));
    }

    fn write_pack_manifest_only(root: &Path, pack_id: &str) {
        let pack_dir = root.join(".agentflow/packs").join(pack_id);
        fs::create_dir_all(&pack_dir).unwrap();
        let manifest = agentflow_pack::PackManifest {
            version: agentflow_pack::PACK_MANIFEST_VERSION.to_string(),
            pack_id: pack_id.to_string(),
            name: format!("{pack_id} test pack"),
            pack_type: agentflow_pack::PackType::Custom,
            pack_version: "0.8.1".to_string(),
            runtime_compatibility: ">=0.8.0".to_string(),
            domain_path: "domain/".to_string(),
            surface_path: "surface/".to_string(),
            connector_path: "connectors/".to_string(),
            required_capabilities: vec![format!("{pack_id}.capability")],
            owned_object_types: vec!["CustomObject".to_string()],
            exposed_commands: vec!["custom.start".to_string()],
            projection_entries: vec!["projection.custom".to_string()],
            migration_policy: agentflow_pack::PackMigrationPolicy::PreviewOnly,
            validation_status: agentflow_pack::PackValidationStatus::Valid,
        };
        write_json(pack_dir.join("pack.json"), &manifest);
    }

    fn write_pack_bundle(root: &Path, pack_id: &str, object_type: &str, workbench_id: &str) {
        write_pack_bundle_with_options(root, pack_id, object_type, workbench_id, true, None);
    }

    fn write_pack_bundle_without_view_mapping(
        root: &Path,
        pack_id: &str,
        object_type: &str,
        workbench_id: &str,
    ) {
        write_pack_bundle_with_options(root, pack_id, object_type, workbench_id, false, None);
    }

    fn write_pack_bundle_with_disabled_connector(
        root: &Path,
        pack_id: &str,
        object_type: &str,
        workbench_id: &str,
        disabled_reason: &str,
    ) {
        write_pack_bundle_with_options(
            root,
            pack_id,
            object_type,
            workbench_id,
            true,
            Some(disabled_reason),
        );
    }

    fn write_pack_bundle_with_options(
        root: &Path,
        pack_id: &str,
        object_type: &str,
        workbench_id: &str,
        include_view_mapping: bool,
        disabled_connector_reason: Option<&str>,
    ) {
        write_pack_manifest_only(root, pack_id);
        let pack_dir = root.join(".agentflow/packs").join(pack_id);
        fs::create_dir_all(pack_dir.join("domain")).unwrap();
        fs::create_dir_all(pack_dir.join("surface")).unwrap();
        fs::create_dir_all(pack_dir.join("connectors")).unwrap();

        let domain = agentflow_pack::PackDomainDefinition {
            version: agentflow_pack::PACK_DOMAIN_VERSION.to_string(),
            pack_id: pack_id.to_string(),
            domain_id: format!("{pack_id}-domain"),
            object_types: vec![agentflow_pack::DomainObjectType {
                object_type_id: object_type.to_string(),
                label: "Custom Object".to_string(),
                description: "Custom pack object from file-backed definition.".to_string(),
            }],
            link_types: Vec::new(),
            state_machines: Vec::new(),
            action_semantics: vec![agentflow_pack::DomainActionSemantic {
                action_type: "startRun".to_string(),
                target_object_type: object_type.to_string(),
                description: "Start custom object work.".to_string(),
                allowed_roles: vec!["work-agent".to_string()],
                contract_ref: "action-contract:issue.start".to_string(),
                arbitration_ref: "runtime-command-surface".to_string(),
                simulation_ref: "simulation.custom.start".to_string(),
                required_evidence: Vec::new(),
            }],
            acceptance_semantics: Vec::new(),
            evidence_policy: agentflow_pack::DomainEvidencePolicy {
                policy_id: "custom.evidence".to_string(),
                required_evidence_kinds: Vec::new(),
                missing_evidence_behavior: "warn".to_string(),
            },
            audit_trigger_hints: Vec::new(),
            migration_compatibility: agentflow_pack::DomainMigrationCompatibility {
                compatible_with_runtime: ">=0.8.0".to_string(),
                migration_policy_ref: "pack.migration.preview-only".to_string(),
            },
            writes_events: false,
        };
        let surface = agentflow_pack::PackSurfaceDefinition {
            version: agentflow_pack::PACK_SURFACE_VERSION.to_string(),
            pack_id: pack_id.to_string(),
            surface_id: format!("{pack_id}-surface"),
            pages: vec![agentflow_pack::SurfacePage {
                page_id: "custom-page".to_string(),
                label: "Custom Page".to_string(),
                description: "Custom pack page.".to_string(),
                kind: agentflow_pack::SurfacePageKind::Workbench,
                view_model_ref: "view-model:custom-page".to_string(),
                command_entry_ids: vec!["custom.start".to_string()],
            }],
            workbenches: vec![agentflow_pack::SurfaceWorkbench {
                workbench_id: workbench_id.to_string(),
                page_id: "custom-page".to_string(),
                label: "Custom Workbench".to_string(),
                primary_object_type: object_type.to_string(),
                timeline_ref: "custom.timeline".to_string(),
            }],
            view_model_mappings: if include_view_mapping {
                vec![agentflow_pack::SurfaceViewModelMapping {
                    mapping_id: "custom-page-view".to_string(),
                    page_id: "custom-page".to_string(),
                    projection_ref: "projection.custom".to_string(),
                    view_model_ref: "view-model:custom-page".to_string(),
                }]
            } else {
                Vec::new()
            },
            command_entry_mappings: vec![agentflow_pack::SurfaceCommandEntryMapping {
                command_entry_id: "custom.start".to_string(),
                page_id: "custom-page".to_string(),
                label: "Start Custom".to_string(),
                command_type: "custom.start".to_string(),
                route: agentflow_pack::SurfaceCommandRoute::RuntimeCommand,
                action_contract_ref: "action-contract:issue.start".to_string(),
            }],
            read_model_dependencies: vec![agentflow_pack::SurfaceReadModelDependency {
                dependency_id: "custom-read-model".to_string(),
                page_id: "custom-page".to_string(),
                projection_ref: "projection.custom".to_string(),
                required: true,
            }],
            navigation_rules: Vec::new(),
            state_policy: agentflow_pack::SurfaceStatePolicy {
                empty_state_ref: "custom.empty".to_string(),
                loading_state_ref: "custom.loading".to_string(),
                error_state_ref: "custom.error".to_string(),
            },
            sidecar_surfaces: Vec::new(),
            writes_authority: false,
        };
        let connector = agentflow_pack::PackConnectorDefinition {
            version: agentflow_pack::PACK_CONNECTOR_VERSION.to_string(),
            pack_id: pack_id.to_string(),
            connector_id: format!("{pack_id}-connectors"),
            connectors: vec![agentflow_pack::PackConnector {
                connector_id: "custom-provider".to_string(),
                provider_type: agentflow_pack::PackConnectorProviderType::Custom,
                supported_actions: vec![agentflow_pack::ConnectorSupportedAction {
                    action_id: "custom.start".to_string(),
                    label: "Start Custom".to_string(),
                    required_capability: format!("{pack_id}.capability"),
                    command_type: "custom.start".to_string(),
                    writes_external: true,
                    evidence_output: "custom-evidence".to_string(),
                }],
                required_capabilities: vec![format!("{pack_id}.capability")],
                health_source: agentflow_pack::ConnectorHealthSource::CapabilityRegistry,
                smoke_policy: agentflow_pack::ConnectorSmokePolicy {
                    required_for_commands: false,
                    provider_smoke_ref: "custom.smoke".to_string(),
                    failure_disables_commands: true,
                },
                evidence_output: agentflow_pack::ConnectorEvidenceOutput {
                    channel: "custom".to_string(),
                    path_policy: "task-evidence".to_string(),
                    authority: false,
                },
                disabled_reason: disabled_connector_reason.unwrap_or_default().to_string(),
                command_boundary: agentflow_pack::ConnectorCommandBoundary {
                    runtime_command_required: true,
                    authority_write: false,
                    output_authority: false,
                    output_channels: vec!["custom".to_string()],
                },
            }],
            writes_authority: false,
        };

        write_json(pack_dir.join("domain/definition.json"), &domain);
        write_json(pack_dir.join("surface/definition.json"), &surface);
        write_json(pack_dir.join("connectors/definition.json"), &connector);
    }

    fn write_json(path: impl AsRef<Path>, value: &impl serde::Serialize) {
        fs::write(path, serde_json::to_string_pretty(value).unwrap()).unwrap();
    }
}
