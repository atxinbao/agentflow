use crate::model::{
    Cardinality, DefinitionKind, DefinitionStatus, LinkTypeDefinition, ObjectTypeDefinition,
    OntologyBundle, OntologyCompatibility, OntologyDefinitionRecord, OntologyMigration,
    OntologyPropertyDefinition, OntologyPropertyValueType, ONTOLOGY_BUNDLE_VERSION,
    ONTOLOGY_RECORD_VERSION,
};
use crate::registry::OntologyRegistry;
use crate::schema::core_object_link_schema_contract;

pub const CORE_ONTOLOGY_ID: &str = "agentflow.core";
pub const CORE_ONTOLOGY_NAMESPACE: &str = "agentflow.core";
pub const CORE_ONTOLOGY_VERSION: &str = "v1-draft";
pub const CORE_ONTOLOGY_REF: &str = "agentflow.core@v1-draft";
pub const SOFTWARE_DEV_REFERENCE_ONTOLOGY_ID: &str = "agentflow.reference.software-dev";
pub const SOFTWARE_DEV_REFERENCE_ONTOLOGY_NAMESPACE: &str = "agentflow.reference.software-dev";
pub const SOFTWARE_DEV_REFERENCE_ONTOLOGY_VERSION: &str = "v1-draft";
pub const SOFTWARE_DEV_REFERENCE_ONTOLOGY_REF: &str = "agentflow.reference.software-dev@v1-draft";
const CORE_OWNER: &str = "agentflow";
const CORE_TIMESTAMP: &str = "2026-06-20T00:00:00Z";

pub fn core_ontology_bundle() -> OntologyBundle {
    let schema_contract = core_object_link_schema_contract();
    let object_types = schema_contract
        .object_schemas
        .iter()
        .map(|schema| {
            object_type(
                &schema.object_type,
                &schema.object_type,
                &schema.description,
                schema
                    .required_fields
                    .iter()
                    .map(|field| {
                        property(
                            field,
                            OntologyPropertyValueType::String,
                            true,
                            "Core object authority field.",
                        )
                    })
                    .collect(),
                schema
                    .allowed_link_types
                    .iter()
                    .map(String::as_str)
                    .collect(),
                vec!["coreProjection"],
            )
        })
        .collect();
    let link_types = schema_contract
        .link_schemas
        .iter()
        .map(|schema| {
            link_type(
                &schema.link_type,
                &schema.source_object_type,
                &schema.target_object_type,
                cardinality_from_schema(&schema.cardinality),
                &schema.description,
            )
        })
        .collect();

    build_ontology_bundle(
        CORE_ONTOLOGY_ID,
        CORE_ONTOLOGY_NAMESPACE,
        CORE_ONTOLOGY_VERSION,
        object_types,
        link_types,
    )
}

pub fn software_dev_reference_ontology_bundle() -> OntologyBundle {
    let object_types = vec![
        object_type(
            "Requirement",
            "Requirement",
            "Standardized requirement intake object.",
            vec![
                property(
                    "requirementId",
                    OntologyPropertyValueType::String,
                    true,
                    "Stable requirement identifier.",
                ),
                property(
                    "intentType",
                    OntologyPropertyValueType::String,
                    true,
                    "Normalized intent type.",
                ),
                property(
                    "summary",
                    OntologyPropertyValueType::String,
                    true,
                    "Requirement summary.",
                ),
            ],
            vec!["derivesFrom"],
            vec!["requirementIntake"],
        ),
        object_type(
            "Spec",
            "Spec",
            "Confirmed requirement boundary and acceptance package.",
            vec![
                property(
                    "specId",
                    OntologyPropertyValueType::String,
                    true,
                    "Stable spec identifier.",
                ),
                property(
                    "title",
                    OntologyPropertyValueType::String,
                    true,
                    "Spec title.",
                ),
                property(
                    "status",
                    OntologyPropertyValueType::String,
                    true,
                    "Current spec status.",
                ),
            ],
            vec!["derivesFrom", "accepts"],
            vec!["specWorkbench"],
        ),
        object_type(
            "Project",
            "Project",
            "Top-level project aggregate projection.",
            vec![
                property(
                    "projectId",
                    OntologyPropertyValueType::String,
                    true,
                    "Stable project identifier.",
                ),
                property(
                    "title",
                    OntologyPropertyValueType::String,
                    true,
                    "Project title.",
                ),
                property(
                    "status",
                    OntologyPropertyValueType::String,
                    true,
                    "Current project status.",
                ),
            ],
            vec!["contains"],
            vec!["projectHome"],
        ),
        object_type(
            "Issue",
            "Issue",
            "Executable work contract.",
            vec![
                property(
                    "issueId",
                    OntologyPropertyValueType::String,
                    true,
                    "Stable issue identifier.",
                ),
                property(
                    "title",
                    OntologyPropertyValueType::String,
                    true,
                    "Issue title.",
                ),
                property(
                    "status",
                    OntologyPropertyValueType::String,
                    true,
                    "Current issue status.",
                ),
                property(
                    "priority",
                    OntologyPropertyValueType::String,
                    false,
                    "Priority hint.",
                ),
                property(
                    "requiredAgentRole",
                    OntologyPropertyValueType::String,
                    false,
                    "Authorized runtime role.",
                ),
                property(
                    "workflowRef",
                    OntologyPropertyValueType::String,
                    false,
                    "Workflow reference.",
                ),
            ],
            vec!["contains", "blocks", "executes", "proves", "requiresFix"],
            vec!["taskWorkbench", "issueIndex"],
        ),
        object_type(
            "Run",
            "Run",
            "One execution attempt for an issue.",
            vec![
                property(
                    "runId",
                    OntologyPropertyValueType::String,
                    true,
                    "Stable run identifier.",
                ),
                property(
                    "status",
                    OntologyPropertyValueType::String,
                    true,
                    "Current run status.",
                ),
                property(
                    "issueId",
                    OntologyPropertyValueType::String,
                    true,
                    "Linked issue identifier.",
                ),
            ],
            vec!["executes", "produces", "supports"],
            vec!["taskTimeline"],
        ),
        object_type(
            "Evidence",
            "Evidence",
            "Verification proof such as logs, test results, or screenshots.",
            vec![
                property(
                    "evidenceId",
                    OntologyPropertyValueType::String,
                    true,
                    "Stable evidence identifier.",
                ),
                property(
                    "kind",
                    OntologyPropertyValueType::String,
                    true,
                    "Evidence kind.",
                ),
                property(
                    "path",
                    OntologyPropertyValueType::String,
                    true,
                    "Local evidence path.",
                ),
            ],
            vec!["proves", "supports", "reviews"],
            vec!["evidenceGraph", "deliveryPackage"],
        ),
        object_type(
            "Artifact",
            "Artifact",
            "Code, document, or public delivery reference produced by a run.",
            vec![
                property(
                    "artifactId",
                    OntologyPropertyValueType::String,
                    true,
                    "Stable artifact identifier.",
                ),
                property(
                    "kind",
                    OntologyPropertyValueType::String,
                    true,
                    "Artifact kind.",
                ),
                property(
                    "reference",
                    OntologyPropertyValueType::String,
                    true,
                    "Artifact reference.",
                ),
            ],
            vec!["produces"],
            vec!["deliveryPackage"],
        ),
        object_type(
            "Decision",
            "Decision",
            "Human confirmation, rejection, or governance decision.",
            vec![
                property(
                    "decisionId",
                    OntologyPropertyValueType::String,
                    true,
                    "Stable decision identifier.",
                ),
                property(
                    "outcome",
                    OntologyPropertyValueType::String,
                    true,
                    "Decision outcome.",
                ),
                property(
                    "reason",
                    OntologyPropertyValueType::String,
                    false,
                    "Decision reason.",
                ),
            ],
            vec!["decides", "accepts"],
            vec!["decisionLog"],
        ),
        object_type(
            "Audit",
            "Audit",
            "Independent audit flow anchored to task evidence.",
            vec![
                property(
                    "auditId",
                    OntologyPropertyValueType::String,
                    true,
                    "Stable audit identifier.",
                ),
                property(
                    "status",
                    OntologyPropertyValueType::String,
                    true,
                    "Current audit status.",
                ),
                property(
                    "targetIssueId",
                    OntologyPropertyValueType::String,
                    false,
                    "Audited issue identifier.",
                ),
            ],
            vec!["reviews"],
            vec!["auditSurface"],
        ),
        object_type(
            "Finding",
            "Finding",
            "Independent audit or review finding.",
            vec![
                property(
                    "findingId",
                    OntologyPropertyValueType::String,
                    true,
                    "Stable finding identifier.",
                ),
                property(
                    "severity",
                    OntologyPropertyValueType::String,
                    true,
                    "Finding severity.",
                ),
                property(
                    "status",
                    OntologyPropertyValueType::String,
                    true,
                    "Finding status.",
                ),
            ],
            vec!["reviews", "requiresFix"],
            vec!["auditSurface"],
        ),
    ];

    let link_types = vec![
        link_type(
            "derivesFrom",
            "Spec",
            "Requirement",
            Cardinality::ManyToOne,
            "Spec derives from a standardized requirement.",
        ),
        link_type(
            "contains",
            "Project",
            "Issue",
            Cardinality::OneToMany,
            "Project contains executable issues.",
        ),
        link_type(
            "blocks",
            "Issue",
            "Issue",
            Cardinality::ManyToMany,
            "Issue blocks another issue.",
        ),
        link_type(
            "executes",
            "Run",
            "Issue",
            Cardinality::ManyToOne,
            "Run executes an issue.",
        ),
        link_type(
            "produces",
            "Run",
            "Artifact",
            Cardinality::OneToMany,
            "Run produces artifacts.",
        ),
        link_type(
            "proves",
            "Evidence",
            "Issue",
            Cardinality::ManyToOne,
            "Evidence proves issue completion or acceptance.",
        ),
        link_type(
            "supports",
            "Evidence",
            "Run",
            Cardinality::ManyToOne,
            "Evidence supports a specific run.",
        ),
        link_type(
            "reviews",
            "Finding",
            "Evidence",
            Cardinality::ManyToMany,
            "Finding reviews evidence.",
        ),
        link_type(
            "requiresFix",
            "Finding",
            "Issue",
            Cardinality::OneToMany,
            "Finding requires fix issue.",
        ),
        link_type(
            "decides",
            "Decision",
            "Requirement",
            Cardinality::ManyToOne,
            "Decision affects requirement handling.",
        ),
        link_type(
            "accepts",
            "Decision",
            "Spec",
            Cardinality::ManyToOne,
            "Decision accepts a spec.",
        ),
    ];

    build_ontology_bundle(
        SOFTWARE_DEV_REFERENCE_ONTOLOGY_ID,
        SOFTWARE_DEV_REFERENCE_ONTOLOGY_NAMESPACE,
        SOFTWARE_DEV_REFERENCE_ONTOLOGY_VERSION,
        object_types,
        link_types,
    )
}

pub fn core_ontology_registry() -> OntologyRegistry {
    OntologyRegistry::load_bundle(core_ontology_bundle())
        .expect("built-in core ontology must validate")
}

pub fn software_dev_reference_ontology_registry() -> OntologyRegistry {
    OntologyRegistry::load_bundle(software_dev_reference_ontology_bundle())
        .expect("built-in Software Dev reference ontology must validate")
}

fn build_ontology_bundle(
    ontology_id: &str,
    namespace: &str,
    definition_version: &str,
    object_types: Vec<ObjectTypeDefinition>,
    link_types: Vec<LinkTypeDefinition>,
) -> OntologyBundle {
    let definition_records = object_types
        .iter()
        .map(|definition| {
            definition_record(
                &definition.id,
                DefinitionKind::ObjectType,
                namespace,
                definition_version,
            )
        })
        .chain(link_types.iter().map(|definition| {
            definition_record(
                &definition.id,
                DefinitionKind::LinkType,
                namespace,
                definition_version,
            )
        }))
        .collect();

    OntologyBundle {
        version: ONTOLOGY_BUNDLE_VERSION.into(),
        ontology_id: ontology_id.into(),
        namespace: namespace.into(),
        definition_version: definition_version.into(),
        status: DefinitionStatus::Draft,
        object_types,
        link_types,
        definition_records,
        compatibility: Some(OntologyCompatibility {
            replay_from_version: definition_version.into(),
        }),
        migration: Some(OntologyMigration {
            strategy: "explicit".into(),
        }),
    }
}

fn object_type(
    id: &str,
    name: &str,
    description: &str,
    properties: Vec<OntologyPropertyDefinition>,
    allowed_link_types: Vec<&str>,
    projection_hints: Vec<&str>,
) -> ObjectTypeDefinition {
    let required_properties = properties
        .iter()
        .filter(|property| property.required)
        .map(|property| property.name.clone())
        .collect();
    ObjectTypeDefinition {
        id: id.into(),
        name: name.into(),
        description: description.into(),
        properties,
        required_properties,
        state_machine_ref: None,
        allowed_link_types: allowed_link_types.into_iter().map(str::to_string).collect(),
        allowed_action_types: Vec::new(),
        projection_hints: projection_hints.into_iter().map(str::to_string).collect(),
    }
}

fn link_type(
    id: &str,
    source_object_type: &str,
    target_object_type: &str,
    cardinality: Cardinality,
    description: &str,
) -> LinkTypeDefinition {
    LinkTypeDefinition {
        id: id.into(),
        source_object_type: source_object_type.into(),
        target_object_type: target_object_type.into(),
        cardinality,
        description: description.into(),
        allowed_actions: Vec::new(),
        projection_hints: Vec::new(),
    }
}

fn cardinality_from_schema(value: &str) -> Cardinality {
    match value {
        "one-to-one" => Cardinality::OneToOne,
        "one-to-many" => Cardinality::OneToMany,
        "many-to-one" => Cardinality::ManyToOne,
        "many-to-many" => Cardinality::ManyToMany,
        _ => Cardinality::ManyToMany,
    }
}

fn property(
    name: &str,
    value_type: OntologyPropertyValueType,
    required: bool,
    description: &str,
) -> OntologyPropertyDefinition {
    OntologyPropertyDefinition {
        name: name.into(),
        value_type,
        required,
        description: description.into(),
    }
}

fn definition_record(
    id: &str,
    kind: DefinitionKind,
    namespace: &str,
    definition_version: &str,
) -> OntologyDefinitionRecord {
    OntologyDefinitionRecord {
        version: ONTOLOGY_RECORD_VERSION.into(),
        id: id.into(),
        namespace: namespace.into(),
        kind,
        definition_version: definition_version.into(),
        status: DefinitionStatus::Draft,
        owner: CORE_OWNER.into(),
        created_at: CORE_TIMESTAMP.into(),
        updated_at: CORE_TIMESTAMP.into(),
        compatibility: None,
        deprecation: None,
    }
}
