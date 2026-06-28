use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const CORE_SKILL_REGISTRY_VERSION: &str = "agentflow-core-skill-registry.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreSkillDefinition {
    pub skill_id: String,
    pub owner_role: String,
    pub summary: String,
    pub allowed_actions: Vec<String>,
    pub allowed_tool_scopes: Vec<String>,
    pub allowed_connector_scopes: Vec<String>,
    pub expected_outputs: Vec<String>,
    pub required_evidence: Vec<String>,
    pub forbidden_scope: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreSkillRegistryContract {
    pub version: String,
    pub status: String,
    pub authority: String,
    pub reference_mapping_boundary: String,
    pub skills: Vec<CoreSkillDefinition>,
    pub forbidden_core_terms: Vec<String>,
}

pub fn core_skill_registry_contract() -> CoreSkillRegistryContract {
    CoreSkillRegistryContract {
        version: CORE_SKILL_REGISTRY_VERSION.to_string(),
        status: "active".to_string(),
        authority: "Core Skill Registry defines which role may perform which generic actions."
            .to_string(),
        reference_mapping_boundary:
            "Reference App mappings may translate skills into domain tasks or provider commands, but mappings are not Core authority."
                .to_string(),
        skills: vec![
            skill(
                "goal-intake-skill",
                "goal-agent",
                "Capture, normalize, and route incoming intent.",
                vec!["captureObject", "normalizeObject", "routeObject"],
                vec!["local-input-read", "local-context-read"],
                vec!["local-runtime"],
                vec!["IntentObject", "Route"],
                vec!["DecisionRef"],
                vec!["unlisted-action", "unowned-object-write", "external-authority-write"],
            ),
            skill(
                "spec-boundary-skill",
                "spec-agent",
                "Prepare accepted boundaries from routed intent.",
                vec!["acceptObject", "attachEvidence"],
                vec!["local-context-read", "local-evidence-write"],
                vec!["local-runtime"],
                vec!["SpecBundle", "EvidenceObject"],
                vec!["DecisionRef", "EvidenceRef"],
                vec!["unlisted-action", "direct-completion", "external-authority-write"],
            ),
            skill(
                "work-execution-skill",
                "work-agent",
                "Start controlled work and attach execution proof.",
                vec![
                    "startObject",
                    "attachEvidence",
                    "attachArtifact",
                    "submitForReview",
                    "blockObject",
                ],
                vec!["local-context-read", "local-artifact-write", "local-evidence-write"],
                vec!["local-runtime", "provider-session"],
                vec!["ExecutionObject", "EvidenceObject", "ArtifactObject"],
                vec!["EvidenceRef", "ArtifactRef"],
                vec!["unlisted-action", "boundary-change", "final-decision-write"],
            ),
            skill(
                "delivery-record-skill",
                "delivery-agent",
                "Prepare public output references after accepted work.",
                vec!["attachArtifact", "completeObject"],
                vec!["local-artifact-read", "local-output-write"],
                vec!["local-runtime"],
                vec!["ArtifactObject", "DecisionObject"],
                vec!["ArtifactRef", "DecisionRef"],
                vec!["unlisted-action", "unreviewed-output", "external-authority-write"],
            ),
            skill(
                "audit-review-skill",
                "audit-agent",
                "Review evidence and record follow-up decisions.",
                vec!["submitForReview", "blockObject", "cancelObject", "supersedeObject"],
                vec!["local-evidence-read", "local-decision-write"],
                vec!["local-runtime"],
                vec!["ReviewObject", "DecisionObject"],
                vec!["EvidenceRef", "DecisionRef"],
                vec!["unlisted-action", "owned-work-execution", "provider-session-write"],
            ),
            skill(
                "human-decision-skill",
                "human-owner",
                "Record final accept, cancel, or replacement decisions.",
                vec!["acceptObject", "completeObject", "cancelObject", "supersedeObject"],
                vec!["local-decision-write", "local-projection-read"],
                vec!["local-runtime"],
                vec!["DecisionObject"],
                vec!["DecisionRef"],
                vec!["unlisted-action", "provider-session-write", "unowned-object-write"],
            ),
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

pub fn validate_core_skill_registry_contract(
    contract: &CoreSkillRegistryContract,
) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    if contract.version != CORE_SKILL_REGISTRY_VERSION {
        errors.push(format!(
            "skill registry version must be `{}`",
            CORE_SKILL_REGISTRY_VERSION
        ));
    }
    if contract.status != "active" {
        errors.push("skill registry status must be active".to_string());
    }
    if !contract
        .reference_mapping_boundary
        .contains("not Core authority")
    {
        errors.push(
            "reference mapping boundary must say mappings are not Core authority".to_string(),
        );
    }

    let allowed_actions: BTreeSet<_> = [
        "captureObject",
        "normalizeObject",
        "routeObject",
        "acceptObject",
        "startObject",
        "attachEvidence",
        "attachArtifact",
        "submitForReview",
        "completeObject",
        "blockObject",
        "cancelObject",
        "supersedeObject",
    ]
    .into_iter()
    .collect();
    let skill_ids: BTreeSet<_> = contract
        .skills
        .iter()
        .map(|skill| skill.skill_id.as_str())
        .collect();
    for required_skill in [
        "goal-intake-skill",
        "spec-boundary-skill",
        "work-execution-skill",
        "delivery-record-skill",
        "audit-review-skill",
        "human-decision-skill",
    ] {
        if !skill_ids.contains(required_skill) {
            errors.push(format!("missing Core skill `{required_skill}`"));
        }
    }

    for skill in &contract.skills {
        if skill.skill_id.trim().is_empty() {
            errors.push("skill id must not be empty".to_string());
        }
        if skill.owner_role.trim().is_empty() {
            errors.push(format!(
                "skill `{}` owner role must not be empty",
                skill.skill_id
            ));
        }
        for action in &skill.allowed_actions {
            if !allowed_actions.contains(action.as_str()) {
                errors.push(format!(
                    "skill `{}` allows unknown action `{action}`",
                    skill.skill_id
                ));
            }
        }
        if skill.allowed_actions.is_empty() {
            errors.push(format!(
                "skill `{}` must allow at least one action",
                skill.skill_id
            ));
        }
        if skill.expected_outputs.is_empty() {
            errors.push(format!(
                "skill `{}` must declare expected outputs",
                skill.skill_id
            ));
        }
        if skill.required_evidence.is_empty() {
            errors.push(format!(
                "skill `{}` must declare required evidence",
                skill.skill_id
            ));
        }
        if skill.forbidden_scope.is_empty() {
            errors.push(format!(
                "skill `{}` must declare forbidden scope",
                skill.skill_id
            ));
        }
    }

    let core_surface = contract
        .skills
        .iter()
        .flat_map(|skill| {
            [
                skill.skill_id.clone(),
                skill.owner_role.clone(),
                skill.summary.clone(),
                skill.allowed_actions.join(" "),
                skill.allowed_tool_scopes.join(" "),
                skill.allowed_connector_scopes.join(" "),
                skill.expected_outputs.join(" "),
                skill.required_evidence.join(" "),
                skill.forbidden_scope.join(" "),
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
                "forbidden industry term `{term}` appears in Core skill registry"
            ));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

#[allow(clippy::too_many_arguments)]
fn skill(
    skill_id: &str,
    owner_role: &str,
    summary: &str,
    allowed_actions: Vec<&str>,
    allowed_tool_scopes: Vec<&str>,
    allowed_connector_scopes: Vec<&str>,
    expected_outputs: Vec<&str>,
    required_evidence: Vec<&str>,
    forbidden_scope: Vec<&str>,
) -> CoreSkillDefinition {
    CoreSkillDefinition {
        skill_id: skill_id.to_string(),
        owner_role: owner_role.to_string(),
        summary: summary.to_string(),
        allowed_actions: allowed_actions.into_iter().map(str::to_string).collect(),
        allowed_tool_scopes: allowed_tool_scopes
            .into_iter()
            .map(str::to_string)
            .collect(),
        allowed_connector_scopes: allowed_connector_scopes
            .into_iter()
            .map(str::to_string)
            .collect(),
        expected_outputs: expected_outputs.into_iter().map(str::to_string).collect(),
        required_evidence: required_evidence.into_iter().map(str::to_string).collect(),
        forbidden_scope: forbidden_scope.into_iter().map(str::to_string).collect(),
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
    fn core_skill_registry_contract_validates() {
        let contract = core_skill_registry_contract();
        validate_core_skill_registry_contract(&contract).unwrap();
        assert_eq!(contract.skills.len(), 6);
    }

    #[test]
    fn core_skill_registry_rejects_unknown_action() {
        let mut contract = core_skill_registry_contract();
        contract.skills[0]
            .allowed_actions
            .push("unknownAction".to_string());

        let errors = validate_core_skill_registry_contract(&contract).unwrap_err();
        assert!(errors.iter().any(|error| error.contains("unknown action")));
    }

    #[test]
    fn core_skill_registry_rejects_missing_forbidden_scope() {
        let mut contract = core_skill_registry_contract();
        contract.skills[0].forbidden_scope.clear();

        let errors = validate_core_skill_registry_contract(&contract).unwrap_err();
        assert!(errors.iter().any(|error| error.contains("forbidden scope")));
    }

    #[test]
    fn core_skill_registry_rejects_industry_pollution() {
        let mut contract = core_skill_registry_contract();
        contract.skills[0]
            .summary
            .push_str(" This must not become a GitHubIssue.");

        let errors = validate_core_skill_registry_contract(&contract).unwrap_err();
        assert!(errors.iter().any(|error| error.contains("github-issue")));
    }
}
