use agentflow_runtime_api::{
    payment_provider_adapter_boundary_contract, PAYMENT_PROVIDER_ADAPTER_BOUNDARY_VERSION,
};
use anyhow::{bail, Context, Result};
use serde::Serialize;
use std::{collections::HashMap, collections::HashSet, fs, path::Path};

fn main() -> Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.len() != 1 {
        bail!("usage: v130_payment_provider_adapter_boundary_proofs <payment-boundary>");
    }

    let contract = payment_provider_adapter_boundary_contract();
    let mut failures = Vec::new();

    if contract.version != PAYMENT_PROVIDER_ADAPTER_BOUNDARY_VERSION {
        failures.push("wrong-payment-provider-adapter-boundary-version".to_string());
    }
    if contract.status != "passed" {
        failures.push("payment-boundary-status-not-passed".to_string());
    }
    if contract.release_version != "v1.3.0" {
        failures.push("wrong-release-version".to_string());
    }

    let required = contract
        .required_fields
        .iter()
        .map(String::as_str)
        .collect::<HashSet<_>>();
    for field in [
        "providerPaymentIntentRef",
        "checkoutSessionRef",
        "entitlementAuthorizationRef",
        "paymentStatus",
        "refundStatus",
        "sourceRefs",
    ] {
        if !required.contains(field) {
            failures.push(format!("missing-required-field-{field}"));
        }
    }

    let fixtures = contract
        .dry_run_fixtures
        .iter()
        .map(|fixture| (fixture.fixture_id.as_str(), fixture))
        .collect::<HashMap<_, _>>();

    for fixture_id in ["paid", "failed", "refunded", "revoked", "missing"] {
        let Some(fixture) = fixtures.get(fixture_id) else {
            failures.push(format!("missing-fixture-{fixture_id}"));
            continue;
        };
        if fixture.source_refs.is_empty() {
            failures.push(format!("{fixture_id}-missing-source-refs"));
        }
        if fixture.provider_checkout_implementation_is_core_authority {
            failures.push(format!(
                "{fixture_id}-checkout-implementation-is-core-authority"
            ));
        }
        if fixture.provider_refund_execution_is_core_authority {
            failures.push(format!("{fixture_id}-refund-execution-is-core-authority"));
        }
    }

    let Some(paid) = fixtures.get("paid") else {
        failures.push("missing-paid-fixture".to_string());
        if !failures.is_empty() {
            bail!("v130 payment provider adapter boundary failed: {failures:?}");
        }
        return write_json(Path::new(&args[0]), &contract);
    };
    if paid.status != "passed" {
        failures.push("paid-fixture-not-passed".to_string());
    }
    if paid.payment_status != "paid" {
        failures.push("paid-fixture-wrong-payment-status".to_string());
    }
    if paid.refund_status != "none" {
        failures.push("paid-fixture-wrong-refund-status".to_string());
    }
    if paid.provider_payment_intent_ref.is_none()
        || paid.checkout_session_ref.is_none()
        || paid.entitlement_authorization_ref.is_none()
    {
        failures.push("paid-fixture-missing-provider-or-entitlement-ref".to_string());
    }
    if !paid.core_consumes_authorization_result || !paid.core_consumes_provider_evidence {
        failures.push("paid-fixture-core-does-not-consume-normalized-result".to_string());
    }

    for fixture_id in ["failed", "refunded", "revoked", "missing"] {
        let Some(fixture) = fixtures.get(fixture_id) else {
            failures.push(format!("missing-negative-fixture-{fixture_id}"));
            continue;
        };
        if fixture.status != "failed-as-expected" {
            failures.push(format!("{fixture_id}-unexpected-status"));
        }
        if fixture.failure_reason.trim().is_empty() {
            failures.push(format!("{fixture_id}-missing-failure-reason"));
        }
    }

    let Some(refunded) = fixtures.get("refunded") else {
        failures.push("missing-refunded-fixture".to_string());
        if !failures.is_empty() {
            bail!("v130 payment provider adapter boundary failed: {failures:?}");
        }
        return write_json(Path::new(&args[0]), &contract);
    };
    if refunded.refund_status != "refunded" {
        failures.push("refunded-fixture-wrong-refund-status".to_string());
    }
    if refunded.entitlement_effect != "entitlement-refunded" {
        failures.push("refunded-fixture-wrong-entitlement-effect".to_string());
    }

    let Some(missing) = fixtures.get("missing") else {
        failures.push("missing-missing-fixture".to_string());
        if !failures.is_empty() {
            bail!("v130 payment provider adapter boundary failed: {failures:?}");
        }
        return write_json(Path::new(&args[0]), &contract);
    };
    if missing.payment_status != "missing" || missing.refund_status != "unknown" {
        failures.push("missing-fixture-wrong-statuses".to_string());
    }
    if missing.provider_payment_intent_ref.is_some()
        || missing.checkout_session_ref.is_some()
        || missing.entitlement_authorization_ref.is_some()
    {
        failures.push("missing-fixture-unexpected-provider-or-entitlement-ref".to_string());
    }

    if !failures.is_empty() {
        bail!("v130 payment provider adapter boundary failed: {failures:?}");
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
