use agentflow_runtime_api::{
    customer_delivery_backend_contract, CUSTOMER_DELIVERY_BACKEND_CONTRACT_VERSION,
};
use anyhow::{bail, Context, Result};
use serde::Serialize;
use std::{collections::HashMap, collections::HashSet, fs, path::Path};

fn main() -> Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.len() != 1 {
        bail!("usage: v130_customer_delivery_backend_contract_proofs <delivery-contract>");
    }

    let contract = customer_delivery_backend_contract();
    let mut failures = Vec::new();

    if contract.version != CUSTOMER_DELIVERY_BACKEND_CONTRACT_VERSION {
        failures.push("wrong-customer-delivery-backend-version".to_string());
    }
    if contract.status != "passed" {
        failures.push("customer-delivery-backend-status-not-passed".to_string());
    }
    if contract.release_version != "v1.3.0" {
        failures.push("wrong-release-version".to_string());
    }

    let required = contract
        .required_bindings
        .iter()
        .map(String::as_str)
        .collect::<HashSet<_>>();
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
        if !required.contains(field) {
            failures.push(format!("missing-required-binding-{field}"));
        }
    }

    let states = contract
        .stable_states
        .iter()
        .map(String::as_str)
        .collect::<HashSet<_>>();
    for state in [
        "accessible",
        "expired",
        "revoked",
        "refunded",
        "repair-needed",
        "rerun-needed",
        "blocked",
    ] {
        if !states.contains(state) {
            failures.push(format!("missing-stable-state-{state}"));
        }
    }

    let accepted = &contract.accepted_delivery_fixture;
    if accepted.status != "passed" {
        failures.push("accepted-fixture-not-passed".to_string());
    }
    if accepted.access_status != "accessible" {
        failures.push("accepted-fixture-not-accessible".to_string());
    }
    if accepted.next_action != "show-download" {
        failures.push("accepted-fixture-wrong-next-action".to_string());
    }
    if !accepted.download_access_visible {
        failures.push("accepted-fixture-download-hidden".to_string());
    }
    if !accepted.access_handle_generated {
        failures.push("accepted-fixture-missing-access-handle".to_string());
    }
    if accepted.access_receipt_ref.is_none() {
        failures.push("accepted-fixture-missing-access-receipt".to_string());
    }
    if !accepted.failure_reasons.is_empty() {
        failures.push("accepted-fixture-has-failure-reasons".to_string());
    }

    let fixtures = contract
        .negative_access_fixtures
        .iter()
        .map(|fixture| (fixture.fixture_id.as_str(), fixture))
        .collect::<HashMap<_, _>>();
    for fixture_id in [
        "expired",
        "revoked",
        "refunded",
        "repair-needed",
        "rerun-needed",
    ] {
        let Some(fixture) = fixtures.get(fixture_id) else {
            failures.push(format!("missing-negative-fixture-{fixture_id}"));
            continue;
        };
        if fixture.status != "failed-as-expected" {
            failures.push(format!("{fixture_id}-unexpected-status"));
        }
        if fixture.download_access_visible {
            failures.push(format!("{fixture_id}-download-access-visible"));
        }
        if fixture.access_handle_generated {
            failures.push(format!("{fixture_id}-access-handle-generated"));
        }
        if fixture.next_action.trim().is_empty() {
            failures.push(format!("{fixture_id}-missing-next-action"));
        }
        if fixture.failure_reasons.is_empty() {
            failures.push(format!("{fixture_id}-missing-failure-reasons"));
        }
        if fixture.source_refs.is_empty() {
            failures.push(format!("{fixture_id}-missing-source-refs"));
        }
    }

    for (fixture_id, expected_next_action) in [
        ("expired", "renew-access"),
        ("revoked", "contact-support"),
        ("refunded", "show-refund-policy"),
        ("repair-needed", "create-repair-proposal"),
        ("rerun-needed", "request-new-authorization"),
    ] {
        let Some(fixture) = fixtures.get(fixture_id) else {
            continue;
        };
        if fixture.next_action != expected_next_action {
            failures.push(format!("{fixture_id}-wrong-next-action"));
        }
    }

    if !failures.is_empty() {
        bail!("v130 customer delivery backend contract failed: {failures:?}");
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
