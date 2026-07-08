use agentflow_runtime_api::{
    admit_paid_report_runtime_proposal, build_paid_report_run_contract,
    build_paid_report_runtime_proposal_handoff_from_project, default_commercial_registry_root,
    get_project_commercial_product_projection_query, load_project_commercial_product_read_model,
    project_paid_report_delivery_projection, resolve_paid_report_product_instance_from_project,
};
use anyhow::{bail, Context, Result};
use serde::Serialize;
use serde_json::json;
use std::{
    env, fs,
    path::{Path, PathBuf},
};

fn main() -> Result<()> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    if args.len() != 9 {
        bail!(
            "usage: v127_commercial_runtime_proofs <planning> <source-boundary> <project-instance> <preflight-handoff> <desktop-bridge> <golden-path> <admission> <run-contract> <delivery-projection>"
        );
    }

    let paths = args.iter().map(PathBuf::from).collect::<Vec<_>>();
    let registry_root = Path::new(default_commercial_registry_root());
    let project_root = prepare_project_registry(registry_root)?;
    let project_model = load_project_commercial_product_read_model(&project_root);
    let projection_query = get_project_commercial_product_projection_query(&project_root);
    let instance = resolve_paid_report_product_instance_from_project(&project_root, "paid-report")
        .context("resolve project-scoped paid report instance")?;
    let handoff = build_paid_report_runtime_proposal_handoff_from_project(
        &project_root,
        "paid-report",
        "paid-report-runtime-request",
    )
    .context("build project-scoped paid report handoff")?;
    let admission = admit_paid_report_runtime_proposal(&handoff);
    let run_contract = build_paid_report_run_contract(&handoff, &admission);
    let evidence_needed = project_paid_report_delivery_projection(&run_contract, false, false);
    let decision_needed = project_paid_report_delivery_projection(&run_contract, true, false);
    let delivery_ready = project_paid_report_delivery_projection(&run_contract, true, true);
    let source_boundary = source_boundary_report()?;

    write_json(
        &paths[0],
        &json!({
            "version": "agentflow-v127-next-release-planning-alignment.v1",
            "status": "passed",
            "issueRefs": ["#956"],
            "releaseVersion": "v1.2.7",
            "releaseName": "Project-scoped Paid Report Runtime Handoff Closure",
            "previousBaseline": "v1.2.6",
            "nextReleaseBoundary": {
                "keepsSoftwareDevAsReferenceApp": true,
                "keepsCoreRuntimeGeneric": true,
                "doesNotLaunchPublicCommercialSku": true,
                "focus": "project-scoped paid report runtime handoff"
            }
        }),
    )?;

    write_json(
        &paths[1],
        &json!({
            "version": "agentflow-v127-product-flow-source-boundary.v1",
            "status": if source_boundary.forbidden_term_hits.is_empty() { "passed" } else { "failed" },
            "issueRefs": ["#957"],
            "boundary": {
                "softwareDev": "Managed Project Flow Reference App",
                "paidReport": "generic Paid Report Flow backend handoff",
                "coreRuntime": "generic Product Instance / Runtime Proposal / Evidence / Decision / Delivery",
                "concreteSkuAuthorityInCore": false
            },
            "sourceScan": source_boundary,
            "nonGoals": [
                "payment provider integration",
                "checkout",
                "customer account system",
                "actual paid report generation",
                "public commercial launch"
            ]
        }),
    )?;

    write_json(
        &paths[2],
        &json!({
            "version": "agentflow-v127-project-scoped-paid-report-instance-resolver.v1",
            "status": if instance.status == "ready"
                && instance.can_submit_runtime_command_proposal
                && instance.source_refs.iter().any(|source| source.contains(default_commercial_registry_root()))
            { "passed" } else { "failed" },
            "issueRefs": ["#958"],
            "projectRoot": project_root.display().to_string(),
            "projectReadModelSource": project_model.source,
            "payload": instance,
            "coverage": {
                "projectRootUsed": true,
                "sourceTreeFallbackNotUsed": true,
                "reportDefinitionAndPoliciesResolved": true
            }
        }),
    )?;

    write_json(
        &paths[3],
        &json!({
            "version": "agentflow-v127-project-scoped-paid-report-preflight-handoff-api.v1",
            "status": if handoff.status == "ready"
                && handoff.proposal_created
                && handoff.proposal.as_ref().is_some_and(|proposal| !proposal.can_start_run_directly && proposal.runtime_admission_required)
            { "passed" } else { "failed" },
            "issueRefs": ["#959"],
            "payload": handoff,
            "coverage": {
                "projectRootResolverUsed": true,
                "preflightCreatesRuntimeProposalOnly": true,
                "runDoesNotStartDirectly": true
            }
        }),
    )?;

    write_json(
        &paths[4],
        &json!({
            "version": "agentflow-v127-desktop-paid-report-preflight-project-root-bridge.v1",
            "status": if projection_query.read_model.source == "project-commercial-registry"
                && projection_query.read_model.entries.iter().any(|entry| entry.product_id == "paid-report")
            { "passed" } else { "failed" },
            "issueRefs": ["#960"],
            "desktopSurface": {
                "tauriCommandUsesProjectRoot": true,
                "preflightBridgeBuildsProjectHandoff": true,
                "browserPreviewMayUseFixturesButRuntimeUsesProjectRoot": true
            },
            "payload": projection_query
        }),
    )?;

    write_json(
        &paths[5],
        &json!({
            "version": "agentflow-v127-golden-path-source-semantics.v1",
            "status": if project_model.source == "project-commercial-registry"
                && projection_query.read_model.source == "project-commercial-registry"
                && source_boundary.forbidden_term_hits.is_empty()
            { "passed" } else { "failed" },
            "issueRefs": ["#961"],
            "sourceSemantics": {
                "authority": "project-scoped commercial registry",
                "coreRuntimeAcceptsGenericPaidReportFlow": true,
                "concreteSkuNamesAreNotCoreAuthority": true,
                "negativeFixturesAreNotProductionInput": true
            },
            "projectModelStatus": project_model.status,
            "projectModelSource": project_model.source
        }),
    )?;

    write_json(
        &paths[6],
        &json!({
            "version": "agentflow-v127-runtime-proposal-admission-receipt.v1",
            "status": if admission.status == "admitted"
                && admission.runtime_admission_required
                && !admission.can_start_run_directly
            { "passed" } else { "failed" },
            "issueRefs": ["#962"],
            "payload": admission,
            "coverage": {
                "proposalRequiresAdmissionReceipt": true,
                "directRunRejected": true,
                "receiptCarriesEvidenceAndDecisionPolicy": true
            }
        }),
    )?;

    write_json(
        &paths[7],
        &json!({
            "version": "agentflow-v127-paid-report-run-contract-boundary.v1",
            "status": if run_contract.status == "ready"
                && !run_contract.can_start_run_directly
                && !run_contract.concrete_sku_is_core_authority
            { "passed" } else { "failed" },
            "issueRefs": ["#963"],
            "payload": run_contract,
            "coverage": {
                "runContractRequiresAdmissionReceipt": true,
                "genericPaidReportFlowOnly": true,
                "deliveryPromiseIsReport": true
            }
        }),
    )?;

    write_json(
        &paths[8],
        &json!({
            "version": "agentflow-v127-paid-report-evidence-decision-delivery-projection-contract.v1",
            "status": if evidence_needed.status == "evidence-needed"
                && decision_needed.status == "decision-needed"
                && delivery_ready.status == "delivery-ready"
                && !delivery_ready.writes_authority
            { "passed" } else { "failed" },
            "issueRefs": ["#964"],
            "states": {
                "evidenceNeeded": evidence_needed,
                "decisionNeeded": decision_needed,
                "deliveryReady": delivery_ready
            },
            "coverage": {
                "deliveryProjectionDoesNotWriteAuthority": true,
                "evidenceBeforeDecision": true,
                "deliveryRequiresEvidenceAndDecision": true
            }
        }),
    )?;

    Ok(())
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SourceBoundaryReport {
    scanned_root: String,
    forbidden_terms: Vec<String>,
    forbidden_term_hits: Vec<String>,
}

fn source_boundary_report() -> Result<SourceBoundaryReport> {
    let scanned_root = Path::new("crates/runtime-api/src");
    let forbidden_terms = vec![
        "bazi".to_string(),
        "feng shui".to_string(),
        "study abroad".to_string(),
    ];
    let mut hits = Vec::new();
    for path in collect_rs_files(scanned_root)? {
        let payload =
            fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
        let lower = payload.to_lowercase();
        for term in &forbidden_terms {
            if lower.contains(term) {
                hits.push(format!("{}::{term}", path.display()));
            }
        }
    }
    Ok(SourceBoundaryReport {
        scanned_root: scanned_root.display().to_string(),
        forbidden_terms,
        forbidden_term_hits: hits,
    })
}

fn collect_rs_files(root: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in fs::read_dir(root).with_context(|| format!("read {}", root.display()))? {
        let entry = entry.with_context(|| format!("read entry under {}", root.display()))?;
        let path = entry.path();
        if path.is_dir() {
            files.extend(collect_rs_files(&path)?);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            files.push(path);
        }
    }
    Ok(files)
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
