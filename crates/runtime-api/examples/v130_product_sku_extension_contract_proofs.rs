use agentflow_runtime_api::{
    evaluate_product_sku_extension, product_sku_extension_contract,
    PRODUCT_SKU_EXTENSION_CONTRACT_VERSION,
};
use anyhow::{bail, Context, Result};
use serde::Serialize;
use std::{collections::HashSet, fs, path::Path};

fn main() -> Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.len() != 1 {
        bail!("usage: v130_product_sku_extension_contract_proofs <sku-contract>");
    }

    let contract = product_sku_extension_contract();
    let mut failures = Vec::new();

    if contract.version != PRODUCT_SKU_EXTENSION_CONTRACT_VERSION {
        failures.push("wrong-product-sku-extension-version".to_string());
    }
    if contract.status != "passed" {
        failures.push("product-sku-extension-status-not-passed".to_string());
    }
    if contract.release_version != "v1.3.0" {
        failures.push("wrong-release-version".to_string());
    }
    if contract.allowed_authority_surface != "Product / Pack / SKU" {
        failures.push("wrong-authority-surface".to_string());
    }

    let required = contract
        .required_fields
        .iter()
        .map(String::as_str)
        .collect::<HashSet<_>>();
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
        if !required.contains(field) {
            failures.push(format!("missing-required-field-{field}"));
        }
    }

    let fixture = &contract.synthetic_sku_fixture;
    if fixture.sku_id.trim().is_empty() {
        failures.push("synthetic-sku-missing-sku-id".to_string());
    }
    if fixture.product_id.trim().is_empty() {
        failures.push("synthetic-sku-missing-product-id".to_string());
    }
    if fixture.required_inputs.is_empty() {
        failures.push("synthetic-sku-missing-required-inputs".to_string());
    }
    if fixture.report_sections.is_empty() {
        failures.push("synthetic-sku-missing-report-sections".to_string());
    }
    if fixture.evidence_policy.is_empty() {
        failures.push("synthetic-sku-missing-evidence-policy".to_string());
    }
    if fixture.decision_policy.is_empty() {
        failures.push("synthetic-sku-missing-decision-policy".to_string());
    }
    if fixture.delivery_policy.is_empty() {
        failures.push("synthetic-sku-missing-delivery-policy".to_string());
    }
    if fixture.pricing_ref.trim().is_empty() {
        failures.push("synthetic-sku-missing-pricing-ref".to_string());
    }
    if fixture.generator_ref.trim().is_empty() {
        failures.push("synthetic-sku-missing-generator-ref".to_string());
    }
    if fixture.source_refs.is_empty() {
        failures.push("synthetic-sku-missing-source-refs".to_string());
    }

    if contract.synthetic_sku_resolution.status != "ready" {
        failures.push("synthetic-sku-resolution-not-ready".to_string());
    }
    if !contract
        .synthetic_sku_resolution
        .can_materialize_product_instance
    {
        failures.push("synthetic-sku-cannot-materialize-product-instance".to_string());
    }
    if contract
        .synthetic_sku_resolution
        .falls_back_to_generic_hardcoded_content
    {
        failures.push("synthetic-sku-falls-back-to-hardcoded-content".to_string());
    }

    let missing = evaluate_product_sku_extension(None);
    if missing.status != "invalid" {
        failures.push("missing-sku-not-invalid".to_string());
    }
    if missing.unavailable_reason != "missing-sku-definition" {
        failures.push("missing-sku-wrong-reason".to_string());
    }
    if missing.can_materialize_product_instance {
        failures.push("missing-sku-can-materialize".to_string());
    }
    if missing.falls_back_to_generic_hardcoded_content {
        failures.push("missing-sku-falls-back-to-hardcoded-content".to_string());
    }

    for fixture_id in [
        "missing-sku-definition",
        "core-runtime-domain-term-as-authority",
        "synthetic-sku-sidecar-promoted-as-live-product",
    ] {
        let Some(negative) = contract
            .negative_fixtures
            .iter()
            .find(|fixture| fixture.fixture_id == fixture_id)
        else {
            failures.push(format!("missing-negative-fixture-{fixture_id}"));
            continue;
        };
        if negative.status != "failed-as-expected" {
            failures.push(format!("{fixture_id}-unexpected-status"));
        }
        if negative.failure_reason.trim().is_empty() {
            failures.push(format!("{fixture_id}-missing-failure-reason"));
        }
        if negative.resolution.falls_back_to_generic_hardcoded_content {
            failures.push(format!("{fixture_id}-falls-back-to-hardcoded-content"));
        }
    }

    let core_text = format!(
        "{} {}",
        contract.authority_boundary, contract.core_runtime_policy
    )
    .to_lowercase();
    for term in &contract.forbidden_core_terms {
        if core_text.contains(term) {
            failures.push(format!("core-runtime-authority-contains-{term}"));
        }
    }

    if !failures.is_empty() {
        bail!("v130 product SKU extension contract failed: {failures:?}");
    }

    write_json(Path::new(&args[0]), &contract)
}

fn write_json(path: &Path, payload: &impl Serialize) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create proof directory {}", parent.display()))?;
    }
    fs::write(
        path,
        serde_json::to_string_pretty(payload).context("serialize proof payload")? + "\n",
    )
    .with_context(|| format!("write proof {}", path.display()))?;
    Ok(())
}
