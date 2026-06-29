//! AgentFlow ontology registry and core object/link definitions.
//!
//! This crate owns the built-in project-world definition layer used by the
//! next runtime foundation. It does not execute actions, append runtime
//! events, rebuild projections, or mutate `.agentflow/**` facts.

pub mod core;
pub mod decision;
pub mod evidence;
pub mod file_registry;
pub mod kernel;
pub mod model;
pub mod registry;
pub mod schema;
pub mod semantics;
pub mod skill;
pub mod storage;
pub mod validation;

pub use core::{
    core_ontology_bundle, core_ontology_registry, software_dev_reference_ontology_bundle,
    software_dev_reference_ontology_registry, CORE_ONTOLOGY_REF,
    SOFTWARE_DEV_REFERENCE_ONTOLOGY_REF,
};
pub use decision::{
    canonical_core_decision_failure_reason_fixture, canonical_core_decision_input_binding_fixture,
    canonical_core_decision_record_fixture, canonical_core_decision_transition_attempt_fixture,
    core_decision_failure_reason_contract, core_decision_input_binding_contract,
    core_decision_model_contract, core_decision_outcome_transition_contract,
    core_evidence_decision_reference_model_contract, validate_core_decision_failure_reason,
    validate_core_decision_failure_reason_contract, validate_core_decision_input_binding,
    validate_core_decision_input_binding_contract, validate_core_decision_model_contract,
    validate_core_decision_outcome_transition_contract, validate_core_decision_record,
    validate_core_decision_transition_attempt,
    validate_core_evidence_decision_reference_model_contract, CoreDecisionBoundAuthorityRef,
    CoreDecisionFailureReason, CoreDecisionFailureReasonContract,
    CoreDecisionInputAuthorityRequirement, CoreDecisionInputBinding,
    CoreDecisionInputBindingContract, CoreDecisionInputs, CoreDecisionKernelOutcome,
    CoreDecisionModelContract, CoreDecisionOutcomeDefinition, CoreDecisionOutcomeTransition,
    CoreDecisionOutcomeTransitionContract, CoreDecisionReadableFact, CoreDecisionReason,
    CoreDecisionReasonShape, CoreDecisionRecord, CoreDecisionReferenceDefinition,
    CoreDecisionRemediationRoute, CoreDecisionSubjectRef, CoreDecisionTransitionAttempt,
    CoreDecisionWriteAuthority, CoreDecisionWriteRef, CoreEvidenceDecisionReferenceModelContract,
    CoreEvidenceReferenceDefinition, CORE_DECISION_FAILURE_REASON_CONTRACT_VERSION,
    CORE_DECISION_INPUT_BINDING_CONTRACT_VERSION, CORE_DECISION_MODEL_CONTRACT_VERSION,
    CORE_DECISION_OUTCOME_TRANSITION_CONTRACT_VERSION, CORE_EVIDENCE_DECISION_MODEL_VERSION,
};
pub use evidence::{
    canonical_core_evidence_authority_trace_fixture,
    canonical_core_evidence_capture_receipt_fixture,
    canonical_core_evidence_completeness_policy_fixture, canonical_core_evidence_pack_fixture,
    canonical_core_external_proof_expectation_fixture,
    canonical_core_external_proof_receipt_fixture, capture_core_evidence_receipt_for_local_file,
    core_evidence_authority_trace_negative_fixtures,
    core_evidence_capture_receipt_negative_fixtures,
    core_evidence_completeness_policy_sample_packs, core_evidence_pack_negative_fixtures,
    core_evidence_source_type_registry_contract, core_external_proof_negative_fixtures,
    core_missing_evidence_negative_fixtures, core_missing_evidence_report_for_pack,
    core_missing_evidence_reports_for_completeness_policy,
    evaluate_core_evidence_completeness_policy, external_core_evidence_reference_receipt,
    software_dev_evidence_reference_mapping_contract,
    software_dev_reference_evidence_completeness_policy,
    software_dev_reference_evidence_fixture_packs, validate_core_evidence_authority_trace,
    validate_core_evidence_capture_receipt, validate_core_evidence_pack_schema,
    validate_core_evidence_pack_source_type, validate_core_evidence_source_type_registry_contract,
    validate_core_external_proof_receipt,
    validate_software_dev_evidence_reference_mapping_contract, CoreEvidenceArtifactRef,
    CoreEvidenceAuthorityFactRef, CoreEvidenceAuthorityTrace,
    CoreEvidenceAuthorityTraceNegativeFixtureResult, CoreEvidenceCaptureLocation,
    CoreEvidenceCaptureReceipt, CoreEvidenceCaptureReceiptNegativeFixtureResult,
    CoreEvidenceCollectionEventLink, CoreEvidenceCompletenessEvaluation,
    CoreEvidenceCompletenessPolicy, CoreEvidenceDigest, CoreEvidencePack,
    CoreEvidencePackNegativeFixtureResult, CoreEvidenceProducerRef, CoreEvidenceProvenance,
    CoreEvidenceReferenceAppSourceExample, CoreEvidenceRequirementGroup, CoreEvidenceRetentionHint,
    CoreEvidenceSourceTypeDefinition, CoreEvidenceSourceTypeRegistryContract,
    CoreEvidenceSubjectRef, CoreEvidenceTraceRefs, CoreExternalProofExpectation,
    CoreExternalProofNegativeFixtureResult, CoreExternalProofReceipt,
    CoreMissingEvidenceNegativeFixtureResult, CoreMissingEvidenceReport,
    SoftwareDevEvidenceReferenceMapping, SoftwareDevEvidenceReferenceMappingContract,
    CORE_EVIDENCE_AUTHORITY_TRACE_VERSION, CORE_EVIDENCE_CAPTURE_RECEIPT_VERSION,
    CORE_EVIDENCE_COMPLETENESS_POLICY_VERSION, CORE_EVIDENCE_PACK_SCHEMA_VERSION,
    CORE_EVIDENCE_SOURCE_TYPE_REGISTRY_VERSION, CORE_EXTERNAL_PROOF_RECEIPT_VERSION,
    CORE_MISSING_EVIDENCE_REPORT_VERSION, SOFTWARE_DEV_EVIDENCE_REFERENCE_MAPPING_VERSION,
};
pub use file_registry::{
    core_file_backed_ontology_registry_projection_contract,
    diagnose_core_file_backed_ontology_registry_projection_contract,
    load_core_file_backed_ontology_registry_projection,
    load_core_file_backed_ontology_registry_projection_with_contract,
    validate_core_file_backed_ontology_registry_projection_contract,
    CoreFileBackedOntologyRegistryLoadError, CoreFileBackedOntologyRegistryProjectionContract,
    CoreFileBackedOntologyRuntimeLoader, CoreFileBackedOntologyRuntimeProjection,
    CoreOntologyProjectionEntryDefinition, CoreOntologyRegistryPollutionDiagnostic,
    CoreOntologyRegistrySourceDefinition, LoadedCoreOntologyRegistrySource,
    CORE_FILE_BACKED_ONTOLOGY_REGISTRY_VERSION,
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
