//! Legacy CLI command gate.
//!
//! 006 retires archived 2026-05 write commands. Command names still parse so
//! users receive a clear migration message, but old writers are no longer
//! executed from the CLI.

use crate::active::{
    claim_next_build_agent_launch, complete_build_agent_issue_from_request,
    prepare_build_agent_review_from_request, start_build_agent_issue,
    write_build_agent_merge_proof,
};
use crate::args::{
    AgentBridgeCommand, BuildAgentCommand, Cli, Command, ProjectionCommand, ReleaseCommand,
    TaskLoopCommand,
};
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
        Command::BuildAgent { command } => match command {
            BuildAgentCommand::Start { issue_id } => {
                let start = start_build_agent_issue(&cwd, &issue_id)?;
                println!("build agent start: ready");
                println!("issue: {}", start.issue_id);
                println!("run: {}", start.run_id);
                println!("stage: {}", start.stage);
                println!("branch: {}", start.branch_name.as_deref().unwrap_or("none"));
                println!("project: {}", start.project_id.as_deref().unwrap_or("none"));
            }
            BuildAgentCommand::ClaimLaunch => {
                if let Some(claim) = claim_next_build_agent_launch(&cwd)? {
                    println!("build agent launch: claimed");
                    println!("event: {}", claim.event_id);
                    println!("issue: {}", claim.issue_id);
                    println!("run: {}", claim.run_id);
                    println!("branch: {}", claim.branch_name.as_deref().unwrap_or("none"));
                    println!("request: {}", claim.launch_request_path.display());
                } else {
                    println!("build agent launch: none");
                }
            }
            BuildAgentCommand::PrepareReview { request } => {
                let prepared = prepare_build_agent_review_from_request(&cwd, &request)?;
                println!("build agent review: prepared");
                println!("issue: {}", prepared.issue_id);
                println!("run: {}", prepared.run_id);
                println!("run status: {}", prepared.run_status);
                println!("evidence: {}", prepared.evidence_path.display());
                println!("validation passed: {}", prepared.validation_passed);
            }
            BuildAgentCommand::WriteMergeProof {
                issue_id,
                run_id,
                provider,
                merge_mode,
                remote_url,
                merged,
            } => {
                let proof = write_build_agent_merge_proof(
                    &cwd,
                    &issue_id,
                    &run_id,
                    &provider,
                    &merge_mode,
                    remote_url,
                    merged,
                )?;
                println!("build agent merge proof: recorded");
                println!("issue: {}", proof.issue_id);
                println!("run: {}", proof.run_id);
                println!("merged: {}", proof.merged);
                println!("path: {}", proof.proof_path.display());
            }
            BuildAgentCommand::Complete { request } => {
                let outcome = complete_build_agent_issue_from_request(&cwd, &request)?;
                println!("build agent completion: done");
                println!("issue: {}", outcome.issue_id);
                println!("run: {}", outcome.run_id);
                println!("run status: {}", outcome.run_status);
                println!("evidence: {}", outcome.evidence_path.display());
                println!("changelog: {}", outcome.changelog_path.display());
                println!("release notes: {}", outcome.release_notes_path.display());
                println!("validation passed: {}", outcome.validation_passed);
                if let Some(next_launch) = outcome.next_launch {
                    println!("next issue: {}", next_launch.issue_id);
                    println!("next run: {}", next_launch.run_id);
                    println!("next stage: in_progress");
                    println!("next request: {}", next_launch.launch_request_path);
                }
            }
        },
        Command::TaskLoop { command } => match command {
            TaskLoopCommand::Schedule { project_id } => {
                let schedule =
                    agentflow_task_loop::TaskLoop::new(&project_id).schedule_next_issue(&cwd)?;
                if let Some(schedule) = schedule {
                    println!("task loop schedule: created");
                    println!("project: {}", schedule.project_id);
                    println!("issue: {}", schedule.issue_id);
                    println!("workflow: {}", schedule.workflow_ref);
                    println!("event: {}", schedule.event_id);
                } else {
                    println!("task loop schedule: none");
                    println!("project: {}", project_id);
                }
            }
            TaskLoopCommand::Launch {
                project_id,
                issue_id,
                provider,
            } => {
                let launch = agentflow_task_loop::TaskLoop::new(&project_id)
                    .request_agent_launch(&cwd, &issue_id, &provider)?;
                println!("task loop launch: requested");
                println!(
                    "project: {}",
                    launch.project_id.as_deref().unwrap_or("none")
                );
                println!("issue: {}", launch.issue_id);
                println!("run: {}", launch.run_id);
                println!("branch: {}", launch.branch_name);
                println!("request: {}", launch.launch_request_path);
                println!("event: {}", launch.event_id);
            }
            TaskLoopCommand::Tick {
                project_id,
                provider,
            } => {
                let loop_driver = agentflow_task_loop::TaskLoop::new(&project_id);
                let tick = loop_driver.tick(&cwd, &provider)?;
                if let Some(tick) = tick {
                    println!("task loop tick: launched");
                    println!(
                        "project: {}",
                        tick.launch.project_id.as_deref().unwrap_or("none")
                    );
                    println!("issue: {}", tick.launch.issue_id);
                    if let Some(schedule) = tick.schedule {
                        println!("workflow: {}", schedule.workflow_ref);
                        println!("schedule event: {}", schedule.event_id);
                    } else {
                        println!("schedule event: reused existing todo");
                    }
                    println!("run: {}", tick.launch.run_id);
                    println!("branch: {}", tick.launch.branch_name);
                    println!("request: {}", tick.launch.launch_request_path);
                    println!("launch event: {}", tick.launch.event_id);
                } else {
                    println!("task loop tick: none");
                    println!("project: {}", project_id);
                }
            }
        },
        Command::AgentBridge { command } => match command {
            AgentBridgeCommand::ClaimNext => {
                let claim = agentflow_agent_bridge::AgentBridge::with_default_providers()
                    .claim_next_launch(&cwd)?;
                if let Some(claim) = claim {
                    println!("agent bridge claim: created");
                    println!("issue: {}", claim.issue_id);
                    println!("run: {}", claim.run_id);
                    println!("provider: {}", claim.provider);
                    println!("session: {}", claim.session_id);
                    println!("status: {}", claim.session_status);
                    println!("event: {}", claim.created_event_id);
                } else {
                    println!("agent bridge claim: none");
                }
            }
        },
        Command::Projection { command } => match command {
            ProjectionCommand::Rebuild => {
                let summary = agentflow_projection::rebuild_projections(&cwd)?;
                println!("projection rebuild: done");
                println!("tasks: {}", summary.task_count);
                println!("projects: {}", summary.project_count);
                println!("index: {}", summary.index_path);
            }
            ProjectionCommand::Task { issue_id } => {
                let projection = agentflow_projection::load_task_projection(&cwd, &issue_id)?;
                println!("{}", serde_json::to_string_pretty(&projection)?);
            }
            ProjectionCommand::Project { project_id } => {
                let projection = agentflow_projection::load_project_projection(&cwd, &project_id)?;
                println!("{}", serde_json::to_string_pretty(&projection)?);
            }
        },
        Command::Release { command } => match command {
            ReleaseCommand::Summary => {
                let summary = agentflow_release::collect_public_release_summary(&cwd)?;
                println!("release summary: generated");
                println!("entries: {}", summary.entries.len());
                println!("{}", summary.changelog_markdown);
            }
            ReleaseCommand::WriteDocs {
                changelog_path,
                release_notes_path,
            } => {
                let summary = agentflow_release::collect_public_release_summary(&cwd)?;
                let target = agentflow_release::PublicReleaseDocumentTarget {
                    changelog_path,
                    release_notes_path,
                };
                let paths =
                    agentflow_release::write_public_release_documents(&cwd, &summary, &target)?;
                println!("release docs: written");
                println!("changelog: {}", paths.changelog_path);
                println!("release notes: {}", paths.release_notes_path);
                println!("entries: {}", summary.entries.len());
            }
        },
        command => {
            let status = legacy_command_status(&command);
            print_legacy_retirement_message(&status);
        }
    }

    Ok(())
}
