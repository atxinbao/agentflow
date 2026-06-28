//! AgentFlow ontology registry and core object/link definitions.
//!
//! This crate owns the built-in project-world definition layer used by the
//! next runtime foundation. It does not execute actions, append runtime
//! events, rebuild projections, or mutate `.agentflow/**` facts.

pub mod core;
pub mod kernel;
pub mod model;
pub mod registry;
pub mod storage;
pub mod validation;

pub use core::{core_ontology_bundle, core_ontology_registry, CORE_ONTOLOGY_REF};
pub use kernel::{
    core_ontology_kernel_contract, validate_core_ontology_kernel_contract, CoreOntologyElement,
    CoreOntologyElementDefinition, CoreOntologyKernelContract, CORE_ONTOLOGY_KERNEL_VERSION,
};
pub use model::{
    Cardinality, DefinitionCompatibility, DefinitionDeprecation, DefinitionKind, DefinitionStatus,
    LinkTypeDefinition, ObjectTypeDefinition, OntologyBundle, OntologyDefinitionRecord,
    OntologyMigration, OntologyPropertyDefinition, OntologyPropertyValueType,
    OntologyValidationError, OntologyValidationReport,
};
pub use registry::OntologyRegistry;
pub use storage::{read_ontology_bundle, write_ontology_bundle};
pub use validation::{validate_link, validate_ontology_bundle};
