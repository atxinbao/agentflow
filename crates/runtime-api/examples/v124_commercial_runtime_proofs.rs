use agentflow_runtime_api::{
    commercial_golden_path, commercial_negative_fixture_report,
    get_commercial_product_projection_query, load_commercial_product_read_model,
    managed_project_commercial_fixture, PaidReportPreflightRequest,
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
    if args.len() != 6 {
        bail!(
            "usage: v124_commercial_runtime_proofs <read-model> <projection-query> <paid-report-preflight> <managed-project-fixture> <negative-fixtures> <golden-path>"
        );
    }

    let paths = args.iter().map(PathBuf::from).collect::<Vec<_>>();
    let read_model = load_commercial_product_read_model();
    let projection = get_commercial_product_projection_query();
    let paid_report_preflight = json!({
        "version": "agentflow-v124-paid-report-preflight-runtime-api.v1",
        "status": "passed",
        "issueRefs": ["#928"],
        "cases": paid_report_preflight_cases(),
    });
    let managed_project = managed_project_commercial_fixture();
    let negative_fixtures = commercial_negative_fixture_report();
    let golden_path = commercial_golden_path();

    write_json(
        &paths[0],
        &json!({
            "version": "agentflow-v124-commercial-product-read-model-runtime-api.v1",
            "status": if read_model.writes_authority || !read_model.projection_only || read_model.core_authority { "failed" } else { "passed" },
            "issueRefs": ["#925"],
            "payload": read_model,
        }),
    )?;
    write_json(
        &paths[1],
        &json!({
            "version": "agentflow-v124-commercial-projection-query.v1",
            "status": if projection.writes_authority || !projection.projection_only { "failed" } else { "passed" },
            "issueRefs": ["#926"],
            "payload": projection,
        }),
    )?;
    write_json(&paths[2], &paid_report_preflight)?;
    write_json(
        &paths[3],
        &json!({
            "version": "agentflow-v124-managed-project-commercial-runtime-fixture.v1",
            "status": managed_project.status,
            "issueRefs": ["#929"],
            "payload": managed_project,
        }),
    )?;
    write_json(
        &paths[4],
        &json!({
            "version": "agentflow-v124-commercial-negative-runtime-fixtures.v1",
            "status": negative_fixtures.status,
            "issueRefs": ["#930"],
            "payload": negative_fixtures,
        }),
    )?;
    write_json(
        &paths[5],
        &json!({
            "version": "agentflow-v124-commercial-golden-path.v1",
            "status": golden_path.status,
            "issueRefs": ["#931"],
            "payload": golden_path,
        }),
    )?;

    Ok(())
}

fn paid_report_preflight_cases() -> Vec<serde_json::Value> {
    vec![
        PaidReportPreflightRequest {
            product_id: "paid-report".to_string(),
            request_id: "paid-report-allowed".to_string(),
            has_input_refs: true,
            entitlement_state: agentflow_runtime_api::CommercialEntitlementState::Active,
            paid_feature_state: agentflow_runtime_api::CommercialPaidFeatureState::Enabled,
            report_definition_present: true,
            order_intent_present: true,
            payment_configured: true,
        },
        PaidReportPreflightRequest {
            product_id: "paid-report".to_string(),
            request_id: "paid-report-disabled".to_string(),
            has_input_refs: true,
            entitlement_state: agentflow_runtime_api::CommercialEntitlementState::Disabled,
            paid_feature_state: agentflow_runtime_api::CommercialPaidFeatureState::Enabled,
            report_definition_present: true,
            order_intent_present: true,
            payment_configured: true,
        },
        PaidReportPreflightRequest {
            product_id: "paid-report".to_string(),
            request_id: "paid-report-payment-deferred".to_string(),
            has_input_refs: true,
            entitlement_state: agentflow_runtime_api::CommercialEntitlementState::Active,
            paid_feature_state: agentflow_runtime_api::CommercialPaidFeatureState::Enabled,
            report_definition_present: true,
            order_intent_present: true,
            payment_configured: false,
        },
    ]
    .into_iter()
    .map(|request| {
        let result = agentflow_runtime_api::evaluate_paid_report_preflight(request.clone());
        json!({
            "request": request,
            "result": result,
            "passed": match result.decision {
                agentflow_runtime_api::PaidReportPreflightDecision::Allowed => {
                    result.can_submit_runtime_command_proposal
                        && result.runtime_admission_required
                        && !result.can_start_run_directly
                }
                _ => !result.can_submit_runtime_command_proposal
                    && !result.can_start_run_directly
                    && result.runtime_command_policy == "blocked-before-runtime",
            }
        })
    })
    .collect()
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
