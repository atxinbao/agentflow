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
};
pub use preflight::preflight_graph_for_target;
pub use protection::check_graph_git_protection;
pub use search::search_project_graph;
pub use test_recommendation::recommend_graph_tests;
pub use watcher::ensure_graph_watcher;
