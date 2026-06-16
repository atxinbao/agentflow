//! AgentFlow CLI argument definitions.

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
    BuildAgent {
        #[command(subcommand)]
        command: BuildAgentCommand,
    },
    TaskLoop {
        #[command(subcommand)]
        command: TaskLoopCommand,
    },
    AgentBridge {
        #[command(subcommand)]
        command: AgentBridgeCommand,
    },
    Projection {
        #[command(subcommand)]
        command: ProjectionCommand,
    },
    Release {
        #[command(subcommand)]
        command: ReleaseCommand,
    },
}

#[derive(Debug, Subcommand)]
pub(crate) enum BuildAgentCommand {
    Start {
        #[arg(long = "issue-id")]
        issue_id: String,
    },
    ClaimLaunch,
    PrepareReview {
        #[arg(long)]
        request: PathBuf,
    },
    WriteMergeProof {
        #[arg(long = "issue-id")]
        issue_id: String,
        #[arg(long = "run-id")]
        run_id: String,
        #[arg(long)]
        provider: String,
        #[arg(long = "merge-mode")]
        merge_mode: String,
        #[arg(long = "remote-url")]
        remote_url: Option<String>,
        #[arg(long)]
        merged: bool,
    },
    Complete {
        #[arg(long)]
        request: PathBuf,
    },
}

#[derive(Debug, Subcommand)]
pub(crate) enum TaskLoopCommand {
    Schedule {
        #[arg(long = "project-id")]
        project_id: String,
    },
    Launch {
        #[arg(long = "project-id")]
        project_id: String,
        #[arg(long = "issue-id")]
        issue_id: String,
        #[arg(long, default_value = "codex")]
        provider: String,
    },
    Tick {
        #[arg(long = "project-id")]
        project_id: String,
        #[arg(long, default_value = "codex")]
        provider: String,
    },
}

#[derive(Debug, Subcommand)]
pub(crate) enum AgentBridgeCommand {
    ClaimNext,
}

#[derive(Debug, Subcommand)]
pub(crate) enum ProjectionCommand {
    Rebuild,
    Task {
        #[arg(long = "issue-id")]
        issue_id: String,
    },
    Project {
        #[arg(long = "project-id")]
        project_id: String,
    },
}

#[derive(Debug, Subcommand)]
pub(crate) enum ReleaseCommand {
    Summary,
    WriteDocs {
        #[arg(long = "changelog-path", default_value = "CHANGELOG.md")]
        changelog_path: std::path::PathBuf,
        #[arg(
            long = "release-notes-path",
            default_value = "docs/release-notes/agentflow-release-notes.md"
        )]
        release_notes_path: std::path::PathBuf,
    },
}
