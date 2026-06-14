//! Active transitional read models.
//!
//! These APIs exist so the current Desktop can keep rendering read-only
//! snapshots while the new AgentFlow workflow is being defined.
//!
//! New write flows must not be added here without a new requirement.

pub mod boundary;
pub mod local_metrics;
pub mod local_project_model;
pub mod local_search;

pub use boundary::WorkbenchBoundary;
pub use local_metrics::{read_local_metrics_snapshot, LocalMetricsSnapshot};
pub use local_project_model::{read_local_project_model_snapshot, LocalProjectModelSnapshot};
pub use local_search::{read_local_search_snapshot, LocalSearchSnapshot};
