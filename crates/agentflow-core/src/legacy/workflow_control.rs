//! Legacy compatibility module.
//!
//! This module belongs to the archived 2026-05 workflow/product-feature system.
//! It is kept only for compatibility and migration.
//! New AgentFlow requirements must not depend on this module unless a new
//! requirement explicitly re-authorizes it.

pub use super::archive_2026_05::{
    read_desktop_workbench_snapshot, read_local_metrics_snapshot, read_local_search_snapshot,
    AgentScopeState, ContextFile, DesktopWorkbenchSnapshot, GoalLoopSelection,
    LocalArtifactMetrics, LocalMetricArtifactRef, LocalMetricRunRef, LocalMetricsSnapshot,
    LocalRunMetrics, LocalSearchQuery, LocalSearchResult, LocalSearchSnapshot, ProjectContext,
    Settings, WorkbenchBoundary, WorkbenchCounts, WorkbenchTextArtifact,
};
