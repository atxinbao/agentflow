//! Legacy CLI output helpers.
//!
//! Output formatting is isolated from command dispatch so legacy command code
//! stays focused on compatibility behavior.

use agentflow_core::legacy::product_feature::ProductFeatureExecutionSnapshot;
use agentflow_core::legacy::run_verify_review::ControlledRunPlan;
use agentflow_core::legacy::team_project_milestone_issue::CreationWriteSummary;

pub(crate) fn print_feature_execution_snapshot(
    snapshot: &ProductFeatureExecutionSnapshot,
    verbose: bool,
) {
    println!("feature project: {}", snapshot.project_id);
    println!("title: {}", snapshot.project_title);
    println!("project status: {}", snapshot.project_canonical_status);
    println!("project raw status: {}", snapshot.project_status);
    println!("goal: {}", snapshot.project_goal);
    println!("ready: {}", snapshot.feature_ready);
    println!("active milestone: {}", snapshot.active_milestone_id);
    println!("next action: {}", snapshot.next_action);
    println!("recommended command: {}", snapshot.recommended_command);

    if let Some(issue) = &snapshot.current_issue {
        println!("current issue: {} [{}]", issue.id, issue.title);
        println!("issue status: {}", issue.canonical_status);
        println!("issue raw status: {}", issue.status);
        println!("ready: {}", issue.ready);
        println!("eligible: {}", issue.eligible);
        println!("leased: {}", issue.leased);
        println!("dry-run recorded: {}", issue.dry_run_recorded);
        println!(
            "latest run: {}",
            issue.latest_run_id.as_deref().unwrap_or("none")
        );
        println!(
            "latest run status: {}",
            issue.latest_run_status.as_deref().unwrap_or("none")
        );
        println!("validation: {}", issue.validation_status);
        println!("execution state: {}", issue.execution_state);
        println!(
            "latest run plan: {}",
            if issue.latest_run_plan.is_empty() {
                "none".to_string()
            } else {
                issue.latest_run_plan.join(" | ")
            }
        );
        println!(
            "expected files: {}",
            if issue.expected_files.is_empty() {
                "none".to_string()
            } else {
                issue.expected_files.join(", ")
            }
        );
        println!(
            "blocked files: {}",
            if issue.blocked_files.is_empty() {
                "none".to_string()
            } else {
                issue.blocked_files.join(", ")
            }
        );
        println!(
            "validation commands: {}",
            if issue.validation_commands.is_empty() {
                "none".to_string()
            } else {
                issue.validation_commands.join(" | ")
            }
        );
        println!(
            "evidence requirements: {}",
            if issue.evidence_requirements.is_empty() {
                "none".to_string()
            } else {
                issue.evidence_requirements.join(", ")
            }
        );
        if issue.failure_reasons.is_empty() {
            println!("failure reasons: none");
        } else {
            println!("failure reasons: {}", issue.failure_reasons.join(", "));
        }
    } else {
        println!("current issue: none");
    }

    if verbose {
        println!("milestones: {}", snapshot.milestones.len());
        for milestone in &snapshot.milestones {
            println!(
                "- {} progress {}/{} ({}%, total {}, canceled {})",
                milestone.id,
                milestone.progress.done_issue_count,
                milestone.progress.non_canceled_issue_count,
                milestone.progress.percent,
                milestone.progress.total_issue_count,
                milestone.progress.canceled_issue_count
            );
        }
        println!("issues: {}", snapshot.issues.len());
        for issue in &snapshot.issues {
            println!(
                "- {} [{}] raw={} action={} validation={} evidence={} review={}",
                issue.id,
                issue.canonical_status,
                issue.status,
                issue.next_action,
                issue.validation_status,
                issue.evidence_path.as_deref().unwrap_or("none"),
                issue.review_path.as_deref().unwrap_or("none")
            );
        }
    }

    println!("read only: {}", snapshot.boundary.read_only);
}

pub(crate) fn print_controlled_run_plan(plan: &ControlledRunPlan) {
    println!("run plan goal: {}", plan.goal);
    print_list_line("run plan steps", &plan.planned_steps, " | ");
    print_list_line("expected files", &plan.expected_files, ", ");
    print_list_line("blocked files", &plan.blocked_files, ", ");
    print_list_line("validation commands", &plan.validation_commands, " | ");
    print_list_line("evidence requirements", &plan.evidence_requirements, ", ");
    print_list_line("rollback plan", &plan.rollback_plan, " | ");
    println!("source edits: none");
    println!("remote objects: none");
    println!("model call: none");
}

pub(crate) fn print_creation_summary(summary: &CreationWriteSummary) {
    println!("creation mode: {}", summary.preview.mode);
    println!("kind: {}", summary.preview.kind);
    println!("id: {}", summary.preview.entity_id);
    println!("title: {}", summary.preview.title);
    println!("action: {}", summary.preview.action);
    println!("writes required: {}", summary.preview.writes_required);
    println!(
        "recommended command: {}",
        summary.preview.recommended_command
    );
    println!("v1 model: {}", summary.preview.v1_contract.model);
    println!("v1 relation: {}", summary.preview.v1_contract.relation);
    if let Some(team) = &summary.preview.v1_contract.team {
        println!("v1 team: {} ({})", team.name, team.team_id);
    }
    if let Some(project) = &summary.preview.v1_contract.project_charter {
        println!("v1 project status: {}", project.status);
        println!("v1 project goal: {}", project.goal);
    }
    if let Some(milestone) = &summary.preview.v1_contract.milestone_gate {
        println!("v1 milestone goal: {}", milestone.goal);
        println!(
            "v1 milestone exit criteria: {}",
            milestone.exit_criteria.len()
        );
    }
    if let Some(issue) = &summary.preview.v1_contract.issue_contract {
        println!("v1 issue initial state: {}", issue.initial_state);
        println!(
            "v1 issue validation commands: {}",
            issue.validation_commands.len()
        );
    }
    println!("files: {}", summary.preview.files.len());
    for file in &summary.preview.files {
        println!("- {} {} [{}]", file.action, file.path, file.kind);
    }
    if summary.written_paths.is_empty() {
        println!("preview only: no facts written");
    } else {
        println!("written files: {}", summary.written_paths.len());
        for path in &summary.written_paths {
            println!("- wrote {}", path.display());
        }
    }
    println!("read only preview: {}", summary.preview.boundary.read_only);
}

fn print_list_line(label: &str, values: &[String], separator: &str) {
    if values.is_empty() {
        println!("{label}: none");
    } else {
        println!("{label}: {}", values.join(separator));
    }
}
