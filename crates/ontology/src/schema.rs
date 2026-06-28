use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const CORE_OBJECT_LINK_SCHEMA_VERSION: &str = "agentflow-core-object-link-schema.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreObjectSchemaDefinition {
    pub object_type: String,
    pub description: String,
    pub required_fields: Vec<String>,
    pub allowed_link_types: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreLinkSchemaDefinition {
    pub link_type: String,
    pub source_object_type: String,
    pub target_object_type: String,
    pub cardinality: String,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreObjectLinkSchemaContract {
    pub version: String,
    pub status: String,
    pub authority: String,
    pub object_schemas: Vec<CoreObjectSchemaDefinition>,
    pub link_schemas: Vec<CoreLinkSchemaDefinition>,
    pub forbidden_core_terms: Vec<String>,
}

pub fn core_object_link_schema_contract() -> CoreObjectLinkSchemaContract {
    CoreObjectLinkSchemaContract {
        version: CORE_OBJECT_LINK_SCHEMA_VERSION.to_string(),
        status: "active".to_string(),
        authority: "Core object and link schemas are industry-neutral project-world primitives. Reference mappings are not Core authority."
            .to_string(),
        object_schemas: vec![
            object_schema("RequestObject", "Captured human intent before confirmation.", &["objectId", "summary", "status"], &["derivesFrom", "decides"]),
            object_schema("IntentObject", "Normalized intent and domain context packet.", &["objectId", "intentType", "status"], &["derivesFrom", "routesTo"]),
            object_schema("GoalObject", "Desired outcome record with acceptance boundary.", &["objectId", "title", "status"], &["derivesFrom", "contains"]),
            object_schema("PlanObject", "Ordered path for producing one or more work units.", &["objectId", "title", "status"], &["contains", "blocks"]),
            object_schema("WorkObject", "Executable unit of project work.", &["objectId", "title", "status"], &["contains", "blocks", "executes", "proves", "requiresFollowUp"]),
            object_schema("ExecutionObject", "One controlled execution attempt for a work object.", &["objectId", "status", "targetObjectId"], &["executes", "produces", "supports"]),
            object_schema("EvidenceObject", "Proof reference for state changes and decisions.", &["objectId", "kind", "reference", "status"], &["proves", "supports", "reviews"]),
            object_schema("ArtifactObject", "Durable output reference produced by execution or review.", &["objectId", "kind", "reference", "status"], &["produces", "supports"]),
            object_schema("DecisionObject", "Recorded judgment or confirmation.", &["objectId", "outcome", "status"], &["decides", "accepts"]),
            object_schema("ReviewObject", "Independent review flow anchored to evidence.", &["objectId", "status", "targetObjectId"], &["reviews", "requiresFollowUp"]),
            object_schema("ProjectionObject", "Derived read model for UI or API consumption.", &["objectId", "status", "sourceRef"], &["supports"]),
        ],
        link_schemas: vec![
            link_schema("derivesFrom", "IntentObject", "RequestObject", "many-to-one", "Object is derived from a previous authority object."),
            link_schema("contains", "GoalObject", "PlanObject", "one-to-many", "Parent object contains child planning objects."),
            link_schema("blocks", "WorkObject", "WorkObject", "many-to-many", "Work object blocks another work object."),
            link_schema("executes", "ExecutionObject", "WorkObject", "many-to-one", "Execution object attempts a work object."),
            link_schema("produces", "ExecutionObject", "ArtifactObject", "one-to-many", "Execution object produces artifacts."),
            link_schema("proves", "EvidenceObject", "WorkObject", "many-to-one", "Evidence proves work completion or acceptance."),
            link_schema("supports", "EvidenceObject", "ExecutionObject", "many-to-one", "Evidence supports a specific execution object."),
            link_schema("reviews", "ReviewObject", "EvidenceObject", "many-to-many", "Review object reviews evidence."),
            link_schema("requiresFollowUp", "ReviewObject", "WorkObject", "one-to-many", "Review object requires follow-up work."),
            link_schema("decides", "DecisionObject", "RequestObject", "many-to-one", "Decision affects request handling."),
            link_schema("accepts", "DecisionObject", "GoalObject", "many-to-one", "Decision accepts a goal boundary."),
            link_schema("routesTo", "IntentObject", "GoalObject", "many-to-one", "Intent routes to a goal object."),
        ],
        forbidden_core_terms: vec![
            "bug".to_string(),
            "feature".to_string(),
            "issue".to_string(),
            "pr".to_string(),
            "pull-request".to_string(),
            "release".to_string(),
            "repository".to_string(),
            "repository-patch".to_string(),
            "test-log".to_string(),
            "github-issue".to_string(),
        ],
    }
}

pub fn validate_core_object_link_schema_contract(
    contract: &CoreObjectLinkSchemaContract,
) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    if contract.version != CORE_OBJECT_LINK_SCHEMA_VERSION {
        errors.push(format!(
            "schema version must be `{}`",
            CORE_OBJECT_LINK_SCHEMA_VERSION
        ));
    }
    if contract.status != "active" {
        errors.push("schema status must be active".to_string());
    }

    let object_ids: BTreeSet<_> = contract
        .object_schemas
        .iter()
        .map(|schema| schema.object_type.as_str())
        .collect();
    let link_ids: BTreeSet<_> = contract
        .link_schemas
        .iter()
        .map(|schema| schema.link_type.as_str())
        .collect();
    for required_object in [
        "RequestObject",
        "IntentObject",
        "GoalObject",
        "PlanObject",
        "WorkObject",
        "ExecutionObject",
        "EvidenceObject",
        "ArtifactObject",
        "DecisionObject",
        "ReviewObject",
        "ProjectionObject",
    ] {
        if !object_ids.contains(required_object) {
            errors.push(format!("missing Core object schema `{required_object}`"));
        }
    }
    for required_link in [
        "derivesFrom",
        "contains",
        "blocks",
        "executes",
        "produces",
        "proves",
        "supports",
        "reviews",
        "requiresFollowUp",
        "decides",
        "accepts",
        "routesTo",
    ] {
        if !link_ids.contains(required_link) {
            errors.push(format!("missing Core link schema `{required_link}`"));
        }
    }

    for object_schema in &contract.object_schemas {
        for required in ["objectId", "status"] {
            if !object_schema
                .required_fields
                .iter()
                .any(|field| field == required)
            {
                errors.push(format!(
                    "object schema `{}` must require `{required}`",
                    object_schema.object_type
                ));
            }
        }
        for link_type in &object_schema.allowed_link_types {
            if !link_ids.contains(link_type.as_str()) {
                errors.push(format!(
                    "object schema `{}` references unknown link `{link_type}`",
                    object_schema.object_type
                ));
            }
        }
    }

    for link_schema in &contract.link_schemas {
        if !object_ids.contains(link_schema.source_object_type.as_str()) {
            errors.push(format!(
                "link schema `{}` references missing source `{}`",
                link_schema.link_type, link_schema.source_object_type
            ));
        }
        if !object_ids.contains(link_schema.target_object_type.as_str()) {
            errors.push(format!(
                "link schema `{}` references missing target `{}`",
                link_schema.link_type, link_schema.target_object_type
            ));
        }
    }

    let core_surface = contract
        .object_schemas
        .iter()
        .flat_map(|schema| {
            [
                schema.object_type.clone(),
                schema.description.clone(),
                schema.required_fields.join(" "),
                schema.allowed_link_types.join(" "),
            ]
        })
        .chain(contract.link_schemas.iter().flat_map(|schema| {
            [
                schema.link_type.clone(),
                schema.source_object_type.clone(),
                schema.target_object_type.clone(),
                schema.cardinality.clone(),
                schema.description.clone(),
            ]
        }))
        .chain([contract.authority.clone()])
        .collect::<Vec<_>>();
    for term in &contract.forbidden_core_terms {
        if core_surface
            .iter()
            .any(|value| contains_forbidden_core_term(value, term))
        {
            errors.push(format!(
                "forbidden industry term `{term}` appears in Core object/link schema"
            ));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn object_schema(
    object_type: &str,
    description: &str,
    required_fields: &[&str],
    allowed_link_types: &[&str],
) -> CoreObjectSchemaDefinition {
    CoreObjectSchemaDefinition {
        object_type: object_type.to_string(),
        description: description.to_string(),
        required_fields: required_fields
            .iter()
            .map(|value| value.to_string())
            .collect(),
        allowed_link_types: allowed_link_types
            .iter()
            .map(|value| value.to_string())
            .collect(),
    }
}

fn link_schema(
    link_type: &str,
    source_object_type: &str,
    target_object_type: &str,
    cardinality: &str,
    description: &str,
) -> CoreLinkSchemaDefinition {
    CoreLinkSchemaDefinition {
        link_type: link_type.to_string(),
        source_object_type: source_object_type.to_string(),
        target_object_type: target_object_type.to_string(),
        cardinality: cardinality.to_string(),
        description: description.to_string(),
    }
}

fn contains_forbidden_core_term(value: &str, term: &str) -> bool {
    let normalized_term = normalized_compact(term);
    if normalized_term.is_empty() {
        return false;
    }

    if normalized_term.len() <= 2 {
        return tokenized(value)
            .iter()
            .any(|token| token == &normalized_term);
    }

    normalized_compact(value).contains(&normalized_term)
}

fn normalized_compact(value: &str) -> String {
    value
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .flat_map(|character| character.to_lowercase())
        .collect()
}

fn tokenized(value: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();

    for character in value.chars() {
        if character.is_ascii_alphanumeric() {
            current.extend(character.to_lowercase());
        } else if !current.is_empty() {
            tokens.push(std::mem::take(&mut current));
        }
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn core_object_link_schema_contract_validates() {
        let contract = core_object_link_schema_contract();
        validate_core_object_link_schema_contract(&contract).unwrap();
        assert_eq!(contract.object_schemas.len(), 11);
        assert_eq!(contract.link_schemas.len(), 12);
    }

    #[test]
    fn core_object_link_schema_rejects_missing_link_target() {
        let mut contract = core_object_link_schema_contract();
        contract.link_schemas[0].target_object_type = "MissingObject".to_string();

        let errors = validate_core_object_link_schema_contract(&contract).unwrap_err();
        assert!(errors.iter().any(|error| error.contains("missing target")));
    }

    #[test]
    fn core_object_link_schema_rejects_unknown_allowed_link() {
        let mut contract = core_object_link_schema_contract();
        contract.object_schemas[0]
            .allowed_link_types
            .push("unknownLink".to_string());

        let errors = validate_core_object_link_schema_contract(&contract).unwrap_err();
        assert!(errors.iter().any(|error| error.contains("unknown link")));
    }

    #[test]
    fn core_object_link_schema_rejects_industry_pollution() {
        let mut contract = core_object_link_schema_contract();
        contract.object_schemas[0]
            .description
            .push_str(" This is a GitHubIssue.");

        let errors = validate_core_object_link_schema_contract(&contract).unwrap_err();
        assert!(errors.iter().any(|error| error.contains("github-issue")));
    }
}
