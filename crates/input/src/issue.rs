use serde::{Deserialize, Serialize};

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
    pub trigger: String,
    pub source_release_id: String,
    pub source_run_id: Option<String>,
    pub expected_outputs: Vec<String>,
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
    pub project_id: Option<String>,
    pub title: String,
    pub summary: String,
    pub kind: InputIssueKind,
    pub priority: InputPriority,
    pub status: InputIssueStatus,
    #[serde(default)]
    pub display_status: DisplayStatus,
    pub risk_level: InputRiskLevel,
    pub scope: Vec<String>,
    pub non_goals: Vec<String>,
    pub acceptance_criteria: Vec<String>,
    pub validation_hints: Vec<String>,
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
            project_id: None,
            title: String::new(),
            summary: String::new(),
            kind: InputIssueKind::default(),
            priority: InputPriority::default(),
            status: InputIssueStatus::default(),
            display_status: DisplayStatus::default(),
            risk_level: InputRiskLevel::default(),
            scope: Vec::new(),
            non_goals: Vec::new(),
            acceptance_criteria: Vec::new(),
            validation_hints: Vec::new(),
            relations: InputIssueRelations::default(),
            panel: InputPanelLink::default(),
            audit: None,
            system: InputSystemRecord::default(),
        }
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
                    forbidden_writes: vec![".agentflow/output/audit/**".to_string()],
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
