use crate::model::ProjectLoopSnapshot;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectLoop {
    project_id: String,
}

impl ProjectLoop {
    pub fn new(project_id: impl Into<String>) -> Self {
        Self {
            project_id: project_id.into(),
        }
    }

    pub fn project_id(&self) -> &str {
        &self.project_id
    }

    pub fn snapshot(&self, updated_at: u64) -> ProjectLoopSnapshot {
        ProjectLoopSnapshot::new(self.project_id.clone(), updated_at)
    }
}
