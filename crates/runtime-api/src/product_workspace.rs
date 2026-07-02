use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
};

pub const PRODUCT_WORKSPACE_CONTRACT_VERSION: &str = "agentflow-product-workspace.v1";
pub const PRODUCT_WORKSPACE_PROJECTION_VERSION: &str = "agentflow-product-workspace-projection.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProductWorkspaceCreationMode {
    Create,
    Recover,
    Inspect,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProductWorkspaceStatus {
    Created,
    Ready,
    Duplicate,
    Partial,
    InvalidRoot,
    MissingProduct,
    WriteFailed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductWorkspaceCreationRequest {
    pub project_name: String,
    pub workspace_root: String,
    pub selected_product_id: String,
    pub initial_goal: String,
    pub creation_mode: ProductWorkspaceCreationMode,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductWorkspacePathSet {
    pub docs_project_dir: String,
    pub goal_doc: String,
    pub roadmap_doc: String,
    pub context_doc: String,
    pub agentflow_root: String,
    pub workspace_manifest: String,
    pub spec_projects_dir: String,
    pub spec_issues_dir: String,
    pub events_dir: String,
    pub tasks_dir: String,
    pub evidence_dir: String,
    pub projection_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductWorkspaceProductBinding {
    pub product_id: String,
    pub pack_id: String,
    pub name: String,
    pub version: String,
    pub source_boundary: String,
    pub manifest_path: String,
    pub source_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductWorkspaceCreationReceipt {
    pub version: String,
    pub receipt_id: String,
    pub status: ProductWorkspaceStatus,
    pub project_name: String,
    pub workspace_id: String,
    pub workspace_root: String,
    pub selected_product_id: String,
    pub creation_mode: ProductWorkspaceCreationMode,
    pub writes_authority: bool,
    pub created_paths: Vec<String>,
    pub existing_paths: Vec<String>,
    pub paths: ProductWorkspacePathSet,
    pub active_product: Option<ProductWorkspaceProductBinding>,
    pub projection_refresh_hint: String,
    pub blockers: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductWorkspaceProjection {
    pub version: String,
    pub workspace_id: String,
    pub project_name: String,
    pub workspace_root: String,
    pub status: ProductWorkspaceStatus,
    pub active_product: Option<ProductWorkspaceProductBinding>,
    pub readiness: String,
    pub blockers: Vec<String>,
    pub docs_ready: bool,
    pub fact_source_ready: bool,
    pub projection_source: String,
    pub rebuild_receipt: String,
}

pub fn create_product_workspace(
    product_source_root: impl AsRef<Path>,
    request: ProductWorkspaceCreationRequest,
) -> ProductWorkspaceCreationReceipt {
    let product_source_root = product_source_root.as_ref();
    let workspace_root = PathBuf::from(request.workspace_root.trim());
    let workspace_id = normalize_workspace_id(&request.project_name);
    let paths = workspace_paths(&workspace_root, &workspace_id);
    let mut created_paths = Vec::new();
    let mut existing_paths = Vec::new();
    let mut blockers = Vec::new();

    if request.workspace_root.trim().is_empty() || workspace_root.is_file() {
        blockers.push("workspace root must be a writable directory path".to_string());
        return workspace_receipt(
            request,
            workspace_id,
            workspace_root,
            paths,
            ProductWorkspaceStatus::InvalidRoot,
            None,
            created_paths,
            existing_paths,
            blockers,
        );
    }

    let product =
        match load_workspace_product_binding(product_source_root, &request.selected_product_id) {
            Ok(binding) => Some(binding),
            Err(error) => {
                blockers.push(error.to_string());
                None
            }
        };
    if product.is_none() {
        return workspace_receipt(
            request,
            workspace_id,
            workspace_root,
            paths,
            ProductWorkspaceStatus::MissingProduct,
            None,
            created_paths,
            existing_paths,
            blockers,
        );
    }

    let manifest_path = workspace_root.join(".agentflow/workspace.json");
    if manifest_path.is_file() {
        existing_paths.push(normalize_path(&manifest_path));
        let status = if request.creation_mode == ProductWorkspaceCreationMode::Recover {
            ProductWorkspaceStatus::Ready
        } else {
            ProductWorkspaceStatus::Duplicate
        };
        return workspace_receipt(
            request,
            workspace_id,
            workspace_root,
            paths,
            status,
            product,
            created_paths,
            existing_paths,
            blockers,
        );
    }

    if workspace_root.join(".agentflow").exists() && !manifest_path.exists() {
        blockers.push("partial .agentflow workspace exists without workspace manifest".to_string());
        return workspace_receipt(
            request,
            workspace_id,
            workspace_root,
            paths,
            ProductWorkspaceStatus::Partial,
            product,
            created_paths,
            existing_paths,
            blockers,
        );
    }

    match materialize_workspace(
        &workspace_root,
        &workspace_id,
        &request,
        product.as_ref().unwrap(),
    ) {
        Ok(paths) => created_paths = paths,
        Err(error) => {
            blockers.push(error.to_string());
            return workspace_receipt(
                request,
                workspace_id,
                workspace_root,
                paths,
                ProductWorkspaceStatus::WriteFailed,
                product,
                created_paths,
                existing_paths,
                blockers,
            );
        }
    }

    let receipt = workspace_receipt(
        request,
        workspace_id,
        workspace_root.clone(),
        paths,
        ProductWorkspaceStatus::Created,
        product,
        created_paths,
        existing_paths,
        blockers,
    );
    let _ = write_product_workspace_projection(&workspace_root, &receipt);
    receipt
}

pub fn load_product_workspace_projection(
    workspace_root: impl AsRef<Path>,
) -> ProductWorkspaceProjection {
    let workspace_root = workspace_root.as_ref();
    let projection_path = workspace_root.join(".agentflow/projections/workspace-state.json");
    if let Ok(payload) = fs::read_to_string(&projection_path) {
        if let Ok(projection) = serde_json::from_str::<ProductWorkspaceProjection>(&payload) {
            return projection;
        }
    }

    let manifest_path = workspace_root.join(".agentflow/workspace.json");
    if let Ok(payload) = fs::read_to_string(&manifest_path) {
        if let Ok(value) = serde_json::from_str::<Value>(&payload) {
            let workspace_id = value
                .get("workspaceId")
                .and_then(Value::as_str)
                .unwrap_or("unknown-workspace")
                .to_string();
            let project_name = value
                .get("projectName")
                .and_then(Value::as_str)
                .unwrap_or("Unknown Project")
                .to_string();
            let active_product = value
                .get("activeProduct")
                .cloned()
                .and_then(|payload| serde_json::from_value(payload).ok());
            return ProductWorkspaceProjection {
                version: PRODUCT_WORKSPACE_PROJECTION_VERSION.to_string(),
                workspace_id,
                project_name,
                workspace_root: normalize_path(workspace_root),
                status: ProductWorkspaceStatus::Ready,
                active_product,
                readiness: "ready".to_string(),
                blockers: Vec::new(),
                docs_ready: workspace_root.join("docs/project/goal.md").is_file(),
                fact_source_ready: workspace_root.join(".agentflow/spec/projects").is_dir()
                    && workspace_root.join(".agentflow/spec/issues").is_dir(),
                projection_source: normalize_path(&manifest_path),
                rebuild_receipt: "projection-rebuilt-from-workspace-manifest".to_string(),
            };
        }
    }

    ProductWorkspaceProjection {
        version: PRODUCT_WORKSPACE_PROJECTION_VERSION.to_string(),
        workspace_id: "missing-workspace".to_string(),
        project_name: "Missing Workspace".to_string(),
        workspace_root: normalize_path(workspace_root),
        status: ProductWorkspaceStatus::Partial,
        active_product: None,
        readiness: "blocked".to_string(),
        blockers: vec!["workspace manifest is missing".to_string()],
        docs_ready: false,
        fact_source_ready: false,
        projection_source: normalize_path(&projection_path),
        rebuild_receipt: "projection-rebuild-blocked".to_string(),
    }
}

fn materialize_workspace(
    workspace_root: &Path,
    workspace_id: &str,
    request: &ProductWorkspaceCreationRequest,
    product: &ProductWorkspaceProductBinding,
) -> Result<Vec<String>> {
    let paths = workspace_paths(workspace_root, workspace_id);
    let dirs = [
        workspace_root.join("docs/project"),
        workspace_root.join("docs/requirements"),
        workspace_root.join(".agentflow/spec/projects"),
        workspace_root.join(".agentflow/spec/issues"),
        workspace_root.join(".agentflow/events"),
        workspace_root.join(".agentflow/tasks"),
        workspace_root
            .join(".agentflow/tasks")
            .join(workspace_id)
            .join("evidence"),
        workspace_root.join(".agentflow/projections"),
    ];
    let mut created = Vec::new();
    for dir in dirs {
        fs::create_dir_all(&dir)?;
        created.push(normalize_path(&dir));
    }

    write_file(
        workspace_root.join("docs/project/README.md"),
        format!(
            "# {}\n\nThis project workspace was created by AgentFlow from Product `{}`.\n",
            request.project_name, product.product_id
        ),
        &mut created,
    )?;
    write_file(
        workspace_root.join("docs/project/goal.md"),
        format!("# Goal\n\n{}\n", request.initial_goal),
        &mut created,
    )?;
    write_file(
        workspace_root.join("docs/project/roadmap.md"),
        "# Roadmap\n\n- Confirm project goal.\n- Materialize Spec Loop work.\n- Run Product command surface.\n".to_string(),
        &mut created,
    )?;
    write_file(
        workspace_root.join("docs/project/context.md"),
        format!(
            "# Context\n\nActive Product: `{}`\nProduct Source: `{}`\n",
            product.product_id, product.source_boundary
        ),
        &mut created,
    )?;
    write_json_file(
        workspace_root.join(".agentflow/workspace.json"),
        &serde_json::json!({
            "version": PRODUCT_WORKSPACE_CONTRACT_VERSION,
            "workspaceId": workspace_id,
            "projectName": request.project_name,
            "initialGoal": request.initial_goal,
            "activeProduct": product,
            "paths": paths,
            "authorityBoundary": {
                "docs": "human-readable project record",
                ".agentflow": "runtime fact source",
                "products": "read-only Product source",
                "apps": "client surfaces read projection"
            }
        }),
        &mut created,
    )?;
    write_json_file(
        workspace_root
            .join(".agentflow/spec/projects")
            .join(format!("{workspace_id}.json")),
        &serde_json::json!({
            "version": "agentflow-spec-project.v1",
            "projectId": workspace_id,
            "title": request.project_name,
            "status": "ready",
            "sourceRequirementPath": "docs/project/goal.md",
            "activeProductId": product.product_id,
            "activeProductSourceRefs": product.source_refs,
        }),
        &mut created,
    )?;
    Ok(created)
}

fn write_product_workspace_projection(
    workspace_root: &Path,
    receipt: &ProductWorkspaceCreationReceipt,
) -> Result<()> {
    let projection = ProductWorkspaceProjection {
        version: PRODUCT_WORKSPACE_PROJECTION_VERSION.to_string(),
        workspace_id: receipt.workspace_id.clone(),
        project_name: receipt.project_name.clone(),
        workspace_root: receipt.workspace_root.clone(),
        status: ProductWorkspaceStatus::Ready,
        active_product: receipt.active_product.clone(),
        readiness: "ready".to_string(),
        blockers: Vec::new(),
        docs_ready: true,
        fact_source_ready: true,
        projection_source: ".agentflow/workspace.json".to_string(),
        rebuild_receipt: format!("rebuild-{}", receipt.receipt_id),
    };
    let path = workspace_root.join(".agentflow/projections/workspace-state.json");
    let payload = serde_json::to_string_pretty(&projection)?;
    fs::write(path, format!("{payload}\n"))?;
    Ok(())
}

fn load_workspace_product_binding(
    product_source_root: &Path,
    product_id: &str,
) -> Result<ProductWorkspaceProductBinding> {
    let registry = agentflow_pack::load_product_registry(product_source_root)?;
    let entry = registry.product(product_id).cloned().ok_or_else(|| {
        anyhow::anyhow!("product `{product_id}` is not registered under products/**")
    })?;
    if !entry.valid {
        anyhow::bail!("product `{product_id}` is invalid: {:?}", entry.diagnostics);
    }
    let definition = agentflow_pack::load_product_definition_from_entry(&entry)?;
    if !definition.valid {
        anyhow::bail!(
            "product `{product_id}` definition is invalid: {:?}",
            definition.diagnostics
        );
    }
    Ok(ProductWorkspaceProductBinding {
        product_id: entry.product_id,
        pack_id: entry.pack_id,
        name: entry.name,
        version: entry.version,
        source_boundary: entry.source_boundary,
        manifest_path: entry.manifest_path.clone(),
        source_refs: vec![
            entry.manifest_path,
            definition.manifest.entrypoints.domain,
            definition.manifest.entrypoints.surface,
            definition.manifest.entrypoints.connectors,
            definition.manifest.entrypoints.flow,
            definition.manifest.entrypoints.projection,
        ],
    })
}

fn workspace_receipt(
    request: ProductWorkspaceCreationRequest,
    workspace_id: String,
    workspace_root: PathBuf,
    paths: ProductWorkspacePathSet,
    status: ProductWorkspaceStatus,
    active_product: Option<ProductWorkspaceProductBinding>,
    created_paths: Vec<String>,
    existing_paths: Vec<String>,
    blockers: Vec<String>,
) -> ProductWorkspaceCreationReceipt {
    let receipt_seed = format!(
        "{}:{}:{}:{:?}",
        workspace_id, request.selected_product_id, request.initial_goal, status
    );
    ProductWorkspaceCreationReceipt {
        version: PRODUCT_WORKSPACE_CONTRACT_VERSION.to_string(),
        receipt_id: format!("workspace-{:016x}", stable_hash(receipt_seed.as_bytes())),
        status,
        project_name: request.project_name,
        workspace_id,
        workspace_root: normalize_path(workspace_root),
        selected_product_id: request.selected_product_id,
        creation_mode: request.creation_mode,
        writes_authority: !created_paths.is_empty(),
        created_paths,
        existing_paths,
        paths,
        active_product,
        projection_refresh_hint: "reload .agentflow/projections/workspace-state.json".to_string(),
        blockers,
    }
}

fn workspace_paths(workspace_root: &Path, workspace_id: &str) -> ProductWorkspacePathSet {
    ProductWorkspacePathSet {
        docs_project_dir: normalize_path(workspace_root.join("docs/project")),
        goal_doc: normalize_path(workspace_root.join("docs/project/goal.md")),
        roadmap_doc: normalize_path(workspace_root.join("docs/project/roadmap.md")),
        context_doc: normalize_path(workspace_root.join("docs/project/context.md")),
        agentflow_root: normalize_path(workspace_root.join(".agentflow")),
        workspace_manifest: normalize_path(workspace_root.join(".agentflow/workspace.json")),
        spec_projects_dir: normalize_path(workspace_root.join(".agentflow/spec/projects")),
        spec_issues_dir: normalize_path(workspace_root.join(".agentflow/spec/issues")),
        events_dir: normalize_path(workspace_root.join(".agentflow/events")),
        tasks_dir: normalize_path(workspace_root.join(".agentflow/tasks")),
        evidence_dir: normalize_path(
            workspace_root
                .join(".agentflow/tasks")
                .join(workspace_id)
                .join("evidence"),
        ),
        projection_path: normalize_path(
            workspace_root.join(".agentflow/projections/workspace-state.json"),
        ),
    }
}

fn write_file(path: PathBuf, payload: String, created: &mut Vec<String>) -> Result<()> {
    fs::write(&path, payload)?;
    created.push(normalize_path(path));
    Ok(())
}

fn write_json_file(path: PathBuf, payload: &Value, created: &mut Vec<String>) -> Result<()> {
    let rendered = serde_json::to_string_pretty(payload)?;
    fs::write(&path, format!("{rendered}\n"))?;
    created.push(normalize_path(path));
    Ok(())
}

fn normalize_workspace_id(value: &str) -> String {
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
    let output = output.trim_matches('-').to_string();
    if output.is_empty() {
        "agentflow-project".to_string()
    } else {
        output
    }
}

fn normalize_path(path: impl AsRef<Path>) -> String {
    path.as_ref().to_string_lossy().replace('\\', "/")
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

    #[test]
    fn product_workspace_bootstraps_software_dev_workspace() {
        let source = workspace_root();
        let dir = tempfile::tempdir().unwrap();
        let receipt = create_product_workspace(
            &source,
            ProductWorkspaceCreationRequest {
                project_name: "My AgentFlow Project".to_string(),
                workspace_root: dir.path().join("workspace").to_string_lossy().to_string(),
                selected_product_id: "software-dev".to_string(),
                initial_goal: "Build a controlled Agent workflow.".to_string(),
                creation_mode: ProductWorkspaceCreationMode::Create,
            },
        );

        assert_eq!(receipt.status, ProductWorkspaceStatus::Created);
        assert_eq!(
            receipt
                .active_product
                .as_ref()
                .map(|product| product.product_id.as_str()),
            Some("software-dev")
        );
        assert!(PathBuf::from(&receipt.paths.goal_doc).is_file());
        assert!(PathBuf::from(&receipt.paths.spec_projects_dir).is_dir());

        let projection = load_product_workspace_projection(&receipt.workspace_root);
        assert_eq!(projection.status, ProductWorkspaceStatus::Ready);
        assert!(projection.docs_ready);
        assert!(projection.fact_source_ready);
    }

    #[test]
    fn product_workspace_rejects_missing_product_and_partial_workspace() {
        let source = workspace_root();
        let dir = tempfile::tempdir().unwrap();
        let missing = create_product_workspace(
            &source,
            ProductWorkspaceCreationRequest {
                project_name: "Missing Product".to_string(),
                workspace_root: dir.path().join("missing").to_string_lossy().to_string(),
                selected_product_id: "not-a-product".to_string(),
                initial_goal: "Test missing product.".to_string(),
                creation_mode: ProductWorkspaceCreationMode::Create,
            },
        );
        assert_eq!(missing.status, ProductWorkspaceStatus::MissingProduct);

        let partial_root = dir.path().join("partial");
        fs::create_dir_all(partial_root.join(".agentflow/spec")).unwrap();
        let partial = create_product_workspace(
            &source,
            ProductWorkspaceCreationRequest {
                project_name: "Partial Product".to_string(),
                workspace_root: partial_root.to_string_lossy().to_string(),
                selected_product_id: "software-dev".to_string(),
                initial_goal: "Test partial workspace.".to_string(),
                creation_mode: ProductWorkspaceCreationMode::Create,
            },
        );
        assert_eq!(partial.status, ProductWorkspaceStatus::Partial);
    }

    fn workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("workspace root")
            .to_path_buf()
    }
}
