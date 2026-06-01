//! Legacy compatibility module.
//!
//! This module belongs to the archived 2026-05 workflow/product-feature system.
//! It is kept only for compatibility and migration.
//! New AgentFlow requirements must not depend on this module unless a new
//! requirement explicitly re-authorizes it.

pub use super::archive_2026_05::{
    create_product_feature, read_product_feature_execution_next,
    read_product_feature_execution_status, ProductFeatureCreationSnapshot,
    ProductFeatureCreationSummary, ProductFeatureDraft, ProductFeatureExecutionIssue,
    ProductFeatureExecutionMilestone, ProductFeatureExecutionSnapshot, ProductFeatureIssueDraft,
    ProductFeatureMilestoneDraft, ProductFeatureProject,
};
