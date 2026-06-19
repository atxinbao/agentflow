use agentflow_ontology::OntologyRegistry;

use crate::model::{
    AcceptedRefKind, ActionApprovalHint, ActionCategory, ActionContract, ActionContractBundle,
    ActionDefinitionStatus, ActionEffect, ActionEffectKind, ActionExpectedEvent,
    ActionFieldDefinition, ActionFieldValueType, ActionIdempotencyPolicy, ActionInputSchema,
    ActionPrecondition, ActionPreconditionKind, ActionSimulationHint, ActionTargetMode,
    ActionTypeDefinition, RequiredEvidenceDefinition, ACTION_CONTRACT_BUNDLE_VERSION,
};
use crate::registry::ActionContractRegistry;

pub const CORE_ACTION_CONTRACT_ID: &str = "agentflow.actions.core";
pub const CORE_ACTION_CONTRACT_NAMESPACE: &str = "agentflow.actions.core";
pub const CORE_ACTION_CONTRACT_VERSION: &str = "v1-draft";
pub const CORE_ACTION_CONTRACT_REF: &str = "agentflow.actions.core@v1-draft";

pub fn core_action_contract_bundle() -> ActionContractBundle {
    let action_types = vec![
        action_type(
            "submitRequirement",
            ActionCategory::Intake,
            ActionTargetMode::CreateObject,
            None,
            Some("Requirement"),
            "Submit Requirement",
            "Record raw requirement input.",
        ),
        action_type(
            "normalizeRequirement",
            ActionCategory::Intake,
            ActionTargetMode::ExistingObject,
            Some("Requirement"),
            None,
            "Normalize Requirement",
            "Normalize requirement content.",
        ),
        action_type(
            "classifyRequirement",
            ActionCategory::Intake,
            ActionTargetMode::ExistingObject,
            Some("Requirement"),
            None,
            "Classify Requirement",
            "Classify normalized requirement.",
        ),
        action_type(
            "draftSpec",
            ActionCategory::Spec,
            ActionTargetMode::ExistingObject,
            Some("Requirement"),
            Some("Spec"),
            "Draft Spec",
            "Generate spec draft from requirement.",
        ),
        action_type(
            "approveSpec",
            ActionCategory::Spec,
            ActionTargetMode::ExistingObject,
            Some("Spec"),
            None,
            "Approve Spec",
            "Record approved spec decision.",
        ),
        action_type(
            "createProject",
            ActionCategory::Planning,
            ActionTargetMode::ExistingObject,
            Some("Spec"),
            Some("Project"),
            "Create Project",
            "Create project aggregate from approved spec.",
        ),
        action_type(
            "createIssue",
            ActionCategory::Planning,
            ActionTargetMode::ExistingObject,
            Some("Project"),
            Some("Issue"),
            "Create Issue",
            "Create executable issue from project.",
        ),
        action_type(
            "activateIssue",
            ActionCategory::Planning,
            ActionTargetMode::ExistingObject,
            Some("Issue"),
            None,
            "Activate Issue",
            "Mark issue as executable.",
        ),
        action_type(
            "startRun",
            ActionCategory::Execution,
            ActionTargetMode::ExistingObject,
            Some("Issue"),
            Some("Run"),
            "Start Run",
            "Start one execution run.",
        ),
        action_type(
            "submitEvidence",
            ActionCategory::Evidence,
            ActionTargetMode::ExistingObject,
            Some("Run"),
            Some("Evidence"),
            "Submit Evidence",
            "Attach evidence to a run.",
        ),
        action_type(
            "submitArtifact",
            ActionCategory::Evidence,
            ActionTargetMode::ExistingObject,
            Some("Run"),
            Some("Artifact"),
            "Submit Artifact",
            "Attach artifact reference to a run.",
        ),
        action_type(
            "markIssueDone",
            ActionCategory::Execution,
            ActionTargetMode::ExistingObject,
            Some("Issue"),
            None,
            "Mark Issue Done",
            "Mark issue done with verification evidence.",
        ),
        action_type(
            "recordDecision",
            ActionCategory::Decision,
            ActionTargetMode::RecordDecision,
            None,
            Some("Decision"),
            "Record Decision",
            "Record human confirmation or rejection.",
        ),
        action_type(
            "requestAudit",
            ActionCategory::Audit,
            ActionTargetMode::ExistingObject,
            Some("Issue"),
            Some("Audit"),
            "Request Audit",
            "Create independent audit request.",
        ),
        action_type(
            "createFinding",
            ActionCategory::Finding,
            ActionTargetMode::ExistingObject,
            Some("Audit"),
            Some("Finding"),
            "Create Finding",
            "Create independent finding.",
        ),
        action_type(
            "linkFixIssue",
            ActionCategory::Finding,
            ActionTargetMode::LinkObjects,
            Some("Finding"),
            None,
            "Link Fix Issue",
            "Link a finding to the fix issue.",
        ),
    ];

    let contracts = vec![
        contract(
            "submitRequirement",
            ActionTargetMode::CreateObject,
            None,
            Some("Requirement"),
            input_schema(
                vec![
                    field(
                        "summary",
                        ActionFieldValueType::String,
                        true,
                        "Requirement summary.",
                    ),
                    field(
                        "requestType",
                        ActionFieldValueType::String,
                        true,
                        "Normalized request type.",
                    ),
                ],
                false,
            ),
            vec![],
            vec![effect(
                "create-requirement",
                ActionEffectKind::CreateObject,
                "Create requirement object.",
                Some("Requirement"),
                None,
                None,
                Some("RequirementSubmitted"),
            )],
            vec![],
            vec![expected_event("RequirementSubmitted", Some("Requirement"))],
            vec![],
        ),
        contract(
            "normalizeRequirement",
            ActionTargetMode::ExistingObject,
            Some("Requirement"),
            None,
            input_schema(
                vec![field(
                    "normalizedSummary",
                    ActionFieldValueType::String,
                    true,
                    "Normalized summary.",
                )],
                false,
            ),
            vec![precondition(
                "target-exists",
                ActionPreconditionKind::TargetExists,
                "Requirement must exist.",
            )],
            vec![effect(
                "emit-normalized",
                ActionEffectKind::EmitEvent,
                "Emit normalized requirement event.",
                Some("Requirement"),
                None,
                None,
                Some("RequirementNormalized"),
            )],
            vec![],
            vec![expected_event("RequirementNormalized", Some("Requirement"))],
            vec![],
        ),
        contract(
            "classifyRequirement",
            ActionTargetMode::ExistingObject,
            Some("Requirement"),
            None,
            input_schema(
                vec![
                    field(
                        "intentType",
                        ActionFieldValueType::Enum,
                        true,
                        "Intent classification.",
                    )
                    .with_enum_values(&[
                        "question",
                        "research",
                        "feature",
                        "bug",
                        "audit",
                        "design-only",
                        "executable-issue",
                    ]),
                    field(
                        "boundarySummary",
                        ActionFieldValueType::String,
                        true,
                        "Boundary summary.",
                    ),
                ],
                false,
            ),
            vec![precondition(
                "target-exists",
                ActionPreconditionKind::TargetExists,
                "Requirement must exist.",
            )],
            vec![effect(
                "emit-classified",
                ActionEffectKind::EmitEvent,
                "Emit requirement classified event.",
                Some("Requirement"),
                None,
                None,
                Some("RequirementClassified"),
            )],
            vec![],
            vec![expected_event("RequirementClassified", Some("Requirement"))],
            vec![],
        ),
        contract(
            "draftSpec",
            ActionTargetMode::ExistingObject,
            Some("Requirement"),
            Some("Spec"),
            input_schema(
                vec![
                    field(
                        "specTitle",
                        ActionFieldValueType::String,
                        true,
                        "Draft spec title.",
                    ),
                    field(
                        "goal",
                        ActionFieldValueType::String,
                        true,
                        "Draft spec goal.",
                    ),
                ],
                false,
            ),
            vec![precondition(
                "target-exists",
                ActionPreconditionKind::TargetExists,
                "Requirement must exist.",
            )],
            vec![effect(
                "create-spec",
                ActionEffectKind::CreateObject,
                "Create spec draft object.",
                Some("Spec"),
                None,
                None,
                Some("SpecDrafted"),
            )],
            vec![required_evidence(
                "specDraftPreview",
                1,
                AcceptedRefKind::ArtifactRef,
                "Spec draft preview must exist.",
            )],
            vec![expected_event("SpecDrafted", Some("Spec"))],
            vec!["derivesFrom"],
        ),
        contract(
            "approveSpec",
            ActionTargetMode::ExistingObject,
            Some("Spec"),
            None,
            input_schema(
                vec![field(
                    "decisionSummary",
                    ActionFieldValueType::String,
                    true,
                    "Approval summary.",
                )],
                false,
            ),
            vec![precondition(
                "target-exists",
                ActionPreconditionKind::TargetExists,
                "Spec must exist.",
            )],
            vec![effect(
                "emit-approved",
                ActionEffectKind::EmitEvent,
                "Emit spec approved event.",
                Some("Spec"),
                None,
                Some("spec.state-machine"),
                Some("SpecApproved"),
            )],
            vec![required_evidence(
                "humanConfirmation",
                1,
                AcceptedRefKind::DecisionRef,
                "Human confirmation is required.",
            )],
            vec![expected_event("SpecApproved", Some("Spec"))],
            vec!["accepts"],
        ),
        contract(
            "createProject",
            ActionTargetMode::ExistingObject,
            Some("Spec"),
            Some("Project"),
            input_schema(
                vec![
                    field(
                        "projectId",
                        ActionFieldValueType::String,
                        true,
                        "Project identifier.",
                    ),
                    field(
                        "projectTitle",
                        ActionFieldValueType::String,
                        true,
                        "Project title.",
                    ),
                ],
                false,
            ),
            vec![precondition(
                "target-exists",
                ActionPreconditionKind::TargetExists,
                "Spec must exist.",
            )],
            vec![effect(
                "create-project",
                ActionEffectKind::CreateObject,
                "Create project object.",
                Some("Project"),
                None,
                None,
                Some("ProjectCreated"),
            )],
            vec![],
            vec![expected_event("ProjectCreated", Some("Project"))],
            vec![],
        ),
        contract(
            "createIssue",
            ActionTargetMode::ExistingObject,
            Some("Project"),
            Some("Issue"),
            input_schema(
                vec![
                    field(
                        "issueId",
                        ActionFieldValueType::String,
                        true,
                        "Issue identifier.",
                    ),
                    field("title", ActionFieldValueType::String, true, "Issue title."),
                ],
                false,
            ),
            vec![precondition(
                "target-exists",
                ActionPreconditionKind::TargetExists,
                "Project must exist.",
            )],
            vec![effect(
                "create-issue",
                ActionEffectKind::CreateObject,
                "Create issue object.",
                Some("Issue"),
                None,
                None,
                Some("IssueCreated"),
            )],
            vec![],
            vec![expected_event("IssueCreated", Some("Issue"))],
            vec!["contains"],
        ),
        contract(
            "activateIssue",
            ActionTargetMode::ExistingObject,
            Some("Issue"),
            None,
            input_schema(
                vec![field(
                    "activationReason",
                    ActionFieldValueType::String,
                    true,
                    "Why the issue is activated.",
                )],
                false,
            ),
            vec![precondition(
                "target-exists",
                ActionPreconditionKind::TargetExists,
                "Issue must exist.",
            )],
            vec![effect(
                "emit-issue-active",
                ActionEffectKind::EmitEvent,
                "Emit issue activated event.",
                Some("Issue"),
                None,
                Some("issue.state-machine"),
                Some("IssueActivated"),
            )],
            vec![],
            vec![expected_event("IssueActivated", Some("Issue"))],
            vec![],
        ),
        contract(
            "startRun",
            ActionTargetMode::ExistingObject,
            Some("Issue"),
            Some("Run"),
            input_schema(
                vec![field(
                    "runId",
                    ActionFieldValueType::String,
                    true,
                    "Run identifier.",
                )],
                false,
            ),
            vec![precondition(
                "target-exists",
                ActionPreconditionKind::TargetExists,
                "Issue must exist.",
            )],
            vec![effect(
                "create-run",
                ActionEffectKind::CreateObject,
                "Create run object.",
                Some("Run"),
                None,
                Some("run.state-machine"),
                Some("RunStarted"),
            )],
            vec![],
            vec![expected_event("RunStarted", Some("Run"))],
            vec!["executes"],
        ),
        contract(
            "submitEvidence",
            ActionTargetMode::ExistingObject,
            Some("Run"),
            Some("Evidence"),
            input_schema(
                vec![field(
                    "evidenceSummary",
                    ActionFieldValueType::String,
                    true,
                    "Evidence summary.",
                )],
                false,
            ),
            vec![precondition(
                "target-exists",
                ActionPreconditionKind::TargetExists,
                "Run must exist.",
            )],
            vec![effect(
                "attach-evidence",
                ActionEffectKind::AttachEvidence,
                "Attach evidence to run.",
                Some("Evidence"),
                None,
                Some("run.state-machine"),
                Some("EvidenceSubmitted"),
            )],
            vec![required_evidence(
                "verificationLog",
                1,
                AcceptedRefKind::EvidenceRef,
                "Verification log reference is required.",
            )],
            vec![expected_event("EvidenceSubmitted", Some("Evidence"))],
            vec!["supports", "proves"],
        ),
        contract(
            "submitArtifact",
            ActionTargetMode::ExistingObject,
            Some("Run"),
            Some("Artifact"),
            input_schema(
                vec![field(
                    "artifactSummary",
                    ActionFieldValueType::String,
                    true,
                    "Artifact summary.",
                )],
                false,
            ),
            vec![precondition(
                "target-exists",
                ActionPreconditionKind::TargetExists,
                "Run must exist.",
            )],
            vec![effect(
                "attach-artifact",
                ActionEffectKind::AttachArtifact,
                "Attach artifact to run.",
                Some("Artifact"),
                None,
                Some("run.state-machine"),
                Some("ArtifactSubmitted"),
            )],
            vec![required_evidence(
                "artifactSummary",
                1,
                AcceptedRefKind::ArtifactRef,
                "Artifact summary reference is required.",
            )],
            vec![expected_event("ArtifactSubmitted", Some("Artifact"))],
            vec!["produces"],
        ),
        contract(
            "markIssueDone",
            ActionTargetMode::ExistingObject,
            Some("Issue"),
            None,
            input_schema(
                vec![field(
                    "completionSummary",
                    ActionFieldValueType::String,
                    true,
                    "Completion summary.",
                )],
                false,
            ),
            vec![precondition(
                "target-exists",
                ActionPreconditionKind::TargetExists,
                "Issue must exist.",
            )],
            vec![effect(
                "issue-done",
                ActionEffectKind::ChangeState,
                "Mark issue done.",
                Some("Issue"),
                None,
                Some("issue.state-machine"),
                Some("IssueMarkedDone"),
            )],
            vec![
                required_evidence(
                    "implementationSummary",
                    1,
                    AcceptedRefKind::ArtifactRef,
                    "Implementation summary is required.",
                ),
                required_evidence(
                    "verificationLog",
                    1,
                    AcceptedRefKind::EvidenceRef,
                    "Verification log is required.",
                ),
                required_evidence(
                    "artifactSummary",
                    1,
                    AcceptedRefKind::ArtifactRef,
                    "Artifact summary is required.",
                ),
            ],
            vec![
                expected_event("IssueMarkedDone", Some("Issue")),
                expected_event("EvidenceLinked", Some("Issue")),
            ],
            vec![],
        ),
        contract(
            "recordDecision",
            ActionTargetMode::RecordDecision,
            None,
            Some("Decision"),
            input_schema(
                vec![
                    field(
                        "outcome",
                        ActionFieldValueType::String,
                        true,
                        "Decision outcome.",
                    ),
                    field(
                        "targetObjectType",
                        ActionFieldValueType::String,
                        true,
                        "Decision target object type.",
                    ),
                    field(
                        "targetObjectId",
                        ActionFieldValueType::String,
                        true,
                        "Decision target object id.",
                    ),
                ],
                false,
            ),
            vec![],
            vec![effect(
                "record-decision",
                ActionEffectKind::RecordDecision,
                "Record human decision.",
                Some("Decision"),
                None,
                None,
                Some("DecisionRecorded"),
            )],
            vec![required_evidence(
                "humanConfirmation",
                1,
                AcceptedRefKind::DecisionRef,
                "Human confirmation record is required.",
            )],
            vec![expected_event("DecisionRecorded", Some("Decision"))],
            vec!["decides", "accepts"],
        ),
        contract(
            "requestAudit",
            ActionTargetMode::ExistingObject,
            Some("Issue"),
            Some("Audit"),
            input_schema(
                vec![field(
                    "reason",
                    ActionFieldValueType::String,
                    true,
                    "Why the audit is requested.",
                )],
                false,
            ),
            vec![precondition(
                "target-exists",
                ActionPreconditionKind::TargetExists,
                "Issue must exist.",
            )],
            vec![effect(
                "request-audit",
                ActionEffectKind::CreateObject,
                "Create audit request.",
                Some("Audit"),
                None,
                Some("audit.state-machine"),
                Some("AuditRequested"),
            )],
            vec![required_evidence(
                "humanConfirmation",
                1,
                AcceptedRefKind::DecisionRef,
                "Audit request needs explicit human confirmation.",
            )],
            vec![expected_event("AuditRequested", Some("Audit"))],
            vec!["reviews"],
        ),
        contract(
            "createFinding",
            ActionTargetMode::ExistingObject,
            Some("Audit"),
            Some("Finding"),
            input_schema(
                vec![
                    field(
                        "severity",
                        ActionFieldValueType::String,
                        true,
                        "Finding severity.",
                    ),
                    field(
                        "summary",
                        ActionFieldValueType::String,
                        true,
                        "Finding summary.",
                    ),
                ],
                false,
            ),
            vec![precondition(
                "target-exists",
                ActionPreconditionKind::TargetExists,
                "Audit must exist.",
            )],
            vec![effect(
                "create-finding",
                ActionEffectKind::CreateObject,
                "Create finding object.",
                Some("Finding"),
                None,
                Some("finding.state-machine"),
                Some("FindingCreated"),
            )],
            vec![required_evidence(
                "auditReport",
                1,
                AcceptedRefKind::ArtifactRef,
                "Audit report is required.",
            )],
            vec![expected_event("FindingCreated", Some("Finding"))],
            vec!["reviews"],
        ),
        contract(
            "linkFixIssue",
            ActionTargetMode::LinkObjects,
            Some("Finding"),
            None,
            input_schema(
                vec![
                    field(
                        "issueId",
                        ActionFieldValueType::String,
                        true,
                        "Fix issue identifier.",
                    ),
                    field(
                        "issueObjectType",
                        ActionFieldValueType::String,
                        true,
                        "Fix issue object type.",
                    )
                    .with_object_type_ref("Issue"),
                ],
                false,
            ),
            vec![precondition(
                "target-exists",
                ActionPreconditionKind::TargetExists,
                "Finding must exist.",
            )],
            vec![effect(
                "link-fix-issue",
                ActionEffectKind::CreateLink,
                "Link finding to fix issue.",
                Some("Issue"),
                Some("requiresFix"),
                None,
                Some("FixIssueLinked"),
            )],
            vec![],
            vec![expected_event("FixIssueLinked", Some("Finding"))],
            vec!["requiresFix"],
        ),
    ];

    ActionContractBundle {
        version: ACTION_CONTRACT_BUNDLE_VERSION.into(),
        registry_id: CORE_ACTION_CONTRACT_ID.into(),
        namespace: CORE_ACTION_CONTRACT_NAMESPACE.into(),
        definition_version: CORE_ACTION_CONTRACT_VERSION.into(),
        status: ActionDefinitionStatus::Draft,
        action_types,
        contracts,
    }
}

pub fn core_action_contract_registry(
    ontology_registry: &OntologyRegistry,
) -> ActionContractRegistry {
    ActionContractRegistry::load_bundle(core_action_contract_bundle(), ontology_registry)
        .expect("built-in core action contracts must validate")
}

fn action_type(
    id: &str,
    category: ActionCategory,
    target_mode: ActionTargetMode,
    target_object_type: Option<&str>,
    creates_object_type: Option<&str>,
    name: &str,
    description: &str,
) -> ActionTypeDefinition {
    ActionTypeDefinition {
        id: id.into(),
        namespace: CORE_ACTION_CONTRACT_NAMESPACE.into(),
        version: CORE_ACTION_CONTRACT_VERSION.into(),
        status: ActionDefinitionStatus::Draft,
        name: name.into(),
        description: description.into(),
        category,
        target_mode,
        target_object_type: target_object_type.map(str::to_string),
        creates_object_type: creates_object_type.map(str::to_string),
        contract_ref: format!("{id}@{CORE_ACTION_CONTRACT_VERSION}"),
    }
}

fn contract(
    action_type: &str,
    target_mode: ActionTargetMode,
    target_object_type: Option<&str>,
    creates_object_type: Option<&str>,
    input_schema: ActionInputSchema,
    preconditions: Vec<ActionPrecondition>,
    effects: Vec<ActionEffect>,
    required_evidence: Vec<RequiredEvidenceDefinition>,
    expected_events: Vec<ActionExpectedEvent>,
    expected_links: Vec<&str>,
) -> ActionContract {
    ActionContract {
        id: format!("{action_type}@{CORE_ACTION_CONTRACT_VERSION}"),
        action_type: action_type.into(),
        namespace: CORE_ACTION_CONTRACT_NAMESPACE.into(),
        version: CORE_ACTION_CONTRACT_VERSION.into(),
        status: ActionDefinitionStatus::Draft,
        target_mode,
        target_object_type: target_object_type.map(str::to_string),
        creates_object_type: creates_object_type.map(str::to_string),
        input_schema,
        preconditions,
        state_transition_ref: None,
        effects,
        required_evidence,
        expected_events,
        expected_links: expected_links.into_iter().map(str::to_string).collect(),
        idempotency: ActionIdempotencyPolicy { required: true },
        conflict_scope_hint: None,
        approval_hint: ActionApprovalHint {
            human_approval_required: matches!(
                action_type,
                "approveSpec" | "recordDecision" | "requestAudit"
            ),
        },
        rollback_hint: None,
        simulation_hint: ActionSimulationHint { enabled: true },
    }
}

fn input_schema(
    fields: Vec<ActionFieldDefinition>,
    allow_additional_fields: bool,
) -> ActionInputSchema {
    let required_fields = fields
        .iter()
        .filter(|field| field.required)
        .map(|field| field.name.clone())
        .collect();
    ActionInputSchema {
        fields,
        required_fields,
        allow_additional_fields,
    }
}

fn field(
    name: &str,
    value_type: ActionFieldValueType,
    required: bool,
    description: &str,
) -> ActionFieldDefinition {
    ActionFieldDefinition {
        name: name.into(),
        value_type,
        required,
        description: description.into(),
        enum_values: Vec::new(),
        object_type_ref: None,
        link_type_ref: None,
    }
}

fn precondition(id: &str, kind: ActionPreconditionKind, description: &str) -> ActionPrecondition {
    ActionPrecondition {
        id: id.into(),
        kind,
        description: description.into(),
        expression: None,
        required_state: None,
        required_link: None,
        required_evidence_type: None,
    }
}

fn effect(
    id: &str,
    kind: ActionEffectKind,
    description: &str,
    object_type: Option<&str>,
    link_type: Option<&str>,
    state_transition_ref: Option<&str>,
    event_type: Option<&str>,
) -> ActionEffect {
    ActionEffect {
        id: id.into(),
        kind,
        description: description.into(),
        object_type: object_type.map(str::to_string),
        link_type: link_type.map(str::to_string),
        state_transition_ref: state_transition_ref.map(str::to_string),
        event_type: event_type.map(str::to_string),
    }
}

fn required_evidence(
    evidence_type: &str,
    min_count: usize,
    accepted_ref_kind: AcceptedRefKind,
    description: &str,
) -> RequiredEvidenceDefinition {
    RequiredEvidenceDefinition {
        evidence_type: evidence_type.into(),
        required: true,
        min_count,
        accepted_ref_kind,
        description: description.into(),
    }
}

fn expected_event(event_type: &str, object_type: Option<&str>) -> ActionExpectedEvent {
    ActionExpectedEvent {
        event_type: event_type.into(),
        object_type: object_type.map(str::to_string),
        required: true,
        payload_fields: Vec::new(),
    }
}

trait ActionFieldBuilder {
    fn with_enum_values(self, values: &[&str]) -> Self;
    fn with_object_type_ref(self, object_type_ref: &str) -> Self;
}

impl ActionFieldBuilder for ActionFieldDefinition {
    fn with_enum_values(mut self, values: &[&str]) -> Self {
        self.enum_values = values.iter().map(|value| (*value).to_string()).collect();
        self
    }

    fn with_object_type_ref(mut self, object_type_ref: &str) -> Self {
        self.object_type_ref = Some(object_type_ref.into());
        self
    }
}
