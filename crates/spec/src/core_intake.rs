//! Core 4-D Spec Intake contract.
//!
//! This module defines the industry-neutral intake kernel used before any
//! Reference App maps the result into domain-specific objects. It intentionally
//! keeps Software Dev terms out of Core authority; those terms may appear only
//! in reference mapping fixtures.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const CORE_4D_SPEC_INTAKE_VERSION: &str = "agentflow-core-4d-spec-intake.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Core4DPhase {
    Deconstruct,
    Diagnose,
    Develop,
    Deliver,
}

impl Core4DPhase {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Deconstruct => "deconstruct",
            Self::Diagnose => "diagnose",
            Self::Develop => "develop",
            Self::Deliver => "deliver",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CoreIntakeRoute {
    Clarify,
    Research,
    Define,
    Plan,
    Task,
    Decide,
    Deliver,
    Evolve,
}

impl CoreIntakeRoute {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Clarify => "clarify",
            Self::Research => "research",
            Self::Define => "define",
            Self::Plan => "plan",
            Self::Task => "task",
            Self::Decide => "decide",
            Self::Deliver => "deliver",
            Self::Evolve => "evolve",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CoreSpecBundleSlice {
    Intent,
    Domain,
    Goal,
    Plan,
    Task,
    Decision,
    Output,
    Feedback,
}

impl CoreSpecBundleSlice {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Intent => "intent",
            Self::Domain => "domain",
            Self::Goal => "goal",
            Self::Plan => "plan",
            Self::Task => "task",
            Self::Decision => "decision",
            Self::Output => "output",
            Self::Feedback => "feedback",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CoreArtifactBoundary {
    Draft,
    Preview,
    Confirmed,
    Materialized,
}

impl CoreArtifactBoundary {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Preview => "preview",
            Self::Confirmed => "confirmed",
            Self::Materialized => "materialized",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Core4DStageContract {
    pub phase: Core4DPhase,
    pub input_refs: Vec<String>,
    pub output_refs: Vec<String>,
    pub allowed_routes: Vec<CoreIntakeRoute>,
    pub authority_boundary: CoreArtifactBoundary,
    pub forbidden_behaviors: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreGapRoute {
    pub route: CoreIntakeRoute,
    pub trigger: String,
    pub output_boundary: CoreArtifactBoundary,
    pub next_step: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IndustryMappingFixture {
    pub industry: String,
    pub core_slice: CoreSpecBundleSlice,
    pub mapped_object: String,
    pub mapped_action: String,
    pub mapped_evidence: String,
    pub mapped_decision: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Core4DSpecIntakeContract {
    pub version: String,
    pub phases: Vec<Core4DStageContract>,
    pub routes: Vec<CoreGapRoute>,
    pub slices: Vec<CoreSpecBundleSlice>,
    pub boundaries: Vec<CoreArtifactBoundary>,
    pub reference_mappings: Vec<IndustryMappingFixture>,
    pub forbidden_core_terms: Vec<String>,
}

pub fn core_4d_spec_intake_contract() -> Core4DSpecIntakeContract {
    Core4DSpecIntakeContract {
        version: CORE_4D_SPEC_INTAKE_VERSION.to_string(),
        phases: vec![
            Core4DStageContract {
                phase: Core4DPhase::Deconstruct,
                input_refs: vec![
                    "raw-human-request".to_string(),
                    "source-references".to_string(),
                ],
                output_refs: vec!["intent-packet".to_string()],
                allowed_routes: vec![CoreIntakeRoute::Clarify, CoreIntakeRoute::Research],
                authority_boundary: CoreArtifactBoundary::Draft,
                forbidden_behaviors: vec![
                    "materialize-authority".to_string(),
                    "dispatch-runtime-action".to_string(),
                ],
            },
            Core4DStageContract {
                phase: Core4DPhase::Diagnose,
                input_refs: vec!["intent-packet".to_string(), "context-artifacts".to_string()],
                output_refs: vec!["gap-model".to_string(), "route-policy".to_string()],
                allowed_routes: vec![
                    CoreIntakeRoute::Clarify,
                    CoreIntakeRoute::Research,
                    CoreIntakeRoute::Define,
                    CoreIntakeRoute::Plan,
                    CoreIntakeRoute::Task,
                    CoreIntakeRoute::Decide,
                    CoreIntakeRoute::Deliver,
                    CoreIntakeRoute::Evolve,
                ],
                authority_boundary: CoreArtifactBoundary::Draft,
                forbidden_behaviors: vec!["treat-reference-app-as-core-authority".to_string()],
            },
            Core4DStageContract {
                phase: Core4DPhase::Develop,
                input_refs: vec!["route-policy".to_string(), "draft-spec-bundle".to_string()],
                output_refs: vec![
                    "preview-artifact".to_string(),
                    "confirmation-request".to_string(),
                ],
                allowed_routes: vec![
                    CoreIntakeRoute::Define,
                    CoreIntakeRoute::Plan,
                    CoreIntakeRoute::Task,
                    CoreIntakeRoute::Decide,
                    CoreIntakeRoute::Deliver,
                    CoreIntakeRoute::Evolve,
                ],
                authority_boundary: CoreArtifactBoundary::Preview,
                forbidden_behaviors: vec!["write-confirmed-spec-without-confirmation".to_string()],
            },
            Core4DStageContract {
                phase: Core4DPhase::Deliver,
                input_refs: vec![
                    "confirmed-preview".to_string(),
                    "confirmation-record".to_string(),
                ],
                output_refs: vec![
                    "confirmed-spec-bundle".to_string(),
                    "materialized-runtime-contract".to_string(),
                ],
                allowed_routes: vec![CoreIntakeRoute::Deliver, CoreIntakeRoute::Evolve],
                authority_boundary: CoreArtifactBoundary::Materialized,
                forbidden_behaviors: vec!["skip-confirmed-boundary".to_string()],
            },
        ],
        routes: vec![
            CoreGapRoute {
                route: CoreIntakeRoute::Clarify,
                trigger: "human-decision-gap".to_string(),
                output_boundary: CoreArtifactBoundary::Draft,
                next_step: "ask-bounded-question".to_string(),
            },
            CoreGapRoute {
                route: CoreIntakeRoute::Research,
                trigger: "fact-gap".to_string(),
                output_boundary: CoreArtifactBoundary::Draft,
                next_step: "collect-evidence".to_string(),
            },
            CoreGapRoute {
                route: CoreIntakeRoute::Define,
                trigger: "goal-gap".to_string(),
                output_boundary: CoreArtifactBoundary::Preview,
                next_step: "draft-goal-slice".to_string(),
            },
            CoreGapRoute {
                route: CoreIntakeRoute::Plan,
                trigger: "sequencing-gap".to_string(),
                output_boundary: CoreArtifactBoundary::Preview,
                next_step: "draft-plan-slice".to_string(),
            },
            CoreGapRoute {
                route: CoreIntakeRoute::Task,
                trigger: "actionable-work-gap".to_string(),
                output_boundary: CoreArtifactBoundary::Preview,
                next_step: "draft-task-slice".to_string(),
            },
            CoreGapRoute {
                route: CoreIntakeRoute::Decide,
                trigger: "acceptance-gap".to_string(),
                output_boundary: CoreArtifactBoundary::Preview,
                next_step: "draft-decision-slice".to_string(),
            },
            CoreGapRoute {
                route: CoreIntakeRoute::Deliver,
                trigger: "output-gap".to_string(),
                output_boundary: CoreArtifactBoundary::Confirmed,
                next_step: "prepare-output-slice".to_string(),
            },
            CoreGapRoute {
                route: CoreIntakeRoute::Evolve,
                trigger: "feedback-gap".to_string(),
                output_boundary: CoreArtifactBoundary::Preview,
                next_step: "draft-feedback-slice".to_string(),
            },
        ],
        slices: vec![
            CoreSpecBundleSlice::Intent,
            CoreSpecBundleSlice::Domain,
            CoreSpecBundleSlice::Goal,
            CoreSpecBundleSlice::Plan,
            CoreSpecBundleSlice::Task,
            CoreSpecBundleSlice::Decision,
            CoreSpecBundleSlice::Output,
            CoreSpecBundleSlice::Feedback,
        ],
        boundaries: vec![
            CoreArtifactBoundary::Draft,
            CoreArtifactBoundary::Preview,
            CoreArtifactBoundary::Confirmed,
            CoreArtifactBoundary::Materialized,
        ],
        reference_mappings: vec![
            IndustryMappingFixture {
                industry: "software-dev".to_string(),
                core_slice: CoreSpecBundleSlice::Task,
                mapped_object: "issue".to_string(),
                mapped_action: "implement-change".to_string(),
                mapped_evidence: "test-log-or-build-log".to_string(),
                mapped_decision: "accepted-or-needs-fix".to_string(),
            },
            IndustryMappingFixture {
                industry: "ui-design".to_string(),
                core_slice: CoreSpecBundleSlice::Output,
                mapped_object: "screen-or-component".to_string(),
                mapped_action: "produce-design-preview".to_string(),
                mapped_evidence: "visual-artifact".to_string(),
                mapped_decision: "approved-or-revise".to_string(),
            },
            IndustryMappingFixture {
                industry: "video-production".to_string(),
                core_slice: CoreSpecBundleSlice::Plan,
                mapped_object: "shot-list-or-scene-plan".to_string(),
                mapped_action: "prepare-production-plan".to_string(),
                mapped_evidence: "storyboard-or-render-preview".to_string(),
                mapped_decision: "ready-or-recut".to_string(),
            },
        ],
        forbidden_core_terms: vec![
            "bug".to_string(),
            "feature".to_string(),
            "pr".to_string(),
            "release".to_string(),
            "patch".to_string(),
            "test-log".to_string(),
            "repository".to_string(),
            "github-issue".to_string(),
        ],
    }
}

pub fn validate_core_4d_spec_intake_contract(
    contract: &Core4DSpecIntakeContract,
) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    let expected_phases = [
        Core4DPhase::Deconstruct,
        Core4DPhase::Diagnose,
        Core4DPhase::Develop,
        Core4DPhase::Deliver,
    ];
    let actual_phases: Vec<_> = contract
        .phases
        .iter()
        .map(|stage| stage.phase.clone())
        .collect();
    if actual_phases != expected_phases {
        errors.push("4-D phases must be deconstruct -> diagnose -> develop -> deliver".to_string());
    }

    let expected_slices = [
        CoreSpecBundleSlice::Intent,
        CoreSpecBundleSlice::Domain,
        CoreSpecBundleSlice::Goal,
        CoreSpecBundleSlice::Plan,
        CoreSpecBundleSlice::Task,
        CoreSpecBundleSlice::Decision,
        CoreSpecBundleSlice::Output,
        CoreSpecBundleSlice::Feedback,
    ];
    let actual_slices: Vec<_> = contract.slices.clone();
    if actual_slices != expected_slices {
        errors.push(
            "Core Spec Bundle slices must be intent/domain/goal/plan/task/decision/output/feedback"
                .to_string(),
        );
    }

    let route_set: BTreeSet<_> = contract
        .routes
        .iter()
        .map(|route| route.route.as_str())
        .collect();
    for route in [
        "clarify", "research", "define", "plan", "task", "decide", "deliver", "evolve",
    ] {
        if !route_set.contains(route) {
            errors.push(format!("missing core route `{route}`"));
        }
    }

    let boundary_set: BTreeSet<_> = contract
        .boundaries
        .iter()
        .map(|boundary| boundary.as_str())
        .collect();
    for boundary in ["draft", "preview", "confirmed", "materialized"] {
        if !boundary_set.contains(boundary) {
            errors.push(format!("missing authority boundary `{boundary}`"));
        }
    }

    let core_surface = [
        contract
            .phases
            .iter()
            .flat_map(|stage| {
                [
                    vec![stage.phase.as_str().to_string()],
                    stage.input_refs.clone(),
                    stage.output_refs.clone(),
                    stage
                        .allowed_routes
                        .iter()
                        .map(|route| route.as_str().to_string())
                        .collect(),
                    stage.forbidden_behaviors.clone(),
                ]
                .concat()
            })
            .collect::<Vec<_>>(),
        contract
            .routes
            .iter()
            .flat_map(|route| {
                vec![
                    route.route.as_str().to_string(),
                    route.trigger.clone(),
                    route.next_step.clone(),
                ]
            })
            .collect::<Vec<_>>(),
        contract
            .slices
            .iter()
            .map(|slice| slice.as_str().to_string())
            .collect::<Vec<_>>(),
    ]
    .concat();

    for term in &contract.forbidden_core_terms {
        if core_surface.iter().any(|value| value == term) {
            errors.push(format!(
                "forbidden industry term `{term}` appears in Core authority"
            ));
        }
    }

    let industries: BTreeSet<_> = contract
        .reference_mappings
        .iter()
        .map(|mapping| mapping.industry.as_str())
        .collect();
    for industry in ["software-dev", "ui-design", "video-production"] {
        if !industries.contains(industry) {
            errors.push(format!("missing reference mapping `{industry}`"));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn core_4d_contract_is_generic_and_valid() {
        let contract = core_4d_spec_intake_contract();
        validate_core_4d_spec_intake_contract(&contract).unwrap();
        assert_eq!(contract.version, CORE_4D_SPEC_INTAKE_VERSION);
        assert_eq!(contract.phases.len(), 4);
        assert_eq!(contract.slices.len(), 8);
        assert!(contract
            .reference_mappings
            .iter()
            .any(|mapping| mapping.industry == "software-dev"));
    }

    #[test]
    fn validation_rejects_core_authority_with_industry_term() {
        let mut contract = core_4d_spec_intake_contract();
        contract.routes[0].trigger = "github-issue".to_string();
        let errors = validate_core_4d_spec_intake_contract(&contract).unwrap_err();
        assert!(errors.iter().any(|error| error.contains("github-issue")));
    }
}
