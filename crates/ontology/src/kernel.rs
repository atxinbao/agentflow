use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const CORE_ONTOLOGY_KERNEL_VERSION: &str = "agentflow-core-ontology-kernel.v1";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CoreOntologyElement {
    Object,
    Link,
    Action,
    State,
    Skill,
    Evidence,
    Decision,
    Artifact,
    Route,
    SpecBundle,
    Projection,
}

impl CoreOntologyElement {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Object => "object",
            Self::Link => "link",
            Self::Action => "action",
            Self::State => "state",
            Self::Skill => "skill",
            Self::Evidence => "evidence",
            Self::Decision => "decision",
            Self::Artifact => "artifact",
            Self::Route => "route",
            Self::SpecBundle => "spec-bundle",
            Self::Projection => "projection",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreOntologyElementDefinition {
    pub element: CoreOntologyElement,
    pub description: String,
    pub authority_boundary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreOntologyKernelContract {
    pub version: String,
    pub status: String,
    pub authority: String,
    pub reference_mapping_boundary: String,
    pub elements: Vec<CoreOntologyElementDefinition>,
    pub forbidden_core_terms: Vec<String>,
}

pub fn core_ontology_kernel_contract() -> CoreOntologyKernelContract {
    CoreOntologyKernelContract {
        version: CORE_ONTOLOGY_KERNEL_VERSION.to_string(),
        status: "active".to_string(),
        authority: "Core defines industry-neutral ontology primitives only.".to_string(),
        reference_mapping_boundary:
            "Reference App mappings may translate Core primitives into domain vocabulary, but mappings are not Core authority."
                .to_string(),
        elements: vec![
            element(CoreOntologyElement::Object, "A typed unit of work, knowledge, output, or review state that can be referenced by actions and projections."),
            element(CoreOntologyElement::Link, "A typed relation between two objects with declared source, target, and cardinality."),
            element(CoreOntologyElement::Action, "A typed operation proposal that may read or write objects through an authorized role."),
            element(CoreOntologyElement::State, "A typed lifecycle value attached to an object or action execution."),
            element(CoreOntologyElement::Skill, "A declared capability package that constrains what an Agent role can do."),
            element(CoreOntologyElement::Evidence, "A proof reference used to support a state transition, decision, or completion claim."),
            element(CoreOntologyElement::Decision, "A recorded judgment or confirmation that changes authority boundaries."),
            element(CoreOntologyElement::Artifact, "A durable output reference produced by a workflow step."),
            element(CoreOntologyElement::Route, "A typed next-step path selected by workflow policy."),
            element(CoreOntologyElement::SpecBundle, "A confirmed contract bundle that can materialize runtime work."),
            element(CoreOntologyElement::Projection, "A read model derived from events and contracts for UI or API consumption."),
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

pub fn validate_core_ontology_kernel_contract(
    contract: &CoreOntologyKernelContract,
) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    if contract.version != CORE_ONTOLOGY_KERNEL_VERSION {
        errors.push(format!(
            "kernel version must be `{}`",
            CORE_ONTOLOGY_KERNEL_VERSION
        ));
    }
    if contract.status != "active" {
        errors.push("kernel status must be active".to_string());
    }
    if !contract
        .reference_mapping_boundary
        .contains("not Core authority")
    {
        errors.push(
            "reference mapping boundary must say mappings are not Core authority".to_string(),
        );
    }

    let required = [
        CoreOntologyElement::Object,
        CoreOntologyElement::Link,
        CoreOntologyElement::Action,
        CoreOntologyElement::State,
        CoreOntologyElement::Skill,
        CoreOntologyElement::Evidence,
        CoreOntologyElement::Decision,
        CoreOntologyElement::Artifact,
        CoreOntologyElement::Route,
        CoreOntologyElement::SpecBundle,
        CoreOntologyElement::Projection,
    ];
    let actual: BTreeSet<_> = contract
        .elements
        .iter()
        .map(|definition| definition.element.clone())
        .collect();
    for required_element in required {
        if !actual.contains(&required_element) {
            errors.push(format!(
                "missing Core ontology element `{}`",
                required_element.as_str()
            ));
        }
    }

    let core_surface = contract
        .elements
        .iter()
        .flat_map(|definition| {
            [
                definition.element.as_str().to_string(),
                definition.description.clone(),
                definition.authority_boundary.clone(),
            ]
        })
        .chain([
            contract.authority.clone(),
            contract.reference_mapping_boundary.clone(),
        ])
        .collect::<Vec<_>>();
    for term in &contract.forbidden_core_terms {
        if core_surface
            .iter()
            .any(|value| contains_forbidden_core_term(value, term))
        {
            errors.push(format!(
                "forbidden industry term `{term}` appears in Core ontology authority"
            ));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn element(element: CoreOntologyElement, description: &str) -> CoreOntologyElementDefinition {
    CoreOntologyElementDefinition {
        element,
        description: description.to_string(),
        authority_boundary: "core-authority".to_string(),
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
    fn core_ontology_kernel_contract_validates() {
        let contract = core_ontology_kernel_contract();
        validate_core_ontology_kernel_contract(&contract).unwrap();
        assert_eq!(contract.elements.len(), 11);
    }

    #[test]
    fn core_ontology_kernel_rejects_missing_required_element() {
        let mut contract = core_ontology_kernel_contract();
        contract
            .elements
            .retain(|definition| definition.element != CoreOntologyElement::Projection);

        let errors = validate_core_ontology_kernel_contract(&contract).unwrap_err();
        assert!(errors.iter().any(|error| error.contains("projection")));
    }

    #[test]
    fn core_ontology_kernel_rejects_software_dev_pollution() {
        let mut contract = core_ontology_kernel_contract();
        contract.elements[0]
            .description
            .push_str(" Do not turn this into a GitHubIssue.");

        let errors = validate_core_ontology_kernel_contract(&contract).unwrap_err();
        assert!(errors.iter().any(|error| error.contains("github-issue")));
    }
}
