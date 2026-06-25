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
    CapabilityRegistryCommand, Cli, Command, CompletionCommand, GovernancePolicyCommand,
    MessageBusCommand, PackCommand, ProjectCommand, ProjectionCommand, ReleaseCommand,
    RuntimeCommandCli, TaskLoopCommand,
};
use clap::Parser;
use formal::{
    audit_request_human, completion_decide, completion_inspect, project_confirm_goal,
    project_confirm_plan, project_intake, project_materialize, project_preview_goal,
    release_confirm, release_prepare, release_publish, release_record_remote, release_record_tag,
};
use serde::de::DeserializeOwned;
use serde_json::json;
use std::{fs, path::Path};

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
            ProjectionCommand::ReplayReport { output } => {
                let report = agentflow_projection::rebuild_projections_with_replay_report(&cwd)?;
                let report_json = serde_json::to_string_pretty(&report)?;
                if let Some(output) = output {
                    if let Some(parent) = output.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    fs::write(&output, report_json + "\n")?;
                    println!("{}", output.display());
                } else {
                    println!("{report_json}");
                }
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
        Command::GovernancePolicy { command } => match command {
            GovernancePolicyCommand::Evaluate {
                role,
                action_type,
                object_type,
                worker_id,
                command,
                audit_sidecar_mode,
                capability_registry,
                output,
            } => {
                let ontology = agentflow_ontology::core_ontology_registry();
                let action_registry =
                    agentflow_action_contract::core_action_contract_registry(&ontology);
                let role_registry =
                    agentflow_role_policy::core_role_policy_registry(&ontology, &action_registry);
                let capability_registry = if let Some(path) = capability_registry {
                    read_json(path)?
                } else {
                    agentflow_capability_registry::default_capability_registry()
                };
                let audit_sidecar_mode = parse_audit_sidecar_mode(&audit_sidecar_mode)?;
                let report = agentflow_governance_policy::evaluate_runtime_governance(
                    &role_registry,
                    &capability_registry,
                    agentflow_governance_policy::GovernancePolicyRequest {
                        actor_role: role,
                        action_type,
                        object_type,
                        worker_id,
                        command,
                        audit_sidecar_mode,
                    },
                );
                write_or_print_json(output.as_deref(), &report)?;
            }
        },
        Command::RuntimeCommand { command } => match command {
            RuntimeCommandCli::Execute { request, output } => {
                let command = read_json::<agentflow_runtime_api::RuntimeCommandRequest>(request)?;
                let response =
                    agentflow_runtime_api::execute_command_via_arbitration(&cwd, &command)?;
                write_or_print_json(output.as_deref(), &response)?;
            }
        },
        Command::MessageBus { command } => match command {
            MessageBusCommand::Decision {
                local_runtime_sufficient,
                cross_process_worker_required,
                cloud_fanout_required,
                event_subscription_required,
                durable_queue_required,
                evidence,
                output,
            } => {
                let report = agentflow_message_bus::evaluate_cross_process_scheduling(
                    agentflow_message_bus::SchedulingDecisionRequest {
                        local_runtime_sufficient,
                        cross_process_worker_required,
                        cloud_fanout_required,
                        event_subscription_required,
                        durable_queue_required,
                        evidence,
                    },
                );
                write_or_print_json(output.as_deref(), &report)?;
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
            PackCommand::ReleaseGateReadiness {
                output_dir,
                runtime_version,
            } => {
                write_pack_release_gate_readiness(
                    &cwd,
                    &output_dir,
                    runtime_version
                        .as_deref()
                        .unwrap_or(env!("CARGO_PKG_VERSION")),
                )?;
                println!("pack release gate readiness: written");
                println!("output: {}", output_dir.display());
            }
            PackCommand::ValidateManifest { manifest_path } => {
                let manifest = agentflow_pack::load_pack_manifest(manifest_path)?;
                let report = agentflow_pack::validate_pack_manifest(&manifest);
                println!("{}", serde_json::to_string_pretty(&report)?);
            }
            PackCommand::MigrationPreview {
                preview_id,
                pack_id,
                from_version,
                to_version,
                affected_objects,
                affected_projections,
                output,
            } => {
                let preview = agentflow_pack::generate_pack_migration_preview(
                    preview_id,
                    pack_id,
                    from_version,
                    to_version,
                    affected_objects,
                    affected_projections,
                );
                write_or_print_json(output.as_deref(), &preview)?;
            }
            PackCommand::MigrationApply {
                preview_path,
                confirmed,
                actor,
                reason,
                output,
            } => {
                let preview: agentflow_pack::PackMigrationPreview = read_json(&preview_path)?;
                let receipt = agentflow_pack::pack_migration_applied_receipt(
                    &preview,
                    &agentflow_pack::PackMigrationApplyConfirmation {
                        preview_id: preview.preview_id.clone(),
                        confirmed,
                        actor,
                        reason,
                    },
                )?;
                write_or_print_json(output.as_deref(), &receipt)?;
            }
            PackCommand::MigrationCancel {
                preview_path,
                actor,
                reason,
                output,
            } => {
                let preview: agentflow_pack::PackMigrationPreview = read_json(&preview_path)?;
                let receipt = agentflow_pack::cancel_pack_migration(&preview, actor, reason)?;
                write_or_print_json(output.as_deref(), &receipt)?;
            }
            PackCommand::MigrationRollback {
                applied_receipt_path,
                actor,
                reason,
                output,
            } => {
                let applied: agentflow_pack::PackMigrationAppliedReceipt =
                    read_json(&applied_receipt_path)?;
                let receipt = agentflow_pack::rollback_pack_migration(&applied, actor, reason)?;
                write_or_print_json(output.as_deref(), &receipt)?;
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
            ReleaseCommand::DeploymentEvidence {
                release_version,
                release_tag,
                source_commit_sha,
                runtime_version,
                release_facts_path,
                remote_release_proof_path,
                config_fingerprint_path,
                pack_version_fingerprint_path,
                event_store_fingerprint_path,
                projection_rebuild_proof_path,
                migration_receipt_path,
                rollback_receipt_path,
                failed_deployment_report_path,
                rollback_target_tag,
                rollback_target_commit_sha,
                output,
            } => {
                let report = agentflow_release::build_deployment_evidence_report(
                    agentflow_release::DeploymentEvidenceInput {
                        release_version,
                        release_tag,
                        source_commit_sha,
                        runtime_version,
                        release_facts_path,
                        remote_release_proof_path,
                        config_fingerprint_path,
                        pack_version_fingerprint_path,
                        event_store_fingerprint_path,
                        projection_rebuild_proof_path,
                        migration_receipt_path,
                        rollback_receipt_path,
                        failed_deployment_report_path,
                        rollback_target_tag,
                        rollback_target_commit_sha,
                    },
                )?;
                write_or_print_json(output.as_deref(), &report)?;
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

fn write_pack_release_gate_readiness(
    project_root: &Path,
    output_dir: &Path,
    runtime_version: &str,
) -> anyhow::Result<()> {
    std::fs::create_dir_all(output_dir)?;

    let api_manifest = agentflow_runtime_api::api_plane_manifest();
    let api_entries = agentflow_pack::pack_readiness_api_entries();
    let pack_registry = agentflow_pack::load_pack_fixture_registry()?;
    for required_pack_id in ["software-dev", "ui-design"] {
        if pack_registry.pack(required_pack_id).is_none() {
            anyhow::bail!("file-backed pack registry missing required pack: {required_pack_id}");
        }
    }
    let software =
        agentflow_pack::software_dev_pack_readiness_artifact(&api_entries, runtime_version);
    let design = agentflow_pack::ui_design_pack_readiness_artifact(&api_entries, runtime_version);
    let pack_artifacts = [&software, &design];

    write_json(
        output_dir.join("software-dev-pack-readiness.json"),
        &software,
    )?;
    write_json(output_dir.join("ui-design-pack-readiness.json"), &design)?;

    write_json(output_dir.join("pack-registry.json"), &pack_registry)?;

    let validation_entries = pack_artifacts
        .iter()
        .map(|artifact| {
            json!({
                "packId": artifact.pack_id,
                "status": artifact.status,
                "validationActive": artifact.validation.active,
                "issueCount": artifact.validation.issues.len(),
                "missingReadModels": artifact.validation.missing_read_models,
                "missingCommandMappings": artifact.validation.missing_command_mappings,
                "warnings": artifact.warnings,
            })
        })
        .collect::<Vec<_>>();
    let validation_passed = pack_artifacts
        .iter()
        .all(|artifact| artifact.can_load && artifact.can_validate && artifact.can_project);
    write_json(
        output_dir.join("pack-validation-report.json"),
        &json!({
            "version": "agentflow-pack-release-gate-validation.v1",
            "status": if validation_passed { "passed" } else { "failed" },
            "statusVocabulary": ["completed", "baseline", "deferred", "carryover"],
            "writesAuthority": false,
            "packs": validation_entries,
            "failureRule": "release gate must fail before publishing Pack System ready if any pack cannot load, validate, or project",
        }),
    )?;

    let simulations = vec![
        agentflow_simulation::simulate_pack_command(
            &agentflow_simulation::PackCommandSimulationRequest {
                simulation_id: "release-gate-pack-software-dev-001".to_string(),
                command: "work.issue.start".to_string(),
                target_object_type: "Issue".to_string(),
                target_object_id: "AF-PACK-READY-001".to_string(),
                actor_role: "work-agent".to_string(),
                validation: software.validation.clone(),
                domain: agentflow_pack::software_dev_domain_definition(),
                surface: agentflow_pack::software_dev_surface_definition(),
                connector: agentflow_pack::software_dev_connector_definition(),
                created_at: "release-gate".to_string(),
            },
        ),
        agentflow_simulation::simulate_pack_command(
            &agentflow_simulation::PackCommandSimulationRequest {
                simulation_id: "release-gate-pack-ui-design-001".to_string(),
                command: "design.wireframe.generate".to_string(),
                target_object_type: "Wireframe".to_string(),
                target_object_id: "wireframe-pack-ready-001".to_string(),
                actor_role: "work-agent".to_string(),
                validation: design.validation.clone(),
                domain: agentflow_pack::ui_design_domain_definition(),
                surface: agentflow_pack::ui_design_surface_definition(),
                connector: agentflow_pack::ui_design_connector_definition(),
                created_at: "release-gate".to_string(),
            },
        ),
    ];
    let simulation_passed = simulations.iter().all(|report| {
        report.decision == agentflow_simulation::SimulationDecision::Accepted
            && !report.writes_authority
            && !report.writes_event_store
            && !report.executes_provider
            && !report.affected_objects.is_empty()
            && !report.required_evidence.is_empty()
            && !report.state_transitions.is_empty()
            && !report.downstream_triggers.is_empty()
            && !report.conflicts.is_empty()
            && !report.gate_impact.is_empty()
            && !report.affected_projections.is_empty()
    });
    write_json(
        output_dir.join("pack-simulation-report.json"),
        &json!({
            "version": "agentflow-pack-release-gate-simulation.v1",
            "status": if simulation_passed { "passed" } else { "failed" },
            "writesAuthority": false,
            "writesEventStore": false,
            "executesProvider": false,
            "reports": simulations,
        }),
    )?;

    let software_projection =
        agentflow_projection::get_pack_industry_workbench_view(project_root, Some("software-dev"))?;
    let design_projection =
        agentflow_projection::get_pack_industry_workbench_view(project_root, Some("ui-design"))?;
    let projection_passed = software_projection.active_pack_id.as_deref() == Some("software-dev")
        && design_projection.active_pack_id.as_deref() == Some("ui-design")
        && !software_projection.pack_list.is_empty()
        && !design_projection.pack_list.is_empty();
    write_json(
        output_dir.join("pack-projection-readiness.json"),
        &json!({
            "version": "agentflow-pack-release-gate-projection-readiness.v1",
            "status": if projection_passed { "passed" } else { "failed" },
            "writesAuthority": false,
            "views": [
                {
                    "packId": "software-dev",
                    "activePackId": software_projection.active_pack_id,
                    "packCount": software_projection.pack_list.len(),
                    "workbenchCount": software_projection.industry_workbenches.len(),
                    "readiness": software_projection.pack_readiness,
                },
                {
                    "packId": "ui-design",
                    "activePackId": design_projection.active_pack_id,
                    "packCount": design_projection.pack_list.len(),
                    "workbenchCount": design_projection.industry_workbenches.len(),
                    "readiness": design_projection.pack_readiness,
                },
            ],
        }),
    )?;

    let pack_api_entries = api_manifest
        .entries
        .iter()
        .filter(|entry| {
            entry.category == "pack_actions"
                || entry.category == "pack_command_surface"
                || entry.api_id == "projection.pack-industry-workbench"
        })
        .collect::<Vec<_>>();
    let api_plane_passed = pack_api_entries
        .iter()
        .any(|entry| entry.category == "pack_actions")
        && pack_api_entries
            .iter()
            .any(|entry| entry.category == "pack_command_surface")
        && pack_api_entries
            .iter()
            .any(|entry| entry.api_id == "projection.pack-industry-workbench");
    write_json(
        output_dir.join("pack-api-plane-manifest.json"),
        &json!({
            "version": "agentflow-pack-release-gate-api-plane.v1",
            "status": if api_plane_passed { "passed" } else { "failed" },
            "sourceManifestVersion": api_manifest.version,
            "entries": pack_api_entries,
        }),
    )?;

    if !(validation_passed && simulation_passed && projection_passed && api_plane_passed) {
        anyhow::bail!(
            "pack release gate readiness failed: validation={validation_passed}, simulation={simulation_passed}, projection={projection_passed}, apiPlane={api_plane_passed}"
        );
    }

    Ok(())
}

fn write_json(path: impl AsRef<Path>, value: &impl serde::Serialize) -> anyhow::Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, serde_json::to_string_pretty(value)? + "\n")?;
    Ok(())
}

fn read_json<T: DeserializeOwned>(path: impl AsRef<Path>) -> anyhow::Result<T> {
    let path = path.as_ref();
    let payload = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&payload)?)
}

fn parse_audit_sidecar_mode(
    value: &str,
) -> anyhow::Result<agentflow_governance_policy::AuditSidecarMode> {
    match value {
        "independent" => Ok(agentflow_governance_policy::AuditSidecarMode::Independent),
        "not-requested" => Ok(agentflow_governance_policy::AuditSidecarMode::NotRequested),
        "bound-to-main-chain" => {
            Ok(agentflow_governance_policy::AuditSidecarMode::BoundToMainChain)
        }
        _ => anyhow::bail!(
            "unsupported audit sidecar mode `{value}`; expected independent, not-requested, or bound-to-main-chain"
        ),
    }
}

fn write_or_print_json(output: Option<&Path>, value: &impl serde::Serialize) -> anyhow::Result<()> {
    if let Some(output) = output {
        write_json(output, value)?;
    } else {
        println!("{}", serde_json::to_string_pretty(value)?);
    }
    Ok(())
}
