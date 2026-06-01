//! Active transitional Desktop workbench snapshot.
//!
//! The implementation is still archived legacy code. This wrapper makes the
//! compatibility dependency explicit while the new AgentFlow read model is
//! being rebuilt.

pub use crate::legacy::workflow_control::{
    read_desktop_workbench_snapshot, DesktopWorkbenchSnapshot,
};
