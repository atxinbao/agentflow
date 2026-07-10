use agentflow_runtime_api::{
    commercial_release_certification, COMMERCIAL_RELEASE_CERTIFICATION_VERSION,
};
use anyhow::{bail, Context, Result};
use serde::Serialize;
use std::{collections::HashSet, fs, path::Path};

fn main() -> Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.len() != 3 {
        bail!("usage: v130_release_certification_proofs <certification> <source-commit> <workflow-run-id>");
    }

    let certification = commercial_release_certification(&args[1], &args[2]);
    let mut failures = Vec::new();

    if certification.version != COMMERCIAL_RELEASE_CERTIFICATION_VERSION {
        failures.push("wrong-v130-release-certification-version".to_string());
    }
    if certification.status != "passed" {
        failures.push("v130-release-certification-not-passed".to_string());
    }
    if certification.release_version != "v1.3.0" || certification.release_tag != "v1.3.0" {
        failures.push("wrong-v130-release-metadata".to_string());
    }
    if certification.source_commit.trim().is_empty() {
        failures.push("missing-source-commit".to_string());
    }
    if certification.workflow_run_id.trim().is_empty() {
        failures.push("missing-workflow-run-id".to_string());
    }
    if certification.artifact_names
        != vec![
            "agentflow-release-certification".to_string(),
            "agentflow-release-gate-full".to_string(),
        ]
    {
        failures.push("wrong-artifact-names".to_string());
    }
    if !certification.commercial_backend_stable {
        failures.push("commercial-backend-not-stable".to_string());
    }

    let expected_proofs = vec![
        "docs/delivery/releases/v1.3.0/proofs/v130-001-v129-release-audit-facts.json",
        "runtime/v130-commercial-backend-stable-contract.json",
        "runtime/v130-paid-report-flow-state-machine.json",
        "runtime/v130-commercial-authority-boundary.json",
        "runtime/v130-product-sku-extension-contract.json",
        "runtime/v130-provider-generator-adapter-boundary.json",
        "runtime/v130-payment-provider-adapter-boundary.json",
        "runtime/v130-customer-delivery-backend-contract.json",
        "runtime/v130-commercial-e2e-golden-scenario.json",
        "runtime/v130-release-certification.json",
    ];
    let proof_set = certification
        .primary_proofs
        .iter()
        .map(String::as_str)
        .collect::<HashSet<_>>();
    for proof in expected_proofs {
        if !proof_set.contains(proof) {
            failures.push(format!("missing-primary-proof-{proof}"));
        }
    }

    let indexed_issues = certification
        .primary_proof_index
        .iter()
        .map(|proof| proof.issue_ref.as_str())
        .collect::<HashSet<_>>();
    for issue in 993..=1001 {
        let issue_ref = format!("#{issue}");
        if !indexed_issues.contains(issue_ref.as_str()) {
            failures.push(format!("missing-primary-proof-index-{issue_ref}"));
        }
    }

    if certification.boundary.public_commercial_launch {
        failures.push("public-commercial-launch-enabled".to_string());
    }
    if certification.boundary.concrete_paid_report_sku {
        failures.push("concrete-paid-report-sku-enabled".to_string());
    }
    if certification.boundary.payment_provider_checkout {
        failures.push("payment-provider-checkout-enabled".to_string());
    }
    if certification.boundary.real_provider_generation {
        failures.push("real-provider-generation-enabled".to_string());
    }
    if certification.boundary.cloud_multi_tenant_launch {
        failures.push("cloud-multi-tenant-launch-enabled".to_string());
    }
    if certification.boundary.full_customer_account_system {
        failures.push("full-customer-account-system-enabled".to_string());
    }
    if certification.boundary.concrete_domain_copy_in_core_runtime {
        failures.push("concrete-domain-copy-in-core-runtime".to_string());
    }
    if !certification.milestone_can_close_only_after_all_v130_issues_complete {
        failures.push("milestone-closeout-not-gated-by-issues".to_string());
    }
    if !certification.milestone_can_close_only_after_release_gate_passes {
        failures.push("milestone-closeout-not-gated-by-release-gate".to_string());
    }
    if certification.coverage.values().any(|passed| !passed) {
        failures.push("coverage-not-passed".to_string());
    }

    if !failures.is_empty() {
        bail!("v130 release certification failed: {failures:?}");
    }

    write_json(Path::new(&args[0]), &certification)
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
