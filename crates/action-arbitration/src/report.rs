use serde::{Deserialize, Serialize};

use crate::model::ArbitrationDecision;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RejectionExplanation {
    pub summary: String,
    #[serde(default)]
    pub reasons: Vec<String>,
}

impl RejectionExplanation {
    pub fn from_decision(decision: &ArbitrationDecision) -> Self {
        let reasons = decision
            .rejected_reasons
            .iter()
            .map(|reason| match &reason.detail {
                Some(detail) => format!("{:?}: {} ({detail})", reason.code, reason.message),
                None => format!("{:?}: {}", reason.code, reason.message),
            })
            .collect::<Vec<_>>();
        let summary = if reasons.is_empty() {
            "arbitration rejected without details".to_string()
        } else {
            reasons.join("; ")
        };
        Self { summary, reasons }
    }
}
