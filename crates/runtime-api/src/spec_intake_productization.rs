//! Product-level Spec Intake productization runtime.
//!
//! This module owns the v1.1.5 preview-first bridge:
//! human intent -> Product route preview -> confirmation -> materialized
//! `docs/requirements/**` and `.agentflow/spec/**` authority. Preview artifacts
//! deliberately live outside `.agentflow/spec/**` so unconfirmed input cannot
//! become task authority by accident.

use agentflow_spec::{
    confirm_goal_draft_preview, confirm_plan_draft_preview,
    materialize_spec_from_requirement_preview, requirement_preview_from_requirement,
    CoreIntakeRoute,
};
use agentflow_workflow_core::{canonicalize_project_root, normalize_relative_to_root};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

pub const PRODUCT_SPEC_INTAKE_VERSION: &str = "agentflow-product-spec-intake.v1";
pub const PRODUCT_SPEC_PREVIEW_VERSION: &str = "agentflow-product-spec-preview.v1";
pub const PRODUCT_SPEC_CONFIRMATION_VERSION: &str = "agentflow-product-spec-confirmation.v1";
pub const PRODUCT_SPEC_MATERIALIZATION_VERSION: &str = "agentflow-product-spec-materialization.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductIntentIntakeRequest {
    pub raw_text: String,
    pub selected_product_id: String,
    pub workspace_id: String,
    pub source_surface: String,
    #[serde(default = "default_locale")]
    pub locale: String,
    #[serde(default)]
    pub attachment_refs: Vec<String>,
    #[serde(default)]
    pub source_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductIntentProductMapping {
    pub product_id: String,
    pub pack_id: String,
    pub product_name: String,
    pub source_boundary: String,
    pub product_terms: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductCoreRouteDecision {
    pub route: CoreIntakeRoute,
    pub reasons: Vec<String>,
    pub missing_inputs: Vec<String>,
    pub allowed_next_actions: Vec<String>,
    pub write_boundary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductTaskPreview {
    pub task_id: String,
    pub title: String,
    pub summary: String,
    pub dependencies: Vec<String>,
    pub acceptance_criteria: Vec<String>,
    pub non_goals: Vec<String>,
    pub validation_commands: Vec<String>,
    pub product_language: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductSpecPreviewArtifact {
    pub version: String,
    pub preview_id: String,
    pub preview_hash: String,
    pub receipt_id: String,
    pub requirement_id: String,
    pub project_id: String,
    pub selected_product_id: String,
    pub workspace_id: String,
    pub source_surface: String,
    pub locale: String,
    pub raw_text: String,
    pub normalized_summary: String,
    pub route_decision: ProductCoreRouteDecision,
    pub goal_preview: String,
    pub roadmap_preview: Vec<String>,
    pub task_previews: Vec<ProductTaskPreview>,
    pub product_mapping: ProductIntentProductMapping,
    pub writes_authority: bool,
    pub preview_artifact_ref: String,
    pub created_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductIntentIntakeReceipt {
    pub version: String,
    pub receipt_id: String,
    pub status: String,
    pub writes_authority: bool,
    pub selected_product_id: String,
    pub workspace_id: String,
    pub source_surface: String,
    pub locale: String,
    pub preview_id: String,
    pub preview_hash: String,
    pub preview_artifact_ref: String,
    pub route_decision: ProductCoreRouteDecision,
    pub missing_inputs: Vec<String>,
    pub next_actions: Vec<String>,
    pub product_mapping: ProductIntentProductMapping,
    pub local_diagnostics: Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProductSpecPreviewDecision {
    Confirm,
    Reject,
    Revise,
    Expire,
}

impl ProductSpecPreviewDecision {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Confirm => "confirm",
            Self::Reject => "reject",
            Self::Revise => "revise",
            Self::Expire => "expire",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductSpecConfirmationRequest {
    pub preview_id: String,
    pub preview_hash: String,
    pub actor: String,
    pub decision: ProductSpecPreviewDecision,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductSpecConfirmationRecord {
    pub version: String,
    pub confirmation_id: String,
    pub preview_id: String,
    pub preview_hash: String,
    pub preview_artifact_ref: String,
    pub actor: String,
    pub decision: ProductSpecPreviewDecision,
    pub accepted: bool,
    pub immutable_binding: bool,
    pub summary: String,
    pub created_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductSpecMaterializationReport {
    pub version: String,
    pub status: String,
    pub preview_id: String,
    pub preview_hash: String,
    pub confirmation_id: String,
    pub requirement_id: String,
    pub project_id: String,
    pub docs_requirement_path: String,
    pub spec_project_path: String,
    pub spec_issue_paths: Vec<String>,
    pub materialized_issue_ids: Vec<String>,
    pub traceability: Vec<String>,
    pub blocked_reason: Option<String>,
    pub created_at: u64,
}

pub fn preview_product_intent(
    product_source_root: impl AsRef<Path>,
    workspace_root: impl AsRef<Path>,
    request: ProductIntentIntakeRequest,
) -> Result<ProductIntentIntakeReceipt> {
    let source_root = canonicalize_project_root(product_source_root)?;
    let workspace_root = ensure_workspace_root(workspace_root)?;
    let mapping = load_product_mapping(&source_root, &request.selected_product_id)?;
    let now = unix_timestamp_seconds();
    let receipt_id = format!(
        "intent-{:016x}",
        stable_hash(
            format!(
                "{}:{}:{}:{}",
                request.workspace_id, request.selected_product_id, request.raw_text, now
            )
            .as_bytes(),
        )
    );
    let requirement_id = format!(
        "{}-{}",
        sanitize_id(&request.workspace_id),
        &receipt_id[7..15]
    );
    let preview_id = format!("preview-{requirement_id}");
    let route_decision = decide_core_route(&request.raw_text);
    let artifact_ref = preview_artifact_ref(&preview_id);
    let mut artifact = ProductSpecPreviewArtifact {
        version: PRODUCT_SPEC_PREVIEW_VERSION.to_string(),
        preview_id: preview_id.clone(),
        preview_hash: String::new(),
        receipt_id: receipt_id.clone(),
        requirement_id: requirement_id.clone(),
        project_id: sanitize_id(&request.workspace_id),
        selected_product_id: request.selected_product_id.clone(),
        workspace_id: request.workspace_id.clone(),
        source_surface: request.source_surface.clone(),
        locale: request.locale.clone(),
        raw_text: request.raw_text.clone(),
        normalized_summary: normalize_summary(&request.raw_text),
        route_decision: route_decision.clone(),
        goal_preview: format!("把输入意图整理成 {} 的可确认目标。", mapping.product_name),
        roadmap_preview: build_roadmap_preview(&route_decision),
        task_previews: build_task_previews(&requirement_id, &route_decision, &mapping),
        product_mapping: mapping.clone(),
        writes_authority: false,
        preview_artifact_ref: artifact_ref.clone(),
        created_at: now,
    };
    artifact.preview_hash = preview_hash(&artifact)?;
    write_json(workspace_root.join(&artifact_ref), &artifact)?;

    Ok(ProductIntentIntakeReceipt {
        version: PRODUCT_SPEC_INTAKE_VERSION.to_string(),
        receipt_id,
        status: "preview-created".to_string(),
        writes_authority: false,
        selected_product_id: request.selected_product_id,
        workspace_id: request.workspace_id,
        source_surface: request.source_surface,
        locale: request.locale,
        preview_id,
        preview_hash: artifact.preview_hash,
        preview_artifact_ref: artifact_ref,
        route_decision: route_decision.clone(),
        missing_inputs: route_decision.missing_inputs,
        next_actions: route_decision.allowed_next_actions,
        product_mapping: mapping,
        local_diagnostics: json!({
            "workspaceRoot": workspace_root.display().to_string(),
            "productSourceRoot": source_root.display().to_string(),
            "attachmentRefs": request.attachment_refs,
            "sourceRefs": request.source_refs,
        }),
    })
}

pub fn confirm_product_spec_preview(
    workspace_root: impl AsRef<Path>,
    request: ProductSpecConfirmationRequest,
) -> Result<ProductSpecConfirmationRecord> {
    let workspace_root = canonicalize_project_root(workspace_root)?;
    let artifact = read_preview_artifact(&workspace_root, &request.preview_id)?;
    let actual_hash = preview_hash(&artifact)?;
    if actual_hash != request.preview_hash || artifact.preview_hash != request.preview_hash {
        anyhow::bail!(
            "preview hash mismatch for {}: expected {}, found {}",
            request.preview_id,
            request.preview_hash,
            actual_hash
        );
    }
    let accepted = request.decision == ProductSpecPreviewDecision::Confirm;
    let now = unix_timestamp_seconds();
    let record = ProductSpecConfirmationRecord {
        version: PRODUCT_SPEC_CONFIRMATION_VERSION.to_string(),
        confirmation_id: format!(
            "confirmation-{:016x}",
            stable_hash(
                format!("{}:{}:{now}", request.preview_id, request.preview_hash).as_bytes()
            )
        ),
        preview_id: request.preview_id,
        preview_hash: request.preview_hash,
        preview_artifact_ref: artifact.preview_artifact_ref,
        actor: request.actor,
        decision: request.decision,
        accepted,
        immutable_binding: true,
        summary: request.summary,
        created_at: now,
    };
    write_json(
        workspace_root.join(confirmation_artifact_ref(&record.preview_id)),
        &record,
    )?;
    Ok(record)
}

pub fn materialize_confirmed_product_spec(
    workspace_root: impl AsRef<Path>,
    preview_id: &str,
) -> Result<ProductSpecMaterializationReport> {
    let workspace_root = canonicalize_project_root(workspace_root)?;
    let artifact = read_preview_artifact(&workspace_root, preview_id)?;
    let confirmation = read_confirmation_record(&workspace_root, preview_id)?;
    let actual_hash = preview_hash(&artifact)?;
    if actual_hash != confirmation.preview_hash
        || artifact.preview_hash != confirmation.preview_hash
    {
        anyhow::bail!(
            "preview {} was modified after confirmation; materialization rejected",
            preview_id
        );
    }
    if !confirmation.accepted || confirmation.decision != ProductSpecPreviewDecision::Confirm {
        anyhow::bail!(
            "preview {} is not confirmed: {}",
            preview_id,
            confirmation.decision.as_str()
        );
    }
    if matches!(
        artifact.route_decision.route,
        CoreIntakeRoute::Clarify | CoreIntakeRoute::Research
    ) {
        anyhow::bail!(
            "route {} cannot materialize authority",
            artifact.route_decision.route.as_str()
        );
    }
    let materialization_path = workspace_root.join(materialization_artifact_ref(preview_id));
    if materialization_path.is_file() {
        let payload = fs::read_to_string(&materialization_path)
            .with_context(|| format!("read {}", materialization_path.display()))?;
        let report: ProductSpecMaterializationReport = serde_json::from_str(&payload)?;
        if report.preview_hash == artifact.preview_hash
            && report.confirmation_id == confirmation.confirmation_id
        {
            return Ok(report);
        }
        anyhow::bail!(
            "materialization record for {} does not match current confirmation",
            preview_id
        );
    }

    let requirement_path = write_requirement_authority(&workspace_root, &artifact, &confirmation)?;
    let mut preview = requirement_preview_from_requirement(
        &workspace_root,
        &requirement_path,
        Some(&artifact.project_id),
    )?;
    preview = confirm_goal_draft_preview(&workspace_root, &preview.requirement_id, "goal-agent")?;
    preview = confirm_plan_draft_preview(&workspace_root, &preview.requirement_id, "spec-agent")?;
    let (project, issues) =
        materialize_spec_from_requirement_preview(&workspace_root, &preview.requirement_id)?;
    let report = ProductSpecMaterializationReport {
        version: PRODUCT_SPEC_MATERIALIZATION_VERSION.to_string(),
        status: "materialized".to_string(),
        preview_id: artifact.preview_id,
        preview_hash: artifact.preview_hash,
        confirmation_id: confirmation.confirmation_id,
        requirement_id: preview.requirement_id,
        project_id: project.project_id.clone(),
        docs_requirement_path: normalize_relative_to_root(&workspace_root, &requirement_path)?,
        spec_project_path: format!(".agentflow/spec/projects/{}.json", project.project_id),
        spec_issue_paths: issues
            .iter()
            .map(|issue| format!(".agentflow/spec/issues/{}.json", issue.issue_id))
            .collect(),
        materialized_issue_ids: issues.iter().map(|issue| issue.issue_id.clone()).collect(),
        traceability: vec![
            artifact.receipt_id,
            confirmation.preview_artifact_ref,
            format!(".agentflow/spec/projects/{}.json", project.project_id),
        ],
        blocked_reason: None,
        created_at: unix_timestamp_seconds(),
    };
    write_json(materialization_path, &report)?;
    Ok(report)
}

pub fn read_product_spec_preview(
    workspace_root: impl AsRef<Path>,
    preview_id: &str,
) -> Result<ProductSpecPreviewArtifact> {
    let workspace_root = canonicalize_project_root(workspace_root)?;
    read_preview_artifact(&workspace_root, preview_id)
}

pub fn read_product_spec_confirmation(
    workspace_root: impl AsRef<Path>,
    preview_id: &str,
) -> Result<ProductSpecConfirmationRecord> {
    let workspace_root = canonicalize_project_root(workspace_root)?;
    read_confirmation_record(&workspace_root, preview_id)
}

fn load_product_mapping(
    product_source_root: &Path,
    product_id: &str,
) -> Result<ProductIntentProductMapping> {
    let registry = agentflow_pack::load_product_registry(product_source_root)?;
    let entry = registry
        .product(product_id)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("product `{product_id}` is not registered"))?;
    if !entry.valid {
        anyhow::bail!("product `{product_id}` is invalid: {:?}", entry.diagnostics);
    }
    Ok(ProductIntentProductMapping {
        product_id: entry.product_id,
        pack_id: entry.pack_id,
        product_name: entry.name,
        source_boundary: entry.source_boundary,
        product_terms: vec![
            "requirement".to_string(),
            "goal".to_string(),
            "roadmap".to_string(),
            "task".to_string(),
        ],
    })
}

fn decide_core_route(raw_text: &str) -> ProductCoreRouteDecision {
    let normalized = raw_text.to_ascii_lowercase();
    let (route, reason) =
        if raw_text.trim().len() < 12 || raw_text.contains('?') || raw_text.contains('？') {
            (CoreIntakeRoute::Clarify, "输入信息不足，需要先澄清。")
        } else if contains_any(&normalized, &["research", "调研", "查一下", "分析一下"]) {
            (CoreIntakeRoute::Research, "当前请求以事实研究为主。")
        } else if contains_any(&normalized, &["plan", "roadmap", "规划", "计划"]) {
            (CoreIntakeRoute::Plan, "当前请求需要先形成计划。")
        } else if contains_any(&normalized, &["decide", "确认", "决策", "取舍"]) {
            (CoreIntakeRoute::Decide, "当前请求需要形成决策。")
        } else if contains_any(&normalized, &["deliver", "交付", "发布", "release"]) {
            (CoreIntakeRoute::Deliver, "当前请求关注交付输出。")
        } else if contains_any(&normalized, &["evolve", "迭代", "升级", "演进"]) {
            (CoreIntakeRoute::Evolve, "当前请求关注演进反馈。")
        } else if contains_any(&normalized, &["define", "定义", "目标"]) {
            (CoreIntakeRoute::Define, "当前请求需要定义目标。")
        } else {
            (CoreIntakeRoute::Task, "当前请求可以形成可执行任务预览。")
        };
    let mut missing_inputs = Vec::new();
    if route == CoreIntakeRoute::Clarify {
        missing_inputs.push("clear-outcome".to_string());
    }
    let write_boundary = match route {
        CoreIntakeRoute::Clarify | CoreIntakeRoute::Research => "no-authority-write",
        _ => "preview-only-until-confirmed",
    }
    .to_string();
    let allowed_next_actions = allowed_next_actions_for_route(&route);
    ProductCoreRouteDecision {
        route,
        reasons: vec![reason.to_string()],
        missing_inputs,
        allowed_next_actions,
        write_boundary,
    }
}

fn allowed_next_actions_for_route(route: &CoreIntakeRoute) -> Vec<String> {
    match route {
        CoreIntakeRoute::Clarify => {
            vec![
                "clarify-requirement".to_string(),
                "regenerate-preview".to_string(),
            ]
        }
        CoreIntakeRoute::Research => {
            vec![
                "prepare-research-response".to_string(),
                "regenerate-preview".to_string(),
            ]
        }
        _ => vec![
            "preview".to_string(),
            "confirm".to_string(),
            "materialize-after-confirmation".to_string(),
        ],
    }
}

fn build_roadmap_preview(route: &ProductCoreRouteDecision) -> Vec<String> {
    match route.route {
        CoreIntakeRoute::Clarify => vec!["补齐缺失信息".to_string(), "重新生成预览".to_string()],
        CoreIntakeRoute::Research => vec!["收集事实".to_string(), "生成研究边界".to_string()],
        _ => vec![
            "确认目标".to_string(),
            "拆解路线图".to_string(),
            "生成任务合同预览".to_string(),
        ],
    }
}

fn build_task_previews(
    requirement_id: &str,
    route: &ProductCoreRouteDecision,
    mapping: &ProductIntentProductMapping,
) -> Vec<ProductTaskPreview> {
    if matches!(
        route.route,
        CoreIntakeRoute::Clarify | CoreIntakeRoute::Research
    ) {
        return Vec::new();
    }
    vec![
        ProductTaskPreview {
            task_id: format!("AF-{}-001", short_numeric_suffix(requirement_id)),
            title: "整理目标与路线图".to_string(),
            summary: "把用户意图整理成可确认 Goal / Roadmap。".to_string(),
            dependencies: Vec::new(),
            acceptance_criteria: vec!["Goal 与 Roadmap 预览可读。".to_string()],
            non_goals: vec!["不启动 Build Agent。".to_string()],
            validation_commands: vec!["cargo test -p agentflow-runtime-api".to_string()],
            product_language: vec![mapping.product_id.clone(), "product-task".to_string()],
        },
        ProductTaskPreview {
            task_id: format!("AF-{}-002", short_numeric_suffix(requirement_id)),
            title: "生成执行任务合同".to_string(),
            summary: "从确认后的 Spec Bundle 生成可执行任务合同。".to_string(),
            dependencies: vec![format!("AF-{}-001", short_numeric_suffix(requirement_id))],
            acceptance_criteria: vec!["SpecIssue 包含依赖、验收标准和验证命令。".to_string()],
            non_goals: vec!["不创建 GitHub issue 作为 authority。".to_string()],
            validation_commands: vec!["cargo test -p agentflow-spec".to_string()],
            product_language: vec![mapping.product_id.clone(), "issue-contract".to_string()],
        },
    ]
}

fn write_requirement_authority(
    root: &Path,
    artifact: &ProductSpecPreviewArtifact,
    confirmation: &ProductSpecConfirmationRecord,
) -> Result<PathBuf> {
    let path = root
        .join("docs/requirements")
        .join(format!("{}.md", artifact.requirement_id));
    let content = format!(
        "# {}\n\n## Requirement Authority\n- requirementId: {}\n- previewId: {}\n- previewHash: {}\n- confirmationId: {}\n- productId: {}\n- route: {}\n\n## Original Intent\n{}\n\n## Goal Preview\n{}\n\n## Roadmap Preview\n{}\n\n## Task Preview\n{}\n\n## Confirmation\n{}\n",
        artifact.normalized_summary,
        artifact.requirement_id,
        artifact.preview_id,
        artifact.preview_hash,
        confirmation.confirmation_id,
        artifact.selected_product_id,
        artifact.route_decision.route.as_str(),
        artifact.raw_text,
        artifact.goal_preview,
        markdown_list(&artifact.roadmap_preview),
        markdown_list(
            &artifact
                .task_previews
                .iter()
                .map(|task| format!("{} - {}", task.task_id, task.title))
                .collect::<Vec<_>>()
        ),
        confirmation.summary
    );
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&path, content)?;
    Ok(path)
}

fn read_preview_artifact(root: &Path, preview_id: &str) -> Result<ProductSpecPreviewArtifact> {
    let path = root.join(preview_artifact_ref(preview_id));
    let payload = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    Ok(serde_json::from_str(&payload)?)
}

fn read_confirmation_record(
    root: &Path,
    preview_id: &str,
) -> Result<ProductSpecConfirmationRecord> {
    let path = root.join(confirmation_artifact_ref(preview_id));
    let payload = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    Ok(serde_json::from_str(&payload)?)
}

fn preview_hash(artifact: &ProductSpecPreviewArtifact) -> Result<String> {
    let mut canonical = artifact.clone();
    canonical.preview_hash.clear();
    let bytes = serde_json::to_vec(&canonical)?;
    Ok(format!("{:016x}", stable_hash(&bytes)))
}

fn preview_artifact_ref(preview_id: &str) -> String {
    format!(".agentflow/previews/spec-intake/{preview_id}/preview.json")
}

fn confirmation_artifact_ref(preview_id: &str) -> String {
    format!(".agentflow/previews/spec-intake/{preview_id}/confirmation.json")
}

fn materialization_artifact_ref(preview_id: &str) -> String {
    format!(".agentflow/previews/spec-intake/{preview_id}/materialization.json")
}

fn ensure_workspace_root(path: impl AsRef<Path>) -> Result<PathBuf> {
    let path = path.as_ref();
    fs::create_dir_all(path)?;
    canonicalize_project_root(path)
}

fn write_json(path: PathBuf, payload: &impl Serialize) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let rendered = serde_json::to_string_pretty(payload)?;
    fs::write(path, format!("{rendered}\n"))?;
    Ok(())
}

fn markdown_list(values: &[String]) -> String {
    if values.is_empty() {
        "- none".to_string()
    } else {
        values
            .iter()
            .map(|value| format!("- {value}"))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

fn normalize_summary(raw_text: &str) -> String {
    raw_text
        .lines()
        .find(|line| !line.trim().is_empty())
        .unwrap_or("Product intent")
        .trim()
        .chars()
        .take(80)
        .collect()
}

fn sanitize_id(value: &str) -> String {
    let mut output = value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>();
    while output.contains("--") {
        output = output.replace("--", "-");
    }
    let trimmed = output.trim_matches('-');
    if trimmed.is_empty() {
        "agentflow-product".to_string()
    } else {
        trimmed.to_string()
    }
}

fn short_numeric_suffix(value: &str) -> String {
    format!("{:03}", stable_hash(value.as_bytes()) % 900 + 100)
}

fn contains_any(haystack: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| haystack.contains(needle))
}

fn default_locale() -> String {
    "zh-CN".to_string()
}

fn unix_timestamp_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

fn stable_hash(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn preview_does_not_write_authority_before_confirmation() {
        let root = workspace_root();
        let dir = tempdir().unwrap();
        let receipt = preview_product_intent(&root, dir.path(), software_request()).unwrap();
        assert!(!receipt.writes_authority);
        assert!(dir.path().join(&receipt.preview_artifact_ref).is_file());
        assert!(!dir.path().join("docs/requirements").exists());
        assert!(!dir.path().join(".agentflow/spec").exists());
    }

    #[test]
    fn confirmed_preview_materializes_docs_and_spec_authority() {
        let root = workspace_root();
        let dir = tempdir().unwrap();
        let receipt = preview_product_intent(&root, dir.path(), software_request()).unwrap();
        let confirmation = confirm_product_spec_preview(
            dir.path(),
            ProductSpecConfirmationRequest {
                preview_id: receipt.preview_id.clone(),
                preview_hash: receipt.preview_hash.clone(),
                actor: "human-owner".to_string(),
                decision: ProductSpecPreviewDecision::Confirm,
                summary: "确认进入 Spec materialization。".to_string(),
            },
        )
        .unwrap();
        assert!(confirmation.accepted);
        let report = materialize_confirmed_product_spec(dir.path(), &receipt.preview_id).unwrap();
        assert_eq!(report.status, "materialized");
        assert!(dir.path().join(&report.docs_requirement_path).is_file());
        assert!(dir.path().join(&report.spec_project_path).is_file());
        assert!(!report.spec_issue_paths.is_empty());
    }

    #[test]
    fn stale_or_rejected_preview_cannot_materialize() {
        let root = workspace_root();
        let dir = tempdir().unwrap();
        let receipt = preview_product_intent(&root, dir.path(), software_request()).unwrap();
        confirm_product_spec_preview(
            dir.path(),
            ProductSpecConfirmationRequest {
                preview_id: receipt.preview_id.clone(),
                preview_hash: receipt.preview_hash.clone(),
                actor: "human-owner".to_string(),
                decision: ProductSpecPreviewDecision::Reject,
                summary: "拒绝当前 preview。".to_string(),
            },
        )
        .unwrap();
        let err = materialize_confirmed_product_spec(dir.path(), &receipt.preview_id)
            .unwrap_err()
            .to_string();
        assert!(err.contains("not confirmed"));

        let receipt =
            preview_product_intent(&root, dir.path().join("stale"), software_request()).unwrap();
        confirm_product_spec_preview(
            dir.path().join("stale"),
            ProductSpecConfirmationRequest {
                preview_id: receipt.preview_id.clone(),
                preview_hash: receipt.preview_hash.clone(),
                actor: "human-owner".to_string(),
                decision: ProductSpecPreviewDecision::Confirm,
                summary: "确认。".to_string(),
            },
        )
        .unwrap();
        let path = dir.path().join("stale").join(&receipt.preview_artifact_ref);
        let mut artifact: ProductSpecPreviewArtifact =
            serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        artifact.normalized_summary.push_str(" modified");
        fs::write(&path, serde_json::to_string_pretty(&artifact).unwrap()).unwrap();
        let err = materialize_confirmed_product_spec(dir.path().join("stale"), &receipt.preview_id)
            .unwrap_err()
            .to_string();
        assert!(err.contains("modified after confirmation"));
    }

    #[test]
    fn no_authority_routes_only_expose_safe_next_actions() {
        let root = workspace_root();
        let dir = tempdir().unwrap();
        let clarify = preview_product_intent(
            &root,
            dir.path().join("clarify"),
            ProductIntentIntakeRequest {
                raw_text: "？".to_string(),
                selected_product_id: "software-dev".to_string(),
                workspace_id: "v116-clarify".to_string(),
                source_surface: "desktop-project-home".to_string(),
                locale: "zh-CN".to_string(),
                attachment_refs: Vec::new(),
                source_refs: Vec::new(),
            },
        )
        .unwrap();
        let research = preview_product_intent(
            &root,
            dir.path().join("research"),
            ProductIntentIntakeRequest {
                raw_text: "research 当前 Agent workflow 方案。".to_string(),
                selected_product_id: "software-dev".to_string(),
                workspace_id: "v116-research".to_string(),
                source_surface: "desktop-project-home".to_string(),
                locale: "zh-CN".to_string(),
                attachment_refs: Vec::new(),
                source_refs: Vec::new(),
            },
        )
        .unwrap();

        for receipt in [clarify, research] {
            assert_eq!(receipt.route_decision.write_boundary, "no-authority-write");
            assert!(!receipt.next_actions.contains(&"confirm".to_string()));
            assert!(!receipt
                .next_actions
                .contains(&"materialize-after-confirmation".to_string()));
        }
    }

    fn software_request() -> ProductIntentIntakeRequest {
        ProductIntentIntakeRequest {
            raw_text: "实现任务页状态时间线，并生成可执行任务合同。".to_string(),
            selected_product_id: "software-dev".to_string(),
            workspace_id: "v115-software".to_string(),
            source_surface: "desktop-project-home".to_string(),
            locale: "zh-CN".to_string(),
            attachment_refs: Vec::new(),
            source_refs: vec!["docs/project/roadmap.md".to_string()],
        }
    }

    fn workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("workspace root")
            .to_path_buf()
    }
}
