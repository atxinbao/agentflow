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
    ];
    let categories = summarize_categories(&entries);
    ApiPlaneManifest {
        version: API_PLANE_MANIFEST_VERSION.to_string(),
        scope: "runtime/projection/command/connector/provider/audit/release".to_string(),
        boundary_rule: "UI and connector outputs are not authority; write actions enter Runtime API / Command Surface first.".to_string(),
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
            "command_surface_actions",
            "connector_actions",
            "provider_actions",
            "audit_actions",
            "release_actions",
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
            entry.category == "release_actions" && entry.boundary == ApiPlaneBoundary::Authority
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
