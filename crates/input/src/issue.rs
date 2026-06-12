use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

pub const AGENT_ROLES_VERSION: &str = "agent-roles.v1";
pub const AGENT_CLAIM_VERSION: &str = "agent-claim.v1";
pub const BUILD_AGENT_EXECUTION_PIPELINE_VERSION: &str = "build-agent-execution-pipeline.v1";
pub const BUILD_AGENT_PIPELINE_STAGE_IDS: [&str; 7] = [
    "issue-preflight",
    "test-design",
    "implement",
    "sandbox-verify",
    "create-pr",
    "merge-pr",
    "writeback-done",
];

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
pub enum InputPriority {
    #[serde(rename = "p0")]
    P0,
    #[serde(rename = "p1", alias = "high")]
    P1,
    #[serde(rename = "p2", alias = "normal")]
    P2,
    #[serde(rename = "p3", alias = "low")]
    P3,
}

impl InputPriority {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::P0 => "p0",
            Self::P1 => "p1",
            Self::P2 => "p2",
            Self::P3 => "p3",
        }
    }
}

impl Default for InputPriority {
    fn default() -> Self {
        Self::P2
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InputIssueStatus {
    #[serde(rename = "backlog")]
    Backlog,
    #[serde(rename = "todo")]
    Todo,
    #[serde(rename = "in_progress")]
    InProgress,
    #[serde(rename = "in_review")]
    InReview,
    #[serde(rename = "done")]
    Done,
    #[serde(rename = "blocked")]
    Blocked,
    #[serde(rename = "cancel")]
    Cancel,
}

impl Default for InputIssueStatus {
    fn default() -> Self {
        Self::Backlog
    }
}

impl InputIssueStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Backlog => "backlog",
            Self::Todo => "todo",
            Self::InProgress => "in_progress",
            Self::InReview => "in_review",
            Self::Done => "done",
            Self::Blocked => "blocked",
            Self::Cancel => "cancel",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DisplayStatus {
    #[serde(rename = "backlog")]
    Backlog,
    #[serde(rename = "todo")]
    Todo,
    #[serde(rename = "in_progress")]
    InProgress,
    #[serde(rename = "in_review")]
    InReview,
    #[serde(rename = "done")]
    Done,
    #[serde(rename = "blocked")]
    Blocked,
    #[serde(rename = "cancel")]
    Cancel,
}

impl DisplayStatus {
    pub fn from_input_status(status: &InputIssueStatus) -> Self {
        match status {
            InputIssueStatus::Backlog => Self::Backlog,
            InputIssueStatus::Todo => Self::Todo,
            InputIssueStatus::InProgress => Self::InProgress,
            InputIssueStatus::InReview => Self::InReview,
            InputIssueStatus::Done => Self::Done,
            InputIssueStatus::Blocked => Self::Blocked,
            InputIssueStatus::Cancel => Self::Cancel,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Backlog => "backlog",
            Self::Todo => "todo",
            Self::InProgress => "in_progress",
            Self::InReview => "in_review",
            Self::Done => "done",
            Self::Blocked => "blocked",
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InputIssueExecutionStage {
    pub stage_id: String,
    pub label: String,
    pub goal: String,
    pub required: bool,
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InputIssueExecutionPipeline {
    #[serde(default = "default_build_agent_execution_pipeline_version")]
    pub version: String,
    #[serde(default)]
    pub agent_role: AgentRole,
    #[serde(default)]
    pub git_providers: Vec<String>,
    #[serde(default)]
    pub stages: Vec<InputIssueExecutionStage>,
    #[serde(default = "default_build_agent_merge_modes")]
    pub merge_modes: Vec<String>,
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
    #[serde(default, alias = "riskLevel")]
    pub execution_risk: InputRiskLevel,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub execution_pipeline: Option<InputIssueExecutionPipeline>,
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
            execution_risk: InputRiskLevel::default(),
            allowed_paths: Vec::new(),
            forbidden_paths: Vec::new(),
            forbidden_actions: Vec::new(),
            scope: Vec::new(),
            non_goals: Vec::new(),
            acceptance_criteria: Vec::new(),
            validation_hints: Vec::new(),
            validation_commands: Vec::new(),
            expected_outputs: BTreeMap::new(),
            execution_pipeline: None,
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
                let Some(pipeline) = self.execution_pipeline.as_ref() else {
                    errors.push(format!(
                        "issue {} is missing executionPipeline",
                        self.issue_id
                    ));
                    return errors;
                };
                if pipeline.version != BUILD_AGENT_EXECUTION_PIPELINE_VERSION {
                    errors.push(format!(
                        "issue {} executionPipeline version must be {}",
                        self.issue_id, BUILD_AGENT_EXECUTION_PIPELINE_VERSION
                    ));
                }
                if pipeline.agent_role != AgentRole::BuildAgent {
                    errors.push(format!(
                        "issue {} executionPipeline agentRole must be build-agent",
                        self.issue_id
                    ));
                }
                for stage_id in BUILD_AGENT_PIPELINE_STAGE_IDS {
                    if !pipeline
                        .stages
                        .iter()
                        .any(|stage| stage.stage_id == stage_id && stage.required)
                    {
                        errors.push(format!(
                            "issue {} executionPipeline is missing required stage {stage_id}",
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
        if !self.issue_id.trim().is_empty() {
            let context_pack_id = self.issue_id.replace('/', "-");
            if self
                .panel
                .context_pack_id
                .as_deref()
                .map(str::trim)
                .unwrap_or_default()
                .is_empty()
            {
                self.panel.context_pack_id = Some(context_pack_id.clone());
            }
            if !self
                .context_pack_path
                .trim()
                .starts_with(".agentflow/panel/context-packs/")
            {
                self.context_pack_path =
                    format!(".agentflow/panel/context-packs/{context_pack_id}.json");
            }
        }
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
        let pipeline_complete = self
            .execution_pipeline
            .as_ref()
            .map(build_agent_execution_pipeline_complete)
            .unwrap_or(false);
        if !pipeline_complete {
            self.execution_pipeline = Some(default_build_agent_execution_pipeline());
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

pub fn default_build_agent_execution_pipeline() -> InputIssueExecutionPipeline {
    InputIssueExecutionPipeline {
        version: default_build_agent_execution_pipeline_version(),
        agent_role: AgentRole::BuildAgent,
        git_providers: Vec::new(),
        merge_modes: default_build_agent_merge_modes(),
        stages: vec![
            InputIssueExecutionStage {
                stage_id: "issue-preflight".to_string(),
                label: "执行前置检测".to_string(),
                goal: "只认当前 AgentFlow input issue。确认 issue 仍在 backlog，依赖已完成、合同完整、Context Pack 可读或可补生成、工作区干净；随后通过 AgentFlow 官方 run loop / runtime preflight 创建当前 run。preflight 通过后先把 issue 切到 todo，再准备进入 in_progress。禁止手写 `.agentflow/**` 只表示不能直接改事实文件，不是禁止调用 AgentFlow 官方命令推进 loop。GitHub/GitLab 不在这个阶段检测。".to_string(),
                required: true,
                evidence: vec![
                    "AgentFlow input issue is the only active task source; executionPipeline is read from that issue contract".to_string(),
                    "no external issue/task/plan/queue/thread/tool state is used as task authority".to_string(),
                    "input issue status is backlog before preflight".to_string(),
                    "blockedBy dependencies are done".to_string(),
                    "Panel Context Pack exists or is generated".to_string(),
                    "current run is created by AgentFlow official runtime entrypoint before source edits".to_string(),
                    "no `.agentflow/**` facts are handwritten; official AgentFlow loop commands are used instead".to_string(),
                    "working tree has no uncommitted user source changes before in_progress".to_string(),
                    "input issue status changed to todo after preflight".to_string(),
                ],
            },
            InputIssueExecutionStage {
                stage_id: "test-design".to_string(),
                label: "测试设计".to_string(),
                goal: "从 SPEC 和当前 issue 推导测试点。能做 TDD 就先补失败测试；不能做 TDD 就记录原因，并给出替代验证方式。".to_string(),
                required: true,
                evidence: vec![
                    "test points derived from SPEC and issue".to_string(),
                    "failing test result or TDD-not-applicable reason".to_string(),
                    "planned sandbox validation commands".to_string(),
                ],
            },
            InputIssueExecutionStage {
                stage_id: "implement".to_string(),
                label: "Agent 执行 issue".to_string(),
                goal: "按测试设计和 issue 合同，在 allowedPaths 内完成代码、配置或测试改动。".to_string(),
                required: true,
                evidence: vec!["git diff --stat".to_string(), "changed-files summary".to_string()],
            },
            InputIssueExecutionStage {
                stage_id: "sandbox-verify".to_string(),
                label: "沙箱验证".to_string(),
                goal: "在本地受控沙箱中运行验证命令，并收集 stdout、stderr、exit code 以及浏览器或截图证据。"
                    .to_string(),
                required: true,
                evidence: vec![
                    "validation command records".to_string(),
                    "browser smoke evidence when applicable".to_string(),
                    "git diff --check".to_string(),
                ],
            },
            InputIssueExecutionStage {
                stage_id: "create-pr".to_string(),
                label: "创建 PR/MR".to_string(),
                goal: "推送任务分支，按 AgentFlow Build Agent PR/MR 模板创建 GitHub PR 或 GitLab MR，并写入任务、范围、验证结果、影响、回滚和 review gate；如果 mergeMode 是 auto-merge-if-eligible，不能停在 Draft PR/MR。".to_string(),
                required: true,
                evidence: vec![
                    "PR/MR URL".to_string(),
                    "AgentFlow Build Agent PR/MR template completed".to_string(),
                    "PR/MR body validation summary".to_string(),
                    "draft or ready state".to_string(),
                ],
            },
            InputIssueExecutionStage {
                stage_id: "merge-pr".to_string(),
                label: "合并 PR/MR".to_string(),
                goal: "默认先走 auto-merge-if-eligible：PR/MR ready 后按 provider 自动合并，并轮询到 merged；如果自动合并条件不满足，就回落到 manual-merge，issue 保持 in_review，等待人合并，再由本地检测确认 merged 后继续。".to_string(),
                required: true,
                evidence: vec![
                    "merge mode".to_string(),
                    "GitHub path: gh pr ready result and gh pr merge --auto result".to_string(),
                    "GitLab path: glab mr update --ready result and glab mr merge --auto-merge result".to_string(),
                    "auto-merge rejection reason when falling back to manual-merge".to_string(),
                    "in_review wait evidence when manual-merge fallback is active".to_string(),
                    "merge commit or merged PR/MR state".to_string(),
                ],
            },
            InputIssueExecutionStage {
                stage_id: "writeback-done".to_string(),
                label: "写回 Done".to_string(),
                goal: "PR/MR 合并后，用预检确认过的新 AgentFlow CLI 调用 build-agent complete，写回 run、evidence、delivery 和任务 Done 状态。".to_string(),
                required: true,
                evidence: vec![
                    "target/release/agentflow build-agent complete --request <completion-request.json> after cargo build --release --bin agentflow"
                        .to_string(),
                    "or target/debug/agentflow build-agent complete --request <completion-request.json>"
                        .to_string(),
                    "issue status done".to_string(),
                ],
            },
        ],
    }
}

fn default_build_agent_execution_pipeline_version() -> String {
    BUILD_AGENT_EXECUTION_PIPELINE_VERSION.to_string()
}

fn default_build_agent_merge_modes() -> Vec<String> {
    vec![
        "auto-merge-if-eligible".to_string(),
        "manual-merge".to_string(),
    ]
}

fn build_agent_execution_pipeline_complete(pipeline: &InputIssueExecutionPipeline) -> bool {
    pipeline.version == BUILD_AGENT_EXECUTION_PIPELINE_VERSION
        && pipeline.agent_role == AgentRole::BuildAgent
        && pipeline.merge_modes.contains(&"manual-merge".to_string())
        && pipeline
            .merge_modes
            .contains(&"auto-merge-if-eligible".to_string())
        && BUILD_AGENT_PIPELINE_STAGE_IDS.iter().all(|stage_id| {
            pipeline
                .stages
                .iter()
                .any(|stage| stage.stage_id == *stage_id && stage.required)
        })
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
