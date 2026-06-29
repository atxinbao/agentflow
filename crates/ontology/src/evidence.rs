use std::{collections::BTreeSet, fs, path::Path};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const CORE_EVIDENCE_PACK_SCHEMA_VERSION: &str = "agentflow-core-evidence-pack.v1";
pub const CORE_EVIDENCE_SOURCE_TYPE_REGISTRY_VERSION: &str =
    "agentflow-core-evidence-source-type-registry.v1";
pub const CORE_EVIDENCE_CAPTURE_RECEIPT_VERSION: &str =
    "agentflow-core-evidence-capture-receipt.v1";
pub const CORE_EVIDENCE_AUTHORITY_TRACE_VERSION: &str =
    "agentflow-core-evidence-authority-trace.v1";

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreEvidenceCaptureReceipt {
    pub version: String,
    pub receipt_id: String,
    pub status: String,
    pub location: CoreEvidenceCaptureLocation,
    pub byte_count: u64,
    pub sha256: String,
    pub captured_at: String,
    pub producer: CoreEvidenceProducerRef,
    pub source_type: String,
    pub retention_hint: CoreEvidenceRetentionHint,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreEvidenceCaptureLocation {
    pub location_kind: String,
    pub path: Option<String>,
    pub uri: Option<String>,
    pub authority: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreEvidenceRetentionHint {
    pub retention_class: String,
    pub expires_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreEvidenceCaptureReceiptNegativeFixtureResult {
    pub fixture_id: String,
    pub status: String,
    pub expected_reason: String,
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreEvidenceAuthorityTrace {
    pub version: String,
    pub status: String,
    pub evidence_id: String,
    pub trace_refs: CoreEvidenceTraceRefs,
    pub authority_facts: Vec<CoreEvidenceAuthorityFactRef>,
    pub collection_event: CoreEvidenceCollectionEventLink,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreEvidenceAuthorityFactRef {
    pub fact_kind: String,
    pub fact_ref: String,
    pub authority_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreEvidenceCollectionEventLink {
    pub event_type: String,
    pub event_ref: String,
    pub event_store_path: String,
    pub correlation_id: String,
    pub causation_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreEvidenceAuthorityTraceNegativeFixtureResult {
    pub fixture_id: String,
    pub status: String,
    pub expected_reason: String,
    pub reasons: Vec<String>,
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

pub fn capture_core_evidence_receipt_for_local_file(
    path: &Path,
    receipt_id: impl Into<String>,
    producer: CoreEvidenceProducerRef,
    source_type: impl Into<String>,
    captured_at: impl Into<String>,
    retention_hint: CoreEvidenceRetentionHint,
) -> anyhow::Result<CoreEvidenceCaptureReceipt> {
    let bytes = fs::read(path)?;
    let receipt = CoreEvidenceCaptureReceipt {
        version: CORE_EVIDENCE_CAPTURE_RECEIPT_VERSION.to_string(),
        receipt_id: receipt_id.into(),
        status: "collected".to_string(),
        location: CoreEvidenceCaptureLocation {
            location_kind: "local-path".to_string(),
            path: Some(path.to_string_lossy().to_string()),
            uri: None,
            authority: "local-artifact".to_string(),
        },
        byte_count: bytes.len() as u64,
        sha256: sha256_hex(&bytes),
        captured_at: captured_at.into(),
        producer,
        source_type: source_type.into(),
        retention_hint,
    };
    validate_core_evidence_capture_receipt(&receipt, Some(&bytes)).map_err(|errors| {
        anyhow::anyhow!("invalid evidence capture receipt: {}", errors.join(","))
    })?;
    Ok(receipt)
}

pub fn external_core_evidence_reference_receipt(
    uri: impl Into<String>,
    receipt_id: impl Into<String>,
    producer: CoreEvidenceProducerRef,
    source_type: impl Into<String>,
    sha256: impl Into<String>,
    captured_at: impl Into<String>,
    retention_hint: CoreEvidenceRetentionHint,
) -> CoreEvidenceCaptureReceipt {
    CoreEvidenceCaptureReceipt {
        version: CORE_EVIDENCE_CAPTURE_RECEIPT_VERSION.to_string(),
        receipt_id: receipt_id.into(),
        status: "collected".to_string(),
        location: CoreEvidenceCaptureLocation {
            location_kind: "external-uri".to_string(),
            path: None,
            uri: Some(uri.into()),
            authority: "external-reference".to_string(),
        },
        byte_count: 0,
        sha256: sha256.into(),
        captured_at: captured_at.into(),
        producer,
        source_type: source_type.into(),
        retention_hint,
    }
}

pub fn validate_core_evidence_capture_receipt(
    receipt: &CoreEvidenceCaptureReceipt,
    expected_bytes: Option<&[u8]>,
) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    if receipt.version != CORE_EVIDENCE_CAPTURE_RECEIPT_VERSION {
        errors.push(reason("receipt-version-mismatch"));
    }
    if receipt.receipt_id.trim().is_empty() {
        errors.push(reason("receipt-id-missing"));
    }
    if receipt.status != "collected" {
        errors.push(reason("receipt-status-unsupported"));
    }
    if receipt.captured_at.trim().is_empty() {
        errors.push(reason("receipt-captured-at-missing"));
    }
    if let Some(expires_at) = &receipt.retention_hint.expires_at {
        if expires_at <= &receipt.captured_at {
            errors.push(reason("receipt-stale"));
        }
    }
    if receipt.retention_hint.retention_class.trim().is_empty() {
        errors.push(reason("receipt-retention-class-missing"));
    }
    if receipt.producer.actor_ref.trim().is_empty() {
        errors.push(reason("receipt-producer-actor-ref-missing"));
    }
    if receipt.producer.role_ref.trim().is_empty() {
        errors.push(reason("receipt-producer-role-ref-missing"));
    }
    if receipt.source_type.trim().is_empty() {
        errors.push(reason("receipt-source-type-missing"));
    }
    validate_source_type_known(&receipt.source_type, &mut errors);
    validate_sha256_string("receipt-sha256", &receipt.sha256, &mut errors);

    match receipt.location.location_kind.as_str() {
        "local-path" => {
            if receipt
                .location
                .path
                .as_deref()
                .unwrap_or_default()
                .trim()
                .is_empty()
            {
                errors.push(reason("receipt-local-path-missing"));
            }
            if receipt.location.uri.is_some() {
                errors.push(reason("receipt-local-uri-not-allowed"));
            }
            if receipt.location.authority != "local-artifact" {
                errors.push(reason("receipt-local-authority-invalid"));
            }
            if receipt.byte_count == 0 {
                errors.push(reason("receipt-artifact-empty"));
            }
            if let Some(bytes) = expected_bytes {
                if bytes.is_empty() {
                    errors.push(reason("receipt-artifact-empty"));
                }
                if receipt.byte_count != bytes.len() as u64 {
                    errors.push(reason("receipt-byte-count-mismatch"));
                }
                if receipt.sha256 != sha256_hex(bytes) {
                    errors.push(reason("receipt-sha256-mismatch"));
                }
            }
        }
        "external-uri" => {
            if receipt
                .location
                .uri
                .as_deref()
                .unwrap_or_default()
                .trim()
                .is_empty()
            {
                errors.push(reason("receipt-external-uri-missing"));
            }
            if receipt.location.path.is_some() {
                errors.push(reason("receipt-external-path-not-allowed"));
            }
            if receipt.location.authority != "external-reference" {
                errors.push(reason("receipt-external-authority-invalid"));
            }
            if expected_bytes.is_some() {
                errors.push(reason("receipt-external-bytes-not-local-authority"));
            }
        }
        _ => errors.push(reason("receipt-location-kind-unsupported")),
    }

    if errors.is_empty() {
        Ok(())
    } else {
        errors.sort();
        errors.dedup();
        Err(errors)
    }
}

pub fn core_evidence_capture_receipt_negative_fixtures(
) -> Vec<CoreEvidenceCaptureReceiptNegativeFixtureResult> {
    let bytes = b"canonical evidence bytes";
    let fixtures = vec![
        ("missing-digest", "receipt-sha256-missing", {
            let mut receipt = canonical_core_evidence_capture_receipt_fixture(bytes);
            receipt.sha256.clear();
            (receipt, Some(bytes.as_slice()))
        }),
        ("empty-artifact", "receipt-artifact-empty", {
            let mut receipt = canonical_core_evidence_capture_receipt_fixture(b"");
            receipt.byte_count = 0;
            (receipt, Some(&[] as &[u8]))
        }),
        ("wrong-digest", "receipt-sha256-mismatch", {
            let mut receipt = canonical_core_evidence_capture_receipt_fixture(bytes);
            receipt.sha256 =
                "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff".to_string();
            (receipt, Some(bytes.as_slice()))
        }),
        ("stale-receipt", "receipt-stale", {
            let mut receipt = canonical_core_evidence_capture_receipt_fixture(bytes);
            receipt.retention_hint.expires_at = Some("2026-06-29T00:00:00Z".to_string());
            receipt.captured_at = "2026-06-29T00:00:01Z".to_string();
            (receipt, Some(bytes.as_slice()))
        }),
    ];

    fixtures
        .into_iter()
        .map(|(fixture_id, expected_reason, (receipt, expected_bytes))| {
            let reasons =
                validate_core_evidence_capture_receipt(&receipt, expected_bytes).unwrap_err();
            CoreEvidenceCaptureReceiptNegativeFixtureResult {
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

pub fn canonical_core_evidence_capture_receipt_fixture(bytes: &[u8]) -> CoreEvidenceCaptureReceipt {
    CoreEvidenceCaptureReceipt {
        version: CORE_EVIDENCE_CAPTURE_RECEIPT_VERSION.to_string(),
        receipt_id: "receipt-core-canonical-001".to_string(),
        status: "collected".to_string(),
        location: CoreEvidenceCaptureLocation {
            location_kind: "local-path".to_string(),
            path: Some(".agentflow/tasks/task-core/evidence/evidence.log".to_string()),
            uri: None,
            authority: "local-artifact".to_string(),
        },
        byte_count: bytes.len() as u64,
        sha256: sha256_hex(bytes),
        captured_at: "2026-06-29T00:00:01Z".to_string(),
        producer: CoreEvidenceProducerRef {
            actor_ref: "actor:work-agent".to_string(),
            role_ref: "role:work".to_string(),
            tool_ref: Some("tool:evidence-capture".to_string()),
            produced_at: "2026-06-29T00:00:01Z".to_string(),
        },
        source_type: "artifact".to_string(),
        retention_hint: CoreEvidenceRetentionHint {
            retention_class: "release-certification".to_string(),
            expires_at: Some("2026-12-31T00:00:00Z".to_string()),
        },
    }
}

pub fn canonical_core_evidence_authority_trace_fixture() -> CoreEvidenceAuthorityTrace {
    let pack = canonical_core_evidence_pack_fixture();
    CoreEvidenceAuthorityTrace {
        version: CORE_EVIDENCE_AUTHORITY_TRACE_VERSION.to_string(),
        status: "linked".to_string(),
        evidence_id: pack.evidence_id.clone(),
        trace_refs: CoreEvidenceTraceRefs {
            spec_refs: vec!["spec:core-evidence-pack".to_string()],
            goal_refs: vec!["goal:evidence-kernel".to_string()],
            task_refs: vec!["task:core-evidence-pack".to_string()],
            run_refs: vec!["run:core-evidence-pack".to_string()],
            action_refs: vec![
                "action-proposal:attach-evidence".to_string(),
                "accepted-action:attach-evidence".to_string(),
            ],
            decision_refs: vec!["decision:accept-evidence".to_string()],
        },
        authority_facts: vec![
            authority_fact(
                "SpecBundle",
                "spec:core-evidence-pack",
                "docs/requirements/core-evidence-pack.md",
            ),
            authority_fact(
                "Task",
                "task:core-evidence-pack",
                ".agentflow/spec/issues/task-core-evidence-pack.json",
            ),
            authority_fact(
                "Run",
                "run:core-evidence-pack",
                ".agentflow/tasks/task-core-evidence-pack/run/run.json",
            ),
            authority_fact(
                "ActionProposal",
                "action-proposal:attach-evidence",
                ".agentflow/tasks/task-core-evidence-pack/action-proposals/attach-evidence.json",
            ),
            authority_fact(
                "AcceptedAction",
                "accepted-action:attach-evidence",
                ".agentflow/tasks/task-core-evidence-pack/actions/attach-evidence.accepted.json",
            ),
        ],
        collection_event: CoreEvidenceCollectionEventLink {
            event_type: "evidence.collected".to_string(),
            event_ref: "event:evidence-collected-001".to_string(),
            event_store_path: ".agentflow/events/task-events.jsonl".to_string(),
            correlation_id: "corr:core-evidence-pack".to_string(),
            causation_id: "action-proposal:attach-evidence".to_string(),
        },
    }
}

pub fn validate_core_evidence_authority_trace(
    pack: &CoreEvidencePack,
    trace: &CoreEvidenceAuthorityTrace,
) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();
    if let Err(schema_errors) = validate_core_evidence_pack_schema(pack) {
        errors.extend(schema_errors);
    }
    if trace.version != CORE_EVIDENCE_AUTHORITY_TRACE_VERSION {
        errors.push(reason("evidence-trace-version-mismatch"));
    }
    if trace.status != "linked" {
        errors.push(reason("evidence-trace-status-unsupported"));
    }
    if trace.evidence_id != pack.evidence_id {
        errors.push(reason("evidence-trace-evidence-id-mismatch"));
    }
    validate_trace_refs(&trace.trace_refs, &mut errors);

    let fact_refs = trace
        .authority_facts
        .iter()
        .map(|fact| fact.fact_ref.as_str())
        .collect::<BTreeSet<_>>();
    for required_ref in trace
        .trace_refs
        .spec_refs
        .iter()
        .chain(trace.trace_refs.task_refs.iter())
        .chain(trace.trace_refs.run_refs.iter())
        .chain(trace.trace_refs.action_refs.iter())
    {
        if !fact_refs.contains(required_ref.as_str()) {
            errors.push(reason("evidence-trace-orphaned"));
        }
    }

    for required_kind in [
        "SpecBundle",
        "Task",
        "Run",
        "ActionProposal",
        "AcceptedAction",
    ] {
        if trace
            .authority_facts
            .iter()
            .all(|fact| fact.fact_kind != required_kind)
        {
            errors.push(reason(&format!(
                "evidence-trace-authority-kind-missing:{required_kind}"
            )));
        }
    }

    for fact in &trace.authority_facts {
        if fact.fact_kind.trim().is_empty()
            || fact.fact_ref.trim().is_empty()
            || fact.authority_path.trim().is_empty()
        {
            errors.push(reason("evidence-trace-authority-fact-incomplete"));
        }
    }

    if trace.collection_event.event_type != "evidence.collected" {
        errors.push(reason("evidence-collection-event-type-invalid"));
    }
    if trace.collection_event.event_ref.trim().is_empty()
        || trace.collection_event.event_store_path.trim().is_empty()
        || trace.collection_event.correlation_id.trim().is_empty()
        || trace.collection_event.causation_id.trim().is_empty()
    {
        errors.push(reason("evidence-collection-event-link-missing"));
    }

    if errors.is_empty() {
        Ok(())
    } else {
        errors.sort();
        errors.dedup();
        Err(errors)
    }
}

pub fn core_evidence_authority_trace_negative_fixtures(
) -> Vec<CoreEvidenceAuthorityTraceNegativeFixtureResult> {
    let pack = canonical_core_evidence_pack_fixture();
    let fixtures = vec![
        ("orphan-evidence", "evidence-trace-orphaned", {
            let mut trace = canonical_core_evidence_authority_trace_fixture();
            trace
                .authority_facts
                .retain(|fact| fact.fact_ref != "run:core-evidence-pack");
            trace
        }),
        (
            "missing-action-proposal",
            "evidence-trace-authority-kind-missing:ActionProposal",
            {
                let mut trace = canonical_core_evidence_authority_trace_fixture();
                trace
                    .authority_facts
                    .retain(|fact| fact.fact_kind != "ActionProposal");
                trace
            },
        ),
        (
            "missing-collection-event",
            "evidence-collection-event-link-missing",
            {
                let mut trace = canonical_core_evidence_authority_trace_fixture();
                trace.collection_event.event_ref.clear();
                trace
            },
        ),
    ];

    fixtures
        .into_iter()
        .map(|(fixture_id, expected_reason, trace)| {
            let reasons = validate_core_evidence_authority_trace(&pack, &trace).unwrap_err();
            CoreEvidenceAuthorityTraceNegativeFixtureResult {
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

fn authority_fact(
    fact_kind: &str,
    fact_ref: &str,
    authority_path: &str,
) -> CoreEvidenceAuthorityFactRef {
    CoreEvidenceAuthorityFactRef {
        fact_kind: fact_kind.to_string(),
        fact_ref: fact_ref.to_string(),
        authority_path: authority_path.to_string(),
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
    if !is_valid_sha256_hex(&digest.value) {
        errors.push(reason(&format!("{path}-value-invalid")));
    }
}

fn validate_sha256_string(path: &str, value: &str, errors: &mut Vec<String>) {
    if value.trim().is_empty() {
        errors.push(reason(&format!("{path}-missing")));
    } else if !is_valid_sha256_hex(value) {
        errors.push(reason(&format!("{path}-invalid")));
    }
}

fn validate_source_type_known(source_type: &str, errors: &mut Vec<String>) {
    let registry = core_evidence_source_type_registry_contract();
    if registry
        .source_types
        .iter()
        .all(|definition| definition.source_type != source_type)
    {
        errors.push(reason("receipt-source-type-unknown"));
    }
}

fn is_valid_sha256_hex(value: &str) -> bool {
    value.len() == 64 && value.chars().all(|character| character.is_ascii_hexdigit())
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
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

    #[test]
    fn core_evidence_capture_receipt_can_be_generated_for_local_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        let artifact_path = temp_dir.path().join("artifact.log");
        std::fs::write(&artifact_path, b"receipt bytes").unwrap();

        let receipt = capture_core_evidence_receipt_for_local_file(
            &artifact_path,
            "receipt-local-001",
            CoreEvidenceProducerRef {
                actor_ref: "actor:work-agent".to_string(),
                role_ref: "role:work".to_string(),
                tool_ref: Some("tool:evidence-capture".to_string()),
                produced_at: "2026-06-29T00:00:01Z".to_string(),
            },
            "artifact",
            "2026-06-29T00:00:01Z",
            CoreEvidenceRetentionHint {
                retention_class: "release-certification".to_string(),
                expires_at: Some("2026-12-31T00:00:00Z".to_string()),
            },
        )
        .unwrap();

        assert_eq!(receipt.version, CORE_EVIDENCE_CAPTURE_RECEIPT_VERSION);
        assert_eq!(receipt.byte_count, 13);
        assert_eq!(receipt.sha256.len(), 64);
        assert_eq!(receipt.location.authority, "local-artifact");
    }

    #[test]
    fn core_evidence_capture_receipt_rejects_digest_mismatch() {
        let bytes = b"receipt bytes";
        let mut receipt = canonical_core_evidence_capture_receipt_fixture(bytes);
        receipt.sha256 =
            "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff".to_string();

        let errors = validate_core_evidence_capture_receipt(&receipt, Some(bytes)).unwrap_err();
        assert!(errors
            .iter()
            .any(|error| error == "receipt-sha256-mismatch"));
    }

    #[test]
    fn core_evidence_capture_receipt_allows_external_reference_without_local_bytes() {
        let receipt = external_core_evidence_reference_receipt(
            "https://example.invalid/proof/123",
            "receipt-external-001",
            CoreEvidenceProducerRef {
                actor_ref: "actor:work-agent".to_string(),
                role_ref: "role:work".to_string(),
                tool_ref: Some("tool:evidence-capture".to_string()),
                produced_at: "2026-06-29T00:00:01Z".to_string(),
            },
            "external-proof",
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "2026-06-29T00:00:01Z",
            CoreEvidenceRetentionHint {
                retention_class: "external-reference".to_string(),
                expires_at: Some("2026-12-31T00:00:00Z".to_string()),
            },
        );

        validate_core_evidence_capture_receipt(&receipt, None).unwrap();
        assert_eq!(receipt.location.authority, "external-reference");
        assert_eq!(receipt.byte_count, 0);
    }

    #[test]
    fn core_evidence_capture_receipt_negative_fixtures_fail_with_stable_reasons() {
        let fixtures = core_evidence_capture_receipt_negative_fixtures();
        assert_eq!(fixtures.len(), 4);
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
    fn core_evidence_authority_trace_fixture_links_to_runtime_authority() {
        let pack = canonical_core_evidence_pack_fixture();
        let trace = canonical_core_evidence_authority_trace_fixture();

        validate_core_evidence_authority_trace(&pack, &trace).unwrap();
        assert_eq!(trace.version, CORE_EVIDENCE_AUTHORITY_TRACE_VERSION);
        assert_eq!(trace.collection_event.event_type, "evidence.collected");
        assert!(trace
            .authority_facts
            .iter()
            .any(|fact| fact.fact_kind == "ActionProposal"));
        assert!(trace
            .authority_facts
            .iter()
            .any(|fact| fact.fact_kind == "AcceptedAction"));
    }

    #[test]
    fn core_evidence_authority_trace_rejects_orphan_evidence() {
        let pack = canonical_core_evidence_pack_fixture();
        let mut trace = canonical_core_evidence_authority_trace_fixture();
        trace
            .authority_facts
            .retain(|fact| fact.fact_ref != "task:core-evidence-pack");

        let errors = validate_core_evidence_authority_trace(&pack, &trace).unwrap_err();
        assert!(errors
            .iter()
            .any(|error| error == "evidence-trace-orphaned"));
    }

    #[test]
    fn core_evidence_authority_trace_negative_fixtures_fail_with_stable_reasons() {
        let fixtures = core_evidence_authority_trace_negative_fixtures();
        assert_eq!(fixtures.len(), 3);
        for fixture in fixtures {
            assert_eq!(
                fixture.status, "passed",
                "fixture {} failed with {:?}",
                fixture.fixture_id, fixture.reasons
            );
            assert!(fixture.reasons.contains(&fixture.expected_reason));
        }
    }
}
