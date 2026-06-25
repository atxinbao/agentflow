//! AgentFlow public release record generator.
//!
//! This crate aggregates completed task projections into public delivery
//! documents such as CHANGELOG entries and release notes. It does not
//! participate in the single-task loop and does not write task artifacts.

pub mod deployment;
pub mod model;
pub mod public_delivery;
pub mod review_surface;
pub mod runtime;

pub use deployment::{build_deployment_evidence_report, DeploymentEvidenceInput};
pub use model::{
    DeliverySummary, DeploymentArtifactRef, DeploymentEvidenceReport, DeploymentSemanticCheck,
    DeploymentShapeEvidence, ExternalReviewAuditSummary, ExternalReviewEvidenceEntry,
    ProjectDeliverySummary, ProjectExternalReviewIndex, ProjectExternalReviewIndexEntry,
    ProjectExternalReviewSurface, ProjectReleaseFacts, ProjectReleaseIndex,
    ProjectReleaseIndexEntry, PublicReleaseDocumentPaths, PublicReleaseDocumentTarget,
    PublicReleaseEntry, PublicReleaseSummary, ReleaseTagProof, RemoteReleaseProof, RollbackModel,
    CHANGELOG_TEMPLATE_VERSION, DELIVERY_SUMMARY_VERSION, DEPLOYMENT_EVIDENCE_REPORT_VERSION,
    PROJECT_DELIVERY_SUMMARY_VERSION, PROJECT_EXTERNAL_REVIEW_INDEX_VERSION,
    PROJECT_EXTERNAL_REVIEW_SURFACE_VERSION, PROJECT_RELEASE_FACTS_VERSION,
    PROJECT_RELEASE_INDEX_VERSION, PUBLIC_RELEASE_SUMMARY_VERSION, RELEASE_NOTES_TEMPLATE_VERSION,
    RELEASE_TAG_PROOF_VERSION, REMOTE_RELEASE_PROOF_VERSION, TASK_PUBLIC_RECORD_TEMPLATE_VERSION,
};
pub use public_delivery::{
    collect_public_release_summary, collect_public_release_summary_for_project,
    load_delivery_summary, load_project_delivery_summary, write_public_release_documents,
};
pub use review_surface::{
    load_project_external_review_surface, sync_project_external_review_surface,
};
pub use runtime::{
    confirm_project_release, load_project_release_facts, load_project_release_index,
    prepare_project_release, publish_project_release, record_project_release_tag,
    record_project_remote_release, sync_project_release,
};
