use agentflow_runtime_api::{
    commercial_e2e_golden_scenario, COMMERCIAL_E2E_GOLDEN_SCENARIO_VERSION,
};
use anyhow::{bail, Context, Result};
use serde::Serialize;
use std::{collections::HashSet, fs, path::Path};

fn main() -> Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.len() != 1 {
        bail!("usage: v130_commercial_e2e_golden_scenario_proofs <golden-scenario>");
    }

    let scenario = commercial_e2e_golden_scenario();
    let mut failures = Vec::new();

    if scenario.version != COMMERCIAL_E2E_GOLDEN_SCENARIO_VERSION {
        failures.push("wrong-commercial-e2e-golden-scenario-version".to_string());
    }
    if scenario.status != "passed" {
        failures.push("commercial-e2e-golden-scenario-status-not-passed".to_string());
    }
    if scenario.release_version != "v1.3.0" {
        failures.push("wrong-release-version".to_string());
    }
    if scenario.concrete_domain_sku_implemented {
        failures.push("concrete-domain-sku-implemented".to_string());
    }

    let fact_types = scenario
        .ordered_facts
        .iter()
        .map(|fact| fact.fact_type.as_str())
        .collect::<Vec<_>>();
    let expected = vec![
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
    ];
    if fact_types != expected {
        failures.push("ordered-facts-do-not-match-backend-chain".to_string());
    }

    for fact in &scenario.ordered_facts {
        if fact.contract_version.trim().is_empty() {
            failures.push(format!("{}-missing-contract-version", fact.fact_id));
        }
        if fact.authority_owner.trim().is_empty() {
            failures.push(format!("{}-missing-authority-owner", fact.fact_id));
        }
        if fact.source_ref.trim().is_empty() {
            failures.push(format!("{}-missing-source-ref", fact.fact_id));
        }
    }

    if scenario.success_path.status != "passed" {
        failures.push("success-path-not-passed".to_string());
    }
    if scenario.success_path.decision_outcome != "accepted" {
        failures.push("success-path-not-accepted".to_string());
    }
    if scenario.success_path.delivery_status != "delivery-ready" {
        failures.push("success-path-not-delivery-ready".to_string());
    }
    if !scenario.success_path.download_access_visible {
        failures.push("success-path-download-hidden".to_string());
    }
    if !scenario.success_path.access_handle_generated {
        failures.push("success-path-missing-access-handle".to_string());
    }
    if scenario.success_path.mutates_delivered_artifact {
        failures.push("success-path-mutates-delivered-artifact".to_string());
    }
    if scenario.success_path.fact_refs.len() != scenario.ordered_facts.len() {
        failures.push("success-path-does-not-reference-all-facts".to_string());
    }

    if scenario.failure_repair_path.status != "failed-as-expected" {
        failures.push("repair-path-unexpected-status".to_string());
    }
    if scenario.failure_repair_path.decision_outcome != "needs-fix" {
        failures.push("repair-path-wrong-decision-outcome".to_string());
    }
    if scenario.failure_repair_path.delivery_status != "repair-needed" {
        failures.push("repair-path-wrong-delivery-status".to_string());
    }
    if scenario.failure_repair_path.download_access_visible {
        failures.push("repair-path-download-visible".to_string());
    }
    if scenario.failure_repair_path.access_handle_generated {
        failures.push("repair-path-access-handle-generated".to_string());
    }
    if scenario.failure_repair_path.mutates_delivered_artifact {
        failures.push("repair-path-mutates-delivered-artifact".to_string());
    }
    if scenario.failure_repair_path.next_action != "create-repair-proposal" {
        failures.push("repair-path-wrong-next-action".to_string());
    }

    let artifacts = scenario
        .certification_artifact_refs
        .iter()
        .map(String::as_str)
        .collect::<HashSet<_>>();
    for artifact in [
        "runtime/v130-commercial-backend-stable-contract.json",
        "runtime/v130-paid-report-flow-state-machine.json",
        "runtime/v130-commercial-authority-boundary.json",
        "runtime/v130-product-sku-extension-contract.json",
        "runtime/v130-provider-generator-adapter-boundary.json",
        "runtime/v130-payment-provider-adapter-boundary.json",
        "runtime/v130-customer-delivery-backend-contract.json",
    ] {
        if !artifacts.contains(artifact) {
            failures.push(format!("missing-certification-artifact-{artifact}"));
        }
    }

    if !failures.is_empty() {
        bail!("v130 commercial E2E golden scenario failed: {failures:?}");
    }

    write_json(Path::new(&args[0]), &scenario)
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
