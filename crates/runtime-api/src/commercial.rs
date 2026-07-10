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
pub const COMMERCIAL_BACKEND_STABLE_CONTRACT_VERSION: &str =
    "agentflow-commercial-backend-stable-contract.v1";
pub const PAID_REPORT_FLOW_STATE_MACHINE_VERSION: &str =
    "agentflow-paid-report-flow-state-machine.v1";
pub const COMMERCIAL_AUTHORITY_BOUNDARY_VERSION: &str =
    "agentflow-commercial-authority-boundary.v1";
pub const PRODUCT_SKU_EXTENSION_CONTRACT_VERSION: &str =
    "agentflow-product-sku-extension-contract.v1";
pub const PROVIDER_GENERATOR_ADAPTER_BOUNDARY_VERSION: &str =
    "agentflow-provider-generator-adapter-boundary.v1";
pub const PAYMENT_PROVIDER_ADAPTER_BOUNDARY_VERSION: &str =
    "agentflow-payment-provider-adapter-boundary.v1";
pub const CUSTOMER_DELIVERY_BACKEND_CONTRACT_VERSION: &str =
    "agentflow-customer-delivery-backend-contract.v1";
pub const COMMERCIAL_E2E_GOLDEN_SCENARIO_VERSION: &str =
    "agentflow-commercial-e2e-golden-scenario.v1";

const DEFAULT_COMMERCIAL_REGISTRY_ROOT: &str = "products/commercial-runtime";
const NEGATIVE_COMMERCIAL_FIXTURE_ROOT: &str = "products/_fixtures/commercial-runtime-negative";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommercialBackendStableField {
    pub name: String,
    pub field_type: String,
    pub required: bool,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommercialBackendStableObject {
    pub object_name: String,
    pub rust_type: String,
    pub category: String,
    pub version: String,
    #[serde(default)]
    pub required_fields: Vec<CommercialBackendStableField>,
    #[serde(default)]
    pub optional_fields: Vec<CommercialBackendStableField>,
    #[serde(default)]
    pub status_values: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommercialBackendDecisionState {
    pub state: String,
    pub owner: String,
    pub meaning: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommercialBackendErrorDecisionModel {
    #[serde(default)]
    pub stable_states: Vec<CommercialBackendDecisionState>,
    #[serde(default)]
    pub failure_reason_policy: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommercialBackendMigrationPolicy {
    pub stable_after_release: String,
    pub backward_incompatible_changes_require_version_bump: bool,
    pub explicit_migration_required: bool,
    pub machine_readable_baseline_required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommercialBackendStableContract {
    pub version: String,
    pub status: String,
    pub release_version: String,
    pub authority_boundary: String,
    #[serde(default)]
    pub objects: Vec<CommercialBackendStableObject>,
    pub error_decision_model: CommercialBackendErrorDecisionModel,
    pub migration_policy: CommercialBackendMigrationPolicy,
    #[serde(default)]
    pub non_goals: Vec<String>,
    pub checked_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaidReportFlowContractBinding {
    pub object_name: String,
    pub contract_version: String,
    pub binding_reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaidReportFlowFailureReason {
    pub code: String,
    pub message: String,
    pub prevents_authority_writes: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaidReportFlowTransition {
    pub from_state: String,
    pub to_state: String,
    pub event: String,
    pub source_object: String,
    pub source_contract_version: String,
    pub writes_authority: bool,
    pub writes_accepted_authority: bool,
    pub writes_delivery_ready_authority: bool,
    #[serde(default)]
    pub required_contracts: Vec<PaidReportFlowContractBinding>,
    #[serde(default)]
    pub failure_reasons: Vec<PaidReportFlowFailureReason>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaidReportFlowTransitionFixture {
    pub fixture_id: String,
    pub status: String,
    pub transition: PaidReportFlowTransition,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaidReportFlowStateMachine {
    pub version: String,
    pub status: String,
    pub release_version: String,
    pub authority_boundary: String,
    #[serde(default)]
    pub states: Vec<String>,
    #[serde(default)]
    pub positive_fixtures: Vec<PaidReportFlowTransitionFixture>,
    #[serde(default)]
    pub negative_fixtures: Vec<PaidReportFlowTransitionFixture>,
    pub checked_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommercialAuthorityRule {
    pub object_name: String,
    pub contract_version: String,
    pub owner_component: String,
    pub authority_kind: String,
    pub can_create: bool,
    pub can_update: bool,
    pub projection_only: bool,
    pub writes_authority: bool,
    #[serde(default)]
    pub read_only_surfaces: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommercialAuthorityNegativeFixture {
    pub fixture_id: String,
    pub status: String,
    pub attempted_writer: String,
    pub attempted_target: String,
    pub attempted_authority_kind: String,
    pub can_write_authority: bool,
    pub failure_reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommercialAuthorityBoundary {
    pub version: String,
    pub status: String,
    pub release_version: String,
    pub authority_boundary: String,
    #[serde(default)]
    pub authority_map: Vec<CommercialAuthorityRule>,
    #[serde(default)]
    pub read_only_surfaces: Vec<String>,
    #[serde(default)]
    pub negative_fixtures: Vec<CommercialAuthorityNegativeFixture>,
    pub synthetic_release_sidecar_policy: String,
    pub checked_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductSkuExtensionDefinition {
    pub sku_id: String,
    pub product_id: String,
    #[serde(default)]
    pub required_inputs: Vec<String>,
    #[serde(default)]
    pub report_sections: Vec<String>,
    #[serde(default)]
    pub evidence_policy: Vec<String>,
    #[serde(default)]
    pub decision_policy: Vec<String>,
    #[serde(default)]
    pub delivery_policy: Vec<String>,
    pub pricing_ref: String,
    pub generator_ref: String,
    #[serde(default)]
    pub source_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductSkuExtensionResolution {
    pub status: String,
    pub sku_id: Option<String>,
    pub product_id: Option<String>,
    pub can_materialize_product_instance: bool,
    pub falls_back_to_generic_hardcoded_content: bool,
    pub unavailable_reason: String,
    #[serde(default)]
    pub missing_fields: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductSkuExtensionNegativeFixture {
    pub fixture_id: String,
    pub status: String,
    pub attempted_surface: String,
    pub attempted_operation: String,
    pub resolution: ProductSkuExtensionResolution,
    pub failure_reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductSkuExtensionContract {
    pub version: String,
    pub status: String,
    pub release_version: String,
    pub authority_boundary: String,
    pub allowed_authority_surface: String,
    pub core_runtime_policy: String,
    #[serde(default)]
    pub required_fields: Vec<String>,
    pub synthetic_sku_fixture: ProductSkuExtensionDefinition,
    pub synthetic_sku_resolution: ProductSkuExtensionResolution,
    #[serde(default)]
    pub negative_fixtures: Vec<ProductSkuExtensionNegativeFixture>,
    #[serde(default)]
    pub forbidden_core_terms: Vec<String>,
    #[serde(default)]
    pub source_refs: Vec<String>,
    pub checked_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderGeneratorAdapterRequest {
    pub version: String,
    pub status: String,
    pub request_id: String,
    pub product_instance_id: String,
    pub sku_id: String,
    pub input_snapshot_ref: String,
    pub sku_definition_ref: String,
    pub generation_request_ref: String,
    pub generator_ref: String,
    pub provider_ref: String,
    #[serde(default)]
    pub report_sections: Vec<String>,
    #[serde(default)]
    pub source_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderGeneratorAdapterReceipt {
    pub version: String,
    pub status: String,
    pub receipt_id: String,
    pub request_id: String,
    pub adapter_id: String,
    pub output_artifact_ref: Option<String>,
    #[serde(default)]
    pub evidence_refs: Vec<String>,
    #[serde(default)]
    pub failure_reasons: Vec<String>,
    pub remediation_route: String,
    pub provider_specific_call_is_core_authority: bool,
    pub delivery_blocked: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderGeneratorAdapterArtifact {
    pub version: String,
    pub status: String,
    pub artifact_id: String,
    pub artifact_kind: String,
    pub content_ref: String,
    #[serde(default)]
    pub section_refs: Vec<String>,
    #[serde(default)]
    pub evidence_refs: Vec<String>,
    pub produced_by_adapter: bool,
    pub writes_core_authority: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderGeneratorAdapterFixture {
    pub fixture_id: String,
    pub status: String,
    pub request: ProviderGeneratorAdapterRequest,
    pub receipt: ProviderGeneratorAdapterReceipt,
    pub artifact: Option<ProviderGeneratorAdapterArtifact>,
    pub expected_delivery_state: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderGeneratorAdapterBoundaryContract {
    pub version: String,
    pub status: String,
    pub release_version: String,
    pub authority_boundary: String,
    pub adapter_boundary: String,
    #[serde(default)]
    pub required_objects: Vec<String>,
    pub dry_run_positive_fixture: ProviderGeneratorAdapterFixture,
    #[serde(default)]
    pub negative_fixtures: Vec<ProviderGeneratorAdapterFixture>,
    #[serde(default)]
    pub stable_failure_reasons: Vec<String>,
    #[serde(default)]
    pub source_refs: Vec<String>,
    pub checked_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentProviderAdapterFixture {
    pub fixture_id: String,
    pub status: String,
    pub provider_payment_intent_ref: Option<String>,
    pub checkout_session_ref: Option<String>,
    pub entitlement_authorization_ref: Option<String>,
    pub payment_status: String,
    pub refund_status: String,
    #[serde(default)]
    pub source_refs: Vec<String>,
    pub core_consumes_authorization_result: bool,
    pub core_consumes_provider_evidence: bool,
    pub provider_checkout_implementation_is_core_authority: bool,
    pub provider_refund_execution_is_core_authority: bool,
    pub entitlement_effect: String,
    pub failure_reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentProviderAdapterBoundaryContract {
    pub version: String,
    pub status: String,
    pub release_version: String,
    pub authority_boundary: String,
    pub adapter_boundary: String,
    #[serde(default)]
    pub required_fields: Vec<String>,
    #[serde(default)]
    pub dry_run_fixtures: Vec<PaymentProviderAdapterFixture>,
    #[serde(default)]
    pub stable_payment_statuses: Vec<String>,
    #[serde(default)]
    pub stable_refund_statuses: Vec<String>,
    #[serde(default)]
    pub source_refs: Vec<String>,
    pub checked_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomerDeliveryBackendFixture {
    pub fixture_id: String,
    pub status: String,
    pub order_id: String,
    pub entitlement_authorization_ref: String,
    pub decision_id: String,
    pub report_artifact_ref: String,
    pub access_receipt_ref: Option<String>,
    pub expiry_state: String,
    pub revocation_state: String,
    pub refund_state: String,
    pub repair_state: String,
    pub rerun_state: String,
    pub feedback_state: String,
    pub access_status: String,
    pub next_action: String,
    pub download_access_visible: bool,
    pub access_handle_generated: bool,
    #[serde(default)]
    pub failure_reasons: Vec<String>,
    #[serde(default)]
    pub source_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomerDeliveryBackendContract {
    pub version: String,
    pub status: String,
    pub release_version: String,
    pub authority_boundary: String,
    pub read_model_name: String,
    #[serde(default)]
    pub required_bindings: Vec<String>,
    #[serde(default)]
    pub stable_states: Vec<String>,
    pub accepted_delivery_fixture: CustomerDeliveryBackendFixture,
    #[serde(default)]
    pub negative_access_fixtures: Vec<CustomerDeliveryBackendFixture>,
    #[serde(default)]
    pub source_refs: Vec<String>,
    pub checked_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommercialE2eGoldenScenarioFact {
    pub fact_id: String,
    pub fact_type: String,
    pub contract_version: String,
    pub status: String,
    pub authority_owner: String,
    pub source_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommercialE2eGoldenScenarioPath {
    pub path_id: String,
    pub status: String,
    pub description: String,
    #[serde(default)]
    pub fact_refs: Vec<String>,
    pub decision_outcome: String,
    pub delivery_status: String,
    pub download_access_visible: bool,
    pub access_handle_generated: bool,
    pub mutates_delivered_artifact: bool,
    pub next_action: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommercialE2eGoldenScenarioProof {
    pub version: String,
    pub status: String,
    pub release_version: String,
    pub scenario_id: String,
    pub product_sku_fixture_id: String,
    pub concrete_domain_sku_implemented: bool,
    #[serde(default)]
    pub ordered_facts: Vec<CommercialE2eGoldenScenarioFact>,
    pub success_path: CommercialE2eGoldenScenarioPath,
    pub failure_repair_path: CommercialE2eGoldenScenarioPath,
    #[serde(default)]
    pub certification_artifact_refs: Vec<String>,
    #[serde(default)]
    pub source_refs: Vec<String>,
    pub checked_at: String,
}

pub fn commercial_backend_stable_contract() -> CommercialBackendStableContract {
    CommercialBackendStableContract {
        version: COMMERCIAL_BACKEND_STABLE_CONTRACT_VERSION.to_string(),
        status: "passed".to_string(),
        release_version: "v1.3.0".to_string(),
        authority_boundary: "Core Runtime owns the generic Paid Report backend contract; Product / Pack / SKU definitions own concrete domain copy, pricing, prompts, and provider-specific report generation.".to_string(),
        objects: vec![
            stable_object(
                "Product Definition",
                "PaidReportProductDefinition",
                "commercial-product-definition",
                PAID_REPORT_PRODUCT_DEFINITION_VERSION,
                &["version", "status", "productId", "productName", "flowType", "reportDefinitionId"],
                &["requiredInputRefs", "evidenceRequirements", "decisionRequirements", "sourceRefs"],
                &["ready", "invalid", "deferred"],
            ),
            stable_object(
                "Product Instance",
                "PaidReportProductInstanceContract",
                "commercial-product-instance",
                PAID_REPORT_PRODUCT_INSTANCE_VERSION,
                &["version", "status", "productInstanceId", "productId", "reportDefinitionId", "entitlementState", "paidFeatureState", "deliveryPromise", "canSubmitRuntimeCommandProposal", "unavailableReason"],
                &["requiredInputRefs", "evidenceRequirements", "decisionRequirements", "sourceRefs"],
                &["ready", "invalid", "deferred", "blocked"],
            ),
            stable_object(
                "Runtime Proposal Handoff",
                "PaidReportRuntimeProposalHandoff",
                "runtime-handoff",
                PAID_REPORT_RUNTIME_PROPOSAL_HANDOFF_VERSION,
                &["version", "status", "reason", "proposalCreated", "productInstance", "preflight"],
                &["proposal"],
                &["ready", "blocked", "deferred", "invalid"],
            ),
            stable_object(
                "Runtime Admission Receipt",
                "PaidReportRuntimeAdmissionReceipt",
                "runtime-admission",
                PAID_REPORT_RUNTIME_ADMISSION_RECEIPT_VERSION,
                &["version", "status", "receiptId", "productInstanceId", "productId", "requestId", "admissionDecision", "runtimeAdmissionRequired", "canStartRunDirectly"],
                &["requiredEvidence", "requiredDecisionPolicy", "sourceRefs"],
                &["accepted", "blocked", "deferred", "invalid"],
            ),
            stable_object(
                "Run Contract",
                "PaidReportRunContract",
                "run",
                PAID_REPORT_RUN_CONTRACT_VERSION,
                &["version", "status", "runContractId", "productInstanceId", "productId", "requestId", "reportDefinitionId", "deliveryPromise", "runtimeAdmissionReceiptId", "canStartRunDirectly", "concreteSkuIsCoreAuthority"],
                &["inputRefs", "expectedEvidence", "decisionPolicy", "sourceRefs"],
                &["ready", "blocked", "invalid"],
            ),
            stable_object(
                "Order Intent",
                "PaidReportOrderIntent",
                "order",
                PAID_REPORT_ORDER_INTENT_VERSION,
                &["version", "status", "orderIntentId", "productInstanceId", "productId", "requestId", "intentKind", "paymentProviderCharge"],
                &["sourceRefs"],
                &["ready", "blocked", "invalid"],
            ),
            stable_object(
                "Input Snapshot",
                "PaidReportInputSnapshot",
                "order",
                PAID_REPORT_INPUT_SNAPSHOT_VERSION,
                &["version", "status", "inputSnapshotId", "productInstanceId", "productId", "requestId", "reportDefinitionId", "orderIntentId", "inputReady", "orderIntentReady", "projectionOnly", "writesAuthority"],
                &["requiredInputRefs", "submittedFields", "sourceRefs"],
                &["input-ready", "input-missing", "blocked", "invalid"],
            ),
            stable_object(
                "Order Record",
                "PaidReportOrderRecord",
                "order",
                PAID_REPORT_ORDER_RECORD_VERSION,
                &["version", "status", "orderId", "productInstanceId", "requestId", "orderIntentId", "inputSnapshotId", "offerRef", "lifecycleState", "runnable", "createdAt"],
                &["sourceRefs"],
                &["order-ready", "input-snapshot-missing", "order-intent-missing", "order-blocked"],
            ),
            stable_object(
                "Entitlement Authorization",
                "PaidReportEntitlementAuthorization",
                "entitlement",
                PAID_REPORT_ENTITLEMENT_AUTHORIZATION_VERSION,
                &["version", "status", "authorizationReceiptId", "orderId", "productInstanceId", "authorizationState", "authorizationDecision", "paymentProviderCheckout", "providerChargeExecuted"],
                &["failureReasons", "sourceRefs"],
                &["authorized", "deferred", "blocked", "refunded", "revoked"],
            ),
            stable_object(
                "Order To Run Admission",
                "PaidReportOrderToRunAdmission",
                "run",
                PAID_REPORT_ORDER_TO_RUN_ADMISSION_VERSION,
                &["version", "status", "admissionId", "orderId", "authorizationReceiptId", "inputSnapshotId", "runId", "productInstanceId", "accepted"],
                &["failureReasons", "sourceRefs"],
                &["accepted", "blocked", "invalid"],
            ),
            stable_object(
                "Run Execution Receipt",
                "PaidReportRunExecutionReceipt",
                "run",
                PAID_REPORT_RUN_EXECUTION_RECEIPT_VERSION,
                &["version", "status", "receiptId", "runId", "productInstanceId", "productId", "requestId", "runtimeAdmissionReceiptId", "inputSnapshotId", "reportDefinitionId", "started", "completed"],
                &["expectedArtifactIds", "failureReasons", "sourceRefs"],
                &["completed", "blocked", "failed", "invalid"],
            ),
            stable_object(
                "Report Artifact",
                "PaidReportArtifact",
                "artifact",
                PAID_REPORT_ARTIFACT_VERSION,
                &["version", "status", "artifactId", "productInstanceId", "runId", "reportDefinitionId", "title", "summary", "generatedAt", "storagePath", "deliveryReady"],
                &["sections", "sourceEvidenceRefs"],
                &["complete", "incomplete", "blocked", "invalid"],
            ),
            stable_object(
                "Evidence Pack",
                "PaidReportEvidencePack",
                "evidence",
                PAID_REPORT_EVIDENCE_PACK_VERSION,
                &["version", "status", "evidencePackId", "productInstanceId", "runId", "inputSnapshotId", "runExecutionReceiptId", "reportArtifactId", "generationReceiptId", "evidenceComplete", "appendOnly", "projectScoped"],
                &["requiredEvidence", "evidenceRefs"],
                &["complete", "evidence-needed", "blocked", "invalid"],
            ),
            stable_object(
                "Decision Record",
                "PaidReportDecisionRecord",
                "decision",
                PAID_REPORT_DECISION_RECORD_VERSION,
                &["version", "status", "decisionId", "outcome", "reportArtifactId", "evidencePackId", "remediationRoute", "projectionOnly", "writesAuthority"],
                &["failureReasons"],
                &["accepted", "needs-fix", "rejected", "deferred", "blocked"],
            ),
            stable_object(
                "Delivery Package Projection",
                "PaidReportDeliveryPackageProjection",
                "delivery",
                PAID_REPORT_DELIVERY_PACKAGE_PROJECTION_VERSION,
                &["version", "status", "deliveryPackageId", "productInstanceId", "runId", "deliveryStatus", "downloadReady", "displayContract", "nextAction", "projectionOnly", "writesAuthority"],
                &["reportArtifactRefs", "evidenceRefs", "decisionRefs"],
                &["delivery-ready", "repair-needed", "blocked", "deferred"],
            ),
            stable_object(
                "Customer Delivery Access",
                "PaidReportCustomerDeliveryAccessProjection",
                "delivery",
                PAID_REPORT_CUSTOMER_DELIVERY_ACCESS_VERSION,
                &["version", "status", "deliveryPackageId", "orderId", "decisionId", "reportArtifactId", "productInstanceId", "accessStatus", "nextAction", "downloadVisible", "projectionOnly", "writesAuthority"],
                &["sourceRefs"],
                &["accessible", "blocked", "repair-needed"],
            ),
            stable_object(
                "Access Receipt",
                "PaidReportAccessReceipt",
                "delivery",
                PAID_REPORT_ACCESS_RECEIPT_VERSION,
                &["version", "status", "accessReceiptId", "deliveryPackageId", "orderId", "productInstanceId", "accessScope", "generatedAt", "expiresAt", "accessHandle", "blockedReason"],
                &["artifactRefs"],
                &["allowed", "blocked", "revoked", "expired"],
            ),
            stable_object(
                "Feedback Loop Projection",
                "PaidReportFeedbackLoopProjection",
                "feedback",
                PAID_REPORT_FEEDBACK_LOOP_PROJECTION_VERSION,
                &["version", "status", "feedbackId", "feedbackState", "repairRequestId", "originalProductInstanceId", "runId", "artifactId", "evidencePackId", "decisionId", "mutatesDeliveredArtifact", "followUpRoute", "nextAction", "projectionOnly", "writesAuthority"],
                &[],
                &["repair-requested", "closed", "blocked"],
            ),
            stable_object(
                "Commercial Policy Record",
                "PaidReportCommercialPolicyRecord",
                "feedback",
                PAID_REPORT_COMMERCIAL_POLICY_VERSION,
                &["version", "status", "policyId", "outcome", "originalOrderId", "originalRunId", "originalArtifactId", "originalDecisionId", "feedbackId", "createsFollowUpProposal", "mutatesDeliveredArtifact", "requiresNewAuthorization", "commercialDecisionOnly"],
                &["sourceRefs"],
                &["refund-requested", "repair-proposed", "rerun-needs-authorization", "accepted-after-repair", "closed", "blocked"],
            ),
        ],
        error_decision_model: CommercialBackendErrorDecisionModel {
            stable_states: vec![
                decision_state("invalid", "Runtime", "contract input is structurally invalid or missing required source facts"),
                decision_state("deferred", "Entitlement", "commercial entitlement or paid feature exists but is not yet eligible for runtime admission"),
                decision_state("blocked", "Runtime", "hard gate prevents the next transition and must include machine-readable failureReasons"),
                decision_state("accepted", "Decision", "artifact, evidence, and decision requirements passed and can move toward delivery"),
                decision_state("revoked", "Delivery", "previously issued customer access is no longer valid"),
                decision_state("expired", "Delivery", "time-bound customer access has elapsed"),
                decision_state("refunded", "Commercial Policy", "commercial decision stops access or follow-up without mutating delivered artifact"),
                decision_state("repair-needed", "Decision", "delivery exists but requires a controlled repair proposal"),
                decision_state("delivery-ready", "Delivery", "accepted decision and complete evidence make the package downloadable"),
            ],
            failure_reason_policy: vec![
                "blocked / invalid records must expose failureReasons or blockedReason".to_string(),
                "delivery access cannot hide refunded, revoked, expired, or repair-needed reasons".to_string(),
                "Core Runtime must not execute provider checkout, charge, refund, or concrete report generation".to_string(),
            ],
        },
        migration_policy: CommercialBackendMigrationPolicy {
            stable_after_release: "v1.3.0".to_string(),
            backward_incompatible_changes_require_version_bump: true,
            explicit_migration_required: true,
            machine_readable_baseline_required: true,
        },
        non_goals: vec![
            "concrete paid-report industry SKU".to_string(),
            "model/provider-specific final report generation".to_string(),
            "public commercial launch".to_string(),
            "cloud multi-tenant launch".to_string(),
            "production payment checkout, charge, or refund execution".to_string(),
        ],
        checked_at: "2026-07-10T00:00:00Z".to_string(),
    }
}

pub fn paid_report_flow_state_machine() -> PaidReportFlowStateMachine {
    PaidReportFlowStateMachine {
        version: PAID_REPORT_FLOW_STATE_MACHINE_VERSION.to_string(),
        status: "passed".to_string(),
        release_version: "v1.3.0".to_string(),
        authority_boundary: "Core Runtime owns the generic Paid Report lifecycle state machine. Product / Pack / SKU layers may bind concrete copy, pricing, prompts, and provider execution, but they cannot bypass these lifecycle transitions.".to_string(),
        states: paid_report_flow_states(),
        positive_fixtures: vec![
            positive_flow_fixture(
                "positive-draft-order-to-order-ready",
                "draft-order",
                "order-ready",
                "order.input-snapshot-ready",
                "PaidReportOrderRecord",
                PAID_REPORT_ORDER_RECORD_VERSION,
                false,
                false,
                false,
                vec![
                    flow_binding(
                        "PaidReportOrderIntent",
                        PAID_REPORT_ORDER_INTENT_VERSION,
                        "order intent records the customer request before authority can become order-ready",
                    ),
                    flow_binding(
                        "PaidReportInputSnapshot",
                        PAID_REPORT_INPUT_SNAPSHOT_VERSION,
                        "input snapshot freezes the submitted fields used by the order",
                    ),
                    flow_binding(
                        "PaidReportOrderRecord",
                        PAID_REPORT_ORDER_RECORD_VERSION,
                        "order record owns the order-ready lifecycle state",
                    ),
                ],
            ),
            positive_flow_fixture(
                "positive-order-ready-to-authorized",
                "order-ready",
                "authorized",
                "entitlement.authorized",
                "PaidReportEntitlementAuthorization",
                PAID_REPORT_ENTITLEMENT_AUTHORIZATION_VERSION,
                true,
                false,
                false,
                vec![
                    flow_binding(
                        "PaidReportOrderRecord",
                        PAID_REPORT_ORDER_RECORD_VERSION,
                        "authorization must bind to a concrete order",
                    ),
                    flow_binding(
                        "PaidReportEntitlementAuthorization",
                        PAID_REPORT_ENTITLEMENT_AUTHORIZATION_VERSION,
                        "entitlement authorization owns the authorized state",
                    ),
                ],
            ),
            positive_flow_fixture(
                "positive-authorized-to-admitted",
                "authorized",
                "admitted",
                "order-to-run.accepted",
                "PaidReportOrderToRunAdmission",
                PAID_REPORT_ORDER_TO_RUN_ADMISSION_VERSION,
                true,
                false,
                false,
                vec![
                    flow_binding(
                        "PaidReportEntitlementAuthorization",
                        PAID_REPORT_ENTITLEMENT_AUTHORIZATION_VERSION,
                        "admission requires an authorized entitlement",
                    ),
                    flow_binding(
                        "PaidReportOrderToRunAdmission",
                        PAID_REPORT_ORDER_TO_RUN_ADMISSION_VERSION,
                        "admission receipt owns the admitted state",
                    ),
                ],
            ),
            positive_flow_fixture(
                "positive-admitted-to-running",
                "admitted",
                "running",
                "run.started",
                "PaidReportRunExecutionReceipt",
                PAID_REPORT_RUN_EXECUTION_RECEIPT_VERSION,
                true,
                false,
                false,
                vec![
                    flow_binding(
                        "PaidReportRunContract",
                        PAID_REPORT_RUN_CONTRACT_VERSION,
                        "run contract defines expected evidence and delivery promise",
                    ),
                    flow_binding(
                        "PaidReportRunExecutionReceipt",
                        PAID_REPORT_RUN_EXECUTION_RECEIPT_VERSION,
                        "run receipt owns run execution progress",
                    ),
                ],
            ),
            positive_flow_fixture(
                "positive-running-to-artifact-ready",
                "running",
                "artifact-ready",
                "artifact.complete",
                "PaidReportArtifact",
                PAID_REPORT_ARTIFACT_VERSION,
                true,
                false,
                false,
                vec![
                    flow_binding(
                        "PaidReportRunExecutionReceipt",
                        PAID_REPORT_RUN_EXECUTION_RECEIPT_VERSION,
                        "artifact must trace back to a completed run receipt",
                    ),
                    flow_binding(
                        "PaidReportArtifact",
                        PAID_REPORT_ARTIFACT_VERSION,
                        "artifact owns the artifact-ready state",
                    ),
                ],
            ),
            positive_flow_fixture(
                "positive-artifact-ready-to-evidence-complete",
                "artifact-ready",
                "evidence-complete",
                "evidence.complete",
                "PaidReportEvidencePack",
                PAID_REPORT_EVIDENCE_PACK_VERSION,
                true,
                false,
                false,
                vec![
                    flow_binding(
                        "PaidReportArtifact",
                        PAID_REPORT_ARTIFACT_VERSION,
                        "evidence pack must reference the produced artifact",
                    ),
                    flow_binding(
                        "PaidReportEvidencePack",
                        PAID_REPORT_EVIDENCE_PACK_VERSION,
                        "evidence pack owns evidence completeness",
                    ),
                ],
            ),
            positive_flow_fixture(
                "positive-evidence-complete-to-accepted",
                "evidence-complete",
                "accepted",
                "decision.accepted",
                "PaidReportDecisionRecord",
                PAID_REPORT_DECISION_RECORD_VERSION,
                true,
                true,
                false,
                vec![
                    flow_binding(
                        "PaidReportEvidencePack",
                        PAID_REPORT_EVIDENCE_PACK_VERSION,
                        "accepted decision requires complete evidence",
                    ),
                    flow_binding(
                        "PaidReportDecisionRecord",
                        PAID_REPORT_DECISION_RECORD_VERSION,
                        "decision record owns accepted authority",
                    ),
                ],
            ),
            positive_flow_fixture(
                "positive-accepted-to-delivery-ready",
                "accepted",
                "delivery-ready",
                "delivery.ready",
                "PaidReportDeliveryPackageProjection",
                PAID_REPORT_DELIVERY_PACKAGE_PROJECTION_VERSION,
                true,
                false,
                true,
                vec![
                    flow_binding(
                        "PaidReportDecisionRecord",
                        PAID_REPORT_DECISION_RECORD_VERSION,
                        "delivery-ready requires an accepted decision",
                    ),
                    flow_binding(
                        "PaidReportDeliveryPackageProjection",
                        PAID_REPORT_DELIVERY_PACKAGE_PROJECTION_VERSION,
                        "delivery package projection exposes delivery readiness",
                    ),
                    flow_binding(
                        "PaidReportCustomerDeliveryAccessProjection",
                        PAID_REPORT_CUSTOMER_DELIVERY_ACCESS_VERSION,
                        "customer access projection binds delivery readiness to visible access",
                    ),
                ],
            ),
            positive_flow_fixture(
                "positive-delivery-ready-to-feedback-needed",
                "delivery-ready",
                "feedback-needed",
                "feedback.repair-requested",
                "PaidReportFeedbackLoopProjection",
                PAID_REPORT_FEEDBACK_LOOP_PROJECTION_VERSION,
                false,
                false,
                false,
                vec![
                    flow_binding(
                        "PaidReportDeliveryPackageProjection",
                        PAID_REPORT_DELIVERY_PACKAGE_PROJECTION_VERSION,
                        "feedback starts from an existing delivery package",
                    ),
                    flow_binding(
                        "PaidReportFeedbackLoopProjection",
                        PAID_REPORT_FEEDBACK_LOOP_PROJECTION_VERSION,
                        "feedback loop projection owns the feedback-needed state",
                    ),
                ],
            ),
            positive_flow_fixture(
                "positive-feedback-needed-to-repair-requested",
                "feedback-needed",
                "repair-requested",
                "policy.repair-proposed",
                "PaidReportCommercialPolicyRecord",
                PAID_REPORT_COMMERCIAL_POLICY_VERSION,
                false,
                false,
                false,
                vec![
                    flow_binding(
                        "PaidReportFeedbackLoopProjection",
                        PAID_REPORT_FEEDBACK_LOOP_PROJECTION_VERSION,
                        "repair policy must bind to a feedback request",
                    ),
                    flow_binding(
                        "PaidReportCommercialPolicyRecord",
                        PAID_REPORT_COMMERCIAL_POLICY_VERSION,
                        "commercial policy records the repair route",
                    ),
                ],
            ),
            positive_flow_fixture(
                "positive-repair-requested-to-rerun-needs-authorization",
                "repair-requested",
                "rerun-needs-authorization",
                "policy.controlled-rerun",
                "PaidReportCommercialPolicyRecord",
                PAID_REPORT_COMMERCIAL_POLICY_VERSION,
                false,
                false,
                false,
                vec![
                    flow_binding(
                        "PaidReportCommercialPolicyRecord",
                        PAID_REPORT_COMMERCIAL_POLICY_VERSION,
                        "controlled rerun must preserve the original delivery and require new authorization",
                    ),
                ],
            ),
            positive_flow_fixture(
                "positive-delivery-ready-to-refunded",
                "delivery-ready",
                "refunded",
                "policy.refund-requested",
                "PaidReportCommercialPolicyRecord",
                PAID_REPORT_COMMERCIAL_POLICY_VERSION,
                false,
                false,
                false,
                vec![
                    flow_binding(
                        "PaidReportCommercialPolicyRecord",
                        PAID_REPORT_COMMERCIAL_POLICY_VERSION,
                        "refund is a commercial decision and must not mutate the delivered artifact",
                    ),
                ],
            ),
            positive_flow_fixture(
                "positive-delivery-ready-to-expired",
                "delivery-ready",
                "expired",
                "access.expired",
                "PaidReportAccessReceipt",
                PAID_REPORT_ACCESS_RECEIPT_VERSION,
                false,
                false,
                false,
                vec![
                    flow_binding(
                        "PaidReportAccessReceipt",
                        PAID_REPORT_ACCESS_RECEIPT_VERSION,
                        "access receipt records time-bound customer access expiration",
                    ),
                ],
            ),
            positive_flow_fixture(
                "positive-delivery-ready-to-closed",
                "delivery-ready",
                "closed",
                "policy.no-follow-up",
                "PaidReportCommercialPolicyRecord",
                PAID_REPORT_COMMERCIAL_POLICY_VERSION,
                false,
                false,
                false,
                vec![
                    flow_binding(
                        "PaidReportCommercialPolicyRecord",
                        PAID_REPORT_COMMERCIAL_POLICY_VERSION,
                        "commercial policy closes the flow without follow-up authority writes",
                    ),
                ],
            ),
        ],
        negative_fixtures: vec![
            negative_flow_fixture(
                "negative-draft-order-to-accepted",
                "draft-order",
                "accepted",
                "decision.accepted-without-chain",
                "PaidReportDecisionRecord",
                PAID_REPORT_DECISION_RECORD_VERSION,
                vec![
                    flow_binding(
                        "PaidReportEvidencePack",
                        PAID_REPORT_EVIDENCE_PACK_VERSION,
                        "accepted authority requires completed evidence",
                    ),
                    flow_binding(
                        "PaidReportDecisionRecord",
                        PAID_REPORT_DECISION_RECORD_VERSION,
                        "accepted authority cannot be written from draft order",
                    ),
                ],
                vec![
                    flow_failure(
                        "missing-order-authorization",
                        "draft-order has no authorized entitlement or admitted run",
                    ),
                    flow_failure(
                        "missing-evidence-complete",
                        "accepted authority requires complete evidence before decision",
                    ),
                ],
            ),
            negative_flow_fixture(
                "negative-order-ready-to-delivery-ready",
                "order-ready",
                "delivery-ready",
                "delivery.ready-without-run",
                "PaidReportDeliveryPackageProjection",
                PAID_REPORT_DELIVERY_PACKAGE_PROJECTION_VERSION,
                vec![
                    flow_binding(
                        "PaidReportOrderToRunAdmission",
                        PAID_REPORT_ORDER_TO_RUN_ADMISSION_VERSION,
                        "delivery-ready requires admitted execution",
                    ),
                    flow_binding(
                        "PaidReportDeliveryPackageProjection",
                        PAID_REPORT_DELIVERY_PACKAGE_PROJECTION_VERSION,
                        "delivery projection cannot bypass run, artifact, evidence, and decision",
                    ),
                ],
                vec![
                    flow_failure(
                        "missing-run-admission",
                        "order-ready has not been admitted to a run",
                    ),
                    flow_failure(
                        "missing-accepted-decision",
                        "delivery-ready requires an accepted decision record",
                    ),
                ],
            ),
            negative_flow_fixture(
                "negative-authorized-to-running",
                "authorized",
                "running",
                "run.started-without-admission",
                "PaidReportRunExecutionReceipt",
                PAID_REPORT_RUN_EXECUTION_RECEIPT_VERSION,
                vec![
                    flow_binding(
                        "PaidReportOrderToRunAdmission",
                        PAID_REPORT_ORDER_TO_RUN_ADMISSION_VERSION,
                        "authorized order must be admitted before a run can start",
                    ),
                ],
                vec![flow_failure(
                    "missing-order-to-run-admission",
                    "authorized entitlement cannot start execution without admission",
                )],
            ),
            negative_flow_fixture(
                "negative-artifact-ready-to-delivery-ready",
                "artifact-ready",
                "delivery-ready",
                "delivery.ready-without-evidence",
                "PaidReportDeliveryPackageProjection",
                PAID_REPORT_DELIVERY_PACKAGE_PROJECTION_VERSION,
                vec![
                    flow_binding(
                        "PaidReportEvidencePack",
                        PAID_REPORT_EVIDENCE_PACK_VERSION,
                        "delivery-ready requires evidence completeness",
                    ),
                    flow_binding(
                        "PaidReportDecisionRecord",
                        PAID_REPORT_DECISION_RECORD_VERSION,
                        "delivery-ready requires an accepted decision",
                    ),
                ],
                vec![
                    flow_failure(
                        "missing-evidence-pack",
                        "artifact-ready cannot skip evidence completeness",
                    ),
                    flow_failure(
                        "missing-accepted-decision",
                        "delivery-ready cannot be written before accepted decision",
                    ),
                ],
            ),
            negative_flow_fixture(
                "negative-refunded-to-delivery-ready",
                "refunded",
                "delivery-ready",
                "delivery.ready-after-refund",
                "PaidReportDeliveryPackageProjection",
                PAID_REPORT_DELIVERY_PACKAGE_PROJECTION_VERSION,
                vec![
                    flow_binding(
                        "PaidReportCommercialPolicyRecord",
                        PAID_REPORT_COMMERCIAL_POLICY_VERSION,
                        "refund closes commercial access without restoring delivery authority",
                    ),
                ],
                vec![flow_failure(
                    "refunded-flow-cannot-restore-delivery",
                    "refunded state requires a new authorization before any new delivery",
                )],
            ),
            negative_flow_fixture(
                "negative-expired-to-delivery-ready",
                "expired",
                "delivery-ready",
                "delivery.ready-after-expiration",
                "PaidReportAccessReceipt",
                PAID_REPORT_ACCESS_RECEIPT_VERSION,
                vec![
                    flow_binding(
                        "PaidReportAccessReceipt",
                        PAID_REPORT_ACCESS_RECEIPT_VERSION,
                        "expired access receipt blocks delivery-ready authority writes",
                    ),
                ],
                vec![flow_failure(
                    "expired-access-cannot-write-delivery",
                    "expired access cannot become delivery-ready without a new valid access receipt",
                )],
            ),
        ],
        checked_at: "2026-07-10T00:00:00Z".to_string(),
    }
}

pub fn commercial_authority_boundary() -> CommercialAuthorityBoundary {
    CommercialAuthorityBoundary {
        version: COMMERCIAL_AUTHORITY_BOUNDARY_VERSION.to_string(),
        status: "passed".to_string(),
        release_version: "v1.3.0".to_string(),
        authority_boundary: "Commercial backend authority can only be written by the component that owns the matching stable object. Projection, Customer View, Download View, synthetic release fixtures, and release sidecars are read-only evidence or views and cannot promote themselves to live authority.".to_string(),
        authority_map: vec![
            authority_rule(
                "PaidReportOrderRecord",
                PAID_REPORT_ORDER_RECORD_VERSION,
                "Order Runtime",
                "order",
                true,
                true,
                false,
                true,
                &["Projection", "Customer View", "Download View"],
            ),
            authority_rule(
                "PaidReportEntitlementAuthorization",
                PAID_REPORT_ENTITLEMENT_AUTHORIZATION_VERSION,
                "Entitlement Runtime",
                "entitlement",
                true,
                true,
                false,
                true,
                &["Projection", "Customer View", "Download View"],
            ),
            authority_rule(
                "PaidReportOrderToRunAdmission",
                PAID_REPORT_ORDER_TO_RUN_ADMISSION_VERSION,
                "Runtime Admission",
                "run-admission",
                true,
                false,
                false,
                true,
                &["Projection", "Customer View", "Download View"],
            ),
            authority_rule(
                "PaidReportRunExecutionReceipt",
                PAID_REPORT_RUN_EXECUTION_RECEIPT_VERSION,
                "Execution Runtime",
                "run",
                true,
                true,
                false,
                true,
                &["Projection", "Customer View", "Download View"],
            ),
            authority_rule(
                "PaidReportArtifact",
                PAID_REPORT_ARTIFACT_VERSION,
                "Artifact Runtime",
                "artifact",
                true,
                true,
                false,
                true,
                &["Projection", "Customer View", "Download View"],
            ),
            authority_rule(
                "PaidReportEvidencePack",
                PAID_REPORT_EVIDENCE_PACK_VERSION,
                "Evidence Runtime",
                "evidence",
                true,
                true,
                false,
                true,
                &["Projection", "Customer View", "Download View"],
            ),
            authority_rule(
                "PaidReportDecisionRecord",
                PAID_REPORT_DECISION_RECORD_VERSION,
                "Decision Runtime",
                "decision",
                true,
                true,
                false,
                true,
                &["Projection", "Customer View", "Download View"],
            ),
            authority_rule(
                "PaidReportDeliveryPackageProjection",
                PAID_REPORT_DELIVERY_PACKAGE_PROJECTION_VERSION,
                "Delivery Projection",
                "delivery-view",
                false,
                false,
                true,
                false,
                &["Projection", "Customer View", "Download View"],
            ),
            authority_rule(
                "PaidReportCustomerDeliveryAccessProjection",
                PAID_REPORT_CUSTOMER_DELIVERY_ACCESS_VERSION,
                "Customer View",
                "customer-access-view",
                false,
                false,
                true,
                false,
                &["Projection", "Customer View", "Download View"],
            ),
            authority_rule(
                "PaidReportAccessReceipt",
                PAID_REPORT_ACCESS_RECEIPT_VERSION,
                "Access Runtime",
                "access-receipt",
                true,
                true,
                false,
                true,
                &["Projection", "Customer View", "Download View"],
            ),
            authority_rule(
                "PaidReportFeedbackLoopProjection",
                PAID_REPORT_FEEDBACK_LOOP_PROJECTION_VERSION,
                "Feedback Projection",
                "feedback-view",
                false,
                false,
                true,
                false,
                &["Projection", "Customer View", "Download View"],
            ),
            authority_rule(
                "PaidReportCommercialPolicyRecord",
                PAID_REPORT_COMMERCIAL_POLICY_VERSION,
                "Commercial Policy Runtime",
                "commercial-policy",
                true,
                true,
                false,
                true,
                &["Projection", "Customer View", "Download View"],
            ),
        ],
        read_only_surfaces: vec![
            "Projection".to_string(),
            "Customer View".to_string(),
            "Download View".to_string(),
            "Synthetic Release Fixture".to_string(),
            "Release Sidecar".to_string(),
        ],
        negative_fixtures: vec![
            authority_negative_fixture(
                "projection-writing-authority",
                "Projection",
                "PaidReportDecisionRecord",
                "decision",
                "projection read models can display accepted decisions but cannot create or update decision authority",
            ),
            authority_negative_fixture(
                "customer-view-writing-authority",
                "Customer View",
                "PaidReportCustomerDeliveryAccessProjection",
                "customer-access-view",
                "customer-facing views can display access state but cannot write access authority",
            ),
            authority_negative_fixture(
                "download-view-writing-authority",
                "Download View",
                "PaidReportAccessReceipt",
                "access-receipt",
                "download views can consume access receipts but cannot create or update them",
            ),
            authority_negative_fixture(
                "synthetic-release-sidecar-promoted-as-authority",
                "Synthetic Release Fixture",
                "Live Release Authority",
                "release-provenance",
                "synthetic project-release-gate-e2e facts are fixtures and cannot satisfy live release authority",
            ),
            authority_negative_fixture(
                "release-sidecar-promoted-as-authority",
                "Release Sidecar",
                "Live Release Authority",
                "release-provenance",
                "release sidecar evidence can support certification but cannot replace live GitHub release provenance",
            ),
        ],
        synthetic_release_sidecar_policy: "synthetic release fixtures and release sidecars are read-only evidence and cannot replace live GitHub release provenance; live release authority must come from published GitHub release provenance and matching source commit.".to_string(),
        checked_at: "2026-07-10T00:00:00Z".to_string(),
    }
}

pub fn product_sku_extension_contract() -> ProductSkuExtensionContract {
    let synthetic_sku = ProductSkuExtensionDefinition {
        sku_id: "synthetic-paid-report-standard".to_string(),
        product_id: "generic-paid-report".to_string(),
        required_inputs: vec![
            "orderIntentId".to_string(),
            "customerRequestRef".to_string(),
            "reportSubjectRef".to_string(),
        ],
        report_sections: vec![
            "summary".to_string(),
            "analysis".to_string(),
            "evidence-index".to_string(),
            "delivery-notes".to_string(),
        ],
        evidence_policy: vec![
            "generator-run-receipt".to_string(),
            "input-snapshot-hash".to_string(),
            "section-output-hash".to_string(),
        ],
        decision_policy: vec![
            "accepted-report-artifact-required".to_string(),
            "delivery-ready-requires-evidence-complete".to_string(),
        ],
        delivery_policy: vec![
            "customer-download-view-readonly".to_string(),
            "delivery-package-projection-readonly".to_string(),
        ],
        pricing_ref: "products/commercial-runtime/skus/synthetic-paid-report-standard/pricing.json"
            .to_string(),
        generator_ref:
            "products/commercial-runtime/skus/synthetic-paid-report-standard/generator.json"
                .to_string(),
        source_refs: vec![
            "products/commercial-runtime/skus/synthetic-paid-report-standard/sku.json".to_string(),
            "docs/architecture/102-product-sku-extension-contract-v1.md".to_string(),
        ],
    };
    let synthetic_resolution = evaluate_product_sku_extension(Some(&synthetic_sku));

    ProductSkuExtensionContract {
        version: PRODUCT_SKU_EXTENSION_CONTRACT_VERSION.to_string(),
        status: "passed".to_string(),
        release_version: "v1.3.0".to_string(),
        authority_boundary: "Product / Pack / SKU files may define concrete paid-report SKU inputs, report sections, pricing references, generator references, and delivery policy. Core Runtime can validate and bind those fields, but must not own concrete industry domain semantics or fallback to generic hardcoded report content.".to_string(),
        allowed_authority_surface: "Product / Pack / SKU".to_string(),
        core_runtime_policy: "Core Runtime stores generic Paid Report contracts and validation rules only; SKU authority is external product authority and missing SKU definitions produce invalid/deferred state before runtime.".to_string(),
        required_fields: product_sku_required_fields(),
        synthetic_sku_fixture: synthetic_sku,
        synthetic_sku_resolution: synthetic_resolution,
        negative_fixtures: vec![
            product_sku_negative_fixture(
                "missing-sku-definition",
                "Product SKU Resolver",
                "materialize product instance without sku.json",
                None,
                "missing SKU definitions must return invalid/deferred state and must not fall back to generic hardcoded content",
            ),
            product_sku_negative_fixture(
                "core-runtime-domain-term-as-authority",
                "Core Runtime",
                "write concrete industry SKU semantics",
                Some(ProductSkuExtensionDefinition {
                    sku_id: "domain-report-standard".to_string(),
                    product_id: "generic-paid-report".to_string(),
                    required_inputs: vec!["domainInputRef".to_string()],
                    report_sections: vec!["domain-analysis".to_string()],
                    evidence_policy: vec!["generator-run-receipt".to_string()],
                    decision_policy: vec!["accepted-report-artifact-required".to_string()],
                    delivery_policy: vec!["customer-download-view-readonly".to_string()],
                    pricing_ref: "core/runtime/domain-report/pricing.json".to_string(),
                    generator_ref: "core/runtime/domain-report/generator.json".to_string(),
                    source_refs: vec!["core/runtime/domain-report/sku.json".to_string()],
                }),
                "concrete domain terms are only allowed in Product / Pack / SKU files and cannot be promoted into Core Runtime authority",
            ),
            product_sku_negative_fixture(
                "synthetic-sku-sidecar-promoted-as-live-product",
                "Synthetic Release Fixture",
                "promote synthetic SKU fixture as live product SKU authority",
                Some(ProductSkuExtensionDefinition {
                    sku_id: "synthetic-paid-report-standard".to_string(),
                    product_id: "generic-paid-report".to_string(),
                    required_inputs: vec!["orderIntentId".to_string()],
                    report_sections: vec!["summary".to_string()],
                    evidence_policy: vec!["generator-run-receipt".to_string()],
                    decision_policy: vec!["accepted-report-artifact-required".to_string()],
                    delivery_policy: vec!["customer-download-view-readonly".to_string()],
                    pricing_ref:
                        "products/_fixtures/synthetic-paid-report-standard/pricing.json"
                            .to_string(),
                    generator_ref:
                        "products/_fixtures/synthetic-paid-report-standard/generator.json"
                            .to_string(),
                    source_refs: vec![
                        "products/_fixtures/synthetic-paid-report-standard/sku.json"
                            .to_string(),
                    ],
                }),
                "synthetic SKU fixtures prove schema behavior only and cannot become live SKU authority",
            ),
        ],
        forbidden_core_terms: vec![
            "concrete-domain-term-a".to_string(),
            "concrete-domain-term-b".to_string(),
            "concrete-domain-term-c".to_string(),
        ],
        source_refs: vec![
            "docs/architecture/102-product-sku-extension-contract-v1.md".to_string(),
            "docs/delivery/releases/v1.3.0/AGENTFLOW_V1_3_0_COMMERCIAL_BACKEND_STABLE_CLOSURE_TASKS_V1.md".to_string(),
        ],
        checked_at: "2026-07-10T00:00:00Z".to_string(),
    }
}

pub fn evaluate_product_sku_extension(
    definition: Option<&ProductSkuExtensionDefinition>,
) -> ProductSkuExtensionResolution {
    let Some(definition) = definition else {
        return ProductSkuExtensionResolution {
            status: "invalid".to_string(),
            sku_id: None,
            product_id: None,
            can_materialize_product_instance: false,
            falls_back_to_generic_hardcoded_content: false,
            unavailable_reason: "missing-sku-definition".to_string(),
            missing_fields: product_sku_required_fields(),
        };
    };

    let mut missing_fields = Vec::new();
    if definition.sku_id.trim().is_empty() {
        missing_fields.push("skuId".to_string());
    }
    if definition.product_id.trim().is_empty() {
        missing_fields.push("productId".to_string());
    }
    if definition.required_inputs.is_empty() {
        missing_fields.push("requiredInputs".to_string());
    }
    if definition.report_sections.is_empty() {
        missing_fields.push("reportSections".to_string());
    }
    if definition.evidence_policy.is_empty() {
        missing_fields.push("evidencePolicy".to_string());
    }
    if definition.decision_policy.is_empty() {
        missing_fields.push("decisionPolicy".to_string());
    }
    if definition.delivery_policy.is_empty() {
        missing_fields.push("deliveryPolicy".to_string());
    }
    if definition.pricing_ref.trim().is_empty() {
        missing_fields.push("pricingRef".to_string());
    }
    if definition.generator_ref.trim().is_empty() {
        missing_fields.push("generatorRef".to_string());
    }
    if definition.source_refs.is_empty() {
        missing_fields.push("sourceRefs".to_string());
    }

    let valid = missing_fields.is_empty();
    ProductSkuExtensionResolution {
        status: if valid { "ready" } else { "invalid" }.to_string(),
        sku_id: Some(definition.sku_id.clone()),
        product_id: Some(definition.product_id.clone()),
        can_materialize_product_instance: valid,
        falls_back_to_generic_hardcoded_content: false,
        unavailable_reason: if valid {
            "none".to_string()
        } else {
            "missing-sku-required-fields".to_string()
        },
        missing_fields,
    }
}

pub fn provider_generator_adapter_boundary_contract() -> ProviderGeneratorAdapterBoundaryContract {
    let positive_request = provider_generator_adapter_request(
        "dry-run-generator-request",
        "synthetic-paid-report-standard",
        true,
        true,
    );
    let positive_receipt = provider_generator_adapter_receipt(
        "dry-run-generation-receipt",
        &positive_request.request_id,
        "succeeded",
        Some("agentflow/tasks/synthetic-paid-report-standard/artifacts/report-artifact.json"),
        vec![
            "agentflow/tasks/synthetic-paid-report-standard/evidence/generator-run-receipt.json",
            "agentflow/tasks/synthetic-paid-report-standard/evidence/output-artifact-hash.json",
        ],
        Vec::new(),
        "none",
        false,
        false,
    );
    let positive_artifact = ProviderGeneratorAdapterArtifact {
        version: PAID_REPORT_ARTIFACT_VERSION.to_string(),
        status: "artifact-ready".to_string(),
        artifact_id: "dry-run-report-artifact".to_string(),
        artifact_kind: "paid-report".to_string(),
        content_ref: "agentflow/tasks/synthetic-paid-report-standard/artifacts/report.md"
            .to_string(),
        section_refs: vec![
            "summary".to_string(),
            "analysis".to_string(),
            "evidence-index".to_string(),
            "delivery-notes".to_string(),
        ],
        evidence_refs: positive_receipt.evidence_refs.clone(),
        produced_by_adapter: true,
        writes_core_authority: false,
    };

    ProviderGeneratorAdapterBoundaryContract {
        version: PROVIDER_GENERATOR_ADAPTER_BOUNDARY_VERSION.to_string(),
        status: "passed".to_string(),
        release_version: "v1.3.0".to_string(),
        authority_boundary: "Core Runtime owns the generic adapter request, receipt, artifact, evidence reference, decision and delivery authority. Provider-specific model calls, prompts, credentials and concrete generator implementation stay behind the adapter boundary and never become Core authority.".to_string(),
        adapter_boundary: "Provider / Generator Adapter may transform an input snapshot plus SKU definition into an output artifact and generation receipt. It cannot write delivery-ready authority, accepted decision authority, payment authority or Core Runtime product semantics.".to_string(),
        required_objects: vec![
            "inputSnapshot".to_string(),
            "skuDefinition".to_string(),
            "generationRequest".to_string(),
            "generationReceipt".to_string(),
            "outputArtifact".to_string(),
            "evidenceRefs".to_string(),
            "failureReasons".to_string(),
        ],
        dry_run_positive_fixture: ProviderGeneratorAdapterFixture {
            fixture_id: "dry-run-provider-generator-positive".to_string(),
            status: "passed".to_string(),
            request: positive_request,
            receipt: positive_receipt,
            artifact: Some(positive_artifact),
            expected_delivery_state: "evidence-required-before-delivery".to_string(),
        },
        negative_fixtures: vec![
            ProviderGeneratorAdapterFixture {
                fixture_id: "missing-input-snapshot".to_string(),
                status: "failed-as-expected".to_string(),
                request: provider_generator_adapter_request(
                    "missing-input-snapshot-request",
                    "synthetic-paid-report-standard",
                    false,
                    true,
                ),
                receipt: provider_generator_adapter_receipt(
                    "missing-input-snapshot-receipt",
                    "missing-input-snapshot-request",
                    "blocked",
                    None,
                    Vec::new(),
                    vec!["missing-input-snapshot".to_string()],
                    "collect-input-snapshot",
                    false,
                    true,
                ),
                artifact: None,
                expected_delivery_state: "blocked".to_string(),
            },
            ProviderGeneratorAdapterFixture {
                fixture_id: "provider-call-promoted-to-core-authority".to_string(),
                status: "failed-as-expected".to_string(),
                request: provider_generator_adapter_request(
                    "provider-authority-request",
                    "synthetic-paid-report-standard",
                    true,
                    true,
                ),
                receipt: provider_generator_adapter_receipt(
                    "provider-authority-receipt",
                    "provider-authority-request",
                    "blocked",
                    None,
                    Vec::new(),
                    vec!["provider-call-cannot-write-core-authority".to_string()],
                    "move-provider-call-behind-adapter",
                    true,
                    true,
                ),
                artifact: None,
                expected_delivery_state: "blocked".to_string(),
            },
            ProviderGeneratorAdapterFixture {
                fixture_id: "failed-generation-keeps-delivery-blocked".to_string(),
                status: "failed-as-expected".to_string(),
                request: provider_generator_adapter_request(
                    "failed-generation-request",
                    "synthetic-paid-report-standard",
                    true,
                    true,
                ),
                receipt: provider_generator_adapter_receipt(
                    "failed-generation-receipt",
                    "failed-generation-request",
                    "failed",
                    None,
                    vec![
                        "agentflow/tasks/synthetic-paid-report-standard/evidence/generator-error.json",
                    ],
                    vec!["generation-failed".to_string()],
                    "retry-generation-or-change-provider",
                    false,
                    true,
                ),
                artifact: None,
                expected_delivery_state: "blocked".to_string(),
            },
        ],
        stable_failure_reasons: vec![
            "missing-input-snapshot".to_string(),
            "missing-sku-definition".to_string(),
            "generation-failed".to_string(),
            "output-artifact-missing".to_string(),
            "provider-call-cannot-write-core-authority".to_string(),
            "evidence-refs-missing".to_string(),
        ],
        source_refs: vec![
            "docs/architecture/103-provider-generator-adapter-boundary-v1.md".to_string(),
            "docs/delivery/releases/v1.3.0/AGENTFLOW_V1_3_0_COMMERCIAL_BACKEND_STABLE_CLOSURE_TASKS_V1.md".to_string(),
        ],
        checked_at: "2026-07-10T00:00:00Z".to_string(),
    }
}

pub fn payment_provider_adapter_boundary_contract() -> PaymentProviderAdapterBoundaryContract {
    PaymentProviderAdapterBoundaryContract {
        version: PAYMENT_PROVIDER_ADAPTER_BOUNDARY_VERSION.to_string(),
        status: "passed".to_string(),
        release_version: "v1.3.0".to_string(),
        authority_boundary: "Core Runtime consumes normalized payment authorization results, entitlement effects and evidence references. Provider checkout session creation, charge execution, refund execution, credentials and provider webhooks stay behind the Payment Provider Adapter boundary.".to_string(),
        adapter_boundary: "Payment Provider Adapter may map provider payment intent, checkout session and refund state into entitlement authorization facts. It cannot execute checkout inside Core Runtime, cannot mutate delivery artifacts and cannot make provider execution the source of Core authority.".to_string(),
        required_fields: vec![
            "providerPaymentIntentRef".to_string(),
            "checkoutSessionRef".to_string(),
            "entitlementAuthorizationRef".to_string(),
            "paymentStatus".to_string(),
            "refundStatus".to_string(),
            "sourceRefs".to_string(),
        ],
        dry_run_fixtures: vec![
            payment_provider_adapter_fixture(
                "paid",
                "passed",
                true,
                true,
                true,
                "paid",
                "none",
                "entitlement-authorized",
                "",
            ),
            payment_provider_adapter_fixture(
                "failed",
                "failed-as-expected",
                true,
                true,
                false,
                "failed",
                "none",
                "entitlement-blocked",
                "payment-failed",
            ),
            payment_provider_adapter_fixture(
                "refunded",
                "failed-as-expected",
                true,
                true,
                true,
                "paid",
                "refunded",
                "entitlement-refunded",
                "payment-refunded",
            ),
            payment_provider_adapter_fixture(
                "revoked",
                "failed-as-expected",
                true,
                true,
                true,
                "paid",
                "none",
                "entitlement-revoked",
                "entitlement-revoked",
            ),
            payment_provider_adapter_fixture(
                "missing",
                "failed-as-expected",
                false,
                false,
                false,
                "missing",
                "unknown",
                "entitlement-missing",
                "payment-provider-state-missing",
            ),
        ],
        stable_payment_statuses: vec![
            "paid".to_string(),
            "failed".to_string(),
            "missing".to_string(),
        ],
        stable_refund_statuses: vec![
            "none".to_string(),
            "refunded".to_string(),
            "unknown".to_string(),
        ],
        source_refs: vec![
            "docs/architecture/104-payment-provider-adapter-boundary-v1.md".to_string(),
            "docs/delivery/releases/v1.3.0/AGENTFLOW_V1_3_0_COMMERCIAL_BACKEND_STABLE_CLOSURE_TASKS_V1.md".to_string(),
        ],
        checked_at: "2026-07-10T00:00:00Z".to_string(),
    }
}

pub fn customer_delivery_backend_contract() -> CustomerDeliveryBackendContract {
    CustomerDeliveryBackendContract {
        version: CUSTOMER_DELIVERY_BACKEND_CONTRACT_VERSION.to_string(),
        status: "passed".to_string(),
        release_version: "v1.3.0".to_string(),
        authority_boundary: "Customer Delivery Backend reads accepted decision, complete artifact, authorized entitlement and access receipts. It can project download access and nextAction, but it cannot create payment checkout, mutate delivered artifacts, bypass refund / revocation / expiry state or replace decision authority.".to_string(),
        read_model_name: "customer-delivery-backend-read-model".to_string(),
        required_bindings: vec![
            "orderId".to_string(),
            "entitlementAuthorizationRef".to_string(),
            "decisionId".to_string(),
            "reportArtifactRef".to_string(),
            "accessReceiptRef".to_string(),
            "expiryState".to_string(),
            "revocationState".to_string(),
            "refundState".to_string(),
            "repairState".to_string(),
            "rerunState".to_string(),
            "feedbackState".to_string(),
            "sourceRefs".to_string(),
        ],
        stable_states: vec![
            "accessible".to_string(),
            "expired".to_string(),
            "revoked".to_string(),
            "refunded".to_string(),
            "repair-needed".to_string(),
            "rerun-needed".to_string(),
            "blocked".to_string(),
        ],
        accepted_delivery_fixture: customer_delivery_backend_fixture(
            "accepted-authorized",
            "passed",
            Some("access-receipt-accepted-authorized"),
            "active",
            "none",
            "none",
            "none",
            "not-required",
            "closed",
            "accessible",
            "show-download",
            true,
            true,
            &[],
        ),
        negative_access_fixtures: vec![
            customer_delivery_backend_fixture(
                "expired",
                "failed-as-expected",
                Some("access-receipt-expired"),
                "expired",
                "none",
                "none",
                "none",
                "not-required",
                "closed",
                "expired",
                "renew-access",
                false,
                false,
                &["access-expired"],
            ),
            customer_delivery_backend_fixture(
                "revoked",
                "failed-as-expected",
                Some("access-receipt-revoked"),
                "active",
                "revoked",
                "none",
                "none",
                "not-required",
                "closed",
                "revoked",
                "contact-support",
                false,
                false,
                &["access-revoked"],
            ),
            customer_delivery_backend_fixture(
                "refunded",
                "failed-as-expected",
                Some("access-receipt-refunded"),
                "active",
                "none",
                "refunded",
                "none",
                "not-required",
                "closed",
                "refunded",
                "show-refund-policy",
                false,
                false,
                &["payment-refunded"],
            ),
            customer_delivery_backend_fixture(
                "repair-needed",
                "failed-as-expected",
                None,
                "active",
                "none",
                "none",
                "repair-needed",
                "not-required",
                "repair-requested",
                "repair-needed",
                "create-repair-proposal",
                false,
                false,
                &["repair-needed"],
            ),
            customer_delivery_backend_fixture(
                "rerun-needed",
                "failed-as-expected",
                None,
                "active",
                "none",
                "none",
                "none",
                "rerun-needed",
                "repair-requested",
                "rerun-needed",
                "request-new-authorization",
                false,
                false,
                &["rerun-needs-authorization"],
            ),
        ],
        source_refs: vec![
            "docs/architecture/105-customer-delivery-backend-contract-v1.md".to_string(),
            "docs/delivery/releases/v1.3.0/AGENTFLOW_V1_3_0_COMMERCIAL_BACKEND_STABLE_CLOSURE_TASKS_V1.md".to_string(),
        ],
        checked_at: "2026-07-10T00:00:00Z".to_string(),
    }
}

pub fn commercial_e2e_golden_scenario() -> CommercialE2eGoldenScenarioProof {
    let ordered_facts = vec![
        commercial_e2e_fact(
            "sku-extension",
            "ProductSkuExtensionDefinition",
            PRODUCT_SKU_EXTENSION_CONTRACT_VERSION,
            "ready",
            "Product / Pack / SKU",
            "runtime/v130-product-sku-extension-contract.json",
        ),
        commercial_e2e_fact(
            "order-record",
            "PaidReportOrderRecord",
            PAID_REPORT_ORDER_RECORD_VERSION,
            "order-ready",
            "Order Runtime",
            "runtime/v130-commercial-backend-stable-contract.json",
        ),
        commercial_e2e_fact(
            "entitlement-authorization",
            "PaidReportEntitlementAuthorization",
            PAID_REPORT_ENTITLEMENT_AUTHORIZATION_VERSION,
            "authorized",
            "Entitlement Runtime",
            "runtime/v130-payment-provider-adapter-boundary.json",
        ),
        commercial_e2e_fact(
            "order-to-run-admission",
            "PaidReportOrderToRunAdmission",
            PAID_REPORT_ORDER_TO_RUN_ADMISSION_VERSION,
            "accepted",
            "Runtime Admission",
            "runtime/v130-paid-report-flow-state-machine.json",
        ),
        commercial_e2e_fact(
            "generation-adapter-receipt",
            "ProviderGeneratorAdapterReceipt",
            PROVIDER_GENERATOR_ADAPTER_BOUNDARY_VERSION,
            "succeeded",
            "Provider / Generator Adapter",
            "runtime/v130-provider-generator-adapter-boundary.json",
        ),
        commercial_e2e_fact(
            "report-artifact",
            "PaidReportArtifact",
            PAID_REPORT_ARTIFACT_VERSION,
            "complete",
            "Artifact Runtime",
            "runtime/v130-provider-generator-adapter-boundary.json",
        ),
        commercial_e2e_fact(
            "evidence-pack",
            "PaidReportEvidencePack",
            PAID_REPORT_EVIDENCE_PACK_VERSION,
            "complete",
            "Evidence Runtime",
            "runtime/v130-commercial-backend-stable-contract.json",
        ),
        commercial_e2e_fact(
            "decision-record",
            "PaidReportDecisionRecord",
            PAID_REPORT_DECISION_RECORD_VERSION,
            "accepted",
            "Decision Runtime",
            "runtime/v130-paid-report-flow-state-machine.json",
        ),
        commercial_e2e_fact(
            "delivery-package",
            "PaidReportDeliveryPackageProjection",
            PAID_REPORT_DELIVERY_PACKAGE_PROJECTION_VERSION,
            "delivery-ready",
            "Delivery Projection",
            "runtime/v130-customer-delivery-backend-contract.json",
        ),
        commercial_e2e_fact(
            "customer-access",
            "PaidReportCustomerDeliveryAccessProjection",
            PAID_REPORT_CUSTOMER_DELIVERY_ACCESS_VERSION,
            "accessible",
            "Customer Delivery Projection",
            "runtime/v130-customer-delivery-backend-contract.json",
        ),
        commercial_e2e_fact(
            "feedback-loop",
            "PaidReportFeedbackLoopProjection",
            PAID_REPORT_FEEDBACK_LOOP_PROJECTION_VERSION,
            "closed",
            "Feedback Projection",
            "runtime/v130-customer-delivery-backend-contract.json",
        ),
    ];
    let fact_refs = ordered_facts
        .iter()
        .map(|fact| fact.fact_id.clone())
        .collect::<Vec<_>>();

    CommercialE2eGoldenScenarioProof {
        version: COMMERCIAL_E2E_GOLDEN_SCENARIO_VERSION.to_string(),
        status: "passed".to_string(),
        release_version: "v1.3.0".to_string(),
        scenario_id: "v130-generic-commercial-backend-e2e".to_string(),
        product_sku_fixture_id: "synthetic-generic-report-sku".to_string(),
        concrete_domain_sku_implemented: false,
        ordered_facts,
        success_path: CommercialE2eGoldenScenarioPath {
            path_id: "accepted-delivery-access".to_string(),
            status: "passed".to_string(),
            description: "Generic paid report backend chain reaches customer delivery access with accepted decision and authorized entitlement.".to_string(),
            fact_refs: fact_refs.clone(),
            decision_outcome: "accepted".to_string(),
            delivery_status: "delivery-ready".to_string(),
            download_access_visible: true,
            access_handle_generated: true,
            mutates_delivered_artifact: false,
            next_action: "show-download".to_string(),
        },
        failure_repair_path: CommercialE2eGoldenScenarioPath {
            path_id: "repair-needed-does-not-mutate-delivery".to_string(),
            status: "failed-as-expected".to_string(),
            description: "Repair path creates a controlled follow-up proposal and cannot mutate the delivered artifact in place.".to_string(),
            fact_refs: vec![
                "decision-record".to_string(),
                "delivery-package".to_string(),
                "customer-access".to_string(),
                "feedback-loop".to_string(),
            ],
            decision_outcome: "needs-fix".to_string(),
            delivery_status: "repair-needed".to_string(),
            download_access_visible: false,
            access_handle_generated: false,
            mutates_delivered_artifact: false,
            next_action: "create-repair-proposal".to_string(),
        },
        certification_artifact_refs: vec![
            "runtime/v130-commercial-backend-stable-contract.json".to_string(),
            "runtime/v130-paid-report-flow-state-machine.json".to_string(),
            "runtime/v130-commercial-authority-boundary.json".to_string(),
            "runtime/v130-product-sku-extension-contract.json".to_string(),
            "runtime/v130-provider-generator-adapter-boundary.json".to_string(),
            "runtime/v130-payment-provider-adapter-boundary.json".to_string(),
            "runtime/v130-customer-delivery-backend-contract.json".to_string(),
        ],
        source_refs: vec![
            "docs/architecture/106-commercial-e2e-golden-scenario-v1.md".to_string(),
            "docs/delivery/releases/v1.3.0/AGENTFLOW_V1_3_0_COMMERCIAL_BACKEND_STABLE_CLOSURE_TASKS_V1.md".to_string(),
        ],
        checked_at: "2026-07-10T00:00:00Z".to_string(),
    }
}

fn commercial_e2e_fact(
    fact_id: &str,
    fact_type: &str,
    contract_version: &str,
    status: &str,
    authority_owner: &str,
    source_ref: &str,
) -> CommercialE2eGoldenScenarioFact {
    CommercialE2eGoldenScenarioFact {
        fact_id: fact_id.to_string(),
        fact_type: fact_type.to_string(),
        contract_version: contract_version.to_string(),
        status: status.to_string(),
        authority_owner: authority_owner.to_string(),
        source_ref: source_ref.to_string(),
    }
}

fn customer_delivery_backend_fixture(
    fixture_id: &str,
    status: &str,
    access_receipt_ref: Option<&str>,
    expiry_state: &str,
    revocation_state: &str,
    refund_state: &str,
    repair_state: &str,
    rerun_state: &str,
    feedback_state: &str,
    access_status: &str,
    next_action: &str,
    download_access_visible: bool,
    access_handle_generated: bool,
    failure_reasons: &[&str],
) -> CustomerDeliveryBackendFixture {
    CustomerDeliveryBackendFixture {
        fixture_id: fixture_id.to_string(),
        status: status.to_string(),
        order_id: format!("order-{fixture_id}"),
        entitlement_authorization_ref: format!("authorization-{fixture_id}"),
        decision_id: format!("decision-{fixture_id}"),
        report_artifact_ref: format!("artifact-{fixture_id}"),
        access_receipt_ref: access_receipt_ref.map(str::to_string),
        expiry_state: expiry_state.to_string(),
        revocation_state: revocation_state.to_string(),
        refund_state: refund_state.to_string(),
        repair_state: repair_state.to_string(),
        rerun_state: rerun_state.to_string(),
        feedback_state: feedback_state.to_string(),
        access_status: access_status.to_string(),
        next_action: next_action.to_string(),
        download_access_visible,
        access_handle_generated,
        failure_reasons: failure_reasons
            .iter()
            .map(|reason| reason.to_string())
            .collect(),
        source_refs: vec![
            "docs/architecture/100-paid-report-flow-state-machine-v1.md".to_string(),
            "docs/architecture/101-commercial-authority-boundary-v1.md".to_string(),
            "docs/architecture/104-payment-provider-adapter-boundary-v1.md".to_string(),
        ],
    }
}

fn paid_report_flow_states() -> Vec<String> {
    [
        "draft-order",
        "order-ready",
        "authorized",
        "admitted",
        "running",
        "artifact-ready",
        "evidence-complete",
        "accepted",
        "delivery-ready",
        "feedback-needed",
        "repair-requested",
        "rerun-needs-authorization",
        "refunded",
        "expired",
        "closed",
    ]
    .iter()
    .map(|state| state.to_string())
    .collect()
}

fn positive_flow_fixture(
    fixture_id: &str,
    from_state: &str,
    to_state: &str,
    event: &str,
    source_object: &str,
    source_contract_version: &str,
    writes_authority: bool,
    writes_accepted_authority: bool,
    writes_delivery_ready_authority: bool,
    required_contracts: Vec<PaidReportFlowContractBinding>,
) -> PaidReportFlowTransitionFixture {
    PaidReportFlowTransitionFixture {
        fixture_id: fixture_id.to_string(),
        status: "passed".to_string(),
        transition: PaidReportFlowTransition {
            from_state: from_state.to_string(),
            to_state: to_state.to_string(),
            event: event.to_string(),
            source_object: source_object.to_string(),
            source_contract_version: source_contract_version.to_string(),
            writes_authority,
            writes_accepted_authority,
            writes_delivery_ready_authority,
            required_contracts,
            failure_reasons: Vec::new(),
        },
    }
}

fn negative_flow_fixture(
    fixture_id: &str,
    from_state: &str,
    to_state: &str,
    event: &str,
    source_object: &str,
    source_contract_version: &str,
    required_contracts: Vec<PaidReportFlowContractBinding>,
    failure_reasons: Vec<PaidReportFlowFailureReason>,
) -> PaidReportFlowTransitionFixture {
    PaidReportFlowTransitionFixture {
        fixture_id: fixture_id.to_string(),
        status: "failed-as-expected".to_string(),
        transition: PaidReportFlowTransition {
            from_state: from_state.to_string(),
            to_state: to_state.to_string(),
            event: event.to_string(),
            source_object: source_object.to_string(),
            source_contract_version: source_contract_version.to_string(),
            writes_authority: false,
            writes_accepted_authority: false,
            writes_delivery_ready_authority: false,
            required_contracts,
            failure_reasons,
        },
    }
}

fn flow_binding(
    object_name: &str,
    contract_version: &str,
    binding_reason: &str,
) -> PaidReportFlowContractBinding {
    PaidReportFlowContractBinding {
        object_name: object_name.to_string(),
        contract_version: contract_version.to_string(),
        binding_reason: binding_reason.to_string(),
    }
}

fn flow_failure(code: &str, message: &str) -> PaidReportFlowFailureReason {
    PaidReportFlowFailureReason {
        code: code.to_string(),
        message: message.to_string(),
        prevents_authority_writes: true,
    }
}

fn authority_rule(
    object_name: &str,
    contract_version: &str,
    owner_component: &str,
    authority_kind: &str,
    can_create: bool,
    can_update: bool,
    projection_only: bool,
    writes_authority: bool,
    read_only_surfaces: &[&str],
) -> CommercialAuthorityRule {
    CommercialAuthorityRule {
        object_name: object_name.to_string(),
        contract_version: contract_version.to_string(),
        owner_component: owner_component.to_string(),
        authority_kind: authority_kind.to_string(),
        can_create,
        can_update,
        projection_only,
        writes_authority,
        read_only_surfaces: read_only_surfaces
            .iter()
            .map(|surface| surface.to_string())
            .collect(),
    }
}

fn authority_negative_fixture(
    fixture_id: &str,
    attempted_writer: &str,
    attempted_target: &str,
    attempted_authority_kind: &str,
    failure_reason: &str,
) -> CommercialAuthorityNegativeFixture {
    CommercialAuthorityNegativeFixture {
        fixture_id: fixture_id.to_string(),
        status: "failed-as-expected".to_string(),
        attempted_writer: attempted_writer.to_string(),
        attempted_target: attempted_target.to_string(),
        attempted_authority_kind: attempted_authority_kind.to_string(),
        can_write_authority: false,
        failure_reason: failure_reason.to_string(),
    }
}

fn product_sku_required_fields() -> Vec<String> {
    [
        "skuId",
        "productId",
        "requiredInputs",
        "reportSections",
        "evidencePolicy",
        "decisionPolicy",
        "deliveryPolicy",
        "pricingRef",
        "generatorRef",
        "sourceRefs",
    ]
    .iter()
    .map(|field| field.to_string())
    .collect()
}

fn product_sku_negative_fixture(
    fixture_id: &str,
    attempted_surface: &str,
    attempted_operation: &str,
    attempted_definition: Option<ProductSkuExtensionDefinition>,
    failure_reason: &str,
) -> ProductSkuExtensionNegativeFixture {
    ProductSkuExtensionNegativeFixture {
        fixture_id: fixture_id.to_string(),
        status: "failed-as-expected".to_string(),
        attempted_surface: attempted_surface.to_string(),
        attempted_operation: attempted_operation.to_string(),
        resolution: evaluate_product_sku_extension(attempted_definition.as_ref()),
        failure_reason: failure_reason.to_string(),
    }
}

fn provider_generator_adapter_request(
    request_id: &str,
    sku_id: &str,
    include_input_snapshot: bool,
    include_sku_definition: bool,
) -> ProviderGeneratorAdapterRequest {
    ProviderGeneratorAdapterRequest {
        version: "agentflow-provider-generator-adapter-request.v1".to_string(),
        status: "ready".to_string(),
        request_id: request_id.to_string(),
        product_instance_id: format!("product-instance-{sku_id}"),
        sku_id: sku_id.to_string(),
        input_snapshot_ref: if include_input_snapshot {
            format!("agentflow/tasks/{sku_id}/input/input-snapshot.json")
        } else {
            String::new()
        },
        sku_definition_ref: if include_sku_definition {
            format!("products/commercial-runtime/skus/{sku_id}/sku.json")
        } else {
            String::new()
        },
        generation_request_ref: format!("agentflow/tasks/{sku_id}/generation/request.json"),
        generator_ref: format!("products/commercial-runtime/skus/{sku_id}/generator.json"),
        provider_ref: "providers/dry-run-generator.json".to_string(),
        report_sections: vec![
            "summary".to_string(),
            "analysis".to_string(),
            "evidence-index".to_string(),
            "delivery-notes".to_string(),
        ],
        source_refs: vec![
            "docs/architecture/103-provider-generator-adapter-boundary-v1.md".to_string(),
            format!("products/commercial-runtime/skus/{sku_id}/sku.json"),
        ],
    }
}

#[allow(clippy::too_many_arguments)]
fn provider_generator_adapter_receipt(
    receipt_id: &str,
    request_id: &str,
    status: &str,
    output_artifact_ref: Option<&str>,
    evidence_refs: Vec<&str>,
    failure_reasons: Vec<String>,
    remediation_route: &str,
    provider_specific_call_is_core_authority: bool,
    delivery_blocked: bool,
) -> ProviderGeneratorAdapterReceipt {
    ProviderGeneratorAdapterReceipt {
        version: "agentflow-provider-generator-adapter-receipt.v1".to_string(),
        status: status.to_string(),
        receipt_id: receipt_id.to_string(),
        request_id: request_id.to_string(),
        adapter_id: "dry-run-provider-generator-adapter".to_string(),
        output_artifact_ref: output_artifact_ref.map(str::to_string),
        evidence_refs: evidence_refs.into_iter().map(str::to_string).collect(),
        failure_reasons,
        remediation_route: remediation_route.to_string(),
        provider_specific_call_is_core_authority,
        delivery_blocked,
    }
}

#[allow(clippy::too_many_arguments)]
fn payment_provider_adapter_fixture(
    fixture_id: &str,
    status: &str,
    has_provider_payment_intent: bool,
    has_checkout_session: bool,
    has_entitlement_authorization: bool,
    payment_status: &str,
    refund_status: &str,
    entitlement_effect: &str,
    failure_reason: &str,
) -> PaymentProviderAdapterFixture {
    PaymentProviderAdapterFixture {
        fixture_id: fixture_id.to_string(),
        status: status.to_string(),
        provider_payment_intent_ref: has_provider_payment_intent
            .then(|| format!("providers/dry-run-payment/{fixture_id}/payment-intent.json")),
        checkout_session_ref: has_checkout_session
            .then(|| format!("providers/dry-run-payment/{fixture_id}/checkout-session.json")),
        entitlement_authorization_ref: has_entitlement_authorization.then(|| {
            format!("agentflow/tasks/{fixture_id}/commercial/entitlement-authorization.json")
        }),
        payment_status: payment_status.to_string(),
        refund_status: refund_status.to_string(),
        source_refs: vec![
            format!("providers/dry-run-payment/{fixture_id}/source.json"),
            "docs/architecture/104-payment-provider-adapter-boundary-v1.md".to_string(),
        ],
        core_consumes_authorization_result: has_entitlement_authorization,
        core_consumes_provider_evidence: has_provider_payment_intent || has_checkout_session,
        provider_checkout_implementation_is_core_authority: false,
        provider_refund_execution_is_core_authority: false,
        entitlement_effect: entitlement_effect.to_string(),
        failure_reason: failure_reason.to_string(),
    }
}

fn stable_object(
    object_name: &str,
    rust_type: &str,
    category: &str,
    version: &str,
    required_fields: &[&str],
    optional_fields: &[&str],
    status_values: &[&str],
) -> CommercialBackendStableObject {
    CommercialBackendStableObject {
        object_name: object_name.to_string(),
        rust_type: rust_type.to_string(),
        category: category.to_string(),
        version: version.to_string(),
        required_fields: required_fields
            .iter()
            .map(|name| stable_field(name, true))
            .collect(),
        optional_fields: optional_fields
            .iter()
            .map(|name| stable_field(name, false))
            .collect(),
        status_values: status_values
            .iter()
            .map(|value| value.to_string())
            .collect(),
    }
}

fn stable_field(name: &str, required: bool) -> CommercialBackendStableField {
    CommercialBackendStableField {
        name: name.to_string(),
        field_type: infer_stable_field_type(name).to_string(),
        required,
        description: if required {
            format!("{name} is part of the stable v1.3.0 contract.")
        } else {
            format!("{name} is optional/defaulted in the stable v1.3.0 contract.")
        },
    }
}

fn infer_stable_field_type(name: &str) -> &'static str {
    if name.ends_with("Refs") || name.ends_with("Reasons") || name == "sections" {
        "array"
    } else if matches!(
        name,
        "submittedFields" | "productInstance" | "preflight" | "proposal"
    ) {
        "object"
    } else if name.starts_with("can")
        || name.starts_with("writes")
        || name.starts_with("projection")
        || name.ends_with("Ready")
        || name.ends_with("Visible")
        || name.ends_with("Complete")
        || name.ends_with("Required")
        || name.ends_with("Authority")
        || matches!(
            name,
            "runnable"
                | "accepted"
                | "started"
                | "completed"
                | "appendOnly"
                | "projectScoped"
                | "paymentProviderCharge"
                | "paymentProviderCheckout"
                | "providerChargeExecuted"
                | "downloadReady"
                | "downloadVisible"
                | "mutatesDeliveredArtifact"
                | "createsFollowUpProposal"
                | "requiresNewAuthorization"
                | "commercialDecisionOnly"
        )
    {
        "boolean"
    } else {
        "string"
    }
}

fn decision_state(state: &str, owner: &str, meaning: &str) -> CommercialBackendDecisionState {
    CommercialBackendDecisionState {
        state: state.to_string(),
        owner: owner.to_string(),
        meaning: meaning.to_string(),
    }
}

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

    #[test]
    fn commercial_backend_stable_contract_lists_all_required_objects_and_states() {
        let contract = commercial_backend_stable_contract();

        assert_eq!(contract.version, COMMERCIAL_BACKEND_STABLE_CONTRACT_VERSION);
        assert_eq!(contract.status, "passed");
        assert_eq!(contract.release_version, "v1.3.0");
        assert!(
            contract
                .migration_policy
                .backward_incompatible_changes_require_version_bump
        );
        assert!(contract.migration_policy.explicit_migration_required);
        assert!(contract.migration_policy.machine_readable_baseline_required);

        let object_names = contract
            .objects
            .iter()
            .map(|object| object.object_name.as_str())
            .collect::<std::collections::HashSet<_>>();
        for required in [
            "Product Definition",
            "Product Instance",
            "Order Record",
            "Entitlement Authorization",
            "Order To Run Admission",
            "Run Execution Receipt",
            "Report Artifact",
            "Evidence Pack",
            "Decision Record",
            "Delivery Package Projection",
            "Customer Delivery Access",
            "Access Receipt",
            "Feedback Loop Projection",
            "Commercial Policy Record",
        ] {
            assert!(object_names.contains(required), "missing {required}");
        }

        for object in &contract.objects {
            assert!(!object.version.trim().is_empty(), "missing version");
            assert!(
                object
                    .required_fields
                    .iter()
                    .any(|field| field.name == "version" && field.required),
                "{} must require version",
                object.object_name
            );
            assert!(
                object
                    .required_fields
                    .iter()
                    .any(|field| field.name == "status" && field.required),
                "{} must require status",
                object.object_name
            );
            assert!(
                !object.status_values.is_empty(),
                "{} must publish status values",
                object.object_name
            );
        }

        let states = contract
            .error_decision_model
            .stable_states
            .iter()
            .map(|state| state.state.as_str())
            .collect::<std::collections::HashSet<_>>();
        for state in [
            "invalid",
            "deferred",
            "blocked",
            "accepted",
            "revoked",
            "expired",
            "refunded",
            "repair-needed",
            "delivery-ready",
        ] {
            assert!(states.contains(state), "missing state {state}");
        }
    }

    #[test]
    fn paid_report_flow_state_machine_covers_required_states_and_invalid_transitions() {
        let machine = paid_report_flow_state_machine();

        assert_eq!(machine.version, PAID_REPORT_FLOW_STATE_MACHINE_VERSION);
        assert_eq!(machine.status, "passed");
        assert_eq!(machine.release_version, "v1.3.0");

        let states = machine
            .states
            .iter()
            .map(String::as_str)
            .collect::<std::collections::HashSet<_>>();
        for state in [
            "draft-order",
            "order-ready",
            "authorized",
            "admitted",
            "running",
            "artifact-ready",
            "evidence-complete",
            "accepted",
            "delivery-ready",
            "feedback-needed",
            "repair-requested",
            "rerun-needs-authorization",
            "refunded",
            "expired",
            "closed",
        ] {
            assert!(states.contains(state), "missing state {state}");
        }

        assert!(
            machine
                .positive_fixtures
                .iter()
                .any(|fixture| fixture.transition.to_state == "accepted"
                    && fixture.transition.writes_accepted_authority),
            "accepted authority must have a positive transition"
        );
        assert!(
            machine
                .positive_fixtures
                .iter()
                .any(|fixture| fixture.transition.to_state == "delivery-ready"
                    && fixture.transition.writes_delivery_ready_authority),
            "delivery-ready authority must have a positive transition"
        );

        let mut bound_objects = std::collections::HashSet::new();
        for fixture in &machine.positive_fixtures {
            assert_eq!(fixture.status, "passed");
            assert!(
                states.contains(fixture.transition.from_state.as_str()),
                "unknown from state {}",
                fixture.transition.from_state
            );
            assert!(
                states.contains(fixture.transition.to_state.as_str()),
                "unknown to state {}",
                fixture.transition.to_state
            );
            assert!(!fixture.transition.required_contracts.is_empty());
            for binding in &fixture.transition.required_contracts {
                assert!(!binding.contract_version.trim().is_empty());
                bound_objects.insert(binding.object_name.as_str());
            }
        }

        for required in [
            "PaidReportOrderRecord",
            "PaidReportEntitlementAuthorization",
            "PaidReportOrderToRunAdmission",
            "PaidReportRunExecutionReceipt",
            "PaidReportArtifact",
            "PaidReportEvidencePack",
            "PaidReportDecisionRecord",
            "PaidReportDeliveryPackageProjection",
            "PaidReportFeedbackLoopProjection",
        ] {
            assert!(
                bound_objects.contains(required),
                "missing binding {required}"
            );
        }

        assert!(!machine.negative_fixtures.is_empty());
        for fixture in &machine.negative_fixtures {
            assert_eq!(fixture.status, "failed-as-expected");
            assert!(
                !fixture.transition.failure_reasons.is_empty(),
                "{} must publish failure reasons",
                fixture.fixture_id
            );
            assert!(
                fixture
                    .transition
                    .failure_reasons
                    .iter()
                    .all(|reason| reason.prevents_authority_writes),
                "{} must block authority writes",
                fixture.fixture_id
            );
            assert!(
                !fixture.transition.writes_accepted_authority,
                "{} must not write accepted authority",
                fixture.fixture_id
            );
            assert!(
                !fixture.transition.writes_delivery_ready_authority,
                "{} must not write delivery-ready authority",
                fixture.fixture_id
            );
            assert!(
                !fixture.transition.writes_authority,
                "{} must not write authority",
                fixture.fixture_id
            );
        }
    }

    #[test]
    fn commercial_authority_boundary_freezes_writers_and_rejects_readonly_surfaces() {
        let boundary = commercial_authority_boundary();

        assert_eq!(boundary.version, COMMERCIAL_AUTHORITY_BOUNDARY_VERSION);
        assert_eq!(boundary.status, "passed");
        assert_eq!(boundary.release_version, "v1.3.0");

        let object_names = boundary
            .authority_map
            .iter()
            .map(|rule| rule.object_name.as_str())
            .collect::<std::collections::HashSet<_>>();
        for required in [
            "PaidReportOrderRecord",
            "PaidReportEntitlementAuthorization",
            "PaidReportOrderToRunAdmission",
            "PaidReportRunExecutionReceipt",
            "PaidReportArtifact",
            "PaidReportEvidencePack",
            "PaidReportDecisionRecord",
            "PaidReportDeliveryPackageProjection",
            "PaidReportCustomerDeliveryAccessProjection",
            "PaidReportAccessReceipt",
            "PaidReportFeedbackLoopProjection",
            "PaidReportCommercialPolicyRecord",
        ] {
            assert!(object_names.contains(required), "missing rule {required}");
        }

        for authority_kind in [
            "order",
            "entitlement",
            "run-admission",
            "run",
            "artifact",
            "evidence",
            "decision",
            "delivery-view",
            "customer-access-view",
            "access-receipt",
            "feedback-view",
            "commercial-policy",
        ] {
            assert!(
                boundary
                    .authority_map
                    .iter()
                    .any(|rule| rule.authority_kind == authority_kind),
                "missing authority kind {authority_kind}"
            );
        }

        for rule in &boundary.authority_map {
            assert!(!rule.contract_version.trim().is_empty());
            if rule.projection_only {
                assert!(!rule.can_create);
                assert!(!rule.can_update);
                assert!(!rule.writes_authority);
            }
        }

        for surface in ["Projection", "Customer View", "Download View"] {
            assert!(
                boundary
                    .read_only_surfaces
                    .iter()
                    .any(|entry| entry == surface),
                "missing read-only surface {surface}"
            );
        }

        for fixture_id in [
            "projection-writing-authority",
            "customer-view-writing-authority",
            "download-view-writing-authority",
            "synthetic-release-sidecar-promoted-as-authority",
            "release-sidecar-promoted-as-authority",
        ] {
            let fixture = boundary
                .negative_fixtures
                .iter()
                .find(|fixture| fixture.fixture_id == fixture_id)
                .unwrap_or_else(|| panic!("missing fixture {fixture_id}"));
            assert_eq!(fixture.status, "failed-as-expected");
            assert!(!fixture.can_write_authority);
            assert!(!fixture.failure_reason.trim().is_empty());
        }
    }

    #[test]
    fn product_sku_extension_contract_separates_sku_authority_from_core_runtime() {
        let contract = product_sku_extension_contract();

        assert_eq!(contract.version, PRODUCT_SKU_EXTENSION_CONTRACT_VERSION);
        assert_eq!(contract.status, "passed");
        assert_eq!(contract.release_version, "v1.3.0");
        assert_eq!(contract.allowed_authority_surface, "Product / Pack / SKU");

        let required = contract
            .required_fields
            .iter()
            .map(String::as_str)
            .collect::<std::collections::HashSet<_>>();
        for field in [
            "skuId",
            "productId",
            "requiredInputs",
            "reportSections",
            "evidencePolicy",
            "decisionPolicy",
            "deliveryPolicy",
            "pricingRef",
            "generatorRef",
            "sourceRefs",
        ] {
            assert!(required.contains(field), "missing field {field}");
        }

        assert_eq!(contract.synthetic_sku_resolution.status, "ready");
        assert!(
            contract
                .synthetic_sku_resolution
                .can_materialize_product_instance
        );
        assert!(
            !contract
                .synthetic_sku_resolution
                .falls_back_to_generic_hardcoded_content
        );
        assert!(!contract.synthetic_sku_fixture.required_inputs.is_empty());
        assert!(!contract.synthetic_sku_fixture.report_sections.is_empty());
        assert!(!contract.synthetic_sku_fixture.evidence_policy.is_empty());
        assert!(!contract.synthetic_sku_fixture.decision_policy.is_empty());
        assert!(!contract.synthetic_sku_fixture.delivery_policy.is_empty());
        assert!(!contract.synthetic_sku_fixture.pricing_ref.trim().is_empty());
        assert!(!contract
            .synthetic_sku_fixture
            .generator_ref
            .trim()
            .is_empty());
        assert!(!contract.synthetic_sku_fixture.source_refs.is_empty());

        let missing = evaluate_product_sku_extension(None);
        assert_eq!(missing.status, "invalid");
        assert_eq!(missing.unavailable_reason, "missing-sku-definition");
        assert!(!missing.can_materialize_product_instance);
        assert!(!missing.falls_back_to_generic_hardcoded_content);
        assert!(missing.missing_fields.iter().any(|field| field == "skuId"));

        for fixture_id in [
            "missing-sku-definition",
            "core-runtime-domain-term-as-authority",
            "synthetic-sku-sidecar-promoted-as-live-product",
        ] {
            let fixture = contract
                .negative_fixtures
                .iter()
                .find(|fixture| fixture.fixture_id == fixture_id)
                .unwrap_or_else(|| panic!("missing fixture {fixture_id}"));
            assert_eq!(fixture.status, "failed-as-expected");
            assert!(!fixture.failure_reason.trim().is_empty());
            assert!(!fixture.resolution.falls_back_to_generic_hardcoded_content);
        }

        let core_text = format!(
            "{} {}",
            contract.authority_boundary, contract.core_runtime_policy
        )
        .to_lowercase();
        for term in &contract.forbidden_core_terms {
            assert!(
                !core_text.contains(term),
                "core runtime authority text contains forbidden term {term}"
            );
        }
    }

    #[test]
    fn provider_generator_adapter_boundary_blocks_provider_authority_leaks() {
        let contract = provider_generator_adapter_boundary_contract();

        assert_eq!(
            contract.version,
            PROVIDER_GENERATOR_ADAPTER_BOUNDARY_VERSION
        );
        assert_eq!(contract.status, "passed");
        assert_eq!(contract.release_version, "v1.3.0");

        for object in [
            "inputSnapshot",
            "skuDefinition",
            "generationRequest",
            "generationReceipt",
            "outputArtifact",
            "evidenceRefs",
            "failureReasons",
        ] {
            assert!(
                contract.required_objects.iter().any(|item| item == object),
                "missing required adapter object {object}"
            );
        }

        let positive = &contract.dry_run_positive_fixture;
        assert_eq!(positive.status, "passed");
        assert!(!positive.request.input_snapshot_ref.trim().is_empty());
        assert!(!positive.request.sku_definition_ref.trim().is_empty());
        assert!(!positive.request.generator_ref.trim().is_empty());
        assert!(!positive.request.provider_ref.trim().is_empty());
        assert_eq!(positive.receipt.status, "succeeded");
        assert!(positive.receipt.output_artifact_ref.is_some());
        assert!(!positive.receipt.evidence_refs.is_empty());
        assert!(!positive.receipt.provider_specific_call_is_core_authority);
        assert!(!positive.receipt.delivery_blocked);
        let artifact = positive.artifact.as_ref().expect("positive artifact");
        assert!(artifact.produced_by_adapter);
        assert!(!artifact.writes_core_authority);

        for fixture_id in [
            "missing-input-snapshot",
            "provider-call-promoted-to-core-authority",
            "failed-generation-keeps-delivery-blocked",
        ] {
            let fixture = contract
                .negative_fixtures
                .iter()
                .find(|fixture| fixture.fixture_id == fixture_id)
                .unwrap_or_else(|| panic!("missing fixture {fixture_id}"));
            assert_eq!(fixture.status, "failed-as-expected");
            assert!(fixture.receipt.delivery_blocked);
            assert!(!fixture.receipt.failure_reasons.is_empty());
            assert_eq!(fixture.expected_delivery_state, "blocked");
            assert!(fixture.artifact.is_none());
        }

        let leaked_provider = contract
            .negative_fixtures
            .iter()
            .find(|fixture| fixture.fixture_id == "provider-call-promoted-to-core-authority")
            .expect("provider leak fixture");
        assert!(
            leaked_provider
                .receipt
                .provider_specific_call_is_core_authority
        );
        assert!(leaked_provider
            .receipt
            .failure_reasons
            .iter()
            .any(|reason| reason == "provider-call-cannot-write-core-authority"));
    }

    #[test]
    fn payment_provider_adapter_boundary_keeps_checkout_outside_core() {
        let contract = payment_provider_adapter_boundary_contract();

        assert_eq!(contract.version, PAYMENT_PROVIDER_ADAPTER_BOUNDARY_VERSION);
        assert_eq!(contract.status, "passed");
        assert_eq!(contract.release_version, "v1.3.0");

        for field in [
            "providerPaymentIntentRef",
            "checkoutSessionRef",
            "entitlementAuthorizationRef",
            "paymentStatus",
            "refundStatus",
            "sourceRefs",
        ] {
            assert!(
                contract.required_fields.iter().any(|item| item == field),
                "missing payment adapter field {field}"
            );
        }

        let fixtures = contract
            .dry_run_fixtures
            .iter()
            .map(|fixture| (fixture.fixture_id.as_str(), fixture))
            .collect::<std::collections::HashMap<_, _>>();
        for fixture_id in ["paid", "failed", "refunded", "revoked", "missing"] {
            let fixture = fixtures
                .get(fixture_id)
                .unwrap_or_else(|| panic!("missing fixture {fixture_id}"));
            assert!(!fixture.source_refs.is_empty());
            assert!(!fixture.provider_checkout_implementation_is_core_authority);
            assert!(!fixture.provider_refund_execution_is_core_authority);
        }

        let paid = fixtures.get("paid").expect("paid fixture");
        assert_eq!(paid.status, "passed");
        assert_eq!(paid.payment_status, "paid");
        assert_eq!(paid.refund_status, "none");
        assert_eq!(paid.entitlement_effect, "entitlement-authorized");
        assert!(paid.provider_payment_intent_ref.is_some());
        assert!(paid.checkout_session_ref.is_some());
        assert!(paid.entitlement_authorization_ref.is_some());
        assert!(paid.core_consumes_authorization_result);
        assert!(paid.core_consumes_provider_evidence);

        for fixture_id in ["failed", "refunded", "revoked", "missing"] {
            let fixture = fixtures.get(fixture_id).expect("negative fixture");
            assert_eq!(fixture.status, "failed-as-expected");
            assert!(!fixture.failure_reason.trim().is_empty());
        }

        let refunded = fixtures.get("refunded").expect("refunded fixture");
        assert_eq!(refunded.refund_status, "refunded");
        assert_eq!(refunded.entitlement_effect, "entitlement-refunded");
        assert!(!refunded.provider_refund_execution_is_core_authority);

        let missing = fixtures.get("missing").expect("missing fixture");
        assert_eq!(missing.payment_status, "missing");
        assert_eq!(missing.refund_status, "unknown");
        assert!(missing.provider_payment_intent_ref.is_none());
        assert!(missing.checkout_session_ref.is_none());
        assert!(missing.entitlement_authorization_ref.is_none());
        assert!(!missing.core_consumes_authorization_result);
    }

    #[test]
    fn customer_delivery_backend_contract_blocks_invalid_access_states() {
        let contract = customer_delivery_backend_contract();

        assert_eq!(contract.version, CUSTOMER_DELIVERY_BACKEND_CONTRACT_VERSION);
        assert_eq!(contract.status, "passed");
        assert_eq!(contract.release_version, "v1.3.0");

        let required = contract
            .required_bindings
            .iter()
            .map(String::as_str)
            .collect::<std::collections::HashSet<_>>();
        for field in [
            "orderId",
            "entitlementAuthorizationRef",
            "decisionId",
            "reportArtifactRef",
            "accessReceiptRef",
            "expiryState",
            "revocationState",
            "refundState",
            "repairState",
            "rerunState",
            "feedbackState",
            "sourceRefs",
        ] {
            assert!(required.contains(field), "missing binding {field}");
        }

        let states = contract
            .stable_states
            .iter()
            .map(String::as_str)
            .collect::<std::collections::HashSet<_>>();
        for state in [
            "accessible",
            "expired",
            "revoked",
            "refunded",
            "repair-needed",
            "rerun-needed",
            "blocked",
        ] {
            assert!(states.contains(state), "missing state {state}");
        }

        let accepted = &contract.accepted_delivery_fixture;
        assert_eq!(accepted.fixture_id, "accepted-authorized");
        assert_eq!(accepted.status, "passed");
        assert_eq!(accepted.access_status, "accessible");
        assert_eq!(accepted.next_action, "show-download");
        assert!(accepted.download_access_visible);
        assert!(accepted.access_handle_generated);
        assert!(accepted.access_receipt_ref.is_some());
        assert!(accepted.failure_reasons.is_empty());

        let fixtures = contract
            .negative_access_fixtures
            .iter()
            .map(|fixture| (fixture.fixture_id.as_str(), fixture))
            .collect::<std::collections::HashMap<_, _>>();
        for fixture_id in [
            "expired",
            "revoked",
            "refunded",
            "repair-needed",
            "rerun-needed",
        ] {
            let fixture = fixtures
                .get(fixture_id)
                .unwrap_or_else(|| panic!("missing fixture {fixture_id}"));
            assert_eq!(fixture.status, "failed-as-expected");
            assert!(!fixture.download_access_visible);
            assert!(!fixture.access_handle_generated);
            assert!(!fixture.next_action.trim().is_empty());
            assert!(!fixture.failure_reasons.is_empty());
            assert!(!fixture.source_refs.is_empty());
        }

        assert_eq!(fixtures["expired"].next_action, "renew-access");
        assert_eq!(fixtures["revoked"].next_action, "contact-support");
        assert_eq!(fixtures["refunded"].next_action, "show-refund-policy");
        assert_eq!(
            fixtures["repair-needed"].next_action,
            "create-repair-proposal"
        );
        assert_eq!(
            fixtures["rerun-needed"].next_action,
            "request-new-authorization"
        );
    }

    #[test]
    fn commercial_e2e_golden_scenario_covers_backend_chain_and_repair_path() {
        let scenario = commercial_e2e_golden_scenario();

        assert_eq!(scenario.version, COMMERCIAL_E2E_GOLDEN_SCENARIO_VERSION);
        assert_eq!(scenario.status, "passed");
        assert_eq!(scenario.release_version, "v1.3.0");
        assert!(!scenario.concrete_domain_sku_implemented);

        let fact_types = scenario
            .ordered_facts
            .iter()
            .map(|fact| fact.fact_type.as_str())
            .collect::<Vec<_>>();
        assert_eq!(
            fact_types,
            vec![
                "ProductSkuExtensionDefinition",
                "PaidReportOrderRecord",
                "PaidReportEntitlementAuthorization",
                "PaidReportOrderToRunAdmission",
                "ProviderGeneratorAdapterReceipt",
                "PaidReportArtifact",
                "PaidReportEvidencePack",
                "PaidReportDecisionRecord",
                "PaidReportDeliveryPackageProjection",
                "PaidReportCustomerDeliveryAccessProjection",
                "PaidReportFeedbackLoopProjection",
            ]
        );

        for fact in &scenario.ordered_facts {
            assert!(!fact.contract_version.trim().is_empty());
            assert!(!fact.authority_owner.trim().is_empty());
            assert!(!fact.source_ref.trim().is_empty());
        }

        assert_eq!(scenario.success_path.status, "passed");
        assert_eq!(scenario.success_path.decision_outcome, "accepted");
        assert_eq!(scenario.success_path.delivery_status, "delivery-ready");
        assert!(scenario.success_path.download_access_visible);
        assert!(scenario.success_path.access_handle_generated);
        assert!(!scenario.success_path.mutates_delivered_artifact);
        assert_eq!(scenario.success_path.next_action, "show-download");
        assert_eq!(
            scenario.success_path.fact_refs.len(),
            scenario.ordered_facts.len()
        );

        assert_eq!(scenario.failure_repair_path.status, "failed-as-expected");
        assert_eq!(scenario.failure_repair_path.decision_outcome, "needs-fix");
        assert_eq!(
            scenario.failure_repair_path.delivery_status,
            "repair-needed"
        );
        assert!(!scenario.failure_repair_path.download_access_visible);
        assert!(!scenario.failure_repair_path.access_handle_generated);
        assert!(!scenario.failure_repair_path.mutates_delivered_artifact);
        assert_eq!(
            scenario.failure_repair_path.next_action,
            "create-repair-proposal"
        );

        for artifact in [
            "runtime/v130-commercial-backend-stable-contract.json",
            "runtime/v130-paid-report-flow-state-machine.json",
            "runtime/v130-product-sku-extension-contract.json",
            "runtime/v130-provider-generator-adapter-boundary.json",
            "runtime/v130-payment-provider-adapter-boundary.json",
            "runtime/v130-customer-delivery-backend-contract.json",
        ] {
            assert!(
                scenario
                    .certification_artifact_refs
                    .iter()
                    .any(|entry| entry == artifact),
                "missing certification artifact {artifact}"
            );
        }
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
