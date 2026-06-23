//! AgentFlow schema version registry and migration preview boundary.
//!
//! This crate is intentionally small: it lists known fact schema versions,
//! detects stale observations, and produces preview-only migration plans. It
//! does not rewrite `.agentflow/**` authority files.

use agentflow_spec::model::{
    REQUIREMENT_PREVIEW_VERSION, SPEC_ISSUE_VERSION, SPEC_PROJECT_VERSION,
};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

pub const SCHEMA_REGISTRY_VERSION: &str = "agentflow-schema-registry.v1";
pub const MIGRATION_PREVIEW_VERSION: &str = "agentflow-migration-preview.v1";
pub const MIGRATION_PREVIEW_RECEIPT_VERSION: &str = "agentflow-migration-preview-receipt.v1";
pub const MIGRATION_APPLY_RECEIPT_VERSION: &str = "agentflow-migration-apply-receipt.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SchemaAuthorityLayer {
    Spec,
    EventStore,
    TaskArtifact,
    Projection,
    Audit,
    Release,
    RuntimeApi,
    State,
    Workflow,
}

impl SchemaAuthorityLayer {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Spec => "spec",
            Self::EventStore => "event-store",
            Self::TaskArtifact => "task-artifact",
            Self::Projection => "projection",
            Self::Audit => "audit",
            Self::Release => "release",
            Self::RuntimeApi => "runtime-api",
            Self::State => "state",
            Self::Workflow => "workflow",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaVersionEntry {
    pub schema_id: String,
    pub current_version: String,
    pub authority_layer: SchemaAuthorityLayer,
    pub path_pattern: String,
    pub migration_strategy: MigrationStrategy,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MigrationStrategy {
    PreviewOnly,
    ExplicitApplyRequired,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaRegistry {
    pub version: String,
    pub entries: Vec<SchemaVersionEntry>,
}

impl SchemaRegistry {
    pub fn find(&self, schema_id: &str) -> Option<&SchemaVersionEntry> {
        self.entries
            .iter()
            .find(|entry| entry.schema_id == schema_id)
    }

    pub fn current_versions(&self) -> Vec<SchemaVersionEntry> {
        self.entries.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaObservation {
    pub schema_id: String,
    pub path: String,
    pub observed_version: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SchemaDetectionStatus {
    Current,
    Legacy,
    MissingVersion,
    UnknownSchema,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaDetection {
    pub observation: SchemaObservation,
    pub status: SchemaDetectionStatus,
    pub current_version: Option<String>,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MigrationPreview {
    pub version: String,
    pub preview_id: String,
    pub mode: MigrationMode,
    pub writes_authority: bool,
    pub detections: Vec<SchemaDetection>,
    pub proposed_actions: Vec<MigrationPreviewAction>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MigrationMode {
    Preview,
    Apply,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MigrationPreviewAction {
    pub schema_id: String,
    pub path: String,
    pub from_version: Option<String>,
    pub to_version: Option<String>,
    pub action: String,
    pub requires_explicit_apply: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MigrationPreviewReceipt {
    pub version: String,
    pub preview_id: String,
    pub receipt_kind: MigrationReceiptKind,
    pub writes_authority: bool,
    pub proposed_action_count: usize,
    pub legacy_count: usize,
    pub missing_version_count: usize,
    pub unknown_schema_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MigrationReceiptKind {
    Preview,
    Applied,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MigrationApplyConfirmation {
    pub preview_id: String,
    pub confirmed: bool,
    pub actor: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MigrationApplyReceipt {
    pub version: String,
    pub preview_id: String,
    pub receipt_kind: MigrationReceiptKind,
    pub applied: bool,
    pub authority_writes: Vec<String>,
    pub deferred_actions: Vec<MigrationPreviewAction>,
    pub actor: String,
    pub reason: String,
}

pub fn core_schema_registry() -> SchemaRegistry {
    SchemaRegistry {
        version: SCHEMA_REGISTRY_VERSION.to_string(),
        entries: vec![
            entry(
                "spec.issue",
                SPEC_ISSUE_VERSION,
                SchemaAuthorityLayer::Spec,
                ".agentflow/spec/issues/*.json",
                MigrationStrategy::ExplicitApplyRequired,
            ),
            entry(
                "spec.project",
                SPEC_PROJECT_VERSION,
                SchemaAuthorityLayer::Spec,
                ".agentflow/spec/projects/*.json",
                MigrationStrategy::ExplicitApplyRequired,
            ),
            entry(
                "spec.requirement-preview",
                REQUIREMENT_PREVIEW_VERSION,
                SchemaAuthorityLayer::Spec,
                ".agentflow/spec/requirements/*/runtime.json",
                MigrationStrategy::ExplicitApplyRequired,
            ),
            entry(
                "event.task",
                agentflow_event_store::TASK_EVENT_VERSION,
                SchemaAuthorityLayer::EventStore,
                ".agentflow/events/*.jsonl",
                MigrationStrategy::ExplicitApplyRequired,
            ),
            entry(
                "event.runtime-envelope",
                agentflow_event_store::RUNTIME_EVENT_ENVELOPE_VERSION,
                SchemaAuthorityLayer::EventStore,
                ".agentflow/events/runtime/*.jsonl",
                MigrationStrategy::ExplicitApplyRequired,
            ),
            entry(
                "task.run",
                agentflow_task_artifacts::TASK_RUN_VERSION,
                SchemaAuthorityLayer::TaskArtifact,
                ".agentflow/tasks/*/runs/*/run.json",
                MigrationStrategy::ExplicitApplyRequired,
            ),
            entry(
                "task.evidence",
                agentflow_task_artifacts::TASK_EVIDENCE_VERSION,
                SchemaAuthorityLayer::TaskArtifact,
                ".agentflow/tasks/*/evidence/*.json",
                MigrationStrategy::ExplicitApplyRequired,
            ),
            entry(
                "projection.task",
                agentflow_projection::TASK_PROJECTION_VERSION,
                SchemaAuthorityLayer::Projection,
                ".agentflow/projections/tasks/*.json",
                MigrationStrategy::PreviewOnly,
            ),
            entry(
                "projection.project",
                agentflow_projection::PROJECT_PROJECTION_VERSION,
                SchemaAuthorityLayer::Projection,
                ".agentflow/projections/projects/*.json",
                MigrationStrategy::PreviewOnly,
            ),
            entry(
                "audit.request",
                agentflow_audit::AUDIT_REQUEST_VERSION,
                SchemaAuthorityLayer::Audit,
                ".agentflow/audit/*/request.json",
                MigrationStrategy::ExplicitApplyRequired,
            ),
            entry(
                "audit.findings",
                agentflow_audit::AUDIT_FINDINGS_VERSION,
                SchemaAuthorityLayer::Audit,
                ".agentflow/audit/*/findings.json",
                MigrationStrategy::ExplicitApplyRequired,
            ),
            entry(
                "release.delivery-summary",
                agentflow_release::DELIVERY_SUMMARY_VERSION,
                SchemaAuthorityLayer::Release,
                ".agentflow/release/delivery/*.json",
                MigrationStrategy::ExplicitApplyRequired,
            ),
            entry(
                "runtime.command-api",
                agentflow_runtime_api::RUNTIME_COMMAND_API_VERSION,
                SchemaAuthorityLayer::RuntimeApi,
                ".agentflow/runtime/commands/*.json",
                MigrationStrategy::ExplicitApplyRequired,
            ),
            entry(
                "state.manifest",
                agentflow_state::STATE_MANIFEST_VERSION,
                SchemaAuthorityLayer::State,
                ".agentflow/state/manifest.json",
                MigrationStrategy::PreviewOnly,
            ),
            entry(
                "workflow.definition",
                agentflow_workflow_core::AGENTFLOW_WORKFLOW_API_VERSION,
                SchemaAuthorityLayer::Workflow,
                ".agentflow/workflows/*.yaml",
                MigrationStrategy::PreviewOnly,
            ),
        ],
    }
}

pub fn detect_schema_version(
    registry: &SchemaRegistry,
    observation: SchemaObservation,
) -> SchemaDetection {
    let Some(entry) = registry.find(&observation.schema_id) else {
        return SchemaDetection {
            observation,
            status: SchemaDetectionStatus::UnknownSchema,
            current_version: None,
            reason: "schema id is not registered".to_string(),
        };
    };

    match observation.observed_version.as_deref() {
        Some(observed) if observed == entry.current_version => SchemaDetection {
            observation,
            status: SchemaDetectionStatus::Current,
            current_version: Some(entry.current_version.clone()),
            reason: "observed version matches registry".to_string(),
        },
        Some(_) => SchemaDetection {
            observation,
            status: SchemaDetectionStatus::Legacy,
            current_version: Some(entry.current_version.clone()),
            reason: "observed version differs from registry".to_string(),
        },
        None => SchemaDetection {
            observation,
            status: SchemaDetectionStatus::MissingVersion,
            current_version: Some(entry.current_version.clone()),
            reason: "version field is missing".to_string(),
        },
    }
}

pub fn generate_migration_preview(
    registry: &SchemaRegistry,
    preview_id: impl Into<String>,
    observations: Vec<SchemaObservation>,
) -> MigrationPreview {
    let detections: Vec<SchemaDetection> = observations
        .into_iter()
        .map(|observation| detect_schema_version(registry, observation))
        .collect();
    let proposed_actions = detections
        .iter()
        .filter_map(|detection| preview_action_for_detection(registry, detection))
        .collect();

    MigrationPreview {
        version: MIGRATION_PREVIEW_VERSION.to_string(),
        preview_id: preview_id.into(),
        mode: MigrationMode::Preview,
        writes_authority: false,
        detections,
        proposed_actions,
    }
}

pub fn migration_preview_receipt(preview: &MigrationPreview) -> MigrationPreviewReceipt {
    MigrationPreviewReceipt {
        version: MIGRATION_PREVIEW_RECEIPT_VERSION.to_string(),
        preview_id: preview.preview_id.clone(),
        receipt_kind: MigrationReceiptKind::Preview,
        writes_authority: false,
        proposed_action_count: preview.proposed_actions.len(),
        legacy_count: preview
            .detections
            .iter()
            .filter(|detection| detection.status == SchemaDetectionStatus::Legacy)
            .count(),
        missing_version_count: preview
            .detections
            .iter()
            .filter(|detection| detection.status == SchemaDetectionStatus::MissingVersion)
            .count(),
        unknown_schema_count: preview
            .detections
            .iter()
            .filter(|detection| detection.status == SchemaDetectionStatus::UnknownSchema)
            .count(),
    }
}

pub fn apply_migration_preview(
    preview: &MigrationPreview,
    confirmation: MigrationApplyConfirmation,
) -> Result<MigrationApplyReceipt> {
    if !confirmation.confirmed {
        return Err(anyhow!("migration apply requires explicit confirmation"));
    }
    if confirmation.preview_id != preview.preview_id {
        return Err(anyhow!(
            "migration confirmation preview id mismatch: expected {}, got {}",
            preview.preview_id,
            confirmation.preview_id
        ));
    }

    Ok(MigrationApplyReceipt {
        version: MIGRATION_APPLY_RECEIPT_VERSION.to_string(),
        preview_id: preview.preview_id.clone(),
        receipt_kind: MigrationReceiptKind::Applied,
        applied: true,
        authority_writes: Vec::new(),
        deferred_actions: preview.proposed_actions.clone(),
        actor: confirmation.actor,
        reason: confirmation.reason,
    })
}

fn preview_action_for_detection(
    registry: &SchemaRegistry,
    detection: &SchemaDetection,
) -> Option<MigrationPreviewAction> {
    if detection.status == SchemaDetectionStatus::Current {
        return None;
    }
    let entry = registry.find(&detection.observation.schema_id)?;
    Some(MigrationPreviewAction {
        schema_id: entry.schema_id.clone(),
        path: detection.observation.path.clone(),
        from_version: detection.observation.observed_version.clone(),
        to_version: Some(entry.current_version.clone()),
        action: match detection.status {
            SchemaDetectionStatus::Legacy => "upgrade-versioned-fact".to_string(),
            SchemaDetectionStatus::MissingVersion => "add-version-field".to_string(),
            SchemaDetectionStatus::UnknownSchema => "manual-review".to_string(),
            SchemaDetectionStatus::Current => "none".to_string(),
        },
        requires_explicit_apply: entry.migration_strategy
            == MigrationStrategy::ExplicitApplyRequired,
    })
}

fn entry(
    schema_id: &str,
    current_version: &str,
    authority_layer: SchemaAuthorityLayer,
    path_pattern: &str,
    migration_strategy: MigrationStrategy,
) -> SchemaVersionEntry {
    SchemaVersionEntry {
        schema_id: schema_id.to_string(),
        current_version: current_version.to_string(),
        authority_layer,
        path_pattern: path_pattern.to_string(),
        migration_strategy,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_lists_current_schema_versions() {
        let registry = core_schema_registry();

        assert_eq!(registry.version, SCHEMA_REGISTRY_VERSION);
        assert!(registry.find("spec.issue").is_some());
        assert!(registry.find("event.task").is_some());
        assert!(registry.find("task.run").is_some());
        assert!(registry.find("projection.project").is_some());
        assert!(registry
            .current_versions()
            .iter()
            .any(|entry| entry.current_version == SPEC_ISSUE_VERSION));
    }

    #[test]
    fn detector_marks_current_legacy_missing_and_unknown_versions() {
        let registry = core_schema_registry();

        let current = detect_schema_version(
            &registry,
            SchemaObservation {
                schema_id: "spec.issue".to_string(),
                path: ".agentflow/spec/issues/AF-1.json".to_string(),
                observed_version: Some(SPEC_ISSUE_VERSION.to_string()),
            },
        );
        assert_eq!(current.status, SchemaDetectionStatus::Current);

        let legacy = detect_schema_version(
            &registry,
            SchemaObservation {
                schema_id: "spec.issue".to_string(),
                path: ".agentflow/spec/issues/AF-1.json".to_string(),
                observed_version: Some("agentflow-spec-issue.v0".to_string()),
            },
        );
        assert_eq!(legacy.status, SchemaDetectionStatus::Legacy);

        let missing = detect_schema_version(
            &registry,
            SchemaObservation {
                schema_id: "task.run".to_string(),
                path: ".agentflow/tasks/AF-1/runs/run-001/run.json".to_string(),
                observed_version: None,
            },
        );
        assert_eq!(missing.status, SchemaDetectionStatus::MissingVersion);

        let unknown = detect_schema_version(
            &registry,
            SchemaObservation {
                schema_id: "pack.future".to_string(),
                path: ".agentflow/packs/future.json".to_string(),
                observed_version: Some("pack.v1".to_string()),
            },
        );
        assert_eq!(unknown.status, SchemaDetectionStatus::UnknownSchema);
    }

    #[test]
    fn migration_preview_is_preview_only_and_does_not_write_authority() {
        let registry = core_schema_registry();
        let preview = generate_migration_preview(
            &registry,
            "preview-001",
            vec![
                SchemaObservation {
                    schema_id: "spec.issue".to_string(),
                    path: ".agentflow/spec/issues/AF-1.json".to_string(),
                    observed_version: Some("agentflow-spec-issue.v0".to_string()),
                },
                SchemaObservation {
                    schema_id: "projection.project".to_string(),
                    path: ".agentflow/projections/projects/project.json".to_string(),
                    observed_version: None,
                },
            ],
        );

        assert_eq!(preview.version, MIGRATION_PREVIEW_VERSION);
        assert_eq!(preview.mode, MigrationMode::Preview);
        assert!(!preview.writes_authority);
        assert_eq!(preview.proposed_actions.len(), 2);
        let receipt = migration_preview_receipt(&preview);
        assert_eq!(receipt.version, MIGRATION_PREVIEW_RECEIPT_VERSION);
        assert_eq!(receipt.receipt_kind, MigrationReceiptKind::Preview);
        assert!(!receipt.writes_authority);
        assert_eq!(receipt.proposed_action_count, 2);
        assert!(preview
            .proposed_actions
            .iter()
            .any(|action| action.requires_explicit_apply));
    }

    #[test]
    fn migration_apply_requires_matching_explicit_confirmation() {
        let registry = core_schema_registry();
        let preview = generate_migration_preview(
            &registry,
            "preview-002",
            vec![SchemaObservation {
                schema_id: "spec.issue".to_string(),
                path: ".agentflow/spec/issues/AF-1.json".to_string(),
                observed_version: Some("agentflow-spec-issue.v0".to_string()),
            }],
        );

        let rejected = apply_migration_preview(
            &preview,
            MigrationApplyConfirmation {
                preview_id: "preview-002".to_string(),
                confirmed: false,
                actor: "codex".to_string(),
                reason: "test".to_string(),
            },
        );
        assert!(rejected.is_err());

        let mismatch = apply_migration_preview(
            &preview,
            MigrationApplyConfirmation {
                preview_id: "wrong-preview".to_string(),
                confirmed: true,
                actor: "codex".to_string(),
                reason: "test".to_string(),
            },
        );
        assert!(mismatch.is_err());

        let receipt = apply_migration_preview(
            &preview,
            MigrationApplyConfirmation {
                preview_id: "preview-002".to_string(),
                confirmed: true,
                actor: "codex".to_string(),
                reason: "explicit test confirmation".to_string(),
            },
        )
        .unwrap();
        assert!(receipt.applied);
        assert_eq!(receipt.receipt_kind, MigrationReceiptKind::Applied);
        assert!(receipt.authority_writes.is_empty());
        assert_eq!(receipt.deferred_actions.len(), 1);
    }
}
