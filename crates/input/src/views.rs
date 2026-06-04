use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InputView {
    pub version: String,
    pub view_id: String,
    pub title: String,
    pub filter: String,
    pub sort: String,
}

impl InputView {
    pub fn active() -> Self {
        Self::new("active", "Active input issues", "status in planned,blocked")
    }

    pub fn blocked() -> Self {
        Self::new("blocked", "Blocked input issues", "status = blocked")
    }

    pub fn by_spec() -> Self {
        Self::new("by-spec", "Issues by SPEC", "group by sourceSpecId")
    }

    pub fn by_project() -> Self {
        Self::new("by-project", "Issues by project", "group by projectId")
    }

    fn new(view_id: &str, title: &str, filter: &str) -> Self {
        Self {
            version: "input-view.v1".to_string(),
            view_id: view_id.to_string(),
            title: title.to_string(),
            filter: filter.to_string(),
            sort: "issueId asc".to_string(),
        }
    }
}
