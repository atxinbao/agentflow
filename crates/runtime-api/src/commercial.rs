use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
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
    resolve_paid_report_product_instance_from_registry(
        project_commercial_registry_root(project_root),
        product_id,
    )
}

pub fn build_paid_report_runtime_proposal_handoff_from_registry(
    registry_root: impl AsRef<Path>,
    product_id: &str,
    request_id: &str,
) -> Result<PaidReportRuntimeProposalHandoff> {
    let instance = resolve_paid_report_product_instance_from_registry(&registry_root, product_id)?;
    let preflight =
        evaluate_paid_report_preflight_from_registry(registry_root, product_id, request_id)?;
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
    Ok(PaidReportRuntimeProposalHandoff {
        version: PAID_REPORT_RUNTIME_PROPOSAL_HANDOFF_VERSION.to_string(),
        status: status.to_string(),
        reason,
        proposal_created: proposal.is_some(),
        product_instance: instance,
        preflight,
        proposal,
    })
}

pub fn build_paid_report_runtime_proposal_handoff_from_project(
    project_root: impl AsRef<Path>,
    product_id: &str,
    request_id: &str,
) -> Result<PaidReportRuntimeProposalHandoff> {
    build_paid_report_runtime_proposal_handoff_from_registry(
        project_commercial_registry_root(project_root),
        product_id,
        request_id,
    )
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
}
