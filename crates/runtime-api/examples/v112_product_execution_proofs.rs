use agentflow_action_contract::ActionSourceSurface;
use agentflow_capability_registry::{default_capability_registry, CapabilityPolicy, WorkerHealth};
use agentflow_runtime_api::{
    dry_run_pack_command, list_product_command_surface, query_pack_surface_route,
    validate_pack_command, PackCommandRequest,
};
use anyhow::{bail, Context, Result};
use serde_json::{json, Value};
use std::{
    env, fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

fn main() -> Result<()> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    if args.len() != 6 {
        bail!(
            "usage: v112_product_execution_proofs <workspace> <runtime-proof> <projection-proof> <registry-proof> <desktop-proof> <multi-product-proof>"
        );
    }
    let workspace = PathBuf::from(&args[0]);
    let runtime_out = PathBuf::from(&args[1]);
    let projection_out = PathBuf::from(&args[2]);
    let registry_out = PathBuf::from(&args[3]);
    let desktop_out = PathBuf::from(&args[4]);
    let multi_product_out = PathBuf::from(&args[5]);

    write_ready_capability_registry(&workspace)?;

    let runtime_payload = runtime_proof(&workspace)?;
    write_json(&runtime_out, &runtime_payload)?;

    let projection_payload = projection_proof(&workspace)?;
    write_json(&projection_out, &projection_payload)?;

    let registry_payload = registry_proof(&workspace)?;
    write_json(&registry_out, &registry_payload)?;

    let desktop_payload = desktop_surface_proof(&workspace)?;
    write_json(&desktop_out, &desktop_payload)?;

    let multi_product_payload = multi_product_state_proof(&workspace, &desktop_payload)?;
    write_json(&multi_product_out, &multi_product_payload)?;

    Ok(())
}

fn runtime_proof(workspace: &Path) -> Result<Value> {
    let positive_route = query_pack_surface_route(workspace, "software-dev", "work.issue.start")?;
    let positive_request = PackCommandRequest {
        pack_id: "software-dev".to_string(),
        command_id: "v112-positive-runtime-001".to_string(),
        command: "work.issue.start".to_string(),
        actor_role: "work-agent".to_string(),
        source_surface: ActionSourceSurface::Desktop,
        target_object_type: positive_route.target_object_type.clone(),
        target_object_id: "AF-V112-001".to_string(),
        input: json!({"reason": "v1.1.2 real product runtime proof"}),
        evidence_refs: Vec::new(),
        artifact_refs: Vec::new(),
        idempotency_key: "v112-positive-runtime-001".to_string(),
        created_at: "2026-07-02T00:00:00Z".to_string(),
    };
    let positive_validation = validate_pack_command(workspace, &positive_request)?;
    let positive_dry_run = dry_run_pack_command(workspace, &positive_request)?;

    let negative_request = PackCommandRequest {
        pack_id: "software-dev".to_string(),
        command_id: "v112-negative-runtime-001".to_string(),
        command: "work.issue.teleport".to_string(),
        actor_role: "work-agent".to_string(),
        source_surface: ActionSourceSurface::Desktop,
        target_object_type: "Issue".to_string(),
        target_object_id: "AF-V112-NEGATIVE".to_string(),
        input: json!({"reason": "unsupported product command must be rejected"}),
        evidence_refs: Vec::new(),
        artifact_refs: Vec::new(),
        idempotency_key: "v112-negative-runtime-001".to_string(),
        created_at: "2026-07-02T00:00:00Z".to_string(),
    };
    let negative_validation = validate_pack_command(workspace, &negative_request)?;
    let negative_dry_run = dry_run_pack_command(workspace, &negative_request)?;

    let checks = json!({
        "positive-validation-valid": positive_validation.valid,
        "positive-dry-run-valid": positive_dry_run.valid,
        "positive-dry-run-does-not-write-authority": !positive_dry_run.writes_authority && !positive_dry_run.writes_event_store && !positive_dry_run.executes_provider,
        "positive-route-has-source-refs": positive_validation.surface_route.as_ref().map(|route| !route.source_refs.is_empty()).unwrap_or(false),
        "positive-route-has-contract-skill-connector-capability": positive_validation.surface_route.as_ref().map(|route| {
            !route.action_contract_ref.is_empty()
                && !route.skill_ref.is_empty()
                && !route.connector_id.is_empty()
                && !route.required_capability.is_empty()
        }).unwrap_or(false),
        "negative-validation-rejected": !negative_validation.valid,
        "negative-dry-run-rejected": !negative_dry_run.valid,
    });
    let status = status_from_checks(&checks);

    Ok(json!({
        "version": "agentflow-v112-real-product-runtime-proof.v1",
        "status": status,
        "writesAuthority": false,
        "positiveRequest": positive_request,
        "positiveValidation": positive_validation,
        "positiveDryRun": positive_dry_run,
        "negativeRequest": negative_request,
        "negativeValidation": negative_validation,
        "negativeDryRun": negative_dry_run,
        "coverage": checks,
        "failedCoverage": failed_checks(&checks),
        "checkedAt": unix_timestamp(),
    }))
}

fn projection_proof(workspace: &Path) -> Result<Value> {
    let software =
        agentflow_projection::get_pack_industry_workbench_view(workspace, Some("software-dev"))?;
    let synthetic = agentflow_projection::get_pack_industry_workbench_view(
        workspace,
        Some("synthetic-review"),
    )?;
    let all = agentflow_projection::get_pack_industry_workbench_view(workspace, None)?;
    let checks = json!({
        "software-dev-view-valid": software.active_pack_id.as_deref() == Some("software-dev") && software.pack_list.iter().any(|item| item.pack_id == "software-dev"),
        "synthetic-review-view-valid": synthetic.active_pack_id.as_deref() == Some("synthetic-review") && synthetic.pack_list.iter().any(|item| item.pack_id == "synthetic-review"),
        "registry-projects-all-products": all.pack_list.len() >= 2,
        "source-refs-use-products": all.source_refs.iter().any(|item| item.contains("products/software-dev")) && all.source_refs.iter().any(|item| item.contains("products/synthetic-review")),
    });
    Ok(json!({
        "version": "agentflow-v112-real-product-projection-proof.v1",
        "status": status_from_checks(&checks),
        "writesAuthority": false,
        "softwareDevView": software,
        "syntheticReviewView": synthetic,
        "allProductsView": all,
        "coverage": checks,
        "failedCoverage": failed_checks(&checks),
        "checkedAt": unix_timestamp(),
    }))
}

fn registry_proof(workspace: &Path) -> Result<Value> {
    let registry = agentflow_pack::load_product_registry(workspace)?;
    let product_ids = registry
        .entries
        .iter()
        .map(|entry| entry.product_id.clone())
        .collect::<Vec<_>>();
    let checks = json!({
        "registry-has-software-dev": product_ids.iter().any(|id| id == "software-dev"),
        "registry-has-synthetic-review": product_ids.iter().any(|id| id == "synthetic-review"),
        "synthetic-review-is-direct-product": registry.entries.iter().any(|entry| entry.product_id == "synthetic-review" && entry.manifest_path.ends_with("products/synthetic-review/product.toml")),
        "registry-does-not-write-authority": !registry.writes_authority,
    });
    Ok(json!({
        "version": "agentflow-v112-registry-discovered-second-product.v1",
        "status": status_from_checks(&checks),
        "registry": registry,
        "productIds": product_ids,
        "coverage": checks,
        "failedCoverage": failed_checks(&checks),
        "checkedAt": unix_timestamp(),
    }))
}

fn desktop_surface_proof(workspace: &Path) -> Result<Value> {
    let surface = list_product_command_surface(workspace)?;
    let checks = json!({
        "surface-has-two-products": surface.summary.product_count >= 2,
        "surface-uses-runtime-validation": surface.commands.iter().any(|command| command.validation.version == agentflow_runtime_api::PACK_COMMAND_SURFACE_VERSION),
        "surface-uses-dry-run": surface.commands.iter().any(|command| command.dry_run.version == agentflow_runtime_api::PACK_COMMAND_SURFACE_VERSION),
        "surface-does-not-write-authority": !surface.writes_authority,
        "software-dev-command-installed": surface.commands.iter().any(|command| command.product_id == "software-dev" && command.command == "work.issue.start"),
        "synthetic-review-command-installed": surface.commands.iter().any(|command| command.product_id == "synthetic-review" && command.command == "synthetic.case.open"),
    });
    Ok(json!({
        "version": "agentflow-v112-desktop-product-command-route-read-model.v1",
        "status": status_from_checks(&checks),
        "readModel": surface,
        "coverage": checks,
        "failedCoverage": failed_checks(&checks),
        "checkedAt": unix_timestamp(),
    }))
}

fn multi_product_state_proof(workspace: &Path, desktop_payload: &Value) -> Result<Value> {
    let surface = desktop_payload
        .get("readModel")
        .cloned()
        .context("desktop payload missing read model")?;
    let missing_command_request = PackCommandRequest {
        pack_id: "synthetic-review".to_string(),
        command_id: "v112-missing-command".to_string(),
        command: "synthetic.case.missing".to_string(),
        actor_role: "work-agent".to_string(),
        source_surface: ActionSourceSurface::Desktop,
        target_object_type: "Case".to_string(),
        target_object_id: "case-v112-missing".to_string(),
        input: json!({}),
        evidence_refs: Vec::new(),
        artifact_refs: Vec::new(),
        idempotency_key: "v112-missing-command".to_string(),
        created_at: "2026-07-02T00:00:00Z".to_string(),
    };
    let missing_command_validation = validate_pack_command(workspace, &missing_command_request)?;
    let disabled_capability_request = PackCommandRequest {
        pack_id: "synthetic-review".to_string(),
        command_id: "v112-disabled-capability".to_string(),
        command: "synthetic.case.open".to_string(),
        actor_role: "work-agent".to_string(),
        source_surface: ActionSourceSurface::Desktop,
        target_object_type: "Case".to_string(),
        target_object_id: "case-v112-disabled".to_string(),
        input: json!({}),
        evidence_refs: Vec::new(),
        artifact_refs: Vec::new(),
        idempotency_key: "v112-disabled-capability".to_string(),
        created_at: "2026-07-02T00:00:00Z".to_string(),
    };
    let disabled_capability_validation =
        validate_pack_command(workspace, &disabled_capability_request)?;
    let checks = json!({
        "surface-includes-valid-products": surface.pointer("/summary/validProductCount").and_then(Value::as_u64).unwrap_or(0) >= 2,
        "surface-includes-invalid-or-deferred-command": surface.pointer("/summary/deferredCommandCount").and_then(Value::as_u64).unwrap_or(0) >= 1,
        "missing-command-is-invalid": !missing_command_validation.valid,
        "disabled-capability-is-deferred": !disabled_capability_validation.valid && disabled_capability_validation.failure_stage.as_deref() == Some("capability"),
    });
    Ok(json!({
        "version": "agentflow-v112-multi-product-console-states.v1",
        "status": status_from_checks(&checks),
        "readModel": surface,
        "invalidState": missing_command_validation,
        "deferredState": disabled_capability_validation,
        "stateLegend": {
            "valid": "command route validates and dry-run can be produced",
            "invalid": "command route or mapping is missing",
            "deferred": "route exists but provider or capability is unavailable"
        },
        "coverage": checks,
        "failedCoverage": failed_checks(&checks),
        "checkedAt": unix_timestamp(),
    }))
}

fn write_ready_capability_registry(workspace: &Path) -> Result<()> {
    let mut registry = default_capability_registry();
    for worker in registry.workers.iter_mut() {
        worker.health = WorkerHealth::Ready;
        worker.requires_auth = false;
        worker.disabled_reason = None;
        for capability in worker.capabilities.iter_mut() {
            capability.available = true;
            capability.requires_auth = false;
            capability.policy = CapabilityPolicy::Allowed;
            capability.disabled_reason = None;
        }
    }
    let path = workspace.join(".agentflow/runtime/capability-registry.json");
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    write_json(&path, &json!(registry))
}

fn status_from_checks(checks: &Value) -> &'static str {
    if checks
        .as_object()
        .map(|object| {
            object
                .values()
                .all(|value| value.as_bool().unwrap_or(false))
        })
        .unwrap_or(false)
    {
        "passed"
    } else {
        "failed"
    }
}

fn failed_checks(checks: &Value) -> Vec<String> {
    checks
        .as_object()
        .map(|object| {
            object
                .iter()
                .filter_map(|(key, value)| {
                    if value.as_bool().unwrap_or(false) {
                        None
                    } else {
                        Some(key.clone())
                    }
                })
                .collect()
        })
        .unwrap_or_default()
}

fn write_json(path: &Path, value: &Value) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(
        path,
        serde_json::to_string_pretty(value).context("serialize JSON")? + "\n",
    )
    .with_context(|| format!("write {}", path.display()))
}

fn unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}
