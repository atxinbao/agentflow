use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const PACK_SURFACE_VERSION: &str = "agentflow-pack-surface.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackSurfaceDefinition {
    pub version: String,
    pub pack_id: String,
    pub surface_id: String,
    pub pages: Vec<SurfacePage>,
    pub workbenches: Vec<SurfaceWorkbench>,
    pub view_model_mappings: Vec<SurfaceViewModelMapping>,
    pub command_entry_mappings: Vec<SurfaceCommandEntryMapping>,
    pub read_model_dependencies: Vec<SurfaceReadModelDependency>,
    pub navigation_rules: Vec<SurfaceNavigationRule>,
    pub state_policy: SurfaceStatePolicy,
    #[serde(default)]
    pub sidecar_surfaces: Vec<SurfaceSidecar>,
    pub writes_authority: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SurfacePageKind {
    Main,
    Workbench,
    Detail,
    Sidecar,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SurfacePage {
    pub page_id: String,
    pub label: String,
    pub description: String,
    pub kind: SurfacePageKind,
    pub view_model_ref: String,
    #[serde(default)]
    pub command_entry_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SurfaceWorkbench {
    pub workbench_id: String,
    pub page_id: String,
    pub label: String,
    pub primary_object_type: String,
    pub timeline_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SurfaceViewModelMapping {
    pub mapping_id: String,
    pub page_id: String,
    pub projection_ref: String,
    pub view_model_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SurfaceCommandRoute {
    RuntimeCommand,
    ActionProposal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SurfaceCommandEntryMapping {
    pub command_entry_id: String,
    pub page_id: String,
    pub label: String,
    pub command_type: String,
    pub route: SurfaceCommandRoute,
    pub action_contract_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SurfaceReadModelDependency {
    pub dependency_id: String,
    pub page_id: String,
    pub projection_ref: String,
    pub required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SurfaceNavigationRule {
    pub from_page_id: String,
    pub to_page_id: String,
    pub trigger: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SurfaceStatePolicy {
    pub empty_state_ref: String,
    pub loading_state_ref: String,
    pub error_state_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SurfaceSidecar {
    pub sidecar_id: String,
    pub page_id: String,
    pub label: String,
    pub blocks_main_chain: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackSurfaceValidationIssue {
    pub field: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackSurfaceValidationReport {
    pub version: String,
    pub pack_id: String,
    pub surface_id: String,
    pub valid: bool,
    pub writes_authority: bool,
    pub issues: Vec<PackSurfaceValidationIssue>,
}

pub fn validate_surface_definition(surface: &PackSurfaceDefinition) -> PackSurfaceValidationReport {
    let mut issues = Vec::new();
    require_non_empty(&mut issues, "version", &surface.version);
    if surface.version != PACK_SURFACE_VERSION {
        issues.push(issue("version", "must be agentflow-pack-surface.v1"));
    }
    require_non_empty(&mut issues, "packId", &surface.pack_id);
    require_non_empty(&mut issues, "surfaceId", &surface.surface_id);
    if surface.writes_authority {
        issues.push(issue(
            "writesAuthority",
            "surface pack definitions must not write runtime authority",
        ));
    }
    if surface.pages.is_empty() {
        issues.push(issue("pages", "must contain at least one page"));
    }
    if surface.view_model_mappings.is_empty() {
        issues.push(issue(
            "viewModelMappings",
            "must map pages to projection-backed view models",
        ));
    }
    if surface.read_model_dependencies.is_empty() {
        issues.push(issue(
            "readModelDependencies",
            "must declare projection read dependencies",
        ));
    }

    let page_ids = surface
        .pages
        .iter()
        .map(|page| page.page_id.as_str())
        .collect::<BTreeSet<_>>();
    let command_ids = surface
        .command_entry_mappings
        .iter()
        .map(|command| command.command_entry_id.as_str())
        .collect::<BTreeSet<_>>();

    for page in &surface.pages {
        require_non_empty(&mut issues, "pages.pageId", &page.page_id);
        require_non_empty(&mut issues, "pages.label", &page.label);
        require_non_empty(&mut issues, "pages.viewModelRef", &page.view_model_ref);
        for command_id in &page.command_entry_ids {
            if !command_ids.contains(command_id.as_str()) {
                issues.push(issue(
                    "pages.commandEntryIds",
                    "must reference a declared command entry",
                ));
            }
        }
    }
    for workbench in &surface.workbenches {
        if !page_ids.contains(workbench.page_id.as_str()) {
            issues.push(issue(
                "workbenches.pageId",
                "must reference a declared page",
            ));
        }
        require_non_empty(
            &mut issues,
            "workbenches.primaryObjectType",
            &workbench.primary_object_type,
        );
    }
    for mapping in &surface.view_model_mappings {
        if !page_ids.contains(mapping.page_id.as_str()) {
            issues.push(issue(
                "viewModelMappings.pageId",
                "must reference a declared page",
            ));
        }
        require_non_empty(
            &mut issues,
            "viewModelMappings.projectionRef",
            &mapping.projection_ref,
        );
        require_non_empty(
            &mut issues,
            "viewModelMappings.viewModelRef",
            &mapping.view_model_ref,
        );
    }
    for command in &surface.command_entry_mappings {
        if !page_ids.contains(command.page_id.as_str()) {
            issues.push(issue(
                "commandEntryMappings.pageId",
                "must reference a declared page",
            ));
        }
        require_non_empty(
            &mut issues,
            "commandEntryMappings.actionContractRef",
            &command.action_contract_ref,
        );
    }
    for dependency in &surface.read_model_dependencies {
        if !page_ids.contains(dependency.page_id.as_str()) {
            issues.push(issue(
                "readModelDependencies.pageId",
                "must reference a declared page",
            ));
        }
        require_non_empty(
            &mut issues,
            "readModelDependencies.projectionRef",
            &dependency.projection_ref,
        );
    }
    for rule in &surface.navigation_rules {
        if !page_ids.contains(rule.from_page_id.as_str()) {
            issues.push(issue(
                "navigationRules.fromPageId",
                "must reference a declared page",
            ));
        }
        if !page_ids.contains(rule.to_page_id.as_str()) {
            issues.push(issue(
                "navigationRules.toPageId",
                "must reference a declared page",
            ));
        }
    }
    for sidecar in &surface.sidecar_surfaces {
        let page = surface
            .pages
            .iter()
            .find(|page| page.page_id == sidecar.page_id);
        match page {
            Some(page) if page.kind == SurfacePageKind::Sidecar => {}
            Some(_) => issues.push(issue(
                "sidecarSurfaces.pageId",
                "must reference a page with sidecar kind",
            )),
            None => issues.push(issue(
                "sidecarSurfaces.pageId",
                "must reference a declared page",
            )),
        }
        if sidecar.blocks_main_chain {
            issues.push(issue(
                "sidecarSurfaces.blocksMainChain",
                "sidecar surfaces must not block the main business chain",
            ));
        }
    }

    PackSurfaceValidationReport {
        version: "agentflow-pack-surface-validation.v1".to_string(),
        pack_id: surface.pack_id.clone(),
        surface_id: surface.surface_id.clone(),
        valid: issues.is_empty(),
        writes_authority: surface.writes_authority,
        issues,
    }
}

pub fn software_dev_surface_definition() -> PackSurfaceDefinition {
    PackSurfaceDefinition {
        version: PACK_SURFACE_VERSION.to_string(),
        pack_id: "software-dev".to_string(),
        surface_id: "software-dev-surface".to_string(),
        pages: vec![
            page("project-home", "Project Home", SurfacePageKind::Main),
            page_with_commands(
                "spec-workbench",
                "Spec Workbench",
                SurfacePageKind::Workbench,
                &["spec.intake.start"],
            ),
            page_with_commands(
                "task-workbench",
                "Task Workbench",
                SurfacePageKind::Workbench,
                &["work.issue.start"],
            ),
            page_with_commands(
                "acceptance",
                "Acceptance",
                SurfacePageKind::Workbench,
                &["acceptance.evaluate"],
            ),
            page_with_commands(
                "delivery",
                "Delivery",
                SurfacePageKind::Detail,
                &["delivery.open"],
            ),
            page("event-timeline", "Event Timeline", SurfacePageKind::Detail),
            page("evidence-graph", "Evidence Graph", SurfacePageKind::Detail),
            page_with_commands(
                "audit-surface",
                "Audit Surface",
                SurfacePageKind::Sidecar,
                &["audit.request.sidecar"],
            ),
            page("finding-review", "Finding Review", SurfacePageKind::Sidecar),
            page(
                "follow-up-proposal",
                "Follow-up Proposal",
                SurfacePageKind::Sidecar,
            ),
        ],
        workbenches: vec![
            workbench("project-home", "Project"),
            workbench("spec-workbench", "Spec"),
            workbench("task-workbench", "Issue"),
            workbench("acceptance", "Acceptance"),
        ],
        view_model_mappings: vec![
            view_model("project-home", "projection.project-home"),
            view_model("spec-workbench", "projection.spec-workbench"),
            view_model("task-workbench", "projection.task-workbench"),
            view_model("acceptance", "projection.acceptance"),
            view_model("delivery", "projection.delivery"),
            view_model("event-timeline", "projection.event-timeline"),
            view_model("evidence-graph", "projection.evidence-graph"),
            view_model("audit-surface", "projection.audit-surface"),
        ],
        command_entry_mappings: vec![
            command(
                "spec.intake.start",
                "spec-workbench",
                "action-contract:spec.intake",
            ),
            command(
                "work.issue.start",
                "task-workbench",
                "action-contract:issue.start",
            ),
            command(
                "acceptance.evaluate",
                "acceptance",
                "action-contract:acceptance.evaluate",
            ),
            command("delivery.open", "delivery", "action-contract:delivery.open"),
            command(
                "audit.request.sidecar",
                "audit-surface",
                "action-contract:audit.request",
            ),
        ],
        read_model_dependencies: vec![
            read_dependency("project-home", "projection.project-home"),
            read_dependency("spec-workbench", "projection.spec-workbench"),
            read_dependency("task-workbench", "projection.task-workbench"),
            read_dependency("acceptance", "projection.acceptance"),
            read_dependency("delivery", "projection.delivery"),
            read_dependency("event-timeline", "projection.event-timeline"),
            read_dependency("evidence-graph", "projection.evidence-graph"),
            read_dependency("audit-surface", "projection.audit-surface"),
        ],
        navigation_rules: vec![
            navigation("project-home", "spec-workbench", "start-spec"),
            navigation("spec-workbench", "task-workbench", "materialize-issues"),
            navigation("task-workbench", "acceptance", "prepare-review"),
            navigation("acceptance", "delivery", "completion-accepted"),
            navigation("delivery", "audit-surface", "request-sidecar-audit"),
        ],
        state_policy: state_policy(),
        sidecar_surfaces: vec![
            sidecar("audit-surface", "Audit Surface"),
            sidecar("finding-review", "Finding Review"),
            sidecar("follow-up-proposal", "Follow-up Proposal"),
        ],
        writes_authority: false,
    }
}

pub fn ui_design_surface_definition() -> PackSurfaceDefinition {
    PackSurfaceDefinition {
        version: PACK_SURFACE_VERSION.to_string(),
        pack_id: "ui-design".to_string(),
        surface_id: "ui-design-surface".to_string(),
        pages: vec![
            page("design-home", "Design Home", SurfacePageKind::Main),
            page("brief-intake", "Brief Intake", SurfacePageKind::Workbench),
            page(
                "direction-board",
                "Direction Board",
                SurfacePageKind::Workbench,
            ),
            page(
                "wireframe-board",
                "Wireframe Board",
                SurfacePageKind::Workbench,
            ),
            page("hifi-review", "HiFi Review", SurfacePageKind::Workbench),
            page("design-system", "Design System", SurfacePageKind::Detail),
            page(
                "handoff-surface",
                "Handoff Surface",
                SurfacePageKind::Detail,
            ),
        ],
        workbenches: vec![
            workbench("design-home", "ProductBrief"),
            workbench("brief-intake", "ProductBrief"),
            workbench("direction-board", "Direction"),
            workbench("wireframe-board", "Wireframe"),
            workbench("hifi-review", "HiFi"),
        ],
        view_model_mappings: vec![
            view_model("design-home", "projection.design-home"),
            view_model("brief-intake", "projection.brief-intake"),
            view_model("direction-board", "projection.direction-board"),
            view_model("wireframe-board", "projection.wireframe-board"),
            view_model("hifi-review", "projection.hifi-review"),
            view_model("design-system", "projection.design-system"),
            view_model("handoff-surface", "projection.handoff-surface"),
        ],
        command_entry_mappings: vec![
            command(
                "design.brief.capture",
                "brief-intake",
                "action-contract:design.brief.capture",
            ),
            command(
                "design.direction.select",
                "direction-board",
                "action-contract:design.direction.select",
            ),
            command(
                "design.wireframe.generate",
                "wireframe-board",
                "action-contract:design.generate-wireframe",
            ),
            command(
                "design.hifi.review",
                "hifi-review",
                "action-contract:design.hifi.review",
            ),
            command(
                "design.handoff.accept",
                "handoff-surface",
                "action-contract:design.accept-handoff",
            ),
        ],
        read_model_dependencies: vec![
            read_dependency("design-home", "projection.design-home"),
            read_dependency("brief-intake", "projection.brief-intake"),
            read_dependency("direction-board", "projection.direction-board"),
            read_dependency("wireframe-board", "projection.wireframe-board"),
            read_dependency("hifi-review", "projection.hifi-review"),
            read_dependency("design-system", "projection.design-system"),
            read_dependency("handoff-surface", "projection.handoff-surface"),
        ],
        navigation_rules: vec![
            navigation("design-home", "brief-intake", "start-brief"),
            navigation("brief-intake", "direction-board", "brief-confirmed"),
            navigation("direction-board", "wireframe-board", "direction-selected"),
            navigation("wireframe-board", "hifi-review", "wireframe-approved"),
            navigation("hifi-review", "handoff-surface", "hifi-accepted"),
        ],
        state_policy: state_policy(),
        sidecar_surfaces: Vec::new(),
        writes_authority: false,
    }
}

fn page(page_id: &str, label: &str, kind: SurfacePageKind) -> SurfacePage {
    page_with_commands(page_id, label, kind, &[])
}

fn page_with_commands(
    page_id: &str,
    label: &str,
    kind: SurfacePageKind,
    command_entry_ids: &[&str],
) -> SurfacePage {
    SurfacePage {
        page_id: page_id.to_string(),
        label: label.to_string(),
        description: format!("{label} surface"),
        kind,
        view_model_ref: format!("view-model:{page_id}"),
        command_entry_ids: command_entry_ids
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
    }
}

fn workbench(page_id: &str, primary_object_type: &str) -> SurfaceWorkbench {
    SurfaceWorkbench {
        workbench_id: format!("{page_id}-workbench"),
        page_id: page_id.to_string(),
        label: page_id.to_string(),
        primary_object_type: primary_object_type.to_string(),
        timeline_ref: format!("timeline:{page_id}"),
    }
}

fn view_model(page_id: &str, projection_ref: &str) -> SurfaceViewModelMapping {
    SurfaceViewModelMapping {
        mapping_id: format!("{page_id}-view-model"),
        page_id: page_id.to_string(),
        projection_ref: projection_ref.to_string(),
        view_model_ref: format!("view-model:{page_id}"),
    }
}

fn command(
    command_entry_id: &str,
    page_id: &str,
    action_contract_ref: &str,
) -> SurfaceCommandEntryMapping {
    SurfaceCommandEntryMapping {
        command_entry_id: command_entry_id.to_string(),
        page_id: page_id.to_string(),
        label: command_entry_id.to_string(),
        command_type: command_entry_id.to_string(),
        route: SurfaceCommandRoute::ActionProposal,
        action_contract_ref: action_contract_ref.to_string(),
    }
}

fn read_dependency(page_id: &str, projection_ref: &str) -> SurfaceReadModelDependency {
    SurfaceReadModelDependency {
        dependency_id: format!("{page_id}-read-model"),
        page_id: page_id.to_string(),
        projection_ref: projection_ref.to_string(),
        required: true,
    }
}

fn navigation(from_page_id: &str, to_page_id: &str, trigger: &str) -> SurfaceNavigationRule {
    SurfaceNavigationRule {
        from_page_id: from_page_id.to_string(),
        to_page_id: to_page_id.to_string(),
        trigger: trigger.to_string(),
    }
}

fn state_policy() -> SurfaceStatePolicy {
    SurfaceStatePolicy {
        empty_state_ref: "surface-state.empty".to_string(),
        loading_state_ref: "surface-state.loading".to_string(),
        error_state_ref: "surface-state.error".to_string(),
    }
}

fn sidecar(page_id: &str, label: &str) -> SurfaceSidecar {
    SurfaceSidecar {
        sidecar_id: page_id.to_string(),
        page_id: page_id.to_string(),
        label: label.to_string(),
        blocks_main_chain: false,
    }
}

fn require_non_empty(issues: &mut Vec<PackSurfaceValidationIssue>, field: &str, value: &str) {
    if value.trim().is_empty() {
        issues.push(issue(field, "must not be empty"));
    }
}

fn issue(field: &str, reason: &str) -> PackSurfaceValidationIssue {
    PackSurfaceValidationIssue {
        field: field.to_string(),
        reason: reason.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        software_dev_surface_definition, ui_design_surface_definition, validate_surface_definition,
        SurfacePageKind,
    };

    #[test]
    fn surface_pack_cannot_write_authority() {
        let mut surface = software_dev_surface_definition();
        surface.writes_authority = true;

        let report = validate_surface_definition(&surface);

        assert!(!report.valid);
        assert!(report
            .issues
            .iter()
            .any(|issue| issue.field == "writesAuthority"));
    }

    #[test]
    fn software_dev_audit_surface_is_sidecar_only() {
        let surface = software_dev_surface_definition();
        let report = validate_surface_definition(&surface);

        assert!(report.valid);
        let audit_page = surface
            .pages
            .iter()
            .find(|page| page.page_id == "audit-surface")
            .unwrap();
        assert_eq!(audit_page.kind, SurfacePageKind::Sidecar);
        assert!(surface
            .sidecar_surfaces
            .iter()
            .any(|sidecar| sidecar.page_id == "audit-surface" && !sidecar.blocks_main_chain));
        assert!(audit_page
            .command_entry_ids
            .contains(&"audit.request.sidecar".to_string()));
    }

    #[test]
    fn software_dev_pages_expose_main_chain_command_entries() {
        let surface = software_dev_surface_definition();

        assert!(validate_surface_definition(&surface).valid);
        let task_page = surface
            .pages
            .iter()
            .find(|page| page.page_id == "task-workbench")
            .unwrap();
        let acceptance_page = surface
            .pages
            .iter()
            .find(|page| page.page_id == "acceptance")
            .unwrap();
        let delivery_page = surface
            .pages
            .iter()
            .find(|page| page.page_id == "delivery")
            .unwrap();

        assert!(task_page
            .command_entry_ids
            .contains(&"work.issue.start".to_string()));
        assert!(acceptance_page
            .command_entry_ids
            .contains(&"acceptance.evaluate".to_string()));
        assert!(delivery_page
            .command_entry_ids
            .contains(&"delivery.open".to_string()));
    }

    #[test]
    fn ui_design_surface_is_not_task_workbench_disguised_as_design() {
        let surface = ui_design_surface_definition();

        assert!(validate_surface_definition(&surface).valid);
        assert!(surface
            .pages
            .iter()
            .any(|page| page.page_id == "wireframe-board"));
        assert!(surface
            .pages
            .iter()
            .any(|page| page.page_id == "hifi-review"));
        assert!(!surface
            .pages
            .iter()
            .any(|page| page.page_id == "task-workbench"));
    }

    #[test]
    fn surface_commands_route_to_runtime_contracts() {
        let surface = software_dev_surface_definition();

        for command in surface.command_entry_mappings {
            assert!(command.action_contract_ref.starts_with("action-contract:"));
        }
    }
}
