use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

pub const AGENT_ROLES_VERSION: &str = "agent-roles.v1";
pub const AGENT_CLAIM_VERSION: &str = "agent-claim.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum InputIssueModel {
    Direct,
    Project,
}

impl Default for InputIssueModel {
    fn default() -> Self {
        Self::Direct
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum IssueCategory {
    Spec,
    Audit,
}

impl IssueCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Spec => "spec",
            Self::Audit => "audit",
        }
    }
}

impl Default for IssueCategory {
    fn default() -> Self {
        Self::Spec
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AgentRole {
    SpecAgent,
    BuildAgent,
    AuditAgent,
}

impl AgentRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::SpecAgent => "spec-agent",
            Self::BuildAgent => "build-agent",
            Self::AuditAgent => "audit-agent",
        }
    }

    pub fn label_zh(&self) -> &'static str {
        match self {
            Self::SpecAgent => "需求助手",
            Self::BuildAgent => "执行助手",
            Self::AuditAgent => "审计助手",
        }
    }
}

impl Default for AgentRole {
    fn default() -> Self {
        Self::BuildAgent
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum InputIssueKind {
    Feature,
    Repair,
    DocsOnly,
    Validation,
    Cleanup,
}

impl Default for InputIssueKind {
    fn default() -> Self {
        Self::Feature
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum InputPriority {
    Low,
    Normal,
    High,
}

impl Default for InputPriority {
    fn default() -> Self {
        Self::Normal
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum InputIssueStatus {
    Planned,
    Blocked,
    ReadyForExecute,
    Done,
    Canceled,
}

impl Default for InputIssueStatus {
    fn default() -> Self {
        Self::Planned
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DisplayStatus {
    Backlog,
    Ready,
    InProgress,
    Review,
    Done,
    Cancel,
}

impl DisplayStatus {
    pub fn from_input_status(status: &InputIssueStatus) -> Self {
        match status {
            InputIssueStatus::Planned | InputIssueStatus::Blocked => Self::Backlog,
            InputIssueStatus::ReadyForExecute => Self::Ready,
            InputIssueStatus::Done => Self::Done,
            InputIssueStatus::Canceled => Self::Cancel,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Backlog => "backlog",
            Self::Ready => "ready",
            Self::InProgress => "in-progress",
            Self::Review => "review",
            Self::Done => "done",
            Self::Cancel => "cancel",
        }
    }
}

impl Default for DisplayStatus {
    fn default() -> Self {
        Self::Backlog
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum InputRiskLevel {
    Low,
    Medium,
    High,
}

impl InputRiskLevel {
    pub fn requires_human_confirmation(&self) -> bool {
        matches!(self, Self::High)
    }
}

impl Default for InputRiskLevel {
    fn default() -> Self {
        Self::Medium
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InputIssueRelations {
    pub blocked_by: Vec<String>,
    pub blocks: Vec<String>,
    pub related: Vec<String>,
    pub duplicate_of: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InputPanelLink {
    pub snapshot_id: Option<String>,
    pub context_pack_id: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InputSystemRecord {
    pub created_by: String,
    pub created_at: u64,
    pub updated_at: u64,
    pub path: String,
    pub revision: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InputIssueAudit {
    #[serde(default)]
    pub audit_id: String,
    pub trigger: String,
    pub source_release_id: String,
    pub source_run_id: Option<String>,
    #[serde(default)]
    pub source_delivery_path: String,
    #[serde(default)]
    pub audit_output_dir: String,
    #[serde(default, deserialize_with = "deserialize_expected_outputs")]
    pub expected_outputs: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InputIssue {
    pub version: String,
    pub issue_id: String,
    pub issue_model: InputIssueModel,
    #[serde(default)]
    pub issue_category: IssueCategory,
    #[serde(default)]
    pub required_agent_role: AgentRole,
    pub source_spec_id: String,
    #[serde(default)]
    pub source_spec_path: String,
    #[serde(default)]
    pub issue_path: String,
    #[serde(default)]
    pub handoff_id: String,
    #[serde(default)]
    pub context_pack_path: String,
    pub project_id: Option<String>,
    pub title: String,
    pub summary: String,
    pub kind: InputIssueKind,
    pub priority: InputPriority,
    pub status: InputIssueStatus,
    #[serde(default)]
    pub display_status: DisplayStatus,
    pub risk_level: InputRiskLevel,
    #[serde(default)]
    pub allowed_paths: Vec<String>,
    #[serde(default)]
    pub forbidden_paths: Vec<String>,
    #[serde(default)]
    pub forbidden_actions: Vec<String>,
    pub scope: Vec<String>,
    pub non_goals: Vec<String>,
    pub acceptance_criteria: Vec<String>,
    pub validation_hints: Vec<String>,
    #[serde(default)]
    pub validation_commands: Vec<String>,
    #[serde(default, deserialize_with = "deserialize_expected_outputs")]
    pub expected_outputs: BTreeMap<String, String>,
    pub relations: InputIssueRelations,
    pub panel: InputPanelLink,
    #[serde(default)]
    pub audit: Option<InputIssueAudit>,
    pub system: InputSystemRecord,
}

impl Default for InputIssue {
    fn default() -> Self {
        Self {
            version: "input-issue.v1".to_string(),
            issue_id: String::new(),
            issue_model: InputIssueModel::default(),
            issue_category: IssueCategory::default(),
            required_agent_role: AgentRole::default(),
            source_spec_id: String::new(),
            source_spec_path: String::new(),
            issue_path: String::new(),
            handoff_id: String::new(),
            context_pack_path: String::new(),
            project_id: None,
            title: String::new(),
            summary: String::new(),
            kind: InputIssueKind::default(),
            priority: InputPriority::default(),
            status: InputIssueStatus::default(),
            display_status: DisplayStatus::default(),
            risk_level: InputRiskLevel::default(),
            allowed_paths: Vec::new(),
            forbidden_paths: Vec::new(),
            forbidden_actions: Vec::new(),
            scope: Vec::new(),
            non_goals: Vec::new(),
            acceptance_criteria: Vec::new(),
            validation_hints: Vec::new(),
            validation_commands: Vec::new(),
            expected_outputs: BTreeMap::new(),
            relations: InputIssueRelations::default(),
            panel: InputPanelLink::default(),
            audit: None,
            system: InputSystemRecord::default(),
        }
    }
}

impl InputIssue {
    pub fn normalize_execution_metadata(&mut self) {
        if self.issue_path.trim().is_empty() && !self.issue_id.trim().is_empty() {
            self.issue_path = format!(".agentflow/input/issues/{}.json", self.issue_id);
        }
        if self.handoff_id.trim().is_empty() && !self.issue_id.trim().is_empty() {
            self.handoff_id = format!("handoff-{}", self.issue_id);
        }
        if self.context_pack_path.trim().is_empty() {
            if let Some(context_pack_id) = self.panel.context_pack_id.as_deref() {
                self.context_pack_path =
                    format!(".agentflow/panel/context-packs/{context_pack_id}.json");
            }
        }
        if self.validation_commands.is_empty() {
            self.validation_commands = self.validation_hints.clone();
        }

        match self.issue_category {
            IssueCategory::Spec => self.normalize_spec_metadata(),
            IssueCategory::Audit => self.normalize_audit_metadata(),
        }
    }

    pub fn target_metadata_errors(&self) -> Vec<String> {
        let mut errors = Vec::new();
        let expected_role = required_role_for_issue_category(&self.issue_category);
        if self.required_agent_role != expected_role {
            errors.push(format!(
                "issue {} category {} requires role {}, found {}",
                self.issue_id,
                self.issue_category.as_str(),
                expected_role.as_str(),
                self.required_agent_role.as_str()
            ));
        }

        match self.issue_category {
            IssueCategory::Spec => {
                if self.source_spec_id.trim().is_empty() {
                    errors.push(format!("issue {} is missing sourceSpecId", self.issue_id));
                }
                if self.source_spec_path.trim().is_empty() {
                    errors.push(format!("issue {} is missing sourceSpecPath", self.issue_id));
                }
                if self.issue_path.trim().is_empty() {
                    errors.push(format!("issue {} is missing issuePath", self.issue_id));
                }
                for key in ["executeRunDir", "evidencePath", "releaseDeliveryDir"] {
                    if self
                        .expected_outputs
                        .get(key)
                        .map(|value| value.trim().is_empty())
                        .unwrap_or(true)
                    {
                        errors.push(format!(
                            "issue {} expectedOutputs is missing {key}",
                            self.issue_id
                        ));
                    }
                }
            }
            IssueCategory::Audit => {
                let Some(audit) = self.audit.as_ref() else {
                    errors.push(format!("issue {} is missing audit metadata", self.issue_id));
                    return errors;
                };
                if audit.audit_id.trim().is_empty() {
                    errors.push(format!("issue {} audit is missing auditId", self.issue_id));
                }
                if audit.source_release_id.trim().is_empty() {
                    errors.push(format!(
                        "issue {} audit is missing sourceReleaseId",
                        self.issue_id
                    ));
                }
                if audit.source_delivery_path.trim().is_empty() {
                    errors.push(format!(
                        "issue {} audit is missing sourceDeliveryPath",
                        self.issue_id
                    ));
                }
                if audit.audit_output_dir.trim().is_empty() {
                    errors.push(format!(
                        "issue {} audit is missing auditOutputDir",
                        self.issue_id
                    ));
                }
                for key in [
                    "audit.json",
                    "audit-report.md",
                    "findings.json",
                    "evidence-map.json",
                    "traceability.json",
                ] {
                    if !audit.expected_outputs.contains_key(key) {
                        errors.push(format!(
                            "issue {} audit.expectedOutputs is missing {key}",
                            self.issue_id
                        ));
                    }
                }
            }
        }
        errors
    }

    pub fn target_metadata_complete(&self) -> bool {
        self.target_metadata_errors().is_empty()
    }

    fn normalize_spec_metadata(&mut self) {
        if self.source_spec_path.trim().is_empty() && !self.source_spec_id.trim().is_empty() {
            self.source_spec_path = format!(
                ".agentflow/input/specs/approved/{}/spec.json",
                self.source_spec_id
            );
        }
        if self.allowed_paths.is_empty() {
            self.allowed_paths = self.scope.clone();
        }
        if self.forbidden_paths.is_empty() {
            self.forbidden_paths = vec![
                ".agentflow/output/audit/**".to_string(),
                ".agentflow/spec/**".to_string(),
                ".agentflow/goal-tree/**".to_string(),
                ".agentflow/define/issues/**".to_string(),
                ".agentflow/define/goals/**".to_string(),
                ".agentflow/define/milestones/**".to_string(),
            ];
        }
        if self.forbidden_actions.is_empty() {
            self.forbidden_actions = vec![
                "process-audit-issue".to_string(),
                "write-audit-report".to_string(),
                "write-audit-findings".to_string(),
                "create-remote-pr-without-authorization".to_string(),
                "write-legacy-agentflow-spec-or-goal-tree".to_string(),
            ];
        }
        if self.expected_outputs.is_empty() && !self.issue_id.trim().is_empty() {
            self.expected_outputs = BTreeMap::from([
                (
                    "executeRunDir".to_string(),
                    format!(".agentflow/execute/runs/{}", self.issue_id),
                ),
                (
                    "evidencePath".to_string(),
                    format!(".agentflow/output/evidence/{}.json", self.issue_id),
                ),
                (
                    "releaseDeliveryDir".to_string(),
                    format!(".agentflow/output/release/{}", self.issue_id),
                ),
            ]);
        }
    }

    fn normalize_audit_metadata(&mut self) {
        if self.allowed_paths.is_empty() {
            self.allowed_paths = vec![
                ".agentflow/output/audit/**".to_string(),
                ".agentflow/output/release/**".to_string(),
                ".agentflow/output/evidence/**".to_string(),
                ".agentflow/input/issues/**".to_string(),
                ".agentflow/input/specs/approved/**".to_string(),
            ];
        }
        if self.forbidden_paths.is_empty() {
            self.forbidden_paths = vec![
                ".agentflow/execute/**".to_string(),
                ".agentflow/output/release/**".to_string(),
                ".agentflow/output/evidence/**".to_string(),
                ".agentflow/spec/**".to_string(),
                ".agentflow/goal-tree/**".to_string(),
            ];
        }
        if self.forbidden_actions.is_empty() {
            self.forbidden_actions = vec![
                "process-spec-issue".to_string(),
                "write-source-code".to_string(),
                "execute-project-commands".to_string(),
                "generate-release-delivery".to_string(),
                "write-legacy-agentflow-spec-or-goal-tree".to_string(),
            ];
        }

        if let Some(audit) = self.audit.as_mut() {
            if audit.audit_id.trim().is_empty() && !self.issue_id.trim().is_empty() {
                audit.audit_id = self.issue_id.clone();
            }
            if audit.source_release_id.trim().is_empty() && !self.source_spec_id.trim().is_empty() {
                audit.source_release_id = self.source_spec_id.clone();
            }
            if audit.source_delivery_path.trim().is_empty()
                && !audit.source_release_id.trim().is_empty()
            {
                audit.source_delivery_path = format!(
                    ".agentflow/output/release/{}/delivery.json",
                    audit.source_release_id
                );
            }
            if audit.audit_output_dir.trim().is_empty() && !audit.audit_id.trim().is_empty() {
                audit.audit_output_dir = format!(".agentflow/output/audit/{}", audit.audit_id);
            }
            if audit.expected_outputs.is_empty() && !audit.audit_output_dir.trim().is_empty() {
                audit.expected_outputs = audit_expected_outputs(&audit.audit_output_dir);
            }
        }
    }
}

pub fn audit_expected_outputs(audit_output_dir: &str) -> BTreeMap<String, String> {
    BTreeMap::from([
        (
            "audit.json".to_string(),
            format!("{audit_output_dir}/audit.json"),
        ),
        (
            "audit-report.md".to_string(),
            format!("{audit_output_dir}/audit-report.md"),
        ),
        (
            "findings.json".to_string(),
            format!("{audit_output_dir}/findings.json"),
        ),
        (
            "evidence-map.json".to_string(),
            format!("{audit_output_dir}/evidence-map.json"),
        ),
        (
            "traceability.json".to_string(),
            format!("{audit_output_dir}/traceability.json"),
        ),
    ])
}

fn deserialize_expected_outputs<'de, D>(
    deserializer: D,
) -> Result<BTreeMap<String, String>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<Value>::deserialize(deserializer)?;
    let Some(value) = value else {
        return Ok(BTreeMap::new());
    };

    match value {
        Value::Object(map) => Ok(map
            .into_iter()
            .filter_map(|(key, value)| value.as_str().map(|value| (key, value.to_string())))
            .collect()),
        Value::Array(values) => Ok(values
            .into_iter()
            .filter_map(|value| {
                value.as_str().map(|path| {
                    (
                        path.rsplit('/').next().unwrap_or(path).to_string(),
                        path.to_string(),
                    )
                })
            })
            .collect()),
        _ => Ok(BTreeMap::new()),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentRoleDescriptor {
    pub agent_role: AgentRole,
    pub label: String,
    pub allowed_issue_categories: Vec<IssueCategory>,
    pub allowed_writes: Vec<String>,
    pub forbidden_writes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentRolesDocument {
    pub version: String,
    pub roles: Vec<AgentRoleDescriptor>,
}

impl Default for AgentRolesDocument {
    fn default() -> Self {
        Self {
            version: AGENT_ROLES_VERSION.to_string(),
            roles: vec![
                AgentRoleDescriptor {
                    agent_role: AgentRole::SpecAgent,
                    label: "需求助手".to_string(),
                    allowed_issue_categories: Vec::new(),
                    allowed_writes: vec![
                        ".agentflow/input/intake/**".to_string(),
                        ".agentflow/input/specs/**".to_string(),
                        ".agentflow/input/issues/**".to_string(),
                    ],
                    forbidden_writes: vec![
                        ".agentflow/execute/**".to_string(),
                        ".agentflow/output/release/**".to_string(),
                        ".agentflow/output/audit/**".to_string(),
                        ".agentflow/spec/**".to_string(),
                        ".agentflow/goal-tree/**".to_string(),
                        ".agentflow/define/goals/**".to_string(),
                        ".agentflow/define/milestones/**".to_string(),
                        ".agentflow/define/issues/**".to_string(),
                    ],
                },
                AgentRoleDescriptor {
                    agent_role: AgentRole::BuildAgent,
                    label: "执行助手".to_string(),
                    allowed_issue_categories: vec![IssueCategory::Spec],
                    allowed_writes: vec![
                        ".agentflow/execute/**".to_string(),
                        ".agentflow/output/evidence/**".to_string(),
                        ".agentflow/output/release/**".to_string(),
                        ".agentflow/state/events/**".to_string(),
                    ],
                    forbidden_writes: vec![
                        ".agentflow/output/audit/**".to_string(),
                        ".agentflow/spec/**".to_string(),
                        ".agentflow/goal-tree/**".to_string(),
                        ".agentflow/define/goals/**".to_string(),
                        ".agentflow/define/milestones/**".to_string(),
                        ".agentflow/define/issues/**".to_string(),
                    ],
                },
                AgentRoleDescriptor {
                    agent_role: AgentRole::AuditAgent,
                    label: "审计助手".to_string(),
                    allowed_issue_categories: vec![IssueCategory::Audit],
                    allowed_writes: vec![
                        ".agentflow/output/audit/**".to_string(),
                        ".agentflow/state/events/**".to_string(),
                    ],
                    forbidden_writes: vec![
                        ".agentflow/execute/**".to_string(),
                        ".agentflow/output/evidence/**".to_string(),
                        ".agentflow/output/release/**".to_string(),
                        ".agentflow/spec/**".to_string(),
                        ".agentflow/goal-tree/**".to_string(),
                        ".agentflow/define/goals/**".to_string(),
                        ".agentflow/define/milestones/**".to_string(),
                        ".agentflow/define/issues/**".to_string(),
                    ],
                },
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentClaim {
    pub version: String,
    pub issue_id: String,
    pub issue_category: IssueCategory,
    pub claimed_agent_role: AgentRole,
    pub handoff_id: String,
    pub created_by: String,
}

impl AgentClaim {
    pub fn new(issue: &InputIssue, claimed_agent_role: AgentRole, handoff_id: String) -> Self {
        Self {
            version: AGENT_CLAIM_VERSION.to_string(),
            issue_id: issue.issue_id.clone(),
            issue_category: issue.issue_category.clone(),
            claimed_agent_role: claimed_agent_role.clone(),
            handoff_id,
            created_by: claimed_agent_role.as_str().to_string(),
        }
    }
}

pub fn required_role_for_issue_category(category: &IssueCategory) -> AgentRole {
    match category {
        IssueCategory::Spec => AgentRole::BuildAgent,
        IssueCategory::Audit => AgentRole::AuditAgent,
    }
}

pub fn validate_agent_issue_permission(
    issue: &InputIssue,
    claimed_role: &AgentRole,
) -> anyhow::Result<()> {
    if &issue.required_agent_role != claimed_role {
        anyhow::bail!(
            "Agent role mismatch: issue requires {}, got {}",
            issue.required_agent_role.as_str(),
            claimed_role.as_str()
        );
    }

    match (&issue.issue_category, claimed_role) {
        (IssueCategory::Spec, AgentRole::BuildAgent) => Ok(()),
        (IssueCategory::Audit, AgentRole::AuditAgent) => Ok(()),
        _ => anyhow::bail!(
            "Agent role {} cannot execute {} issue",
            claimed_role.as_str(),
            issue.issue_category.as_str()
        ),
    }
}

pub fn validate_agent_claim(issue: &InputIssue, claim: &AgentClaim) -> anyhow::Result<()> {
    if claim.version != AGENT_CLAIM_VERSION {
        anyhow::bail!("agent claim version mismatch: {}", claim.version);
    }
    if claim.issue_id != issue.issue_id {
        anyhow::bail!(
            "agent claim issue mismatch: issue {}, claim {}",
            issue.issue_id,
            claim.issue_id
        );
    }
    if claim.issue_category != issue.issue_category {
        anyhow::bail!(
            "agent claim issueCategory mismatch: issue {}, claim {}",
            issue.issue_category.as_str(),
            claim.issue_category.as_str()
        );
    }
    validate_agent_issue_permission(issue, &claim.claimed_agent_role)
}

pub fn validate_agent_write_paths(
    role: &AgentRole,
    paths: &[String],
    roles: &AgentRolesDocument,
) -> anyhow::Result<()> {
    let descriptor = roles
        .roles
        .iter()
        .find(|descriptor| descriptor.agent_role == *role)
        .ok_or_else(|| anyhow::anyhow!("missing role descriptor for {}", role.as_str()))?;

    for path in paths {
        if descriptor
            .forbidden_writes
            .iter()
            .any(|pattern| path_matches_role_pattern(path, pattern))
        {
            anyhow::bail!(
                "Agent role {} cannot write forbidden path {}",
                role.as_str(),
                path
            );
        }
        if !descriptor
            .allowed_writes
            .iter()
            .any(|pattern| path_matches_role_pattern(path, pattern))
        {
            anyhow::bail!("Agent role {} cannot write path {}", role.as_str(), path);
        }
    }

    Ok(())
}

pub fn path_matches_role_pattern(path: &str, pattern: &str) -> bool {
    let normalized_path = path.trim_start_matches("./");
    let normalized_pattern = pattern.trim_start_matches("./");
    if let Some(prefix) = normalized_pattern.strip_suffix("/**") {
        return normalized_path == prefix || normalized_path.starts_with(&format!("{prefix}/"));
    }
    normalized_path == normalized_pattern
}
