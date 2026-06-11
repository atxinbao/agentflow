use crate::{
    manager::load_issue_for_run,
    model::{
        ExecuteCheckStatus, ExecuteHumanConfirmation, ExecuteLease, ExecuteLeaseStatus,
        ExecutePreflight, ExecutePreflightCheck, ExecuteRun, ExecuteRunStatus,
        EXECUTE_PREFLIGHT_VERSION,
    },
    storage::{
        canonical_project_root, read_json, read_run, rebuild_index, run_dir,
        unix_timestamp_seconds, update_run_status, write_json, write_run,
    },
};
use agentflow_input::issue::{
    validate_agent_issue_permission, AgentRole, InputIssue, InputRiskLevel,
};
use anyhow::Result;
use std::path::Path;

pub fn confirm_high_risk_execute_run(
    project_root: impl AsRef<Path>,
    run_id: String,
    confirmation_text: String,
) -> Result<ExecuteHumanConfirmation> {
    let root = canonical_project_root(project_root)?;
    let run = read_run(&root, &run_id)?;
    let issue = load_issue_for_run(&root, &run)?;
    if !issue.execution_risk.requires_human_confirmation() {
        anyhow::bail!(
            "run {} is not high risk and does not require confirmation",
            run.run_id
        );
    }
    if confirmation_text.trim().is_empty() {
        anyhow::bail!("high risk confirmation text cannot be empty");
    }
    let confirmation = ExecuteHumanConfirmation {
        version: "execute-human-confirmation.v1".to_string(),
        run_id: run.run_id,
        issue_id: issue.issue_id,
        risk_level: "high".to_string(),
        confirmed_by: "human".to_string(),
        confirmed_at: crate::storage::unix_timestamp_seconds(),
        confirmation_text,
        scope: "execute-run".to_string(),
    };
    write_json(
        &run_dir(&root, &run_id).join("confirmations/high-risk-confirmation.json"),
        &confirmation,
    )?;
    Ok(confirmation)
}

pub fn execute_run_preflight(
    project_root: impl AsRef<Path>,
    run_id: String,
) -> Result<ExecutePreflight> {
    let root = canonical_project_root(project_root)?;
    let mut run = read_run(&root, &run_id)?;
    let issue = load_issue_for_run(&root, &run).ok();
    let mut checks = Vec::new();

    push_check(
        &mut checks,
        "ownership",
        agentflow_agent_manual::assert_agentflow_workspace_owned_or_creatable(&root).is_ok(),
        "AgentFlow workspace ownership is ready.",
        "AgentFlow workspace ownership is not ready.",
    );

    let define_status = agentflow_agent_manual::validate_agent_working_manual(&root);
    push_check(
        &mut checks,
        "define",
        define_status
            .as_ref()
            .map(|status| status.ready)
            .unwrap_or(false),
        "Agent working manual is ready.",
        "Agent working manual is missing or degraded.",
    );

    let panel_status = agentflow_panel::load_project_panel_status(&root);
    let panel_ok = panel_status
        .as_ref()
        .map(|status| {
            matches!(
                status.status,
                agentflow_panel::PanelStatus::Ready | agentflow_panel::PanelStatus::Degraded
            )
        })
        .unwrap_or(false);
    push_check(
        &mut checks,
        "panel",
        panel_ok,
        "Project panel is ready or degraded.",
        "Project panel is missing or failed.",
    );

    let input_status = agentflow_input::validate_input_workspace(&root);
    push_check(
        &mut checks,
        "input",
        input_status
            .as_ref()
            .map(|snapshot| snapshot.ready)
            .unwrap_or(false),
        "Input workspace is ready.",
        "Input workspace is not ready.",
    );

    match issue.as_ref() {
        Some(issue) => {
            checks.push(ExecutePreflightCheck {
                name: "issue".to_string(),
                status: ExecuteCheckStatus::Passed,
                message: Some("Input issue exists.".to_string()),
                risk_level: None,
                human_confirmation_required: None,
                confirmed: None,
            });
            checks.push(ExecutePreflightCheck {
                name: "source-spec-id".to_string(),
                status: if issue.source_spec_id.trim().is_empty() {
                    ExecuteCheckStatus::Blocked
                } else {
                    ExecuteCheckStatus::Passed
                },
                message: Some(if issue.source_spec_id.trim().is_empty() {
                    "Issue sourceSpecId is missing.".to_string()
                } else {
                    "Issue sourceSpecId exists.".to_string()
                }),
                risk_level: None,
                human_confirmation_required: None,
                confirmed: None,
            });
            let role_check = validate_agent_issue_permission(issue, &AgentRole::BuildAgent);
            checks.push(ExecutePreflightCheck {
                name: "agent-role".to_string(),
                status: if role_check.is_ok() {
                    ExecuteCheckStatus::Passed
                } else {
                    ExecuteCheckStatus::Blocked
                },
                message: Some(
                    role_check
                        .map(|_| "Issue is assigned to Build Agent.".to_string())
                        .unwrap_or_else(|error| format!("Agent 角色不匹配：{error}")),
                ),
                risk_level: None,
                human_confirmation_required: None,
                confirmed: None,
            });

            checks.push(prepare_context_pack_for_issue(&root, &mut run, issue));

            let approved_spec = root
                .join(".agentflow/input/specs/approved")
                .join(&issue.source_spec_id);
            checks.push(ExecutePreflightCheck {
                name: "approved-spec".to_string(),
                status: if approved_spec.is_dir() && approved_spec.join("approval.json").is_file() {
                    ExecuteCheckStatus::Passed
                } else {
                    ExecuteCheckStatus::Blocked
                },
                message: Some(
                    if approved_spec.is_dir() && approved_spec.join("approval.json").is_file() {
                        "Approved SPEC and approval.json exist.".to_string()
                    } else {
                        "Approved SPEC or approval.json is missing.".to_string()
                    },
                ),
                risk_level: None,
                human_confirmation_required: None,
                confirmed: None,
            });

            let high_risk = matches!(issue.execution_risk, InputRiskLevel::High);
            let confirmed = !high_risk
                || run_dir(&root, &run_id)
                    .join("confirmations/high-risk-confirmation.json")
                    .is_file();
            checks.push(ExecutePreflightCheck {
                name: "risk".to_string(),
                status: if confirmed {
                    ExecuteCheckStatus::Passed
                } else {
                    ExecuteCheckStatus::Blocked
                },
                message: Some(if high_risk && !confirmed {
                    "High risk issue requires human confirmation before execute.".to_string()
                } else {
                    "Risk check passed.".to_string()
                }),
                risk_level: Some(format!("{:?}", issue.execution_risk).to_lowercase()),
                human_confirmation_required: Some(high_risk),
                confirmed: Some(confirmed),
            });

            let lease_check = issue_lease_state(&root, &issue.issue_id);
            checks.push(ExecutePreflightCheck {
                name: "lease".to_string(),
                status: lease_check.status,
                message: Some(lease_check.message),
                risk_level: None,
                human_confirmation_required: None,
                confirmed: None,
            });

            let validation_available = !issue.validation_hints.is_empty()
                || root.join(".agentflow/panel/tests.json").is_file();
            checks.push(ExecutePreflightCheck {
                name: "validation-hints".to_string(),
                status: if validation_available {
                    ExecuteCheckStatus::Passed
                } else {
                    ExecuteCheckStatus::Blocked
                },
                message: Some(if validation_available {
                    "Validation hints or panel tests are available.".to_string()
                } else {
                    "Validation hints and panel tests are missing.".to_string()
                }),
                risk_level: None,
                human_confirmation_required: None,
                confirmed: None,
            });
        }
        None => checks.push(ExecutePreflightCheck {
            name: "issue".to_string(),
            status: ExecuteCheckStatus::Blocked,
            message: Some("Input issue cannot be loaded.".to_string()),
            risk_level: None,
            human_confirmation_required: None,
            confirmed: None,
        }),
    }

    push_check(
        &mut checks,
        "working-tree-readable",
        std::fs::read_dir(&root).is_ok(),
        "Project root is readable.",
        "Project root cannot be read.",
    );

    let blocked_reason = checks
        .iter()
        .find(|check| {
            matches!(
                check.status,
                ExecuteCheckStatus::Blocked | ExecuteCheckStatus::Failed
            )
        })
        .and_then(|check| check.message.clone());
    let ready = blocked_reason.is_none();
    let preflight = ExecutePreflight {
        version: EXECUTE_PREFLIGHT_VERSION.to_string(),
        run_id: run.run_id.clone(),
        issue_id: run.issue_id.clone(),
        status: if ready { "ready" } else { "blocked" }.to_string(),
        checks,
        blocked_reason,
    };
    write_json(&run_dir(&root, &run_id).join("preflight.json"), &preflight)?;
    update_run_status(
        &root,
        &run_id,
        if ready {
            ExecuteRunStatus::Preflight
        } else {
            ExecuteRunStatus::Blocked
        },
    )?;
    if let Err(error) = rebuild_index(&root) {
        if has_unreadable_lease_block(&preflight) {
            return Ok(preflight);
        }
        return Err(error);
    }
    Ok(preflight)
}

fn prepare_context_pack_for_issue(
    root: &Path,
    run: &mut ExecuteRun,
    issue: &InputIssue,
) -> ExecutePreflightCheck {
    let objective = if issue.summary.trim().is_empty() {
        issue.scope.join("\n")
    } else {
        issue.summary.clone()
    };
    let context_pack_id = context_pack_id_for_issue(issue);
    let canonical_path = format!(".agentflow/panel/context-packs/{context_pack_id}.json");

    match agentflow_panel::panel_preflight(
        root,
        "issue",
        Some(&issue.issue_id),
        &issue.title,
        &objective,
        &issue.acceptance_criteria,
    ) {
        Ok(snapshot) if snapshot.ready => {
            let context_pack_path = snapshot.context_pack_path.unwrap_or(canonical_path);
            run.input.context_pack_id = Some(context_pack_id);
            run.input.context_pack_path = Some(context_pack_path.clone());
            run.updated_at = unix_timestamp_seconds();

            if let Err(error) = write_run(root, run) {
                return ExecutePreflightCheck {
                    name: "context-pack".to_string(),
                    status: ExecuteCheckStatus::Failed,
                    message: Some(format!(
                        "Panel Context Pack generated but run update failed: {error}"
                    )),
                    risk_level: None,
                    human_confirmation_required: None,
                    confirmed: None,
                };
            }

            ExecutePreflightCheck {
                name: "context-pack".to_string(),
                status: ExecuteCheckStatus::Passed,
                message: Some(format!("Panel Context Pack is ready: {context_pack_path}.")),
                risk_level: None,
                human_confirmation_required: None,
                confirmed: None,
            }
        }
        Ok(snapshot) => ExecutePreflightCheck {
            name: "context-pack".to_string(),
            status: ExecuteCheckStatus::Blocked,
            message: Some(format!(
                "Panel Context Pack cannot be generated: {}",
                snapshot.reason
            )),
            risk_level: None,
            human_confirmation_required: None,
            confirmed: None,
        },
        Err(error) => ExecutePreflightCheck {
            name: "context-pack".to_string(),
            status: ExecuteCheckStatus::Blocked,
            message: Some(format!("Panel Context Pack cannot be generated: {error}")),
            risk_level: None,
            human_confirmation_required: None,
            confirmed: None,
        },
    }
}

fn context_pack_id_for_issue(issue: &InputIssue) -> String {
    issue.issue_id.replace('/', "-")
}

fn push_check(
    checks: &mut Vec<ExecutePreflightCheck>,
    name: &str,
    passed: bool,
    passed_message: &str,
    failed_message: &str,
) {
    checks.push(ExecutePreflightCheck {
        name: name.to_string(),
        status: if passed {
            ExecuteCheckStatus::Passed
        } else {
            ExecuteCheckStatus::Blocked
        },
        message: Some(
            if passed {
                passed_message
            } else {
                failed_message
            }
            .to_string(),
        ),
        risk_level: None,
        human_confirmation_required: None,
        confirmed: None,
    });
}

struct LeasePreflightCheck {
    status: ExecuteCheckStatus,
    message: String,
}

fn issue_lease_state(root: &Path, issue_id: &str) -> LeasePreflightCheck {
    let lease_path = root
        .join(".agentflow/execute/leases")
        .join(format!("{issue_id}.json"));

    if !lease_path.is_file() {
        return LeasePreflightCheck {
            status: ExecuteCheckStatus::Passed,
            message: "No active lease exists for this issue.".to_string(),
        };
    }

    match read_json::<ExecuteLease>(&lease_path) {
        Ok(lease) if matches!(lease.status, ExecuteLeaseStatus::Active) => LeasePreflightCheck {
            status: ExecuteCheckStatus::Blocked,
            message: "Active lease already exists for this issue.".to_string(),
        },
        Ok(lease) if matches!(lease.status, ExecuteLeaseStatus::Released) => LeasePreflightCheck {
            status: ExecuteCheckStatus::Passed,
            message: "Existing lease is released and does not block execute.".to_string(),
        },
        Ok(_) => LeasePreflightCheck {
            status: ExecuteCheckStatus::Blocked,
            message: "Lease state is unsupported and blocks execute.".to_string(),
        },
        Err(error) => LeasePreflightCheck {
            status: ExecuteCheckStatus::Blocked,
            message: format!("Lease state unreadable: {error}"),
        },
    }
}

fn has_unreadable_lease_block(preflight: &ExecutePreflight) -> bool {
    preflight.checks.iter().any(|check| {
        check.name == "lease"
            && matches!(check.status, ExecuteCheckStatus::Blocked)
            && check
                .message
                .as_deref()
                .is_some_and(|message| message.contains("Lease state unreadable"))
    })
}
