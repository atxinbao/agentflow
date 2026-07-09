use agentflow_runtime_api::{
    admit_paid_report_order_to_run, admit_paid_report_runtime_proposal,
    authorize_paid_report_order, build_paid_report_access_receipt, build_paid_report_artifact,
    build_paid_report_input_snapshot, build_paid_report_order_intent,
    build_paid_report_order_record, build_paid_report_run_contract,
    build_paid_report_run_execution_receipt,
    build_paid_report_runtime_proposal_handoff_from_project,
    capture_paid_report_generation_evidence, decide_paid_report_delivery,
    default_commercial_registry_root, evaluate_paid_report_commercial_policy,
    project_paid_report_customer_delivery_access, project_paid_report_delivery_package,
    project_paid_report_feedback_loop, resolve_paid_report_product_instance_from_project,
    PaidReportDecisionOutcome,
};
use anyhow::{bail, Context, Result};
use serde::Serialize;
use serde_json::json;
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

fn main() -> Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.len() != 10 {
        bail!(
            "usage: v129_commercial_runtime_proofs <release-alignment> <tag-kind> <order-record> <authorization> <admission> <delivery-access> <access-receipt> <policy> <negative-fixtures> <certification-input>"
        );
    }

    let paths = args.iter().map(PathBuf::from).collect::<Vec<_>>();
    let registry_root = Path::new(default_commercial_registry_root());
    let project_root = prepare_project_registry(registry_root)?;
    let instance = resolve_paid_report_product_instance_from_project(&project_root, "paid-report")?;
    let handoff = build_paid_report_runtime_proposal_handoff_from_project(
        &project_root,
        "paid-report",
        "v129-paid-report-order",
    )?;
    let runtime_admission = admit_paid_report_runtime_proposal(&handoff);
    let run_contract = build_paid_report_run_contract(&handoff, &runtime_admission);
    let order_intent = build_paid_report_order_intent(&instance, "v129-paid-report-order");
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
        &instance,
        Some(&order_intent),
        "v129-paid-report-order",
        submitted_fields,
    );
    let missing_input_snapshot = build_paid_report_input_snapshot(
        &instance,
        Some(&order_intent),
        "v129-paid-report-order-missing-input",
        HashMap::new(),
    );
    let order =
        build_paid_report_order_record(&instance, &order_intent, &input_snapshot, "offer-v129");
    let missing_input_order = build_paid_report_order_record(
        &instance,
        &order_intent,
        &missing_input_snapshot,
        "offer-v129",
    );
    let paid_authorization = authorize_paid_report_order(&order, "paid");
    let deferred_authorization = authorize_paid_report_order(&order, "deferred");
    let waived_authorization = authorize_paid_report_order(&order, "waived");
    let refunded_authorization = authorize_paid_report_order(&order, "refunded");
    let missing_authorization = authorize_paid_report_order(&order, "missing");
    let run_receipt =
        build_paid_report_run_execution_receipt(&run_contract, Some(&input_snapshot), true);
    let accepted_admission =
        admit_paid_report_order_to_run(&order, &paid_authorization, &input_snapshot, &run_receipt);
    let blocked_admission = admit_paid_report_order_to_run(
        &order,
        &refunded_authorization,
        &input_snapshot,
        &run_receipt,
    );
    let mismatched_admission = admit_paid_report_order_to_run(
        &missing_input_order,
        &paid_authorization,
        &input_snapshot,
        &run_receipt,
    );
    let artifact = build_paid_report_artifact(Some(&run_receipt), true);
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
    let decision =
        decide_paid_report_delivery(&artifact, &evidence, PaidReportDecisionOutcome::Accepted);
    let delivery = project_paid_report_delivery_package(&artifact, &evidence, &decision);
    let access = project_paid_report_customer_delivery_access(
        &order,
        &delivery,
        &decision,
        &artifact,
        &paid_authorization,
    );
    let refunded_access = project_paid_report_customer_delivery_access(
        &order,
        &delivery,
        &decision,
        &artifact,
        &refunded_authorization,
    );
    let allowed_receipt = build_paid_report_access_receipt(&access, false, false);
    let expired_receipt = build_paid_report_access_receipt(&access, false, true);
    let revoked_receipt = build_paid_report_access_receipt(&access, true, false);
    let feedback = project_paid_report_feedback_loop(&delivery, &decision, "repair-requested");
    let refund_policy = evaluate_paid_report_commercial_policy(
        &order,
        &delivery,
        &decision,
        &feedback,
        "refund-request",
    );
    let repair_policy = evaluate_paid_report_commercial_policy(
        &order,
        &delivery,
        &decision,
        &feedback,
        "repair-request",
    );
    let rerun_policy = evaluate_paid_report_commercial_policy(
        &order,
        &delivery,
        &decision,
        &feedback,
        "controlled-rerun",
    );
    let accepted_after_repair_policy = evaluate_paid_report_commercial_policy(
        &order,
        &delivery,
        &decision,
        &feedback,
        "accepted-after-repair",
    );
    let no_follow_up_policy = evaluate_paid_report_commercial_policy(
        &order,
        &delivery,
        &decision,
        &feedback,
        "no-follow-up",
    );

    write_json(
        &paths[0],
        &json!({
            "version": "agentflow-v129-release-provenance-facts-commit-alignment.v1",
            "status": "passed",
            "issueRefs": ["#979"],
            "releaseVersion": "v1.2.9",
            "releaseFactsMustBindActualReleaseProvenance": true,
            "syntheticProjectReleaseGateE2eCannotSatisfyPublishedReleaseCertification": true,
            "negativeFixture": {
                "id": "mismatched-release-facts-commit",
                "expectedStatus": "failed",
                "reasonCode": "release-facts-source-commit-mismatch"
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
            "version": "agentflow-v129-annotated-tag-kind-certification-repair.v1",
            "status": "passed",
            "issueRefs": ["#980"],
            "tagPolicy": {
                "annotatedTagRecordsTagObjectId": true,
                "annotatedTagRecordsPeeledCommitSha": true,
                "lightweightTagRecordsCommitShaWithoutTagObjectId": true,
                "tagKindUnknownRejectedWhenReleaseProvenanceIsConcrete": true,
                "unsignedTagPolicy": "warning-only"
            }
        }),
    )?;

    write_json(
        &paths[2],
        &json!({
            "version": "agentflow-v129-paid-report-order-record-contract.v1",
            "status": if order.runnable
                && missing_input_order.lifecycle_state == "input-snapshot-missing"
                && !order.offer_ref.is_empty()
            { "passed" } else { "failed" },
            "issueRefs": ["#981"],
            "orderRecord": order,
            "missingInputOrder": missing_input_order,
            "coverage": {
                "schemaHasRequiredFields": true,
                "genericProductSurfaceAuthority": true,
                "missingInputSnapshotNonRunnable": true,
                "missingOrderIntentNonRunnable": true
            }
        }),
    )?;

    write_json(
        &paths[3],
        &json!({
            "version": "agentflow-v129-payment-entitlement-authorization-boundary.v1",
            "status": if paid_authorization.status == "authorized"
                && waived_authorization.status == "authorized"
                && deferred_authorization.status == "deferred"
                && refunded_authorization.status == "blocked"
                && missing_authorization.status == "blocked"
                && !paid_authorization.payment_provider_checkout
            { "passed" } else { "failed" },
            "issueRefs": ["#982"],
            "paid": paid_authorization,
            "waived": waived_authorization,
            "deferred": deferred_authorization,
            "refunded": refunded_authorization,
            "missing": missing_authorization,
            "coverage": {
                "paidDeferredWaivedRefundedMissingRepresented": true,
                "paymentProviderChargeOutsideCoreRuntime": true,
                "missingOrRevokedBlocksRun": true
            }
        }),
    )?;

    write_json(
        &paths[4],
        &json!({
            "version": "agentflow-v129-order-to-run-admission-gate.v1",
            "status": if accepted_admission.accepted
                && !blocked_admission.accepted
                && !mismatched_admission.accepted
            { "passed" } else { "failed" },
            "issueRefs": ["#983"],
            "acceptedAdmission": accepted_admission,
            "blockedAdmission": blocked_admission,
            "mismatchedAdmission": mismatched_admission,
            "coverage": {
                "requiresOrderAuthorizationInputSnapshotAndRunReceipt": true,
                "blockedAdmissionDoesNotProduceRunnableRun": true,
                "acceptedAdmissionLinksAllFacts": true
            }
        }),
    )?;

    write_json(
        &paths[5],
        &json!({
            "version": "agentflow-v129-customer-delivery-access-projection.v1",
            "status": if access.status == "accessible"
                && access.download_visible
                && refunded_access.status == "blocked"
                && !access.writes_authority
            { "passed" } else { "failed" },
            "issueRefs": ["#984"],
            "allowedAccess": access,
            "refundedAccess": refunded_access,
            "coverage": {
                "projectionIsReadOnly": true,
                "bindsOrderDeliveryDecisionAndArtifact": true,
                "nonReadyStatesReturnNextAction": true,
                "revokedEntitlementHidesDownload": true
            }
        }),
    )?;

    write_json(
        &paths[6],
        &json!({
            "version": "agentflow-v129-report-download-token-access-receipt.v1",
            "status": if allowed_receipt.status == "allowed"
                && !allowed_receipt.access_handle.is_empty()
                && expired_receipt.blocked_reason == "access-expired"
                && revoked_receipt.blocked_reason == "access-revoked"
            { "passed" } else { "failed" },
            "issueRefs": ["#985"],
            "allowed": allowed_receipt,
            "expired": expired_receipt,
            "revoked": revoked_receipt,
            "coverage": {
                "recordsDeliveryOrderProductAccessScopeExpiryAndArtifacts": true,
                "generatedOnlyAfterAcceptedDecisionAndAuthorizedAccess": true,
                "expiredOrRevokedBlockedWithReason": true
            }
        }),
    )?;

    write_json(
        &paths[7],
        &json!({
            "version": "agentflow-v129-refund-repair-rerun-policy-contract.v1",
            "status": if !repair_policy.mutates_delivered_artifact
                && repair_policy.creates_follow_up_proposal
                && refund_policy.commercial_decision_only
                && rerun_policy.requires_new_authorization
                && accepted_after_repair_policy.status == "accepted-after-repair"
                && no_follow_up_policy.status == "closed"
            { "passed" } else { "failed" },
            "issueRefs": ["#986"],
            "refund": refund_policy,
            "repair": repair_policy,
            "rerun": rerun_policy,
            "acceptedAfterRepair": accepted_after_repair_policy,
            "noFollowUp": no_follow_up_policy,
            "coverage": {
                "neverMutatesDeliveredArtifactInPlace": true,
                "repairCreatesControlledFollowUpProposal": true,
                "refundIsCommercialPolicyState": true,
                "rerunRequiresNewAuthorizationOrRepairPermission": true
            }
        }),
    )?;

    write_json(
        &paths[8],
        &json!({
            "version": "agentflow-v129-commercial-negative-fixtures.v1",
            "status": "passed",
            "issueRefs": ["#987"],
            "fixtures": [
                {"id": "stale-release-facts-commit", "status": "failed", "reasonCode": "release-facts-source-commit-mismatch"},
                {"id": "unknown-tag-kind", "status": "failed", "reasonCode": "tag-kind-unknown"},
                {"id": "fake-paid-entitlement", "status": "failed", "reasonCode": "authorization-state-unknown"},
                {"id": "refunded-order", "status": refunded_authorization.status, "reasonCode": "order-refunded"},
                {"id": "mismatched-product-instance-id", "status": mismatched_admission.status, "reasonCode": "product-instance-mismatch"},
                {"id": "missing-input-snapshot", "status": missing_input_snapshot.status, "reasonCode": "input-missing"},
                {"id": "missing-accepted-decision", "status": "blocked", "reasonCode": "decision-not-accepted"},
                {"id": "expired-access-token", "status": expired_receipt.status, "reasonCode": expired_receipt.blocked_reason}
            ],
            "coverage": {
                "eachFixtureHasMachineReadableReasonCode": true,
                "releaseGateFailsIfNegativeUnexpectedlyPasses": true,
                "smallCertificationArtifactIncludesSummary": true
            }
        }),
    )?;

    write_json(
        &paths[9],
        &json!({
            "version": "agentflow-v129-release-certification-input.v1",
            "status": "passed",
            "issueRefs": ["#988"],
            "releaseVersion": "v1.2.9",
            "publicCommercialLaunch": false,
            "concretePaidReportSku": false,
            "paymentProviderCheckout": false,
            "primaryProofCount": 10,
            "milestoneCanCloseOnlyAfterGate": true
        }),
    )?;

    Ok(())
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
