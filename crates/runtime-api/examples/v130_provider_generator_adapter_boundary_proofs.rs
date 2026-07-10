use agentflow_runtime_api::{
    provider_generator_adapter_boundary_contract, PROVIDER_GENERATOR_ADAPTER_BOUNDARY_VERSION,
};
use anyhow::{bail, Context, Result};
use serde::Serialize;
use std::{collections::HashSet, fs, path::Path};

fn main() -> Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.len() != 1 {
        bail!("usage: v130_provider_generator_adapter_boundary_proofs <adapter-boundary>");
    }

    let contract = provider_generator_adapter_boundary_contract();
    let mut failures = Vec::new();

    if contract.version != PROVIDER_GENERATOR_ADAPTER_BOUNDARY_VERSION {
        failures.push("wrong-provider-generator-adapter-boundary-version".to_string());
    }
    if contract.status != "passed" {
        failures.push("adapter-boundary-status-not-passed".to_string());
    }
    if contract.release_version != "v1.3.0" {
        failures.push("wrong-release-version".to_string());
    }

    let required = contract
        .required_objects
        .iter()
        .map(String::as_str)
        .collect::<HashSet<_>>();
    for object in [
        "inputSnapshot",
        "skuDefinition",
        "generationRequest",
        "generationReceipt",
        "outputArtifact",
        "evidenceRefs",
        "failureReasons",
    ] {
        if !required.contains(object) {
            failures.push(format!("missing-required-object-{object}"));
        }
    }

    let positive = &contract.dry_run_positive_fixture;
    if positive.status != "passed" {
        failures.push("positive-fixture-not-passed".to_string());
    }
    if positive.request.input_snapshot_ref.trim().is_empty() {
        failures.push("positive-missing-input-snapshot".to_string());
    }
    if positive.request.sku_definition_ref.trim().is_empty() {
        failures.push("positive-missing-sku-definition".to_string());
    }
    if positive.request.generator_ref.trim().is_empty() {
        failures.push("positive-missing-generator-ref".to_string());
    }
    if positive.request.provider_ref.trim().is_empty() {
        failures.push("positive-missing-provider-ref".to_string());
    }
    if positive.receipt.status != "succeeded" {
        failures.push("positive-receipt-not-succeeded".to_string());
    }
    if positive.receipt.output_artifact_ref.is_none() {
        failures.push("positive-missing-output-artifact-ref".to_string());
    }
    if positive.receipt.evidence_refs.is_empty() {
        failures.push("positive-missing-evidence-refs".to_string());
    }
    if positive.receipt.provider_specific_call_is_core_authority {
        failures.push("positive-provider-call-promoted-to-core-authority".to_string());
    }
    if positive.receipt.delivery_blocked {
        failures.push("positive-delivery-blocked".to_string());
    }
    let Some(artifact) = positive.artifact.as_ref() else {
        failures.push("positive-missing-artifact".to_string());
        if !failures.is_empty() {
            bail!("v130 provider/generator adapter boundary failed: {failures:?}");
        }
        return write_json(Path::new(&args[0]), &contract);
    };
    if !artifact.produced_by_adapter {
        failures.push("positive-artifact-not-produced-by-adapter".to_string());
    }
    if artifact.writes_core_authority {
        failures.push("positive-artifact-writes-core-authority".to_string());
    }

    let fixtures = contract
        .negative_fixtures
        .iter()
        .map(|fixture| (fixture.fixture_id.as_str(), fixture))
        .collect::<std::collections::HashMap<_, _>>();
    for fixture_id in [
        "missing-input-snapshot",
        "provider-call-promoted-to-core-authority",
        "failed-generation-keeps-delivery-blocked",
    ] {
        let Some(fixture) = fixtures.get(fixture_id) else {
            failures.push(format!("missing-negative-fixture-{fixture_id}"));
            continue;
        };
        if fixture.status != "failed-as-expected" {
            failures.push(format!("{fixture_id}-unexpected-status"));
        }
        if !fixture.receipt.delivery_blocked {
            failures.push(format!("{fixture_id}-delivery-not-blocked"));
        }
        if fixture.receipt.failure_reasons.is_empty() {
            failures.push(format!("{fixture_id}-missing-failure-reasons"));
        }
        if fixture.expected_delivery_state != "blocked" {
            failures.push(format!("{fixture_id}-wrong-expected-delivery-state"));
        }
        if fixture.artifact.is_some() {
            failures.push(format!("{fixture_id}-unexpected-artifact"));
        }
    }

    let Some(provider_leak) = fixtures.get("provider-call-promoted-to-core-authority") else {
        failures.push("missing-provider-leak-fixture".to_string());
        if !failures.is_empty() {
            bail!("v130 provider/generator adapter boundary failed: {failures:?}");
        }
        return write_json(Path::new(&args[0]), &contract);
    };
    if !provider_leak
        .receipt
        .provider_specific_call_is_core_authority
    {
        failures.push("provider-leak-fixture-does-not-mark-core-authority-attempt".to_string());
    }

    if !failures.is_empty() {
        bail!("v130 provider/generator adapter boundary failed: {failures:?}");
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
