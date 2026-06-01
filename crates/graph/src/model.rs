use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GraphStatus {
    Missing,
    Indexing,
    Ready,
    Stale,
    Failed,
    Degraded,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphStatusSnapshot {
    pub version: String,
    pub project_root: String,
    pub status: GraphStatus,
    pub file_count: usize,
    pub symbol_count: usize,
    pub relation_count: usize,
    pub updated_at: Option<u64>,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphManifestSnapshot {
    pub version: String,
    pub project_root: String,
    pub languages: Vec<String>,
    pub top_level_dirs: Vec<String>,
    pub important_files: Vec<String>,
    pub source_files: usize,
    pub test_files: usize,
    pub doc_files: usize,
    pub config_files: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphFileRecord {
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
pub struct GraphSymbolRecord {
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
pub struct GraphRelationRecord {
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
pub struct GraphChunkRecord {
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
pub struct GraphSearchSnapshot {
    pub version: String,
    pub query: String,
    pub results: Vec<GraphSearchResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphSearchResult {
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
pub struct GraphContextPack {
    pub version: String,
    pub target_type: String,
    pub target_id: Option<String>,
    pub query: String,
    pub created_at: u64,
    pub graph_revision: Option<String>,
    pub recommended_files: Vec<GraphContextFile>,
    pub recommended_symbols: Vec<GraphContextSymbol>,
    pub recommended_tests: Vec<GraphContextFile>,
    pub impact_hints: Vec<GraphContextHint>,
    pub test_hints: Vec<GraphTestHint>,
    pub confidence: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphContextFile {
    pub path: String,
    pub reason: String,
    pub score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphContextSymbol {
    pub name: String,
    pub kind: String,
    pub path: String,
    pub line: usize,
    pub score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphContextHint {
    pub path: String,
    pub reason: String,
    pub confidence: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphTestHint {
    pub command_hint: String,
    pub reason: String,
    pub confidence: String,
}

#[derive(Debug, Clone)]
pub struct GraphIndex {
    pub files: Vec<GraphFileRecord>,
    pub symbols: Vec<GraphSymbolRecord>,
    pub relations: Vec<GraphRelationRecord>,
    pub chunks: Vec<GraphChunkRecord>,
}
