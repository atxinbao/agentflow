use crate::model::{AuditGateKind, AuditGateStatus, LOOP_AUDIT_GATE_VERSION};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectAuditGate {
    pub version: String,
    pub project_id: String,
    pub issue_id: Option<String>,
    pub run_id: Option<String>,
    pub kind: AuditGateKind,
    pub status: AuditGateStatus,
    pub output_dir: Option<String>,
    pub updated_at: u64,
}

impl ProjectAuditGate {
    pub fn delivery(
        project_id: impl Into<String>,
        issue_id: impl Into<String>,
        run_id: impl Into<String>,
        updated_at: u64,
    ) -> Self {
        let run_id = run_id.into();
        Self {
            version: LOOP_AUDIT_GATE_VERSION.to_string(),
            project_id: project_id.into(),
            issue_id: Some(issue_id.into()),
            output_dir: Some(format!(".agentflow/output/audit/delivery-{run_id}")),
            run_id: Some(run_id),
            kind: AuditGateKind::Delivery,
            status: AuditGateStatus::Pending,
            updated_at,
        }
    }

    pub fn project_final(project_id: impl Into<String>, updated_at: u64) -> Self {
        let project_id = project_id.into();
        Self {
            version: LOOP_AUDIT_GATE_VERSION.to_string(),
            issue_id: None,
            run_id: None,
            output_dir: Some(format!(
                ".agentflow/output/audit/project-{project_id}-final"
            )),
            project_id,
            kind: AuditGateKind::ProjectFinal,
            status: AuditGateStatus::Pending,
            updated_at,
        }
    }
}
