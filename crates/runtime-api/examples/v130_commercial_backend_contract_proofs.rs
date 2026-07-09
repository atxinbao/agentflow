use agentflow_runtime_api::{
    commercial_backend_stable_contract, COMMERCIAL_BACKEND_STABLE_CONTRACT_VERSION,
};
use anyhow::{bail, Context, Result};
use serde::Serialize;
use std::{collections::HashSet, fs, path::Path};

fn main() -> Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.len() != 1 {
        bail!("usage: v130_commercial_backend_contract_proofs <stable-contract>");
    }

    let contract = commercial_backend_stable_contract();
    let mut failures = Vec::new();

    if contract.version != COMMERCIAL_BACKEND_STABLE_CONTRACT_VERSION {
        failures.push("wrong-contract-version".to_string());
    }
    if contract.status != "passed" {
        failures.push("contract-status-not-passed".to_string());
    }
    if contract.release_version != "v1.3.0" {
        failures.push("wrong-release-version".to_string());
    }
    if !contract
        .migration_policy
        .backward_incompatible_changes_require_version_bump
    {
        failures.push("missing-version-bump-policy".to_string());
    }
    if !contract.migration_policy.explicit_migration_required {
        failures.push("missing-explicit-migration-policy".to_string());
    }

    let required_objects = [
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
    ];
    let object_names = contract
        .objects
        .iter()
        .map(|object| object.object_name.as_str())
        .collect::<HashSet<_>>();
    for object in required_objects {
        if !object_names.contains(object) {
            failures.push(format!("missing-object-{object}"));
        }
    }

    for object in &contract.objects {
        if object.version.trim().is_empty() {
            failures.push(format!("{}-missing-version", object.object_name));
        }
        if object.required_fields.iter().any(|field| !field.required) {
            failures.push(format!(
                "{}-required-field-not-required",
                object.object_name
            ));
        }
        if !object
            .required_fields
            .iter()
            .any(|field| field.name == "version")
        {
            failures.push(format!("{}-missing-version-field", object.object_name));
        }
        if !object
            .required_fields
            .iter()
            .any(|field| field.name == "status")
        {
            failures.push(format!("{}-missing-status-field", object.object_name));
        }
        if object.status_values.is_empty() {
            failures.push(format!("{}-missing-status-values", object.object_name));
        }
    }

    let states = contract
        .error_decision_model
        .stable_states
        .iter()
        .map(|state| state.state.as_str())
        .collect::<HashSet<_>>();
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
        if !states.contains(state) {
            failures.push(format!("missing-state-{state}"));
        }
    }

    if !failures.is_empty() {
        bail!("v130 commercial backend stable contract failed: {failures:?}");
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
