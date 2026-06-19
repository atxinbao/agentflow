use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObjectStateValidationError {
    pub code: String,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObjectStateValidationWarning {
    pub code: String,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObjectStateValidationReport {
    pub valid: bool,
    #[serde(default)]
    pub errors: Vec<ObjectStateValidationError>,
    #[serde(default)]
    pub warnings: Vec<ObjectStateValidationWarning>,
}

impl ObjectStateValidationReport {
    pub fn success() -> Self {
        Self {
            valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn push_error(
        &mut self,
        code: impl Into<String>,
        message: impl Into<String>,
        path: impl Into<Option<String>>,
    ) {
        self.valid = false;
        self.errors.push(ObjectStateValidationError {
            code: code.into(),
            message: message.into(),
            path: path.into(),
        });
    }

    pub fn push_warning(
        &mut self,
        code: impl Into<String>,
        message: impl Into<String>,
        path: impl Into<Option<String>>,
    ) {
        self.warnings.push(ObjectStateValidationWarning {
            code: code.into(),
            message: message.into(),
            path: path.into(),
        });
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransitionDecision {
    pub allowed: bool,
    pub object_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requested_state: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved_state: Option<String>,
    pub requested_action_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub matched_action_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_state: Option<String>,
    pub reason: String,
    #[serde(default)]
    pub matched_via_compatibility_alias: bool,
    #[serde(default)]
    pub required_evidence: Vec<String>,
    #[serde(default)]
    pub emitted_events: Vec<String>,
    #[serde(default)]
    pub warnings: Vec<String>,
}

impl TransitionDecision {
    pub fn denied(
        object_type: impl Into<String>,
        requested_state: Option<String>,
        requested_action_type: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            allowed: false,
            object_type: object_type.into(),
            requested_state,
            resolved_state: None,
            requested_action_type: requested_action_type.into(),
            matched_action_type: None,
            next_state: None,
            reason: reason.into(),
            matched_via_compatibility_alias: false,
            required_evidence: Vec::new(),
            emitted_events: Vec::new(),
            warnings: Vec::new(),
        }
    }
}
