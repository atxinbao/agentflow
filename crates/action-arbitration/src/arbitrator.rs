use std::collections::BTreeSet;

use agentflow_action_contract::{
    validate_action_proposal, ActionContract, ActionPreconditionKind, ActionProposal,
    ActionProposalValidationStatus,
};

use crate::locks::{check_object_lock as check_lock, default_lock_kind_for_object};
use crate::model::{
    AcceptedAction, ArbitrationContext, ArbitrationDecision, ArbitrationDecisionStatus,
    ArbitrationRequest, HumanDecisionRequest, HumanDecisionResponseKind, ObjectLock,
    ObjectLockPlan,
};
use crate::reasons::{RejectionReason, RejectionReasonCode};
use crate::report::RejectionExplanation;

#[derive(Debug, Default, Clone, Copy)]
pub struct ActionArbitrator;

impl ActionArbitrator {
    pub fn arbitrate_action(
        &self,
        request: &ArbitrationRequest,
        context: &ArbitrationContext,
    ) -> ArbitrationDecision {
        arbitrate_action(request, context)
    }

    pub fn explain_rejection(&self, decision: &ArbitrationDecision) -> RejectionExplanation {
        RejectionExplanation::from_decision(decision)
    }
}

pub fn arbitrate_action(
    request: &ArbitrationRequest,
    context: &ArbitrationContext,
) -> ArbitrationDecision {
    let proposal_report = validate_action_proposal(
        &request.proposal,
        &context.action_contract_registry,
        &context.ontology_registry,
    );
    if proposal_report.status != ActionProposalValidationStatus::Valid {
        return ArbitrationDecision::rejected(
            request,
            map_proposal_validation_reasons(&proposal_report),
        );
    }

    let proposal = proposal_report
        .normalized_proposal
        .expect("valid proposal should keep normalized copy");
    let contract = context
        .action_contract_registry
        .get_action_contract(&proposal.action_type, &proposal.contract_version)
        .expect("validated proposal must have contract");
    let object_type = contract
        .target_object_type
        .as_deref()
        .or(contract.creates_object_type.as_deref());

    let role_decision = context.role_policy_registry.can_role_propose_action(
        &proposal.actor_role,
        &proposal.action_type,
        object_type,
    );
    if !role_decision.allowed {
        return ArbitrationDecision::rejected(
            request,
            vec![map_role_rejection(
                role_decision.reason.as_str(),
                object_type,
            )],
        );
    }

    let human_decision_required = requires_human_decision(contract, &proposal, context)
        || role_decision.requires_human_approval;
    if human_decision_required {
        return ArbitrationDecision {
            decision_id: format!("decision-{}", request.request_id),
            request_id: request.request_id.clone(),
            proposal_id: request.proposal.proposal_id.clone(),
            status: ArbitrationDecisionStatus::HumanDecisionRequired,
            accepted_action: None,
            rejected_reasons: Vec::new(),
            required_human_decision: Some(HumanDecisionRequest {
                decision_kind: HumanDecisionResponseKind::ApprovalRequired,
                target_object_ref: proposal.target_object_ref.clone(),
                question: format!(
                    "action `{}` requires explicit human decision before Runtime can accept it",
                    proposal.action_type
                ),
                allowed_responses: vec!["approve".into(), "reject".into()],
                required_evidence_type: "humanConfirmation".into(),
            }),
            lock_plan: ObjectLockPlan::default(),
            would_emit_events: Vec::new(),
            created_at: request.requested_at.clone(),
        };
    }

    if let Some(reasons) = validate_dependency_preconditions(contract, context) {
        return ArbitrationDecision::rejected(request, reasons);
    }

    if let Some(reasons) = validate_evidence(contract, &proposal, context) {
        return ArbitrationDecision::rejected(request, reasons);
    }

    let current_state = match (
        &proposal.target_object_ref,
        contract.target_object_type.as_deref(),
    ) {
        (Some(target), Some(_)) => match context.current_state_for(target) {
            Some(state) => Some(state.to_string()),
            None => {
                return ArbitrationDecision::rejected(
                    request,
                    vec![RejectionReason::new(
                        RejectionReasonCode::UnknownTargetObject,
                        format!(
                            "target object `{}`:`{}` does not have a current state fact",
                            target.object_type, target.id
                        ),
                        None,
                    )],
                );
            }
        },
        _ => None,
    };

    let state_object_type = object_type.expect("validated action must resolve object type");
    let transition = context.state_machine_registry.is_transition_defined(
        state_object_type,
        current_state.as_deref(),
        &proposal.action_type,
    );
    if !transition.allowed {
        return ArbitrationDecision::rejected(
            request,
            vec![RejectionReason::new(
                RejectionReasonCode::InvalidObjectState,
                transition.reason,
                None,
            )],
        );
    }

    if !transition.required_evidence.is_empty() {
        let missing = transition
            .required_evidence
            .iter()
            .filter(|required| {
                context.evidence_count_by_type(&proposal.evidence_refs, required) == 0
            })
            .cloned()
            .collect::<Vec<_>>();
        if !missing.is_empty() {
            return ArbitrationDecision::rejected(
                request,
                vec![RejectionReason::new(
                    RejectionReasonCode::MissingRequiredEvidence,
                    format!(
                        "transition for action `{}` still misses evidence types: {}",
                        proposal.action_type,
                        missing.join(", ")
                    ),
                    None,
                )],
            );
        }
    }

    let lock_plan = proposal
        .target_object_ref
        .as_ref()
        .map(|target| {
            let lock_kind = default_lock_kind_for_object(target.object_type.as_str());
            let decision = check_lock(target, lock_kind, context);
            if !decision.available {
                return Err(ArbitrationDecision::rejected(
                    request,
                    vec![RejectionReason::new(
                        RejectionReasonCode::ObjectLockUnavailable,
                        decision
                            .reason
                            .unwrap_or_else(|| "object lock unavailable".into()),
                        decision.blocking_lock.map(|lock| {
                            format!(
                                "{:?}:{}:{}",
                                lock.lock_kind, lock.object_type, lock.object_id
                            )
                        }),
                    )],
                ));
            }
            Ok(ObjectLockPlan {
                acquire: vec![ObjectLock {
                    lock_id: format!("lock-{}", proposal.proposal_id),
                    object_type: target.object_type.clone(),
                    object_id: target.id.clone(),
                    lock_kind,
                    owner_proposal_id: proposal.proposal_id.clone(),
                    owner_role: proposal.actor_role.clone(),
                    expires_at: None,
                    reason: format!("accepted action `{}`", proposal.action_type),
                }],
                release: Vec::new(),
            })
        })
        .transpose();

    let lock_plan = match lock_plan {
        Ok(plan) => plan.unwrap_or_default(),
        Err(rejected) => return rejected,
    };

    let accepted_action =
        build_accepted_action(&proposal, contract, &transition, request, lock_plan.clone());

    ArbitrationDecision {
        decision_id: format!("decision-{}", request.request_id),
        request_id: request.request_id.clone(),
        proposal_id: proposal.proposal_id.clone(),
        status: ArbitrationDecisionStatus::Accepted,
        accepted_action: Some(accepted_action),
        rejected_reasons: Vec::new(),
        required_human_decision: None,
        lock_plan: lock_plan.clone(),
        would_emit_events: collect_expected_events(contract, &transition.emitted_events),
        created_at: request.requested_at.clone(),
    }
}

pub fn check_object_lock(
    target: &agentflow_action_contract::ActionRef,
    requested_kind: crate::model::ObjectLockKind,
    context: &ArbitrationContext,
) -> crate::locks::LockDecision {
    crate::locks::check_object_lock(target, requested_kind, context)
}

pub fn build_accepted_action(
    proposal: &ActionProposal,
    contract: &ActionContract,
    transition: &agentflow_object_state::TransitionDecision,
    request: &ArbitrationRequest,
    lock_plan: ObjectLockPlan,
) -> AcceptedAction {
    AcceptedAction {
        accepted_action_id: format!("accepted-{}", proposal.proposal_id),
        proposal_id: proposal.proposal_id.clone(),
        action_type: proposal.action_type.clone(),
        actor_role: proposal.actor_role.clone(),
        target_object_ref: proposal.target_object_ref.clone(),
        from_state: transition.resolved_state.clone(),
        to_state: transition.next_state.clone(),
        evidence_refs: proposal.evidence_refs.clone(),
        artifact_refs: proposal.artifact_refs.clone(),
        expected_events: collect_expected_events(contract, &transition.emitted_events),
        lock_plan,
        definition_versions: request.definition_versions.clone(),
    }
}

fn map_proposal_validation_reasons(
    report: &agentflow_action_contract::ActionProposalValidationReport,
) -> Vec<RejectionReason> {
    if report.errors.is_empty() {
        return vec![RejectionReason::new(
            RejectionReasonCode::InvalidActionProposal,
            "proposal validation failed without detailed error",
            Some(format!("{:?}", report.status)),
        )];
    }

    report
        .errors
        .iter()
        .map(|error| {
            let code = match error.code.as_str() {
                "unknownActionType" => RejectionReasonCode::UnknownActionType,
                "unknownContractVersion" | "ontologyVersionMismatch" | "contractRetired" => {
                    RejectionReasonCode::DefinitionVersionMismatch
                }
                _ => RejectionReasonCode::InvalidActionProposal,
            };
            RejectionReason::new(code, error.message.clone(), error.path.clone())
        })
        .collect()
}

fn map_role_rejection(reason: &str, object_type: Option<&str>) -> RejectionReason {
    let (code, message) = match reason {
        "unknownRole" => (
            RejectionReasonCode::UnknownActorRole,
            "actor role is not recognized by role policy".to_string(),
        ),
        "objectScopeMissing" | "objectTypeMismatch" => (
            RejectionReasonCode::RoleCannotAccessObject,
            format!(
                "actor role cannot access target object `{}`",
                object_type.unwrap_or("unknown")
            ),
        ),
        _ => (
            RejectionReasonCode::RoleCannotExecuteAction,
            "actor role cannot propose this action".to_string(),
        ),
    };
    RejectionReason::new(code, message, Some(reason.to_string()))
}

fn requires_human_decision(
    contract: &ActionContract,
    proposal: &ActionProposal,
    context: &ArbitrationContext,
) -> bool {
    let human_confirmation_present =
        context.evidence_count_by_type(&proposal.evidence_refs, "humanConfirmation") > 0;
    let human_confirmation_required = contract
        .required_evidence
        .iter()
        .any(|required| required.evidence_type == "humanConfirmation")
        || contract.approval_hint.human_approval_required
        || contract
            .preconditions
            .iter()
            .any(|precondition| precondition.kind == ActionPreconditionKind::HumanDecisionExists);

    human_confirmation_required && !human_confirmation_present
}

fn validate_dependency_preconditions(
    contract: &ActionContract,
    context: &ArbitrationContext,
) -> Option<Vec<RejectionReason>> {
    let unmet = contract
        .preconditions
        .iter()
        .filter(|precondition| precondition.kind == ActionPreconditionKind::DependencySatisfied)
        .map(|precondition| {
            precondition
                .expression
                .clone()
                .unwrap_or_else(|| precondition.id.clone())
        })
        .filter(|dependency_key| !context.dependency_satisfied(dependency_key))
        .collect::<Vec<_>>();

    if unmet.is_empty() {
        None
    } else {
        Some(vec![RejectionReason::new(
            RejectionReasonCode::DependencyNotSatisfied,
            format!("unmet dependencies: {}", unmet.join(", ")),
            None,
        )])
    }
}

fn validate_evidence(
    contract: &ActionContract,
    proposal: &ActionProposal,
    context: &ArbitrationContext,
) -> Option<Vec<RejectionReason>> {
    let missing_refs = proposal
        .evidence_refs
        .iter()
        .filter(|evidence_ref| context.evidence_by_ref(evidence_ref).is_none())
        .cloned()
        .collect::<Vec<_>>();
    if !missing_refs.is_empty() {
        return Some(vec![RejectionReason::new(
            RejectionReasonCode::MissingRequiredEvidence,
            format!("unknown evidence refs: {}", missing_refs.join(", ")),
            None,
        )]);
    }

    let missing_types = contract
        .required_evidence
        .iter()
        .filter(|required| {
            if required.evidence_type == "humanConfirmation" {
                return false;
            }
            let count =
                context.evidence_count_by_type(&proposal.evidence_refs, &required.evidence_type);
            count < required.min_count.max(usize::from(required.required))
        })
        .map(|required| {
            format!(
                "{} ({:?})",
                required.evidence_type, required.accepted_ref_kind
            )
        })
        .collect::<Vec<_>>();

    if missing_types.is_empty() {
        None
    } else {
        Some(vec![RejectionReason::new(
            RejectionReasonCode::MissingRequiredEvidence,
            format!(
                "missing required evidence types: {}",
                missing_types.join(", ")
            ),
            None,
        )])
    }
}

fn collect_expected_events(contract: &ActionContract, transition_events: &[String]) -> Vec<String> {
    let mut expected = BTreeSet::new();
    for event in &contract.expected_events {
        expected.insert(event.event_type.clone());
    }
    for event in transition_events {
        expected.insert(event.clone());
    }
    expected.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use agentflow_action_contract::{
        core_action_contract_registry, ActionProposal, ActionRef, ActionSourceSurface,
    };
    use agentflow_object_state::core_object_state_registry;
    use agentflow_ontology::core_ontology_registry;
    use agentflow_role_policy::core_role_policy_registry;

    use crate::model::{
        ArbitrationContext, ArbitrationDecisionStatus, ArbitrationRequest, DefinitionVersions,
        DependencyFact, EvidenceFact, ObjectLock, ObjectLockKind, StateFact,
    };
    use crate::reasons::RejectionReasonCode;

    use super::arbitrate_action;

    #[test]
    fn valid_proposal_returns_accepted() {
        let mut context = core_context();
        context.insert_state(StateFact {
            object_type: "Issue".into(),
            object_id: "ISS-1".into(),
            state_id: "reviewReady".into(),
        });
        context.insert_evidence(evidence("artifact-1", "implementationSummary"));
        context.insert_evidence(evidence("log-1", "verificationLog"));
        context.insert_evidence(evidence("artifact-2", "artifactSummary"));

        let request = request(
            proposal(
                "markIssueDone",
                "BuildAgent",
                Some(ActionRef {
                    object_type: "Issue".into(),
                    id: "ISS-1".into(),
                }),
                vec!["artifact-1", "log-1", "artifact-2"],
            ),
            "req-1",
        );

        let decision = arbitrate_action(&request, &context);
        assert_eq!(decision.status, ArbitrationDecisionStatus::Accepted);
        let accepted = decision.accepted_action.expect("accepted action");
        assert_eq!(accepted.proposal_id, "proposal-markIssueDone");
        assert_eq!(accepted.from_state.as_deref(), Some("reviewReady"));
        assert_eq!(accepted.to_state.as_deref(), Some("done"));
        assert!(accepted
            .expected_events
            .iter()
            .any(|event| event == "IssueMarkedDone"));
    }

    #[test]
    fn unknown_action_returns_rejected() {
        let context = core_context();
        let request = request(
            proposal("unknownAction", "BuildAgent", None, Vec::<&str>::new()),
            "req-2",
        );

        let decision = arbitrate_action(&request, &context);
        assert_eq!(decision.status, ArbitrationDecisionStatus::Rejected);
        assert_eq!(
            decision.rejected_reasons[0].code,
            RejectionReasonCode::UnknownActionType
        );
    }

    #[test]
    fn build_agent_cannot_create_finding() {
        let mut context = core_context();
        context.insert_state(StateFact {
            object_type: "Audit".into(),
            object_id: "AUD-1".into(),
            state_id: "running".into(),
        });
        context.insert_evidence(evidence("audit-report-1", "auditReport"));

        let request = request(
            proposal(
                "createFinding",
                "BuildAgent",
                Some(ActionRef {
                    object_type: "Audit".into(),
                    id: "AUD-1".into(),
                }),
                vec!["audit-report-1"],
            ),
            "req-3",
        );

        let decision = arbitrate_action(&request, &context);
        assert_eq!(decision.status, ArbitrationDecisionStatus::Rejected);
        assert_eq!(
            decision.rejected_reasons[0].code,
            RejectionReasonCode::RoleCannotExecuteAction
        );
    }

    #[test]
    fn audit_agent_cannot_mark_issue_done() {
        let mut context = core_context();
        context.insert_state(StateFact {
            object_type: "Issue".into(),
            object_id: "ISS-1".into(),
            state_id: "reviewReady".into(),
        });
        context.insert_evidence(evidence("artifact-1", "implementationSummary"));
        context.insert_evidence(evidence("log-1", "verificationLog"));
        context.insert_evidence(evidence("artifact-2", "artifactSummary"));

        let request = request(
            proposal(
                "markIssueDone",
                "AuditAgent",
                Some(ActionRef {
                    object_type: "Issue".into(),
                    id: "ISS-1".into(),
                }),
                vec!["artifact-1", "log-1", "artifact-2"],
            ),
            "req-4",
        );

        let decision = arbitrate_action(&request, &context);
        assert_eq!(decision.status, ArbitrationDecisionStatus::Rejected);
        assert_eq!(
            decision.rejected_reasons[0].code,
            RejectionReasonCode::RoleCannotExecuteAction
        );
    }

    #[test]
    fn invalid_object_state_returns_rejected() {
        let mut context = core_context();
        context.insert_state(StateFact {
            object_type: "Issue".into(),
            object_id: "ISS-1".into(),
            state_id: "done".into(),
        });
        context.insert_evidence(evidence("artifact-1", "implementationSummary"));
        context.insert_evidence(evidence("log-1", "verificationLog"));
        context.insert_evidence(evidence("artifact-2", "artifactSummary"));

        let request = request(
            proposal(
                "markIssueDone",
                "BuildAgent",
                Some(ActionRef {
                    object_type: "Issue".into(),
                    id: "ISS-1".into(),
                }),
                vec!["artifact-1", "log-1", "artifact-2"],
            ),
            "req-5",
        );

        let decision = arbitrate_action(&request, &context);
        assert_eq!(decision.status, ArbitrationDecisionStatus::Rejected);
        assert_eq!(
            decision.rejected_reasons[0].code,
            RejectionReasonCode::InvalidObjectState
        );
    }

    #[test]
    fn missing_evidence_returns_rejected() {
        let mut context = core_context();
        context.insert_state(StateFact {
            object_type: "Issue".into(),
            object_id: "ISS-1".into(),
            state_id: "reviewReady".into(),
        });
        context.insert_evidence(evidence("artifact-1", "implementationSummary"));

        let request = request(
            proposal(
                "markIssueDone",
                "BuildAgent",
                Some(ActionRef {
                    object_type: "Issue".into(),
                    id: "ISS-1".into(),
                }),
                vec!["artifact-1", "missing-log", "missing-artifact"],
            ),
            "req-6",
        );

        let decision = arbitrate_action(&request, &context);
        assert_eq!(decision.status, ArbitrationDecisionStatus::Rejected);
        assert_eq!(
            decision.rejected_reasons[0].code,
            RejectionReasonCode::MissingRequiredEvidence
        );
    }

    #[test]
    fn unmet_dependency_returns_rejected() {
        let mut context = core_context();
        let mut contract = context.action_contract_registry.bundle().clone();
        let action = contract
            .contracts
            .iter_mut()
            .find(|contract| contract.action_type == "activateIssue")
            .unwrap();
        action
            .preconditions
            .push(agentflow_action_contract::ActionPrecondition {
                id: "dep-build-ready".into(),
                kind: agentflow_action_contract::ActionPreconditionKind::DependencySatisfied,
                description: "Dependency must be ready.".into(),
                expression: Some("dep:build-ready".into()),
                required_state: None,
                required_link: None,
                required_evidence_type: None,
            });
        context.action_contract_registry =
            agentflow_action_contract::ActionContractRegistry::load_bundle(
                contract,
                &context.ontology_registry,
            )
            .unwrap();
        context.insert_state(StateFact {
            object_type: "Issue".into(),
            object_id: "ISS-1".into(),
            state_id: "proposed".into(),
        });
        context.insert_dependency(DependencyFact {
            dependency_key: "dep:build-ready".into(),
            satisfied: false,
            reason: None,
        });

        let request = request(
            proposal(
                "activateIssue",
                "BuildAgent",
                Some(ActionRef {
                    object_type: "Issue".into(),
                    id: "ISS-1".into(),
                }),
                Vec::<&str>::new(),
            ),
            "req-7",
        );

        let decision = arbitrate_action(&request, &context);
        assert_eq!(decision.status, ArbitrationDecisionStatus::Rejected);
        assert_eq!(
            decision.rejected_reasons[0].code,
            RejectionReasonCode::DependencyNotSatisfied
        );
    }

    #[test]
    fn active_write_lock_returns_rejected() {
        let mut context = core_context();
        context.insert_state(StateFact {
            object_type: "Issue".into(),
            object_id: "ISS-1".into(),
            state_id: "reviewReady".into(),
        });
        context.insert_evidence(evidence("artifact-1", "implementationSummary"));
        context.insert_evidence(evidence("log-1", "verificationLog"));
        context.insert_evidence(evidence("artifact-2", "artifactSummary"));
        context.push_lock(ObjectLock {
            lock_id: "lock-1".into(),
            object_type: "Issue".into(),
            object_id: "ISS-1".into(),
            lock_kind: ObjectLockKind::RunExecution,
            owner_proposal_id: "other-proposal".into(),
            owner_role: "BuildAgent".into(),
            expires_at: None,
            reason: "existing run".into(),
        });

        let request = request(
            proposal(
                "markIssueDone",
                "BuildAgent",
                Some(ActionRef {
                    object_type: "Issue".into(),
                    id: "ISS-1".into(),
                }),
                vec!["artifact-1", "log-1", "artifact-2"],
            ),
            "req-8",
        );

        let decision = arbitrate_action(&request, &context);
        assert_eq!(decision.status, ArbitrationDecisionStatus::Rejected);
        assert_eq!(
            decision.rejected_reasons[0].code,
            RejectionReasonCode::ObjectLockUnavailable
        );
    }

    #[test]
    fn approve_spec_without_human_decision_returns_human_decision_required() {
        let mut context = core_context();
        context.insert_state(StateFact {
            object_type: "Spec".into(),
            object_id: "SPEC-1".into(),
            state_id: "drafted".into(),
        });
        context.insert_evidence(evidence("non-human-1", "implementationSummary"));

        let request = request(
            proposal(
                "approveSpec",
                "HumanOwner",
                Some(ActionRef {
                    object_type: "Spec".into(),
                    id: "SPEC-1".into(),
                }),
                vec!["non-human-1"],
            ),
            "req-9",
        );

        let decision = arbitrate_action(&request, &context);
        assert_eq!(
            decision.status,
            ArbitrationDecisionStatus::HumanDecisionRequired
        );
        assert_eq!(
            decision
                .required_human_decision
                .expect("human decision request")
                .required_evidence_type,
            "humanConfirmation"
        );
    }

    #[test]
    fn accepted_action_includes_causation_proposal_id() {
        let mut context = core_context();
        context.insert_state(StateFact {
            object_type: "Issue".into(),
            object_id: "ISS-1".into(),
            state_id: "reviewReady".into(),
        });
        context.insert_evidence(evidence("artifact-1", "implementationSummary"));
        context.insert_evidence(evidence("log-1", "verificationLog"));
        context.insert_evidence(evidence("artifact-2", "artifactSummary"));

        let request = request(
            proposal(
                "markIssueDone",
                "BuildAgent",
                Some(ActionRef {
                    object_type: "Issue".into(),
                    id: "ISS-1".into(),
                }),
                vec!["artifact-1", "log-1", "artifact-2"],
            ),
            "req-10",
        );

        let decision = arbitrate_action(&request, &context);
        let accepted = decision.accepted_action.expect("accepted action");
        assert_eq!(
            accepted.accepted_action_id,
            "accepted-proposal-markIssueDone"
        );
        assert_eq!(accepted.proposal_id, "proposal-markIssueDone");
    }

    #[test]
    fn rejected_action_includes_stable_reason() {
        let context = core_context();
        let request = request(
            proposal("unknownAction", "BuildAgent", None, Vec::<&str>::new()),
            "req-11",
        );

        let decision = arbitrate_action(&request, &context);
        assert_eq!(
            decision.rejected_reasons[0].code,
            RejectionReasonCode::UnknownActionType
        );
        assert!(decision.rejected_reasons[0]
            .message
            .contains("unknown action type"));
    }

    #[test]
    fn issue_done_does_not_create_audit_accepted_action() {
        let mut context = core_context();
        context.insert_state(StateFact {
            object_type: "Issue".into(),
            object_id: "ISS-1".into(),
            state_id: "done".into(),
        });

        let request = request(
            proposal(
                "requestAudit",
                "HumanOwner",
                Some(ActionRef {
                    object_type: "Issue".into(),
                    id: "ISS-1".into(),
                }),
                Vec::<&str>::new(),
            ),
            "req-12",
        );

        let decision = arbitrate_action(&request, &context);
        assert_ne!(decision.status, ArbitrationDecisionStatus::Accepted);
    }

    #[test]
    fn lock_release_is_not_hidden_inside_decision() {
        let mut context = core_context();
        context.insert_state(StateFact {
            object_type: "Issue".into(),
            object_id: "ISS-1".into(),
            state_id: "reviewReady".into(),
        });
        context.insert_evidence(evidence("artifact-1", "implementationSummary"));
        context.insert_evidence(evidence("log-1", "verificationLog"));
        context.insert_evidence(evidence("artifact-2", "artifactSummary"));

        let request = request(
            proposal(
                "markIssueDone",
                "BuildAgent",
                Some(ActionRef {
                    object_type: "Issue".into(),
                    id: "ISS-1".into(),
                }),
                vec!["artifact-1", "log-1", "artifact-2"],
            ),
            "req-13",
        );

        let decision = arbitrate_action(&request, &context);
        assert!(decision.lock_plan.release.is_empty());
    }

    fn core_context() -> ArbitrationContext {
        let ontology = core_ontology_registry();
        let actions = core_action_contract_registry(&ontology);
        let roles = core_role_policy_registry(&ontology, &actions);
        let states = core_object_state_registry(&ontology, &actions).unwrap();
        ArbitrationContext::new(ontology, actions, roles, states)
    }

    fn request(proposal: ActionProposal, request_id: &str) -> ArbitrationRequest {
        ArbitrationRequest {
            request_id: request_id.into(),
            proposal,
            definition_versions: DefinitionVersions {
                ontology_version: "v1-draft".into(),
                contract_version: "v1-draft".into(),
                role_policy_version: "v1-draft".into(),
                object_state_version: "v1-draft".into(),
            },
            requested_at: "2026-06-20T00:00:00Z".into(),
        }
    }

    fn proposal(
        action_type: &str,
        actor_role: &str,
        target_object_ref: Option<ActionRef>,
        evidence_refs: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> ActionProposal {
        ActionProposal {
            proposal_id: format!("proposal-{action_type}"),
            idempotency_key: format!("runtime:issue:ISS-1:{action_type}:v1"),
            action_type: action_type.into(),
            actor_role: actor_role.into(),
            source_surface: ActionSourceSurface::Agent,
            target_object_ref,
            input: proposal_input(action_type),
            evidence_refs: evidence_refs
                .into_iter()
                .map(|value| value.as_ref().to_string())
                .collect(),
            artifact_refs: Vec::new(),
            reason: None,
            expected_effects: Vec::new(),
            ontology_version: "v1-draft".into(),
            contract_version: "v1-draft".into(),
            created_at: "2026-06-20T00:00:00Z".into(),
        }
    }

    fn proposal_input(action_type: &str) -> serde_json::Value {
        match action_type {
            "markIssueDone" => json!({
                "completionSummary": "done"
            }),
            "createFinding" => json!({
                "severity": "high",
                "summary": "finding"
            }),
            "approveSpec" => json!({
                "decisionSummary": "approved"
            }),
            "activateIssue" => json!({
                "activationReason": "start"
            }),
            "requestAudit" => json!({
                "reason": "need audit"
            }),
            _ => json!({
                "summary": "test"
            }),
        }
    }

    fn evidence(evidence_ref: &str, evidence_type: &str) -> EvidenceFact {
        EvidenceFact {
            evidence_ref: evidence_ref.into(),
            evidence_type: evidence_type.into(),
        }
    }
}
