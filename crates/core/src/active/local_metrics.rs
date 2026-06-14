//! Active transitional local metrics snapshot.
//!
//! The implementation is still archived legacy code. This wrapper keeps the
//! current Desktop read path alive without promoting the old workflow as the new
//! product model.

pub use crate::legacy::workflow_control::{
    read_local_metrics_snapshot, LocalMetricArtifactRef, LocalMetricRunRef, LocalMetricsSnapshot,
};
