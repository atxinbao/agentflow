//! AgentFlow Runtime Command / Query API boundary.
//!
//! 该 crate 负责对外暴露 Runtime 的正式入口：写侧统一进入 command
//! request / proposal / arbitration，读侧统一返回 projection query
//! surface。Desktop 与 CLI 可以在迁移期通过这里调用现有 formal
//! runtime wrappers，但不应该继续直接依赖底层 spec / release / projection
//! 写读实现。

pub mod api_plane;
pub mod commands;
pub mod errors;
pub mod formal;
pub mod handoff;
pub mod mapping;
pub mod pack;
pub mod query;
pub mod responses;
pub mod work_proposals;
pub mod work_state;

pub use api_plane::{
    api_plane_manifest, write_api_plane_manifest, ApiPlaneAccess, ApiPlaneBoundary, ApiPlaneEntry,
    ApiPlaneManifest, API_PLANE_MANIFEST_VERSION,
};
pub use commands::{
    execute_command_via_arbitration, execute_command_via_arbitration_with_context,
    validate_runtime_command, RuntimeCommandRequest, RuntimeCommandRoute, RuntimeEvidencePolicyRef,
    RuntimeExpectedOutputRef,
};
pub use errors::{RuntimeCommandError, RuntimeCommandErrorCode};
pub use formal::{
    audit_request_human, completion_decide, completion_inspect, project_confirm_goal,
    project_confirm_plan, project_intake, project_materialize, project_preview_goal,
    release_confirm, release_prepare, release_publish, release_record_remote, release_record_tag,
    ProjectMaterializationResult,
};
pub use handoff::{write_work_command_handoff_from_spec_issue, WorkCommandHandoff};
pub use mapping::{
    action_contract_ref_for_action_type, action_type_for_action_contract_ref, core_runtime_route,
    map_command_to_action_proposal, pack_runtime_route, RuntimeQueryHint,
    CORE_RUNTIME_COMMAND_TYPE,
};
pub use pack::{
    dry_run_pack_command, get_pack_registry, get_pack_validation_artifact, list_pack_commands,
    pack_registry_read_receipt, pack_validation_artifact_read_receipt,
    query_pack_capability_status, query_pack_surface_route, submit_pack_action_proposal,
    validate_pack_command, PackCapabilityStatusView, PackCommandDryRunReport, PackCommandEntryView,
    PackCommandListView, PackCommandRequest, PackCommandValidationReport, PackRegistryReadReceipt,
    PackRegistryView, PackSurfaceRouteView, PackValidationArtifactReadReceipt,
    PackValidationArtifactView, PACK_COMMAND_SURFACE_VERSION,
};
pub use query::{
    get_audit_surface_view, get_delivery_package_view, get_pack_industry_workbench_view,
    get_project_home_view, get_requirement_intake_view, get_runtime_health_view,
    get_spec_loop_view, get_spec_preview_view, get_task_workbench_view, get_work_loop_run_view,
    get_work_loop_session_view, RUNTIME_QUERY_API_VERSION,
};
pub use responses::{
    RuntimeCommandDecision, RuntimeCommandResponse, RuntimeCommandStatus,
    RuntimeCommandValidationReport, RUNTIME_COMMAND_API_VERSION,
};
pub use work_proposals::{
    write_work_action_proposals_from_spec_issue, WorkActionProposalContract,
    WorkActionProposalEntry, WorkProposalStageAction, WORK_ACTION_PROPOSAL_CONTRACT_VERSION,
};
pub use work_state::{
    assert_issue_activation_allowed, assert_issue_mark_done_allowed,
    assert_issue_start_run_allowed, assert_issue_transition, assert_run_transition,
    issue_surface_state_id, run_surface_state_id,
};
