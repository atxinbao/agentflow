use serde::{Deserialize, Serialize};

use crate::model::RuntimeAgentRole;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RolePolicyValidationError {
    pub code: String,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RolePolicyValidationReport {
    pub valid: bool,
    #[serde(default)]
    pub errors: Vec<RolePolicyValidationError>,
}

impl RolePolicyValidationReport {
    pub fn success() -> Self {
        Self {
            valid: true,
            errors: Vec::new(),
        }
    }

    pub fn push_error(
        &mut self,
        code: impl Into<String>,
        message: impl Into<String>,
        path: impl Into<Option<String>>,
    ) {
        self.valid = false;
        self.errors.push(RolePolicyValidationError {
            code: code.into(),
            message: message.into(),
            path: path.into(),
        });
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoleCapabilityDecision {
    pub allowed: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime_role: Option<RuntimeAgentRole>,
    pub action_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub object_type: Option<String>,
    pub reason: String,
    #[serde(default)]
    pub requires_handoff: bool,
    #[serde(default)]
    pub requires_human_approval: bool,
}

impl RoleCapabilityDecision {
    pub fn denied(
        action_type: impl Into<String>,
        object_type: Option<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            allowed: false,
            runtime_role: None,
            action_type: action_type.into(),
            object_type,
            reason: reason.into(),
            requires_handoff: false,
            requires_human_approval: false,
        }
    }
}
