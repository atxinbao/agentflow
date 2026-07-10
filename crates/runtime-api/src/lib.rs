//! AgentFlow Runtime Command / Query API boundary.
//!
//! 该 crate 负责对外暴露 Runtime 的正式入口：写侧统一进入 command
//! request / proposal / arbitration，读侧统一返回 projection query
//! surface。Desktop 与 CLI 可以在迁移期通过这里调用现有 formal
//! runtime wrappers，但不应该继续直接依赖底层 spec / release / projection
//! 写读实现。

pub mod api_plane;
pub mod commands;
pub mod commercial;
pub mod errors;
pub mod executor_adapter_real_execution;
pub mod formal;
pub mod handoff;
pub mod mapping;
pub mod pack;
pub mod product_onboarding;
pub mod product_workspace;
pub mod project_sharing;
pub mod query;
pub mod responses;
pub mod role_permission_handoff;
pub mod spec_intake_productization;
pub mod team_delivery_decision_history;
pub mod team_workflow;
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
pub use commercial::{
    admit_paid_report_order_to_run, admit_paid_report_runtime_proposal,
    authorize_paid_report_order, build_commercial_product_read_model,
    build_paid_report_access_receipt, build_paid_report_artifact, build_paid_report_input_snapshot,
    build_paid_report_order_intent, build_paid_report_order_record, build_paid_report_run_contract,
    build_paid_report_run_execution_receipt,
    build_paid_report_runtime_proposal_handoff_from_project,
    build_paid_report_runtime_proposal_handoff_from_registry,
    capture_paid_report_generation_evidence, commercial_authority_boundary,
    commercial_backend_stable_contract, commercial_golden_path,
    commercial_golden_path_from_registry, commercial_negative_fixture_report,
    decide_paid_report_delivery, default_commercial_product_inputs,
    default_commercial_registry_root, evaluate_commercial_product,
    evaluate_paid_report_commercial_policy, evaluate_paid_report_preflight,
    evaluate_paid_report_preflight_from_registry, evaluate_product_sku_extension,
    get_commercial_product_projection_query, get_project_commercial_product_projection_query,
    is_fixture_only_product_id, load_commercial_product_read_model,
    load_project_commercial_product_read_model, load_registry_commercial_product_read_model,
    managed_project_commercial_fixture, negative_commercial_fixture_root,
    paid_report_flow_state_machine, product_sku_extension_contract,
    production_registry_has_fixture_only_products, project_paid_report_customer_delivery_access,
    project_paid_report_delivery_package, project_paid_report_delivery_projection,
    project_paid_report_feedback_loop, resolve_paid_report_product_instance_from_project,
    resolve_paid_report_product_instance_from_registry, CommercialAuthorityBoundary,
    CommercialAuthorityNegativeFixture, CommercialAuthorityRule, CommercialAvailability,
    CommercialBackendDecisionState, CommercialBackendErrorDecisionModel,
    CommercialBackendMigrationPolicy, CommercialBackendStableContract,
    CommercialBackendStableField, CommercialBackendStableObject, CommercialCommandPolicy,
    CommercialDeliveryPromise, CommercialEntitlementFixture, CommercialEntitlementSourceConfig,
    CommercialEntitlementState, CommercialFixtureResult, CommercialFlowType,
    CommercialGoldenPathProof, CommercialNegativeFixtureReport, CommercialPaidFeatureState,
    CommercialProductDefinition, CommercialProductInput, CommercialProductReadModel,
    CommercialProductReadModelEntry, CommercialProductRegistryConfig, CommercialProjectionQuery,
    ManagedProjectCommercialFixture, PaidReportAccessReceipt, PaidReportArtifact,
    PaidReportArtifactSection, PaidReportCommercialPolicyRecord,
    PaidReportCustomerDeliveryAccessProjection, PaidReportDecisionOutcome,
    PaidReportDecisionRecord, PaidReportDeliveryPackageProjection, PaidReportDeliveryProjection,
    PaidReportEntitlementAuthorization, PaidReportEvidencePack, PaidReportFeedbackLoopProjection,
    PaidReportFlowContractBinding, PaidReportFlowFailureReason, PaidReportFlowStateMachine,
    PaidReportFlowTransition, PaidReportFlowTransitionFixture, PaidReportInputSnapshot,
    PaidReportOrderIntent, PaidReportOrderRecord, PaidReportOrderToRunAdmission,
    PaidReportPreflightDecision, PaidReportPreflightRequest, PaidReportPreflightResult,
    PaidReportProductInstanceContract, PaidReportRunContract, PaidReportRunExecutionReceipt,
    PaidReportRuntimeAdmissionReceipt, PaidReportRuntimeProposal, PaidReportRuntimeProposalHandoff,
    ProductSkuExtensionContract, ProductSkuExtensionDefinition, ProductSkuExtensionNegativeFixture,
    ProductSkuExtensionResolution, COMMERCIAL_AUTHORITY_BOUNDARY_VERSION,
    COMMERCIAL_BACKEND_STABLE_CONTRACT_VERSION, COMMERCIAL_ENTITLEMENT_SOURCE_VERSION,
    COMMERCIAL_GOLDEN_PATH_VERSION, COMMERCIAL_NEGATIVE_FIXTURE_VERSION,
    COMMERCIAL_PRODUCT_READ_MODEL_VERSION, COMMERCIAL_PRODUCT_REGISTRY_VERSION,
    COMMERCIAL_PROJECTION_QUERY_VERSION, MANAGED_PROJECT_COMMERCIAL_FIXTURE_VERSION,
    PAID_REPORT_ACCESS_RECEIPT_VERSION, PAID_REPORT_ARTIFACT_VERSION,
    PAID_REPORT_COMMERCIAL_POLICY_VERSION, PAID_REPORT_CUSTOMER_DELIVERY_ACCESS_VERSION,
    PAID_REPORT_DECISION_RECORD_VERSION, PAID_REPORT_DELIVERY_PACKAGE_PROJECTION_VERSION,
    PAID_REPORT_DELIVERY_PROJECTION_VERSION, PAID_REPORT_ENTITLEMENT_AUTHORIZATION_VERSION,
    PAID_REPORT_EVIDENCE_PACK_VERSION, PAID_REPORT_FEEDBACK_LOOP_PROJECTION_VERSION,
    PAID_REPORT_FLOW_STATE_MACHINE_VERSION, PAID_REPORT_INPUT_SNAPSHOT_VERSION,
    PAID_REPORT_ORDER_INTENT_VERSION, PAID_REPORT_ORDER_RECORD_VERSION,
    PAID_REPORT_ORDER_TO_RUN_ADMISSION_VERSION, PAID_REPORT_PREFLIGHT_VERSION,
    PAID_REPORT_PRODUCT_DEFINITION_VERSION, PAID_REPORT_PRODUCT_INSTANCE_VERSION,
    PAID_REPORT_RUNTIME_ADMISSION_RECEIPT_VERSION, PAID_REPORT_RUNTIME_PROPOSAL_HANDOFF_VERSION,
    PAID_REPORT_RUN_CONTRACT_VERSION, PAID_REPORT_RUN_EXECUTION_RECEIPT_VERSION,
    PRODUCT_SKU_EXTENSION_CONTRACT_VERSION,
};
pub use errors::{RuntimeCommandError, RuntimeCommandErrorCode};
pub use executor_adapter_real_execution::{
    capture_executor_evidence, check_executor_diff_boundary, check_executor_workspace_health,
    create_executor_handoff_package, get_executor_flow_read_model, rebuild_executor_projection,
    record_executor_lifecycle, recover_failed_executor_command, resume_executor_run,
    write_executor_result_to_issue, ExecutorActionVisibility, ExecutorCommandEvidenceInput,
    ExecutorCommandRecoveryAction, ExecutorCommandRecoveryReceipt, ExecutorCommandRecoveryRequest,
    ExecutorDecisionProjection, ExecutorDeliveryPackageProjection, ExecutorDiffBoundaryReport,
    ExecutorDiffBoundaryRequest, ExecutorDiffInputFile, ExecutorEvidenceCaptureReport,
    ExecutorEvidenceCaptureRequest, ExecutorEvidenceGraphLink, ExecutorEvidenceGraphNode,
    ExecutorEvidenceGraphProjection, ExecutorFlowReadModel, ExecutorHandoffPackage,
    ExecutorHandoffRequest, ExecutorLifecycleAction, ExecutorLifecycleReceipt,
    ExecutorLifecycleRequest, ExecutorPortableDiagnosticRef, ExecutorProjectionRebuildReceipt,
    ExecutorRecoveryProjection, ExecutorRepairActionProjection, ExecutorResultOutcome,
    ExecutorResultWritebackReport, ExecutorResultWritebackRequest, ExecutorResumeProjection,
    ExecutorRunResumeReceipt, ExecutorRunResumeRequest, ExecutorWorkspaceHealthReport,
    EXECUTOR_COMMAND_RECOVERY_RECEIPT_VERSION, EXECUTOR_DIFF_BOUNDARY_REPORT_VERSION,
    EXECUTOR_EVIDENCE_CAPTURE_VERSION, EXECUTOR_FLOW_READ_MODEL_VERSION,
    EXECUTOR_HANDOFF_PACKAGE_VERSION, EXECUTOR_LIFECYCLE_RECEIPT_VERSION,
    EXECUTOR_PROJECTION_REBUILD_RECEIPT_VERSION, EXECUTOR_RESULT_WRITEBACK_VERSION,
    EXECUTOR_RESUME_RECEIPT_VERSION, EXECUTOR_WORKSPACE_HEALTH_REPORT_VERSION,
};
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
    dry_run_pack_command, dry_run_product_command, get_pack_registry, get_pack_validation_artifact,
    list_pack_commands, list_product_command_surface, pack_registry_read_receipt,
    pack_validation_artifact_read_receipt, query_pack_capability_status, query_pack_surface_route,
    submit_pack_action_proposal, submit_product_command, validate_pack_command,
    PackCapabilityStatusView, PackCommandDryRunReport, PackCommandEntryView, PackCommandListView,
    PackCommandRequest, PackCommandValidationReport, PackRegistryReadReceipt, PackRegistryView,
    PackSurfaceRouteView, PackValidationArtifactReadReceipt, PackValidationArtifactView,
    ProductCommandDryRunReceipt, ProductCommandEvidenceHandoff, ProductCommandState,
    ProductCommandStateLegendEntry, ProductCommandSubmitReceipt, ProductCommandSubmitRequest,
    ProductCommandSubmitResponse, ProductCommandSurfaceActionView,
    ProductCommandSurfaceProductView, ProductCommandSurfaceSummary, ProductCommandSurfaceView,
    PACK_COMMAND_SURFACE_VERSION, PRODUCT_COMMAND_SUBMIT_VERSION, PRODUCT_COMMAND_SURFACE_VERSION,
};
pub use product_onboarding::{
    check_product_onboarding_readiness, first_run_onboarding_contract, guided_sample_run_plan,
    run_guided_sample, ProductFirstRunOnboardingContract, ProductGuidedSampleRunPlan,
    ProductGuidedSampleRunReceipt, ProductGuidedSampleStage, ProductOnboardingReadinessReport,
    ProductOnboardingStateContract, ProductOnboardingStatus, ProductReadinessItem,
    ProductReadinessStatus, PRODUCT_FIRST_RUN_ONBOARDING_CONTRACT_VERSION,
    PRODUCT_GUIDED_SAMPLE_RUN_PLAN_VERSION, PRODUCT_GUIDED_SAMPLE_RUN_RECEIPT_VERSION,
    PRODUCT_ONBOARDING_READINESS_VERSION,
};
pub use product_workspace::{
    create_product_workspace, load_product_workspace_projection, ProductWorkspaceCreationMode,
    ProductWorkspaceCreationReceipt, ProductWorkspaceCreationRequest,
    ProductWorkspaceLocalDiagnostics, ProductWorkspacePathSet, ProductWorkspaceProductBinding,
    ProductWorkspaceProjection, ProductWorkspaceStatus, PRODUCT_WORKSPACE_CONTRACT_VERSION,
    PRODUCT_WORKSPACE_PROJECTION_VERSION,
};
pub use project_sharing::{
    project_sharing_read_model, ProjectSharingField, ProjectSharingReadModel,
    ProjectSharingTaskSummary, PROJECT_SHARING_READ_MODEL_VERSION,
};
pub use query::{
    get_audit_surface_view, get_core_file_backed_ontology_registry_view, get_delivery_package_view,
    get_pack_industry_workbench_view, get_project_home_view, get_requirement_intake_view,
    get_runtime_health_view, get_spec_loop_view, get_spec_preview_view, get_task_workbench_view,
    get_work_loop_run_view, get_work_loop_session_view, RUNTIME_QUERY_API_VERSION,
};
pub use responses::{
    RuntimeCommandDecision, RuntimeCommandResponse, RuntimeCommandStatus,
    RuntimeCommandValidationReport, RUNTIME_COMMAND_API_VERSION,
};
pub use role_permission_handoff::{
    role_permission_handoff_view, RolePermissionHandoffState, RolePermissionHandoffView,
    RolePermissionNegativeFixture, RolePermissionViewRole, ROLE_PERMISSION_HANDOFF_VIEW_VERSION,
};
pub use spec_intake_productization::{
    confirm_product_spec_preview, materialize_confirmed_product_spec, preview_product_intent,
    read_product_spec_confirmation, read_product_spec_preview, ProductCoreRouteDecision,
    ProductIntentIntakeReceipt, ProductIntentIntakeRequest, ProductIntentProductMapping,
    ProductSpecConfirmationRecord, ProductSpecConfirmationRequest,
    ProductSpecMaterializationReport, ProductSpecPreviewArtifact, ProductSpecPreviewDecision,
    ProductTaskPreview, PRODUCT_SPEC_CONFIRMATION_VERSION, PRODUCT_SPEC_INTAKE_VERSION,
    PRODUCT_SPEC_MATERIALIZATION_VERSION, PRODUCT_SPEC_PREVIEW_VERSION,
};
pub use team_delivery_decision_history::{
    team_delivery_decision_history_view, TeamAuditSidecar, TeamDeliveryDecisionHistoryView,
    TeamFeedbackHook, TeamHistoryEntry, TeamHistorySummary, TEAM_DELIVERY_DECISION_HISTORY_VERSION,
};
pub use team_workflow::{
    team_workflow_boundary_contract, TeamWorkflowBoundaryContract, TeamWorkflowCapability,
    TeamWorkflowDeliveryHistoryBoundary, TeamWorkflowFeedbackBoundary, TeamWorkflowHandoffBoundary,
    TeamWorkflowPermissionView, TeamWorkflowRoleBoundary, TEAM_WORKFLOW_BOUNDARY_CONTRACT_VERSION,
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
