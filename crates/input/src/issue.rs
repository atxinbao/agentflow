use serde::{ser::SerializeStruct, Deserialize, Serialize, Serializer};

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

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InputIssue {
    pub version: String,
    pub issue_id: String,
    pub issue_model: InputIssueModel,
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
    pub system: InputSystemRecord,
}

impl Serialize for InputIssue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("InputIssue", 19)?;
        state.serialize_field("version", &self.version)?;
        state.serialize_field("issueId", &self.issue_id)?;
        state.serialize_field("issueModel", &self.issue_model)?;
        state.serialize_field("sourceSpecId", &self.source_spec_id)?;
        state.serialize_field("projectId", &self.project_id)?;
        state.serialize_field("title", &self.title)?;
        state.serialize_field("summary", &self.summary)?;
        state.serialize_field("kind", &self.kind)?;
        state.serialize_field("priority", &self.priority)?;
        state.serialize_field("status", &self.status)?;
        state.serialize_field(
            "displayStatus",
            &DisplayStatus::from_input_status(&self.status),
        )?;
        state.serialize_field("riskLevel", &self.risk_level)?;
        state.serialize_field("scope", &self.scope)?;
        state.serialize_field("nonGoals", &self.non_goals)?;
        state.serialize_field("acceptanceCriteria", &self.acceptance_criteria)?;
        state.serialize_field("validationHints", &self.validation_hints)?;
        state.serialize_field("relations", &self.relations)?;
        state.serialize_field("panel", &self.panel)?;
        state.serialize_field("system", &self.system)?;
        state.end()
    }
}

impl Default for InputIssue {
    fn default() -> Self {
        Self {
            version: "input-issue.v1".to_string(),
            issue_id: String::new(),
            issue_model: InputIssueModel::default(),
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
            system: InputSystemRecord::default(),
        }
    }
}
