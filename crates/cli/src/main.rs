mod active;
mod args;
mod formal;

use active::{
    claim_next_build_agent_launch, complete_build_agent_issue_from_request,
    prepare_build_agent_review_from_request, start_build_agent_issue,
    write_build_agent_closeout_proof,
};
use args::{
    AgentDispatcherCommand, ApiPlaneCommand, AuditCommand, BuildAgentCommand,
    CapabilityRegistryCommand, Cli, Command, CompletionCommand, PackCommand, ProjectCommand,
    ProjectionCommand, ReleaseCommand, TaskLoopCommand,
};
use clap::Parser;
use formal::{
    audit_request_human, completion_decide, completion_inspect, project_confirm_goal,
    project_confirm_plan, project_intake, project_materialize, project_preview_goal,
    release_confirm, release_prepare, release_publish, release_record_remote, release_record_tag,
};

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let cwd = std::env::current_dir()?;

    match cli.command {
        Command::Project { command } => match command {
            ProjectCommand::Intake {
                requirement_path,
                project_id,
            } => {
                let preview = project_intake(&cwd, &requirement_path, project_id.as_deref())?;
                println!("{}", serde_json::to_string_pretty(&preview)?);
            }
            ProjectCommand::PreviewGoal { requirement_id } => {
                let preview = project_preview_goal(&cwd, &requirement_id)?;
                println!("{}", serde_json::to_string_pretty(&preview)?);
            }
            ProjectCommand::ConfirmGoal {
                requirement_id,
                actor,
            } => {
                let preview = project_confirm_goal(&cwd, &requirement_id, &actor)?;
                println!("{}", serde_json::to_string_pretty(&preview)?);
            }
            ProjectCommand::ConfirmPlan {
                requirement_id,
                actor,
            } => {
                let preview = project_confirm_plan(&cwd, &requirement_id, &actor)?;
                println!("{}", serde_json::to_string_pretty(&preview)?);
            }
            ProjectCommand::Materialize { requirement_id } => {
                let result = project_materialize(&cwd, &requirement_id)?;
                println!("{}", serde_json::to_string_pretty(&result)?);
            }
        },
        Command::Audit { command } => match command {
            AuditCommand::RequestHuman {
                run_id,
                issue_id,
                reason,
                public_delivery_path,
            } => {
                let report = audit_request_human(
                    &cwd,
                    &run_id,
                    issue_id.as_deref(),
                    &reason,
                    &public_delivery_path,
                )?;
                println!("{}", serde_json::to_string_pretty(&report)?);
            }
        },
        Command::Completion { command } => match command {
            CompletionCommand::Inspect { project_id } => {
                let runtime = completion_inspect(&cwd, &project_id)?;
                println!("{}", serde_json::to_string_pretty(&runtime)?);
            }
            CompletionCommand::Decide {
                project_id,
                outcome,
                actor,
                summary,
                rationale,
            } => {
                let runtime =
                    completion_decide(&cwd, &project_id, &outcome, &actor, &summary, rationale)?;
                println!("{}", serde_json::to_string_pretty(&runtime)?);
            }
        },
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
            BuildAgentCommand::WriteCloseoutProof {
                issue_id,
                run_id,
                provider,
                merge_mode,
                remote_url,
                provider_issue_refs,
                attestation_path,
            } => {
                let proof = write_build_agent_closeout_proof(
                    &cwd,
                    &issue_id,
                    &run_id,
                    &provider,
                    &merge_mode,
                    remote_url,
                    provider_issue_refs,
                    attestation_path.as_deref(),
                )?;
                println!("build agent closeout proof: recorded");
                println!("issue: {}", proof.issue_id);
                println!("run: {}", proof.run_id);
                println!("merged: {}", proof.merged);
                println!("issue closed: {}", proof.issue_closed);
                println!("path: {}", proof.proof_path.display());
            }
            BuildAgentCommand::Complete { request } => {
                let outcome = complete_build_agent_issue_from_request(&cwd, &request)?;
                println!("build agent completion: done");
                println!("issue: {}", outcome.issue_id);
                println!("run: {}", outcome.run_id);
                println!("run status: {}", outcome.run_status);
                println!("evidence: {}", outcome.evidence_path.display());
                println!("closeout proof: {}", outcome.closeout_proof_path.display());
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
        Command::AgentDispatcher { command } => match command {
            AgentDispatcherCommand::ClaimNext => {
                let claim = agentflow_agent_dispatcher::AgentDispatcher::with_default_providers()
                    .claim_next_launch(&cwd)?;
                if let Some(claim) = claim {
                    println!("agent dispatcher claim: created");
                    println!("issue: {}", claim.issue_id);
                    println!("run: {}", claim.run_id);
                    println!("provider: {}", claim.provider);
                    println!("session: {}", claim.session_id);
                    println!("status: {}", claim.session_status);
                    println!("event: {}", claim.created_event_id);
                } else {
                    println!("agent dispatcher claim: none");
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
        Command::ApiPlane { command } => match command {
            ApiPlaneCommand::Manifest { output } => {
                let manifest = agentflow_runtime_api::api_plane_manifest();
                if let Some(output) = output {
                    agentflow_runtime_api::write_api_plane_manifest(output, &manifest)?;
                } else {
                    println!("{}", serde_json::to_string_pretty(&manifest)?);
                }
            }
        },
        Command::CapabilityRegistry { command } => match command {
            CapabilityRegistryCommand::Manifest { output } => {
                let registry = agentflow_capability_registry::default_capability_registry();
                if let Some(output) = output {
                    if let Some(parent) = output.parent() {
                        std::fs::create_dir_all(parent)?;
                    }
                    std::fs::write(output, serde_json::to_string_pretty(&registry)? + "\n")?;
                } else {
                    println!("{}", serde_json::to_string_pretty(&registry)?);
                }
            }
        },
        Command::Pack { command } => match command {
            PackCommand::Registry { output } => {
                let registry = agentflow_runtime_api::get_pack_registry(&cwd)?;
                if let Some(output) = output {
                    if let Some(parent) = output.parent() {
                        std::fs::create_dir_all(parent)?;
                    }
                    std::fs::write(output, serde_json::to_string_pretty(&registry)? + "\n")?;
                } else {
                    println!("{}", serde_json::to_string_pretty(&registry)?);
                }
            }
            PackCommand::ValidateManifest { manifest_path } => {
                let manifest = agentflow_pack::load_pack_manifest(manifest_path)?;
                let report = agentflow_pack::validate_pack_manifest(&manifest);
                println!("{}", serde_json::to_string_pretty(&report)?);
            }
        },
        Command::Release { command } => match command {
            ReleaseCommand::Prepare { project_id } => {
                let facts = release_prepare(&cwd, &project_id)?;
                println!("{}", serde_json::to_string_pretty(&facts)?);
            }
            ReleaseCommand::Confirm { project_id } => {
                let facts = release_confirm(&cwd, &project_id)?;
                println!("{}", serde_json::to_string_pretty(&facts)?);
            }
            ReleaseCommand::RecordTag {
                project_id,
                tag_name,
                tag_commit_sha,
                actor,
            } => {
                let facts =
                    release_record_tag(&cwd, &project_id, &tag_name, &tag_commit_sha, &actor)?;
                println!("{}", serde_json::to_string_pretty(&facts)?);
            }
            ReleaseCommand::RecordRemote {
                project_id,
                provider,
                release_id,
                release_url,
                tag_name,
                release_commit_sha,
                artifact_manifest_path,
                actor,
            } => {
                let facts = release_record_remote(
                    &cwd,
                    &project_id,
                    &provider,
                    &release_id,
                    &release_url,
                    &tag_name,
                    &release_commit_sha,
                    &artifact_manifest_path,
                    &actor,
                )?;
                println!("{}", serde_json::to_string_pretty(&facts)?);
            }
            ReleaseCommand::Publish { project_id } => {
                let facts = release_publish(&cwd, &project_id)?;
                println!("{}", serde_json::to_string_pretty(&facts)?);
            }
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
        Command::ProviderSmoke {
            provider,
            issue_id,
            run_id,
            working_directory,
            launch_request_path,
        } => {
            let working_directory = working_directory
                .unwrap_or_else(|| cwd.clone())
                .display()
                .to_string();
            let mut request = agentflow_mcp::McpProviderSmokeRequest::new(
                provider.clone(),
                issue_id,
                run_id,
                working_directory,
                launch_request_path,
            );
            request.enabled = true;
            let bridge = agentflow_mcp::default_provider_bridge();
            let provider_impl = bridge.provider(&provider).ok_or_else(|| {
                anyhow::anyhow!("unsupported provider smoke provider: {provider}")
            })?;
            let artifact = agentflow_mcp::run_provider_smoke_gate(&cwd, provider_impl, &request)?;
            println!("{}", serde_json::to_string_pretty(&artifact)?);
        }
    }

    Ok(())
}
