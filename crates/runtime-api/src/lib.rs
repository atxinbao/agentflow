//! AgentFlow Runtime Command / Query API boundary.
//!
//! 该 crate 负责对外暴露 Runtime 的正式入口：写侧统一进入 command
//! request / proposal / arbitration，读侧统一返回 projection query
//! surface。Desktop 与 CLI 可以在迁移期通过这里调用现有 formal
//! runtime wrappers，但不应该继续直接依赖底层 spec / release / projection
//! 写读实现。

pub mod commands;
pub mod errors;
pub mod formal;
pub mod handoff;
pub mod mapping;
pub mod query;
pub mod responses;
pub mod work_proposals;

pub use commands::{
    execute_command_via_arbitration, execute_command_via_arbitration_with_context,
    validate_runtime_command, RuntimeCommandRequest,
};
pub use errors::{RuntimeCommandError, RuntimeCommandErrorCode};
pub use formal::{
    audit_request_human, completion_decide, completion_inspect, project_confirm_goal,
    project_confirm_plan, project_intake, project_materialize, project_preview_goal,
    release_confirm, release_prepare, release_publish, release_record_remote, release_record_tag,
    ProjectMaterializationResult,
};
pub use handoff::{write_work_command_handoff_from_spec_issue, WorkCommandHandoff};
pub use mapping::{map_command_to_action_proposal, RuntimeQueryHint};
pub use query::{
    get_audit_surface_view, get_delivery_package_view, get_project_home_view,
    get_requirement_intake_view, get_runtime_health_view, get_spec_loop_view,
    get_spec_preview_view, get_task_workbench_view, RUNTIME_QUERY_API_VERSION,
};
pub use responses::{
    RuntimeCommandDecision, RuntimeCommandResponse, RuntimeCommandStatus,
    RuntimeCommandValidationReport, RUNTIME_COMMAND_API_VERSION,
};
pub use work_proposals::{
    write_work_action_proposals_from_spec_issue, WorkActionProposalContract,
    WorkActionProposalEntry, WorkProposalStageAction, WORK_ACTION_PROPOSAL_CONTRACT_VERSION,
};
