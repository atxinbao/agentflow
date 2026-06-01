//! Legacy CLI argument definitions.
//!
//! The command names are preserved for compatibility while the implementations
//! remain isolated in `legacy.rs`.

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "agentflow")]
#[command(about = "Local-first AI engineering execution spine")]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Command,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Command {
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
pub(crate) enum GoalCommand {
    Bootstrap {
        #[arg(long)]
        force: bool,
    },
    Check,
    Next,
}

#[derive(Debug, Subcommand)]
pub(crate) enum FeatureCommand {
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
pub(crate) enum TeamCommand {
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
pub(crate) enum MilestoneCommand {
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
pub(crate) enum IssueCommand {
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
pub(crate) enum IndexCommand {
    Rebuild,
}

#[derive(Debug, Subcommand)]
pub(crate) enum ViewCommand {
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
pub(crate) enum UpdateCommand {
    Summary,
}

#[derive(Debug, Subcommand)]
pub(crate) enum StateCommand {
    Check,
}

#[derive(Debug, Subcommand)]
pub(crate) enum ProjectCommand {
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
