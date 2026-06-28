//! AgentFlow ontology registry and core object/link definitions.
//!
//! This crate owns the built-in project-world definition layer used by the
//! next runtime foundation. It does not execute actions, append runtime
//! events, rebuild projections, or mutate `.agentflow/**` facts.

pub mod core;
pub mod decision;
pub mod kernel;
pub mod model;
pub mod registry;
pub mod schema;
pub mod semantics;
pub mod skill;
pub mod storage;
pub mod validation;

pub use core::{core_ontology_bundle, core_ontology_registry, CORE_ONTOLOGY_REF};
pub use decision::{
    core_evidence_decision_reference_model_contract,
    validate_core_evidence_decision_reference_model_contract, CoreDecisionOutcomeDefinition,
    CoreDecisionReferenceDefinition, CoreEvidenceDecisionReferenceModelContract,
    CoreEvidenceReferenceDefinition, CORE_EVIDENCE_DECISION_MODEL_VERSION,
};
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
pub use schema::{
    core_object_link_schema_contract, validate_core_object_link_schema_contract,
    CoreLinkSchemaDefinition, CoreObjectLinkSchemaContract, CoreObjectSchemaDefinition,
    CORE_OBJECT_LINK_SCHEMA_VERSION,
};
pub use semantics::{
    core_action_state_semantics_contract, validate_core_action_state_semantics_contract,
    CoreActionSemanticDefinition, CoreActionStateSemanticsContract, CoreStateSemanticDefinition,
    CoreStateTransitionDefinition, CORE_ACTION_STATE_SEMANTICS_VERSION,
};
pub use skill::{
    core_skill_registry_contract, validate_core_skill_registry_contract, CoreSkillDefinition,
    CoreSkillRegistryContract, CORE_SKILL_REGISTRY_VERSION,
};
pub use storage::{read_ontology_bundle, write_ontology_bundle};
pub use validation::{validate_link, validate_ontology_bundle};
