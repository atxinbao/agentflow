//! Legacy CLI command gate.
//!
//! 006 retires archived 2026-05 write commands. Command names still parse so
//! users receive a clear migration message, but old writers are no longer
//! executed from the CLI.

use crate::args::{Cli, Command};
use crate::retirement::{
    legacy_command_status, print_legacy_retirement_message, should_disable_legacy_command,
};
use agentflow_core::active::{
    read_local_metrics_snapshot, read_local_project_model_snapshot, read_local_search_snapshot,
};
use anyhow::{bail, Result};
use clap::Parser;

pub(crate) fn run() -> Result<()> {
    let cli = Cli::parse();
    let cwd = std::env::current_dir()?;

    if should_disable_legacy_command(&cli.command) {
        let status = legacy_command_status(&cli.command);
        print_legacy_retirement_message(&status);
        return Ok(());
    }

    match cli.command {
        Command::Metrics => {
            let metrics = read_local_metrics_snapshot(&cwd)?;
            println!("metrics root: {}", metrics.project_root);
            println!("initialized: {}", metrics.initialized);
            println!(
                "issues: total {} completed {} planned {} active {}",
                metrics.issues.total,
                metrics.issues.completed,
                metrics.issues.planned,
                metrics.issues.active
            );
            println!(
                "runs: total {} passed {} failed {} missing validation {}",
                metrics.runs.total,
                metrics.runs.passed,
                metrics.runs.failed,
                metrics.runs.missing_validation
            );
            println!(
                "artifacts: evidence {} reviews {} project updates {} saved views {}",
                metrics.artifacts.evidence_reports,
                metrics.artifacts.reviews,
                metrics.artifacts.project_updates,
                metrics.artifacts.saved_views
            );
            println!("goal ready: {}", metrics.goal_ready);
            println!(
                "active issue: {}",
                metrics.active_issue_id.as_deref().unwrap_or("none")
            );
            println!("next action: {}", metrics.next_action);
            println!("recommended command: {}", metrics.recommended_command);
            if let Some(run) = &metrics.latest_run {
                println!(
                    "latest run: {} -> {} [{} / {}]",
                    run.id, run.issue_id, run.status, run.validation_status
                );
            } else {
                println!("latest run: none");
            }
            if let Some(evidence) = &metrics.latest_evidence {
                println!("latest evidence: {} ({})", evidence.path, evidence.title);
            } else {
                println!("latest evidence: none");
            }
            if let Some(review) = &metrics.latest_review {
                println!("latest review: {} ({})", review.path, review.title);
            } else {
                println!("latest review: none");
            }
            println!("read only: {}", metrics.boundary.read_only);
        }
        Command::Projects => {
            let snapshot = read_local_project_model_snapshot(&cwd)?;
            println!("projects root: {}", snapshot.project_root);
            println!("initialized: {}", snapshot.initialized);
            if let Some(workspace) = &snapshot.workspace {
                println!(
                    "workspace: {} ({}) active project {}",
                    workspace.id, workspace.name, workspace.active_project_id
                );
                println!(
                    "workspace counts: issues {} completed {}",
                    workspace.issue_count, workspace.completed_issue_count
                );
            } else {
                println!("workspace: none");
            }
            println!("teams: {}", snapshot.teams.len());
            for team in &snapshot.teams {
                println!(
                    "- team {} ({}) issues {} wip {}",
                    team.id,
                    team.name,
                    team.issue_ids.len(),
                    team.wip_limit
                );
            }
            println!("projects: {}", snapshot.projects.len());
            for project in &snapshot.projects {
                println!(
                    "- project {} [{}] {}",
                    project.id, project.canonical_status, project.name
                );
                println!("  raw status: {}", project.status);
                println!("  goal: {}", project.goal);
                println!("  active milestone: {}", project.active_milestone_id);
                println!(
                    "  issues: {} completed {}",
                    project.issue_count, project.completed_issue_count
                );
                println!(
                    "  next intent: {}",
                    project.next_issue_intent.as_deref().unwrap_or("none")
                );
                println!(
                    "  recommended command: {}",
                    project
                        .recommended_command
                        .as_deref()
                        .unwrap_or("agentflow goal next")
                );
                for milestone in &project.milestones {
                    println!(
                        "  - milestone {} progress {}/{} ({}%, total {}, canceled {})",
                        milestone.id,
                        milestone.progress.done_issue_count,
                        milestone.progress.non_canceled_issue_count,
                        milestone.progress.percent,
                        milestone.progress.total_issue_count,
                        milestone.progress.canceled_issue_count
                    );
                    for issue_id in &milestone.issue_ids {
                        if let Some(issue) = snapshot
                            .issue_refs
                            .iter()
                            .find(|issue| &issue.id == issue_id)
                        {
                            println!(
                                "    - issue {} [{}] raw={} run {} validation {} evidence {} update {}",
                                issue.id,
                                issue.canonical_status,
                                issue.status,
                                issue.latest_run_id.as_deref().unwrap_or("none"),
                                issue.validation_status,
                                issue.evidence_path.as_deref().unwrap_or("none"),
                                issue.project_update_path.as_deref().unwrap_or("none")
                            );
                        }
                    }
                }
            }
            println!("issue refs: {}", snapshot.issue_refs.len());
            println!(
                "goal loop: {} -> {}",
                snapshot.goal_loop_selection.next_action,
                snapshot.goal_loop_selection.recommended_command
            );
            println!("read only: {}", snapshot.boundary.read_only);
        }
        Command::Search { query } => {
            let query = query.join(" ");
            if query.trim().is_empty() {
                bail!("query is required");
            }
            let snapshot = read_local_search_snapshot(&cwd, &query)?;
            println!("search root: {}", snapshot.project_root);
            println!("initialized: {}", snapshot.initialized);
            println!("query: {}", snapshot.query.query);
            println!("results: {}", snapshot.results.len());
            println!("read only: {}", snapshot.boundary.read_only);
            println!("searched paths: {}", snapshot.searched_paths.len());
            for result in &snapshot.results {
                println!("- {}:{} [{}]", result.path, result.line, result.entity_kind);
                println!("  sourceType: {}", result.source_type);
                println!(
                    "  entityId: {}",
                    result.entity_id.as_deref().unwrap_or("none")
                );
                println!("  title: {}", result.title);
                println!("  field: {}", result.field);
                println!("  score: {}", result.score);
                println!("  snippet: {}", result.snippet);
            }
        }
        command => {
            let status = legacy_command_status(&command);
            print_legacy_retirement_message(&status);
        }
    }

    Ok(())
}
