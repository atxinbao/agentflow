use crate::model::{
    ProjectReleaseFacts, ProjectReleaseIndex, ProjectReleaseIndexEntry,
    PublicReleaseDocumentTarget, ReleaseTagProof, RemoteReleaseProof,
    PROJECT_RELEASE_FACTS_VERSION, PROJECT_RELEASE_INDEX_VERSION, RELEASE_TAG_PROOF_VERSION,
    REMOTE_RELEASE_PROOF_VERSION,
};
use crate::public_delivery::{
    collect_public_release_summary_for_project, write_public_release_documents,
};
use crate::review_surface::sync_project_external_review_surface;
use agentflow_event_store::EventActor;
use agentflow_workflow_core::{canonicalize_project_root, join_relative_path, ProjectId};
use agentflow_workflow_runtime::{
    apply_canonical_workflow_event, RuntimeContext, StaticActionRegistry, StaticGuardRegistry,
    WorkflowFlowType,
};
use anyhow::{Context, Result};
use serde::Deserialize;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProjectProjectionSnapshot {
    project_id: String,
    title: String,
    #[serde(default)]
    completion: Option<CompletionProjectionSnapshot>,
    #[serde(default)]
    delivery: Option<ProjectDeliverySnapshot>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CompletionProjectionSnapshot {
    current_state: String,
    latest_outcome: Option<String>,
    #[serde(default = "default_completion_release_readiness")]
    release_readiness: String,
}

fn default_completion_release_readiness() -> String {
    "blocked".to_string()
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProjectDeliverySnapshot {
    status: String,
    #[serde(default)]
    missing_count: usize,
    #[serde(default)]
    summary_line: String,
}

#[derive(Debug, Clone)]
struct ReleaseGate {
    current_state: String,
    gate_status: String,
    gate_reason: String,
    completion_state: String,
    completion_outcome: Option<String>,
    delivery_status: String,
}

#[derive(Debug, Clone)]
struct ReleaseRuntimeContext {
    root: PathBuf,
    project_id: String,
    projection: ProjectProjectionSnapshot,
    target: PublicReleaseDocumentTarget,
    gate: ReleaseGate,
    existing: Option<ProjectReleaseFacts>,
}

const PUBLICATION_STAGE_PENDING: &str = "pending";
const PUBLICATION_STAGE_PUBLIC_RECORD_WRITTEN: &str = "public-record-written";
const PUBLICATION_STAGE_TAG_CREATED: &str = "tag-created";
const PUBLICATION_STAGE_REMOTE_RELEASE_CREATED: &str = "remote-release-created";
const PUBLICATION_STAGE_PUBLISHED: &str = "published";

fn publication_stage_rank(stage: &str) -> usize {
    match stage {
        PUBLICATION_STAGE_PENDING => 0,
        PUBLICATION_STAGE_PUBLIC_RECORD_WRITTEN => 1,
        PUBLICATION_STAGE_TAG_CREATED => 2,
        PUBLICATION_STAGE_REMOTE_RELEASE_CREATED => 3,
        PUBLICATION_STAGE_PUBLISHED => 4,
        _ => 0,
    }
}

fn release_summary_line(
    current_state: &str,
    publication_stage: &str,
    target: &PublicReleaseDocumentTarget,
    gate_reason: &str,
) -> String {
    match current_state {
        "published" => "Release 已发布，远端发布证明与公开记录已经对齐。".to_string(),
        "in_progress" => match publication_stage {
            PUBLICATION_STAGE_REMOTE_RELEASE_CREATED => {
                "远端 Release 证明已写入，下一步可以正式标记 published。".to_string()
            }
            PUBLICATION_STAGE_TAG_CREATED => {
                "Release Tag 证明已写入，下一步需要记录远端 Release 证明。".to_string()
            }
            PUBLICATION_STAGE_PUBLIC_RECORD_WRITTEN => {
                "Release 公开记录已写入，下一步需要记录 Tag 证明。".to_string()
            }
            _ => "Release 已确认，正在等待公开记录写入。".to_string(),
        },
        "ready" => "Release 已准备好，等待确认发布内容。".to_string(),
        "blocked" => format!("Release 已阻断：{}。", gate_reason),
        _ if publication_stage == PUBLICATION_STAGE_PUBLIC_RECORD_WRITTEN => format!(
            "Release 公开记录已写入 {} 和 {}。",
            target.changelog_path.display(),
            target.release_notes_path.display()
        ),
        _ => gate_reason.to_string(),
    }
}

pub fn prepare_project_release(
    project_root: impl AsRef<Path>,
    project_id: impl AsRef<str>,
) -> Result<ProjectReleaseFacts> {
    let context = build_release_runtime_context(project_root, project_id)?;
    let now = unix_timestamp_seconds();

    if context
        .existing
        .as_ref()
        .is_some_and(|facts| facts.current_state == "published")
    {
        return refresh_published_release(&context, now);
    }

    if context.gate.gate_status != "ready" {
        let facts = build_release_facts(
            &context,
            context.gate.current_state.clone(),
            context.gate.gate_status.clone(),
            context.gate.gate_reason.clone(),
            0,
            context
                .existing
                .as_ref()
                .and_then(|facts| facts.latest_event_id.clone()),
            context
                .existing
                .as_ref()
                .and_then(|facts| facts.published_at),
            now,
        );
        return persist_release_facts(&context, facts);
    }

    let summary = collect_public_release_summary_for_project(
        &context.root,
        Some(context.project_id.as_str()),
    )?;
    if summary.entries.is_empty() {
        let facts = build_release_facts(
            &context,
            "blocked".to_string(),
            "blocked".to_string(),
            "项目已进入 release 阶段，但当前还没有可汇总的已完成任务交付。".to_string(),
            0,
            context
                .existing
                .as_ref()
                .and_then(|facts| facts.latest_event_id.clone()),
            context
                .existing
                .as_ref()
                .and_then(|facts| facts.published_at),
            now,
        );
        return persist_release_facts(&context, facts);
    }

    let existing_state = context
        .existing
        .as_ref()
        .map(|facts| facts.current_state.as_str())
        .unwrap_or("pending");
    let current_state = match existing_state {
        "in_progress" => "in_progress",
        "ready" => "ready",
        "published" => "published",
        _ => "pending",
    };
    let mut latest_event_id = context
        .existing
        .as_ref()
        .and_then(|facts| facts.latest_event_id.clone());
    if current_state == "pending" {
        latest_event_id = transition_release_state(
            &context.root,
            &context.project_id,
            "pending",
            "delivery.ready",
            &context.target,
            summary.entries.len(),
            &context.gate,
        )?
        .or(latest_event_id);
    }

    let persisted_state = if current_state == "in_progress" {
        "in_progress"
    } else {
        "ready"
    };
    let persisted_reason = if persisted_state == "in_progress" {
        match context
            .existing
            .as_ref()
            .map(|facts| facts.publication_stage.as_str())
        {
            Some(PUBLICATION_STAGE_REMOTE_RELEASE_CREATED) => {
                "远端 Release 证明已写入，下一步可以正式标记 published。"
            }
            Some(PUBLICATION_STAGE_TAG_CREATED) => {
                "Release Tag 已记录，正在等待远端 Release 证明。"
            }
            Some(PUBLICATION_STAGE_PUBLIC_RECORD_WRITTEN) => {
                "Release 公开记录已写入，正在等待 Tag 证明。"
            }
            _ => "Release 说明已确认，正在等待正式发布。",
        }
    } else {
        "Release 已就绪，下一步可以确认并生成公开说明。"
    };

    let facts = build_release_facts(
        &context,
        persisted_state.to_string(),
        "ready".to_string(),
        persisted_reason.to_string(),
        summary.entries.len(),
        latest_event_id,
        context
            .existing
            .as_ref()
            .and_then(|facts| facts.published_at),
        now,
    );
    persist_release_facts(&context, facts)
}

pub fn confirm_project_release(
    project_root: impl AsRef<Path>,
    project_id: impl AsRef<str>,
) -> Result<ProjectReleaseFacts> {
    let context = build_release_runtime_context(project_root, project_id)?;
    let now = unix_timestamp_seconds();

    if context
        .existing
        .as_ref()
        .is_some_and(|facts| facts.current_state == "published")
    {
        return refresh_published_release(&context, now);
    }

    let prepared = prepare_project_release(&context.root, &context.project_id)?;
    if prepared.current_state != "ready" && prepared.current_state != "in_progress" {
        anyhow::bail!(
            "release confirm requires ready state, found {}",
            prepared.current_state
        );
    }
    if prepared.current_state == "in_progress"
        && prepared.publication_stage != PUBLICATION_STAGE_PENDING
    {
        return Ok(prepared);
    }

    let summary = collect_public_release_summary_for_project(
        &context.root,
        Some(context.project_id.as_str()),
    )?;
    let paths = write_public_release_documents(&context.root, &summary, &context.target)?;
    let latest_event_id = transition_release_state(
        &context.root,
        &context.project_id,
        "ready",
        "delivery.started",
        &PublicReleaseDocumentTarget {
            changelog_path: PathBuf::from(paths.changelog_path.clone()),
            release_notes_path: PathBuf::from(paths.release_notes_path.clone()),
        },
        summary.entries.len(),
        &context.gate,
    )?
    .or(prepared.latest_event_id.clone());

    let mut facts = build_release_facts(
        &context,
        "in_progress".to_string(),
        "ready".to_string(),
        "Release 公开记录已生成，下一步需要记录 Tag 证明。".to_string(),
        summary.entries.len(),
        latest_event_id,
        prepared.published_at,
        now,
    );
    facts.publication_stage = PUBLICATION_STAGE_PUBLIC_RECORD_WRITTEN.to_string();
    facts.public_record_written_at = Some(now);
    facts.changelog_path = paths.changelog_path;
    facts.release_notes_path = paths.release_notes_path;
    facts.summary_line = release_summary_line(
        facts.current_state.as_str(),
        facts.publication_stage.as_str(),
        &context.target,
        facts.gate_reason.as_str(),
    );
    persist_release_facts(&context, facts)
}

pub fn record_project_release_tag(
    project_root: impl AsRef<Path>,
    project_id: impl AsRef<str>,
    tag_name: impl AsRef<str>,
    tag_commit_sha: impl AsRef<str>,
    actor: impl AsRef<str>,
) -> Result<ProjectReleaseFacts> {
    let context = build_release_runtime_context(project_root, project_id)?;
    let now = unix_timestamp_seconds();
    let confirmed = confirm_project_release(&context.root, &context.project_id)?;
    if confirmed.current_state != "in_progress" {
        anyhow::bail!(
            "release record-tag requires in_progress state, found {}",
            confirmed.current_state
        );
    }

    let tag_name = tag_name.as_ref().trim();
    let tag_commit_sha = tag_commit_sha.as_ref().trim();
    let actor = actor.as_ref().trim();
    if tag_name.is_empty() || tag_commit_sha.is_empty() || actor.is_empty() {
        anyhow::bail!("release tag proof requires tag name, commit sha, and actor");
    }

    let proof = ReleaseTagProof {
        version: RELEASE_TAG_PROOF_VERSION.to_string(),
        project_id: context.project_id.clone(),
        tag_name: tag_name.to_string(),
        tag_commit_sha: tag_commit_sha.to_string(),
        actor: actor.to_string(),
        recorded_at: now,
    };
    let proof_relative_path = project_release_tag_proof_path(&context.project_id)?;
    let proof_path = join_relative_path(&context.root, proof_relative_path.clone())?;
    write_json(&proof_path, &proof)?;

    let summary = collect_public_release_summary_for_project(
        &context.root,
        Some(context.project_id.as_str()),
    )?;
    let mut facts = build_release_facts(
        &context,
        "in_progress".to_string(),
        "ready".to_string(),
        "Release Tag 证明已写入，下一步需要记录远端 Release 证明。".to_string(),
        summary.entries.len(),
        confirmed.latest_event_id.clone(),
        confirmed.published_at,
        now,
    );
    facts.publication_stage = PUBLICATION_STAGE_TAG_CREATED.to_string();
    facts.public_record_written_at = confirmed.public_record_written_at.or(Some(now));
    facts.tag_name = Some(tag_name.to_string());
    facts.tag_commit_sha = Some(tag_commit_sha.to_string());
    facts.tag_proof_path = Some(proof_relative_path.display().to_string());
    facts.summary_line = release_summary_line(
        facts.current_state.as_str(),
        facts.publication_stage.as_str(),
        &context.target,
        facts.gate_reason.as_str(),
    );
    persist_release_facts(&context, facts)
}

pub fn record_project_remote_release(
    project_root: impl AsRef<Path>,
    project_id: impl AsRef<str>,
    provider: impl AsRef<str>,
    release_id: impl AsRef<str>,
    release_url: impl AsRef<str>,
    tag_name: impl AsRef<str>,
    release_commit_sha: impl AsRef<str>,
    artifact_manifest_path: impl AsRef<str>,
    actor: impl AsRef<str>,
) -> Result<ProjectReleaseFacts> {
    let context = build_release_runtime_context(project_root, project_id)?;
    let now = unix_timestamp_seconds();
    let tagged = if context.existing.as_ref().is_some_and(|facts| {
        publication_stage_rank(facts.publication_stage.as_str())
            >= publication_stage_rank(PUBLICATION_STAGE_TAG_CREATED)
    }) {
        context.existing.clone().unwrap()
    } else {
        let existing = context.existing.as_ref();
        record_project_release_tag(
            &context.root,
            &context.project_id,
            tag_name.as_ref(),
            existing
                .and_then(|facts| facts.tag_commit_sha.as_deref())
                .unwrap_or_else(|| release_commit_sha.as_ref()),
            actor.as_ref(),
        )?
    };
    if tagged.current_state != "in_progress" {
        anyhow::bail!(
            "release record-remote requires in_progress state, found {}",
            tagged.current_state
        );
    }

    let provider = provider.as_ref().trim();
    let release_id = release_id.as_ref().trim();
    let release_url = release_url.as_ref().trim();
    let tag_name = tag_name.as_ref().trim();
    let release_commit_sha = release_commit_sha.as_ref().trim();
    let artifact_manifest_path = artifact_manifest_path.as_ref().trim();
    let actor = actor.as_ref().trim();
    if provider.is_empty()
        || release_id.is_empty()
        || release_url.is_empty()
        || tag_name.is_empty()
        || release_commit_sha.is_empty()
        || artifact_manifest_path.is_empty()
        || actor.is_empty()
    {
        anyhow::bail!(
            "remote release proof requires provider, release id/url, tag, commit sha, artifact manifest path, and actor"
        );
    }

    let manifest_path = resolve_workspace_path(&context.root, artifact_manifest_path)?;
    let manifest_sha256 = sha256_file(&manifest_path)?;
    let manifest_display = workspace_or_absolute_display(&context.root, &manifest_path);
    let proof = RemoteReleaseProof {
        version: REMOTE_RELEASE_PROOF_VERSION.to_string(),
        project_id: context.project_id.clone(),
        provider: provider.to_string(),
        release_id: release_id.to_string(),
        release_url: release_url.to_string(),
        tag_name: tag_name.to_string(),
        release_commit_sha: release_commit_sha.to_string(),
        artifact_manifest_path: Some(manifest_display.clone()),
        artifact_manifest_sha256: Some(manifest_sha256.clone()),
        actor: actor.to_string(),
        recorded_at: now,
    };
    let proof_relative_path = project_remote_release_proof_path(&context.project_id)?;
    let proof_path = join_relative_path(&context.root, proof_relative_path.clone())?;
    write_json(&proof_path, &proof)?;

    let summary = collect_public_release_summary_for_project(
        &context.root,
        Some(context.project_id.as_str()),
    )?;
    let mut facts = build_release_facts(
        &context,
        "in_progress".to_string(),
        "ready".to_string(),
        "远端 Release 证明已写入，下一步可以正式标记 published。".to_string(),
        summary.entries.len(),
        tagged.latest_event_id.clone(),
        tagged.published_at,
        now,
    );
    facts.publication_stage = PUBLICATION_STAGE_REMOTE_RELEASE_CREATED.to_string();
    facts.public_record_written_at = tagged.public_record_written_at.or(Some(now));
    facts.tag_name = Some(tag_name.to_string());
    facts.tag_commit_sha = Some(
        tagged
            .tag_commit_sha
            .unwrap_or_else(|| release_commit_sha.to_string()),
    );
    facts.tag_proof_path = tagged.tag_proof_path;
    facts.remote_provider = Some(provider.to_string());
    facts.remote_release_id = Some(release_id.to_string());
    facts.remote_release_url = Some(release_url.to_string());
    facts.remote_release_commit_sha = Some(release_commit_sha.to_string());
    facts.remote_release_proof_path = Some(proof_relative_path.display().to_string());
    facts.artifact_manifest_path = Some(manifest_display);
    facts.artifact_manifest_sha256 = Some(manifest_sha256);
    facts.summary_line = release_summary_line(
        facts.current_state.as_str(),
        facts.publication_stage.as_str(),
        &context.target,
        facts.gate_reason.as_str(),
    );
    persist_release_facts(&context, facts)
}

pub fn publish_project_release(
    project_root: impl AsRef<Path>,
    project_id: impl AsRef<str>,
) -> Result<ProjectReleaseFacts> {
    let context = build_release_runtime_context(project_root, project_id)?;
    let now = unix_timestamp_seconds();

    if context
        .existing
        .as_ref()
        .is_some_and(|facts| facts.current_state == "published")
    {
        return refresh_published_release(&context, now);
    }

    let confirmed = confirm_project_release(&context.root, &context.project_id)?;
    if confirmed.current_state != "in_progress" {
        anyhow::bail!(
            "release publish requires in_progress state, found {}",
            confirmed.current_state
        );
    }
    if confirmed.publication_stage != PUBLICATION_STAGE_REMOTE_RELEASE_CREATED {
        anyhow::bail!(
            "release publish requires remote release proof, found publication stage {}",
            confirmed.publication_stage
        );
    }

    let summary = collect_public_release_summary_for_project(
        &context.root,
        Some(context.project_id.as_str()),
    )?;
    let paths = write_public_release_documents(&context.root, &summary, &context.target)?;
    let latest_event_id = transition_release_state(
        &context.root,
        &context.project_id,
        "in_progress",
        "delivery.published",
        &PublicReleaseDocumentTarget {
            changelog_path: PathBuf::from(paths.changelog_path.clone()),
            release_notes_path: PathBuf::from(paths.release_notes_path.clone()),
        },
        summary.entries.len(),
        &context.gate,
    )?
    .or(confirmed.latest_event_id.clone());

    let mut facts = build_release_facts(
        &context,
        "published".to_string(),
        "ready".to_string(),
        "Release 已发布，远端发布证明与公开记录已对齐。".to_string(),
        summary.entries.len(),
        latest_event_id,
        Some(now),
        now,
    );
    facts.publication_stage = PUBLICATION_STAGE_PUBLISHED.to_string();
    facts.summary_line = release_summary_line(
        facts.current_state.as_str(),
        facts.publication_stage.as_str(),
        &context.target,
        facts.gate_reason.as_str(),
    );
    persist_release_facts(&context, facts)
}

pub fn sync_project_release(
    project_root: impl AsRef<Path>,
    project_id: impl AsRef<str>,
) -> Result<ProjectReleaseFacts> {
    let prepared = prepare_project_release(&project_root, project_id.as_ref())?;
    if prepared.current_state != "ready" {
        return Ok(prepared);
    }
    let confirmed = confirm_project_release(&project_root, project_id.as_ref())?;
    if confirmed.current_state != "in_progress"
        || confirmed.publication_stage != PUBLICATION_STAGE_REMOTE_RELEASE_CREATED
    {
        return Ok(confirmed);
    }
    publish_project_release(project_root, project_id)
}

pub fn load_project_release_facts(
    project_root: impl AsRef<Path>,
    project_id: &str,
) -> Result<ProjectReleaseFacts> {
    let root = canonicalize_project_root(project_root)?;
    read_json(&project_release_facts_path(&root, project_id)?)
}

pub fn load_project_release_index(project_root: impl AsRef<Path>) -> Result<ProjectReleaseIndex> {
    let root = canonicalize_project_root(project_root)?;
    read_json(&root.join(".agentflow/indexes/releases.json"))
}

fn build_release_runtime_context(
    project_root: impl AsRef<Path>,
    project_id: impl AsRef<str>,
) -> Result<ReleaseRuntimeContext> {
    let root = canonicalize_project_root(project_root)?;
    let project_id = ProjectId::parse(project_id.as_ref())?;
    let projection = load_project_projection_snapshot(&root, project_id.as_str())?;
    let target = default_project_release_target(project_id.as_str())?;
    let existing = load_project_release_facts(&root, project_id.as_str()).ok();
    let gate = evaluate_release_gate(&projection);
    Ok(ReleaseRuntimeContext {
        root,
        project_id: project_id.as_str().to_string(),
        projection,
        target,
        gate,
        existing,
    })
}

fn refresh_published_release(
    context: &ReleaseRuntimeContext,
    now: u64,
) -> Result<ProjectReleaseFacts> {
    let summary = collect_public_release_summary_for_project(
        &context.root,
        Some(context.project_id.as_str()),
    )?;
    let paths = write_public_release_documents(&context.root, &summary, &context.target)?;
    let mut facts = build_release_facts(
        context,
        "published".to_string(),
        "ready".to_string(),
        "Release 已经发布，远端发布证明仍然有效。".to_string(),
        summary.entries.len(),
        context
            .existing
            .as_ref()
            .and_then(|facts| facts.latest_event_id.clone()),
        context
            .existing
            .as_ref()
            .and_then(|facts| facts.published_at)
            .or(Some(now)),
        now,
    );
    facts.publication_stage = PUBLICATION_STAGE_PUBLISHED.to_string();
    facts.changelog_path = paths.changelog_path;
    facts.release_notes_path = paths.release_notes_path;
    persist_release_facts(context, facts)
}

fn build_release_facts(
    context: &ReleaseRuntimeContext,
    current_state: String,
    gate_status: String,
    gate_reason: String,
    entry_count: usize,
    latest_event_id: Option<String>,
    published_at: Option<u64>,
    now: u64,
) -> ProjectReleaseFacts {
    let inherited_publication_stage = context
        .existing
        .as_ref()
        .map(|facts| facts.publication_stage.clone())
        .unwrap_or_else(|| PUBLICATION_STAGE_PENDING.to_string());
    let summary_line = release_summary_line(
        current_state.as_str(),
        inherited_publication_stage.as_str(),
        &context.target,
        &gate_reason,
    );

    ProjectReleaseFacts {
        version: PROJECT_RELEASE_FACTS_VERSION.to_string(),
        project_id: context.projection.project_id.clone(),
        project_title: context.projection.title.clone(),
        current_state,
        publication_stage: inherited_publication_stage,
        gate_status,
        gate_reason,
        completion_state: context.gate.completion_state.clone(),
        completion_outcome: context.gate.completion_outcome.clone(),
        delivery_status: context.gate.delivery_status.clone(),
        public_record_written_at: context
            .existing
            .as_ref()
            .and_then(|facts| facts.public_record_written_at),
        changelog_path: context.target.changelog_path.display().to_string(),
        release_notes_path: context.target.release_notes_path.display().to_string(),
        entry_count,
        summary_line,
        tag_name: context
            .existing
            .as_ref()
            .and_then(|facts| facts.tag_name.clone()),
        tag_commit_sha: context
            .existing
            .as_ref()
            .and_then(|facts| facts.tag_commit_sha.clone()),
        tag_proof_path: context
            .existing
            .as_ref()
            .and_then(|facts| facts.tag_proof_path.clone()),
        remote_provider: context
            .existing
            .as_ref()
            .and_then(|facts| facts.remote_provider.clone()),
        remote_release_id: context
            .existing
            .as_ref()
            .and_then(|facts| facts.remote_release_id.clone()),
        remote_release_url: context
            .existing
            .as_ref()
            .and_then(|facts| facts.remote_release_url.clone()),
        remote_release_commit_sha: context
            .existing
            .as_ref()
            .and_then(|facts| facts.remote_release_commit_sha.clone()),
        remote_release_proof_path: context
            .existing
            .as_ref()
            .and_then(|facts| facts.remote_release_proof_path.clone()),
        artifact_manifest_path: context
            .existing
            .as_ref()
            .and_then(|facts| facts.artifact_manifest_path.clone()),
        artifact_manifest_sha256: context
            .existing
            .as_ref()
            .and_then(|facts| facts.artifact_manifest_sha256.clone()),
        latest_event_id,
        published_at,
        updated_at: now,
    }
}

fn persist_release_facts(
    context: &ReleaseRuntimeContext,
    facts: ProjectReleaseFacts,
) -> Result<ProjectReleaseFacts> {
    write_project_release_facts(&context.root, &facts)?;
    write_project_release_index(&context.root)?;
    sync_project_external_review_surface(&context.root, &facts)?;
    Ok(facts)
}

fn transition_release_state(
    root: &Path,
    project_id: &str,
    current_state: &str,
    event_type: &str,
    target: &PublicReleaseDocumentTarget,
    entry_count: usize,
    gate: &ReleaseGate,
) -> Result<Option<String>> {
    let guards = match event_type {
        "delivery.ready" => StaticGuardRegistry::all_pass(["delivery.input.ready"]),
        "delivery.started" => StaticGuardRegistry::all_pass(["delivery.public_record.ready"]),
        "delivery.published" => StaticGuardRegistry::all_pass(["delivery.publish.confirmed"]),
        _ => StaticGuardRegistry::default(),
    };
    let actions = match event_type {
        "delivery.ready" => StaticActionRegistry::all_complete(["delivery.ready.write"]),
        "delivery.started" => StaticActionRegistry::all_complete(["delivery.summary.write"]),
        "delivery.published" => StaticActionRegistry::all_complete(["delivery.publish.write"]),
        _ => StaticActionRegistry::default(),
    };
    let payload = json!({
        "projectId": project_id,
        "completionState": gate.completion_state,
        "completionOutcome": gate.completion_outcome,
        "deliveryStatus": gate.delivery_status,
        "changelogPath": target.changelog_path.display().to_string(),
        "releaseNotesUrl": target.release_notes_path.display().to_string(),
        "releaseEntryCount": entry_count,
    });
    let transition = apply_canonical_workflow_event(
        root,
        WorkflowFlowType::Delivery,
        current_state,
        event_type,
        RuntimeContext {
            actor: EventActor {
                role: "release-runtime".to_string(),
                kind: "system".to_string(),
            },
            payload,
            ..RuntimeContext::project(
                project_id.to_string(),
                EventActor {
                    role: "release-runtime".to_string(),
                    kind: "system".to_string(),
                },
            )
        },
        &guards,
        &actions,
    )?;
    Ok(transition.event_id)
}

fn evaluate_release_gate(projection: &ProjectProjectionSnapshot) -> ReleaseGate {
    let completion_state = projection
        .completion
        .as_ref()
        .map(|completion| completion.current_state.clone())
        .unwrap_or_else(|| "missing".to_string());
    let completion_outcome = projection
        .completion
        .as_ref()
        .and_then(|completion| completion.latest_outcome.clone());
    let delivery_status = projection
        .delivery
        .as_ref()
        .map(|delivery| delivery.status.clone())
        .unwrap_or_else(|| "missing".to_string());

    let completion_accepted = projection.completion.as_ref().is_some_and(|completion| {
        completion.current_state == "accepted"
            || completion.latest_outcome.as_deref() == Some("accept")
    });
    if !completion_accepted {
        return ReleaseGate {
            current_state: "pending".to_string(),
            gate_status: "waiting-completion".to_string(),
            gate_reason: "项目还没有通过 completion accept，release 不能开始。".to_string(),
            completion_state,
            completion_outcome,
            delivery_status,
        };
    }

    if projection
        .completion
        .as_ref()
        .is_some_and(|completion| completion.release_readiness != "ready")
    {
        return ReleaseGate {
            current_state: "blocked".to_string(),
            gate_status: "blocked".to_string(),
            gate_reason: "completion 已接受，但完成门仍缺少证据、交付、审计或目标满足度证明。"
                .to_string(),
            completion_state,
            completion_outcome,
            delivery_status,
        };
    }

    let delivery = projection.delivery.as_ref();
    if delivery.is_none() {
        return ReleaseGate {
            current_state: "blocked".to_string(),
            gate_status: "blocked".to_string(),
            gate_reason: "项目缺少公开交付摘要，release 不能开始。".to_string(),
            completion_state,
            completion_outcome,
            delivery_status,
        };
    }
    let delivery = delivery.unwrap();
    if delivery.missing_count > 0 {
        return ReleaseGate {
            current_state: "blocked".to_string(),
            gate_status: "blocked".to_string(),
            gate_reason: if delivery.summary_line.trim().is_empty() {
                "项目公开交付记录还没整理完整。".to_string()
            } else {
                delivery.summary_line.clone()
            },
            completion_state,
            completion_outcome,
            delivery_status,
        };
    }

    ReleaseGate {
        current_state: "ready".to_string(),
        gate_status: "ready".to_string(),
        gate_reason: "项目已经完成 completion accept，公开交付记录也已齐备，可以进入正式 release。"
            .to_string(),
        completion_state,
        completion_outcome,
        delivery_status,
    }
}

fn load_project_projection_snapshot(
    root: &Path,
    project_id: &str,
) -> Result<ProjectProjectionSnapshot> {
    let project_id = ProjectId::parse(project_id)?;
    read_json(&join_relative_path(
        root,
        PathBuf::from(".agentflow")
            .join("projections")
            .join("projects")
            .join(format!("{}.json", project_id.as_str())),
    )?)
}

fn write_project_release_facts(root: &Path, facts: &ProjectReleaseFacts) -> Result<()> {
    ensure_directory(&root.join(".agentflow/release/projects"))?;
    write_json(&project_release_facts_path(root, &facts.project_id)?, facts)
}

fn write_project_release_index(root: &Path) -> Result<()> {
    ensure_directory(&root.join(".agentflow/release/projects"))?;
    ensure_directory(&root.join(".agentflow/indexes"))?;
    let mut releases = Vec::new();
    for entry in fs::read_dir(root.join(".agentflow/release/projects"))
        .with_context(|| "read .agentflow/release/projects".to_string())?
    {
        let entry = entry?;
        if entry.path().extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        let facts: ProjectReleaseFacts = read_json(&entry.path())?;
        releases.push(ProjectReleaseIndexEntry {
            project_id: facts.project_id,
            current_state: facts.current_state,
            publication_stage: facts.publication_stage,
            gate_status: facts.gate_status,
            changelog_path: facts.changelog_path,
            release_notes_path: facts.release_notes_path,
            published_at: facts.published_at,
            updated_at: facts.updated_at,
        });
    }
    releases.sort_by(|left, right| left.project_id.cmp(&right.project_id));
    write_json(
        &root.join(".agentflow/indexes/releases.json"),
        &ProjectReleaseIndex {
            version: PROJECT_RELEASE_INDEX_VERSION.to_string(),
            updated_at: unix_timestamp_seconds(),
            releases,
        },
    )
}

fn project_release_facts_path(root: &Path, project_id: &str) -> Result<PathBuf> {
    let project_id = ProjectId::parse(project_id)?;
    join_relative_path(
        root,
        PathBuf::from(".agentflow")
            .join("release")
            .join("projects")
            .join(format!("{}.json", project_id.as_str())),
    )
}

fn project_release_tag_proof_path(project_id: &str) -> Result<PathBuf> {
    let project_id = ProjectId::parse(project_id)?;
    Ok(PathBuf::from(".agentflow")
        .join("release")
        .join("proofs")
        .join(project_id.as_str())
        .join("tag.json"))
}

fn project_remote_release_proof_path(project_id: &str) -> Result<PathBuf> {
    let project_id = ProjectId::parse(project_id)?;
    Ok(PathBuf::from(".agentflow")
        .join("release")
        .join("proofs")
        .join(project_id.as_str())
        .join("remote-release.json"))
}

fn default_project_release_target(project_id: &str) -> Result<PublicReleaseDocumentTarget> {
    let project_id = ProjectId::parse(project_id)?;
    Ok(PublicReleaseDocumentTarget {
        changelog_path: PathBuf::from("CHANGELOG.md"),
        release_notes_path: PathBuf::from(format!("docs/release-notes/{}.md", project_id.as_str())),
    })
}

fn resolve_workspace_path(root: &Path, raw: &str) -> Result<PathBuf> {
    let candidate = PathBuf::from(raw);
    let path = if candidate.is_absolute() {
        candidate
    } else {
        root.join(candidate)
    };
    if !path.is_file() {
        anyhow::bail!("release artifact manifest is missing: {}", path.display());
    }
    Ok(path)
}

fn workspace_or_absolute_display(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .map(|relative| relative.display().to_string())
        .unwrap_or_else(|_| path.display().to_string())
}

fn sha256_file(path: &Path) -> Result<String> {
    let bytes = fs::read(path).with_context(|| format!("read {}", path.display()))?;
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    Ok(format!("{:x}", hasher.finalize()))
}

fn ensure_directory(path: &Path) -> Result<()> {
    fs::create_dir_all(path).with_context(|| format!("create {}", path.display()))
}

fn write_json<T: serde::Serialize>(path: &Path, value: &T) -> Result<()> {
    if let Some(parent) = path.parent() {
        ensure_directory(parent)?;
    }
    fs::write(path, serde_json::to_string_pretty(value)? + "\n")
        .with_context(|| format!("write {}", path.display()))
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", path.display()))
}

fn unix_timestamp_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use agentflow_event_store::load_task_events;
    use agentflow_spec::{
        issue_from_requirement, project_from_requirement, write_spec_issue, write_spec_project,
        SpecIssueDraft, SpecProjectDraft,
    };
    use tempfile::tempdir;

    fn write_requirement(root: &Path) -> PathBuf {
        let path = root.join("docs/requirements/054-release-runtime-test.md");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(
            &path,
            "# Release Runtime Test\n\n用于 release runtime 测试。\n",
        )
        .unwrap();
        path
    }

    fn write_project_fixture(root: &Path) -> String {
        let requirement = write_requirement(root);
        let mut project_draft = SpecProjectDraft::new("project-release-runtime");
        project_draft.title = Some("Release Runtime Project".to_string());
        project_draft.summary = Some("release runtime test".to_string());
        project_draft.objective = Some("验证 release runtime".to_string());
        project_draft.issue_ids = vec!["AF-REL-001".to_string()];
        let project = project_from_requirement(root, &requirement, project_draft).unwrap();
        write_spec_project(root, &project).unwrap();

        let mut issue_draft = SpecIssueDraft::new("AF-REL-001");
        issue_draft.project_id = Some(project.project_id.clone());
        issue_draft.title = Some("完成公开交付".to_string());
        issue_draft.summary = Some("整理公开交付记录。".to_string());
        let issue = issue_from_requirement(root, &requirement, issue_draft).unwrap();
        write_spec_issue(root, &issue).unwrap();
        project.project_id
    }

    fn write_project_projection(
        root: &Path,
        project_id: &str,
        completion_state: &str,
        completion_outcome: Option<&str>,
        release_readiness: &str,
        delivery_status: &str,
        missing_count: usize,
    ) {
        let projection = serde_json::json!({
            "projectId": project_id,
            "title": "Release Runtime Project",
            "completion": {
                "currentState": completion_state,
                "latestOutcome": completion_outcome,
                "releaseReadiness": release_readiness,
            },
            "delivery": {
                "status": delivery_status,
                "missingCount": missing_count,
                "summaryLine": if missing_count == 0 { "公开交付已统一写入 PR/MR body、CHANGELOG.md。"} else {"项目公开交付仍缺少 CHANGELOG.md 或 release notes。"}
            }
        });
        let path = root.join(".agentflow/projections/projects");
        fs::create_dir_all(&path).unwrap();
        fs::write(
            path.join(format!("{project_id}.json")),
            serde_json::to_string_pretty(&projection).unwrap(),
        )
        .unwrap();
    }

    fn write_task_projection(root: &Path, issue_id: &str, project_id: &str) {
        let projection = serde_json::json!({
            "issueId": issue_id,
            "projectId": project_id,
            "currentState": "done",
            "publicDelivery": {
                "prUrl": "https://github.com/acme/repo/pull/1",
                "mergeCommit": "merge-1",
                "changelogPath": "CHANGELOG.md",
                "releaseNotesUrl": "docs/release-notes/project-release-runtime.md"
            },
            "delivery": {
                "status": "published",
                "evidenceStatus": "ready",
                "evidencePath": ".agentflow/tasks/AF-REL-001/evidence/evidence.json",
                "prUrl": "https://github.com/acme/repo/pull/1",
                "mergeCommit": "merge-1",
                "publicRecordPath": "CHANGELOG.md"
            },
            "updatedAt": 10
        });
        let path = root.join(".agentflow/projections/tasks");
        fs::create_dir_all(&path).unwrap();
        fs::write(
            path.join(format!("{issue_id}.json")),
            serde_json::to_string_pretty(&projection).unwrap(),
        )
        .unwrap();
    }

    fn write_release_manifest(root: &Path, project_id: &str) -> String {
        let path = root
            .join("artifacts")
            .join(format!("{project_id}-release-manifest.json"));
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(
            &path,
            serde_json::to_string_pretty(&serde_json::json!({
                "projectId": project_id,
                "artifacts": ["CHANGELOG.md", format!("docs/release-notes/{project_id}.md")],
                "generatedBy": "release-runtime-test"
            }))
            .unwrap(),
        )
        .unwrap();
        format!("artifacts/{project_id}-release-manifest.json")
    }

    #[test]
    fn sync_project_release_stops_at_public_record_written_without_remote_proof() {
        let dir = tempdir().unwrap();
        let project_id = write_project_fixture(dir.path());
        write_task_projection(dir.path(), "AF-REL-001", &project_id);
        write_project_projection(
            dir.path(),
            &project_id,
            "accepted",
            Some("accept"),
            "ready",
            "published",
            0,
        );

        let facts = sync_project_release(dir.path(), &project_id).unwrap();
        let events = load_task_events(dir.path()).unwrap();
        let delivery_events = events
            .into_iter()
            .filter(|event| event.project_id.as_deref() == Some(project_id.as_str()))
            .map(|event| event.event_type)
            .collect::<Vec<_>>();

        assert_eq!(facts.current_state, "in_progress");
        assert_eq!(facts.publication_stage, "public-record-written");
        assert_eq!(facts.gate_status, "ready");
        assert_eq!(facts.entry_count, 1);
        assert_eq!(
            delivery_events,
            vec!["delivery.ready".to_string(), "delivery.started".to_string(),]
        );
        assert!(dir.path().join("CHANGELOG.md").is_file());
        assert!(dir
            .path()
            .join("docs/release-notes/project-release-runtime.md")
            .is_file());
        assert!(dir
            .path()
            .join("docs/reviews/project-release-runtime.md")
            .is_file());
        assert!(dir
            .path()
            .join(".agentflow/release/reviews/project-release-runtime.json")
            .is_file());
        let review =
            crate::review_surface::load_project_external_review_surface(dir.path(), &project_id)
                .unwrap();
        assert_eq!(review.review_status, "not-ready");
        assert_eq!(review.total_entries, 1);
        let index = load_project_release_index(dir.path()).unwrap();
        assert_eq!(index.releases.len(), 1);
        assert_eq!(index.releases[0].publication_stage, "public-record-written");
    }

    #[test]
    fn publish_project_release_requires_remote_release_proof() {
        let dir = tempdir().unwrap();
        let project_id = write_project_fixture(dir.path());
        write_task_projection(dir.path(), "AF-REL-001", &project_id);
        write_project_projection(
            dir.path(),
            &project_id,
            "accepted",
            Some("accept"),
            "ready",
            "published",
            0,
        );

        let confirmed = confirm_project_release(dir.path(), &project_id).unwrap();
        assert_eq!(confirmed.publication_stage, "public-record-written");
        let err = publish_project_release(dir.path(), &project_id)
            .unwrap_err()
            .to_string();
        assert!(err.contains("remote release proof"));
    }

    #[test]
    fn release_publish_requires_tag_and_remote_proof_chain() {
        let dir = tempdir().unwrap();
        let project_id = write_project_fixture(dir.path());
        write_task_projection(dir.path(), "AF-REL-001", &project_id);
        write_project_projection(
            dir.path(),
            &project_id,
            "accepted",
            Some("accept"),
            "ready",
            "published",
            0,
        );

        let confirmed = confirm_project_release(dir.path(), &project_id).unwrap();
        assert_eq!(confirmed.publication_stage, "public-record-written");
        let tagged = record_project_release_tag(
            dir.path(),
            &project_id,
            "v0.5.1",
            "tag-commit-001",
            "release-agent",
        )
        .unwrap();
        assert_eq!(tagged.publication_stage, "tag-created");
        let manifest_path = write_release_manifest(dir.path(), &project_id);
        let remote = record_project_remote_release(
            dir.path(),
            &project_id,
            "github",
            "rel-001",
            "https://github.com/acme/repo/releases/tag/v0.5.1",
            "v0.5.1",
            "tag-commit-001",
            &manifest_path,
            "release-agent",
        )
        .unwrap();
        assert_eq!(remote.publication_stage, "remote-release-created");
        assert_eq!(remote.remote_provider.as_deref(), Some("github"));
        assert_eq!(remote.tag_name.as_deref(), Some("v0.5.1"));
        assert!(remote.artifact_manifest_sha256.is_some());

        let published = publish_project_release(dir.path(), &project_id).unwrap();
        assert_eq!(published.current_state, "published");
        assert_eq!(published.publication_stage, "published");
        assert_eq!(published.remote_release_id.as_deref(), Some("rel-001"));
        assert!(published.remote_release_proof_path.is_some());
    }

    #[test]
    fn sync_project_release_blocks_when_completion_is_not_accepted() {
        let dir = tempdir().unwrap();
        let project_id = write_project_fixture(dir.path());
        write_task_projection(dir.path(), "AF-REL-001", &project_id);
        write_project_projection(
            dir.path(),
            &project_id,
            "continue",
            Some("continue"),
            "blocked-remaining-issues",
            "ready",
            0,
        );

        let facts = sync_project_release(dir.path(), &project_id).unwrap();

        assert_eq!(facts.current_state, "pending");
        assert_eq!(facts.gate_status, "waiting-completion");
        assert!(facts.gate_reason.contains("completion accept"));
    }

    #[test]
    fn sync_project_release_blocks_when_public_delivery_is_incomplete() {
        let dir = tempdir().unwrap();
        let project_id = write_project_fixture(dir.path());
        write_task_projection(dir.path(), "AF-REL-001", &project_id);
        write_project_projection(
            dir.path(),
            &project_id,
            "accepted",
            Some("accept"),
            "blocked-missing-delivery",
            "ready",
            1,
        );

        let facts = sync_project_release(dir.path(), &project_id).unwrap();

        assert_eq!(facts.current_state, "blocked");
        assert_eq!(facts.gate_status, "blocked");
        assert!(facts.gate_reason.contains("完成门"));
    }

    #[test]
    fn sync_project_release_rejects_path_like_project_ids() {
        let dir = tempdir().unwrap();
        let err = sync_project_release(dir.path(), "../bad")
            .unwrap_err()
            .to_string();
        assert!(err.contains("safe local id"));
    }
}
