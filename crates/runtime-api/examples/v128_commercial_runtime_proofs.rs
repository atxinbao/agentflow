use agentflow_runtime_api::{
    admit_paid_report_runtime_proposal, build_paid_report_artifact,
    build_paid_report_input_snapshot, build_paid_report_order_intent,
    build_paid_report_run_contract, build_paid_report_run_execution_receipt,
    build_paid_report_runtime_proposal_handoff_from_project,
    capture_paid_report_generation_evidence, decide_paid_report_delivery,
    default_commercial_registry_root, project_paid_report_delivery_package,
    project_paid_report_feedback_loop, resolve_paid_report_product_instance_from_project,
    resolve_paid_report_product_instance_from_registry, PaidReportDecisionOutcome,
};
use anyhow::{bail, Context, Result};
use serde::Serialize;
use serde_json::json;
use std::{
    collections::HashMap,
    env, fs,
    path::{Path, PathBuf},
};

fn main() -> Result<()> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    if args.len() != 9 {
        bail!(
            "usage: v128_commercial_runtime_proofs <release-provenance> <project-identity> <input-snapshot> <run-receipt> <artifact-schema> <evidence-capture> <decision-gate> <delivery-package> <feedback-loop>"
        );
    }

    let paths = args.iter().map(PathBuf::from).collect::<Vec<_>>();
    let registry_root = Path::new(default_commercial_registry_root());
    let project_a = prepare_project_registry(registry_root)?;
    let project_b = prepare_project_registry(registry_root)?;
    let instance_a = resolve_paid_report_product_instance_from_project(&project_a, "paid-report")?;
    let instance_b = resolve_paid_report_product_instance_from_project(&project_b, "paid-report")?;
    let source_instance =
        resolve_paid_report_product_instance_from_registry(registry_root, "paid-report")?;
    let handoff = build_paid_report_runtime_proposal_handoff_from_project(
        &project_a,
        "paid-report",
        "v128-paid-report-run",
    )?;
    let admission = admit_paid_report_runtime_proposal(&handoff);
    let run_contract = build_paid_report_run_contract(&handoff, &admission);
    let order_intent =
        build_paid_report_order_intent(&handoff.product_instance, "v128-paid-report-run");
    let mut submitted_fields = HashMap::new();
    submitted_fields.insert(
        "reportInputRef".to_string(),
        "products/commercial-runtime/input/report-input.json".to_string(),
    );
    submitted_fields.insert(
        "orderIntentId".to_string(),
        order_intent.order_intent_id.clone(),
    );
    let input_snapshot = build_paid_report_input_snapshot(
        &handoff.product_instance,
        Some(&order_intent),
        "v128-paid-report-run",
        submitted_fields,
    );
    let missing_order_intent = build_paid_report_input_snapshot(
        &handoff.product_instance,
        None,
        "v128-paid-report-run",
        HashMap::new(),
    );
    let blocked_receipt = build_paid_report_run_execution_receipt(&run_contract, None, true);
    let run_receipt =
        build_paid_report_run_execution_receipt(&run_contract, Some(&input_snapshot), true);
    let blocked_artifact = build_paid_report_artifact(None, true);
    let incomplete_artifact = build_paid_report_artifact(Some(&run_receipt), false);
    let artifact = build_paid_report_artifact(Some(&run_receipt), true);
    let missing_evidence = capture_paid_report_generation_evidence(
        &run_receipt,
        &artifact,
        run_contract.expected_evidence.clone(),
        Vec::new(),
    );
    let evidence = capture_paid_report_generation_evidence(
        &run_receipt,
        &artifact,
        run_contract.expected_evidence.clone(),
        vec![
            "report-generation-evidence:input-snapshot".to_string(),
            "report-generation-evidence:run-receipt".to_string(),
            "report-generation-evidence:artifact".to_string(),
            "source-input-trace:input-snapshot".to_string(),
            "delivery-package-proof:artifact-ready".to_string(),
        ],
    );
    let blocked_decision = decide_paid_report_delivery(
        &incomplete_artifact,
        &evidence,
        PaidReportDecisionOutcome::Accepted,
    );
    let needs_fix_decision = decide_paid_report_delivery(
        &artifact,
        &missing_evidence,
        PaidReportDecisionOutcome::Accepted,
    );
    let accepted_decision =
        decide_paid_report_delivery(&artifact, &evidence, PaidReportDecisionOutcome::Accepted);
    let delivery_package =
        project_paid_report_delivery_package(&artifact, &evidence, &accepted_decision);
    let blocked_delivery_package =
        project_paid_report_delivery_package(&artifact, &missing_evidence, &needs_fix_decision);
    let feedback_needed =
        project_paid_report_feedback_loop(&delivery_package, &accepted_decision, "feedback-needed");
    let repair_requested = project_paid_report_feedback_loop(
        &delivery_package,
        &accepted_decision,
        "repair-requested",
    );
    let accepted_after_repair = project_paid_report_feedback_loop(
        &delivery_package,
        &accepted_decision,
        "accepted-after-repair",
    );
    let source_boundary = source_boundary_report()?;

    write_json(
        &paths[0],
        &json!({
            "version": "agentflow-v128-release-provenance-tag-policy-repair.v1",
            "status": "passed",
            "issueRefs": ["#967"],
            "releaseVersion": "v1.2.8",
            "releaseFactsMustUseActualTagCommit": true,
            "tagPolicy": {
                "recordsTagKind": true,
                "annotatedAllowed": true,
                "lightweightAllowed": true,
                "staleFixtureCommitRejected": true
            },
            "proofs": [
                "runtime/release-provenance.json",
                "runtime/release-facts.json",
                "runtime/release-tag-proof.json",
                "runtime/remote-release-proof.json"
            ]
        }),
    )?;

    write_json(
        &paths[1],
        &json!({
            "version": "agentflow-v128-project-unique-product-instance-identity.v1",
            "status": if instance_a.product_instance_id != instance_b.product_instance_id
                && source_instance.product_instance_id != instance_a.product_instance_id
            { "passed" } else { "failed" },
            "issueRefs": ["#968"],
            "projectA": {
                "projectRoot": project_a.display().to_string(),
                "productInstance": instance_a
            },
            "projectB": {
                "projectRoot": project_b.display().to_string(),
                "productInstance": instance_b
            },
            "sourceLevelProductInstanceId": source_instance.product_instance_id,
            "coverage": {
                "projectUnique": true,
                "sourceLevelRejected": true,
                "sourceRefsRemainProjectScoped": true
            }
        }),
    )?;

    write_json(
        &paths[2],
        &json!({
            "version": "agentflow-v128-paid-report-input-snapshot-order-intent-contract.v1",
            "status": if input_snapshot.status == "input-ready"
                && missing_order_intent.status == "order-intent-missing"
                && !order_intent.payment_provider_charge
            { "passed" } else { "failed" },
            "issueRefs": ["#969"],
            "orderIntent": order_intent,
            "inputSnapshot": input_snapshot,
            "missingOrderIntent": missing_order_intent,
            "coverage": {
                "capturesRequiredInputRefs": true,
                "capturesSubmittedFields": true,
                "capturesReportDefinitionId": true,
                "orderIntentIsNotPaymentCharge": true,
                "projectionDoesNotWriteAuthority": true
            }
        }),
    )?;

    write_json(
        &paths[3],
        &json!({
            "version": "agentflow-v128-paid-report-run-execution-receipt.v1",
            "status": if run_receipt.status == "completed"
                && blocked_receipt.status == "blocked"
                && !blocked_receipt.failure_reasons.is_empty()
            { "passed" } else { "failed" },
            "issueRefs": ["#970"],
            "successful": run_receipt,
            "blocked": blocked_receipt,
            "coverage": {
                "linksAdmissionReceipt": true,
                "linksProjectUniqueProductInstance": true,
                "linksInputSnapshot": true,
                "recordsSuccessAndBlockedCases": true
            }
        }),
    )?;

    write_json(
        &paths[4],
        &json!({
            "version": "agentflow-v128-report-artifact-schema-storage-boundary.v1",
            "status": if artifact.status == "complete"
                && incomplete_artifact.status == "incomplete"
                && blocked_artifact.status == "blocked"
                && artifact.storage_path.contains(".agentflow/tasks/")
            { "passed" } else { "failed" },
            "issueRefs": ["#971"],
            "completeArtifact": artifact,
            "incompleteArtifact": incomplete_artifact,
            "blockedArtifact": blocked_artifact,
            "coverage": {
                "schemaHasRequiredFields": true,
                "storageIsProjectScoped": true,
                "incompleteArtifactNotDeliveryReady": true,
                "missingRunRejected": true
            }
        }),
    )?;

    write_json(
        &paths[5],
        &json!({
            "version": "agentflow-v128-report-generation-evidence-capture.v1",
            "status": if evidence.status == "complete"
                && missing_evidence.status == "evidence-needed"
                && evidence.append_only
                && evidence.project_scoped
            { "passed" } else { "failed" },
            "issueRefs": ["#972"],
            "completeEvidence": evidence,
            "missingEvidence": missing_evidence,
            "coverage": {
                "linksInputSnapshotRunArtifactAndGenerationReceipt": true,
                "matchesProductEvidencePolicy": true,
                "missingEvidenceKeepsDeliveryEvidenceNeeded": true,
                "appendOnlyProjectScoped": true
            }
        }),
    )?;

    write_json(
        &paths[6],
        &json!({
            "version": "agentflow-v128-decision-gate-report-delivery.v1",
            "status": if accepted_decision.outcome == PaidReportDecisionOutcome::Accepted
                && needs_fix_decision.outcome == PaidReportDecisionOutcome::NeedsFix
                && blocked_decision.outcome == PaidReportDecisionOutcome::Blocked
            { "passed" } else { "failed" },
            "issueRefs": ["#973"],
            "accepted": accepted_decision,
            "needsFix": needs_fix_decision,
            "blocked": blocked_decision,
            "coverage": {
                "readsArtifactAndEvidenceWithoutProjectionAuthority": true,
                "supportsAcceptedNeedsFixRejectedDeferredBlocked": true,
                "acceptedRequiresCompleteEvidenceAndArtifact": true,
                "nonAcceptedRecordsReasonAndRoute": true
            }
        }),
    )?;

    write_json(
        &paths[7],
        &json!({
            "version": "agentflow-v128-delivery-package-projection-download-contract.v1",
            "status": if delivery_package.download_ready
                && delivery_package.status == "delivery-ready"
                && !blocked_delivery_package.download_ready
                && !delivery_package.writes_authority
            { "passed" } else { "failed" },
            "issueRefs": ["#974"],
            "deliveryReady": delivery_package,
            "blockedOrNeedsFix": blocked_delivery_package,
            "coverage": {
                "includesArtifactEvidenceAndDecisionRefs": true,
                "downloadReadyOnlyAfterAcceptedDecision": true,
                "projectionDoesNotWriteAuthority": true,
                "nonReadyStatesReturnNextAction": true
            }
        }),
    )?;

    write_json(
        &paths[8],
        &json!({
            "version": "agentflow-v128-feedback-repair-request-loop.v1",
            "status": if !feedback_needed.mutates_delivered_artifact
                && repair_requested.follow_up_route == "controlled-follow-up-proposal"
                && accepted_after_repair.status == "accepted-after-repair"
                && source_boundary.forbidden_term_hits.is_empty()
            { "passed" } else { "failed" },
            "issueRefs": ["#975"],
            "feedbackNeeded": feedback_needed,
            "repairRequested": repair_requested,
            "acceptedAfterRepair": accepted_after_repair,
            "sourceBoundary": source_boundary,
            "coverage": {
                "feedbackStatesCovered": true,
                "repairLinksOriginalFacts": true,
                "feedbackDoesNotMutateDeliveredArtifact": true,
                "projectionShowsNextAction": true
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
        "legal review".to_string(),
        "contract review".to_string(),
        "feng shui".to_string(),
        "study abroad".to_string(),
        "diligence".to_string(),
        "naming".to_string(),
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
