use anyhow::{bail, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "agentflow")]
#[command(about = "Local-first AI engineering execution spine")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Init {
        #[arg(long = "from-goal")]
        from_goal: PathBuf,
        #[arg(long)]
        force: bool,
    },
    Goal {
        #[command(subcommand)]
        command: GoalCommand,
    },
    Feature {
        #[command(subcommand)]
        command: FeatureCommand,
    },
    Team {
        #[command(subcommand)]
        command: TeamCommand,
    },
    Milestone {
        #[command(subcommand)]
        command: MilestoneCommand,
    },
    Issue {
        #[command(subcommand)]
        command: IssueCommand,
    },
    Context,
    Plan {
        intent: Vec<String>,
    },
    Run {
        issue_id: String,
        #[arg(long)]
        dry_run: bool,
    },
    Verify {
        issue_id: String,
    },
    Review {
        issue_id: String,
    },
    Index {
        #[command(subcommand)]
        command: IndexCommand,
    },
    View {
        #[command(subcommand)]
        command: ViewCommand,
    },
    Update {
        #[command(subcommand)]
        command: UpdateCommand,
    },
    Metrics,
    Eligibility {
        issue_id: Option<String>,
    },
    Lease,
    Project {
        #[command(subcommand)]
        command: ProjectCommand,
    },
    Projects,
    ProjectSeed {
        #[arg(long)]
        write: bool,
        #[arg(long)]
        yes: bool,
    },
    IssueLink {
        issue_id: String,
        #[arg(long)]
        write: bool,
        #[arg(long)]
        yes: bool,
    },
    Search {
        query: Vec<String>,
    },
    ReviewAssistant {
        issue_id: String,
    },
    State {
        #[command(subcommand)]
        command: StateCommand,
    },
}

#[derive(Debug, Subcommand)]
enum GoalCommand {
    Bootstrap {
        #[arg(long)]
        force: bool,
    },
    Check,
    Next,
}

#[derive(Debug, Subcommand)]
enum FeatureCommand {
    Create {
        goal: Vec<String>,
        #[arg(long = "team-id", default_value = "core")]
        team_id: String,
        #[arg(long = "project-title")]
        project_title: Option<String>,
        #[arg(long = "non-goal")]
        non_goals: Vec<String>,
        #[arg(long = "success-criterion")]
        success_criteria: Vec<String>,
        #[arg(long = "risk-level", default_value = "medium")]
        risk_level: String,
        #[arg(long = "scope-boundary")]
        scope_boundaries: Vec<String>,
        #[arg(long)]
        write: bool,
        #[arg(long)]
        yes: bool,
    },
    Status,
    Next,
}

#[derive(Debug, Subcommand)]
enum TeamCommand {
    Create {
        name: Vec<String>,
        #[arg(long = "team-id")]
        team_id: Option<String>,
        #[arg(long)]
        write: bool,
        #[arg(long)]
        yes: bool,
    },
}

#[derive(Debug, Subcommand)]
enum MilestoneCommand {
    Create {
        title: Vec<String>,
        #[arg(long = "milestone-id")]
        milestone_id: Option<String>,
        #[arg(long = "project-id")]
        project_id: Option<String>,
        #[arg(long)]
        description: Option<String>,
        #[arg(long)]
        target: Option<String>,
        #[arg(long)]
        write: bool,
        #[arg(long)]
        yes: bool,
    },
}

#[derive(Debug, Subcommand)]
enum IssueCommand {
    Create {
        title: Vec<String>,
        #[arg(long = "project-id")]
        project_id: Option<String>,
        #[arg(long = "milestone-id")]
        milestone_id: Option<String>,
        #[arg(long = "team-id")]
        team_id: Option<String>,
        #[arg(long = "risk-level", default_value = "medium")]
        risk_level: String,
        #[arg(long = "scope")]
        scope: Vec<String>,
        #[arg(long = "non-goal")]
        non_goals: Vec<String>,
        #[arg(long = "validation-command")]
        validation_commands: Vec<String>,
        #[arg(long = "evidence-requirement")]
        evidence_requirements: Vec<String>,
        #[arg(long = "rollback-plan")]
        rollback_plan: Vec<String>,
        #[arg(long)]
        write: bool,
        #[arg(long)]
        yes: bool,
    },
}

#[derive(Debug, Subcommand)]
enum IndexCommand {
    Rebuild,
}

#[derive(Debug, Subcommand)]
enum ViewCommand {
    Save {
        name: String,
        #[arg(long)]
        issue_status: Option<String>,
        #[arg(long)]
        run_status: Option<String>,
        #[arg(long)]
        validation_status: Option<String>,
        #[arg(long)]
        issue_id: Option<String>,
    },
    Show {
        name: String,
    },
}

#[derive(Debug, Subcommand)]
enum UpdateCommand {
    Summary,
}

#[derive(Debug, Subcommand)]
enum StateCommand {
    Check,
}

#[derive(Debug, Subcommand)]
enum ProjectCommand {
    Create {
        title: Vec<String>,
        #[arg(long = "project-id")]
        project_id: Option<String>,
        #[arg(long = "team-id")]
        team_id: Option<String>,
        #[arg(long, default_value = "draft")]
        status: String,
        #[arg(long)]
        goal: Option<String>,
        #[arg(long)]
        write: bool,
        #[arg(long)]
        yes: bool,
    },
    Closure,
    CodeAudit,
    DocsRefresh,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let cwd = std::env::current_dir()?;

    match cli.command {
        Command::Init { from_goal, force } => {
            let summary = agentflow_core::init_from_goal(&cwd, &from_goal, force)?;
            println!("initialized {}", summary.project_dir.display());
            println!("wrote {}", summary.goal_json.display());
            println!("wrote {}", summary.index_json.display());
        }
        Command::Goal {
            command: GoalCommand::Bootstrap { force },
        } => {
            let summary = agentflow_core::bootstrap_goal_protocol(&cwd, force)?;
            println!("bootstrapped goal protocol");
            println!("wrote {}", summary.project_definition_json.display());
            println!("wrote {}", summary.scope_state_json.display());
            println!("files written: {}", summary.files_written);
            println!("files checked: {}", summary.files_checked);
        }
        Command::Goal {
            command: GoalCommand::Check,
        } => {
            let summary = agentflow_core::check_goal_readiness(&cwd)?;
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
            let summary = agentflow_core::write_goal_next(&cwd)?;
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
            let draft = agentflow_core::ProductFeatureDraft {
                project_title: project_title.unwrap_or_else(|| feature_goal.clone()),
                feature_goal,
                team_id,
                non_goals,
                success_criteria,
                risk_level,
                scope_boundaries,
            };
            let summary = agentflow_core::create_product_feature(&cwd, draft, write, yes)?;
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
            let snapshot = agentflow_core::read_product_feature_execution_status(&cwd)?;
            print_feature_execution_snapshot(&snapshot, true);
        }
        Command::Feature {
            command: FeatureCommand::Next,
        } => {
            let snapshot = agentflow_core::read_product_feature_execution_next(&cwd)?;
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
            let summary = agentflow_core::create_team(
                &cwd,
                agentflow_core::TeamDraft { name, team_id },
                write,
                yes,
            )?;
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
            let summary = agentflow_core::create_project(
                &cwd,
                agentflow_core::ProjectDraft {
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
            let summary = agentflow_core::create_milestone(
                &cwd,
                agentflow_core::MilestoneDraft {
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
            let summary = agentflow_core::create_issue(
                &cwd,
                agentflow_core::IssueDraft {
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
            let summary = agentflow_core::write_context(&cwd)?;
            println!("context files: {}", summary.file_count);
            println!("wrote {}", summary.context_json.display());
            println!("wrote {}", summary.context_markdown.display());
        }
        Command::Plan { intent } => {
            let intent = intent.join(" ");
            if intent.trim().is_empty() {
                bail!("intent is required");
            }
            let summary = agentflow_core::plan_issue(&cwd, &intent)?;
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
            let summary = agentflow_core::run_issue(&cwd, &issue_id)?;
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
            let summary = agentflow_core::verify_issue(&cwd, &issue_id)?;
            println!("verified {} with {}", issue_id, summary.run_id);
            println!("passed: {}", summary.passed);
            println!("commands: {}", summary.commands.len());
        }
        Command::Review { issue_id } => {
            let summary = agentflow_core::review_issue(&cwd, &issue_id)?;
            println!("reviewed {} with {}", issue_id, summary.run_id);
            println!("passed: {}", summary.passed);
            println!("wrote {}", summary.evidence_path.display());
            println!("wrote {}", summary.review_path.display());
            println!("wrote {}", summary.update_path.display());
        }
        Command::Index {
            command: IndexCommand::Rebuild,
        } => {
            let summary = agentflow_core::rebuild_index(&cwd)?;
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
                let filter = agentflow_core::SavedViewFilter {
                    issue_status,
                    run_status,
                    validation_status,
                    issue_id,
                };
                let summary = agentflow_core::save_view(&cwd, &name, filter)?;
                println!("saved view {}", summary.view_id);
                println!("wrote {}", summary.view_path.display());
                println!("indexed {}", summary.sqlite_path.display());
            }
            ViewCommand::Show { name } => {
                let result = agentflow_core::show_view(&cwd, &name)?;
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
            let summary = agentflow_core::write_project_summary(&cwd)?;
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
            let metrics = agentflow_core::read_local_metrics_snapshot(&cwd)?;
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
            let summary = agentflow_core::write_workflow_eligibility(&cwd, issue_id.as_deref())?;
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
            let summary = agentflow_core::write_workflow_lease_snapshot(&cwd)?;
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
            let summary = agentflow_core::write_project_closure_state(&cwd)?;
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
            let summary = agentflow_core::write_project_code_audit_snapshot(&cwd)?;
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
            let summary = agentflow_core::write_project_docs_refresh_snapshot(&cwd)?;
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
            let snapshot = agentflow_core::read_local_project_model_snapshot(&cwd)?;
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
                let summary = agentflow_core::write_local_project_seed(&cwd, yes)?;
                println!("project seed root: {}", summary.preview.project_root);
                println!("initialized: {}", summary.preview.initialized);
                println!("written files: {}", summary.written_paths.len());
                for path in &summary.written_paths {
                    println!("- wrote {}", path.display());
                }
                println!("confirmation: --write --yes");
            } else {
                let preview = agentflow_core::read_local_project_seed_preview(&cwd)?;
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
                let summary = agentflow_core::write_issue_project_link(&cwd, &issue_id, yes)?;
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
                let preview = agentflow_core::read_issue_project_link_preview(&cwd, &issue_id)?;
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
            let snapshot = agentflow_core::read_local_search_snapshot(&cwd, &query)?;
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
            let summary = agentflow_core::write_review_assistant(&cwd, &issue_id)?;
            println!("review assistant {}", summary.issue_id);
            println!("ready: {}", summary.ready);
            println!("checks: {}", summary.checks.len());
            println!("wrote {}", summary.assistant_path.display());
        }
        Command::State {
            command: StateCommand::Check,
        } => {
            let summary = agentflow_core::write_workflow_state_check(&cwd)?;
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

fn print_feature_execution_snapshot(
    snapshot: &agentflow_core::ProductFeatureExecutionSnapshot,
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

fn print_controlled_run_plan(plan: &agentflow_core::ControlledRunPlan) {
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

fn print_creation_summary(summary: &agentflow_core::CreationWriteSummary) {
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

fn required_joined_arg(values: Vec<String>, label: &str) -> Result<String> {
    let value = values.join(" ");
    if value.trim().is_empty() {
        bail!("{label} is required");
    }
    Ok(value)
}

fn print_list_line(label: &str, values: &[String], separator: &str) {
    if values.is_empty() {
        println!("{label}: none");
    } else {
        println!("{label}: {}", values.join(separator));
    }
}
