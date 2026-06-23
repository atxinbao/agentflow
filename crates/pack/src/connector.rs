use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const PACK_CONNECTOR_VERSION: &str = "agentflow-pack-connector.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackConnectorDefinition {
    pub version: String,
    pub pack_id: String,
    pub connector_id: String,
    pub connectors: Vec<PackConnector>,
    pub writes_authority: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PackConnectorProviderType {
    Git,
    Github,
    Gitlab,
    Codex,
    Claude,
    BrowserPreview,
    Figma,
    ImageAssets,
    FrontendRepo,
    DesignExport,
    Custom,
}

impl PackConnectorProviderType {
    pub fn worker_id(&self) -> &'static str {
        match self {
            Self::Git => "git-provider",
            Self::Github => "github",
            Self::Gitlab => "gitlab",
            Self::Codex => "codex",
            Self::Claude => "claude",
            Self::BrowserPreview => "browser-preview",
            Self::Figma | Self::DesignExport => "mcp-connector",
            Self::ImageAssets | Self::FrontendRepo => "git-provider",
            Self::Custom => "mcp-connector",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackConnector {
    pub connector_id: String,
    pub provider_type: PackConnectorProviderType,
    pub supported_actions: Vec<ConnectorSupportedAction>,
    pub required_capabilities: Vec<String>,
    pub health_source: ConnectorHealthSource,
    pub smoke_policy: ConnectorSmokePolicy,
    pub evidence_output: ConnectorEvidenceOutput,
    pub disabled_reason: String,
    pub command_boundary: ConnectorCommandBoundary,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectorSupportedAction {
    pub action_id: String,
    pub label: String,
    pub required_capability: String,
    pub command_type: String,
    pub writes_external: bool,
    pub evidence_output: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConnectorHealthSource {
    CapabilityRegistry,
    ProviderSmoke,
    RuntimePreflight,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectorSmokePolicy {
    pub required_for_commands: bool,
    pub provider_smoke_ref: String,
    pub failure_disables_commands: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectorEvidenceOutput {
    pub channel: String,
    pub path_policy: String,
    pub authority: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectorCommandBoundary {
    pub runtime_command_required: bool,
    pub authority_write: bool,
    pub output_authority: bool,
    #[serde(default)]
    pub output_channels: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackConnectorValidationIssue {
    pub field: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackConnectorValidationReport {
    pub version: String,
    pub pack_id: String,
    pub connector_id: String,
    pub valid: bool,
    pub writes_authority: bool,
    pub issues: Vec<PackConnectorValidationIssue>,
}

pub fn validate_connector_definition(
    definition: &PackConnectorDefinition,
) -> PackConnectorValidationReport {
    let mut issues = Vec::new();
    require_non_empty(&mut issues, "version", &definition.version);
    if definition.version != PACK_CONNECTOR_VERSION {
        issues.push(issue("version", "must be agentflow-pack-connector.v1"));
    }
    require_non_empty(&mut issues, "packId", &definition.pack_id);
    require_non_empty(&mut issues, "connectorId", &definition.connector_id);
    if definition.writes_authority {
        issues.push(issue(
            "writesAuthority",
            "connector pack definitions must not write runtime authority",
        ));
    }
    if definition.connectors.is_empty() {
        issues.push(issue("connectors", "must contain at least one connector"));
    }

    let mut connector_ids = BTreeSet::new();
    for connector in &definition.connectors {
        require_non_empty(
            &mut issues,
            "connectors.connectorId",
            &connector.connector_id,
        );
        if !connector_ids.insert(connector.connector_id.as_str()) {
            issues.push(issue("connectors.connectorId", "must be unique"));
        }
        if connector.supported_actions.is_empty() {
            issues.push(issue(
                "connectors.supportedActions",
                "must contain at least one supported action",
            ));
        }
        if connector.required_capabilities.is_empty() {
            issues.push(issue(
                "connectors.requiredCapabilities",
                "must declare capability requirements",
            ));
        }
        if connector.evidence_output.authority {
            issues.push(issue(
                "connectors.evidenceOutput.authority",
                "connector output must not be runtime authority",
            ));
        }
        if !connector.command_boundary.runtime_command_required {
            issues.push(issue(
                "connectors.commandBoundary.runtimeCommandRequired",
                "external writes must go through Runtime Command Surface",
            ));
        }
        if connector.command_boundary.authority_write {
            issues.push(issue(
                "connectors.commandBoundary.authorityWrite",
                "connector commands must not write AgentFlow authority directly",
            ));
        }
        if connector.command_boundary.output_authority {
            issues.push(issue(
                "connectors.commandBoundary.outputAuthority",
                "connector outputs must not be project facts",
            ));
        }

        let required_capabilities = connector
            .required_capabilities
            .iter()
            .map(String::as_str)
            .collect::<BTreeSet<_>>();
        for action in &connector.supported_actions {
            require_non_empty(
                &mut issues,
                "connectors.supportedActions.actionId",
                &action.action_id,
            );
            require_non_empty(
                &mut issues,
                "connectors.supportedActions.requiredCapability",
                &action.required_capability,
            );
            require_non_empty(
                &mut issues,
                "connectors.supportedActions.commandType",
                &action.command_type,
            );
            if !required_capabilities.contains(action.required_capability.as_str()) {
                issues.push(issue(
                    "connectors.supportedActions.requiredCapability",
                    "must be listed in connector requiredCapabilities",
                ));
            }
            if action.writes_external && !connector.command_boundary.runtime_command_required {
                issues.push(issue(
                    "connectors.supportedActions.writesExternal",
                    "external write actions require Runtime Command Surface",
                ));
            }
        }
    }

    PackConnectorValidationReport {
        version: "agentflow-pack-connector-validation.v1".to_string(),
        pack_id: definition.pack_id.clone(),
        connector_id: definition.connector_id.clone(),
        valid: issues.is_empty(),
        writes_authority: definition.writes_authority,
        issues,
    }
}

pub fn software_dev_connector_definition() -> PackConnectorDefinition {
    PackConnectorDefinition {
        version: PACK_CONNECTOR_VERSION.to_string(),
        pack_id: "software-dev".to_string(),
        connector_id: "software-dev-connectors".to_string(),
        connectors: vec![
            connector(
                "github",
                PackConnectorProviderType::Github,
                &[
                    action("github.repo.read", "repo.read", "github.repo.read", false),
                    action(
                        "github.pull-request.create",
                        "pull_request.create",
                        "github.pull-request.create",
                        true,
                    ),
                ],
            ),
            connector(
                "git",
                PackConnectorProviderType::Git,
                &[
                    action("git.status", "git.status", "git.status", false),
                    action("git.diff", "git.diff", "git.diff", false),
                ],
            ),
            connector(
                "codex",
                PackConnectorProviderType::Codex,
                &[
                    action("codex.launch", "launch", "work-agent.launch", true),
                    action(
                        "codex.complete",
                        "build_agent.complete",
                        "work-agent.complete",
                        true,
                    ),
                ],
            ),
            connector(
                "claude",
                PackConnectorProviderType::Claude,
                &[action("claude.launch", "launch", "work-agent.launch", true)],
            ),
            connector(
                "browser-preview",
                PackConnectorProviderType::BrowserPreview,
                &[action(
                    "browser-preview.smoke",
                    "browser_preview.smoke",
                    "browser-preview.smoke",
                    false,
                )],
            ),
        ],
        writes_authority: false,
    }
}

pub fn ui_design_connector_definition() -> PackConnectorDefinition {
    PackConnectorDefinition {
        version: PACK_CONNECTOR_VERSION.to_string(),
        pack_id: "ui-design".to_string(),
        connector_id: "ui-design-connectors".to_string(),
        connectors: vec![
            connector(
                "figma",
                PackConnectorProviderType::Figma,
                &[action(
                    "figma.read",
                    "mcp.provider.list",
                    "figma.read",
                    false,
                )],
            ),
            connector(
                "image-assets",
                PackConnectorProviderType::ImageAssets,
                &[action("assets.read", "git.status", "assets.read", false)],
            ),
            connector(
                "frontend-repo",
                PackConnectorProviderType::FrontendRepo,
                &[
                    action("frontend.status", "git.status", "frontend.status", false),
                    action("frontend.diff", "git.diff", "frontend.diff", false),
                ],
            ),
            connector(
                "design-export",
                PackConnectorProviderType::DesignExport,
                &[action(
                    "design-export.read",
                    "mcp.provider.list",
                    "design-export.read",
                    false,
                )],
            ),
            connector(
                "browser-preview",
                PackConnectorProviderType::BrowserPreview,
                &[action(
                    "browser-preview.smoke",
                    "browser_preview.smoke",
                    "browser-preview.smoke",
                    false,
                )],
            ),
        ],
        writes_authority: false,
    }
}

fn connector(
    connector_id: &str,
    provider_type: PackConnectorProviderType,
    actions: &[ConnectorSupportedAction],
) -> PackConnector {
    let required_capabilities = actions
        .iter()
        .map(|action| action.required_capability.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    PackConnector {
        connector_id: connector_id.to_string(),
        provider_type,
        supported_actions: actions.to_vec(),
        required_capabilities,
        health_source: ConnectorHealthSource::CapabilityRegistry,
        smoke_policy: ConnectorSmokePolicy {
            required_for_commands: true,
            provider_smoke_ref: format!("provider-smoke:{connector_id}"),
            failure_disables_commands: true,
        },
        evidence_output: ConnectorEvidenceOutput {
            channel: "evidence".to_string(),
            path_policy: "connector-output-is-evidence-not-authority".to_string(),
            authority: false,
        },
        disabled_reason: "capability-registry.disabled-reason".to_string(),
        command_boundary: ConnectorCommandBoundary {
            runtime_command_required: true,
            authority_write: false,
            output_authority: false,
            output_channels: vec!["context".to_string(), "evidence".to_string()],
        },
    }
}

fn action(
    action_id: &str,
    required_capability: &str,
    command_type: &str,
    writes_external: bool,
) -> ConnectorSupportedAction {
    ConnectorSupportedAction {
        action_id: action_id.to_string(),
        label: action_id.to_string(),
        required_capability: required_capability.to_string(),
        command_type: command_type.to_string(),
        writes_external,
        evidence_output: "connector.evidence".to_string(),
    }
}

fn require_non_empty(issues: &mut Vec<PackConnectorValidationIssue>, field: &str, value: &str) {
    if value.trim().is_empty() {
        issues.push(issue(field, "must not be empty"));
    }
}

fn issue(field: &str, reason: &str) -> PackConnectorValidationIssue {
    PackConnectorValidationIssue {
        field: field.to_string(),
        reason: reason.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        software_dev_connector_definition, ui_design_connector_definition,
        validate_connector_definition, PackConnectorProviderType,
    };

    #[test]
    fn connector_pack_cannot_write_authority() {
        let mut definition = software_dev_connector_definition();
        definition.writes_authority = true;

        let report = validate_connector_definition(&definition);

        assert!(!report.valid);
        assert!(report
            .issues
            .iter()
            .any(|issue| issue.field == "writesAuthority"));
    }

    #[test]
    fn external_write_actions_require_runtime_command_boundary() {
        let mut definition = software_dev_connector_definition();
        let connector = definition
            .connectors
            .iter_mut()
            .find(|connector| connector.connector_id == "github")
            .unwrap();
        connector.command_boundary.runtime_command_required = false;

        let report = validate_connector_definition(&definition);

        assert!(!report.valid);
        assert!(report
            .issues
            .iter()
            .any(|issue| { issue.field == "connectors.commandBoundary.runtimeCommandRequired" }));
    }

    #[test]
    fn built_in_connectors_cover_software_and_design_surfaces() {
        let software = software_dev_connector_definition();
        let design = ui_design_connector_definition();

        assert!(validate_connector_definition(&software).valid);
        assert!(validate_connector_definition(&design).valid);
        assert!(software
            .connectors
            .iter()
            .any(|connector| connector.provider_type == PackConnectorProviderType::Github));
        assert!(software
            .connectors
            .iter()
            .any(|connector| connector.provider_type == PackConnectorProviderType::Codex));
        assert!(design
            .connectors
            .iter()
            .any(|connector| connector.provider_type == PackConnectorProviderType::Figma));
        assert!(!design
            .connectors
            .iter()
            .any(|connector| connector.provider_type == PackConnectorProviderType::Codex));
    }
}
