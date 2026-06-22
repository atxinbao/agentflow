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
    Project {
        #[command(subcommand)]
        command: ProjectCommand,
    },
    Audit {
        #[command(subcommand)]
        command: AuditCommand,
    },
    Completion {
        #[command(subcommand)]
        command: CompletionCommand,
    },
    BuildAgent {
        #[command(subcommand)]
        command: BuildAgentCommand,
    },
    TaskLoop {
        #[command(subcommand)]
        command: TaskLoopCommand,
    },
    AgentDispatcher {
        #[command(subcommand)]
        command: AgentDispatcherCommand,
    },
    Projection {
        #[command(subcommand)]
        command: ProjectionCommand,
    },
    ApiPlane {
        #[command(subcommand)]
        command: ApiPlaneCommand,
    },
    Release {
        #[command(subcommand)]
        command: ReleaseCommand,
    },
    ProviderSmoke {
        #[arg(long, default_value = "codex")]
        provider: String,
        #[arg(long = "issue-id")]
        issue_id: String,
        #[arg(long = "run-id")]
        run_id: String,
        #[arg(long = "working-directory")]
        working_directory: Option<PathBuf>,
        #[arg(
            long = "launch-request-path",
            default_value = ".agentflow/tmp/provider-smoke-request.md"
        )]
        launch_request_path: String,
    },
}

#[derive(Debug, Subcommand)]
pub(crate) enum ProjectCommand {
    Intake {
        #[arg(long = "requirement-path")]
        requirement_path: PathBuf,
        #[arg(long = "project-id")]
        project_id: Option<String>,
    },
    PreviewGoal {
        #[arg(long = "requirement-id")]
        requirement_id: String,
    },
    ConfirmGoal {
        #[arg(long = "requirement-id")]
        requirement_id: String,
        #[arg(long, default_value = "goal-agent")]
        actor: String,
    },
    ConfirmPlan {
        #[arg(long = "requirement-id")]
        requirement_id: String,
        #[arg(long, default_value = "goal-agent")]
        actor: String,
    },
    Materialize {
        #[arg(long = "requirement-id")]
        requirement_id: String,
    },
}

#[derive(Debug, Subcommand)]
pub(crate) enum CompletionCommand {
    Inspect {
        #[arg(long = "project-id")]
        project_id: String,
    },
    Decide {
        #[arg(long = "project-id")]
        project_id: String,
        #[arg(long)]
        outcome: String,
        #[arg(long, default_value = "goal-agent")]
        actor: String,
        #[arg(long)]
        summary: String,
        #[arg(long = "rationale")]
        rationale: Vec<String>,
    },
}

#[derive(Debug, Subcommand)]
pub(crate) enum AuditCommand {
    RequestHuman {
        #[arg(long = "run-id")]
        run_id: String,
        #[arg(long = "issue-id")]
        issue_id: Option<String>,
        #[arg(long)]
        reason: String,
        #[arg(long = "public-delivery-path", default_value = "CHANGELOG.md")]
        public_delivery_path: String,
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
    WriteCloseoutProof {
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
        #[arg(long = "provider-issue-ref")]
        provider_issue_refs: Vec<String>,
        #[arg(long = "attestation-path")]
        attestation_path: Option<PathBuf>,
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
pub(crate) enum AgentDispatcherCommand {
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
pub(crate) enum ApiPlaneCommand {
    Manifest {
        #[arg(long)]
        output: Option<PathBuf>,
    },
}

#[derive(Debug, Subcommand)]
pub(crate) enum ReleaseCommand {
    Prepare {
        #[arg(long = "project-id")]
        project_id: String,
    },
    Confirm {
        #[arg(long = "project-id")]
        project_id: String,
    },
    RecordTag {
        #[arg(long = "project-id")]
        project_id: String,
        #[arg(long = "tag-name")]
        tag_name: String,
        #[arg(long = "tag-commit-sha")]
        tag_commit_sha: String,
        #[arg(long, default_value = "release-agent")]
        actor: String,
    },
    RecordRemote {
        #[arg(long = "project-id")]
        project_id: String,
        #[arg(long)]
        provider: String,
        #[arg(long = "release-id")]
        release_id: String,
        #[arg(long = "release-url")]
        release_url: String,
        #[arg(long = "tag-name")]
        tag_name: String,
        #[arg(long = "release-commit-sha")]
        release_commit_sha: String,
        #[arg(long = "artifact-manifest-path")]
        artifact_manifest_path: String,
        #[arg(long, default_value = "release-agent")]
        actor: String,
    },
    Publish {
        #[arg(long = "project-id")]
        project_id: String,
    },
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
