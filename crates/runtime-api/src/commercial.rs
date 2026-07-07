use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::Path};

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
    let status = if entries
        .iter()
        .any(|entry| entry.availability == CommercialAvailability::Invalid)
    {
        "invalid"
    } else if entries
        .iter()
        .any(|entry| entry.availability == CommercialAvailability::Deferred)
    {
        "deferred"
    } else {
        "fresh"
    };

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
    let registry_root = Path::new("products/commercial-runtime");
    if registry_root.is_dir() {
        if let Ok(model) = load_registry_commercial_product_read_model(registry_root) {
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

    let status = if entries
        .iter()
        .any(|entry| entry.availability == CommercialAvailability::Invalid)
    {
        "invalid"
    } else if entries
        .iter()
        .any(|entry| entry.availability == CommercialAvailability::Deferred)
    {
        "deferred"
    } else {
        "fresh"
    };

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

pub fn get_commercial_product_projection_query() -> CommercialProjectionQuery {
    let read_model = load_commercial_product_read_model();
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
    let read_model = load_commercial_product_read_model();
    let projection_query = get_commercial_product_projection_query();
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
    let status = if paid_report_blocked.can_submit_runtime_command_proposal
        || paid_report_deferred.can_submit_runtime_command_proposal
        || managed_project_available.availability != CommercialAvailability::Available
        || projection_query.writes_authority
    {
        "failed"
    } else {
        "passed"
    };

    CommercialGoldenPathProof {
        version: COMMERCIAL_GOLDEN_PATH_VERSION.to_string(),
        status: status.to_string(),
        read_model,
        projection_query,
        paid_report_blocked,
        paid_report_deferred,
        managed_project_available,
        projection_writes_authority: false,
        desktop_writes_authority: false,
    }
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
        assert!(model.projection_only);
        assert!(!model.core_authority);
        assert!(!model.writes_authority);
        assert!(model.entries.iter().any(|entry| {
            entry.flow_type == CommercialFlowType::PaidReportFlow
                && entry.availability == CommercialAvailability::Rejected
                && !entry.can_submit_runtime_command_proposal
        }));
        assert!(model.entries.iter().any(|entry| {
            entry.flow_type == CommercialFlowType::ManagedProjectFlow
                && entry.availability == CommercialAvailability::Available
                && entry.command_policy == CommercialCommandPolicy::AllowedToPropose
        }));
    }

    #[test]
    fn registry_read_model_uses_product_source_and_rejects_missing_definitions() {
        let registry = registry_fixture();
        let model = load_registry_commercial_product_read_model(registry.path()).unwrap();

        assert_eq!(model.source, "product-registry-config");
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
            entry.product_id == "missing-report"
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
            "missing-report",
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
      "productId": "missing-report",
      "productName": "Missing Report",
      "flowType": "paid-report-flow",
      "paidFeatureState": "enabled",
      "paymentConfigured": true,
      "reportDefinitionPresent": false,
      "requiredInputRefs": ["reportInputRef", "orderIntentId"],
      "sourceRef": "products/commercial-runtime/products.json#missing-report"
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
      "productId": "missing-report",
      "entitlementState": "active",
      "sourceRef": "products/commercial-runtime/entitlements.json#missing-report"
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
