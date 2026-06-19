use serde::{Deserialize, Serialize};

pub const ONTOLOGY_BUNDLE_VERSION: &str = "agentflow-ontology-bundle.v1";
pub const ONTOLOGY_RECORD_VERSION: &str = "agentflow-ontology-record.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DefinitionStatus {
    Draft,
    Active,
    Deprecated,
    Retired,
}

impl Default for DefinitionStatus {
    fn default() -> Self {
        Self::Draft
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DefinitionKind {
    ObjectType,
    LinkType,
    StateMachine,
    ActionType,
    FunctionType,
    AgentRolePolicy,
    ProjectionModel,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Cardinality {
    OneToOne,
    OneToMany,
    ManyToOne,
    ManyToMany,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OntologyPropertyValueType {
    String,
    Integer,
    Number,
    Boolean,
    Object,
    Array,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OntologyCompatibility {
    pub replay_from_version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OntologyMigration {
    pub strategy: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DefinitionCompatibility {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub aliases: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub replaces: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DefinitionDeprecation {
    pub reason: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub replaced_by: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OntologyPropertyDefinition {
    pub name: String,
    pub value_type: OntologyPropertyValueType,
    #[serde(default)]
    pub required: bool,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObjectTypeDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub properties: Vec<OntologyPropertyDefinition>,
    #[serde(default)]
    pub required_properties: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state_machine_ref: Option<String>,
    #[serde(default)]
    pub allowed_link_types: Vec<String>,
    #[serde(default)]
    pub allowed_action_types: Vec<String>,
    #[serde(default)]
    pub projection_hints: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkTypeDefinition {
    pub id: String,
    pub source_object_type: String,
    pub target_object_type: String,
    pub cardinality: Cardinality,
    pub description: String,
    #[serde(default)]
    pub allowed_actions: Vec<String>,
    #[serde(default)]
    pub projection_hints: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OntologyDefinitionRecord {
    pub version: String,
    pub id: String,
    pub namespace: String,
    pub kind: DefinitionKind,
    pub definition_version: String,
    pub status: DefinitionStatus,
    pub owner: String,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compatibility: Option<DefinitionCompatibility>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deprecation: Option<DefinitionDeprecation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OntologyBundle {
    pub version: String,
    pub ontology_id: String,
    pub namespace: String,
    pub definition_version: String,
    pub status: DefinitionStatus,
    #[serde(default)]
    pub object_types: Vec<ObjectTypeDefinition>,
    #[serde(default)]
    pub link_types: Vec<LinkTypeDefinition>,
    #[serde(default)]
    pub definition_records: Vec<OntologyDefinitionRecord>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compatibility: Option<OntologyCompatibility>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub migration: Option<OntologyMigration>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OntologyValidationError {
    pub code: String,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OntologyValidationReport {
    pub valid: bool,
    #[serde(default)]
    pub errors: Vec<OntologyValidationError>,
}

impl OntologyValidationReport {
    pub fn success() -> Self {
        Self {
            valid: true,
            errors: Vec::new(),
        }
    }

    pub fn push_error(
        &mut self,
        code: impl Into<String>,
        message: impl Into<String>,
        path: impl Into<Option<String>>,
    ) {
        self.valid = false;
        self.errors.push(OntologyValidationError {
            code: code.into(),
            message: message.into(),
            path: path.into(),
        });
    }

    pub fn merge(&mut self, other: Self) {
        if !other.valid {
            self.valid = false;
        }
        self.errors.extend(other.errors);
    }
}
