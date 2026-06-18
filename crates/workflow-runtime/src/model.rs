use agentflow_event_store::EventActor;
use agentflow_workflow_core::{WorkflowAgentRole, WorkflowHandoffMode, WorkflowSkillPack};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeContext {
    pub aggregate_type: String,
    pub aggregate_id: String,
    pub issue_id: Option<String>,
    pub project_id: Option<String>,
    pub run_id: Option<String>,
    pub actor: EventActor,
    pub correlation_id: Option<String>,
    pub causation_id: Option<String>,
    pub artifact_refs: Vec<String>,
    pub payload: Value,
}

impl RuntimeContext {
    pub fn issue(issue_id: impl Into<String>, actor: EventActor) -> Self {
        let issue_id = issue_id.into();
        Self {
            aggregate_type: "issue".to_string(),
            aggregate_id: issue_id.clone(),
            issue_id: Some(issue_id),
            project_id: None,
            run_id: None,
            actor,
            correlation_id: None,
            causation_id: None,
            artifact_refs: Vec::new(),
            payload: Value::Null,
        }
    }

    pub fn project(project_id: impl Into<String>, actor: EventActor) -> Self {
        let project_id = project_id.into();
        Self {
            aggregate_type: "project".to_string(),
            aggregate_id: project_id.clone(),
            issue_id: None,
            project_id: Some(project_id),
            run_id: None,
            actor,
            correlation_id: None,
            causation_id: None,
            artifact_refs: Vec::new(),
            payload: Value::Null,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GuardCheck {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GuardOutcome {
    pub name: String,
    pub passed: bool,
    pub reason: Option<String>,
}

impl GuardOutcome {
    pub fn passed(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            passed: true,
            reason: None,
        }
    }

    pub fn failed(name: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            passed: false,
            reason: Some(reason.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionExecution {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionOutcome {
    pub name: String,
    pub artifact_refs: Vec<String>,
}

impl ActionOutcome {
    pub fn completed(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            artifact_refs: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeTransition {
    pub transition_id: String,
    pub from_state: String,
    pub to_state: String,
    pub event_type: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeStateBinding {
    pub state_id: String,
    pub role: WorkflowAgentRole,
    pub skill_pack: Option<WorkflowSkillPack>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeHandoffBinding {
    pub transition_id: String,
    pub from_role: WorkflowAgentRole,
    pub to_role: WorkflowAgentRole,
    pub mode: WorkflowHandoffMode,
    pub payload_ref: String,
    pub expected_state: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeTransitionResult {
    pub applied: bool,
    pub transition: Option<RuntimeTransition>,
    pub current_binding: Option<RuntimeStateBinding>,
    pub next_binding: Option<RuntimeStateBinding>,
    pub handoff: Option<RuntimeHandoffBinding>,
    pub guard_outcomes: Vec<GuardOutcome>,
    pub action_outcomes: Vec<ActionOutcome>,
    pub event_id: Option<String>,
    pub blocked_reason: Option<String>,
}
