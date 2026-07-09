use agentflow_runtime_api::{commercial_authority_boundary, COMMERCIAL_AUTHORITY_BOUNDARY_VERSION};
use anyhow::{bail, Context, Result};
use serde::Serialize;
use std::{collections::HashSet, fs, path::Path};

fn main() -> Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.len() != 1 {
        bail!("usage: v130_commercial_authority_boundary_proofs <authority-boundary>");
    }

    let boundary = commercial_authority_boundary();
    let mut failures = Vec::new();

    if boundary.version != COMMERCIAL_AUTHORITY_BOUNDARY_VERSION {
        failures.push("wrong-authority-boundary-version".to_string());
    }
    if boundary.status != "passed" {
        failures.push("authority-boundary-status-not-passed".to_string());
    }
    if boundary.release_version != "v1.3.0" {
        failures.push("wrong-release-version".to_string());
    }

    let object_names = boundary
        .authority_map
        .iter()
        .map(|rule| rule.object_name.as_str())
        .collect::<HashSet<_>>();
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
        if !object_names.contains(required) {
            failures.push(format!("missing-authority-rule-{required}"));
        }
    }

    for rule in &boundary.authority_map {
        if rule.contract_version.trim().is_empty() {
            failures.push(format!("{}-missing-contract-version", rule.object_name));
        }
        if rule.projection_only && (rule.can_create || rule.can_update || rule.writes_authority) {
            failures.push(format!("{}-projection-writes-authority", rule.object_name));
        }
    }

    let readonly = boundary
        .read_only_surfaces
        .iter()
        .map(String::as_str)
        .collect::<HashSet<_>>();
    for surface in [
        "Projection",
        "Customer View",
        "Download View",
        "Synthetic Release Fixture",
        "Release Sidecar",
    ] {
        if !readonly.contains(surface) {
            failures.push(format!("missing-read-only-surface-{surface}"));
        }
    }

    for fixture_id in [
        "projection-writing-authority",
        "customer-view-writing-authority",
        "download-view-writing-authority",
        "synthetic-release-sidecar-promoted-as-authority",
        "release-sidecar-promoted-as-authority",
    ] {
        let Some(fixture) = boundary
            .negative_fixtures
            .iter()
            .find(|fixture| fixture.fixture_id == fixture_id)
        else {
            failures.push(format!("missing-negative-fixture-{fixture_id}"));
            continue;
        };
        if fixture.status != "failed-as-expected" {
            failures.push(format!("{fixture_id}-unexpected-status"));
        }
        if fixture.can_write_authority {
            failures.push(format!("{fixture_id}-can-write-authority"));
        }
        if fixture.failure_reason.trim().is_empty() {
            failures.push(format!("{fixture_id}-missing-failure-reason"));
        }
    }

    if !boundary
        .synthetic_release_sidecar_policy
        .contains("cannot replace live GitHub release provenance")
    {
        failures.push("missing-synthetic-sidecar-policy".to_string());
    }

    if !failures.is_empty() {
        bail!("v130 commercial authority boundary failed: {failures:?}");
    }

    write_json(Path::new(&args[0]), &boundary)
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
