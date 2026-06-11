use crate::model::IssueLoopProjection;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IssueLoop {
    project_id: String,
    issue_id: String,
}

impl IssueLoop {
    pub fn new(project_id: impl Into<String>, issue_id: impl Into<String>) -> Self {
        Self {
            project_id: project_id.into(),
            issue_id: issue_id.into(),
        }
    }

    pub fn project_id(&self) -> &str {
        &self.project_id
    }

    pub fn issue_id(&self) -> &str {
        &self.issue_id
    }

    pub fn projection(&self, updated_at: u64) -> IssueLoopProjection {
        IssueLoopProjection::new(self.project_id.clone(), self.issue_id.clone(), updated_at)
    }
}
