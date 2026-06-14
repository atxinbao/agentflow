use crate::model::IssueLoopStage;
use std::path::Path;

pub(crate) fn derived_review_substate(
    root: &Path,
    issue_id: &str,
    stage: &IssueLoopStage,
) -> Option<String> {
    match stage {
        IssueLoopStage::InReview => projection_review_substate(root, issue_id),
        IssueLoopStage::Done => Some("merged".to_string()),
        _ => None,
    }
}

fn projection_review_substate(root: &Path, issue_id: &str) -> Option<String> {
    let projection = agentflow_projection::load_task_projection(root, issue_id).ok()?;
    if non_empty(projection.public_delivery.changelog_path.as_deref())
        || non_empty(projection.public_delivery.release_notes_url.as_deref())
    {
        return Some("public-delivery-recorded".to_string());
    }
    if non_empty(projection.public_delivery.pr_url.as_deref()) {
        return Some("review-requested".to_string());
    }
    let evidence_path = projection.public_delivery.evidence_path.as_deref()?;
    if non_empty(Some(evidence_path)) && root.join(evidence_path).is_file() {
        return Some("evidence-prepared".to_string());
    }
    None
}

fn non_empty(value: Option<&str>) -> bool {
    value.is_some_and(|value| !value.trim().is_empty())
}
