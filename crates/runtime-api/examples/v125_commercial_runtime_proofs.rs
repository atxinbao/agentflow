use agentflow_runtime_api::{
    commercial_golden_path, commercial_negative_fixture_report, default_commercial_registry_root,
    evaluate_paid_report_preflight_from_registry, get_commercial_product_projection_query,
    load_registry_commercial_product_read_model, managed_project_commercial_fixture,
    negative_commercial_fixture_root, production_registry_has_fixture_only_products,
    CommercialAvailability, PaidReportPreflightDecision,
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
    if args.len() != 5 {
        bail!(
            "usage: v125_commercial_runtime_proofs <registry-read-model> <entitlement-source> <paid-report-definition> <desktop-runtime-guard> <golden-path>"
        );
    }

    let paths = args.iter().map(PathBuf::from).collect::<Vec<_>>();
    let registry_root_buf = PathBuf::from(default_commercial_registry_root());
    let registry_root = registry_root_buf.as_path();
    let negative_registry_root_buf = PathBuf::from(negative_commercial_fixture_root());
    let negative_registry_root = negative_registry_root_buf.as_path();
    let read_model = load_registry_commercial_product_read_model(registry_root)
        .context("load product registry-backed commercial read model")?;
    let paid_report_allowed =
        evaluate_paid_report_preflight_from_registry(registry_root, "paid-report", "ready")
            .context("evaluate paid report ready fixture")?;
    let paid_report_deferred = evaluate_paid_report_preflight_from_registry(
        registry_root,
        "paid-report-preview",
        "payment-deferred",
    )
    .context("evaluate paid report deferred fixture")?;
    let missing_report = evaluate_paid_report_preflight_from_registry(
        negative_registry_root,
        "paid-report-missing-report",
        "missing-report",
    )
    .context("evaluate missing report fixture")?;
    let missing_input = evaluate_paid_report_preflight_from_registry(
        negative_registry_root,
        "paid-report-missing-input",
        "missing-input",
    )
    .context("evaluate missing input fixture")?;
    let negative_read_model = load_registry_commercial_product_read_model(negative_registry_root)
        .context("load negative fixture commercial read model")?;
    let production_has_fixture_only_products =
        production_registry_has_fixture_only_products(registry_root)
            .context("scan production registry fixture-only ids")?;
    let projection = get_commercial_product_projection_query();
    let managed_project = managed_project_commercial_fixture();
    let negative = commercial_negative_fixture_report();
    let golden_path = commercial_golden_path();

    let paid_report_entry = read_model
        .entries
        .iter()
        .find(|entry| entry.product_id == "paid-report")
        .context("paid-report registry entry missing")?;
    let missing_report_entry = negative_read_model
        .entries
        .iter()
        .find(|entry| entry.product_id == "paid-report-missing-report")
        .context("paid-report-missing-report negative fixture entry missing")?;
    let missing_input_entry = negative_read_model
        .entries
        .iter()
        .find(|entry| entry.product_id == "paid-report-missing-input")
        .context("paid-report-missing-input negative fixture entry missing")?;

    write_json(
        &paths[0],
        &json!({
            "version": "agentflow-v125-product-registry-commercial-read-model.v1",
            "status": if read_model.source == "product-registry-config"
                && read_model.projection_only
                && !read_model.writes_authority
                && paid_report_entry.availability == CommercialAvailability::Available
                && !production_has_fixture_only_products
                && missing_report_entry.availability == CommercialAvailability::Invalid
                && missing_input_entry.availability == CommercialAvailability::Invalid
            { "passed" } else { "failed" },
            "issueRefs": ["#937"],
            "source": read_model.source,
            "sourceRefs": read_model.source_refs,
            "payload": read_model,
            "coverage": {
                "sourceIsProductRegistryConfig": true,
                "defaultInputsNotProductionSource": true,
                "writesAuthority": false,
                "productionRegistryExcludesFixtureOnlyProducts": !production_has_fixture_only_products,
                "negativeFixtureRegistrySource": negative_commercial_fixture_root(),
                "missingReportDefinitionRejected": missing_report_entry.unavailable_reason == "missing-report-definition",
                "missingRequiredInputsRejected": missing_input_entry.unavailable_reason == "missing-required-inputs"
            }
        }),
    )?;
    write_json(
        &paths[1],
        &json!({
            "version": "agentflow-v125-entitlement-source-fixture.v1",
            "status": if paid_report_allowed.decision == PaidReportPreflightDecision::Allowed
                && paid_report_deferred.decision == PaidReportPreflightDecision::Deferred
                && missing_report.decision == PaidReportPreflightDecision::Invalid
            { "passed" } else { "failed" },
            "issueRefs": ["#938"],
            "source": "products/commercial-runtime/entitlements.json",
            "cases": [
                preflight_case("active-paid-report", &paid_report_allowed),
                preflight_case("deferred-paid-report-preview", &paid_report_deferred),
                preflight_case("missing-report-definition", &missing_report),
                preflight_case("missing-required-inputs", &missing_input)
            ]
        }),
    )?;
    write_json(
        &paths[2],
        &json!({
            "version": "agentflow-v125-paid-report-product-definition.v1",
            "status": if paid_report_allowed.evidence_requirements.contains(&"report-generation-evidence".to_string())
                && paid_report_allowed.evidence_requirements.contains(&"delivery-package-proof".to_string())
                && paid_report_allowed.decision_requirements.contains(&"report-delivery-decision".to_string())
                && missing_report.decision == PaidReportPreflightDecision::Invalid
            { "passed" } else { "failed" },
            "issueRefs": ["#939"],
            "requiredInputRefs": ["reportInputRef", "orderIntentId"],
            "evidenceRequirements": paid_report_allowed.evidence_requirements,
            "decisionRequirements": paid_report_allowed.decision_requirements,
            "positiveCase": preflight_case("paid-report-ready", &paid_report_allowed),
            "negativeCase": preflight_case("paid-report-missing-report", &missing_report)
        }),
    )?;
    write_json(
        &paths[3],
        &json!({
            "version": "agentflow-v125-desktop-runtime-only-commercial-surface.v1",
            "status": if projection.writes_authority || !projection.projection_only { "failed" } else { "passed" },
            "issueRefs": ["#940"],
            "desktopSurface": {
                "productionSource": "runtime-tauri-read-model",
                "browserPreviewFallbackMustBeMarked": true,
                "runtimeMissingIsNonReady": true,
                "writesAuthority": false
            },
            "payload": projection
        }),
    )?;
    write_json(
        &paths[4],
        &json!({
            "version": "agentflow-v125-commercial-golden-path-registry.v1",
            "status": if golden_path.status == "passed"
                && managed_project.status == "passed"
                && negative.status == "passed"
                && paid_report_allowed.decision == PaidReportPreflightDecision::Allowed
                && paid_report_deferred.decision == PaidReportPreflightDecision::Deferred
                && missing_report.decision == PaidReportPreflightDecision::Invalid
            { "passed" } else { "failed" },
            "issueRefs": ["#941"],
            "readModelSource": "product-registry-config",
            "managedProject": managed_project,
            "negativeFixtures": negative,
            "goldenPath": golden_path,
            "paidReportReady": preflight_case("paid-report-ready", &paid_report_allowed),
            "paidReportDeferred": preflight_case("paid-report-deferred", &paid_report_deferred),
            "paidReportMissingReport": preflight_case("paid-report-missing-report", &missing_report)
        }),
    )?;

    Ok(())
}

fn preflight_case(id: &str, result: &agentflow_runtime_api::PaidReportPreflightResult) -> Value {
    json!({
        "id": id,
        "decision": result.decision,
        "canSubmitRuntimeCommandProposal": result.can_submit_runtime_command_proposal,
        "canStartRunDirectly": result.can_start_run_directly,
        "runtimeCommandPolicy": result.runtime_command_policy,
        "unavailableReason": result.unavailable_reason,
        "evidenceRequirements": result.evidence_requirements,
        "decisionRequirements": result.decision_requirements,
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
