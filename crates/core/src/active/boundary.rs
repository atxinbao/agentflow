//! Active transitional boundary model.
//!
//! This file only re-exports the read-only Desktop boundary shape that is still
//! needed by the current UI. New write flows must be defined by a new
//! requirement before they can enter `active`.

pub use crate::legacy::workflow_control::WorkbenchBoundary;
