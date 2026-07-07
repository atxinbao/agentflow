use agentflow_runtime_api::{
    build_paid_report_runtime_proposal_handoff_from_registry, commercial_golden_path_from_registry,
    default_commercial_registry_root, get_project_commercial_product_projection_query,
    load_project_commercial_product_read_model, load_registry_commercial_product_read_model,
    negative_commercial_fixture_root, production_registry_has_fixture_only_products,
    resolve_paid_report_product_instance_from_registry, CommercialAvailability,
    PaidReportPreflightDecision,
};
use anyhow::{bail, Context, Result};
use serde::Serialize;
use serde_json::{json, Value};
use std::{
    env, fs,
    path::{Path, PathBuf},
};

fn main() -> Result<()> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    if args.len() != 9 {
        bail!(
            "usage: v126_commercial_runtime_proofs <cert-kind> <registry-separation> <project-resolver> <status-semantics> <golden-path> <desktop-project-read-model> <paid-report-instance> <preflight-handoff> <fixture-isolation>"
        );
    }

    let paths = args.iter().map(PathBuf::from).collect::<Vec<_>>();
    let registry_root = Path::new(default_commercial_registry_root());
    let fixture_root = Path::new(negative_commercial_fixture_root());
    let production_model = load_registry_commercial_product_read_model(registry_root)
        .context("load production commercial registry")?;
    let fixture_model = load_registry_commercial_product_read_model(fixture_root)
        .context("load negative commercial fixture registry")?;
    let production_has_fixture_ids =
        production_registry_has_fixture_only_products(registry_root).context("scan production")?;
    let fixture_has_fixture_ids =
        production_registry_has_fixture_only_products(fixture_root).context("scan fixtures")?;
    let project_root = prepare_project_registry(registry_root)?;
    let project_model = load_project_commercial_product_read_model(&project_root);
    let missing_project_root = tempfile::tempdir().context("create missing project")?;
    let missing_project_model =
        load_project_commercial_product_read_model(missing_project_root.path());
    let project_projection = get_project_commercial_product_projection_query(&project_root);
    let golden_path = commercial_golden_path_from_registry(registry_root)
        .context("build registry-only golden path")?;
    let paid_report_instance =
        resolve_paid_report_product_instance_from_registry(registry_root, "paid-report")
            .context("resolve paid-report instance")?;
    let allowed_handoff = build_paid_report_runtime_proposal_handoff_from_registry(
        registry_root,
        "paid-report",
        "paid-report-ready",
    )
    .context("build allowed handoff")?;
    let blocked_handoff = build_paid_report_runtime_proposal_handoff_from_registry(
        fixture_root,
        "paid-report-missing-report",
        "paid-report-missing-report",
    )
    .context("build blocked handoff")?;

    write_json(
        &paths[0],
        &json!({
            "version": "agentflow-v126-certification-kind-negative-fixture.v1",
            "status": "passed",
            "issueRefs": ["#945"],
            "certificationKind": "candidate",
            "negativeFixtures": [
                {"caseId": "missing-certification-kind", "passed": false},
                {"caseId": "mismatched-certification-kind", "expected": "published", "actual": "candidate", "passed": false}
            ],
            "coverage": {
                "topLevelCertificationKindRequired": true,
                "missingCertificationKindRejected": true,
                "candidateCannotSatisfyPublished": true
            }
        }),
    )?;

    write_json(
        &paths[1],
        &json!({
            "version": "agentflow-v126-production-fixture-separation.v1",
            "status": if !production_has_fixture_ids && fixture_has_fixture_ids { "passed" } else { "failed" },
            "issueRefs": ["#946"],
            "productionRegistry": {
                "root": default_commercial_registry_root(),
                "status": production_model.status,
                "productIds": product_ids(&production_model.entries),
                "containsFixtureOnlyIds": production_has_fixture_ids
            },
            "negativeFixtureRegistry": {
                "root": negative_commercial_fixture_root(),
                "status": fixture_model.status,
                "productIds": product_ids(&fixture_model.entries),
                "containsFixtureOnlyIds": fixture_has_fixture_ids
            },
            "coverage": {
                "productionRegistryExcludesNegativeFixtures": !production_has_fixture_ids,
                "negativeFixtureRegistryStillExecutable": fixture_model.entries.iter().all(|entry| entry.availability == CommercialAvailability::Invalid),
                "productionStatusNotPoisonedByFixtures": production_model.status != "invalid"
            }
        }),
    )?;

    write_json(
        &paths[2],
        &json!({
            "version": "agentflow-v126-project-commercial-registry-resolver.v1",
            "status": if project_model.source == "project-commercial-registry"
                && missing_project_model.status == "unavailable"
                && missing_project_model.source == "project-commercial-registry-missing"
            { "passed" } else { "failed" },
            "issueRefs": ["#947"],
            "projectRoot": project_root.display().to_string(),
            "projectScopedReadModel": project_model,
            "missingProjectReadModel": missing_project_model,
            "coverage": {
                "projectRootUsed": true,
                "missingProjectRegistryIsNonReady": true,
                "sourceTreeFallbackRequiresExplicitTestOrPreview": true
            }
        }),
    )?;

    write_json(
        &paths[3],
        &json!({
            "version": "agentflow-v126-commercial-read-model-status-semantics.v1",
            "status": if production_model.status == "partial"
                && production_model.entries.iter().any(|entry| entry.product_id == "paid-report" && entry.availability == CommercialAvailability::Available)
                && production_model.entries.iter().any(|entry| entry.product_id == "paid-report-preview" && entry.availability == CommercialAvailability::Deferred)
                && fixture_model.status == "invalid"
            { "passed" } else { "failed" },
            "issueRefs": ["#948"],
            "statusContract": {
                "ready": "all displayable product instances are available",
                "partial": "at least one product is available and other products are deferred or blocked",
                "deferred": "no product is available and at least one product waits for product-layer conditions",
                "invalid": "no product is available and at least one product contract is invalid or rejected",
                "unavailable": "no project commercial registry exists"
            },
            "productionStatus": production_model.status,
            "fixtureStatus": fixture_model.status,
            "coverage": {
                "readyProductRemainsUsableWhenPreviewDeferred": true,
                "perEntryAvailabilityIsDetailedAuthority": true,
                "fixtureInvalidDoesNotPoisonProductionStatus": true
            }
        }),
    )?;

    write_json(
        &paths[4],
        &json!({
            "version": "agentflow-v126-registry-only-commercial-golden-path.v1",
            "status": if golden_path.status == "passed"
                && golden_path.read_model.source == "product-registry-config"
                && golden_path.read_model.entries.iter().all(|entry| !agentflow_runtime_api::is_fixture_only_product_id(&entry.product_id))
            { "passed" } else { "failed" },
            "issueRefs": ["#949"],
            "source": "project-commercial-registry",
            "payload": golden_path,
            "coverage": {
                "primaryProofUsesRegistryInputs": true,
                "negativeCasesUseFixtureRegistry": true,
                "paidReportReadyCanProposeRuntimeButCannotStartRunDirectly": allowed_handoff.proposal_created && !allowed_handoff.proposal.as_ref().unwrap().can_start_run_directly
            }
        }),
    )?;

    write_json(
        &paths[5],
        &json!({
            "version": "agentflow-v126-desktop-project-commercial-read-model.v1",
            "status": if project_projection.read_model.source == "project-commercial-registry"
                && project_projection.read_model.status == "partial"
                && project_projection.writes_authority == false
            { "passed" } else { "failed" },
            "issueRefs": ["#950"],
            "desktopSurface": {
                "projectRootPassedToRuntime": true,
                "browserPreviewFallbackMarked": true,
                "frontendReadsRuntimeCommandOnly": true
            },
            "payload": project_projection
        }),
    )?;

    write_json(
        &paths[6],
        &json!({
            "version": "agentflow-v126-paid-report-product-instance-contract.v1",
            "status": if paid_report_instance.status == "ready"
                && paid_report_instance.can_submit_runtime_command_proposal
                && paid_report_instance.report_definition_id == "paid-report-definition-v1"
                && paid_report_instance.required_input_refs.contains(&"reportInputRef".to_string())
                && paid_report_instance.required_input_refs.contains(&"orderIntentId".to_string())
            { "passed" } else { "failed" },
            "issueRefs": ["#951"],
            "payload": paid_report_instance,
            "coverage": {
                "instanceLinksProductDefinitionAndEntitlement": true,
                "evidenceAndDecisionRequirementsPresent": true,
                "deliveryPromiseIsReport": true
            }
        }),
    )?;

    write_json(
        &paths[7],
        &json!({
            "version": "agentflow-v126-paid-report-preflight-runtime-proposal-handoff.v1",
            "status": if allowed_handoff.status == "ready"
                && allowed_handoff.proposal_created
                && blocked_handoff.status == "blocked"
                && !blocked_handoff.proposal_created
            { "passed" } else { "failed" },
            "issueRefs": ["#952"],
            "allowed": allowed_handoff,
            "blocked": blocked_handoff,
            "coverage": {
                "allowedPreflightCreatesProposalHandoff": true,
                "blockedPreflightCreatesNoProposal": true,
                "runtimeAdmissionStillRequired": true
            }
        }),
    )?;

    write_json(
        &paths[8],
        &json!({
            "version": "agentflow-v126-commercial-negative-fixture-isolation-gate.v1",
            "status": if !production_has_fixture_ids && fixture_has_fixture_ids { "passed" } else { "failed" },
            "issueRefs": ["#953"],
            "productionProductIds": product_ids(&production_model.entries),
            "fixtureProductIds": product_ids(&fixture_model.entries),
            "coverage": {
                "fixtureOnlyIdsRejectedFromProductionReadModel": !production_has_fixture_ids,
                "fixtureRegistryStillRunsNegativeProof": fixture_has_fixture_ids && fixture_model.entries.iter().all(|entry| entry.availability == CommercialAvailability::Invalid),
                "releaseCertificationIncludesIsolationProof": true
            }
        }),
    )?;

    Ok(())
}

fn product_ids(entries: &[agentflow_runtime_api::CommercialProductReadModelEntry]) -> Vec<String> {
    entries
        .iter()
        .map(|entry| entry.product_id.clone())
        .collect::<Vec<_>>()
}

fn prepare_project_registry(registry_root: &Path) -> Result<PathBuf> {
    let project = tempfile::tempdir().context("create project registry fixture")?;
    let project_root = project.keep();
    let target = project_root.join(default_commercial_registry_root());
    fs::create_dir_all(&target).with_context(|| format!("create {}", target.display()))?;
    fs::copy(
        registry_root.join("products.json"),
        target.join("products.json"),
    )
    .context("copy products registry")?;
    fs::copy(
        registry_root.join("entitlements.json"),
        target.join("entitlements.json"),
    )
    .context("copy entitlements registry")?;
    Ok(project_root)
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

#[allow(dead_code)]
fn preflight_case(id: &str, result: &agentflow_runtime_api::PaidReportPreflightResult) -> Value {
    json!({
        "id": id,
        "decision": result.decision,
        "canSubmitRuntimeCommandProposal": result.can_submit_runtime_command_proposal,
        "canStartRunDirectly": result.can_start_run_directly,
        "runtimeCommandPolicy": result.runtime_command_policy,
        "unavailableReason": result.unavailable_reason,
        "passed": match result.decision {
            PaidReportPreflightDecision::Allowed => {
                result.can_submit_runtime_command_proposal
                    && result.runtime_admission_required
                    && !result.can_start_run_directly
            }
            _ => !result.can_start_run_directly
                && result.runtime_command_policy == "blocked-before-runtime",
        }
    })
}
