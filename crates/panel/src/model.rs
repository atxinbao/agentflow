use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PanelStatus {
    Missing,
    Indexing,
    Ready,
    Stale,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PanelStatusSnapshot {
    pub version: String,
    pub project_root: String,
    pub status: PanelStatus,
    pub file_count: usize,
    pub symbol_count: usize,
    pub relation_count: usize,
    pub updated_at: Option<u64>,
    pub last_error: Option<String>,
    pub watcher_status: Option<String>,
    pub watcher_backend: Option<String>,
    pub watcher_detail: Option<PanelWatcherDetail>,
    pub preflight_status: Option<String>,
    pub protection_status: Option<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PanelWatcherDetail {
    pub platform: String,
    pub recursive: bool,
    pub ignored_path_count: usize,
    pub last_event_at: Option<u64>,
    pub last_event_kind: Option<String>,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PanelManifestSnapshot {
    pub version: String,
    pub project_root: String,
    pub languages: Vec<String>,
    pub top_level_dirs: Vec<String>,
    pub important_files: Vec<String>,
    pub source_files: usize,
    pub test_files: usize,
    pub doc_files: usize,
    pub config_files: usize,
    pub platforms: Vec<String>,
    pub entry_points: Vec<String>,
    pub mobile_components: Vec<String>,
    pub mobile_configs: Vec<String>,
    pub mobile_tests: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PanelFileRecord {
    pub id: String,
    pub path: String,
    pub name: String,
    pub extension: Option<String>,
    pub language: String,
    pub kind: String,
    pub size_bytes: u64,
    pub line_count: usize,
    pub modified_at: Option<u64>,
    pub content_hash: String,
    pub is_source: bool,
    pub is_test: bool,
    pub is_doc: bool,
    pub is_config: bool,
    pub is_generated: bool,
    pub is_ignored: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PanelSymbolRecord {
    pub id: String,
    pub file_id: String,
    pub language: String,
    pub name: String,
    pub kind: String,
    pub signature: Option<String>,
    pub start_line: usize,
    pub end_line: usize,
    pub parent_symbol_id: Option<String>,
    pub visibility: Option<String>,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PanelRelationRecord {
    pub id: String,
    pub from_type: String,
    pub from_id: String,
    pub to_type: String,
    pub to_id: String,
    pub relation_kind: String,
    pub confidence: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PanelChunkRecord {
    pub id: String,
    pub file_id: String,
    pub symbol_id: Option<String>,
    pub path: String,
    pub start_line: usize,
    pub end_line: usize,
    pub text: String,
    pub token_estimate: usize,
    pub content_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PanelSearchSnapshot {
    pub version: String,
    pub query: String,
    pub results: Vec<PanelSearchResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PanelSearchResult {
    pub kind: String,
    pub path: String,
    pub title: String,
    pub language: Option<String>,
    pub symbol_kind: Option<String>,
    pub line: Option<usize>,
    pub snippet: Option<String>,
    pub score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PanelContextPack {
    pub version: String,
    pub target_type: String,
    pub target_id: Option<String>,
    pub query: String,
    pub created_at: u64,
    pub panel_revision: Option<String>,
    pub recommended_files: Vec<PanelContextFile>,
    pub recommended_symbols: Vec<PanelContextSymbol>,
    pub recommended_tests: Vec<PanelContextFile>,
    pub impact_hints: Vec<PanelContextHint>,
    pub test_hints: Vec<PanelTestHint>,
    pub confidence: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PanelContextFile {
    pub path: String,
    pub reason: String,
    pub score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PanelContextSymbol {
    pub name: String,
    pub kind: String,
    pub path: String,
    pub line: usize,
    pub score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PanelContextHint {
    pub path: String,
    pub reason: String,
    pub confidence: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PanelTestHint {
    pub command_hint: String,
    pub reason: String,
    pub confidence: String,
    pub scope: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PanelPreflightSnapshot {
    pub version: String,
    pub project_root: String,
    pub target_type: String,
    pub target_id: Option<String>,
    pub status: String,
    pub ready: bool,
    pub reason: String,
    pub panel_status: PanelStatus,
    pub context_pack_path: Option<String>,
    pub recommended_files: Vec<PanelContextFile>,
    pub recommended_symbols: Vec<PanelContextSymbol>,
    pub recommended_tests: Vec<PanelContextFile>,
    pub impact_hints: Vec<PanelContextHint>,
    pub test_hints: Vec<PanelTestHint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PanelImpactSnapshot {
    pub version: String,
    pub project_root: String,
    pub possibly_affected_files: Vec<PanelContextHint>,
    pub possibly_affected_symbols: Vec<PanelContextSymbol>,
    pub possibly_affected_tests: Vec<PanelContextFile>,
    pub reasons: Vec<String>,
    pub confidence: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PanelProtectionSnapshot {
    pub version: String,
    pub project_root: String,
    pub status: String,
    pub panel_output_root: String,
    pub git_exclude_path: Option<String>,
    pub protected_by_info_exclude: bool,
    pub writes_only_panel_output: bool,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PanelWatcherSnapshot {
    pub version: String,
    pub project_root: String,
    pub status: String,
    pub backend: String,
    pub recursive: bool,
    pub debounce_ms: u64,
    pub ignored_path_count: usize,
    pub last_event_at: Option<u64>,
    pub last_event_kind: Option<String>,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PanelIndex {
    pub files: Vec<PanelFileRecord>,
    pub symbols: Vec<PanelSymbolRecord>,
    pub relations: Vec<PanelRelationRecord>,
    pub chunks: Vec<PanelChunkRecord>,
}
