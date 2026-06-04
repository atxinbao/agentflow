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

pub use context_pack::{build_panel_context_pack, load_panel_context_pack};
pub use impact::analyze_panel_impact;
pub use manager::{
    index_project_panel, load_project_panel_manifest, load_project_panel_status,
    prepare_project_panel, PanelPrepareMode,
};
pub use model::{
    PanelChunkRecord, PanelContextFile, PanelContextHint, PanelContextPack, PanelContextSymbol,
    PanelFileRecord, PanelImpactSnapshot, PanelIndex, PanelManifestSnapshot,
    PanelPreflightSnapshot, PanelProtectionSnapshot, PanelRelationRecord, PanelSearchResult,
    PanelSearchSnapshot, PanelStatus, PanelStatusSnapshot, PanelSymbolRecord, PanelTestHint,
    PanelWatcherDetail, PanelWatcherSnapshot,
};
pub use preflight::panel_preflight;
pub use protection::check_panel_git_protection;
pub use search::search_project_panel;
pub use test_recommendation::recommend_panel_tests;
pub use watcher::ensure_panel_watcher;
