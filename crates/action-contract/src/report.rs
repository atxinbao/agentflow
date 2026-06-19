use serde::{Deserialize, Serialize};

use crate::model::ActionProposal;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionContractValidationError {
    pub code: String,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionContractValidationReport {
    pub valid: bool,
    #[serde(default)]
    pub errors: Vec<ActionContractValidationError>,
}

impl ActionContractValidationReport {
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
        self.errors.push(ActionContractValidationError {
            code: code.into(),
            message: message.into(),
            path: path.into(),
        });
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ActionProposalValidationStatus {
    Valid,
    Invalid,
    Unsupported,
    VersionMismatch,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionProposalValidationReport {
    pub proposal_id: String,
    pub action_type: String,
    pub contract_version: String,
    pub status: ActionProposalValidationStatus,
    #[serde(default)]
    pub errors: Vec<ActionContractValidationError>,
    #[serde(default)]
    pub warnings: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub normalized_proposal: Option<ActionProposal>,
}

impl ActionProposalValidationReport {
    pub fn valid(proposal: ActionProposal) -> Self {
        Self {
            proposal_id: proposal.proposal_id.clone(),
            action_type: proposal.action_type.clone(),
            contract_version: proposal.contract_version.clone(),
            status: ActionProposalValidationStatus::Valid,
            errors: Vec::new(),
            warnings: Vec::new(),
            normalized_proposal: Some(proposal),
        }
    }

    pub fn invalid(proposal: &ActionProposal, status: ActionProposalValidationStatus) -> Self {
        Self {
            proposal_id: proposal.proposal_id.clone(),
            action_type: proposal.action_type.clone(),
            contract_version: proposal.contract_version.clone(),
            status,
            errors: Vec::new(),
            warnings: Vec::new(),
            normalized_proposal: None,
        }
    }

    pub fn push_error(
        &mut self,
        code: impl Into<String>,
        message: impl Into<String>,
        path: impl Into<Option<String>>,
    ) {
        self.errors.push(ActionContractValidationError {
            code: code.into(),
            message: message.into(),
            path: path.into(),
        });
        if matches!(self.status, ActionProposalValidationStatus::Valid) {
            self.status = ActionProposalValidationStatus::Invalid;
        }
        self.normalized_proposal = None;
    }
}
