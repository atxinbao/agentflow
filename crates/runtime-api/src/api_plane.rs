use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

pub const API_PLANE_MANIFEST_VERSION: &str = "agentflow-api-plane-manifest.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ApiPlaneBoundary {
    Authority,
    Readonly,
    Command,
    Internal,
}

impl ApiPlaneBoundary {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Authority => "authority",
            Self::Readonly => "readonly",
            Self::Command => "command",
            Self::Internal => "internal",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ApiPlaneAccess {
    LocalOnly,
    SdkCandidate,
    Internal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiPlaneEntry {
    pub api_id: String,
    pub category: String,
    pub label: String,
    pub boundary: ApiPlaneBoundary,
    pub access: ApiPlaneAccess,
    pub source_crate: String,
    pub owner_module: String,
    pub command_or_function: String,
    pub description: String,
}

impl ApiPlaneEntry {
    fn new(
        api_id: &str,
        category: &str,
        label: &str,
        boundary: ApiPlaneBoundary,
        access: ApiPlaneAccess,
        source_crate: &str,
        owner_module: &str,
        command_or_function: &str,
        description: &str,
    ) -> Self {
        Self {
            api_id: api_id.to_string(),
            category: category.to_string(),
            label: label.to_string(),
            boundary,
            access,
            source_crate: source_crate.to_string(),
            owner_module: owner_module.to_string(),
            command_or_function: command_or_function.to_string(),
            description: description.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiPlaneCategorySummary {
    pub category: String,
    pub total: usize,
    pub authority: usize,
    pub readonly: usize,
    pub command: usize,
    pub internal: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiPlaneManifest {
    pub version: String,
    pub scope: String,
    pub boundary_rule: String,
    pub entries: Vec<ApiPlaneEntry>,
    pub categories: Vec<ApiPlaneCategorySummary>,
}

pub fn api_plane_manifest() -> ApiPlaneManifest {
    let entries = vec![
        runtime_command(
            "runtime.command.validate",
            "validate_runtime_command",
            "Validate Runtime Command",
        ),
        runtime_command(
            "runtime.command.execute",
            "execute_command_via_arbitration",
            "Execute Runtime Command through arbitration",
        ),
        runtime_command("runtime.project.intake", "project_intake", "Project intake"),
        runtime_command(
            "runtime.project.preview-goal",
            "project_preview_goal",
            "Preview project goal",
        ),
        runtime_command(
            "runtime.project.confirm-goal",
            "project_confirm_goal",
            "Confirm project goal",
        ),
        runtime_command(
            "runtime.project.confirm-plan",
            "project_confirm_plan",
            "Confirm project plan",
        ),
        runtime_command(
            "runtime.project.materialize",
            "project_materialize",
            "Materialize project facts",
        ),
        projection_query(
            "projection.requirement-intake",
            "get_requirement_intake_view",
            "Requirement intake view",
        ),
        projection_query(
            "projection.spec-preview",
            "get_spec_preview_view",
            "Spec preview view",
        ),
        projection_query(
            "projection.spec-loop",
            "get_spec_loop_view",
            "Spec loop view",
        ),
        projection_query(
            "projection.project-home",
            "get_project_home_view",
            "Project home view",
        ),
        projection_query(
            "projection.task-workbench",
            "get_task_workbench_view",
            "Task workbench view",
        ),
        projection_query(
            "projection.work-loop-run",
            "get_work_loop_run_view",
            "Work loop run view",
        ),
        projection_query(
            "projection.work-loop-session",
            "get_work_loop_session_view",
            "Work loop session view",
        ),
        projection_query(
            "projection.audit-surface",
            "get_audit_surface_view",
            "Audit surface view",
        ),
        projection_query(
            "projection.delivery-package",
            "get_delivery_package_view",
            "Delivery package view",
        ),
        projection_query(
            "projection.runtime-health",
            "get_runtime_health_view",
            "Runtime health view",
        ),
        projection_query(
            "projection.pack-industry-workbench",
            "get_pack_industry_workbench_view",
            "Pack industry workbench view",
        ),
        projection_query(
            "projection.team-workflow-boundary",
            "team_workflow_boundary_contract",
            "Team workflow boundary contract",
        ),
        projection_query(
            "projection.project-sharing",
            "project_sharing_read_model",
            "Project sharing read model",
        ),
        event_api_readonly(
            "event.runtime.replay",
            "replay_runtime_events",
            "Replay runtime event envelopes",
        ),
        event_api_readonly(
            "event.task.replay",
            "replay_task_events_from_cursor",
            "Replay task events from cursor",
        ),
        event_api_internal(
            "event.runtime.append-accepted-action",
            "append_accepted_action_event",
            "Append accepted runtime action event",
        ),
        event_api_internal(
            "event.task.claim",
            "claim_task_event",
            "Claim task event for internal consumer",
        ),
        command_surface(
            "command.surface.action-proposal",
            "map_command_to_action_proposal",
            "Map command to action proposal",
        ),
        command_surface(
            "command.surface.query-hint",
            "runtime_query_hint_for_command",
            "Runtime query hint",
        ),
        connector_action("connector.git.status", "git.status", "Read git status"),
        connector_action("connector.git.diff", "git.diff", "Read git diff"),
        connector_action(
            "connector.github.pr-create",
            "pull_request.create",
            "Request GitHub PR creation",
        ),
        connector_action(
            "connector.github.issue-close",
            "issue.close",
            "Request GitHub issue closeout",
        ),
        connector_action(
            "connector.gitlab.mr-create",
            "merge_request.create",
            "Request GitLab MR creation",
        ),
        provider_action(
            "provider.codex.launch",
            "launch",
            "Launch Codex provider session",
        ),
        provider_action(
            "provider.codex.poll",
            "session.poll",
            "Poll provider session",
        ),
        provider_action(
            "provider.codex.logs",
            "session.logs",
            "Read provider session logs",
        ),
        provider_action(
            "provider.codex.cancel",
            "session.cancel",
            "Cancel provider session",
        ),
        audit_action(
            "audit.request-human",
            "audit_request_human",
            "Request human audit",
        ),
        audit_action(
            "audit.surface-view",
            "get_audit_surface_view",
            "Read audit surface",
        ),
        release_action(
            "release.prepare",
            "release_prepare",
            "Prepare release facts",
        ),
        release_action(
            "release.confirm",
            "release_confirm",
            "Confirm release facts",
        ),
        release_action(
            "release.record-tag",
            "release_record_tag",
            "Record release tag",
        ),
        release_action(
            "release.record-remote",
            "release_record_remote",
            "Record remote release",
        ),
        release_action("release.publish", "release_publish", "Publish release"),
        pack_action(
            "pack.registry.list",
            "get_pack_registry",
            "List local Pack registry",
        ),
        pack_action(
            "pack.manifest.validate",
            "validate_pack_manifest",
            "Validate Pack manifest",
        ),
        pack_action(
            "pack.bundle.validate",
            "validate_pack_bundle",
            "Validate Pack bundle",
        ),
        pack_action(
            "pack.validation.read",
            "get_pack_validation_artifact",
            "Read Pack validation artifact",
        ),
        pack_action(
            "pack.migration.preview",
            "generate_pack_migration_preview",
            "Generate Pack migration preview",
        ),
        pack_command_readonly(
            "pack.command.list",
            "list_pack_commands",
            "List Pack commands",
        ),
        pack_command_readonly(
            "pack.command.validate",
            "validate_pack_command",
            "Validate Pack command",
        ),
        pack_command_readonly(
            "pack.command.dry-run",
            "dry_run_pack_command",
            "Dry-run Pack command",
        ),
        pack_command(
            "pack.command.submit-proposal",
            "submit_pack_action_proposal",
            "Submit Pack action proposal",
        ),
        pack_command_readonly(
            "pack.capability.status",
            "query_pack_capability_status",
            "Query Pack capability status",
        ),
        pack_command_readonly(
            "pack.surface.route",
            "query_pack_surface_route",
            "Query Pack surface route",
        ),
        pack_command_internal(
            "pack.command.resolve-runtime",
            "runtime_command_type_for_action_contract",
            "Resolve Pack command runtime mapping",
        ),
    ];
    let categories = summarize_categories(&entries);
    ApiPlaneManifest {
        version: API_PLANE_MANIFEST_VERSION.to_string(),
        scope: "runtime/projection/event/command/connector/provider/audit/release/pack".to_string(),
        boundary_rule: "SDK, UI, connector, provider, and Pack outputs are not authority; write actions enter Runtime API / Command Surface first, while SDK-visible query and event paths remain readonly.".to_string(),
        entries,
        categories,
    }
}

pub fn write_api_plane_manifest(
    output_path: impl AsRef<Path>,
    manifest: &ApiPlaneManifest,
) -> Result<()> {
    let path = output_path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_string_pretty(manifest)? + "\n")?;
    Ok(())
}

fn runtime_command(api_id: &str, function: &str, label: &str) -> ApiPlaneEntry {
    ApiPlaneEntry::new(
        api_id,
        "runtime_commands",
        label,
        ApiPlaneBoundary::Command,
        ApiPlaneAccess::LocalOnly,
        "agentflow-runtime-api",
        "commands/formal",
        function,
        "Runtime write-side command; must pass validation, proposal, and arbitration.",
    )
}

fn projection_query(api_id: &str, function: &str, label: &str) -> ApiPlaneEntry {
    ApiPlaneEntry::new(
        api_id,
        "projection_queries",
        label,
        ApiPlaneBoundary::Readonly,
        ApiPlaneAccess::SdkCandidate,
        "agentflow-runtime-api",
        "query",
        function,
        "Projection read model query; never writes authority.",
    )
}

fn event_api_readonly(api_id: &str, function: &str, label: &str) -> ApiPlaneEntry {
    ApiPlaneEntry::new(
        api_id,
        "event_api",
        label,
        ApiPlaneBoundary::Readonly,
        ApiPlaneAccess::SdkCandidate,
        "agentflow-event-store",
        "runtime/storage",
        function,
        "Event read API; SDK may replay or inspect event receipts but cannot append authority facts.",
    )
}

fn event_api_internal(api_id: &str, function: &str, label: &str) -> ApiPlaneEntry {
    ApiPlaneEntry::new(
        api_id,
        "event_api",
        label,
        ApiPlaneBoundary::Internal,
        ApiPlaneAccess::Internal,
        "agentflow-event-store",
        "runtime/storage",
        function,
        "Internal event write or claim API; only Runtime-controlled flows may call it.",
    )
}

fn command_surface(api_id: &str, function: &str, label: &str) -> ApiPlaneEntry {
    ApiPlaneEntry::new(
        api_id,
        "command_surface_actions",
        label,
        ApiPlaneBoundary::Command,
        ApiPlaneAccess::LocalOnly,
        "agentflow-runtime-api",
        "mapping",
        function,
        "Command Surface helper; maps UI or agent request into runtime-controlled action proposal.",
    )
}

fn connector_action(api_id: &str, capability: &str, label: &str) -> ApiPlaneEntry {
    ApiPlaneEntry::new(
        api_id,
        "connector_actions",
        label,
        ApiPlaneBoundary::Command,
        ApiPlaneAccess::LocalOnly,
        "agentflow-capability-registry",
        "connector-boundary",
        capability,
        "Connector capability; output is context, evidence, or external fact, not authority.",
    )
}

fn provider_action(api_id: &str, capability: &str, label: &str) -> ApiPlaneEntry {
    ApiPlaneEntry::new(
        api_id,
        "provider_actions",
        label,
        ApiPlaneBoundary::Command,
        ApiPlaneAccess::LocalOnly,
        "agentflow-mcp",
        "provider",
        capability,
        "Provider session action; state is projected through MCP session snapshots.",
    )
}

fn audit_action(api_id: &str, function: &str, label: &str) -> ApiPlaneEntry {
    ApiPlaneEntry::new(
        api_id,
        "audit_actions",
        label,
        ApiPlaneBoundary::Command,
        ApiPlaneAccess::LocalOnly,
        "agentflow-runtime-api",
        "formal/query",
        function,
        "Audit sidecar action or read model; audit does not block Work Done.",
    )
}

fn release_action(api_id: &str, function: &str, label: &str) -> ApiPlaneEntry {
    ApiPlaneEntry::new(
        api_id,
        "release_actions",
        label,
        ApiPlaneBoundary::Authority,
        ApiPlaneAccess::LocalOnly,
        "agentflow-runtime-api",
        "formal",
        function,
        "Release authority command; writes release facts through runtime-controlled entry points.",
    )
}

fn pack_action(api_id: &str, function: &str, label: &str) -> ApiPlaneEntry {
    ApiPlaneEntry::new(
        api_id,
        "pack_actions",
        label,
        ApiPlaneBoundary::Readonly,
        ApiPlaneAccess::SdkCandidate,
        "agentflow-pack",
        "manifest/registry",
        function,
        "Pack definition read or validation entry; never writes runtime authority.",
    )
}

fn pack_command_readonly(api_id: &str, function: &str, label: &str) -> ApiPlaneEntry {
    ApiPlaneEntry::new(
        api_id,
        "pack_command_surface",
        label,
        ApiPlaneBoundary::Readonly,
        ApiPlaneAccess::SdkCandidate,
        "agentflow-runtime-api",
        "pack",
        function,
        "Pack command read entry; resolves Pack definitions but never writes authority.",
    )
}

fn pack_command(api_id: &str, function: &str, label: &str) -> ApiPlaneEntry {
    ApiPlaneEntry::new(
        api_id,
        "pack_command_surface",
        label,
        ApiPlaneBoundary::Command,
        ApiPlaneAccess::LocalOnly,
        "agentflow-runtime-api",
        "pack",
        function,
        "Pack command write entry; must map to Runtime Command then pass Action Contract and Arbitration.",
    )
}

fn pack_command_internal(api_id: &str, function: &str, label: &str) -> ApiPlaneEntry {
    ApiPlaneEntry::new(
        api_id,
        "pack_command_surface",
        label,
        ApiPlaneBoundary::Internal,
        ApiPlaneAccess::Internal,
        "agentflow-runtime-api",
        "pack",
        function,
        "Internal Pack command helper; not a fact source and not a public API.",
    )
}

fn summarize_categories(entries: &[ApiPlaneEntry]) -> Vec<ApiPlaneCategorySummary> {
    let mut categories = entries
        .iter()
        .map(|entry| entry.category.clone())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .map(|category| {
            let matching = entries
                .iter()
                .filter(|entry| entry.category == category)
                .collect::<Vec<_>>();
            ApiPlaneCategorySummary {
                category,
                total: matching.len(),
                authority: matching
                    .iter()
                    .filter(|entry| entry.boundary == ApiPlaneBoundary::Authority)
                    .count(),
                readonly: matching
                    .iter()
                    .filter(|entry| entry.boundary == ApiPlaneBoundary::Readonly)
                    .count(),
                command: matching
                    .iter()
                    .filter(|entry| entry.boundary == ApiPlaneBoundary::Command)
                    .count(),
                internal: matching
                    .iter()
                    .filter(|entry| entry.boundary == ApiPlaneBoundary::Internal)
                    .count(),
            }
        })
        .collect::<Vec<_>>();
    categories.sort_by(|left, right| left.category.cmp(&right.category));
    categories
}

#[cfg(test)]
mod tests {
    use super::{api_plane_manifest, write_api_plane_manifest, ApiPlaneBoundary};

    #[test]
    fn manifest_covers_required_planes_and_boundaries() {
        let manifest = api_plane_manifest();
        let required_categories = [
            "runtime_commands",
            "projection_queries",
            "event_api",
            "command_surface_actions",
            "connector_actions",
            "provider_actions",
            "audit_actions",
            "release_actions",
            "pack_actions",
            "pack_command_surface",
        ];

        for category in required_categories {
            assert!(
                manifest
                    .entries
                    .iter()
                    .any(|entry| entry.category == category),
                "missing category {category}"
            );
        }

        assert!(manifest
            .entries
            .iter()
            .all(|entry| !entry.api_id.is_empty() && !entry.command_or_function.is_empty()));
        assert!(manifest.entries.iter().any(|entry| {
            entry.category == "projection_queries" && entry.boundary == ApiPlaneBoundary::Readonly
        }));
        assert!(manifest.entries.iter().any(|entry| {
            entry.category == "event_api" && entry.boundary == ApiPlaneBoundary::Readonly
        }));
        assert!(manifest.entries.iter().any(|entry| {
            entry.category == "event_api" && entry.boundary == ApiPlaneBoundary::Internal
        }));
        assert!(manifest.entries.iter().any(|entry| {
            entry.category == "release_actions" && entry.boundary == ApiPlaneBoundary::Authority
        }));
        assert!(manifest.entries.iter().any(|entry| {
            entry.category == "pack_actions" && entry.boundary == ApiPlaneBoundary::Readonly
        }));
        assert!(manifest.entries.iter().any(|entry| {
            entry.category == "pack_command_surface" && entry.boundary == ApiPlaneBoundary::Readonly
        }));
        assert!(manifest.entries.iter().any(|entry| {
            entry.category == "pack_command_surface" && entry.boundary == ApiPlaneBoundary::Command
        }));
        assert!(manifest.entries.iter().any(|entry| {
            entry.category == "pack_command_surface" && entry.boundary == ApiPlaneBoundary::Internal
        }));
        assert!(manifest.entries.iter().all(|entry| matches!(
            entry.boundary,
            ApiPlaneBoundary::Authority
                | ApiPlaneBoundary::Readonly
                | ApiPlaneBoundary::Command
                | ApiPlaneBoundary::Internal
        )));
    }

    #[test]
    fn sdk_candidates_are_readonly_and_cannot_bypass_runtime_authority() {
        let manifest = api_plane_manifest();

        let sdk_entries = manifest
            .entries
            .iter()
            .filter(|entry| entry.access == super::ApiPlaneAccess::SdkCandidate)
            .collect::<Vec<_>>();
        assert!(
            !sdk_entries.is_empty(),
            "manifest should expose readonly SDK candidate APIs"
        );
        assert!(sdk_entries.iter().all(|entry| {
            entry.boundary == ApiPlaneBoundary::Readonly
                && !matches!(
                    entry.category.as_str(),
                    "runtime_commands"
                        | "command_surface_actions"
                        | "connector_actions"
                        | "provider_actions"
                        | "release_actions"
                )
        }));
    }

    #[test]
    fn manifest_covers_command_query_and_event_contract_paths() {
        let manifest = api_plane_manifest();

        let required_api_ids = [
            "runtime.command.validate",
            "runtime.command.execute",
            "projection.task-workbench",
            "projection.pack-industry-workbench",
            "event.runtime.replay",
            "event.task.replay",
            "event.runtime.append-accepted-action",
            "pack.command.list",
            "pack.command.validate",
            "pack.command.dry-run",
            "pack.command.submit-proposal",
            "pack.surface.route",
            "pack.capability.status",
        ];

        for api_id in required_api_ids {
            assert!(
                manifest.entries.iter().any(|entry| entry.api_id == api_id),
                "missing API Plane contract entry {api_id}"
            );
        }
    }

    #[test]
    fn manifest_can_be_written_to_disk() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("api-plane-manifest.json");
        let manifest = api_plane_manifest();

        write_api_plane_manifest(&path, &manifest).unwrap();
        let payload = std::fs::read_to_string(path).unwrap();

        assert!(payload.contains("agentflow-api-plane-manifest.v1"));
        assert!(payload.contains("runtime_commands"));
    }
}
