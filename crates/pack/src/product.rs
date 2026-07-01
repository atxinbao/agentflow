use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
};

pub const PRODUCT_REGISTRY_VERSION: &str = "agentflow-product-registry.v1";
pub const PRODUCT_DEFINITION_VERSION: &str = "agentflow-product-definition.v1";
pub const PRODUCT_TO_PACK_CONTRACT_VERSION: &str = "agentflow-product-to-pack-contract.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ProductAuthority {
    pub writes_core_authority: bool,
    pub writes_runtime_authority: bool,
    pub source_of_product_definitions: bool,
    pub runtime_materializes_facts: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ProductEntrypoints {
    pub domain: String,
    pub surface: String,
    pub connectors: String,
    pub flow: String,
    pub projection: String,
    pub golden_fixture: String,
    pub negative_fixtures: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ProductManifest {
    pub product_id: String,
    pub name: String,
    pub version: String,
    pub status: String,
    pub source_boundary: String,
    pub core_boundary: String,
    pub fixture_mirror: String,
    pub pack_id: String,
    pub authority: ProductAuthority,
    pub entrypoints: ProductEntrypoints,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductRegistryDiagnostic {
    pub code: String,
    pub message: String,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductRegistryEntry {
    pub product_id: String,
    pub name: String,
    pub version: String,
    pub status: String,
    pub pack_id: String,
    pub product_root: String,
    pub manifest_path: String,
    pub source_boundary: String,
    pub fixture_mirror: String,
    pub writes_authority: bool,
    pub valid: bool,
    #[serde(default)]
    pub diagnostics: Vec<ProductRegistryDiagnostic>,
    pub entrypoints: ProductEntrypoints,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductRegistry {
    pub version: String,
    pub root: String,
    pub writes_authority: bool,
    pub entries: Vec<ProductRegistryEntry>,
}

impl ProductRegistry {
    pub fn product(&self, product_id: &str) -> Option<&ProductRegistryEntry> {
        self.entries
            .iter()
            .find(|entry| entry.product_id == product_id || entry.pack_id == product_id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductSurfacePage {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub reads: Vec<String>,
    pub writes_authority: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductSurfaceCommand {
    pub id: String,
    pub label: String,
    pub runtime_command: String,
    pub requires_projection_freshness: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductSurfaceDefinition {
    pub version: String,
    pub product_id: String,
    pub source_boundary: String,
    pub core_authority: bool,
    #[serde(default)]
    pub pages: Vec<ProductSurfacePage>,
    #[serde(default)]
    pub commands: Vec<ProductSurfaceCommand>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductConnector {
    pub id: String,
    #[serde(rename = "type")]
    pub connector_type: String,
    pub authority: bool,
    #[serde(default)]
    pub produces: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductConnectorDefinition {
    pub version: String,
    pub product_id: String,
    pub source_boundary: String,
    pub core_authority: bool,
    #[serde(default)]
    pub connectors: Vec<ProductConnector>,
    #[serde(default)]
    pub invalid_authority_sources: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductDefinition {
    pub version: String,
    pub product_id: String,
    pub pack_id: String,
    pub manifest: ProductManifest,
    pub domain: Value,
    pub surface: ProductSurfaceDefinition,
    pub connectors: ProductConnectorDefinition,
    pub flow: Value,
    pub projection: Value,
    pub golden_fixture: Value,
    pub negative_fixtures: Value,
    pub writes_authority: bool,
    pub valid: bool,
    #[serde(default)]
    pub diagnostics: Vec<ProductRegistryDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductCommandRoute {
    pub product_id: String,
    pub pack_id: String,
    pub command: String,
    pub label: String,
    pub runtime_command: String,
    pub action_contract_ref: String,
    pub target_object_type: String,
    pub source_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductToPackContract {
    pub version: String,
    pub product_id: String,
    pub pack_id: String,
    pub source_boundary: String,
    pub fixture_mirror: String,
    pub fixture_mirror_is_authority: bool,
    pub writes_authority: bool,
    #[serde(default)]
    pub commands: Vec<ProductCommandRoute>,
    #[serde(default)]
    pub diagnostics: Vec<ProductRegistryDiagnostic>,
    pub valid: bool,
}

pub fn product_root(project_root: impl AsRef<Path>) -> PathBuf {
    project_root.as_ref().join("products")
}

pub fn load_product_registry(project_root: impl AsRef<Path>) -> Result<ProductRegistry> {
    let root = product_root(project_root);
    if !root.exists() {
        return Ok(ProductRegistry {
            version: PRODUCT_REGISTRY_VERSION.to_string(),
            root: normalize_path(&root),
            writes_authority: false,
            entries: Vec::new(),
        });
    }

    let mut entries = Vec::new();
    for product_dir in fs::read_dir(&root).with_context(|| format!("read {}", root.display()))? {
        let product_dir = product_dir?;
        if !product_dir.file_type()?.is_dir() {
            continue;
        }
        let product_root = product_dir.path();
        let manifest_path = product_root.join("product.toml");
        if !manifest_path.is_file() {
            continue;
        }
        let manifest = load_product_manifest(&manifest_path)?;
        entries.push(product_registry_entry(
            product_root,
            manifest_path,
            manifest,
        ));
    }
    entries.sort_by(|left, right| left.product_id.cmp(&right.product_id));

    Ok(ProductRegistry {
        version: PRODUCT_REGISTRY_VERSION.to_string(),
        root: normalize_path(&root),
        writes_authority: false,
        entries,
    })
}

pub fn load_product_manifest(path: impl AsRef<Path>) -> Result<ProductManifest> {
    let path = path.as_ref();
    let payload = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    toml::from_str::<ProductManifest>(&payload)
        .with_context(|| format!("parse product manifest {}", path.display()))
}

pub fn load_product_definition(
    project_root: impl AsRef<Path>,
    product_id: &str,
) -> Result<ProductDefinition> {
    let registry = load_product_registry(project_root)?;
    let entry = registry.product(product_id).cloned().ok_or_else(|| {
        anyhow::anyhow!("product `{product_id}` is not registered under products/**")
    })?;
    load_product_definition_from_entry(&entry)
}

pub fn load_product_definition_from_entry(
    entry: &ProductRegistryEntry,
) -> Result<ProductDefinition> {
    let product_root = PathBuf::from(&entry.product_root);
    let manifest = load_product_manifest(&entry.manifest_path)?;
    let mut diagnostics = entry.diagnostics.clone();
    let domain = load_json_entrypoint(
        &product_root,
        &manifest.entrypoints.domain,
        &mut diagnostics,
    );
    let surface = load_json_entrypoint(
        &product_root,
        &manifest.entrypoints.surface,
        &mut diagnostics,
    );
    let connectors = load_json_entrypoint(
        &product_root,
        &manifest.entrypoints.connectors,
        &mut diagnostics,
    );
    let flow = load_json_entrypoint(&product_root, &manifest.entrypoints.flow, &mut diagnostics);
    let projection = load_json_entrypoint(
        &product_root,
        &manifest.entrypoints.projection,
        &mut diagnostics,
    );
    let golden_fixture = load_json_entrypoint(
        &product_root,
        &manifest.entrypoints.golden_fixture,
        &mut diagnostics,
    );
    let negative_fixtures = load_json_entrypoint(
        &product_root,
        &manifest.entrypoints.negative_fixtures,
        &mut diagnostics,
    );

    let surface =
        serde_json::from_value::<ProductSurfaceDefinition>(surface?).with_context(|| {
            format!(
                "parse {}",
                definition_path(&product_root, &manifest.entrypoints.surface).display()
            )
        })?;
    let connectors = serde_json::from_value::<ProductConnectorDefinition>(connectors?)
        .with_context(|| {
            format!(
                "parse {}",
                definition_path(&product_root, &manifest.entrypoints.connectors).display()
            )
        })?;

    let writes_authority = manifest.authority.writes_core_authority
        || manifest.authority.writes_runtime_authority
        || manifest.authority.runtime_materializes_facts
        || surface.core_authority
        || connectors.core_authority
        || connectors
            .connectors
            .iter()
            .any(|connector| connector.authority);

    if writes_authority {
        diagnostics.push(diagnostic(
            "product-authority-write",
            "product definitions must not write Core or Runtime authority",
            &entry.manifest_path,
        ));
    }

    Ok(ProductDefinition {
        version: PRODUCT_DEFINITION_VERSION.to_string(),
        product_id: manifest.product_id.clone(),
        pack_id: manifest.pack_id.clone(),
        manifest,
        domain: domain?,
        surface,
        connectors,
        flow: flow?,
        projection: projection?,
        golden_fixture: golden_fixture?,
        negative_fixtures: negative_fixtures?,
        writes_authority,
        valid: diagnostics.is_empty(),
        diagnostics,
    })
}

pub fn product_to_pack_contract(
    project_root: impl AsRef<Path>,
    product_id: &str,
) -> Result<ProductToPackContract> {
    let definition = load_product_definition(project_root, product_id)?;
    let mut diagnostics = definition.diagnostics.clone();
    let commands = definition
        .surface
        .commands
        .iter()
        .filter_map(
            |command| match product_command_route(&definition, command) {
                Ok(route) => Some(route),
                Err(error) => {
                    diagnostics.push(diagnostic(
                        "product-command-unmapped",
                        error.to_string(),
                        &definition.manifest.entrypoints.surface,
                    ));
                    None
                }
            },
        )
        .collect::<Vec<_>>();

    if commands.is_empty() {
        diagnostics.push(diagnostic(
            "product-command-empty",
            "product surface must expose at least one runtime command route",
            &definition.manifest.entrypoints.surface,
        ));
    }

    Ok(ProductToPackContract {
        version: PRODUCT_TO_PACK_CONTRACT_VERSION.to_string(),
        product_id: definition.product_id.clone(),
        pack_id: definition.pack_id.clone(),
        source_boundary: definition.manifest.source_boundary.clone(),
        fixture_mirror: definition.manifest.fixture_mirror.clone(),
        fixture_mirror_is_authority: false,
        writes_authority: definition.writes_authority,
        valid: diagnostics.is_empty(),
        commands,
        diagnostics,
    })
}

pub fn product_command_route(
    definition: &ProductDefinition,
    command: &ProductSurfaceCommand,
) -> Result<ProductCommandRoute> {
    let (action_contract_ref, target_object_type) = product_command_mapping(command)?;
    let product_root = PathBuf::from(&definition.manifest.source_boundary);
    Ok(ProductCommandRoute {
        product_id: definition.product_id.clone(),
        pack_id: definition.pack_id.clone(),
        command: command.id.clone(),
        label: command.label.clone(),
        runtime_command: command.runtime_command.clone(),
        action_contract_ref: action_contract_ref.to_string(),
        target_object_type: target_object_type.to_string(),
        source_refs: vec![
            normalize_path(&product_root.join("product.toml")),
            normalize_path(&product_root.join(&definition.manifest.entrypoints.surface)),
            normalize_path(&product_root.join(&definition.manifest.entrypoints.connectors)),
            normalize_path(&product_root.join(&definition.manifest.entrypoints.flow)),
            normalize_path(&product_root.join(&definition.manifest.entrypoints.projection)),
        ],
    })
}

pub fn product_command_mapping(
    command: &ProductSurfaceCommand,
) -> Result<(&'static str, &'static str)> {
    match (command.id.as_str(), command.runtime_command.as_str()) {
        ("work.issue.start", "runtime.command.start-work") => {
            Ok(("action-contract:issue.start", "Issue"))
        }
        ("work.issue.review", "runtime.command.prepare-review") => {
            Ok(("action-contract:delivery.prepare", "Run"))
        }
        _ => bail!(
            "product command `{}` has no product-to-pack action contract mapping",
            command.id
        ),
    }
}

fn product_registry_entry(
    product_root: PathBuf,
    manifest_path: PathBuf,
    manifest: ProductManifest,
) -> ProductRegistryEntry {
    let mut diagnostics = validate_product_manifest(&manifest_path, &product_root, &manifest);
    for entrypoint in [
        &manifest.entrypoints.domain,
        &manifest.entrypoints.surface,
        &manifest.entrypoints.connectors,
        &manifest.entrypoints.flow,
        &manifest.entrypoints.projection,
        &manifest.entrypoints.golden_fixture,
        &manifest.entrypoints.negative_fixtures,
    ] {
        let path = definition_path(&product_root, entrypoint);
        if !path.is_file() {
            diagnostics.push(diagnostic(
                "product-entrypoint-missing",
                format!("entrypoint `{entrypoint}` does not exist"),
                &normalize_path(&path),
            ));
        }
    }

    ProductRegistryEntry {
        product_id: manifest.product_id,
        name: manifest.name,
        version: manifest.version,
        status: manifest.status,
        pack_id: manifest.pack_id,
        product_root: normalize_path(&product_root),
        manifest_path: normalize_path(&manifest_path),
        source_boundary: manifest.source_boundary,
        fixture_mirror: manifest.fixture_mirror,
        writes_authority: manifest.authority.writes_core_authority
            || manifest.authority.writes_runtime_authority
            || manifest.authority.runtime_materializes_facts,
        valid: diagnostics.is_empty(),
        diagnostics,
        entrypoints: manifest.entrypoints,
    }
}

fn validate_product_manifest(
    manifest_path: &Path,
    product_root: &Path,
    manifest: &ProductManifest,
) -> Vec<ProductRegistryDiagnostic> {
    let mut diagnostics = Vec::new();
    require_non_empty(
        &mut diagnostics,
        "productId",
        &manifest.product_id,
        manifest_path,
    );
    require_non_empty(&mut diagnostics, "name", &manifest.name, manifest_path);
    require_non_empty(
        &mut diagnostics,
        "version",
        &manifest.version,
        manifest_path,
    );
    require_non_empty(&mut diagnostics, "packId", &manifest.pack_id, manifest_path);
    if manifest.source_boundary.trim().is_empty() {
        diagnostics.push(diagnostic(
            "product-source-boundary-missing",
            "sourceBoundary must be declared",
            &normalize_path(manifest_path),
        ));
    }
    if manifest.authority.writes_core_authority
        || manifest.authority.writes_runtime_authority
        || manifest.authority.runtime_materializes_facts
    {
        diagnostics.push(diagnostic(
            "product-authority-write",
            "product manifest must not write Core or Runtime authority",
            &normalize_path(manifest_path),
        ));
    }
    if !manifest.authority.source_of_product_definitions {
        diagnostics.push(diagnostic(
            "product-source-not-authoritative",
            "product manifest must mark products/** as source of product definitions",
            &normalize_path(manifest_path),
        ));
    }
    if manifest.product_id
        != product_root
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("")
    {
        diagnostics.push(diagnostic(
            "product-id-path-mismatch",
            "productId must match products/<product-id>",
            &normalize_path(manifest_path),
        ));
    }
    diagnostics
}

fn require_non_empty(
    diagnostics: &mut Vec<ProductRegistryDiagnostic>,
    field: &str,
    value: &str,
    path: &Path,
) {
    if value.trim().is_empty() {
        diagnostics.push(diagnostic(
            "product-field-missing",
            format!("{field} must not be empty"),
            &normalize_path(path),
        ));
    }
}

fn load_json_entrypoint(
    root: &Path,
    relative_path: &str,
    diagnostics: &mut Vec<ProductRegistryDiagnostic>,
) -> Result<Value> {
    let path = definition_path(root, relative_path);
    let payload = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str::<Value>(&payload).with_context(|| {
        diagnostics.push(diagnostic(
            "product-entrypoint-invalid-json",
            format!("entrypoint `{relative_path}` is not valid JSON"),
            &normalize_path(&path),
        ));
        format!("parse {}", path.display())
    })
}

fn definition_path(root: &Path, relative_path: &str) -> PathBuf {
    root.join(relative_path)
}

fn diagnostic(
    code: impl Into<String>,
    message: impl Into<String>,
    path: impl Into<String>,
) -> ProductRegistryDiagnostic {
    ProductRegistryDiagnostic {
        code: code.into(),
        message: message.into(),
        path: path.into(),
    }
}

fn normalize_path(path: impl AsRef<Path>) -> String {
    path.as_ref().to_string_lossy().replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::{load_product_definition, load_product_registry, product_to_pack_contract};
    use std::path::Path;

    #[test]
    fn registry_loads_software_dev_product_without_writing_authority() {
        let root = workspace_root();

        let registry = load_product_registry(&root).unwrap();

        assert!(!registry.writes_authority);
        let product = registry.product("software-dev").unwrap();
        assert!(product.valid);
        assert!(!product.writes_authority);
        assert_eq!(product.pack_id, "software-dev");
        assert_eq!(
            product.entrypoints.surface,
            "surface/definition.json".to_string()
        );
    }

    #[test]
    fn product_definition_loads_all_stage_entrypoints() {
        let root = workspace_root();

        let definition = load_product_definition(&root, "software-dev").unwrap();

        assert!(definition.valid);
        assert!(!definition.writes_authority);
        assert_eq!(definition.surface.commands.len(), 2);
        assert_eq!(definition.connectors.connectors.len(), 4);
        assert!(definition.golden_fixture["primaryProofs"]
            .as_array()
            .unwrap()
            .iter()
            .any(|value| value == "product-contract"));
    }

    #[test]
    fn product_to_pack_contract_maps_surface_commands() {
        let root = workspace_root();

        let contract = product_to_pack_contract(&root, "software-dev").unwrap();

        assert!(contract.valid);
        assert!(!contract.writes_authority);
        assert!(!contract.fixture_mirror_is_authority);
        assert!(contract
            .commands
            .iter()
            .any(|command| command.command == "work.issue.start"
                && command.action_contract_ref == "action-contract:issue.start"));
        assert!(contract
            .commands
            .iter()
            .any(|command| command.command == "work.issue.review"
                && command.action_contract_ref == "action-contract:delivery.prepare"));
    }

    #[test]
    fn missing_product_returns_empty_registry_and_no_fallback() {
        let dir = tempfile::tempdir().unwrap();

        let registry = load_product_registry(dir.path()).unwrap();

        assert!(registry.entries.is_empty());
        assert!(registry.product("software-dev").is_none());
    }

    fn workspace_root() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .unwrap()
            .to_path_buf()
    }
}
