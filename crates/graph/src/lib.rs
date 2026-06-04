mod context_pack;
mod db;
mod impact;
mod manager;
mod model;
mod parser;
mod parser_registry;
mod preflight;
mod protection;
mod scanner;
mod search;
mod test_recommendation;
mod watcher;

pub use context_pack::{build_context_pack, load_context_pack};
pub use impact::analyze_graph_impact;
pub use manager::{
    index_project_graph, load_project_graph_manifest, load_project_graph_status,
    prepare_project_graph, GraphPrepareMode,
};
pub use model::{
    GraphContextPack, GraphFileRecord, GraphImpactSnapshot, GraphManifestSnapshot,
    GraphPreflightSnapshot, GraphProtectionSnapshot, GraphRelationRecord, GraphSearchResult,
    GraphSearchSnapshot, GraphStatus, GraphStatusSnapshot, GraphSymbolRecord, GraphWatcherSnapshot,
    PanelChunkRecord, PanelContextFile, PanelContextHint, PanelContextPack, PanelContextSymbol,
    PanelFileRecord, PanelImpactSnapshot, PanelIndex, PanelManifestSnapshot,
    PanelPreflightSnapshot, PanelProtectionSnapshot, PanelRelationRecord, PanelSearchResult,
    PanelSearchSnapshot, PanelStatus, PanelStatusSnapshot, PanelSymbolRecord, PanelTestHint,
    PanelWatcherDetail, PanelWatcherSnapshot,
};
pub use preflight::preflight_graph_for_target;
pub use protection::check_graph_git_protection;
pub use search::search_project_graph;
pub use test_recommendation::recommend_graph_tests;
pub use watcher::ensure_graph_watcher;

pub type PanelPrepareMode = GraphPrepareMode;

pub fn prepare_project_panel(
    project_root: impl AsRef<std::path::Path>,
    mode: PanelPrepareMode,
) -> anyhow::Result<PanelStatusSnapshot> {
    prepare_project_graph(project_root, mode)
}

pub fn index_project_panel(
    project_root: impl AsRef<std::path::Path>,
) -> anyhow::Result<PanelStatusSnapshot> {
    index_project_graph(project_root)
}

pub fn load_project_panel_status(
    project_root: impl AsRef<std::path::Path>,
) -> anyhow::Result<PanelStatusSnapshot> {
    load_project_graph_status(project_root)
}

pub fn load_project_panel_manifest(
    project_root: impl AsRef<std::path::Path>,
) -> anyhow::Result<PanelManifestSnapshot> {
    load_project_graph_manifest(project_root)
}

pub fn search_project_panel(
    project_root: impl AsRef<std::path::Path>,
    query: &str,
    limit: Option<usize>,
) -> anyhow::Result<PanelSearchSnapshot> {
    search_project_graph(project_root, query, limit)
}

pub fn build_panel_context_pack(
    project_root: impl AsRef<std::path::Path>,
    target_type: &str,
    target_id: Option<&str>,
    title: &str,
    objective: &str,
    acceptance_criteria: &[String],
) -> anyhow::Result<PanelContextPack> {
    build_context_pack(
        project_root,
        target_type,
        target_id,
        title,
        objective,
        acceptance_criteria,
    )
}

pub fn load_panel_context_pack(
    project_root: impl AsRef<std::path::Path>,
    target_id: &str,
) -> anyhow::Result<Option<PanelContextPack>> {
    load_context_pack(project_root, target_id)
}

pub fn panel_preflight(
    project_root: impl AsRef<std::path::Path>,
    target_type: &str,
    target_id: Option<&str>,
    title: &str,
    objective: &str,
    acceptance_criteria: &[String],
) -> anyhow::Result<PanelPreflightSnapshot> {
    preflight_graph_for_target(
        project_root,
        target_type,
        target_id,
        title,
        objective,
        acceptance_criteria,
    )
}

pub fn analyze_panel_impact(
    project_root: impl AsRef<std::path::Path>,
    changed_files: &[String],
    target_files: &[String],
    target_symbols: &[String],
    query: Option<&str>,
) -> anyhow::Result<PanelImpactSnapshot> {
    analyze_graph_impact(
        project_root,
        changed_files,
        target_files,
        target_symbols,
        query,
    )
}

pub fn check_panel_git_protection(
    project_root: impl AsRef<std::path::Path>,
) -> anyhow::Result<PanelProtectionSnapshot> {
    check_graph_git_protection(project_root)
}

pub fn ensure_panel_watcher(
    project_root: impl AsRef<std::path::Path>,
) -> anyhow::Result<PanelWatcherSnapshot> {
    ensure_graph_watcher(project_root)
}
