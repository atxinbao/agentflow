use serde::{Deserialize, Serialize};

pub const TASK_PROJECTION_VERSION: &str = "task-projection.v2";
pub const PROJECT_PROJECTION_VERSION: &str = "project-projection.v3";
pub const ISSUE_STATUS_INDEX_VERSION: &str = "issue-status-index.v3";
pub const REQUIREMENT_PREVIEW_PROJECTION_VERSION: &str = "requirement-preview-projection.v1";
pub const REQUIREMENT_PREVIEW_INDEX_VERSION: &str = "requirement-preview-index.v1";
pub const SPEC_LOOP_PROJECTION_VERSION: &str = "spec-loop-projection.v1";
pub const COMPLETION_DECISION_PROJECTION_VERSION: &str = "completion-decision-projection.v1";
pub const COMPLETION_DECISION_INDEX_VERSION: &str = "completion-decision-index.v1";
pub const PROJECTION_KERNEL_CONTRACT_VERSION: &str = "projection-kernel-contract.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectionKernelSourceRef {
    pub ref_kind: String,
    pub path_pattern: String,
    pub authority: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectionKernelForbiddenAuthorityWrite {
    pub target: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectionKernelLifecycleSemantics {
    pub state: String,
    pub meaning: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectionKernelNegativeFixture {
    pub fixture_id: String,
    pub rejected_ref_kind: String,
    pub rejected_target: String,
    pub expected_result: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreReadModelSchema {
    pub model_kind: String,
    pub read_model_version: String,
    pub object_type: String,
    pub identity_fields: Vec<String>,
    pub required_fields: Vec<String>,
    pub source_ref_kinds: Vec<String>,
    pub freshness_states: Vec<String>,
    pub status_values: Vec<String>,
    pub reason_link_fields: Vec<String>,
    pub evidence_link_fields: Vec<String>,
    pub authority_boundary_fields: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreReadModelSchemaNegativeFixture {
    pub fixture_id: String,
    pub model_kind: String,
    pub invalid_combination: Vec<String>,
    pub expected_result: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectionKernelContract {
    pub version: String,
    pub status: String,
    pub writes_authority: bool,
    pub accepted_source_refs: Vec<ProjectionKernelSourceRef>,
    pub forbidden_authority_writes: Vec<ProjectionKernelForbiddenAuthorityWrite>,
    pub required_fields: Vec<String>,
    pub lifecycle_semantics: Vec<ProjectionKernelLifecycleSemantics>,
    pub negative_fixtures: Vec<ProjectionKernelNegativeFixture>,
    pub core_read_model_schemas: Vec<CoreReadModelSchema>,
    pub read_model_negative_fixtures: Vec<CoreReadModelSchemaNegativeFixture>,
}

pub fn projection_kernel_contract() -> ProjectionKernelContract {
    ProjectionKernelContract {
        version: PROJECTION_KERNEL_CONTRACT_VERSION.to_string(),
        status: "active".to_string(),
        writes_authority: false,
        accepted_source_refs: vec![
            ProjectionKernelSourceRef {
                ref_kind: "spec-authority".to_string(),
                path_pattern: ".agentflow/spec/**".to_string(),
                authority: "Spec Kernel".to_string(),
            },
            ProjectionKernelSourceRef {
                ref_kind: "event-authority".to_string(),
                path_pattern: ".agentflow/events/**".to_string(),
                authority: "Event Store".to_string(),
            },
            ProjectionKernelSourceRef {
                ref_kind: "task-evidence-authority".to_string(),
                path_pattern: ".agentflow/tasks/<issue-id>/evidence/**".to_string(),
                authority: "Evidence Kernel".to_string(),
            },
            ProjectionKernelSourceRef {
                ref_kind: "decision-authority".to_string(),
                path_pattern: ".agentflow/runtime/decisions/**".to_string(),
                authority: "Decision Kernel".to_string(),
            },
            ProjectionKernelSourceRef {
                ref_kind: "delivery-authority".to_string(),
                path_pattern: ".agentflow/release/**".to_string(),
                authority: "Delivery Kernel".to_string(),
            },
        ],
        forbidden_authority_writes: vec![
            "Spec",
            "Runtime",
            "Evidence",
            "Decision",
            "Completion",
            "Delivery",
            "Audit",
        ]
        .into_iter()
        .map(|target| ProjectionKernelForbiddenAuthorityWrite {
            target: target.to_string(),
            reason: "Projection is a derived read surface and cannot write authority facts."
                .to_string(),
        })
        .collect(),
        required_fields: vec![
            "version".to_string(),
            "status".to_string(),
            "sourceRefs".to_string(),
            "readModelVersion".to_string(),
            "viewModelVersion".to_string(),
            "freshness".to_string(),
            "rebuiltAt".to_string(),
        ],
        lifecycle_semantics: vec![
            ProjectionKernelLifecycleSemantics {
                state: "stale".to_string(),
                meaning: "Source facts changed after the read model was built.".to_string(),
            },
            ProjectionKernelLifecycleSemantics {
                state: "invalid".to_string(),
                meaning: "Required source facts are missing or inconsistent.".to_string(),
            },
            ProjectionKernelLifecycleSemantics {
                state: "deferred".to_string(),
                meaning: "A pack-specific projection is intentionally unavailable.".to_string(),
            },
            ProjectionKernelLifecycleSemantics {
                state: "fresh".to_string(),
                meaning: "The read model was rebuilt from the current accepted sources."
                    .to_string(),
            },
        ],
        negative_fixtures: vec![
            ProjectionKernelNegativeFixture {
                fixture_id: "projection-ref-as-authority".to_string(),
                rejected_ref_kind: "ProjectionRef".to_string(),
                rejected_target: "Decision".to_string(),
                expected_result: "rejected".to_string(),
            },
            ProjectionKernelNegativeFixture {
                fixture_id: "provider-session-as-authority".to_string(),
                rejected_ref_kind: "ProviderSessionRef".to_string(),
                rejected_target: "Completion".to_string(),
                expected_result: "rejected".to_string(),
            },
            ProjectionKernelNegativeFixture {
                fixture_id: "github-issue-as-authority".to_string(),
                rejected_ref_kind: "GitHubIssueRef".to_string(),
                rejected_target: "Spec".to_string(),
                expected_result: "rejected".to_string(),
            },
        ],
        core_read_model_schemas: projection_kernel_core_read_model_schemas(),
        read_model_negative_fixtures: projection_kernel_read_model_negative_fixtures(),
    }
}

pub fn projection_kernel_rejects_authority_write(target: &str) -> bool {
    projection_kernel_contract()
        .forbidden_authority_writes
        .iter()
        .any(|write| write.target == target)
}

pub fn projection_kernel_core_read_model_schemas() -> Vec<CoreReadModelSchema> {
    let stable_required_fields = [
        "objectId",
        "objectType",
        "readModelVersion",
        "sourceRefs",
        "freshness",
        "status",
        "reasonLinks",
        "evidenceLinks",
        "authorityBoundary",
        "updatedAt",
    ];
    let freshness_states = ["fresh", "stale", "invalid", "deferred"];
    let authority_boundary_fields = [
        "writesAuthority",
        "projectionAuthority",
        "sourceAuthority",
        "readOnly",
    ];

    [
        (
            "spec",
            "core-spec-read-model.v1",
            "SpecObject",
            vec!["spec-authority", "event-authority"],
            vec!["draft", "approved", "invalid", "deferred"],
        ),
        (
            "evidence",
            "core-evidence-read-model.v1",
            "EvidenceObject",
            vec!["task-evidence-authority", "event-authority"],
            vec!["missing", "partial", "complete", "invalid"],
        ),
        (
            "decision",
            "core-decision-read-model.v1",
            "DecisionObject",
            vec![
                "decision-authority",
                "spec-authority",
                "task-evidence-authority",
                "event-authority",
            ],
            vec!["pending", "accepted", "rejected", "deferred", "blocked"],
        ),
        (
            "delivery",
            "core-delivery-read-model.v1",
            "DeliveryObject",
            vec!["delivery-authority", "event-authority"],
            vec!["draft", "ready", "published", "invalid"],
        ),
    ]
    .into_iter()
    .map(
        |(model_kind, read_model_version, object_type, source_ref_kinds, status_values)| {
            CoreReadModelSchema {
                model_kind: model_kind.to_string(),
                read_model_version: read_model_version.to_string(),
                object_type: object_type.to_string(),
                identity_fields: vec!["objectId".to_string(), "objectType".to_string()],
                required_fields: stable_required_fields
                    .iter()
                    .map(|field| (*field).to_string())
                    .collect(),
                source_ref_kinds: source_ref_kinds.into_iter().map(str::to_string).collect(),
                freshness_states: freshness_states
                    .iter()
                    .map(|state| (*state).to_string())
                    .collect(),
                status_values: status_values.into_iter().map(str::to_string).collect(),
                reason_link_fields: vec![
                    "reasonLinks".to_string(),
                    "invalidReasons".to_string(),
                    "deferredReasons".to_string(),
                ],
                evidence_link_fields: vec![
                    "evidenceLinks".to_string(),
                    "sourceRefs".to_string(),
                    "eventRefs".to_string(),
                ],
                authority_boundary_fields: authority_boundary_fields
                    .iter()
                    .map(|field| (*field).to_string())
                    .collect(),
            }
        },
    )
    .collect()
}

pub fn projection_kernel_read_model_negative_fixtures() -> Vec<CoreReadModelSchemaNegativeFixture> {
    vec![
        CoreReadModelSchemaNegativeFixture {
            fixture_id: "spec-read-model-missing-spec-source-ref".to_string(),
            model_kind: "spec".to_string(),
            invalid_combination: vec!["missing:spec-authority".to_string()],
            expected_result: "rejected".to_string(),
            reason: "Spec read model must be sourced from Spec authority.".to_string(),
        },
        CoreReadModelSchemaNegativeFixture {
            fixture_id: "evidence-read-model-missing-evidence-ref".to_string(),
            model_kind: "evidence".to_string(),
            invalid_combination: vec!["missing:task-evidence-authority".to_string()],
            expected_result: "rejected".to_string(),
            reason: "Evidence read model must retain task evidence refs.".to_string(),
        },
        CoreReadModelSchemaNegativeFixture {
            fixture_id: "decision-read-model-missing-evidence-ref".to_string(),
            model_kind: "decision".to_string(),
            invalid_combination: vec![
                "missing:decision-authority".to_string(),
                "missing:task-evidence-authority".to_string(),
            ],
            expected_result: "rejected".to_string(),
            reason: "Decision read model must bind decision and evidence refs.".to_string(),
        },
        CoreReadModelSchemaNegativeFixture {
            fixture_id: "delivery-read-model-missing-public-record-ref".to_string(),
            model_kind: "delivery".to_string(),
            invalid_combination: vec!["missing:delivery-authority".to_string()],
            expected_result: "rejected".to_string(),
            reason: "Delivery read model must keep public delivery refs.".to_string(),
        },
    ]
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProjectionPhase {
    Past,
    Current,
    Future,
    Exception,
}

impl ProjectionPhase {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Past => "past",
            Self::Current => "current",
            Self::Future => "future",
            Self::Exception => "exception",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskTimelineEvent {
    pub event_id: String,
    pub event_type: String,
    pub timestamp: u64,
    pub actor_role: String,
    pub actor_kind: String,
    pub summary: String,
    pub artifact_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskTimelineItem {
    pub state: String,
    pub phase: ProjectionPhase,
    pub entered_at: Option<u64>,
    pub events: Vec<TaskTimelineEvent>,
    pub summary: String,
    pub live_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectionPublicDelivery {
    pub evidence_path: Option<String>,
    pub pr_url: Option<String>,
    pub merge_commit: Option<String>,
    pub changelog_path: Option<String>,
    pub release_notes_url: Option<String>,
}

fn default_projection_runtime_status() -> String {
    "missing".to_string()
}

fn default_projection_delivery_status() -> String {
    "missing".to_string()
}

fn default_projection_audit_status() -> String {
    "not-requested".to_string()
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectionRuntimeSummary {
    pub run_id: Option<String>,
    #[serde(default = "default_projection_runtime_status")]
    pub run_status: String,
    pub branch_name: Option<String>,
    #[serde(default)]
    pub checkpoint_count: usize,
    pub latest_checkpoint_id: Option<String>,
    pub latest_checkpoint_state: Option<String>,
    pub latest_checkpoint_summary: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectionSessionSummary {
    pub provider: Option<String>,
    pub provider_kind: Option<String>,
    pub provider_status: Option<String>,
    pub owner_id: Option<String>,
    pub session_id: Option<String>,
    pub status: Option<String>,
    #[serde(default)]
    pub attempt_count: u32,
    pub working_directory: Option<String>,
    pub workspace_root: Option<String>,
    pub worktree_root: Option<String>,
    pub runtime_root: Option<String>,
    pub temp_root: Option<String>,
    pub cache_root: Option<String>,
    pub evidence_root: Option<String>,
    pub launch_requested_at: Option<u64>,
    pub claimed_at: Option<u64>,
    pub created_at: Option<u64>,
    pub updated_at: Option<u64>,
    pub launch_request_path: Option<String>,
    pub plan_path: Option<String>,
    pub log_path: Option<String>,
    pub last_message_path: Option<String>,
    pub exit_proof_path: Option<String>,
    pub merge_proof_path: Option<String>,
    pub merge_state: Option<String>,
    pub writeback_state: Option<String>,
    pub selection_status: Option<String>,
    pub selection_reason: Option<String>,
    pub degradation_reason: Option<String>,
    #[serde(default)]
    pub supported_roles: Vec<String>,
    #[serde(default)]
    pub supported_skill_packs: Vec<String>,
    #[serde(default)]
    pub required_capabilities: Vec<String>,
    #[serde(default)]
    pub degraded_capabilities: Vec<String>,
    #[serde(default)]
    pub missing_required_capabilities: Vec<String>,
    #[serde(default)]
    pub missing_degraded_capabilities: Vec<String>,
    pub recovery_reason: Option<String>,
    pub last_error: Option<String>,
    pub branch_name: Option<String>,
    pub process_group_id: Option<u32>,
    pub started_at: Option<u64>,
    pub last_heartbeat_at: Option<u64>,
    pub permission_mode: Option<String>,
    pub approval_policy: Option<String>,
    pub sandbox_mode: Option<String>,
    pub supervision_mode: Option<String>,
    pub governance_policy_version: Option<String>,
    pub claim_policy: Option<String>,
    pub timeout_policy: Option<String>,
    pub timeout_seconds: Option<u64>,
    pub timeout_at: Option<u64>,
    pub timed_out_at: Option<u64>,
    pub takeover_policy: Option<String>,
    pub retry_policy: Option<String>,
    pub max_attempts: Option<u32>,
    pub cancel_policy: Option<String>,
    pub cancel_requested_at: Option<u64>,
    pub cancelled_at: Option<u64>,
    pub resumed_from_attempt: Option<u32>,
    pub takeover_session_id: Option<String>,
    pub terminal_reason: Option<String>,
    pub retryable: Option<bool>,
    pub exited_at: Option<u64>,
    pub exit_code: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectionDeliverySummary {
    #[serde(default = "default_projection_delivery_status")]
    pub status: String,
    #[serde(default = "default_projection_delivery_status")]
    pub evidence_status: String,
    pub evidence_path: Option<String>,
    pub pr_url: Option<String>,
    pub merge_commit: Option<String>,
    pub public_record_path: Option<String>,
    #[serde(default)]
    pub public_record_targets: Vec<String>,
    #[serde(default)]
    pub public_record_markdown: String,
    #[serde(default)]
    pub summary_line: String,
    #[serde(default)]
    pub public_record_items: Vec<String>,
    #[serde(default)]
    pub missing_public_records: Vec<String>,
    #[serde(default)]
    pub current_issue_id: Option<String>,
    #[serde(default)]
    pub published_count: usize,
    #[serde(default)]
    pub ready_count: usize,
    #[serde(default)]
    pub missing_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectionAuditSummary {
    #[serde(default = "default_projection_audit_status")]
    pub status: String,
    pub latest_audit_id: Option<String>,
    #[serde(default)]
    pub source_issue_id: Option<String>,
    pub report_path: Option<String>,
    pub requested_at: Option<u64>,
    #[serde(default)]
    pub summary_line: String,
    #[serde(default)]
    pub findings_count: usize,
    #[serde(default)]
    pub findings: Vec<String>,
    #[serde(default)]
    pub evidence_gaps: Vec<String>,
    #[serde(default)]
    pub repair_recommendations: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectionAcceptanceSubGateSummary {
    pub gate: String,
    pub passed: bool,
    #[serde(default)]
    pub failure_reasons: Vec<String>,
    pub repair_suggestion: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectionAcceptanceTraceabilitySummary {
    pub issue_id: String,
    pub run_id: String,
    pub acceptance_decision_path: String,
    pub evidence_path: String,
    pub validation_path: String,
    pub closeout_proof_path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pr_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub merge_commit_sha: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectionAcceptanceSummary {
    pub outcome: String,
    pub passed: bool,
    pub summary: String,
    #[serde(default)]
    pub failure_reasons: Vec<String>,
    #[serde(default)]
    pub next_steps: Vec<String>,
    #[serde(default)]
    pub sub_gates: Vec<ProjectionAcceptanceSubGateSummary>,
    pub traceability: ProjectionAcceptanceTraceabilitySummary,
    pub checked_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectBrainProjection {
    pub project_path: String,
    pub goal_path: String,
    pub plan_path: String,
    pub decisions_path: String,
    pub health_path: String,
    pub brain_status: String,
    pub goal_status: String,
    pub plan_status: String,
    pub decision_status: String,
    pub health_status: String,
    pub missing_documents: Vec<String>,
    pub open_questions: Vec<String>,
    pub next_recommended_action: String,
    pub next_recommended_action_label: String,
    pub next_recommended_action_reason: String,
    pub readonly: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectCompletionProjection {
    pub current_state: String,
    pub latest_outcome: Option<String>,
    pub next_recommended_action: String,
    pub next_recommended_action_label: String,
    pub next_recommended_action_reason: String,
    pub total_issue_count: usize,
    pub completed_issue_count: usize,
    pub canceled_issue_count: usize,
    pub remaining_issue_count: usize,
    pub blocked_issue_count: usize,
    #[serde(default)]
    pub task_evidence_ready_count: usize,
    #[serde(default)]
    pub task_evidence_missing_count: usize,
    #[serde(default = "default_projection_delivery_status")]
    pub delivery_status: String,
    #[serde(default)]
    pub delivery_missing_count: usize,
    #[serde(default)]
    pub audit_required: bool,
    #[serde(default = "default_projection_audit_status")]
    pub audit_status: String,
    #[serde(default)]
    pub audit_blocking_findings: usize,
    #[serde(default = "default_projection_completion_goal_recheck_status")]
    pub goal_recheck_status: String,
    #[serde(default = "default_projection_completion_project_health_status")]
    pub project_health_status: String,
    #[serde(default = "default_projection_completion_release_readiness")]
    pub release_readiness: String,
    pub open_questions: Vec<String>,
    pub rationale: Vec<String>,
    pub updated_at: u64,
}

fn default_projection_completion_goal_recheck_status() -> String {
    "not-ready".to_string()
}

fn default_projection_completion_project_health_status() -> String {
    "missing".to_string()
}

fn default_projection_completion_release_readiness() -> String {
    "blocked".to_string()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectReleaseProjection {
    pub current_state: String,
    pub publication_stage: String,
    pub gate_status: String,
    pub gate_reason: String,
    pub completion_state: String,
    pub completion_outcome: Option<String>,
    pub delivery_status: String,
    pub public_record_written_at: Option<u64>,
    pub changelog_path: String,
    pub release_notes_path: String,
    pub entry_count: usize,
    pub summary_line: String,
    pub tag_name: Option<String>,
    pub tag_commit_sha: Option<String>,
    pub tag_proof_path: Option<String>,
    pub remote_provider: Option<String>,
    pub remote_release_id: Option<String>,
    pub remote_release_url: Option<String>,
    pub remote_release_commit_sha: Option<String>,
    pub remote_release_proof_path: Option<String>,
    pub artifact_manifest_path: Option<String>,
    pub artifact_manifest_sha256: Option<String>,
    pub published_at: Option<u64>,
    pub updated_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectExternalReviewProjection {
    pub review_status: String,
    pub handoff_path: String,
    pub total_entries: usize,
    pub summary_line: String,
    pub latest_audit_status: Option<String>,
    pub findings_count: usize,
    #[serde(default)]
    pub risk_items: Vec<String>,
    pub generated_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskProjection {
    pub version: String,
    pub issue_id: String,
    pub project_id: Option<String>,
    pub workflow_ref: String,
    pub current_state: String,
    pub display_status: String,
    pub current_transition: Option<String>,
    pub latest_run_id: Option<String>,
    pub branch_name: Option<String>,
    pub timeline: Vec<TaskTimelineItem>,
    pub public_delivery: ProjectionPublicDelivery,
    #[serde(default)]
    pub runtime: ProjectionRuntimeSummary,
    #[serde(default)]
    pub session: ProjectionSessionSummary,
    #[serde(default)]
    pub delivery: ProjectionDeliverySummary,
    #[serde(default)]
    pub audit: ProjectionAuditSummary,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub acceptance: Option<ProjectionAcceptanceSummary>,
    pub updated_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectIssueLanes {
    pub current: Vec<String>,
    pub past: Vec<String>,
    pub future: Vec<String>,
    pub blocked: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectBlockerSummary {
    pub issue_id: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectProjection {
    pub version: String,
    pub project_id: String,
    pub title: String,
    pub objective: String,
    pub status: String,
    #[serde(default)]
    pub stage_key: String,
    #[serde(default)]
    pub stage_label: String,
    #[serde(default)]
    pub stage_summary: String,
    pub issue_ids: Vec<String>,
    pub current_issue_id: Option<String>,
    #[serde(default)]
    pub lanes: ProjectIssueLanes,
    #[serde(default)]
    pub next_action: String,
    #[serde(default)]
    pub next_action_label: String,
    #[serde(default)]
    pub next_action_reason: String,
    #[serde(default)]
    pub blockers: Vec<ProjectBlockerSummary>,
    #[serde(default)]
    pub completion_hint: String,
    #[serde(default)]
    pub completion: Option<ProjectCompletionProjection>,
    #[serde(default)]
    pub release: Option<ProjectReleaseProjection>,
    #[serde(default)]
    pub external_review: Option<ProjectExternalReviewProjection>,
    #[serde(default)]
    pub delivery: Option<ProjectionDeliverySummary>,
    #[serde(default)]
    pub audit: Option<ProjectionAuditSummary>,
    pub issue_count: usize,
    pub completed_issue_count: usize,
    pub project_brain: ProjectBrainProjection,
    pub updated_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueStatusIndexEntry {
    pub issue_id: String,
    pub project_id: Option<String>,
    pub title: String,
    pub current_state: String,
    pub display_status: String,
    pub workflow_ref: String,
    pub projection_path: String,
    pub updated_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueStatusIndex {
    pub version: String,
    pub updated_at: u64,
    pub issues: Vec<IssueStatusIndexEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectionSummary {
    pub task_count: usize,
    pub project_count: usize,
    pub index_path: String,
}

pub const PROJECTION_REPLAY_REPORT_VERSION: &str = "projection-replay-report.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProjectionReplayStatus {
    Passed,
    Failed,
}

impl ProjectionReplayStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Passed => "passed",
            Self::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectionReplayFailure {
    pub stage: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectionReplayReport {
    pub version: String,
    pub status: ProjectionReplayStatus,
    pub source_refs: Vec<String>,
    pub event_count: usize,
    pub task_count: usize,
    pub project_count: usize,
    pub rebuilt_paths: Vec<String>,
    pub input_digest: Option<String>,
    pub output_digest: Option<String>,
    pub receipt_id: Option<String>,
    pub deterministic: bool,
    pub failures: Vec<ProjectionReplayFailure>,
    pub writes_authority: bool,
    pub projection_authority: bool,
    pub generated_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequirementPreviewProjection {
    pub version: String,
    pub requirement_id: String,
    pub requirement_path: String,
    pub project_id: String,
    pub project_title: String,
    pub lifecycle: String,
    pub current_state: String,
    pub goal_status: String,
    pub plan_status: Option<String>,
    pub next_recommended_action: String,
    pub next_recommended_action_label: String,
    pub next_recommended_action_reason: String,
    pub issue_contract_draft_count: usize,
    pub materialized_project_id: Option<String>,
    pub materialized_issue_ids: Vec<String>,
    pub updated_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpecLoopStageProjection {
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
pub struct SpecLoopAuthorityLayerProjection {
    pub authority_layer: String,
    pub path: String,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpecLoopTraceabilityEdge {
    pub from_ref: String,
    pub to_ref: String,
    pub relation: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpecLoopActionProposalProjection {
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
pub struct SpecLoopProjection {
    pub version: String,
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
    pub stages: Vec<SpecLoopStageProjection>,
    #[serde(default)]
    pub authority_layers: Vec<SpecLoopAuthorityLayerProjection>,
    #[serde(default)]
    pub traceability: Vec<SpecLoopTraceabilityEdge>,
    #[serde(default)]
    pub runtime_action_proposals: Vec<SpecLoopActionProposalProjection>,
    pub updated_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequirementPreviewIndexEntry {
    pub requirement_id: String,
    pub project_id: String,
    pub current_state: String,
    pub lifecycle: String,
    pub next_recommended_action: String,
    pub projection_path: String,
    pub updated_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequirementPreviewIndex {
    pub version: String,
    pub updated_at: u64,
    pub previews: Vec<RequirementPreviewIndexEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompletionDecisionProjection {
    pub version: String,
    pub project_id: String,
    pub project_title: String,
    pub current_state: String,
    pub latest_outcome: Option<String>,
    pub next_recommended_action: String,
    pub next_recommended_action_label: String,
    pub next_recommended_action_reason: String,
    pub total_issue_count: usize,
    pub completed_issue_count: usize,
    pub canceled_issue_count: usize,
    pub remaining_issue_count: usize,
    pub blocked_issue_count: usize,
    #[serde(default)]
    pub task_evidence_ready_count: usize,
    #[serde(default)]
    pub task_evidence_missing_count: usize,
    #[serde(default = "default_projection_delivery_status")]
    pub delivery_status: String,
    #[serde(default)]
    pub delivery_missing_count: usize,
    #[serde(default)]
    pub audit_required: bool,
    #[serde(default = "default_projection_audit_status")]
    pub audit_status: String,
    #[serde(default)]
    pub audit_blocking_findings: usize,
    #[serde(default = "default_projection_completion_goal_recheck_status")]
    pub goal_recheck_status: String,
    #[serde(default = "default_projection_completion_project_health_status")]
    pub project_health_status: String,
    #[serde(default = "default_projection_completion_release_readiness")]
    pub release_readiness: String,
    pub open_questions: Vec<String>,
    pub rationale: Vec<String>,
    pub projection_path: String,
    pub updated_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompletionDecisionIndexEntry {
    pub project_id: String,
    pub current_state: String,
    pub latest_outcome: Option<String>,
    pub next_recommended_action: String,
    pub projection_path: String,
    pub updated_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompletionDecisionIndex {
    pub version: String,
    pub updated_at: u64,
    pub decisions: Vec<CompletionDecisionIndexEntry>,
}

#[cfg(test)]
mod tests {
    use super::{
        projection_kernel_contract, projection_kernel_core_read_model_schemas,
        projection_kernel_read_model_negative_fixtures, projection_kernel_rejects_authority_write,
        PROJECTION_KERNEL_CONTRACT_VERSION,
    };

    #[test]
    fn projection_kernel_contract_is_read_only_authority_boundary() {
        let contract = projection_kernel_contract();

        assert_eq!(contract.version, PROJECTION_KERNEL_CONTRACT_VERSION);
        assert_eq!(contract.status, "active");
        assert!(!contract.writes_authority);

        let source_patterns = contract
            .accepted_source_refs
            .iter()
            .map(|source| source.path_pattern.as_str())
            .collect::<Vec<_>>();
        assert!(source_patterns.contains(&".agentflow/spec/**"));
        assert!(source_patterns.contains(&".agentflow/events/**"));
        assert!(source_patterns.contains(&".agentflow/tasks/<issue-id>/evidence/**"));
        assert!(source_patterns.contains(&".agentflow/release/**"));

        for target in [
            "Spec",
            "Runtime",
            "Evidence",
            "Decision",
            "Completion",
            "Delivery",
            "Audit",
        ] {
            assert!(projection_kernel_rejects_authority_write(target));
        }
    }

    #[test]
    fn projection_kernel_contract_serializes_negative_fixtures() {
        let payload = serde_json::to_value(projection_kernel_contract()).unwrap();

        assert_eq!(payload["version"], PROJECTION_KERNEL_CONTRACT_VERSION);
        assert_eq!(payload["writesAuthority"], false);
        assert!(payload["negativeFixtures"]
            .as_array()
            .unwrap()
            .iter()
            .any(|fixture| fixture["rejectedRefKind"] == "ProjectionRef"));
        assert!(payload["requiredFields"]
            .as_array()
            .unwrap()
            .iter()
            .any(|field| field == "freshness"));
    }

    #[test]
    fn core_read_model_schemas_cover_stable_fields_and_boundaries() {
        let schemas = projection_kernel_core_read_model_schemas();
        let model_kinds = schemas
            .iter()
            .map(|schema| schema.model_kind.as_str())
            .collect::<Vec<_>>();

        assert_eq!(schemas.len(), 4);
        for model_kind in ["spec", "evidence", "decision", "delivery"] {
            assert!(model_kinds.contains(&model_kind));
        }

        for schema in schemas {
            for required in [
                "objectId",
                "objectType",
                "readModelVersion",
                "sourceRefs",
                "freshness",
                "status",
                "reasonLinks",
                "evidenceLinks",
                "authorityBoundary",
                "updatedAt",
            ] {
                assert!(schema.required_fields.contains(&required.to_string()));
            }
            assert!(schema
                .authority_boundary_fields
                .contains(&"writesAuthority".to_string()));
            assert!(schema
                .authority_boundary_fields
                .contains(&"projectionAuthority".to_string()));
            assert!(schema.freshness_states.contains(&"invalid".to_string()));
            assert!(schema.freshness_states.contains(&"deferred".to_string()));
            assert!(!schema.source_ref_kinds.is_empty());
        }
    }

    #[test]
    fn core_read_model_negative_fixtures_reject_missing_sources() {
        let fixtures = projection_kernel_read_model_negative_fixtures();
        let fixture_ids = fixtures
            .iter()
            .map(|fixture| fixture.fixture_id.as_str())
            .collect::<Vec<_>>();

        for fixture_id in [
            "spec-read-model-missing-spec-source-ref",
            "evidence-read-model-missing-evidence-ref",
            "decision-read-model-missing-evidence-ref",
            "delivery-read-model-missing-public-record-ref",
        ] {
            assert!(fixture_ids.contains(&fixture_id));
        }

        assert!(fixtures
            .iter()
            .all(|fixture| fixture.expected_result == "rejected"));
        assert!(fixtures
            .iter()
            .all(|fixture| !fixture.invalid_combination.is_empty()));
    }
}
