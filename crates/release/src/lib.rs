//! AgentFlow public release record generator.
//!
//! This crate aggregates completed task projections into public delivery
//! documents such as CHANGELOG entries and release notes. It does not
//! participate in the single-task loop and does not write task artifacts.

pub mod model;
pub mod public_delivery;

pub use model::{
    PublicReleaseDocumentPaths, PublicReleaseDocumentTarget, PublicReleaseEntry,
    PublicReleaseSummary, PUBLIC_RELEASE_SUMMARY_VERSION,
};
pub use public_delivery::{collect_public_release_summary, write_public_release_documents};
