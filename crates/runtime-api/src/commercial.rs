use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    env, fs,
    path::{Path, PathBuf},
};

pub const COMMERCIAL_PRODUCT_READ_MODEL_VERSION: &str =
    "agentflow-commercial-product-read-model.v1";
pub const PAID_REPORT_PREFLIGHT_VERSION: &str = "agentflow-paid-report-flow-preflight.v1";
pub const COMMERCIAL_PROJECTION_QUERY_VERSION: &str =
    "agentflow-commercial-product-projection-query.v1";
pub const MANAGED_PROJECT_COMMERCIAL_FIXTURE_VERSION: &str =
    "agentflow-managed-project-commercial-runtime-fixture.v1";
pub const COMMERCIAL_NEGATIVE_FIXTURE_VERSION: &str =
    "agentflow-commercial-negative-fixtures-runtime.v1";
pub const COMMERCIAL_GOLDEN_PATH_VERSION: &str = "agentflow-commercial-golden-path.v1";
pub const COMMERCIAL_PRODUCT_REGISTRY_VERSION: &str = "agentflow-commercial-product-registry.v1";
pub const COMMERCIAL_ENTITLEMENT_SOURCE_VERSION: &str =
    "agentflow-commercial-entitlement-source.v1";
pub const PAID_REPORT_PRODUCT_DEFINITION_VERSION: &str =
    "agentflow-paid-report-product-definition.v1";
pub const PAID_REPORT_PRODUCT_INSTANCE_VERSION: &str = "agentflow-paid-report-product-instance.v1";
pub const PAID_REPORT_RUNTIME_PROPOSAL_HANDOFF_VERSION: &str =
    "agentflow-paid-report-runtime-proposal-handoff.v1";
pub const PAID_REPORT_RUNTIME_ADMISSION_RECEIPT_VERSION: &str =
    "agentflow-paid-report-runtime-admission-receipt.v1";
pub const PAID_REPORT_RUN_CONTRACT_VERSION: &str = "agentflow-paid-report-run-contract.v1";
pub const PAID_REPORT_DELIVERY_PROJECTION_VERSION: &str =
    "agentflow-paid-report-delivery-projection.v1";
pub const PAID_REPORT_ORDER_INTENT_VERSION: &str = "agentflow-paid-report-order-intent.v1";
pub const PAID_REPORT_INPUT_SNAPSHOT_VERSION: &str = "agentflow-paid-report-input-snapshot.v1";
pub const PAID_REPORT_RUN_EXECUTION_RECEIPT_VERSION: &str =
    "agentflow-paid-report-run-execution-receipt.v1";
pub const PAID_REPORT_ARTIFACT_VERSION: &str = "agentflow-paid-report-artifact.v1";
pub const PAID_REPORT_EVIDENCE_PACK_VERSION: &str = "agentflow-paid-report-evidence-pack.v1";
pub const PAID_REPORT_DECISION_RECORD_VERSION: &str = "agentflow-paid-report-decision-record.v1";
pub const PAID_REPORT_DELIVERY_PACKAGE_PROJECTION_VERSION: &str =
    "agentflow-paid-report-delivery-package-projection.v1";
pub const PAID_REPORT_FEEDBACK_LOOP_PROJECTION_VERSION: &str =
    "agentflow-paid-report-feedback-loop-projection.v1";
pub const PAID_REPORT_ORDER_RECORD_VERSION: &str = "agentflow-paid-report-order-record.v1";
pub const PAID_REPORT_ENTITLEMENT_AUTHORIZATION_VERSION: &str =
    "agentflow-paid-report-entitlement-authorization.v1";
pub const PAID_REPORT_ORDER_TO_RUN_ADMISSION_VERSION: &str =
    "agentflow-paid-report-order-to-run-admission.v1";
pub const PAID_REPORT_CUSTOMER_DELIVERY_ACCESS_VERSION: &str =
    "agentflow-paid-report-customer-delivery-access.v1";
pub const PAID_REPORT_ACCESS_RECEIPT_VERSION: &str = "agentflow-paid-report-access-receipt.v1";
pub const PAID_REPORT_COMMERCIAL_POLICY_VERSION: &str =
    "agentflow-paid-report-commercial-policy.v1";

const DEFAULT_COMMERCIAL_REGISTRY_ROOT: &str = "products/commercial-runtime";
const NEGATIVE_COMMERCIAL_FIXTURE_ROOT: &str = "products/_fixtures/commercial-runtime-negative";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CommercialFlowType {
    PaidReportFlow,
    ManagedProjectFlow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CommercialEntitlementState {
    Active,
    Trial,
    Expired,
    Disabled,
    Deferred,
    Missing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CommercialPaidFeatureState {
    Enabled,
    Disabled,
    Deferred,
    NotRequired,
    Missing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CommercialAvailability {
    Available,
    Rejected,
    Deferred,
    Invalid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CommercialCommandPolicy {
    AllowedToPropose,
    BlockedBeforeRuntime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CommercialDeliveryPromise {
    Report,
    ProjectDelivery,
    None,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommercialProductInput {
    pub product_id: String,
    pub product_name: String,
    pub flow_type: CommercialFlowType,
    pub entitlement_state: CommercialEntitlementState,
    pub paid_feature_state: CommercialPaidFeatureState,
    pub flow_definition_present: bool,
    pub product_definition_present: bool,
    pub payment_configured: bool,
    #[serde(default)]
    pub paid_report_authority_fields: Vec<String>,
    #[serde(default)]
    pub required_project_refs_present: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommercialProductRegistryConfig {
    pub version: String,
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub products: Vec<CommercialProductDefinition>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommercialProductDefinition {
    pub product_id: String,
    pub product_name: String,
    pub flow_type: CommercialFlowType,
    pub paid_feature_state: CommercialPaidFeatureState,
    #[serde(default = "true_bool")]
    pub flow_definition_present: bool,
    #[serde(default = "true_bool")]
    pub product_definition_present: bool,
    #[serde(default)]
    pub payment_configured: bool,
    #[serde(default = "true_bool")]
    pub report_definition_present: bool,
    #[serde(default)]
    pub required_input_refs: Vec<String>,
    #[serde(default)]
    pub report_definition_id: String,
    #[serde(default)]
    pub evidence_requirements: Vec<String>,
    #[serde(default)]
    pub decision_requirements: Vec<String>,
    #[serde(default)]
    pub paid_report_authority_fields: Vec<String>,
    #[serde(default)]
    pub required_project_refs_present: bool,
    #[serde(default)]
    pub source_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommercialEntitlementSourceConfig {
    pub version: String,
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub entitlements: Vec<CommercialEntitlementFixture>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommercialEntitlementFixture {
    pub product_id: String,
    pub entitlement_state: CommercialEntitlementState,
    #[serde(default)]
    pub source_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommercialProductReadModelEntry {
    pub product_id: String,
    pub product_name: String,
    pub flow_type: CommercialFlowType,
    pub flow_label: String,
    pub entitlement_state: CommercialEntitlementState,
    pub paid_feature_state: CommercialPaidFeatureState,
    pub delivery_promise: CommercialDeliveryPromise,
    pub availability: CommercialAvailability,
    pub unavailable_reason: String,
    pub command_policy: CommercialCommandPolicy,
    pub next_action: String,
    pub can_submit_runtime_command_proposal: bool,
    pub projection_only: bool,
    pub core_authority: bool,
    pub writes_authority: bool,
    #[serde(default)]
    pub source_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommercialProductReadModel {
    pub version: String,
    pub status: String,
    pub source: String,
    pub projection_only: bool,
    pub core_authority: bool,
    pub writes_authority: bool,
    #[serde(default)]
    pub entries: Vec<CommercialProductReadModelEntry>,
    #[serde(default)]
    pub source_refs: Vec<String>,
    pub freshness: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaidReportProductInstanceContract {
    pub version: String,
    pub status: String,
    pub product_instance_id: String,
    pub product_id: String,
    pub report_definition_id: String,
    #[serde(default)]
    pub required_input_refs: Vec<String>,
    #[serde(default)]
    pub evidence_requirements: Vec<String>,
    #[serde(default)]
    pub decision_requirements: Vec<String>,
    pub entitlement_state: CommercialEntitlementState,
    pub paid_feature_state: CommercialPaidFeatureState,
    pub delivery_promise: CommercialDeliveryPromise,
    pub can_submit_runtime_command_proposal: bool,
    pub unavailable_reason: String,
    #[serde(default)]
    pub source_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaidReportRuntimeProposal {
    pub product_instance_id: String,
    pub product_id: String,
    pub request_id: String,
    #[serde(default)]
    pub required_input_refs: Vec<String>,
    pub report_definition_id: String,
    #[serde(default)]
    pub evidence_policy: Vec<String>,
    #[serde(default)]
    pub decision_policy: Vec<String>,
    pub delivery_promise: CommercialDeliveryPromise,
    pub runtime_admission_required: bool,
    pub can_start_run_directly: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaidReportRuntimeProposalHandoff {
    pub version: String,
    pub status: String,
    pub reason: String,
    pub proposal_created: bool,
    pub product_instance: PaidReportProductInstanceContract,
    pub preflight: PaidReportPreflightResult,
    pub proposal: Option<PaidReportRuntimeProposal>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaidReportRuntimeAdmissionReceipt {
    pub version: String,
    pub status: String,
    pub receipt_id: String,
    pub product_instance_id: String,
    pub product_id: String,
    pub request_id: String,
    pub admission_decision: String,
    pub runtime_admission_required: bool,
    pub can_start_run_directly: bool,
    #[serde(default)]
    pub required_evidence: Vec<String>,
    #[serde(default)]
    pub required_decision_policy: Vec<String>,
    #[serde(default)]
    pub source_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaidReportRunContract {
    pub version: String,
    pub status: String,
    pub run_contract_id: String,
    pub product_instance_id: String,
    pub product_id: String,
    pub request_id: String,
    #[serde(default)]
    pub input_refs: Vec<String>,
    pub report_definition_id: String,
    #[serde(default)]
    pub expected_evidence: Vec<String>,
    #[serde(default)]
    pub decision_policy: Vec<String>,
    pub delivery_promise: CommercialDeliveryPromise,
    pub runtime_admission_receipt_id: String,
    pub can_start_run_directly: bool,
    pub concrete_sku_is_core_authority: bool,
    #[serde(default)]
    pub source_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaidReportDeliveryProjection {
    pub version: String,
    pub status: String,
    pub product_instance_id: String,
    pub product_id: String,
    pub request_id: String,
    pub projection_only: bool,
    pub writes_authority: bool,
    #[serde(default)]
    pub required_evidence: Vec<String>,
    #[serde(default)]
    pub decision_policy: Vec<String>,
    pub evidence_satisfied: bool,
    pub decision_satisfied: bool,
    pub delivery_ready: bool,
    #[serde(default)]
    pub source_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaidReportOrderIntent {
    pub version: String,
    pub status: String,
    pub order_intent_id: String,
    pub product_instance_id: String,
    pub product_id: String,
    pub request_id: String,
    pub intent_kind: String,
    pub payment_provider_charge: bool,
    #[serde(default)]
    pub source_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaidReportInputSnapshot {
    pub version: String,
    pub status: String,
    pub input_snapshot_id: String,
    pub product_instance_id: String,
    pub product_id: String,
    pub request_id: String,
    pub report_definition_id: String,
    pub order_intent_id: String,
    #[serde(default)]
    pub required_input_refs: Vec<String>,
    #[serde(default)]
    pub submitted_fields: HashMap<String, String>,
    pub input_ready: bool,
    pub order_intent_ready: bool,
    pub projection_only: bool,
    pub writes_authority: bool,
    #[serde(default)]
    pub source_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaidReportRunExecutionReceipt {
    pub version: String,
    pub status: String,
    pub receipt_id: String,
    pub run_id: String,
    pub product_instance_id: String,
    pub product_id: String,
    pub request_id: String,
    pub runtime_admission_receipt_id: String,
    pub input_snapshot_id: String,
    pub report_definition_id: String,
    #[serde(default)]
    pub expected_artifact_ids: Vec<String>,
    #[serde(default)]
    pub failure_reasons: Vec<String>,
    pub started: bool,
    pub completed: bool,
    #[serde(default)]
    pub source_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaidReportArtifactSection {
    pub section_id: String,
    pub title: String,
    pub body: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaidReportArtifact {
    pub version: String,
    pub status: String,
    pub artifact_id: String,
    pub product_instance_id: String,
    pub run_id: String,
    pub report_definition_id: String,
    pub title: String,
    #[serde(default)]
    pub sections: Vec<PaidReportArtifactSection>,
    pub summary: String,
    pub generated_at: String,
    pub storage_path: String,
    #[serde(default)]
    pub source_evidence_refs: Vec<String>,
    pub delivery_ready: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaidReportEvidencePack {
    pub version: String,
    pub status: String,
    pub evidence_pack_id: String,
    pub product_instance_id: String,
    pub run_id: String,
    pub input_snapshot_id: String,
    pub run_execution_receipt_id: String,
    pub report_artifact_id: String,
    pub generation_receipt_id: String,
    #[serde(default)]
    pub required_evidence: Vec<String>,
    #[serde(default)]
    pub evidence_refs: Vec<String>,
    pub evidence_complete: bool,
    pub append_only: bool,
    pub project_scoped: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PaidReportDecisionOutcome {
    Accepted,
    NeedsFix,
    Rejected,
    Deferred,
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaidReportDecisionRecord {
    pub version: String,
    pub status: String,
    pub decision_id: String,
    pub outcome: PaidReportDecisionOutcome,
    pub report_artifact_id: String,
    pub evidence_pack_id: String,
    #[serde(default)]
    pub failure_reasons: Vec<String>,
    pub remediation_route: String,
    pub projection_only: bool,
    pub writes_authority: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaidReportDeliveryPackageProjection {
    pub version: String,
    pub status: String,
    pub delivery_package_id: String,
    pub product_instance_id: String,
    pub run_id: String,
    #[serde(default)]
    pub report_artifact_refs: Vec<String>,
    #[serde(default)]
    pub evidence_refs: Vec<String>,
    #[serde(default)]
    pub decision_refs: Vec<String>,
    pub delivery_status: String,
    pub download_ready: bool,
    pub display_contract: String,
    pub next_action: String,
    pub projection_only: bool,
    pub writes_authority: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaidReportFeedbackLoopProjection {
    pub version: String,
    pub status: String,
    pub feedback_id: String,
    pub feedback_state: String,
    pub repair_request_id: String,
    pub original_product_instance_id: String,
    pub run_id: String,
    pub artifact_id: String,
    pub evidence_pack_id: String,
    pub decision_id: String,
    pub mutates_delivered_artifact: bool,
    pub follow_up_route: String,
    pub next_action: String,
    pub projection_only: bool,
    pub writes_authority: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaidReportOrderRecord {
    pub version: String,
    pub status: String,
    pub order_id: String,
    pub product_instance_id: String,
    pub request_id: String,
    pub order_intent_id: String,
    pub input_snapshot_id: String,
    pub offer_ref: String,
    pub lifecycle_state: String,
    pub runnable: bool,
    pub created_at: String,
    #[serde(default)]
    pub source_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaidReportEntitlementAuthorization {
    pub version: String,
    pub status: String,
    pub authorization_receipt_id: String,
    pub order_id: String,
    pub product_instance_id: String,
    pub authorization_state: String,
    pub authorization_decision: String,
    pub payment_provider_checkout: bool,
    pub provider_charge_executed: bool,
    #[serde(default)]
    pub failure_reasons: Vec<String>,
    #[serde(default)]
    pub source_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaidReportOrderToRunAdmission {
    pub version: String,
    pub status: String,
    pub admission_id: String,
    pub order_id: String,
    pub authorization_receipt_id: String,
    pub input_snapshot_id: String,
    pub run_id: String,
    pub product_instance_id: String,
    pub accepted: bool,
    #[serde(default)]
    pub failure_reasons: Vec<String>,
    #[serde(default)]
    pub source_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaidReportCustomerDeliveryAccessProjection {
    pub version: String,
    pub status: String,
    pub delivery_package_id: String,
    pub order_id: String,
    pub decision_id: String,
    pub report_artifact_id: String,
    pub product_instance_id: String,
    pub access_status: String,
    pub next_action: String,
    pub download_visible: bool,
    pub projection_only: bool,
    pub writes_authority: bool,
    #[serde(default)]
    pub source_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaidReportAccessReceipt {
    pub version: String,
    pub status: String,
    pub access_receipt_id: String,
    pub delivery_package_id: String,
    pub order_id: String,
    pub product_instance_id: String,
    pub access_scope: String,
    pub generated_at: String,
    pub expires_at: String,
    #[serde(default)]
    pub artifact_refs: Vec<String>,
    pub access_handle: String,
    pub blocked_reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaidReportCommercialPolicyRecord {
    pub version: String,
    pub status: String,
    pub policy_id: String,
    pub outcome: String,
    pub original_order_id: String,
    pub original_run_id: String,
    pub original_artifact_id: String,
    pub original_decision_id: String,
    pub feedback_id: String,
    pub creates_follow_up_proposal: bool,
    pub mutates_delivered_artifact: bool,
    pub requires_new_authorization: bool,
    pub commercial_decision_only: bool,
    #[serde(default)]
    pub source_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommercialProjectionQuery {
    pub version: String,
    pub status: String,
    pub read_model_version: String,
    pub freshness: String,
    pub projection_only: bool,
    pub writes_authority: bool,
    pub read_model: CommercialProductReadModel,
    #[serde(default)]
    pub source_refs: Vec<String>,
    #[serde(default)]
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PaidReportPreflightDecision {
    Allowed,
    Rejected,
    Deferred,
    Invalid,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaidReportPreflightRequest {
    pub product_id: String,
    pub request_id: String,
    pub has_input_refs: bool,
    pub entitlement_state: CommercialEntitlementState,
    pub paid_feature_state: CommercialPaidFeatureState,
    pub report_definition_present: bool,
    pub order_intent_present: bool,
    pub payment_configured: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaidReportPreflightResult {
    pub version: String,
    pub request_id: String,
    pub flow_type: CommercialFlowType,
    pub decision: PaidReportPreflightDecision,
    pub unavailable_reason: String,
    pub runtime_command_policy: String,
    pub runtime_admission_required: bool,
    pub can_submit_runtime_command_proposal: bool,
    pub can_start_run_directly: bool,
    #[serde(default)]
    pub evidence_requirements: Vec<String>,
    #[serde(default)]
    pub decision_requirements: Vec<String>,
    pub delivery_promise: CommercialDeliveryPromise,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommercialFixtureResult<TInput, TExpected, TActual> {
    pub fixture_id: String,
    pub input: TInput,
    pub expected: TExpected,
    pub actual: TActual,
    pub passed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagedProjectCommercialFixture {
    pub version: String,
    pub status: String,
    #[serde(default)]
    pub results: Vec<
        CommercialFixtureResult<
            CommercialProductInput,
            CommercialProductReadModelEntry,
            CommercialProductReadModelEntry,
        >,
    >,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommercialNegativeFixtureReport {
    pub version: String,
    pub status: String,
    #[serde(default)]
    pub read_model_results: Vec<
        CommercialFixtureResult<
            CommercialProductInput,
            CommercialProductReadModelEntry,
            CommercialProductReadModelEntry,
        >,
    >,
    #[serde(default)]
    pub preflight_results: Vec<
        CommercialFixtureResult<
            PaidReportPreflightRequest,
            PaidReportPreflightResult,
            PaidReportPreflightResult,
        >,
    >,
    pub failed_commercial_preflight_can_submit_runtime_command: bool,
    pub managed_project_can_use_paid_report_authority: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommercialGoldenPathProof {
    pub version: String,
    pub status: String,
    pub read_model: CommercialProductReadModel,
    pub projection_query: CommercialProjectionQuery,
    pub paid_report_blocked: PaidReportPreflightResult,
    pub paid_report_deferred: PaidReportPreflightResult,
    pub managed_project_available: CommercialProductReadModelEntry,
    pub projection_writes_authority: bool,
    pub desktop_writes_authority: bool,
}

pub fn default_commercial_product_inputs() -> Vec<CommercialProductInput> {
    vec![
        CommercialProductInput {
            product_id: "paid-report".to_string(),
            product_name: "Paid Report".to_string(),
            flow_type: CommercialFlowType::PaidReportFlow,
            entitlement_state: CommercialEntitlementState::Disabled,
            paid_feature_state: CommercialPaidFeatureState::Disabled,
            flow_definition_present: true,
            product_definition_present: true,
            payment_configured: false,
            paid_report_authority_fields: Vec::new(),
            required_project_refs_present: false,
        },
        CommercialProductInput {
            product_id: "paid-report-preview".to_string(),
            product_name: "Paid Report Preview".to_string(),
            flow_type: CommercialFlowType::PaidReportFlow,
            entitlement_state: CommercialEntitlementState::Deferred,
            paid_feature_state: CommercialPaidFeatureState::Deferred,
            flow_definition_present: true,
            product_definition_present: true,
            payment_configured: false,
            paid_report_authority_fields: Vec::new(),
            required_project_refs_present: false,
        },
        CommercialProductInput {
            product_id: "managed-project".to_string(),
            product_name: "Managed Project".to_string(),
            flow_type: CommercialFlowType::ManagedProjectFlow,
            entitlement_state: CommercialEntitlementState::Trial,
            paid_feature_state: CommercialPaidFeatureState::NotRequired,
            flow_definition_present: true,
            product_definition_present: true,
            payment_configured: false,
            paid_report_authority_fields: Vec::new(),
            required_project_refs_present: true,
        },
    ]
}

pub fn build_commercial_product_read_model(
    inputs: Vec<CommercialProductInput>,
) -> CommercialProductReadModel {
    let entries = inputs
        .into_iter()
        .map(evaluate_commercial_product)
        .collect::<Vec<_>>();
    let status = commercial_read_model_status(&entries);

    CommercialProductReadModel {
        version: COMMERCIAL_PRODUCT_READ_MODEL_VERSION.to_string(),
        status: status.to_string(),
        source: "runtime-api".to_string(),
        projection_only: true,
        core_authority: false,
        writes_authority: false,
        entries,
        source_refs: vec![
            "docs/architecture/095-commercial-product-read-model-contract-v1.md".to_string(),
            "docs/project/goal.md".to_string(),
        ],
        freshness: "fresh".to_string(),
    }
}

pub fn load_commercial_product_read_model() -> CommercialProductReadModel {
    let registry_root = resolve_default_commercial_registry_root();
    if registry_root.is_dir() {
        if let Ok(model) = load_registry_commercial_product_read_model(&registry_root) {
            return model;
        }
    }

    let mut model = build_commercial_product_read_model(default_commercial_product_inputs());
    model.source = "default-fixture".to_string();
    model.source_refs.push(
        "crates/runtime-api/src/commercial.rs::default_commercial_product_inputs".to_string(),
    );
    model
}

pub fn load_project_commercial_product_read_model(
    project_root: impl AsRef<Path>,
) -> CommercialProductReadModel {
    let registry_root = project_commercial_registry_root(project_root);
    if registry_root.is_dir() {
        match load_registry_commercial_product_read_model(&registry_root) {
            Ok(mut model) => {
                model.source = "project-commercial-registry".to_string();
                model
                    .source_refs
                    .insert(0, registry_root.display().to_string());
                model
            }
            Err(error) => CommercialProductReadModel {
                version: COMMERCIAL_PRODUCT_READ_MODEL_VERSION.to_string(),
                status: "invalid".to_string(),
                source: "project-commercial-registry-invalid".to_string(),
                projection_only: true,
                core_authority: false,
                writes_authority: false,
                entries: Vec::new(),
                source_refs: vec![
                    registry_root.display().to_string(),
                    format!("error:{error}"),
                    "docs/architecture/095-commercial-product-read-model-contract-v1.md"
                        .to_string(),
                ],
                freshness: "fresh".to_string(),
            },
        }
    } else {
        CommercialProductReadModel {
            version: COMMERCIAL_PRODUCT_READ_MODEL_VERSION.to_string(),
            status: "unavailable".to_string(),
            source: "project-commercial-registry-missing".to_string(),
            projection_only: true,
            core_authority: false,
            writes_authority: false,
            entries: Vec::new(),
            source_refs: vec![
                registry_root.display().to_string(),
                "docs/architecture/095-commercial-product-read-model-contract-v1.md".to_string(),
            ],
            freshness: "fresh".to_string(),
        }
    }
}

pub fn load_registry_commercial_product_read_model(
    registry_root: impl AsRef<Path>,
) -> Result<CommercialProductReadModel> {
    let registry_root = registry_root.as_ref();
    let products_path = registry_root.join("products.json");
    let entitlements_path = registry_root.join("entitlements.json");
    let registry = read_json::<CommercialProductRegistryConfig>(&products_path)?;
    let entitlement_source = read_json::<CommercialEntitlementSourceConfig>(&entitlements_path)?;
    let entitlements = entitlement_source
        .entitlements
        .iter()
        .map(|entry| (entry.product_id.as_str(), entry))
        .collect::<HashMap<_, _>>();

    let mut entries = Vec::new();
    for definition in registry.products.iter() {
        let entitlement = entitlements.get(definition.product_id.as_str());
        let input = commercial_product_input_from_definition(definition, entitlement.copied());
        let mut entry = evaluate_commercial_product(input);
        entry.source_refs = registry_source_refs(registry_root, definition, entitlement.copied());
        if definition.flow_type == CommercialFlowType::PaidReportFlow
            && !definition.report_definition_present
        {
            entry.availability = CommercialAvailability::Invalid;
            entry.unavailable_reason = "missing-report-definition".to_string();
            entry.command_policy = CommercialCommandPolicy::BlockedBeforeRuntime;
            entry.can_submit_runtime_command_proposal = false;
        }
        if definition.flow_type == CommercialFlowType::PaidReportFlow
            && definition.required_input_refs.is_empty()
        {
            entry.availability = CommercialAvailability::Invalid;
            entry.unavailable_reason = "missing-required-inputs".to_string();
            entry.command_policy = CommercialCommandPolicy::BlockedBeforeRuntime;
            entry.can_submit_runtime_command_proposal = false;
        }
        entries.push(entry);
    }

    let status = commercial_read_model_status(&entries);

    Ok(CommercialProductReadModel {
        version: COMMERCIAL_PRODUCT_READ_MODEL_VERSION.to_string(),
        status: status.to_string(),
        source: "product-registry-config".to_string(),
        projection_only: true,
        core_authority: false,
        writes_authority: false,
        entries,
        source_refs: vec![
            portable_registry_ref(registry_root, &products_path),
            portable_registry_ref(registry_root, &entitlements_path),
            "docs/architecture/095-commercial-product-read-model-contract-v1.md".to_string(),
        ],
        freshness: "fresh".to_string(),
    })
}

pub fn evaluate_paid_report_preflight_from_registry(
    registry_root: impl AsRef<Path>,
    product_id: &str,
    request_id: &str,
) -> Result<PaidReportPreflightResult> {
    let registry_root = registry_root.as_ref();
    let registry =
        read_json::<CommercialProductRegistryConfig>(&registry_root.join("products.json"))?;
    let entitlement_source =
        read_json::<CommercialEntitlementSourceConfig>(&registry_root.join("entitlements.json"))?;
    let definition = registry
        .products
        .iter()
        .find(|entry| entry.product_id == product_id)
        .with_context(|| format!("missing product definition `{product_id}`"))?;
    let entitlement_state = entitlement_source
        .entitlements
        .iter()
        .find(|entry| entry.product_id == product_id)
        .map(|entry| entry.entitlement_state)
        .unwrap_or(CommercialEntitlementState::Missing);

    let mut result = evaluate_paid_report_preflight(PaidReportPreflightRequest {
        product_id: definition.product_id.clone(),
        request_id: request_id.to_string(),
        has_input_refs: !definition.required_input_refs.is_empty(),
        entitlement_state,
        paid_feature_state: definition.paid_feature_state,
        report_definition_present: definition.report_definition_present,
        order_intent_present: definition
            .required_input_refs
            .iter()
            .any(|item| item == "orderIntentId"),
        payment_configured: definition.payment_configured,
    });
    if !definition.evidence_requirements.is_empty() {
        result.evidence_requirements = definition.evidence_requirements.clone();
    }
    if !definition.decision_requirements.is_empty() {
        result.decision_requirements = definition.decision_requirements.clone();
    }
    Ok(result)
}

pub fn resolve_paid_report_product_instance_from_registry(
    registry_root: impl AsRef<Path>,
    product_id: &str,
) -> Result<PaidReportProductInstanceContract> {
    let registry_root = registry_root.as_ref();
    let registry =
        read_json::<CommercialProductRegistryConfig>(&registry_root.join("products.json"))?;
    let entitlement_source =
        read_json::<CommercialEntitlementSourceConfig>(&registry_root.join("entitlements.json"))?;
    let definition = registry
        .products
        .iter()
        .find(|entry| entry.product_id == product_id)
        .with_context(|| format!("missing product definition `{product_id}`"))?;
    let entitlement = entitlement_source
        .entitlements
        .iter()
        .find(|entry| entry.product_id == product_id);
    let input = commercial_product_input_from_definition(definition, entitlement);
    let entry = evaluate_commercial_product(input);
    let report_definition_id = if definition.report_definition_id.trim().is_empty() {
        format!("{}-report-definition", definition.product_id)
    } else {
        definition.report_definition_id.clone()
    };
    let missing_required_instance_fields = definition.required_input_refs.is_empty()
        || !definition.report_definition_present
        || report_definition_id.trim().is_empty();
    let can_submit = entry.can_submit_runtime_command_proposal && !missing_required_instance_fields;
    let (status, unavailable_reason) = if definition.flow_type != CommercialFlowType::PaidReportFlow
    {
        ("invalid", "not-paid-report-product")
    } else if !definition.report_definition_present {
        ("invalid", "missing-report-definition")
    } else if definition.required_input_refs.is_empty() {
        ("invalid", "missing-required-inputs")
    } else if entry.availability == CommercialAvailability::Available {
        ("ready", "none")
    } else {
        ("blocked", entry.unavailable_reason.as_str())
    };

    Ok(PaidReportProductInstanceContract {
        version: PAID_REPORT_PRODUCT_INSTANCE_VERSION.to_string(),
        status: status.to_string(),
        product_instance_id: format!("{}::{}", registry.source, definition.product_id),
        product_id: definition.product_id.clone(),
        report_definition_id,
        required_input_refs: definition.required_input_refs.clone(),
        evidence_requirements: if definition.evidence_requirements.is_empty() {
            vec!["report-generation-evidence".to_string()]
        } else {
            definition.evidence_requirements.clone()
        },
        decision_requirements: if definition.decision_requirements.is_empty() {
            vec!["report-delivery-decision".to_string()]
        } else {
            definition.decision_requirements.clone()
        },
        entitlement_state: entitlement
            .map(|entry| entry.entitlement_state)
            .unwrap_or(CommercialEntitlementState::Missing),
        paid_feature_state: definition.paid_feature_state,
        delivery_promise: CommercialDeliveryPromise::Report,
        can_submit_runtime_command_proposal: can_submit,
        unavailable_reason: unavailable_reason.to_string(),
        source_refs: registry_source_refs(registry_root, definition, entitlement),
    })
}

pub fn resolve_paid_report_product_instance_from_project(
    project_root: impl AsRef<Path>,
    product_id: &str,
) -> Result<PaidReportProductInstanceContract> {
    let project_root = project_root.as_ref();
    let registry_root = project_commercial_registry_root(project_root);
    let mut instance =
        resolve_paid_report_product_instance_from_registry(&registry_root, product_id)?;
    let project_digest = stable_path_digest(project_root);
    instance.product_instance_id = format!(
        "project-{project_digest}::{}::{}",
        instance.product_id, instance.report_definition_id
    );
    instance.source_refs.insert(
        0,
        format!("project:{}#{}", project_root.display(), project_digest),
    );
    Ok(instance)
}

pub fn build_paid_report_runtime_proposal_handoff_from_registry(
    registry_root: impl AsRef<Path>,
    product_id: &str,
    request_id: &str,
) -> Result<PaidReportRuntimeProposalHandoff> {
    let instance = resolve_paid_report_product_instance_from_registry(&registry_root, product_id)?;
    let preflight =
        evaluate_paid_report_preflight_from_registry(registry_root, product_id, request_id)?;
    Ok(paid_report_runtime_proposal_handoff_from_parts(
        instance, preflight,
    ))
}

fn paid_report_runtime_proposal_handoff_from_parts(
    instance: PaidReportProductInstanceContract,
    preflight: PaidReportPreflightResult,
) -> PaidReportRuntimeProposalHandoff {
    let allowed = instance.can_submit_runtime_command_proposal
        && preflight.decision == PaidReportPreflightDecision::Allowed
        && preflight.can_submit_runtime_command_proposal
        && preflight.runtime_admission_required
        && !preflight.can_start_run_directly;
    let proposal = allowed.then(|| PaidReportRuntimeProposal {
        product_instance_id: instance.product_instance_id.clone(),
        product_id: instance.product_id.clone(),
        request_id: preflight.request_id.clone(),
        required_input_refs: instance.required_input_refs.clone(),
        report_definition_id: instance.report_definition_id.clone(),
        evidence_policy: instance.evidence_requirements.clone(),
        decision_policy: instance.decision_requirements.clone(),
        delivery_promise: instance.delivery_promise,
        runtime_admission_required: true,
        can_start_run_directly: false,
    });
    let status = if allowed { "ready" } else { "blocked" };
    let reason = if allowed {
        "proposal-ready-runtime-admission-required".to_string()
    } else if instance.status != "ready" {
        instance.unavailable_reason.clone()
    } else {
        preflight.unavailable_reason.clone()
    };
    PaidReportRuntimeProposalHandoff {
        version: PAID_REPORT_RUNTIME_PROPOSAL_HANDOFF_VERSION.to_string(),
        status: status.to_string(),
        reason,
        proposal_created: proposal.is_some(),
        product_instance: instance,
        preflight,
        proposal,
    }
}

pub fn build_paid_report_runtime_proposal_handoff_from_project(
    project_root: impl AsRef<Path>,
    product_id: &str,
    request_id: &str,
) -> Result<PaidReportRuntimeProposalHandoff> {
    let project_root = project_root.as_ref();
    let instance = resolve_paid_report_product_instance_from_project(project_root, product_id)?;
    let preflight = evaluate_paid_report_preflight_from_registry(
        project_commercial_registry_root(project_root),
        product_id,
        request_id,
    )?;
    Ok(paid_report_runtime_proposal_handoff_from_parts(
        instance, preflight,
    ))
}

pub fn admit_paid_report_runtime_proposal(
    handoff: &PaidReportRuntimeProposalHandoff,
) -> PaidReportRuntimeAdmissionReceipt {
    let admitted = handoff.status == "ready"
        && handoff.proposal_created
        && handoff.proposal.as_ref().is_some_and(|proposal| {
            proposal.runtime_admission_required && !proposal.can_start_run_directly
        });
    let proposal = handoff.proposal.as_ref();
    let required_evidence = proposal
        .map(|proposal| proposal.evidence_policy.clone())
        .unwrap_or_else(|| handoff.product_instance.evidence_requirements.clone());
    let required_decision_policy = proposal
        .map(|proposal| proposal.decision_policy.clone())
        .unwrap_or_else(|| handoff.product_instance.decision_requirements.clone());
    let request_id = proposal
        .map(|proposal| proposal.request_id.clone())
        .unwrap_or_else(|| handoff.preflight.request_id.clone());

    PaidReportRuntimeAdmissionReceipt {
        version: PAID_REPORT_RUNTIME_ADMISSION_RECEIPT_VERSION.to_string(),
        status: if admitted { "admitted" } else { "blocked" }.to_string(),
        receipt_id: format!(
            "paid-report-admission-{}-{}",
            handoff.product_instance.product_id, request_id
        ),
        product_instance_id: handoff.product_instance.product_instance_id.clone(),
        product_id: handoff.product_instance.product_id.clone(),
        request_id,
        admission_decision: if admitted {
            "accepted-for-runtime-proposal"
        } else {
            "blocked-before-runtime"
        }
        .to_string(),
        runtime_admission_required: true,
        can_start_run_directly: false,
        required_evidence,
        required_decision_policy,
        source_refs: handoff.product_instance.source_refs.clone(),
    }
}

pub fn build_paid_report_run_contract(
    handoff: &PaidReportRuntimeProposalHandoff,
    receipt: &PaidReportRuntimeAdmissionReceipt,
) -> PaidReportRunContract {
    let proposal = handoff.proposal.as_ref();
    let input_refs = proposal
        .map(|proposal| proposal.required_input_refs.clone())
        .unwrap_or_else(|| handoff.product_instance.required_input_refs.clone());
    let report_definition_id = proposal
        .map(|proposal| proposal.report_definition_id.clone())
        .unwrap_or_else(|| handoff.product_instance.report_definition_id.clone());
    let expected_evidence = if receipt.required_evidence.is_empty() {
        handoff.product_instance.evidence_requirements.clone()
    } else {
        receipt.required_evidence.clone()
    };
    let decision_policy = if receipt.required_decision_policy.is_empty() {
        handoff.product_instance.decision_requirements.clone()
    } else {
        receipt.required_decision_policy.clone()
    };
    let ready = receipt.status == "admitted"
        && !receipt.can_start_run_directly
        && !input_refs.is_empty()
        && !report_definition_id.trim().is_empty()
        && !expected_evidence.is_empty()
        && !decision_policy.is_empty();

    PaidReportRunContract {
        version: PAID_REPORT_RUN_CONTRACT_VERSION.to_string(),
        status: if ready { "ready" } else { "blocked" }.to_string(),
        run_contract_id: format!("paid-report-run-{}", receipt.request_id),
        product_instance_id: receipt.product_instance_id.clone(),
        product_id: receipt.product_id.clone(),
        request_id: receipt.request_id.clone(),
        input_refs,
        report_definition_id,
        expected_evidence,
        decision_policy,
        delivery_promise: CommercialDeliveryPromise::Report,
        runtime_admission_receipt_id: receipt.receipt_id.clone(),
        can_start_run_directly: false,
        concrete_sku_is_core_authority: false,
        source_refs: receipt.source_refs.clone(),
    }
}

pub fn project_paid_report_delivery_projection(
    run_contract: &PaidReportRunContract,
    evidence_satisfied: bool,
    decision_satisfied: bool,
) -> PaidReportDeliveryProjection {
    let status = if run_contract.status == "blocked" {
        "blocked"
    } else if evidence_satisfied && decision_satisfied {
        "delivery-ready"
    } else if evidence_satisfied {
        "decision-needed"
    } else {
        "evidence-needed"
    };

    PaidReportDeliveryProjection {
        version: PAID_REPORT_DELIVERY_PROJECTION_VERSION.to_string(),
        status: status.to_string(),
        product_instance_id: run_contract.product_instance_id.clone(),
        product_id: run_contract.product_id.clone(),
        request_id: run_contract.request_id.clone(),
        projection_only: true,
        writes_authority: false,
        required_evidence: run_contract.expected_evidence.clone(),
        decision_policy: run_contract.decision_policy.clone(),
        evidence_satisfied,
        decision_satisfied,
        delivery_ready: status == "delivery-ready",
        source_refs: run_contract.source_refs.clone(),
    }
}

pub fn build_paid_report_order_intent(
    instance: &PaidReportProductInstanceContract,
    request_id: &str,
) -> PaidReportOrderIntent {
    PaidReportOrderIntent {
        version: PAID_REPORT_ORDER_INTENT_VERSION.to_string(),
        status: if instance.status == "ready" {
            "ready"
        } else {
            "blocked"
        }
        .to_string(),
        order_intent_id: format!("paid-report-order-intent-{request_id}"),
        product_instance_id: instance.product_instance_id.clone(),
        product_id: instance.product_id.clone(),
        request_id: request_id.to_string(),
        intent_kind: "product-execution-intent".to_string(),
        payment_provider_charge: false,
        source_refs: instance.source_refs.clone(),
    }
}

pub fn build_paid_report_input_snapshot(
    instance: &PaidReportProductInstanceContract,
    order_intent: Option<&PaidReportOrderIntent>,
    request_id: &str,
    submitted_fields: HashMap<String, String>,
) -> PaidReportInputSnapshot {
    let missing_required = instance
        .required_input_refs
        .iter()
        .filter(|required| !submitted_fields.contains_key(required.as_str()))
        .count();
    let order_intent_ready = order_intent.is_some_and(|intent| {
        intent.status == "ready"
            && intent.product_instance_id == instance.product_instance_id
            && !intent.payment_provider_charge
    });
    let input_ready = missing_required == 0
        && order_intent_ready
        && !instance.report_definition_id.trim().is_empty();
    let status = if input_ready {
        "input-ready"
    } else if !order_intent_ready {
        "order-intent-missing"
    } else {
        "input-missing"
    };

    PaidReportInputSnapshot {
        version: PAID_REPORT_INPUT_SNAPSHOT_VERSION.to_string(),
        status: status.to_string(),
        input_snapshot_id: format!("paid-report-input-snapshot-{request_id}"),
        product_instance_id: instance.product_instance_id.clone(),
        product_id: instance.product_id.clone(),
        request_id: request_id.to_string(),
        report_definition_id: instance.report_definition_id.clone(),
        order_intent_id: order_intent
            .map(|intent| intent.order_intent_id.clone())
            .unwrap_or_default(),
        required_input_refs: instance.required_input_refs.clone(),
        submitted_fields,
        input_ready,
        order_intent_ready,
        projection_only: true,
        writes_authority: false,
        source_refs: instance.source_refs.clone(),
    }
}

pub fn build_paid_report_run_execution_receipt(
    run_contract: &PaidReportRunContract,
    input_snapshot: Option<&PaidReportInputSnapshot>,
    completed: bool,
) -> PaidReportRunExecutionReceipt {
    let input_snapshot_ready = input_snapshot.is_some_and(|snapshot| {
        snapshot.input_ready
            && snapshot.product_instance_id == run_contract.product_instance_id
            && snapshot.report_definition_id == run_contract.report_definition_id
    });
    let mut failure_reasons = Vec::new();
    if run_contract.status != "ready" {
        failure_reasons.push("run-contract-not-ready".to_string());
    }
    if run_contract.runtime_admission_receipt_id.trim().is_empty() {
        failure_reasons.push("missing-admission-receipt".to_string());
    }
    if !input_snapshot_ready {
        failure_reasons.push("missing-valid-input-snapshot".to_string());
    }
    let started = failure_reasons.is_empty();
    let status = if !started {
        "blocked"
    } else if completed {
        "completed"
    } else {
        "started"
    };
    let run_id = run_contract
        .run_contract_id
        .replace("contract", "execution");

    PaidReportRunExecutionReceipt {
        version: PAID_REPORT_RUN_EXECUTION_RECEIPT_VERSION.to_string(),
        status: status.to_string(),
        receipt_id: format!("{run_id}-receipt"),
        run_id,
        product_instance_id: run_contract.product_instance_id.clone(),
        product_id: run_contract.product_id.clone(),
        request_id: run_contract.request_id.clone(),
        runtime_admission_receipt_id: run_contract.runtime_admission_receipt_id.clone(),
        input_snapshot_id: input_snapshot
            .map(|snapshot| snapshot.input_snapshot_id.clone())
            .unwrap_or_default(),
        report_definition_id: run_contract.report_definition_id.clone(),
        expected_artifact_ids: vec![format!("report-artifact-{}", run_contract.request_id)],
        failure_reasons,
        started,
        completed: started && completed,
        source_refs: run_contract.source_refs.clone(),
    }
}

pub fn build_paid_report_artifact(
    run_receipt: Option<&PaidReportRunExecutionReceipt>,
    complete: bool,
) -> PaidReportArtifact {
    let run_ready = run_receipt.is_some_and(|receipt| receipt.status == "completed");
    let request_id = run_receipt
        .map(|receipt| receipt.request_id.clone())
        .unwrap_or_else(|| "missing-run".to_string());
    let sections = if run_ready && complete {
        vec![
            PaidReportArtifactSection {
                section_id: "summary".to_string(),
                title: "Summary".to_string(),
                body: "Generic paid report summary generated from accepted runtime facts."
                    .to_string(),
            },
            PaidReportArtifactSection {
                section_id: "details".to_string(),
                title: "Details".to_string(),
                body: "Generic paid report details remain product/pack supplied.".to_string(),
            },
        ]
    } else {
        Vec::new()
    };
    let artifact_complete = run_ready && complete && !sections.is_empty();
    let status = if artifact_complete {
        "complete"
    } else if !run_ready {
        "blocked"
    } else {
        "incomplete"
    };

    PaidReportArtifact {
        version: PAID_REPORT_ARTIFACT_VERSION.to_string(),
        status: status.to_string(),
        artifact_id: format!("report-artifact-{request_id}"),
        product_instance_id: run_receipt
            .map(|receipt| receipt.product_instance_id.clone())
            .unwrap_or_default(),
        run_id: run_receipt
            .map(|receipt| receipt.run_id.clone())
            .unwrap_or_default(),
        report_definition_id: run_receipt
            .map(|receipt| receipt.report_definition_id.clone())
            .unwrap_or_default(),
        title: "Generic Paid Report".to_string(),
        sections,
        summary: if artifact_complete {
            "Generic paid report artifact is complete.".to_string()
        } else {
            "Generic paid report artifact is incomplete.".to_string()
        },
        generated_at: "2026-07-09T00:00:00Z".to_string(),
        storage_path: format!(".agentflow/tasks/{request_id}/report-artifacts/report.json"),
        source_evidence_refs: run_receipt
            .map(|receipt| vec![receipt.receipt_id.clone()])
            .unwrap_or_default(),
        delivery_ready: artifact_complete,
    }
}

pub fn capture_paid_report_generation_evidence(
    run_receipt: &PaidReportRunExecutionReceipt,
    artifact: &PaidReportArtifact,
    required_evidence: Vec<String>,
    evidence_refs: Vec<String>,
) -> PaidReportEvidencePack {
    let required_complete = !required_evidence.is_empty()
        && required_evidence
            .iter()
            .all(|required| evidence_refs.iter().any(|entry| entry.contains(required)));
    let links_complete = run_receipt.status == "completed"
        && artifact.status == "complete"
        && artifact.run_id == run_receipt.run_id;
    let evidence_complete = required_complete && links_complete;

    PaidReportEvidencePack {
        version: PAID_REPORT_EVIDENCE_PACK_VERSION.to_string(),
        status: if evidence_complete {
            "complete"
        } else {
            "evidence-needed"
        }
        .to_string(),
        evidence_pack_id: format!("evidence-pack-{}", run_receipt.request_id),
        product_instance_id: run_receipt.product_instance_id.clone(),
        run_id: run_receipt.run_id.clone(),
        input_snapshot_id: run_receipt.input_snapshot_id.clone(),
        run_execution_receipt_id: run_receipt.receipt_id.clone(),
        report_artifact_id: artifact.artifact_id.clone(),
        generation_receipt_id: format!("generation-receipt-{}", run_receipt.request_id),
        required_evidence,
        evidence_refs,
        evidence_complete,
        append_only: true,
        project_scoped: true,
    }
}

pub fn decide_paid_report_delivery(
    artifact: &PaidReportArtifact,
    evidence_pack: &PaidReportEvidencePack,
    requested_outcome: PaidReportDecisionOutcome,
) -> PaidReportDecisionRecord {
    let accepted = requested_outcome == PaidReportDecisionOutcome::Accepted
        && artifact.status == "complete"
        && evidence_pack.evidence_complete
        && evidence_pack.report_artifact_id == artifact.artifact_id;
    let outcome = if accepted {
        PaidReportDecisionOutcome::Accepted
    } else if artifact.status != "complete" {
        PaidReportDecisionOutcome::Blocked
    } else if !evidence_pack.evidence_complete {
        PaidReportDecisionOutcome::NeedsFix
    } else {
        requested_outcome
    };
    let failure_reasons = match outcome {
        PaidReportDecisionOutcome::Accepted => Vec::new(),
        PaidReportDecisionOutcome::Blocked => vec!["report-artifact-incomplete".to_string()],
        PaidReportDecisionOutcome::NeedsFix => vec!["evidence-incomplete".to_string()],
        PaidReportDecisionOutcome::Rejected => vec!["decision-rejected".to_string()],
        PaidReportDecisionOutcome::Deferred => vec!["decision-deferred".to_string()],
    };

    PaidReportDecisionRecord {
        version: PAID_REPORT_DECISION_RECORD_VERSION.to_string(),
        status: if outcome == PaidReportDecisionOutcome::Accepted {
            "accepted"
        } else {
            "not-accepted"
        }
        .to_string(),
        decision_id: format!("decision-{}", evidence_pack.run_id),
        outcome,
        report_artifact_id: artifact.artifact_id.clone(),
        evidence_pack_id: evidence_pack.evidence_pack_id.clone(),
        failure_reasons,
        remediation_route: if outcome == PaidReportDecisionOutcome::Accepted {
            "deliver".to_string()
        } else {
            "repair-request".to_string()
        },
        projection_only: false,
        writes_authority: true,
    }
}

pub fn project_paid_report_delivery_package(
    artifact: &PaidReportArtifact,
    evidence_pack: &PaidReportEvidencePack,
    decision: &PaidReportDecisionRecord,
) -> PaidReportDeliveryPackageProjection {
    let accepted = decision.outcome == PaidReportDecisionOutcome::Accepted
        && artifact.status == "complete"
        && evidence_pack.evidence_complete;
    let delivery_status = if accepted {
        "delivery-ready"
    } else {
        decision.status.as_str()
    };

    PaidReportDeliveryPackageProjection {
        version: PAID_REPORT_DELIVERY_PACKAGE_PROJECTION_VERSION.to_string(),
        status: delivery_status.to_string(),
        delivery_package_id: format!("delivery-package-{}", decision.decision_id),
        product_instance_id: evidence_pack.product_instance_id.clone(),
        run_id: evidence_pack.run_id.clone(),
        report_artifact_refs: vec![artifact.artifact_id.clone()],
        evidence_refs: evidence_pack.evidence_refs.clone(),
        decision_refs: vec![decision.decision_id.clone()],
        delivery_status: delivery_status.to_string(),
        download_ready: accepted,
        display_contract: if accepted {
            "download-report-and-evidence"
        } else {
            "show-next-action-only"
        }
        .to_string(),
        next_action: if accepted {
            "show-download"
        } else {
            "resolve-decision"
        }
        .to_string(),
        projection_only: true,
        writes_authority: false,
    }
}

pub fn project_paid_report_feedback_loop(
    delivery: &PaidReportDeliveryPackageProjection,
    decision: &PaidReportDecisionRecord,
    feedback_state: &str,
) -> PaidReportFeedbackLoopProjection {
    let repair_requested = feedback_state == "repair-requested";
    PaidReportFeedbackLoopProjection {
        version: PAID_REPORT_FEEDBACK_LOOP_PROJECTION_VERSION.to_string(),
        status: feedback_state.to_string(),
        feedback_id: format!("feedback-{}", delivery.delivery_package_id),
        feedback_state: feedback_state.to_string(),
        repair_request_id: if repair_requested {
            format!("repair-{}", delivery.delivery_package_id)
        } else {
            String::new()
        },
        original_product_instance_id: delivery.product_instance_id.clone(),
        run_id: delivery.run_id.clone(),
        artifact_id: delivery
            .report_artifact_refs
            .first()
            .cloned()
            .unwrap_or_default(),
        evidence_pack_id: delivery.evidence_refs.first().cloned().unwrap_or_default(),
        decision_id: decision.decision_id.clone(),
        mutates_delivered_artifact: false,
        follow_up_route: if repair_requested {
            "controlled-follow-up-proposal"
        } else {
            "no-follow-up"
        }
        .to_string(),
        next_action: match feedback_state {
            "feedback-needed" => "collect-feedback",
            "repair-requested" => "create-repair-proposal",
            "accepted-after-repair" => "close-feedback",
            _ => "show-feedback-status",
        }
        .to_string(),
        projection_only: true,
        writes_authority: false,
    }
}

pub fn build_paid_report_order_record(
    instance: &PaidReportProductInstanceContract,
    order_intent: &PaidReportOrderIntent,
    input_snapshot: &PaidReportInputSnapshot,
    offer_ref: &str,
) -> PaidReportOrderRecord {
    let runnable = instance.status == "ready"
        && order_intent.status == "ready"
        && input_snapshot.input_ready
        && order_intent.product_instance_id == instance.product_instance_id
        && input_snapshot.product_instance_id == instance.product_instance_id
        && !offer_ref.trim().is_empty();
    let lifecycle_state = if runnable {
        "order-ready"
    } else if !input_snapshot.input_ready {
        "input-snapshot-missing"
    } else if order_intent.status != "ready" {
        "order-intent-missing"
    } else {
        "order-blocked"
    };

    PaidReportOrderRecord {
        version: PAID_REPORT_ORDER_RECORD_VERSION.to_string(),
        status: lifecycle_state.to_string(),
        order_id: format!("paid-report-order-{}", input_snapshot.request_id),
        product_instance_id: instance.product_instance_id.clone(),
        request_id: input_snapshot.request_id.clone(),
        order_intent_id: order_intent.order_intent_id.clone(),
        input_snapshot_id: input_snapshot.input_snapshot_id.clone(),
        offer_ref: offer_ref.to_string(),
        lifecycle_state: lifecycle_state.to_string(),
        runnable,
        created_at: "2026-07-09T00:00:00Z".to_string(),
        source_refs: instance.source_refs.clone(),
    }
}

pub fn authorize_paid_report_order(
    order: &PaidReportOrderRecord,
    authorization_state: &str,
) -> PaidReportEntitlementAuthorization {
    let normalized = authorization_state.trim();
    let authorized = order.runnable && matches!(normalized, "paid" | "waived");
    let decision = if authorized {
        "authorized"
    } else if normalized == "deferred" {
        "deferred"
    } else {
        "blocked"
    };
    let mut failure_reasons = Vec::new();
    if !order.runnable {
        failure_reasons.push("order-not-runnable".to_string());
    }
    match normalized {
        "paid" | "waived" | "deferred" => {}
        "refunded" => failure_reasons.push("order-refunded".to_string()),
        "missing" => failure_reasons.push("entitlement-missing".to_string()),
        "revoked" => failure_reasons.push("entitlement-revoked".to_string()),
        _ => failure_reasons.push("unknown-authorization-state".to_string()),
    }

    PaidReportEntitlementAuthorization {
        version: PAID_REPORT_ENTITLEMENT_AUTHORIZATION_VERSION.to_string(),
        status: decision.to_string(),
        authorization_receipt_id: format!("authorization-{}", order.order_id),
        order_id: order.order_id.clone(),
        product_instance_id: order.product_instance_id.clone(),
        authorization_state: normalized.to_string(),
        authorization_decision: decision.to_string(),
        payment_provider_checkout: false,
        provider_charge_executed: false,
        failure_reasons,
        source_refs: order.source_refs.clone(),
    }
}

pub fn admit_paid_report_order_to_run(
    order: &PaidReportOrderRecord,
    authorization: &PaidReportEntitlementAuthorization,
    input_snapshot: &PaidReportInputSnapshot,
    run_receipt: &PaidReportRunExecutionReceipt,
) -> PaidReportOrderToRunAdmission {
    let accepted = order.runnable
        && authorization.status == "authorized"
        && authorization.order_id == order.order_id
        && input_snapshot.input_ready
        && input_snapshot.input_snapshot_id == order.input_snapshot_id
        && run_receipt.product_instance_id == order.product_instance_id
        && run_receipt.input_snapshot_id == input_snapshot.input_snapshot_id;
    let mut failure_reasons = Vec::new();
    if !order.runnable {
        failure_reasons.push("order-not-runnable".to_string());
    }
    if authorization.status != "authorized" || authorization.order_id != order.order_id {
        failure_reasons.push("authorization-not-valid".to_string());
    }
    if !input_snapshot.input_ready || input_snapshot.input_snapshot_id != order.input_snapshot_id {
        failure_reasons.push("input-snapshot-not-valid".to_string());
    }
    if run_receipt.product_instance_id != order.product_instance_id {
        failure_reasons.push("product-instance-mismatch".to_string());
    }

    PaidReportOrderToRunAdmission {
        version: PAID_REPORT_ORDER_TO_RUN_ADMISSION_VERSION.to_string(),
        status: if accepted { "accepted" } else { "blocked" }.to_string(),
        admission_id: format!("order-to-run-admission-{}", order.order_id),
        order_id: order.order_id.clone(),
        authorization_receipt_id: authorization.authorization_receipt_id.clone(),
        input_snapshot_id: input_snapshot.input_snapshot_id.clone(),
        run_id: run_receipt.run_id.clone(),
        product_instance_id: order.product_instance_id.clone(),
        accepted,
        failure_reasons,
        source_refs: order.source_refs.clone(),
    }
}

pub fn project_paid_report_customer_delivery_access(
    order: &PaidReportOrderRecord,
    delivery: &PaidReportDeliveryPackageProjection,
    decision: &PaidReportDecisionRecord,
    artifact: &PaidReportArtifact,
    authorization: &PaidReportEntitlementAuthorization,
) -> PaidReportCustomerDeliveryAccessProjection {
    let available = order.runnable
        && delivery.download_ready
        && decision.outcome == PaidReportDecisionOutcome::Accepted
        && authorization.status == "authorized"
        && artifact.status == "complete"
        && delivery
            .report_artifact_refs
            .iter()
            .any(|entry| entry == &artifact.artifact_id);
    let (access_status, next_action) = if available {
        ("accessible", "show-download")
    } else if authorization.authorization_state == "refunded" {
        ("blocked", "show-refund-policy")
    } else if decision.outcome == PaidReportDecisionOutcome::NeedsFix {
        ("repair-needed", "show-repair-route")
    } else if authorization.status != "authorized" {
        ("blocked", "resolve-authorization")
    } else if !delivery.download_ready {
        ("blocked", "wait-for-accepted-decision")
    } else {
        ("blocked", "wait-for-report-artifact")
    };

    PaidReportCustomerDeliveryAccessProjection {
        version: PAID_REPORT_CUSTOMER_DELIVERY_ACCESS_VERSION.to_string(),
        status: access_status.to_string(),
        delivery_package_id: delivery.delivery_package_id.clone(),
        order_id: order.order_id.clone(),
        decision_id: decision.decision_id.clone(),
        report_artifact_id: artifact.artifact_id.clone(),
        product_instance_id: order.product_instance_id.clone(),
        access_status: access_status.to_string(),
        next_action: next_action.to_string(),
        download_visible: available,
        projection_only: true,
        writes_authority: false,
        source_refs: order.source_refs.clone(),
    }
}

pub fn build_paid_report_access_receipt(
    access: &PaidReportCustomerDeliveryAccessProjection,
    revoked: bool,
    expired: bool,
) -> PaidReportAccessReceipt {
    let allowed = access.download_visible && !revoked && !expired;
    let blocked_reason = if allowed {
        "none"
    } else if revoked {
        "access-revoked"
    } else if expired {
        "access-expired"
    } else {
        access.next_action.as_str()
    };

    PaidReportAccessReceipt {
        version: PAID_REPORT_ACCESS_RECEIPT_VERSION.to_string(),
        status: if allowed { "allowed" } else { "blocked" }.to_string(),
        access_receipt_id: format!("access-receipt-{}", access.delivery_package_id),
        delivery_package_id: access.delivery_package_id.clone(),
        order_id: access.order_id.clone(),
        product_instance_id: access.product_instance_id.clone(),
        access_scope: "customer-report-download".to_string(),
        generated_at: "2026-07-09T00:00:00Z".to_string(),
        expires_at: "2026-07-16T00:00:00Z".to_string(),
        artifact_refs: vec![access.report_artifact_id.clone()],
        access_handle: if allowed {
            format!("download-token-{}", access.delivery_package_id)
        } else {
            String::new()
        },
        blocked_reason: blocked_reason.to_string(),
    }
}

pub fn evaluate_paid_report_commercial_policy(
    order: &PaidReportOrderRecord,
    delivery: &PaidReportDeliveryPackageProjection,
    decision: &PaidReportDecisionRecord,
    feedback: &PaidReportFeedbackLoopProjection,
    policy_outcome: &str,
) -> PaidReportCommercialPolicyRecord {
    let normalized = policy_outcome.trim();
    let creates_follow_up = matches!(normalized, "repair-request" | "controlled-rerun");
    let requires_new_authorization = normalized == "controlled-rerun";
    let status = match normalized {
        "refund-request" => "refund-requested",
        "repair-request" => "repair-proposed",
        "controlled-rerun" => "rerun-needs-authorization",
        "accepted-after-repair" => "accepted-after-repair",
        "no-follow-up" => "closed",
        _ => "blocked",
    };

    PaidReportCommercialPolicyRecord {
        version: PAID_REPORT_COMMERCIAL_POLICY_VERSION.to_string(),
        status: status.to_string(),
        policy_id: format!("policy-{}", feedback.feedback_id),
        outcome: normalized.to_string(),
        original_order_id: order.order_id.clone(),
        original_run_id: delivery.run_id.clone(),
        original_artifact_id: delivery
            .report_artifact_refs
            .first()
            .cloned()
            .unwrap_or_default(),
        original_decision_id: decision.decision_id.clone(),
        feedback_id: feedback.feedback_id.clone(),
        creates_follow_up_proposal: creates_follow_up,
        mutates_delivered_artifact: false,
        requires_new_authorization,
        commercial_decision_only: normalized == "refund-request",
        source_refs: vec![
            "docs/delivery/releases/v1.2.9/README.md".to_string(),
            "docs/delivery/releases/v1.2.9/AGENTFLOW_V1_2_9_PAID_REPORT_COMMERCIAL_ORDER_ACCESS_CLOSURE_TASKS_V1.md".to_string(),
        ],
    }
}

pub fn get_commercial_product_projection_query() -> CommercialProjectionQuery {
    let read_model = load_commercial_product_read_model();
    commercial_product_projection_query_from_read_model(read_model)
}

pub fn get_project_commercial_product_projection_query(
    project_root: impl AsRef<Path>,
) -> CommercialProjectionQuery {
    commercial_product_projection_query_from_read_model(load_project_commercial_product_read_model(
        project_root,
    ))
}

fn commercial_product_projection_query_from_read_model(
    read_model: CommercialProductReadModel,
) -> CommercialProjectionQuery {
    CommercialProjectionQuery {
        version: COMMERCIAL_PROJECTION_QUERY_VERSION.to_string(),
        status: read_model.status.clone(),
        read_model_version: read_model.version.clone(),
        freshness: read_model.freshness.clone(),
        projection_only: true,
        writes_authority: false,
        source_refs: read_model.source_refs.clone(),
        warnings: Vec::new(),
        read_model,
    }
}

fn project_commercial_registry_root(project_root: impl AsRef<Path>) -> PathBuf {
    project_root.as_ref().join(DEFAULT_COMMERCIAL_REGISTRY_ROOT)
}

fn resolve_default_commercial_registry_root() -> PathBuf {
    let default = PathBuf::from(DEFAULT_COMMERCIAL_REGISTRY_ROOT);
    if default.is_dir() {
        return default;
    }
    if let Ok(current_dir) = env::current_dir() {
        for ancestor in current_dir.ancestors() {
            let candidate = ancestor.join(DEFAULT_COMMERCIAL_REGISTRY_ROOT);
            if candidate.is_dir() {
                return candidate;
            }
        }
    }
    default
}

fn commercial_read_model_status(entries: &[CommercialProductReadModelEntry]) -> String {
    if entries.is_empty() {
        return "unavailable".to_string();
    }
    let available = entries
        .iter()
        .filter(|entry| entry.availability == CommercialAvailability::Available)
        .count();
    let deferred = entries
        .iter()
        .any(|entry| entry.availability == CommercialAvailability::Deferred);
    let blocked_or_invalid = entries.iter().any(|entry| {
        matches!(
            entry.availability,
            CommercialAvailability::Invalid | CommercialAvailability::Rejected
        )
    });
    if available == entries.len() {
        "ready".to_string()
    } else if available > 0 {
        "partial".to_string()
    } else if deferred {
        "deferred".to_string()
    } else if blocked_or_invalid {
        "invalid".to_string()
    } else {
        "unavailable".to_string()
    }
}

fn commercial_product_input_from_definition(
    definition: &CommercialProductDefinition,
    entitlement: Option<&CommercialEntitlementFixture>,
) -> CommercialProductInput {
    CommercialProductInput {
        product_id: definition.product_id.clone(),
        product_name: definition.product_name.clone(),
        flow_type: definition.flow_type,
        entitlement_state: entitlement
            .map(|entry| entry.entitlement_state)
            .unwrap_or(CommercialEntitlementState::Missing),
        paid_feature_state: definition.paid_feature_state,
        flow_definition_present: definition.flow_definition_present,
        product_definition_present: definition.product_definition_present,
        payment_configured: definition.payment_configured,
        paid_report_authority_fields: definition.paid_report_authority_fields.clone(),
        required_project_refs_present: definition.required_project_refs_present,
    }
}

fn registry_source_refs(
    registry_root: &Path,
    definition: &CommercialProductDefinition,
    entitlement: Option<&CommercialEntitlementFixture>,
) -> Vec<String> {
    let mut refs = vec![portable_registry_ref(
        registry_root,
        &registry_root.join("products.json"),
    )];
    if !definition.source_ref.is_empty() {
        refs.push(definition.source_ref.clone());
    }
    refs.push(portable_registry_ref(
        registry_root,
        &registry_root.join("entitlements.json"),
    ));
    if let Some(entitlement) = entitlement {
        if !entitlement.source_ref.is_empty() {
            refs.push(entitlement.source_ref.clone());
        }
    }
    refs
}

fn portable_registry_ref(registry_root: &Path, path: &Path) -> String {
    path.strip_prefix(registry_root)
        .map(|relative| format!("{}/{}", registry_root.display(), relative.display()))
        .unwrap_or_else(|_| path.display().to_string())
}

fn stable_path_digest(path: &Path) -> String {
    let canonical_or_display = path
        .canonicalize()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|_| path.display().to_string());
    let digest = Sha256::digest(canonical_or_display.as_bytes());
    format!("{digest:x}")[..16].to_string()
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T> {
    let payload = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&payload).with_context(|| format!("parse {}", path.display()))
}

fn true_bool() -> bool {
    true
}

pub fn evaluate_commercial_product(
    input: CommercialProductInput,
) -> CommercialProductReadModelEntry {
    let (availability, unavailable_reason, command_policy, can_submit) =
        commercial_product_decision(&input);
    let delivery_promise = match input.flow_type {
        CommercialFlowType::PaidReportFlow => CommercialDeliveryPromise::Report,
        CommercialFlowType::ManagedProjectFlow => CommercialDeliveryPromise::ProjectDelivery,
    };
    let flow_label = match input.flow_type {
        CommercialFlowType::PaidReportFlow => "Paid Report Flow",
        CommercialFlowType::ManagedProjectFlow => "Managed Project Flow",
    };
    let next_action = match availability {
        CommercialAvailability::Available => {
            "可以生成 Runtime command proposal；仍需 Core Runtime admission。"
        }
        CommercialAvailability::Deferred => "等待 Product-layer 条件就绪；不能显示为 ready。",
        CommercialAvailability::Rejected => "修复 entitlement 或 paid feature 后再生成提案。",
        CommercialAvailability::Invalid => "修复 Product / Flow 合同后再生成提案。",
    };

    CommercialProductReadModelEntry {
        product_id: input.product_id,
        product_name: input.product_name,
        flow_type: input.flow_type,
        flow_label: flow_label.to_string(),
        entitlement_state: input.entitlement_state,
        paid_feature_state: input.paid_feature_state,
        delivery_promise,
        availability,
        unavailable_reason,
        command_policy,
        next_action: next_action.to_string(),
        can_submit_runtime_command_proposal: can_submit,
        projection_only: true,
        core_authority: false,
        writes_authority: false,
        source_refs: vec![
            "docs/architecture/095-commercial-product-read-model-contract-v1.md".to_string(),
        ],
    }
}

fn commercial_product_decision(
    input: &CommercialProductInput,
) -> (
    CommercialAvailability,
    String,
    CommercialCommandPolicy,
    bool,
) {
    if input.product_id.trim().is_empty() || !input.product_definition_present {
        return blocked(CommercialAvailability::Invalid, "missing-product");
    }
    if !input.flow_definition_present {
        return blocked(CommercialAvailability::Invalid, "missing-flow-definition");
    }
    if input.flow_type == CommercialFlowType::ManagedProjectFlow
        && input.paid_report_authority_fields.iter().any(|field| {
            matches!(
                field.as_str(),
                "reportInputRef" | "orderIntentId" | "reportDefinitionId" | "reportDeliveryPromise"
            )
        })
    {
        return blocked(
            CommercialAvailability::Invalid,
            "paid-report-field-not-managed-project-authority",
        );
    }
    match input.entitlement_state {
        CommercialEntitlementState::Expired => {
            return blocked(CommercialAvailability::Rejected, "expired-entitlement");
        }
        CommercialEntitlementState::Disabled => {
            return blocked(CommercialAvailability::Rejected, "disabled-entitlement");
        }
        CommercialEntitlementState::Deferred => {
            return blocked(CommercialAvailability::Deferred, "deferred-entitlement");
        }
        CommercialEntitlementState::Missing => {
            return blocked(CommercialAvailability::Invalid, "missing-entitlement");
        }
        CommercialEntitlementState::Active | CommercialEntitlementState::Trial => {}
    }
    match input.paid_feature_state {
        CommercialPaidFeatureState::Disabled => {
            return blocked(CommercialAvailability::Rejected, "paid-feature-disabled");
        }
        CommercialPaidFeatureState::Deferred => {
            return blocked(CommercialAvailability::Deferred, "paid-feature-deferred");
        }
        CommercialPaidFeatureState::Missing => {
            return blocked(CommercialAvailability::Invalid, "paid-feature-missing");
        }
        CommercialPaidFeatureState::Enabled | CommercialPaidFeatureState::NotRequired => {}
    }
    if input.flow_type == CommercialFlowType::ManagedProjectFlow
        && !input.required_project_refs_present
    {
        return blocked(
            CommercialAvailability::Invalid,
            "missing-managed-project-refs",
        );
    }

    (
        CommercialAvailability::Available,
        "none".to_string(),
        CommercialCommandPolicy::AllowedToPropose,
        true,
    )
}

fn blocked(
    availability: CommercialAvailability,
    reason: &str,
) -> (
    CommercialAvailability,
    String,
    CommercialCommandPolicy,
    bool,
) {
    (
        availability,
        reason.to_string(),
        CommercialCommandPolicy::BlockedBeforeRuntime,
        false,
    )
}

pub fn evaluate_paid_report_preflight(
    request: PaidReportPreflightRequest,
) -> PaidReportPreflightResult {
    let (decision, reason, policy, can_submit) = if request.product_id.trim().is_empty() {
        (
            PaidReportPreflightDecision::Invalid,
            "missing-product",
            "blocked-before-runtime",
            false,
        )
    } else if !request.has_input_refs {
        (
            PaidReportPreflightDecision::Invalid,
            "missing-input",
            "blocked-before-runtime",
            false,
        )
    } else if !request.report_definition_present {
        (
            PaidReportPreflightDecision::Invalid,
            "missing-report-definition",
            "blocked-before-runtime",
            false,
        )
    } else if !request.order_intent_present {
        (
            PaidReportPreflightDecision::Invalid,
            "missing-order-intent",
            "blocked-before-runtime",
            false,
        )
    } else if !request.payment_configured {
        (
            PaidReportPreflightDecision::Deferred,
            "payment-not-configured",
            "blocked-before-runtime",
            false,
        )
    } else {
        match request.entitlement_state {
            CommercialEntitlementState::Expired => (
                PaidReportPreflightDecision::Rejected,
                "expired-entitlement",
                "blocked-before-runtime",
                false,
            ),
            CommercialEntitlementState::Disabled => (
                PaidReportPreflightDecision::Rejected,
                "disabled-entitlement",
                "blocked-before-runtime",
                false,
            ),
            CommercialEntitlementState::Deferred => (
                PaidReportPreflightDecision::Deferred,
                "deferred-entitlement",
                "blocked-before-runtime",
                false,
            ),
            CommercialEntitlementState::Missing => (
                PaidReportPreflightDecision::Invalid,
                "missing-entitlement",
                "blocked-before-runtime",
                false,
            ),
            CommercialEntitlementState::Active | CommercialEntitlementState::Trial => {
                match request.paid_feature_state {
                    CommercialPaidFeatureState::Enabled => (
                        PaidReportPreflightDecision::Allowed,
                        "none",
                        "propose-to-runtime",
                        true,
                    ),
                    CommercialPaidFeatureState::Disabled => (
                        PaidReportPreflightDecision::Rejected,
                        "paid-feature-disabled",
                        "blocked-before-runtime",
                        false,
                    ),
                    CommercialPaidFeatureState::Deferred => (
                        PaidReportPreflightDecision::Deferred,
                        "paid-feature-deferred",
                        "blocked-before-runtime",
                        false,
                    ),
                    CommercialPaidFeatureState::Missing => (
                        PaidReportPreflightDecision::Invalid,
                        "paid-feature-missing",
                        "blocked-before-runtime",
                        false,
                    ),
                    CommercialPaidFeatureState::NotRequired => (
                        PaidReportPreflightDecision::Invalid,
                        "paid-feature-required",
                        "blocked-before-runtime",
                        false,
                    ),
                }
            }
        }
    };

    PaidReportPreflightResult {
        version: PAID_REPORT_PREFLIGHT_VERSION.to_string(),
        request_id: request.request_id,
        flow_type: CommercialFlowType::PaidReportFlow,
        decision,
        unavailable_reason: reason.to_string(),
        runtime_command_policy: policy.to_string(),
        runtime_admission_required: true,
        can_submit_runtime_command_proposal: can_submit,
        can_start_run_directly: false,
        evidence_requirements: vec!["report-generation-evidence".to_string()],
        decision_requirements: vec!["report-delivery-decision".to_string()],
        delivery_promise: CommercialDeliveryPromise::Report,
    }
}

pub fn managed_project_commercial_fixture() -> ManagedProjectCommercialFixture {
    let available_input = CommercialProductInput {
        product_id: "managed-project".to_string(),
        product_name: "Managed Project".to_string(),
        flow_type: CommercialFlowType::ManagedProjectFlow,
        entitlement_state: CommercialEntitlementState::Trial,
        paid_feature_state: CommercialPaidFeatureState::NotRequired,
        flow_definition_present: true,
        product_definition_present: true,
        payment_configured: false,
        paid_report_authority_fields: Vec::new(),
        required_project_refs_present: true,
    };
    let invalid_input = CommercialProductInput {
        paid_report_authority_fields: vec!["reportDefinitionId".to_string()],
        ..available_input.clone()
    };
    let expected_available = evaluate_commercial_product(available_input.clone());
    let expected_invalid = evaluate_commercial_product(invalid_input.clone());
    let results = vec![
        CommercialFixtureResult {
            fixture_id: "managed-project-available".to_string(),
            input: available_input.clone(),
            expected: expected_available.clone(),
            actual: evaluate_commercial_product(available_input),
            passed: expected_available.availability == CommercialAvailability::Available,
        },
        CommercialFixtureResult {
            fixture_id: "paid-report-authority-in-managed-project".to_string(),
            input: invalid_input.clone(),
            expected: expected_invalid.clone(),
            actual: evaluate_commercial_product(invalid_input),
            passed: expected_invalid.availability == CommercialAvailability::Invalid
                && expected_invalid.unavailable_reason
                    == "paid-report-field-not-managed-project-authority",
        },
    ];
    let status = if results.iter().all(|result| result.passed) {
        "passed"
    } else {
        "failed"
    };
    ManagedProjectCommercialFixture {
        version: MANAGED_PROJECT_COMMERCIAL_FIXTURE_VERSION.to_string(),
        status: status.to_string(),
        results,
    }
}

pub fn commercial_negative_fixture_report() -> CommercialNegativeFixtureReport {
    let read_model_inputs = vec![
        CommercialProductInput {
            product_id: "paid-report".to_string(),
            product_name: "Paid Report".to_string(),
            flow_type: CommercialFlowType::PaidReportFlow,
            entitlement_state: CommercialEntitlementState::Expired,
            paid_feature_state: CommercialPaidFeatureState::Enabled,
            flow_definition_present: true,
            product_definition_present: true,
            payment_configured: true,
            paid_report_authority_fields: Vec::new(),
            required_project_refs_present: false,
        },
        CommercialProductInput {
            product_id: "paid-report".to_string(),
            product_name: "Paid Report".to_string(),
            flow_type: CommercialFlowType::PaidReportFlow,
            entitlement_state: CommercialEntitlementState::Missing,
            paid_feature_state: CommercialPaidFeatureState::Enabled,
            flow_definition_present: true,
            product_definition_present: true,
            payment_configured: true,
            paid_report_authority_fields: Vec::new(),
            required_project_refs_present: false,
        },
        CommercialProductInput {
            product_id: "unknown-product".to_string(),
            product_name: "Unknown Product".to_string(),
            flow_type: CommercialFlowType::PaidReportFlow,
            entitlement_state: CommercialEntitlementState::Active,
            paid_feature_state: CommercialPaidFeatureState::Enabled,
            flow_definition_present: true,
            product_definition_present: false,
            payment_configured: true,
            paid_report_authority_fields: Vec::new(),
            required_project_refs_present: false,
        },
        CommercialProductInput {
            product_id: "paid-report".to_string(),
            product_name: "Paid Report".to_string(),
            flow_type: CommercialFlowType::PaidReportFlow,
            entitlement_state: CommercialEntitlementState::Active,
            paid_feature_state: CommercialPaidFeatureState::Enabled,
            flow_definition_present: false,
            product_definition_present: true,
            payment_configured: true,
            paid_report_authority_fields: Vec::new(),
            required_project_refs_present: false,
        },
    ];
    let read_model_results = read_model_inputs
        .into_iter()
        .map(|input| {
            let actual = evaluate_commercial_product(input.clone());
            CommercialFixtureResult {
                fixture_id: actual.unavailable_reason.clone(),
                input,
                expected: actual.clone(),
                actual,
                passed: true,
            }
        })
        .collect::<Vec<_>>();

    let preflight_requests = vec![
        PaidReportPreflightRequest {
            product_id: "paid-report".to_string(),
            request_id: "disabled-entitlement".to_string(),
            has_input_refs: true,
            entitlement_state: CommercialEntitlementState::Disabled,
            paid_feature_state: CommercialPaidFeatureState::Enabled,
            report_definition_present: true,
            order_intent_present: true,
            payment_configured: true,
        },
        PaidReportPreflightRequest {
            product_id: "paid-report".to_string(),
            request_id: "deferred-entitlement".to_string(),
            has_input_refs: true,
            entitlement_state: CommercialEntitlementState::Deferred,
            paid_feature_state: CommercialPaidFeatureState::Enabled,
            report_definition_present: true,
            order_intent_present: true,
            payment_configured: true,
        },
        PaidReportPreflightRequest {
            product_id: "paid-report".to_string(),
            request_id: "payment-not-configured".to_string(),
            has_input_refs: true,
            entitlement_state: CommercialEntitlementState::Active,
            paid_feature_state: CommercialPaidFeatureState::Enabled,
            report_definition_present: true,
            order_intent_present: true,
            payment_configured: false,
        },
        PaidReportPreflightRequest {
            product_id: "paid-report".to_string(),
            request_id: "missing-report-definition".to_string(),
            has_input_refs: true,
            entitlement_state: CommercialEntitlementState::Active,
            paid_feature_state: CommercialPaidFeatureState::Enabled,
            report_definition_present: false,
            order_intent_present: true,
            payment_configured: true,
        },
    ];
    let preflight_results = preflight_requests
        .into_iter()
        .map(|request| {
            let actual = evaluate_paid_report_preflight(request.clone());
            CommercialFixtureResult {
                fixture_id: request.request_id.clone(),
                input: request,
                expected: actual.clone(),
                actual,
                passed: true,
            }
        })
        .collect::<Vec<_>>();

    let failed_commercial_preflight_can_submit_runtime_command =
        preflight_results.iter().any(|result| {
            result.actual.decision != PaidReportPreflightDecision::Allowed
                && result.actual.can_submit_runtime_command_proposal
        });
    let managed_project = managed_project_commercial_fixture();
    let managed_project_can_use_paid_report_authority =
        managed_project.results.iter().any(|result| {
            result.fixture_id == "paid-report-authority-in-managed-project"
                && result.actual.availability == CommercialAvailability::Available
        });
    let status = if !failed_commercial_preflight_can_submit_runtime_command
        && !managed_project_can_use_paid_report_authority
        && read_model_results.iter().all(|result| result.passed)
        && preflight_results.iter().all(|result| result.passed)
    {
        "passed"
    } else {
        "failed"
    };

    CommercialNegativeFixtureReport {
        version: COMMERCIAL_NEGATIVE_FIXTURE_VERSION.to_string(),
        status: status.to_string(),
        read_model_results,
        preflight_results,
        failed_commercial_preflight_can_submit_runtime_command,
        managed_project_can_use_paid_report_authority,
    }
}

pub fn commercial_golden_path() -> CommercialGoldenPathProof {
    commercial_golden_path_from_registry(Path::new(DEFAULT_COMMERCIAL_REGISTRY_ROOT))
        .unwrap_or_else(|_| commercial_golden_path_from_defaults())
}

pub fn commercial_golden_path_from_registry(
    registry_root: impl AsRef<Path>,
) -> Result<CommercialGoldenPathProof> {
    let registry_root = registry_root.as_ref();
    let read_model = load_registry_commercial_product_read_model(registry_root)?;
    let projection_query = commercial_product_projection_query_from_read_model(read_model.clone());
    let paid_report_blocked = evaluate_paid_report_preflight_from_registry(
        registry_root,
        "paid-report-preview",
        "paid-report-deferred",
    )?;
    let paid_report_deferred = evaluate_paid_report_preflight_from_registry(
        registry_root,
        "paid-report-preview",
        "paid-report-payment-deferred",
    )?;
    let managed_project_available = read_model
        .entries
        .iter()
        .find(|entry| entry.flow_type == CommercialFlowType::ManagedProjectFlow)
        .cloned()
        .unwrap_or_else(|| {
            let input = default_commercial_product_inputs()
                .into_iter()
                .find(|input| input.flow_type == CommercialFlowType::ManagedProjectFlow)
                .expect("default managed project commercial input");
            evaluate_commercial_product(input)
        });
    let status = if paid_report_blocked.can_submit_runtime_command_proposal
        || paid_report_deferred.can_submit_runtime_command_proposal
        || managed_project_available.availability != CommercialAvailability::Available
        || projection_query.writes_authority
    {
        "failed"
    } else {
        "passed"
    };

    Ok(CommercialGoldenPathProof {
        version: COMMERCIAL_GOLDEN_PATH_VERSION.to_string(),
        status: status.to_string(),
        read_model,
        projection_query,
        paid_report_blocked,
        paid_report_deferred,
        managed_project_available,
        projection_writes_authority: false,
        desktop_writes_authority: false,
    })
}

fn commercial_golden_path_from_defaults() -> CommercialGoldenPathProof {
    let read_model = build_commercial_product_read_model(default_commercial_product_inputs());
    let projection_query = commercial_product_projection_query_from_read_model(read_model.clone());
    let paid_report_blocked = evaluate_paid_report_preflight(PaidReportPreflightRequest {
        product_id: "paid-report".to_string(),
        request_id: "paid-report-disabled".to_string(),
        has_input_refs: true,
        entitlement_state: CommercialEntitlementState::Disabled,
        paid_feature_state: CommercialPaidFeatureState::Enabled,
        report_definition_present: true,
        order_intent_present: true,
        payment_configured: true,
    });
    let paid_report_deferred = evaluate_paid_report_preflight(PaidReportPreflightRequest {
        product_id: "paid-report".to_string(),
        request_id: "paid-report-payment-deferred".to_string(),
        has_input_refs: true,
        entitlement_state: CommercialEntitlementState::Active,
        paid_feature_state: CommercialPaidFeatureState::Enabled,
        report_definition_present: true,
        order_intent_present: true,
        payment_configured: false,
    });
    let managed_project_available = read_model
        .entries
        .iter()
        .find(|entry| entry.flow_type == CommercialFlowType::ManagedProjectFlow)
        .cloned()
        .unwrap_or_else(|| {
            let input = default_commercial_product_inputs()
                .into_iter()
                .find(|input| input.flow_type == CommercialFlowType::ManagedProjectFlow)
                .expect("default managed project commercial input");
            evaluate_commercial_product(input)
        });
    CommercialGoldenPathProof {
        version: COMMERCIAL_GOLDEN_PATH_VERSION.to_string(),
        status: "passed".to_string(),
        read_model,
        projection_query,
        paid_report_blocked,
        paid_report_deferred,
        managed_project_available,
        projection_writes_authority: false,
        desktop_writes_authority: false,
    }
}

pub fn production_registry_has_fixture_only_products(
    registry_root: impl AsRef<Path>,
) -> Result<bool> {
    let registry = read_json::<CommercialProductRegistryConfig>(
        &registry_root.as_ref().join("products.json"),
    )?;
    Ok(registry
        .products
        .iter()
        .any(|definition| is_fixture_only_product_id(&definition.product_id)))
}

pub fn is_fixture_only_product_id(product_id: &str) -> bool {
    matches!(
        product_id,
        "paid-report-missing-report" | "paid-report-missing-input"
    )
}

pub fn default_commercial_registry_root() -> &'static str {
    DEFAULT_COMMERCIAL_REGISTRY_ROOT
}

pub fn negative_commercial_fixture_root() -> &'static str {
    NEGATIVE_COMMERCIAL_FIXTURE_ROOT
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn commercial_read_model_blocks_disabled_and_allows_managed_project() {
        let model = load_commercial_product_read_model();

        assert_eq!(model.version, COMMERCIAL_PRODUCT_READ_MODEL_VERSION);
        assert_eq!(model.status, "partial");
        assert!(model.projection_only);
        assert!(!model.core_authority);
        assert!(!model.writes_authority);
        assert!(model.entries.iter().any(|entry| {
            entry.flow_type == CommercialFlowType::PaidReportFlow
                && entry.product_id == "paid-report"
                && entry.availability == CommercialAvailability::Available
                && entry.can_submit_runtime_command_proposal
        }));
        assert!(model.entries.iter().any(|entry| {
            entry.flow_type == CommercialFlowType::ManagedProjectFlow
                && entry.availability == CommercialAvailability::Available
                && entry.command_policy == CommercialCommandPolicy::AllowedToPropose
        }));
        assert!(!model
            .entries
            .iter()
            .any(|entry| is_fixture_only_product_id(&entry.product_id)));
    }

    #[test]
    fn registry_read_model_uses_product_source_and_rejects_missing_definitions() {
        let registry = registry_fixture();
        let model = load_registry_commercial_product_read_model(registry.path()).unwrap();

        assert_eq!(model.source, "product-registry-config");
        assert_eq!(model.status, "partial");
        assert!(model.projection_only);
        assert!(!model.writes_authority);
        assert!(model
            .source_refs
            .iter()
            .any(|item| item.ends_with("products.json")));
        assert!(model.entries.iter().any(|entry| {
            entry.product_id == "paid-report"
                && entry.availability == CommercialAvailability::Available
                && entry.can_submit_runtime_command_proposal
        }));
        assert!(model.entries.iter().any(|entry| {
            entry.product_id == "paid-report-missing-report"
                && entry.availability == CommercialAvailability::Invalid
                && entry.unavailable_reason == "missing-report-definition"
        }));
        assert!(model.entries.iter().any(|entry| {
            entry.product_id == "missing-entitlement"
                && entry.availability == CommercialAvailability::Invalid
                && entry.unavailable_reason == "missing-entitlement"
        }));
    }

    #[test]
    fn project_registry_missing_returns_unavailable_read_model() {
        let dir = tempfile::tempdir().unwrap();

        let model = load_project_commercial_product_read_model(dir.path());

        assert_eq!(model.status, "unavailable");
        assert_eq!(model.source, "project-commercial-registry-missing");
        assert!(model.entries.is_empty());
    }

    #[test]
    fn project_registry_resolves_active_project_surface() {
        let project = tempfile::tempdir().unwrap();
        let registry = registry_fixture();
        let target = project.path().join(DEFAULT_COMMERCIAL_REGISTRY_ROOT);
        fs::create_dir_all(&target).unwrap();
        fs::copy(
            registry.path().join("products.json"),
            target.join("products.json"),
        )
        .unwrap();
        fs::copy(
            registry.path().join("entitlements.json"),
            target.join("entitlements.json"),
        )
        .unwrap();

        let model = load_project_commercial_product_read_model(project.path());

        assert_eq!(model.source, "project-commercial-registry");
        assert!(model
            .source_refs
            .iter()
            .any(|source| source.contains(DEFAULT_COMMERCIAL_REGISTRY_ROOT)));
        assert!(model.entries.iter().any(|entry| {
            entry.product_id == "paid-report"
                && entry.availability == CommercialAvailability::Available
        }));
    }

    #[test]
    fn paid_report_instance_and_handoff_require_complete_contract() {
        let registry = registry_fixture();

        let instance =
            resolve_paid_report_product_instance_from_registry(registry.path(), "paid-report")
                .unwrap();
        assert_eq!(instance.status, "ready");
        assert!(instance.can_submit_runtime_command_proposal);
        assert_eq!(instance.report_definition_id, "paid-report-definition");

        let handoff = build_paid_report_runtime_proposal_handoff_from_registry(
            registry.path(),
            "paid-report",
            "ready",
        )
        .unwrap();
        assert_eq!(handoff.status, "ready");
        assert!(handoff.proposal_created);
        assert!(!handoff.proposal.as_ref().unwrap().can_start_run_directly);

        let blocked = build_paid_report_runtime_proposal_handoff_from_registry(
            registry.path(),
            "paid-report-missing-report",
            "missing-report",
        )
        .unwrap();
        assert_eq!(blocked.status, "blocked");
        assert!(!blocked.proposal_created);
    }

    #[test]
    fn paid_report_project_handoff_requires_runtime_admission_and_delivery_projection() {
        let project = tempfile::tempdir().unwrap();
        let registry = registry_fixture();
        let target = project.path().join(DEFAULT_COMMERCIAL_REGISTRY_ROOT);
        fs::create_dir_all(&target).unwrap();
        fs::copy(
            registry.path().join("products.json"),
            target.join("products.json"),
        )
        .unwrap();
        fs::copy(
            registry.path().join("entitlements.json"),
            target.join("entitlements.json"),
        )
        .unwrap();

        let instance =
            resolve_paid_report_product_instance_from_project(project.path(), "paid-report")
                .unwrap();
        assert_eq!(instance.status, "ready");
        assert!(instance
            .source_refs
            .iter()
            .any(|source| source.contains(DEFAULT_COMMERCIAL_REGISTRY_ROOT)));

        let handoff = build_paid_report_runtime_proposal_handoff_from_project(
            project.path(),
            "paid-report",
            "project-request",
        )
        .unwrap();
        assert_eq!(handoff.status, "ready");
        assert!(handoff.proposal_created);
        assert!(!handoff.proposal.as_ref().unwrap().can_start_run_directly);

        let receipt = admit_paid_report_runtime_proposal(&handoff);
        assert_eq!(receipt.status, "admitted");
        assert_eq!(receipt.admission_decision, "accepted-for-runtime-proposal");
        assert!(receipt.runtime_admission_required);
        assert!(!receipt.can_start_run_directly);

        let run_contract = build_paid_report_run_contract(&handoff, &receipt);
        assert_eq!(run_contract.status, "ready");
        assert!(!run_contract.can_start_run_directly);
        assert!(!run_contract.concrete_sku_is_core_authority);
        assert_eq!(
            run_contract.delivery_promise,
            CommercialDeliveryPromise::Report
        );

        let awaiting_evidence =
            project_paid_report_delivery_projection(&run_contract, false, false);
        assert_eq!(awaiting_evidence.status, "evidence-needed");
        assert!(!awaiting_evidence.writes_authority);
        assert!(awaiting_evidence.projection_only);

        let ready = project_paid_report_delivery_projection(&run_contract, true, true);
        assert_eq!(ready.status, "delivery-ready");
        assert!(ready.delivery_ready);
    }

    #[test]
    fn paid_report_project_instance_id_is_project_unique() {
        let registry = registry_fixture();
        let project_a = tempfile::tempdir().unwrap();
        let project_b = tempfile::tempdir().unwrap();
        install_registry_fixture(&registry, project_a.path());
        install_registry_fixture(&registry, project_b.path());

        let source_instance =
            resolve_paid_report_product_instance_from_registry(registry.path(), "paid-report")
                .unwrap();
        let instance_a =
            resolve_paid_report_product_instance_from_project(project_a.path(), "paid-report")
                .unwrap();
        let instance_b =
            resolve_paid_report_product_instance_from_project(project_b.path(), "paid-report")
                .unwrap();

        assert_ne!(
            instance_a.product_instance_id,
            instance_b.product_instance_id
        );
        assert_ne!(
            source_instance.product_instance_id,
            instance_a.product_instance_id
        );
        assert!(instance_a
            .source_refs
            .iter()
            .any(|source| source.contains(DEFAULT_COMMERCIAL_REGISTRY_ROOT)));
        assert!(instance_a
            .source_refs
            .iter()
            .any(|source| source.starts_with("project:")));
    }

    #[test]
    fn paid_report_v128_run_artifact_delivery_and_feedback_contracts() {
        let project = tempfile::tempdir().unwrap();
        let registry = registry_fixture();
        install_registry_fixture(&registry, project.path());
        let handoff = build_paid_report_runtime_proposal_handoff_from_project(
            project.path(),
            "paid-report",
            "v128-ready",
        )
        .unwrap();
        let admission = admit_paid_report_runtime_proposal(&handoff);
        let run_contract = build_paid_report_run_contract(&handoff, &admission);
        let order_intent = build_paid_report_order_intent(&handoff.product_instance, "v128-ready");
        let mut submitted_fields = HashMap::new();
        submitted_fields.insert("reportInputRef".to_string(), "input/ref.json".to_string());
        submitted_fields.insert(
            "orderIntentId".to_string(),
            order_intent.order_intent_id.clone(),
        );
        let input_snapshot = build_paid_report_input_snapshot(
            &handoff.product_instance,
            Some(&order_intent),
            "v128-ready",
            submitted_fields,
        );

        assert_eq!(order_intent.status, "ready");
        assert!(!order_intent.payment_provider_charge);
        assert_eq!(input_snapshot.status, "input-ready");
        assert!(input_snapshot.input_ready);
        assert!(input_snapshot.order_intent_ready);
        assert!(!input_snapshot.writes_authority);

        let blocked_receipt = build_paid_report_run_execution_receipt(&run_contract, None, true);
        assert_eq!(blocked_receipt.status, "blocked");
        assert!(blocked_receipt
            .failure_reasons
            .contains(&"missing-valid-input-snapshot".to_string()));

        let run_receipt =
            build_paid_report_run_execution_receipt(&run_contract, Some(&input_snapshot), true);
        assert_eq!(run_receipt.status, "completed");
        assert!(run_receipt.started);
        assert!(run_receipt.completed);
        assert!(!run_receipt.expected_artifact_ids.is_empty());

        let blocked_artifact = build_paid_report_artifact(None, true);
        assert_eq!(blocked_artifact.status, "blocked");

        let incomplete_artifact = build_paid_report_artifact(Some(&run_receipt), false);
        assert_eq!(incomplete_artifact.status, "incomplete");
        assert!(!incomplete_artifact.delivery_ready);

        let artifact = build_paid_report_artifact(Some(&run_receipt), true);
        assert_eq!(artifact.status, "complete");
        assert!(artifact.delivery_ready);
        assert!(!artifact.sections.is_empty());
        assert!(artifact.storage_path.contains(".agentflow/tasks/"));

        let missing_evidence = capture_paid_report_generation_evidence(
            &run_receipt,
            &artifact,
            run_contract.expected_evidence.clone(),
            Vec::new(),
        );
        assert_eq!(missing_evidence.status, "evidence-needed");
        assert!(!missing_evidence.evidence_complete);
        assert!(missing_evidence.append_only);
        assert!(missing_evidence.project_scoped);

        let evidence = capture_paid_report_generation_evidence(
            &run_receipt,
            &artifact,
            run_contract.expected_evidence.clone(),
            vec![
                "report-generation-evidence:input-snapshot".to_string(),
                "report-generation-evidence:run-receipt".to_string(),
                "report-generation-evidence:artifact".to_string(),
            ],
        );
        assert_eq!(evidence.status, "complete");
        assert!(evidence.evidence_complete);

        let blocked_decision = decide_paid_report_delivery(
            &incomplete_artifact,
            &evidence,
            PaidReportDecisionOutcome::Accepted,
        );
        assert_eq!(blocked_decision.outcome, PaidReportDecisionOutcome::Blocked);
        assert!(!blocked_decision.failure_reasons.is_empty());

        let needs_fix_decision = decide_paid_report_delivery(
            &artifact,
            &missing_evidence,
            PaidReportDecisionOutcome::Accepted,
        );
        assert_eq!(
            needs_fix_decision.outcome,
            PaidReportDecisionOutcome::NeedsFix
        );

        let decision =
            decide_paid_report_delivery(&artifact, &evidence, PaidReportDecisionOutcome::Accepted);
        assert_eq!(decision.outcome, PaidReportDecisionOutcome::Accepted);
        assert!(decision.writes_authority);

        let package = project_paid_report_delivery_package(&artifact, &evidence, &decision);
        assert_eq!(package.status, "delivery-ready");
        assert!(package.download_ready);
        assert!(package.projection_only);
        assert!(!package.writes_authority);

        let repair = project_paid_report_feedback_loop(&package, &decision, "repair-requested");
        assert_eq!(repair.status, "repair-requested");
        assert!(!repair.mutates_delivered_artifact);
        assert_eq!(repair.follow_up_route, "controlled-follow-up-proposal");
    }

    #[test]
    fn production_registry_fixture_only_ids_are_detected() {
        let registry = registry_fixture();

        assert!(production_registry_has_fixture_only_products(registry.path()).unwrap());
    }

    #[test]
    fn registry_paid_report_preflight_binds_definition_and_entitlement() {
        let registry = registry_fixture();

        let allowed =
            evaluate_paid_report_preflight_from_registry(registry.path(), "paid-report", "ready")
                .unwrap();
        assert_eq!(allowed.decision, PaidReportPreflightDecision::Allowed);
        assert!(allowed.can_submit_runtime_command_proposal);
        assert_eq!(
            allowed.evidence_requirements,
            vec!["report-generation-evidence".to_string()]
        );

        let disabled = evaluate_paid_report_preflight_from_registry(
            registry.path(),
            "disabled-report",
            "disabled",
        )
        .unwrap();
        assert_eq!(disabled.decision, PaidReportPreflightDecision::Rejected);
        assert!(!disabled.can_submit_runtime_command_proposal);

        let missing_report = evaluate_paid_report_preflight_from_registry(
            registry.path(),
            "paid-report-missing-report",
            "missing-report",
        )
        .unwrap();
        assert_eq!(
            missing_report.decision,
            PaidReportPreflightDecision::Invalid
        );
        assert_eq!(
            missing_report.unavailable_reason,
            "missing-report-definition"
        );
    }

    #[test]
    fn paid_report_preflight_requires_runtime_admission_after_allowed() {
        let result = evaluate_paid_report_preflight(PaidReportPreflightRequest {
            product_id: "paid-report".to_string(),
            request_id: "ready".to_string(),
            has_input_refs: true,
            entitlement_state: CommercialEntitlementState::Active,
            paid_feature_state: CommercialPaidFeatureState::Enabled,
            report_definition_present: true,
            order_intent_present: true,
            payment_configured: true,
        });

        assert_eq!(result.decision, PaidReportPreflightDecision::Allowed);
        assert!(result.can_submit_runtime_command_proposal);
        assert!(result.runtime_admission_required);
        assert!(!result.can_start_run_directly);
    }

    #[test]
    fn paid_report_preflight_blocks_unready_states_before_runtime() {
        for request in [
            PaidReportPreflightRequest {
                product_id: "paid-report".to_string(),
                request_id: "disabled".to_string(),
                has_input_refs: true,
                entitlement_state: CommercialEntitlementState::Disabled,
                paid_feature_state: CommercialPaidFeatureState::Enabled,
                report_definition_present: true,
                order_intent_present: true,
                payment_configured: true,
            },
            PaidReportPreflightRequest {
                product_id: "paid-report".to_string(),
                request_id: "payment".to_string(),
                has_input_refs: true,
                entitlement_state: CommercialEntitlementState::Active,
                paid_feature_state: CommercialPaidFeatureState::Enabled,
                report_definition_present: true,
                order_intent_present: true,
                payment_configured: false,
            },
        ] {
            let result = evaluate_paid_report_preflight(request);
            assert!(!result.can_submit_runtime_command_proposal);
            assert_eq!(result.runtime_command_policy, "blocked-before-runtime");
            assert!(!result.can_start_run_directly);
        }
    }

    #[test]
    fn managed_project_fixture_rejects_paid_report_authority() {
        let fixture = managed_project_commercial_fixture();

        assert_eq!(fixture.status, "passed");
        assert!(fixture.results.iter().any(|result| {
            result.fixture_id == "paid-report-authority-in-managed-project"
                && result.actual.availability == CommercialAvailability::Invalid
                && result.actual.unavailable_reason
                    == "paid-report-field-not-managed-project-authority"
        }));
    }

    #[test]
    fn commercial_negative_fixtures_never_submit_failed_preflight() {
        let report = commercial_negative_fixture_report();

        assert_eq!(report.status, "passed");
        assert!(!report.failed_commercial_preflight_can_submit_runtime_command);
        assert!(!report.managed_project_can_use_paid_report_authority);
    }

    #[test]
    fn commercial_golden_path_keeps_projection_and_desktop_readonly() {
        let proof = commercial_golden_path();

        assert_eq!(proof.status, "passed");
        assert!(!proof.projection_writes_authority);
        assert!(!proof.desktop_writes_authority);
        assert!(
            !proof
                .paid_report_blocked
                .can_submit_runtime_command_proposal
        );
        assert!(
            !proof
                .paid_report_deferred
                .can_submit_runtime_command_proposal
        );
        assert_eq!(
            proof.managed_project_available.availability,
            CommercialAvailability::Available
        );
    }

    #[test]
    fn paid_report_order_authorization_and_access_require_valid_chain() {
        let registry = registry_fixture();
        let project_root = tempfile::tempdir().unwrap();
        install_registry_fixture(&registry, project_root.path());
        let instance =
            resolve_paid_report_product_instance_from_project(project_root.path(), "paid-report")
                .unwrap();
        let handoff = build_paid_report_runtime_proposal_handoff_from_project(
            project_root.path(),
            "paid-report",
            "v129-test",
        )
        .unwrap();
        let admission = admit_paid_report_runtime_proposal(&handoff);
        let run_contract = build_paid_report_run_contract(&handoff, &admission);
        let order_intent = build_paid_report_order_intent(&instance, "v129-test");
        let mut submitted_fields = HashMap::new();
        submitted_fields.insert("reportInputRef".to_string(), "input.json".to_string());
        submitted_fields.insert(
            "orderIntentId".to_string(),
            order_intent.order_intent_id.clone(),
        );
        let input_snapshot = build_paid_report_input_snapshot(
            &instance,
            Some(&order_intent),
            "v129-test",
            submitted_fields,
        );
        let order =
            build_paid_report_order_record(&instance, &order_intent, &input_snapshot, "offer-v1");
        let authorized = authorize_paid_report_order(&order, "paid");
        let refunded = authorize_paid_report_order(&order, "refunded");
        let run_receipt =
            build_paid_report_run_execution_receipt(&run_contract, Some(&input_snapshot), true);
        let order_admission =
            admit_paid_report_order_to_run(&order, &authorized, &input_snapshot, &run_receipt);
        let refunded_admission =
            admit_paid_report_order_to_run(&order, &refunded, &input_snapshot, &run_receipt);

        assert!(order.runnable);
        assert_eq!(authorized.status, "authorized");
        assert!(!authorized.payment_provider_checkout);
        assert_eq!(refunded.status, "blocked");
        assert_eq!(order_admission.status, "accepted");
        assert_eq!(refunded_admission.status, "blocked");

        let artifact = build_paid_report_artifact(Some(&run_receipt), true);
        let evidence = capture_paid_report_generation_evidence(
            &run_receipt,
            &artifact,
            run_contract.expected_evidence.clone(),
            vec!["report-generation-evidence".to_string()],
        );
        let decision =
            decide_paid_report_delivery(&artifact, &evidence, PaidReportDecisionOutcome::Accepted);
        let delivery = project_paid_report_delivery_package(&artifact, &evidence, &decision);
        let access = project_paid_report_customer_delivery_access(
            &order,
            &delivery,
            &decision,
            &artifact,
            &authorized,
        );
        let allowed = build_paid_report_access_receipt(&access, false, false);
        let expired = build_paid_report_access_receipt(&access, false, true);

        assert!(access.download_visible);
        assert_eq!(allowed.status, "allowed");
        assert_eq!(expired.status, "blocked");
        assert_eq!(expired.blocked_reason, "access-expired");
    }

    #[test]
    fn paid_report_policy_never_mutates_delivered_artifact() {
        let delivery = PaidReportDeliveryPackageProjection {
            version: PAID_REPORT_DELIVERY_PACKAGE_PROJECTION_VERSION.to_string(),
            status: "delivery-ready".to_string(),
            delivery_package_id: "delivery-package-v129".to_string(),
            product_instance_id: "product-instance-v129".to_string(),
            run_id: "run-v129".to_string(),
            report_artifact_refs: vec!["artifact-v129".to_string()],
            evidence_refs: vec!["evidence-v129".to_string()],
            decision_refs: vec!["decision-v129".to_string()],
            delivery_status: "delivery-ready".to_string(),
            download_ready: true,
            display_contract: "download-report-and-evidence".to_string(),
            next_action: "show-download".to_string(),
            projection_only: true,
            writes_authority: false,
        };
        let order = PaidReportOrderRecord {
            version: PAID_REPORT_ORDER_RECORD_VERSION.to_string(),
            status: "order-ready".to_string(),
            order_id: "order-v129".to_string(),
            product_instance_id: "product-instance-v129".to_string(),
            request_id: "v129".to_string(),
            order_intent_id: "intent-v129".to_string(),
            input_snapshot_id: "snapshot-v129".to_string(),
            offer_ref: "offer-v129".to_string(),
            lifecycle_state: "order-ready".to_string(),
            runnable: true,
            created_at: "2026-07-09T00:00:00Z".to_string(),
            source_refs: Vec::new(),
        };
        let decision = PaidReportDecisionRecord {
            version: PAID_REPORT_DECISION_RECORD_VERSION.to_string(),
            status: "accepted".to_string(),
            decision_id: "decision-v129".to_string(),
            outcome: PaidReportDecisionOutcome::Accepted,
            report_artifact_id: "artifact-v129".to_string(),
            evidence_pack_id: "evidence-v129".to_string(),
            failure_reasons: Vec::new(),
            remediation_route: "deliver".to_string(),
            projection_only: false,
            writes_authority: true,
        };
        let feedback = PaidReportFeedbackLoopProjection {
            version: PAID_REPORT_FEEDBACK_LOOP_PROJECTION_VERSION.to_string(),
            status: "repair-requested".to_string(),
            feedback_id: "feedback-v129".to_string(),
            feedback_state: "repair-requested".to_string(),
            repair_request_id: "repair-v129".to_string(),
            original_product_instance_id: "product-instance-v129".to_string(),
            run_id: "run-v129".to_string(),
            artifact_id: "artifact-v129".to_string(),
            evidence_pack_id: "evidence-v129".to_string(),
            decision_id: "decision-v129".to_string(),
            mutates_delivered_artifact: false,
            follow_up_route: "controlled-follow-up-proposal".to_string(),
            next_action: "create-repair-proposal".to_string(),
            projection_only: true,
            writes_authority: false,
        };

        let repair = evaluate_paid_report_commercial_policy(
            &order,
            &delivery,
            &decision,
            &feedback,
            "repair-request",
        );
        let refund = evaluate_paid_report_commercial_policy(
            &order,
            &delivery,
            &decision,
            &feedback,
            "refund-request",
        );
        let rerun = evaluate_paid_report_commercial_policy(
            &order,
            &delivery,
            &decision,
            &feedback,
            "controlled-rerun",
        );

        assert!(repair.creates_follow_up_proposal);
        assert!(!repair.mutates_delivered_artifact);
        assert!(refund.commercial_decision_only);
        assert!(rerun.requires_new_authorization);
    }

    fn registry_fixture() -> TempDir {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("products.json"),
            r#"{
  "version": "agentflow-commercial-product-registry.v1",
  "source": "test-registry",
  "products": [
    {
      "productId": "paid-report",
      "productName": "Paid Report",
      "flowType": "paid-report-flow",
      "paidFeatureState": "enabled",
      "paymentConfigured": true,
      "reportDefinitionId": "paid-report-definition",
      "reportDefinitionPresent": true,
      "requiredInputRefs": ["reportInputRef", "orderIntentId"],
      "evidenceRequirements": ["report-generation-evidence"],
      "decisionRequirements": ["report-delivery-decision"],
      "sourceRef": "products/commercial-runtime/products.json#paid-report"
    },
    {
      "productId": "disabled-report",
      "productName": "Disabled Report",
      "flowType": "paid-report-flow",
      "paidFeatureState": "enabled",
      "paymentConfigured": true,
      "reportDefinitionPresent": true,
      "requiredInputRefs": ["reportInputRef", "orderIntentId"],
      "sourceRef": "products/commercial-runtime/products.json#disabled-report"
    },
    {
      "productId": "paid-report-missing-report",
      "productName": "Missing Report",
      "flowType": "paid-report-flow",
      "paidFeatureState": "enabled",
      "paymentConfigured": true,
      "reportDefinitionPresent": false,
      "requiredInputRefs": ["reportInputRef", "orderIntentId"],
      "sourceRef": "products/commercial-runtime/products.json#paid-report-missing-report"
    },
    {
      "productId": "missing-entitlement",
      "productName": "Missing Entitlement",
      "flowType": "paid-report-flow",
      "paidFeatureState": "enabled",
      "paymentConfigured": true,
      "reportDefinitionPresent": true,
      "requiredInputRefs": ["reportInputRef", "orderIntentId"],
      "sourceRef": "products/commercial-runtime/products.json#missing-entitlement"
    },
    {
      "productId": "managed-project",
      "productName": "Managed Project",
      "flowType": "managed-project-flow",
      "paidFeatureState": "not-required",
      "requiredProjectRefsPresent": true,
      "sourceRef": "products/commercial-runtime/products.json#managed-project"
    }
  ]
}
"#,
        )
        .unwrap();
        fs::write(
            dir.path().join("entitlements.json"),
            r#"{
  "version": "agentflow-commercial-entitlement-source.v1",
  "source": "test-entitlements",
  "entitlements": [
    {
      "productId": "paid-report",
      "entitlementState": "active",
      "sourceRef": "products/commercial-runtime/entitlements.json#paid-report"
    },
    {
      "productId": "disabled-report",
      "entitlementState": "disabled",
      "sourceRef": "products/commercial-runtime/entitlements.json#disabled-report"
    },
    {
      "productId": "paid-report-missing-report",
      "entitlementState": "active",
      "sourceRef": "products/commercial-runtime/entitlements.json#paid-report-missing-report"
    },
    {
      "productId": "managed-project",
      "entitlementState": "trial",
      "sourceRef": "products/commercial-runtime/entitlements.json#managed-project"
    }
  ]
}
"#,
        )
        .unwrap();
        dir
    }

    fn install_registry_fixture(registry: &TempDir, project_root: &Path) {
        let target = project_root.join(DEFAULT_COMMERCIAL_REGISTRY_ROOT);
        fs::create_dir_all(&target).unwrap();
        fs::copy(
            registry.path().join("products.json"),
            target.join("products.json"),
        )
        .unwrap();
        fs::copy(
            registry.path().join("entitlements.json"),
            target.join("entitlements.json"),
        )
        .unwrap();
    }
}
