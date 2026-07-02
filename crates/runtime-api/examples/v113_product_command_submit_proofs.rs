use agentflow_action_contract::ActionSourceSurface;
use agentflow_capability_registry::{default_capability_registry, CapabilityPolicy, WorkerHealth};
use agentflow_runtime_api::{
    dry_run_pack_command, list_product_command_surface, query_pack_surface_route,
    submit_product_command, validate_pack_command, PackCommandRequest, ProductCommandState,
    ProductCommandSubmitRequest,
};
use anyhow::{bail, Result};
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
            "usage: v113_product_command_submit_proofs <workspace> <state-proof> <submit-proof> <runtime-submit-proof> <evidence-proof> <multi-state-proof>"
        );
    }
    let workspace = PathBuf::from(&args[0]);
    let state_out = PathBuf::from(&args[1]);
    let submit_out = PathBuf::from(&args[2]);
    let runtime_submit_out = PathBuf::from(&args[3]);
    let evidence_out = PathBuf::from(&args[4]);
    let multi_state_out = PathBuf::from(&args[5]);

    write_product_fixtures(&workspace)?;
    write_ready_capability_registry(&workspace)?;

    let submit_payload = submit_contract_proof(&workspace)?;
    let runtime_submit_payload = runtime_submit_api_proof(&workspace)?;
    let evidence_payload = evidence_handoff_proof(&runtime_submit_payload)?;
    let state_payload = state_contract_proof(&workspace, &runtime_submit_payload)?;
    let multi_state_payload = multi_product_state_proof(&workspace, &state_payload)?;

    write_json(&state_out, &state_payload)?;
    write_json(&submit_out, &submit_payload)?;
    write_json(&runtime_submit_out, &runtime_submit_payload)?;
    write_json(&evidence_out, &evidence_payload)?;
    write_json(&multi_state_out, &multi_state_payload)?;

    Ok(())
}

fn state_contract_proof(workspace: &Path, runtime_submit_payload: &Value) -> Result<Value> {
    let valid_route = query_pack_surface_route(workspace, "software-dev", "work.issue.start")?;
    let valid_request = PackCommandRequest {
        pack_id: "software-dev".to_string(),
        command_id: "v113-state-valid".to_string(),
        command: "work.issue.start".to_string(),
        actor_role: "work-agent".to_string(),
        source_surface: ActionSourceSurface::Desktop,
        target_object_type: valid_route.target_object_type.clone(),
        target_object_id: "AF-V113-STATE-VALID".to_string(),
        input: json!({"reason": "v1.1.3 valid state proof"}),
        evidence_refs: Vec::new(),
        artifact_refs: Vec::new(),
        idempotency_key: "v113-state-valid".to_string(),
        created_at: "2026-07-02T00:00:00Z".to_string(),
    };
    let valid_validation = validate_pack_command(workspace, &valid_request)?;
    let valid_dry_run = dry_run_pack_command(workspace, &valid_request)?;

    let invalid_request = PackCommandRequest {
        pack_id: "software-dev".to_string(),
        command_id: "v113-state-invalid".to_string(),
        command: "work.issue.teleport".to_string(),
        actor_role: "work-agent".to_string(),
        source_surface: ActionSourceSurface::Desktop,
        target_object_type: "Issue".to_string(),
        target_object_id: "AF-V113-STATE-INVALID".to_string(),
        input: json!({"reason": "missing command must be invalid"}),
        evidence_refs: Vec::new(),
        artifact_refs: Vec::new(),
        idempotency_key: "v113-state-invalid".to_string(),
        created_at: "2026-07-02T00:00:00Z".to_string(),
    };
    let invalid_validation = validate_pack_command(workspace, &invalid_request)?;

    let deferred_request = PackCommandRequest {
        pack_id: "synthetic-review".to_string(),
        command_id: "v113-state-deferred".to_string(),
        command: "synthetic.case.open".to_string(),
        actor_role: "work-agent".to_string(),
        source_surface: ActionSourceSurface::Desktop,
        target_object_type: "Case".to_string(),
        target_object_id: "AF-V113-STATE-DEFERRED".to_string(),
        input: json!({"reason": "provider capability unavailable must be deferred"}),
        evidence_refs: Vec::new(),
        artifact_refs: Vec::new(),
        idempotency_key: "v113-state-deferred".to_string(),
        created_at: "2026-07-02T00:00:00Z".to_string(),
    };
    let deferred_validation = validate_pack_command(workspace, &deferred_request)?;
    let submitted_state = runtime_submit_payload
        .pointer("/acceptedSubmit/state")
        .and_then(Value::as_str)
        .unwrap_or_default();

    let checks = json!({
        "valid-command-state-valid": valid_validation.valid && valid_dry_run.valid,
        "invalid-command-has-invalid-stage": !invalid_validation.valid && invalid_validation.failure_stage.is_some(),
        "deferred-command-has-capability-stage": !deferred_validation.valid && deferred_validation.failure_stage.as_deref() == Some("capability"),
        "submitted-command-state-submitted": submitted_state == "submitted",
        "failure-stage-is-machine-readable": invalid_validation.failure_stage.is_some() && deferred_validation.failure_stage.is_some(),
    });

    Ok(json!({
        "version": "agentflow-v113-product-command-state-contract.v1",
        "status": status_from_checks(&checks),
        "states": ["valid", "invalid", "deferred", "unavailable", "rejected", "submitted"],
        "validState": {
            "validation": valid_validation,
            "dryRun": valid_dry_run,
        },
        "invalidState": invalid_validation,
        "deferredState": deferred_validation,
        "submittedState": runtime_submit_payload.pointer("/acceptedSubmit"),
        "coverage": checks,
        "failedCoverage": failed_checks(&checks),
        "checkedAt": unix_timestamp(),
    }))
}

fn submit_contract_proof(workspace: &Path) -> Result<Value> {
    let missing_receipt = submit_product_command(
        workspace,
        ProductCommandSubmitRequest {
            pack_id: "software-dev".to_string(),
            command: "work.issue.start".to_string(),
            target_object_id: Some("AF-V113-MISSING-RECEIPT".to_string()),
            dry_run_receipt_id: None,
            validation_evidence_ref: None,
            input: json!({"reason": "missing dry-run receipt must reject"}),
            evidence_refs: Vec::new(),
            artifact_refs: Vec::new(),
            idempotency_key: Some("v113-missing-receipt".to_string()),
            actor_role: Some("work-agent".to_string()),
            created_at: Some("2026-07-02T00:00:00Z".to_string()),
        },
    )?;
    let invalid = submit_product_command(
        workspace,
        ProductCommandSubmitRequest {
            pack_id: "software-dev".to_string(),
            command: "work.issue.teleport".to_string(),
            target_object_id: Some("AF-V113-INVALID-SUBMIT".to_string()),
            dry_run_receipt_id: Some("dry-run-invalid-submit".to_string()),
            validation_evidence_ref: None,
            input: json!({}),
            evidence_refs: Vec::new(),
            artifact_refs: Vec::new(),
            idempotency_key: Some("v113-invalid-submit".to_string()),
            actor_role: Some("work-agent".to_string()),
            created_at: Some("2026-07-02T00:00:00Z".to_string()),
        },
    )?;
    let checks = json!({
        "missing-receipt-rejected": missing_receipt.state == ProductCommandState::Rejected && missing_receipt.dry_run_required,
        "invalid-command-not-submitted": invalid.state == ProductCommandState::Invalid && !invalid.accepted,
        "rejected-path-writes-no-runtime-response": missing_receipt.runtime_response.is_none() && invalid.runtime_response.is_none(),
    });
    Ok(json!({
        "version": "agentflow-v113-product-command-submit-contract.v1",
        "status": status_from_checks(&checks),
        "missingReceipt": missing_receipt,
        "invalidSubmit": invalid,
        "coverage": checks,
        "failedCoverage": failed_checks(&checks),
        "checkedAt": unix_timestamp(),
    }))
}

fn runtime_submit_api_proof(workspace: &Path) -> Result<Value> {
    let accepted = submit_product_command(
        workspace,
        ProductCommandSubmitRequest {
            pack_id: "software-dev".to_string(),
            command: "work.issue.start".to_string(),
            target_object_id: Some("AF-V113-SUBMIT".to_string()),
            dry_run_receipt_id: Some("dry-run-v113-submit".to_string()),
            validation_evidence_ref: None,
            input: json!({"reason": "v1.1.3 submit Runtime API proof"}),
            evidence_refs: vec!["runtime/v113-product-command-state-contract.json".to_string()],
            artifact_refs: Vec::new(),
            idempotency_key: Some("v113-runtime-submit".to_string()),
            actor_role: Some("work-agent".to_string()),
            created_at: Some("2026-07-02T00:00:00Z".to_string()),
        },
    )?;
    let checks = json!({
        "accepted-submit-state-submitted": accepted.state == ProductCommandState::Submitted,
        "accepted-submit-has-runtime-response": accepted.runtime_response.is_some(),
        "accepted-submit-has-receipt": accepted.receipt.is_some(),
        "affected-projections-present": !accepted.affected_projections.is_empty(),
    });
    Ok(json!({
        "version": "agentflow-v113-runtime-product-command-submit-api.v1",
        "status": status_from_checks(&checks),
        "acceptedSubmit": accepted,
        "coverage": checks,
        "failedCoverage": failed_checks(&checks),
        "checkedAt": unix_timestamp(),
    }))
}

fn evidence_handoff_proof(runtime_submit_payload: &Value) -> Result<Value> {
    let handoff = runtime_submit_payload
        .pointer("/acceptedSubmit/evidenceHandoff")
        .cloned()
        .unwrap_or(Value::Null);
    let checks = json!({
        "handoff-present": handoff.is_object(),
        "evidence-policy-bound": handoff.pointer("/evidencePolicyRef").and_then(Value::as_str).is_some_and(|value| !value.is_empty()),
        "acceptance-policy-bound": handoff.pointer("/acceptancePolicyRef").and_then(Value::as_str).is_some_and(|value| !value.is_empty()),
        "required-evidence-listed": handoff.pointer("/requiredEvidence").and_then(Value::as_array).is_some_and(|items| items.len() >= 2),
        "projection-handoff-listed": handoff.pointer("/affectedProjections").and_then(Value::as_array).is_some_and(|items| !items.is_empty()),
    });
    Ok(json!({
        "version": "agentflow-v113-product-command-evidence-handoff.v1",
        "status": status_from_checks(&checks),
        "handoff": handoff,
        "coverage": checks,
        "failedCoverage": failed_checks(&checks),
        "checkedAt": unix_timestamp(),
    }))
}

fn multi_product_state_proof(workspace: &Path, state_payload: &Value) -> Result<Value> {
    let surface = list_product_command_surface(workspace)?;
    let states = surface
        .commands
        .iter()
        .map(|command| {
            json!({
                "productId": command.product_id,
                "command": command.command,
                "state": command.state,
                "failureStage": command.failure_stage,
                "reason": command.reason,
            })
        })
        .collect::<Vec<_>>();
    let checks = json!({
        "surface-has-two-products": surface.summary.product_count >= 2,
        "surface-has-valid-state": surface.summary.valid_command_count >= 1,
        "surface-has-deferred-or-unavailable-state": surface.summary.deferred_command_count + surface.summary.unavailable_command_count >= 1,
        "state-legend-has-six-states": surface.state_legend.len() == 6,
        "state-proof-passed": state_payload.get("status").and_then(Value::as_str) == Some("passed"),
    });
    Ok(json!({
        "version": "agentflow-v113-multi-product-state-ui-proof.v1",
        "status": status_from_checks(&checks),
        "readModel": surface,
        "stateRows": states,
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
    fs::create_dir_all(path.parent().expect("registry parent"))?;
    write_json(&path, &registry)?;
    Ok(())
}

fn write_product_fixtures(workspace: &Path) -> Result<()> {
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("runtime-api crate parent")
        .parent()
        .expect("workspace root")
        .join("products");
    let target_root = workspace.join("products");
    for product_id in ["software-dev", "synthetic-review"] {
        let source = source_root.join(product_id);
        let target = target_root.join(product_id);
        if is_same_path(&source, &target) {
            continue;
        }
        copy_dir_all(&source, &target)?;
    }
    Ok(())
}

fn is_same_path(left: &Path, right: &Path) -> bool {
    match (fs::canonicalize(left), fs::canonicalize(right)) {
        (Ok(left), Ok(right)) => left == right,
        _ => left == right,
    }
}

fn copy_dir_all(source: &Path, target: &Path) -> Result<()> {
    if target.exists() {
        fs::remove_dir_all(target)?;
    }
    fs::create_dir_all(target)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        let target_path = target.join(entry.file_name());
        if source_path.is_dir() {
            copy_dir_all(&source_path, &target_path)?;
        } else {
            fs::copy(&source_path, &target_path)?;
        }
    }
    Ok(())
}

fn status_from_checks(checks: &Value) -> &'static str {
    if checks.as_object().is_some_and(|object| {
        object
            .values()
            .all(|value| value.as_bool().unwrap_or(false))
    }) {
        "passed"
    } else {
        "failed"
    }
}

fn failed_checks(checks: &Value) -> Vec<String> {
    checks
        .as_object()
        .into_iter()
        .flat_map(|object| object.iter())
        .filter_map(|(key, value)| {
            if value.as_bool().unwrap_or(false) {
                None
            } else {
                Some(key.clone())
            }
        })
        .collect()
}

fn write_json(path: &Path, value: &impl serde::Serialize) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_string_pretty(value)? + "\n")?;
    Ok(())
}

fn unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
