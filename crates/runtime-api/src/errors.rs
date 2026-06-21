use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RuntimeCommandErrorCode {
    InvalidCommand,
    UnsupportedCommand,
    MissingField,
    MappingFailed,
    ArbitrationRejected,
    ArbitrationQueued,
    ArbitrationSuperseded,
    ArbitrationCancelled,
    HumanDecisionRequired,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeCommandError {
    pub code: RuntimeCommandErrorCode,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

impl RuntimeCommandError {
    pub fn new(
        code: RuntimeCommandErrorCode,
        message: impl Into<String>,
        path: Option<impl Into<String>>,
    ) -> Self {
        Self {
            code,
            message: message.into(),
            path: path.map(|value| value.into()),
        }
    }
}
