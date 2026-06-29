use serde::{Deserialize, Serialize};

pub const CORE_EVIDENCE_PACK_SCHEMA_VERSION: &str = "agentflow-core-evidence-pack.v1";
pub const CORE_EVIDENCE_SOURCE_TYPE_REGISTRY_VERSION: &str =
    "agentflow-core-evidence-source-type-registry.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreEvidencePack {
    pub version: String,
    pub evidence_id: String,
    pub status: String,
    pub producer: CoreEvidenceProducerRef,
    pub subject: CoreEvidenceSubjectRef,
    pub source_type: String,
    pub digest: CoreEvidenceDigest,
    pub artifact_refs: Vec<CoreEvidenceArtifactRef>,
    pub provenance: CoreEvidenceProvenance,
    pub trace_refs: CoreEvidenceTraceRefs,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreEvidenceProducerRef {
    pub actor_ref: String,
    pub role_ref: String,
    pub tool_ref: Option<String>,
    pub produced_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreEvidenceSubjectRef {
    pub subject_ref_kind: String,
    pub subject_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreEvidenceDigest {
    pub algorithm: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreEvidenceArtifactRef {
    pub artifact_ref: String,
    pub artifact_kind: String,
    pub digest: Option<CoreEvidenceDigest>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreEvidenceProvenance {
    pub capture_ref: String,
    pub capture_method: String,
    pub collected_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreEvidenceTraceRefs {
    pub spec_refs: Vec<String>,
    pub goal_refs: Vec<String>,
    pub task_refs: Vec<String>,
    pub run_refs: Vec<String>,
    pub action_refs: Vec<String>,
    pub decision_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreEvidencePackNegativeFixtureResult {
    pub fixture_id: String,
    pub status: String,
    pub expected_reason: String,
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreEvidenceSourceTypeDefinition {
    pub source_type: String,
    pub required_fields: Vec<String>,
    pub allowed_statuses: Vec<String>,
    pub validation_rule: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreEvidenceReferenceAppSourceExample {
    pub reference_app: String,
    pub example_source: String,
    pub source_type: String,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreEvidenceSourceTypeRegistryContract {
    pub version: String,
    pub status: String,
    pub authority: String,
    pub reference_mapping_boundary: String,
    pub source_types: Vec<CoreEvidenceSourceTypeDefinition>,
    pub reference_app_examples: Vec<CoreEvidenceReferenceAppSourceExample>,
}

pub fn canonical_core_evidence_pack_fixture() -> CoreEvidencePack {
    CoreEvidencePack {
        version: CORE_EVIDENCE_PACK_SCHEMA_VERSION.to_string(),
        evidence_id: "evidence-core-canonical-001".to_string(),
        status: "collected".to_string(),
        producer: CoreEvidenceProducerRef {
            actor_ref: "actor:work-agent".to_string(),
            role_ref: "role:work".to_string(),
            tool_ref: Some("tool:local-validator".to_string()),
            produced_at: "2026-06-29T00:00:00Z".to_string(),
        },
        subject: CoreEvidenceSubjectRef {
            subject_ref_kind: "TaskRef".to_string(),
            subject_ref: "task:core-evidence-pack".to_string(),
        },
        source_type: "artifact".to_string(),
        digest: CoreEvidenceDigest {
            algorithm: "sha256".to_string(),
            value: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string(),
        },
        artifact_refs: vec![CoreEvidenceArtifactRef {
            artifact_ref: "artifact:core-evidence-pack-canonical".to_string(),
            artifact_kind: "generic-artifact".to_string(),
            digest: Some(CoreEvidenceDigest {
                algorithm: "sha256".to_string(),
                value: "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789"
                    .to_string(),
            }),
        }],
        provenance: CoreEvidenceProvenance {
            capture_ref: "capture:local-run".to_string(),
            capture_method: "local-runtime".to_string(),
            collected_at: "2026-06-29T00:00:01Z".to_string(),
        },
        trace_refs: CoreEvidenceTraceRefs {
            spec_refs: vec!["spec:core-evidence-pack".to_string()],
            goal_refs: vec!["goal:evidence-kernel".to_string()],
            task_refs: vec!["task:core-evidence-pack".to_string()],
            run_refs: vec!["run:core-evidence-pack".to_string()],
            action_refs: vec!["action:attach-evidence".to_string()],
            decision_refs: vec!["decision:accept-evidence".to_string()],
        },
    }
}

pub fn core_evidence_source_type_registry_contract() -> CoreEvidenceSourceTypeRegistryContract {
    let allowed_statuses = vec![
        "collected".to_string(),
        "missing".to_string(),
        "invalid".to_string(),
        "deferred".to_string(),
        "superseded".to_string(),
    ];
    let required_fields = vec![
        "producer".to_string(),
        "subject".to_string(),
        "digest".to_string(),
        "artifactRefs".to_string(),
        "provenance".to_string(),
        "traceRefs".to_string(),
    ];

    CoreEvidenceSourceTypeRegistryContract {
        version: CORE_EVIDENCE_SOURCE_TYPE_REGISTRY_VERSION.to_string(),
        status: "active".to_string(),
        authority: "Core Evidence Source Type registry defines generic proof source categories."
            .to_string(),
        reference_mapping_boundary:
            "Reference App examples may map domain proof into Core source types, but mappings are not Core authority."
                .to_string(),
        source_types: [
            "artifact",
            "log",
            "screenshot",
            "external-proof",
            "command-output",
            "diff",
            "provenance",
            "human-confirmation",
        ]
        .into_iter()
        .map(|source_type| CoreEvidenceSourceTypeDefinition {
            source_type: source_type.to_string(),
            required_fields: required_fields.clone(),
            allowed_statuses: allowed_statuses.clone(),
            validation_rule:
                "source type evidence must provide producer, subject, digest, artifact refs, provenance, and trace refs"
                    .to_string(),
        })
        .collect(),
        reference_app_examples: vec![
            reference_example("software-dev", "changed-content-proof", "diff"),
            reference_example("software-dev", "local-command-proof", "command-output"),
            reference_example("software-dev", "ui-proof", "screenshot"),
            reference_example("software-dev", "merge-proof", "external-proof"),
        ],
    }
}

pub fn validate_core_evidence_pack_schema(pack: &CoreEvidencePack) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    if pack.version != CORE_EVIDENCE_PACK_SCHEMA_VERSION {
        errors.push(reason("version-mismatch"));
    }
    if pack.evidence_id.trim().is_empty() {
        errors.push(reason("evidence-id-missing"));
    }
    if !["collected", "missing", "invalid", "deferred", "superseded"]
        .contains(&pack.status.as_str())
    {
        errors.push(reason("status-unsupported"));
    }
    if pack.producer.actor_ref.trim().is_empty() {
        errors.push(reason("producer-actor-ref-missing"));
    }
    if pack.producer.role_ref.trim().is_empty() {
        errors.push(reason("producer-role-ref-missing"));
    }
    if pack.producer.produced_at.trim().is_empty() {
        errors.push(reason("producer-produced-at-missing"));
    }
    if pack.subject.subject_ref_kind.trim().is_empty() {
        errors.push(reason("subject-ref-kind-missing"));
    }
    if pack.subject.subject_ref.trim().is_empty() {
        errors.push(reason("subject-ref-missing"));
    }
    if pack.source_type.trim().is_empty() {
        errors.push(reason("source-type-missing"));
    }
    validate_digest("digest", &pack.digest, &mut errors);
    if pack.artifact_refs.is_empty() {
        errors.push(reason("artifact-refs-missing"));
    }
    for (index, artifact) in pack.artifact_refs.iter().enumerate() {
        if artifact.artifact_ref.trim().is_empty() {
            errors.push(reason(&format!("artifact-ref-{index}-missing")));
        }
        if artifact.artifact_kind.trim().is_empty() {
            errors.push(reason(&format!("artifact-kind-{index}-missing")));
        }
        if let Some(digest) = &artifact.digest {
            validate_digest(&format!("artifact-digest-{index}"), digest, &mut errors);
        }
    }
    if pack.provenance.capture_ref.trim().is_empty() {
        errors.push(reason("provenance-capture-ref-missing"));
    }
    if pack.provenance.capture_method.trim().is_empty() {
        errors.push(reason("provenance-capture-method-missing"));
    }
    if pack.provenance.collected_at.trim().is_empty() {
        errors.push(reason("provenance-collected-at-missing"));
    }
    validate_trace_refs(&pack.trace_refs, &mut errors);
    validate_core_surface_terms(pack, &mut errors);

    if errors.is_empty() {
        Ok(())
    } else {
        errors.sort();
        errors.dedup();
        Err(errors)
    }
}

pub fn validate_core_evidence_source_type_registry_contract(
    registry: &CoreEvidenceSourceTypeRegistryContract,
) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    if registry.version != CORE_EVIDENCE_SOURCE_TYPE_REGISTRY_VERSION {
        errors.push(reason("source-type-registry-version-mismatch"));
    }
    if registry.status != "active" {
        errors.push(reason("source-type-registry-status-unsupported"));
    }
    if !registry
        .reference_mapping_boundary
        .contains("not Core authority")
    {
        errors.push(reason("source-type-reference-boundary-missing"));
    }

    let required_source_types = [
        "artifact",
        "log",
        "screenshot",
        "external-proof",
        "command-output",
        "diff",
        "provenance",
        "human-confirmation",
    ];
    for source_type in required_source_types {
        if !registry
            .source_types
            .iter()
            .any(|definition| definition.source_type == source_type)
        {
            errors.push(reason(&format!("source-type-missing:{source_type}")));
        }
    }

    let required_statuses = ["collected", "missing", "invalid", "deferred", "superseded"];
    for definition in &registry.source_types {
        if definition.source_type.trim().is_empty() {
            errors.push(reason("source-type-empty"));
        }
        for status in required_statuses {
            if !definition
                .allowed_statuses
                .iter()
                .any(|allowed| allowed == status)
            {
                errors.push(reason(&format!(
                    "source-type-status-missing:{}:{status}",
                    definition.source_type
                )));
            }
        }
        for required_field in [
            "producer",
            "subject",
            "digest",
            "artifactRefs",
            "provenance",
            "traceRefs",
        ] {
            if !definition
                .required_fields
                .iter()
                .any(|field| field == required_field)
            {
                errors.push(reason(&format!(
                    "source-type-field-missing:{}:{required_field}",
                    definition.source_type
                )));
            }
        }
    }

    for example in &registry.reference_app_examples {
        if registry
            .source_types
            .iter()
            .all(|definition| definition.source_type != example.source_type)
        {
            errors.push(reason(&format!(
                "reference-example-source-type-unknown:{}",
                example.example_source
            )));
        }
        if example.status != "reference-only" {
            errors.push(reason(&format!(
                "reference-example-status-invalid:{}",
                example.example_source
            )));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        errors.sort();
        errors.dedup();
        Err(errors)
    }
}

pub fn validate_core_evidence_pack_source_type(
    pack: &CoreEvidencePack,
    registry: &CoreEvidenceSourceTypeRegistryContract,
) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();
    if let Err(schema_errors) = validate_core_evidence_pack_schema(pack) {
        errors.extend(schema_errors);
    }
    if let Err(registry_errors) = validate_core_evidence_source_type_registry_contract(registry) {
        errors.extend(registry_errors);
    }

    let Some(source_type) = registry
        .source_types
        .iter()
        .find(|definition| definition.source_type == pack.source_type)
    else {
        errors.push(reason("source-type-unknown"));
        errors.sort();
        errors.dedup();
        return Err(errors);
    };

    if !source_type
        .allowed_statuses
        .iter()
        .any(|status| status == &pack.status)
    {
        errors.push(reason("source-status-unsupported"));
    }
    if source_type
        .required_fields
        .iter()
        .any(|field| field == "artifactRefs")
        && pack.artifact_refs.is_empty()
    {
        errors.push(reason("source-required-artifact-refs-missing"));
    }
    if source_type
        .required_fields
        .iter()
        .any(|field| field == "provenance")
        && pack.provenance.capture_ref.trim().is_empty()
    {
        errors.push(reason("source-required-provenance-missing"));
    }

    if errors.is_empty() {
        Ok(())
    } else {
        errors.sort();
        errors.dedup();
        Err(errors)
    }
}

pub fn core_evidence_pack_negative_fixtures() -> Vec<CoreEvidencePackNegativeFixtureResult> {
    let fixtures = vec![
        ("missing-evidence-id", "evidence-id-missing", {
            let mut pack = canonical_core_evidence_pack_fixture();
            pack.evidence_id.clear();
            pack
        }),
        ("missing-source-type", "source-type-missing", {
            let mut pack = canonical_core_evidence_pack_fixture();
            pack.source_type.clear();
            pack
        }),
        ("missing-digest", "digest-value-invalid", {
            let mut pack = canonical_core_evidence_pack_fixture();
            pack.digest.value.clear();
            pack
        }),
        (
            "unsupported-digest-algorithm",
            "digest-algorithm-unsupported",
            {
                let mut pack = canonical_core_evidence_pack_fixture();
                pack.digest.algorithm = "md5".to_string();
                pack
            },
        ),
        ("missing-artifact-refs", "artifact-refs-missing", {
            let mut pack = canonical_core_evidence_pack_fixture();
            pack.artifact_refs.clear();
            pack
        }),
        ("missing-provenance", "provenance-capture-ref-missing", {
            let mut pack = canonical_core_evidence_pack_fixture();
            pack.provenance.capture_ref.clear();
            pack
        }),
        ("missing-trace-refs", "trace-spec-refs-missing", {
            let mut pack = canonical_core_evidence_pack_fixture();
            pack.trace_refs.spec_refs.clear();
            pack
        }),
        (
            "industry-term-pollution",
            "forbidden-core-term:github-issue",
            {
                let mut pack = canonical_core_evidence_pack_fixture();
                pack.subject.subject_ref = "github-issue:123".to_string();
                pack
            },
        ),
    ];

    fixtures
        .into_iter()
        .map(|(fixture_id, expected_reason, pack)| {
            let reasons = validate_core_evidence_pack_schema(&pack).unwrap_err();
            CoreEvidencePackNegativeFixtureResult {
                fixture_id: fixture_id.to_string(),
                status: if reasons.iter().any(|reason| reason == expected_reason) {
                    "passed".to_string()
                } else {
                    "failed".to_string()
                },
                expected_reason: expected_reason.to_string(),
                reasons,
            }
        })
        .collect()
}

fn reference_example(
    reference_app: &str,
    example_source: &str,
    source_type: &str,
) -> CoreEvidenceReferenceAppSourceExample {
    CoreEvidenceReferenceAppSourceExample {
        reference_app: reference_app.to_string(),
        example_source: example_source.to_string(),
        source_type: source_type.to_string(),
        status: "reference-only".to_string(),
    }
}

fn validate_digest(path: &str, digest: &CoreEvidenceDigest, errors: &mut Vec<String>) {
    if digest.algorithm != "sha256" {
        errors.push(reason(&format!("{path}-algorithm-unsupported")));
    }
    if digest.value.len() != 64
        || !digest
            .value
            .chars()
            .all(|character| character.is_ascii_hexdigit())
    {
        errors.push(reason(&format!("{path}-value-invalid")));
    }
}

fn validate_trace_refs(trace_refs: &CoreEvidenceTraceRefs, errors: &mut Vec<String>) {
    let trace_sets = [
        ("spec-refs", &trace_refs.spec_refs),
        ("goal-refs", &trace_refs.goal_refs),
        ("task-refs", &trace_refs.task_refs),
        ("run-refs", &trace_refs.run_refs),
        ("action-refs", &trace_refs.action_refs),
        ("decision-refs", &trace_refs.decision_refs),
    ];

    for (name, refs) in trace_sets {
        if refs.iter().all(|value| value.trim().is_empty()) {
            errors.push(reason(&format!("trace-{name}-missing")));
        }
    }
}

fn validate_core_surface_terms(pack: &CoreEvidencePack, errors: &mut Vec<String>) {
    let core_surface = [
        pack.evidence_id.as_str(),
        pack.status.as_str(),
        pack.producer.actor_ref.as_str(),
        pack.producer.role_ref.as_str(),
        pack.producer.tool_ref.as_deref().unwrap_or_default(),
        pack.subject.subject_ref_kind.as_str(),
        pack.subject.subject_ref.as_str(),
        pack.source_type.as_str(),
        pack.provenance.capture_ref.as_str(),
        pack.provenance.capture_method.as_str(),
    ]
    .into_iter()
    .chain(pack.artifact_refs.iter().flat_map(|artifact| {
        [
            artifact.artifact_ref.as_str(),
            artifact.artifact_kind.as_str(),
        ]
    }))
    .chain(pack.trace_refs.spec_refs.iter().map(String::as_str))
    .chain(pack.trace_refs.goal_refs.iter().map(String::as_str))
    .chain(pack.trace_refs.task_refs.iter().map(String::as_str))
    .chain(pack.trace_refs.run_refs.iter().map(String::as_str))
    .chain(pack.trace_refs.action_refs.iter().map(String::as_str))
    .chain(pack.trace_refs.decision_refs.iter().map(String::as_str));

    for value in core_surface {
        for term in FORBIDDEN_CORE_TERMS {
            if contains_forbidden_core_term(value, term) {
                errors.push(reason(&format!("forbidden-core-term:{term}")));
            }
        }
    }
}

fn reason(code: &str) -> String {
    code.to_string()
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

const FORBIDDEN_CORE_TERMS: &[&str] = &[
    "bug",
    "feature",
    "issue",
    "pr",
    "pull-request",
    "release",
    "repository",
    "repository-patch",
    "test-log",
    "github-issue",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn core_evidence_pack_schema_canonical_fixture_validates() {
        let pack = canonical_core_evidence_pack_fixture();
        validate_core_evidence_pack_schema(&pack).unwrap();
        assert_eq!(pack.version, CORE_EVIDENCE_PACK_SCHEMA_VERSION);
        assert_eq!(pack.trace_refs.spec_refs.len(), 1);
        assert_eq!(pack.trace_refs.decision_refs.len(), 1);
    }

    #[test]
    fn core_evidence_pack_schema_negative_fixtures_fail_with_stable_reasons() {
        let fixtures = core_evidence_pack_negative_fixtures();
        assert_eq!(fixtures.len(), 8);
        for fixture in fixtures {
            assert_eq!(
                fixture.status, "passed",
                "fixture {} failed with {:?}",
                fixture.fixture_id, fixture.reasons
            );
            assert!(fixture.reasons.contains(&fixture.expected_reason));
        }
    }

    #[test]
    fn core_evidence_pack_schema_rejects_industry_pollution() {
        let mut pack = canonical_core_evidence_pack_fixture();
        pack.artifact_refs[0].artifact_kind = "repository-patch".to_string();

        let errors = validate_core_evidence_pack_schema(&pack).unwrap_err();
        assert!(errors
            .iter()
            .any(|error| error == "forbidden-core-term:repository-patch"));
    }

    #[test]
    fn core_evidence_source_type_registry_contract_validates() {
        let registry = core_evidence_source_type_registry_contract();
        validate_core_evidence_source_type_registry_contract(&registry).unwrap();
        assert_eq!(registry.source_types.len(), 8);
        assert_eq!(registry.reference_app_examples.len(), 4);
    }

    #[test]
    fn core_evidence_source_type_registry_rejects_unknown_source_type() {
        let registry = core_evidence_source_type_registry_contract();
        let mut pack = canonical_core_evidence_pack_fixture();
        pack.source_type = "unknown-source".to_string();

        let errors = validate_core_evidence_pack_source_type(&pack, &registry).unwrap_err();
        assert!(errors.iter().any(|error| error == "source-type-unknown"));
    }

    #[test]
    fn core_evidence_source_type_registry_resolves_reference_examples() {
        let registry = core_evidence_source_type_registry_contract();
        let source_types = registry
            .source_types
            .iter()
            .map(|definition| definition.source_type.as_str())
            .collect::<std::collections::BTreeSet<_>>();

        for example in &registry.reference_app_examples {
            assert_eq!(example.status, "reference-only");
            assert!(source_types.contains(example.source_type.as_str()));
        }
    }
}
