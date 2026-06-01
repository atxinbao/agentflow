//! Active transitional local search snapshot.
//!
//! The implementation is still archived legacy code. This wrapper keeps search
//! read-only and separate from new write requirements.

pub use crate::legacy::workflow_control::{read_local_search_snapshot, LocalSearchSnapshot};
