use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RejectionReasonCode {
    UnknownActionType,
    InvalidActionProposal,
    UnknownActorRole,
    RoleCannotExecuteAction,
    RoleCannotAccessObject,
    UnknownTargetObject,
    InvalidObjectState,
    MissingRequiredEvidence,
    DependencyNotSatisfied,
    ObjectLockUnavailable,
    HumanDecisionMissing,
    DefinitionVersionMismatch,
    ConflictDetected,
    ProposalQueued,
    ProposalSuperseded,
    ProposalCancelled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RejectionReason {
    pub code: RejectionReasonCode,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

impl RejectionReason {
    pub fn new(
        code: RejectionReasonCode,
        message: impl Into<String>,
        detail: impl Into<Option<String>>,
    ) -> Self {
        Self {
            code,
            message: message.into(),
            detail: detail.into(),
        }
    }
}
