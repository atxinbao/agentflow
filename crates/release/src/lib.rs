//! AgentFlow public release record generator.
//!
//! This crate aggregates completed task projections into public delivery
//! documents such as CHANGELOG entries and release notes. It does not
//! participate in the single-task loop and does not write task artifacts.

pub mod model;
pub mod public_delivery;
pub mod runtime;

pub use model::{
    DeliverySummary, ProjectDeliverySummary, ProjectReleaseFacts, ProjectReleaseIndex,
    ProjectReleaseIndexEntry, PublicReleaseDocumentPaths, PublicReleaseDocumentTarget,
    PublicReleaseEntry, PublicReleaseSummary, DELIVERY_SUMMARY_VERSION,
    PROJECT_DELIVERY_SUMMARY_VERSION, PROJECT_RELEASE_FACTS_VERSION, PROJECT_RELEASE_INDEX_VERSION,
    PUBLIC_RELEASE_SUMMARY_VERSION,
};
pub use public_delivery::{
    collect_public_release_summary, collect_public_release_summary_for_project,
    load_delivery_summary, load_project_delivery_summary, write_public_release_documents,
};
pub use runtime::{load_project_release_facts, load_project_release_index, sync_project_release};
