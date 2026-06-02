mod ids;
mod integrity;
mod manager;
pub mod model;
mod storage;

pub use integrity::validate_goal_tree;

// Agent-only / system-only write APIs remain exported for future planning flow,
// migrations, and internal tests. Desktop human UI must only call read and
// validation APIs through its Tauri command surface.
pub use manager::{
    archive_goal, archive_issue, archive_milestone, context_snapshot_from_parts, create_goal,
    create_issue, create_milestone, empty_issue_context_snapshot, load_goal_tree_snapshot,
    record_issue_graph_context_path, reorder_goal_tree, update_goal, update_issue,
    update_milestone,
};
pub use model::{
    CreateGoalInput, CreateIssueInput, CreateMilestoneInput, GoalAgentDraft, GoalHumanContract,
    GoalRecord, GoalSystemState, GoalTreeIndex, GoalTreeIssueContextSnapshot,
    GoalTreeRecommendedFile, GoalTreeSnapshot, GoalTreeValidationIssue, GoalTreeValidationSnapshot,
    IssueAgentDraft, IssueHumanContract, IssueRecord, IssueSystemState, MilestoneAgentDraft,
    MilestoneHumanContract, MilestoneRecord, MilestoneSystemState, ReorderGoalTreeInput,
    UpdateGoalInput, UpdateIssueInput, UpdateMilestoneInput,
};
