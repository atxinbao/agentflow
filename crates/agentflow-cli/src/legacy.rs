//! Legacy CLI commands from archived 2026-05 workflow.
//!
//! These commands are kept for compatibility only.
//! New AgentFlow flows must not be added here.

use crate::args::{
    Cli, Command, FeatureCommand, GoalCommand, IndexCommand, IssueCommand, MilestoneCommand,
    ProjectCommand, StateCommand, TeamCommand, UpdateCommand, ViewCommand,
};
use crate::print::{
    print_controlled_run_plan, print_creation_summary, print_feature_execution_snapshot,
};
use agentflow_core::active::{
    read_local_metrics_snapshot, read_local_project_model_snapshot, read_local_search_snapshot,
};
use agentflow_core::legacy::eligibility_lease::{
    write_workflow_eligibility, write_workflow_lease_snapshot,
};
use agentflow_core::legacy::goal_protocol::{
    bootstrap_goal_protocol, check_goal_readiness, init_from_goal, write_goal_next,
};
use agentflow_core::legacy::product_feature::{
    create_product_feature, read_product_feature_execution_next,
    read_product_feature_execution_status, ProductFeatureDraft,
};
use agentflow_core::legacy::project_audit_docs_refresh::{
    write_project_code_audit_snapshot, write_project_docs_refresh_snapshot,
};
use agentflow_core::legacy::project_closure::write_project_closure_state;
use agentflow_core::legacy::run_verify_review::{
    plan_issue, review_issue, run_issue, verify_issue, write_context, write_project_summary,
    write_review_assistant,
};
use agentflow_core::legacy::saved_view::{save_view, show_view, SavedViewFilter};
use agentflow_core::legacy::sqlite_index::rebuild_index;
use agentflow_core::legacy::team_project_milestone_issue::{
    create_issue, create_milestone, create_project, create_team, read_issue_project_link_preview,
    read_local_project_seed_preview, write_issue_project_link, write_local_project_seed,
    IssueDraft, MilestoneDraft, ProjectDraft, TeamDraft,
};
use agentflow_core::legacy::workflow_control::write_workflow_state_check;
use anyhow::{bail, Result};
use clap::Parser;

pub(crate) fn run() -> Result<()> {
    let cli = Cli::parse();
    let cwd = std::env::current_dir()?;

    match cli.command {
        Command::Init { from_goal, force } => {
            let summary = init_from_goal(&cwd, &from_goal, force)?;
            println!("initialized {}", summary.project_dir.display());
            println!("wrote {}", summary.goal_json.display());
            println!("wrote {}", summary.index_json.display());
        }
        Command::Goal {
            command: GoalCommand::Bootstrap { force },
        } => {
            let summary = bootstrap_goal_protocol(&cwd, force)?;
            println!("bootstrapped goal protocol");
            println!("wrote {}", summary.project_definition_json.display());
            println!("wrote {}", summary.scope_state_json.display());
            println!("files written: {}", summary.files_written);
            println!("files checked: {}", summary.files_checked);
        }
        Command::Goal {
            command: GoalCommand::Check,
        } => {
            let summary = check_goal_readiness(&cwd)?;
            println!("goal ok: {}", summary.objective);
            println!("first candidate: {}", summary.first_candidate);
            println!("ready: {}", summary.ready);
            for check in &summary.checks {
                println!("- {} [{}] {}", check.name, check.status, check.path);
            }
            if !summary.ready {
                bail!("goal readiness failed; run `agentflow goal bootstrap`");
            }
        }
        Command::Goal {
            command: GoalCommand::Next,
        } => {
            let summary = write_goal_next(&cwd)?;
            println!("goal ready: {}", summary.goal_ready);
            println!(
                "active issue: {}",
                summary.active_issue_id.as_deref().unwrap_or("none")
            );
            println!("next action: {}", summary.next_action);
            println!("recommended intent: {}", summary.recommended_issue_intent);
            println!("recommended command: {}", summary.recommended_command);
            println!("wrote {}", summary.goal_loop_json.display());
            println!("wrote {}", summary.summary_path.display());
        }
        Command::Feature {
            command:
                FeatureCommand::Create {
                    goal,
                    team_id,
                    project_title,
                    non_goals,
                    success_criteria,
                    risk_level,
                    scope_boundaries,
                    write,
                    yes,
                },
        } => {
            let feature_goal = goal.join(" ");
            if feature_goal.trim().is_empty() {
                bail!("feature goal is required");
            }
            let draft = ProductFeatureDraft {
                project_title: project_title.unwrap_or_else(|| feature_goal.clone()),
                feature_goal,
                team_id,
                non_goals,
                success_criteria,
                risk_level,
                scope_boundaries,
            };
            let summary = create_product_feature(&cwd, draft, write, yes)?;
            println!("feature mode: {}", summary.snapshot.mode);
            println!("project: {}", summary.snapshot.project.id);
            println!("team: {}", summary.snapshot.project.team_id);
            println!(
                "active milestone: {}",
                summary.snapshot.project.active_milestone_id
            );
            println!("issues: {}", summary.snapshot.issues.len());
            for issue in &summary.snapshot.issues {
                println!(
                    "- {} [{}] status={} {}",
                    issue.id, issue.milestone_id, issue.status, issue.title
                );
            }
            println!(
                "recommended command: {}",
                summary.snapshot.recommended_command
            );
            if summary.written_paths.is_empty() {
                println!("preview only: no facts written");
            } else {
                for path in &summary.written_paths {
                    println!("wrote {}", path.display());
                }
            }
        }
        Command::Feature {
            command: FeatureCommand::Status,
        } => {
            let snapshot = read_product_feature_execution_status(&cwd)?;
            print_feature_execution_snapshot(&snapshot, true);
        }
        Command::Feature {
            command: FeatureCommand::Next,
        } => {
            let snapshot = read_product_feature_execution_next(&cwd)?;
            print_feature_execution_snapshot(&snapshot, false);
        }
        Command::Team {
            command:
                TeamCommand::Create {
                    name,
                    team_id,
                    write,
                    yes,
                },
        } => {
            let name = required_joined_arg(name, "team name")?;
            let summary = create_team(&cwd, TeamDraft { name, team_id }, write, yes)?;
            print_creation_summary(&summary);
        }
        Command::Project {
            command:
                ProjectCommand::Create {
                    title,
                    project_id,
                    team_id,
                    status,
                    goal,
                    write,
                    yes,
                },
        } => {
            let title = required_joined_arg(title, "project title")?;
            let summary = create_project(
                &cwd,
                ProjectDraft {
                    title,
                    project_id,
                    team_id,
                    status,
                    goal,
                },
                write,
                yes,
            )?;
            print_creation_summary(&summary);
        }
        Command::Milestone {
            command:
                MilestoneCommand::Create {
                    title,
                    milestone_id,
                    project_id,
                    description,
                    target,
                    write,
                    yes,
                },
        } => {
            let title = required_joined_arg(title, "milestone title")?;
            let summary = create_milestone(
                &cwd,
                MilestoneDraft {
                    title,
                    milestone_id,
                    project_id,
                    description,
                    target,
                },
                write,
                yes,
            )?;
            print_creation_summary(&summary);
        }
        Command::Issue {
            command:
                IssueCommand::Create {
                    title,
                    project_id,
                    milestone_id,
                    team_id,
                    risk_level,
                    scope,
                    non_goals,
                    validation_commands,
                    evidence_requirements,
                    rollback_plan,
                    write,
                    yes,
                },
        } => {
            let title = required_joined_arg(title, "issue title")?;
            let summary = create_issue(
                &cwd,
                IssueDraft {
                    title,
                    project_id,
                    milestone_id,
                    team_id,
                    risk_level,
                    scope,
                    non_goals,
                    validation_commands,
                    evidence_requirements,
                    rollback_plan,
                },
                write,
                yes,
            )?;
            print_creation_summary(&summary);
        }
        Command::Context => {
            let summary = write_context(&cwd)?;
            println!("context files: {}", summary.file_count);
            println!("wrote {}", summary.context_json.display());
            println!("wrote {}", summary.context_markdown.display());
        }
        Command::Plan { intent } => {
            let intent = intent.join(" ");
            if intent.trim().is_empty() {
                bail!("intent is required");
            }
            let summary = plan_issue(&cwd, &intent)?;
            println!("planned {}", summary.issue_id);
            println!("wrote {}", summary.issue_markdown.display());
            println!("wrote {}", summary.issue_json.display());
            if let Some(link) = &summary.project_link {
                println!(
                    "linked team={} project={} milestone={}",
                    link.team_id, link.project_id, link.milestone_id
                );
            }
            for path in &summary.updated_project_seed_paths {
                println!("updated {}", path.display());
            }
        }
        Command::Run { issue_id, dry_run } => {
            if !dry_run {
                bail!("agentflow run only supports --dry-run in Product Feature Controlled Run v0");
            }
            let summary = run_issue(&cwd, &issue_id)?;
            println!("started {} for {}", summary.run_id, issue_id);
            println!("mode: dry-run");
            println!(
                "project: {}",
                summary.run.project_id.as_deref().unwrap_or("none")
            );
            println!(
                "milestone: {}",
                summary.run.milestone_id.as_deref().unwrap_or("none")
            );
            println!(
                "lease: {}",
                summary.run.lease_id.as_deref().unwrap_or("none")
            );
            println!(
                "validation readiness: {}",
                if summary.run.run_plan.validation_commands.is_empty() {
                    "missing"
                } else {
                    "ready"
                }
            );
            print_controlled_run_plan(&summary.run.run_plan);
            println!("wrote {}", summary.run_json.display());
        }
        Command::Verify { issue_id } => {
            let summary = verify_issue(&cwd, &issue_id)?;
            println!("verified {} with {}", issue_id, summary.run_id);
            println!("passed: {}", summary.passed);
            println!("commands: {}", summary.commands.len());
        }
        Command::Review { issue_id } => {
            let summary = review_issue(&cwd, &issue_id)?;
            println!("reviewed {} with {}", issue_id, summary.run_id);
            println!("passed: {}", summary.passed);
            println!("wrote {}", summary.evidence_path.display());
            println!("wrote {}", summary.review_path.display());
            println!("wrote {}", summary.update_path.display());
        }
        Command::Index {
            command: IndexCommand::Rebuild,
        } => {
            let summary = rebuild_index(&cwd)?;
            println!("rebuilt {}", summary.sqlite_path.display());
            println!("issues: {}", summary.issue_count);
            println!("runs: {}", summary.run_count);
            println!("updates: {}", summary.update_count);
            println!("saved views: {}", summary.saved_view_count);
        }
        Command::View { command } => match command {
            ViewCommand::Save {
                name,
                issue_status,
                run_status,
                validation_status,
                issue_id,
            } => {
                let filter = SavedViewFilter {
                    issue_status,
                    run_status,
                    validation_status,
                    issue_id,
                };
                let summary = save_view(&cwd, &name, filter)?;
                println!("saved view {}", summary.view_id);
                println!("wrote {}", summary.view_path.display());
                println!("indexed {}", summary.sqlite_path.display());
            }
            ViewCommand::Show { name } => {
                let result = show_view(&cwd, &name)?;
                println!("view {} ({})", result.view.id, result.view.name);
                println!("issues: {}", result.issues.len());
                for issue in &result.issues {
                    println!("- {} [{}] {}", issue.id, issue.status, issue.title);
                }
                println!("runs: {}", result.runs.len());
                for run in &result.runs {
                    println!(
                        "- {} -> {} [{} / {}]",
                        run.id, run.issue_id, run.status, run.validation_status
                    );
                }
            }
        },
        Command::Update {
            command: UpdateCommand::Summary,
        } => {
            let summary = write_project_summary(&cwd)?;
            println!("wrote {}", summary.summary_path.display());
            println!(
                "issues: {} completed: {}",
                summary.issue_count, summary.completed_issue_count
            );
            println!("runs: {}", summary.run_count);
            println!("updates: {}", summary.update_count);
            println!("saved views: {}", summary.saved_view_count);
        }
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
        Command::Eligibility { issue_id } => {
            let summary = write_workflow_eligibility(&cwd, issue_id.as_deref())?;
            println!(
                "eligibility ready issues: {}",
                summary.snapshot.summary.ready_issue_count
            );
            println!(
                "eligible issues: {}",
                summary.snapshot.summary.eligible_issue_count
            );
            println!(
                "eligible issue: {}",
                summary
                    .snapshot
                    .eligible_issue_id
                    .as_deref()
                    .unwrap_or("none")
            );
            println!("next action: {}", summary.snapshot.summary.next_action);
            println!(
                "recommended command: {}",
                summary.snapshot.summary.recommended_command
            );
            for candidate in &summary.snapshot.candidates {
                println!(
                    "- {} eligible={} leased={} reasons={}",
                    candidate.issue_id,
                    candidate.eligible,
                    candidate.leased,
                    if candidate.failure_reasons.is_empty() {
                        "none".to_string()
                    } else {
                        candidate.failure_reasons.join(",")
                    }
                );
            }
            println!("wrote {}", summary.snapshot_path.display());
            println!("wrote {}", summary.summary_path.display());
        }
        Command::Lease => {
            let summary = write_workflow_lease_snapshot(&cwd)?;
            println!("active leases: {}", summary.snapshot.active_leases.len());
            println!("stale leases: {}", summary.snapshot.stale_leases.len());
            println!(
                "recommended command: {}",
                summary.snapshot.recommended_command
            );
            println!("wrote {}", summary.snapshot_path.display());
            println!("wrote {}", summary.summary_path.display());
        }
        Command::Project {
            command: ProjectCommand::Closure,
        } => {
            let summary = write_project_closure_state(&cwd)?;
            println!("project closure state: {}", summary.snapshot.closure_state);
            println!("can mark done: {}", summary.snapshot.can_mark_done);
            println!(
                "active project: {}",
                summary
                    .snapshot
                    .active_project_id
                    .as_deref()
                    .unwrap_or("none")
            );
            println!(
                "active milestone: {}",
                summary
                    .snapshot
                    .active_milestone_id
                    .as_deref()
                    .unwrap_or("none")
            );
            println!(
                "recommended command: {}",
                summary.snapshot.recommended_command
            );
            for gate in &summary.snapshot.gates {
                println!(
                    "- {} [{}] {}",
                    gate.id,
                    gate.status,
                    gate.path.as_deref().unwrap_or("none")
                );
            }
            if summary.snapshot.done_blocked_reasons.is_empty() {
                println!("done blockers: none");
            } else {
                println!(
                    "done blockers: {}",
                    summary.snapshot.done_blocked_reasons.len()
                );
                for reason in &summary.snapshot.done_blocked_reasons {
                    println!("- {}", reason);
                }
            }
            println!("wrote {}", summary.snapshot_path.display());
            println!("wrote {}", summary.summary_path.display());
        }
        Command::Project {
            command: ProjectCommand::CodeAudit,
        } => {
            let summary = write_project_code_audit_snapshot(&cwd)?;
            println!("project code audit state: {}", summary.snapshot.audit_state);
            println!("closure state: {}", summary.snapshot.closure_state);
            println!(
                "active project: {}",
                summary
                    .snapshot
                    .active_project_id
                    .as_deref()
                    .unwrap_or("none")
            );
            println!("source files: {}", summary.snapshot.counts.source_files);
            println!("findings: {}", summary.snapshot.counts.findings);
            println!("blockers: {}", summary.snapshot.counts.blockers);
            println!(
                "recommended command: {}",
                summary.snapshot.recommended_command
            );
            for check in &summary.snapshot.checks {
                println!(
                    "- {} [{}] candidates={}",
                    check.id, check.status, check.candidate_count
                );
            }
            println!("wrote {}", summary.snapshot_path.display());
            println!("wrote {}", summary.summary_path.display());
        }
        Command::Project {
            command: ProjectCommand::DocsRefresh,
        } => {
            let summary = write_project_docs_refresh_snapshot(&cwd)?;
            println!(
                "project docs refresh state: {}",
                summary.snapshot.docs_refresh_state
            );
            println!("closure state: {}", summary.snapshot.closure_state);
            println!(
                "active project: {}",
                summary
                    .snapshot
                    .active_project_id
                    .as_deref()
                    .unwrap_or("none")
            );
            println!("checked docs: {}", summary.snapshot.counts.checked_docs);
            println!(
                "update-needed docs: {}",
                summary.snapshot.counts.update_needed_docs
            );
            println!("missing docs: {}", summary.snapshot.counts.missing_docs);
            println!(
                "required updates: {}",
                summary.snapshot.counts.required_updates
            );
            println!("blockers: {}", summary.snapshot.counts.blockers);
            println!(
                "recommended command: {}",
                summary.snapshot.recommended_command
            );
            for doc in &summary.snapshot.checked_docs {
                println!("- {} [{}] {}", doc.path, doc.status, doc.category);
            }
            println!("wrote {}", summary.snapshot_path.display());
            println!("wrote {}", summary.summary_path.display());
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
        Command::ProjectSeed { write, yes } => {
            if write {
                let summary = write_local_project_seed(&cwd, yes)?;
                println!("project seed root: {}", summary.preview.project_root);
                println!("initialized: {}", summary.preview.initialized);
                println!("written files: {}", summary.written_paths.len());
                for path in &summary.written_paths {
                    println!("- wrote {}", path.display());
                }
                println!("confirmation: --write --yes");
            } else {
                let preview = read_local_project_seed_preview(&cwd)?;
                println!("project seed root: {}", preview.project_root);
                println!("initialized: {}", preview.initialized);
                println!("writes required: {}", preview.writes_required);
                println!("read only preview: {}", preview.boundary.read_only);
                println!("files: {}", preview.files.len());
                for file in &preview.files {
                    println!("- {} {} [{}]", file.action, file.path, file.kind);
                }
                println!("confirmation gates: {}", preview.confirmation_gates.len());
                for gate in &preview.confirmation_gates {
                    println!("- gate {}", gate);
                }
                println!("recommended write command: agentflow project-seed --write --yes");
            }
        }
        Command::IssueLink {
            issue_id,
            write,
            yes,
        } => {
            if write {
                let summary = write_issue_project_link(&cwd, &issue_id, yes)?;
                println!("issue link root: {}", summary.preview.project_root);
                println!("issue: {}", summary.preview.issue_id);
                println!("title: {}", summary.preview.issue_title);
                println!("action: written");
                println!(
                    "project link: team={} project={} milestone={} source={}",
                    summary.preview.project_link.team_id,
                    summary.preview.project_link.project_id,
                    summary.preview.project_link.milestone_id,
                    summary.preview.project_link.link_source
                );
                println!("written files: {}", summary.written_paths.len());
                for path in &summary.written_paths {
                    println!("- wrote {}", path.display());
                }
                println!("confirmation: --write --yes");
            } else {
                let preview = read_issue_project_link_preview(&cwd, &issue_id)?;
                println!("issue link root: {}", preview.project_root);
                println!("initialized: {}", preview.initialized);
                println!("issue: {}", preview.issue_id);
                println!("title: {}", preview.issue_title);
                println!("action: {}", preview.action);
                println!("writes required: {}", preview.writes_required);
                println!("read only preview: {}", preview.boundary.read_only);
                println!("json: {}", preview.issue_json_path);
                println!("markdown: {}", preview.issue_markdown_path);
                println!(
                    "project link: team={} project={} milestone={} source={}",
                    preview.project_link.team_id,
                    preview.project_link.project_id,
                    preview.project_link.milestone_id,
                    preview.project_link.link_source
                );
                println!("confirmation gates: {}", preview.confirmation_gates.len());
                for gate in &preview.confirmation_gates {
                    println!("- gate {}", gate);
                }
                println!(
                    "recommended write command: agentflow issue-link {} --write --yes",
                    preview.issue_id
                );
            }
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
        Command::ReviewAssistant { issue_id } => {
            let summary = write_review_assistant(&cwd, &issue_id)?;
            println!("review assistant {}", summary.issue_id);
            println!("ready: {}", summary.ready);
            println!("checks: {}", summary.checks.len());
            println!("wrote {}", summary.assistant_path.display());
        }
        Command::State {
            command: StateCommand::Check,
        } => {
            let summary = write_workflow_state_check(&cwd)?;
            println!("workflow state ready: {}", summary.snapshot.ready);
            println!(
                "counts: projects {} milestones {} issues {} errors {} warnings {}",
                summary.snapshot.counts.projects,
                summary.snapshot.counts.milestones,
                summary.snapshot.counts.issues,
                summary.snapshot.counts.errors,
                summary.snapshot.counts.warnings
            );
            println!("wrote {}", summary.snapshot_path.display());
            println!("wrote {}", summary.summary_path.display());
            if !summary.snapshot.ready {
                bail!("workflow state check failed");
            }
        }
    }

    Ok(())
}

fn required_joined_arg(values: Vec<String>, label: &str) -> Result<String> {
    let value = values.join(" ");
    if value.trim().is_empty() {
        bail!("{label} is required");
    }
    Ok(value)
}
