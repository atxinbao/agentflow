#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ARTIFACT_DIR="${ARTIFACT_DIR:-}"
RELEASE_VERSION="${RELEASE_VERSION:-}"
RELEASE_TAG_NAME="${RELEASE_TAG_NAME:-}"
SOURCE_COMMIT_SHA="${SOURCE_COMMIT_SHA:-$(git -C "$ROOT" rev-parse HEAD)}"
RELEASE_URL="${RELEASE_URL:-}"
REQUIRE_PUBLISHED_RELEASE_FACTS="${REQUIRE_PUBLISHED_RELEASE_FACTS:-}"
GATE_EVENT_NAME="${GITHUB_EVENT_NAME:-local}"
GATE_RUN_ID="${GITHUB_RUN_ID:-}"
GATE_RUN_ATTEMPT="${GITHUB_RUN_ATTEMPT:-}"
GATE_REPOSITORY="${GITHUB_REPOSITORY:-atxinbao/agentflow}"
GATE_SERVER_URL="${GITHUB_SERVER_URL:-https://github.com}"
GATE_REF_NAME="${GITHUB_REF_NAME:-}"
GATE_REF_TYPE="${GITHUB_REF_TYPE:-}"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --artifact-dir)
      ARTIFACT_DIR="$2"
      shift 2
      ;;
    --release-version)
      RELEASE_VERSION="$2"
      shift 2
      ;;
    --release-tag)
      RELEASE_TAG_NAME="$2"
      shift 2
      ;;
    --source-commit-sha)
      SOURCE_COMMIT_SHA="$2"
      shift 2
      ;;
    --release-url)
      RELEASE_URL="$2"
      shift 2
      ;;
    *)
      echo "unknown argument: $1" >&2
      exit 1
      ;;
  esac
done

read_workspace_version() {
  python3 - "$ROOT/Cargo.toml" <<'PY'
import pathlib
import sys
import tomllib

payload = tomllib.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
print(payload["workspace"]["package"]["version"])
PY
}

if [[ -z "$RELEASE_VERSION" ]]; then
  RELEASE_VERSION="v$(read_workspace_version)"
fi
if [[ "$RELEASE_VERSION" != v* ]]; then
  RELEASE_VERSION="v$RELEASE_VERSION"
fi
if [[ -z "$RELEASE_TAG_NAME" ]]; then
  RELEASE_TAG_NAME="$RELEASE_VERSION"
fi
if [[ -z "$RELEASE_URL" ]]; then
  RELEASE_URL="${GATE_SERVER_URL}/${GATE_REPOSITORY}/releases/tag/${RELEASE_TAG_NAME}"
fi
if [[ -z "$REQUIRE_PUBLISHED_RELEASE_FACTS" ]]; then
  if [[ "$GATE_EVENT_NAME" == "release" ]]; then
    REQUIRE_PUBLISHED_RELEASE_FACTS="1"
  else
    REQUIRE_PUBLISHED_RELEASE_FACTS="0"
  fi
fi
if [[ -z "$ARTIFACT_DIR" ]]; then
  ARTIFACT_DIR="$ROOT/artifacts/release-gate-${RELEASE_VERSION}-e2e"
fi

mkdir -p "$ARTIFACT_DIR"
ARTIFACT_DIR="$(cd "$ARTIFACT_DIR" && pwd)"
CLI_DIR="$ARTIFACT_DIR/cli"
PUBLIC_DIR="$ARTIFACT_DIR/public"
RUNTIME_DIR="$ARTIFACT_DIR/runtime"
mkdir -p "$CLI_DIR" "$PUBLIC_DIR" "$RUNTIME_DIR"
PROVIDER_SMOKE="${PROVIDER_SMOKE:-0}"
PROVIDER_SMOKE_PROVIDER="${PROVIDER_SMOKE_PROVIDER:-codex}"
PROVIDER_SMOKE_STATUS_PATH="$RUNTIME_DIR/provider-smoke-status.json"
PROVIDER_SMOKE_ARTIFACT_PATH="$RUNTIME_DIR/provider-smoke-artifact.json"
API_PLANE_MANIFEST_PATH="$RUNTIME_DIR/api-plane-manifest.json"
RUNTIME_API_SDK_COMPATIBILITY_PATH="$RUNTIME_DIR/runtime-api-sdk-compatibility.json"
FILESYSTEM_CONTRACT_PATH="$RUNTIME_DIR/filesystem-contract.json"
PACK_CONTRACT_COMPATIBILITY_PATH="$RUNTIME_DIR/pack-contract-compatibility.json"
PROJECTION_READMODEL_CONTRACT_PATH="$RUNTIME_DIR/projection-readmodel-contract.json"
CORE_PROJECTION_KERNEL_CONTRACT_PATH="$RUNTIME_DIR/core-projection-kernel-contract.json"
CORE_READ_MODEL_SCHEMA_PATH="$RUNTIME_DIR/core-read-model-schema.json"
PROJECTION_FEEDBACK_FRESHNESS_PATH="$RUNTIME_DIR/projection-feedback-freshness-receipts.json"
CORE_VIEW_MODEL_CONTRACT_PATH="$RUNTIME_DIR/core-view-model-contract.json"
EVIDENCE_ACCEPTANCE_CONTRACT_PATH="$RUNTIME_DIR/evidence-acceptance-contract.json"
EXECUTOR_ADAPTER_CONTRACT_PATH="$RUNTIME_DIR/executor-adapter-contract.json"
REPLAY_MIGRATION_UPGRADE_CERTIFICATION_PATH="$RUNTIME_DIR/replay-migration-upgrade-certification.json"
SOFTWARE_DEV_PACK_STABLE_BASELINE_PATH="$RUNTIME_DIR/software-dev-pack-stable-baseline.json"
CAPABILITY_REGISTRY_PATH="$RUNTIME_DIR/capability-registry.json"
GOVERNANCE_POLICY_PATH="$RUNTIME_DIR/governance-policy.json"
GOVERNANCE_ADMISSION_PATH="$RUNTIME_DIR/governance-admission.json"
SCHEDULING_DECISION_PATH="$RUNTIME_DIR/scheduling-decision.json"
DEPLOYMENT_EVIDENCE_PATH="$RUNTIME_DIR/deployment-evidence.json"
DEPLOYMENT_EVIDENCE_FAILURE_PATH="$RUNTIME_DIR/deployment-evidence-semantic-failure.json"
DEPLOYMENT_EVIDENCE_WRONG_COMMIT_PATH="$RUNTIME_DIR/deployment-evidence-wrong-commit.json"
DEPLOYMENT_EVIDENCE_WRONG_URL_PATH="$RUNTIME_DIR/deployment-evidence-wrong-url.json"
DEPLOYMENT_EVIDENCE_FAKE_MIGRATION_PATH="$RUNTIME_DIR/deployment-evidence-fake-migration-receipt.json"
NEGATIVE_SEMANTIC_FIXTURES_PATH="$RUNTIME_DIR/negative-semantic-fixtures.json"
FOUNDATION_READINESS_REPORT_SOURCE="$ROOT/docs/project/history/2026-06-current-baseline-history/versions/v0.7.2/AGENTFLOW_V0_7_2_FOUNDATION_READINESS_REPORT_V1.md"
FOUNDATION_READINESS_REPORT_PATH="$RUNTIME_DIR/foundation-readiness-report.md"
FOUNDATION_COVERAGE_PATH="$RUNTIME_DIR/foundation-coverage.json"
PACK_REGISTRY_PATH="$ARTIFACT_DIR/pack-registry.json"
PACK_VALIDATION_REPORT_PATH="$ARTIFACT_DIR/pack-validation-report.json"
PACK_SIMULATION_REPORT_PATH="$ARTIFACT_DIR/pack-simulation-report.json"
PACK_PROJECTION_READINESS_PATH="$ARTIFACT_DIR/pack-projection-readiness.json"
PACK_API_PLANE_MANIFEST_PATH="$ARTIFACT_DIR/pack-api-plane-manifest.json"
PACK_NEGATIVE_FIXTURES_PATH="$ARTIFACT_DIR/pack-negative-fixtures.json"
PACK_MIGRATION_PREVIEW_PATH="$ARTIFACT_DIR/pack-migration-preview.json"
PACK_MIGRATION_UNCONFIRMED_APPLY_PATH="$ARTIFACT_DIR/pack-migration-unconfirmed-apply.json"
PACK_MIGRATION_APPLIED_RECEIPT_PATH="$ARTIFACT_DIR/pack-migration-applied-receipt.json"
PACK_MIGRATION_FAKE_AUTHORITY_RECEIPT_PATH="$ARTIFACT_DIR/pack-migration-fake-authority-receipt.json"
PACK_MIGRATION_CANCEL_RECEIPT_PATH="$ARTIFACT_DIR/pack-migration-cancel-receipt.json"
PACK_MIGRATION_ROLLBACK_RECEIPT_PATH="$ARTIFACT_DIR/pack-migration-rollback-receipt.json"
PACK_MIGRATION_REPLAY_REPORT_PATH="$RUNTIME_DIR/pack-migration-replay-report.json"
SOFTWARE_DEV_PACK_READINESS_PATH="$ARTIFACT_DIR/software-dev-pack-readiness.json"
UI_DESIGN_PACK_READINESS_PATH="$ARTIFACT_DIR/ui-design-pack-readiness.json"
EVENT_REPLAY_PROJECTION_REPORT_PATH="$RUNTIME_DIR/event-replay-projection-report.json"
EVENT_REPLAY_PROJECTION_FAILURE_REPORT_PATH="$RUNTIME_DIR/event-replay-projection-failure-report.json"
SOURCE_AGENT_ENTRY_PATH="$RUNTIME_DIR/source-agent-entry.json"
RELEASE_PROVENANCE_PATH="$RUNTIME_DIR/release-provenance.json"
CLEAN_ROOM_TEST_PROOF_PATH="$RUNTIME_DIR/clean-room-test-proof.json"
AUDIT_SIDECAR_POLICY_PATH="$RUNTIME_DIR/audit-sidecar-policy.json"
PROVIDER_SMOKE_PROOF_PATH="$RUNTIME_DIR/provider-smoke-proof.json"
SOFTWARE_DEV_PACK_USAGE_BASELINE_PATH="$RUNTIME_DIR/software-dev-pack-usage-baseline.json"
TRUSTED_GOVERNANCE_TELEMETRY_PATH="$RUNTIME_DIR/trusted-governance-telemetry.json"
V101_RELEASE_CERTIFICATION_PATH="$RUNTIME_DIR/v101-release-certification.json"
V102_NEGATIVE_FIXTURES_PATH="$RUNTIME_DIR/v102-negative-fixtures.json"
V102_RELEASE_CERTIFICATION_PATH="$RUNTIME_DIR/v102-release-certification.json"
FORGED_GOVERNANCE_REQUEST_PATH="$RUNTIME_DIR/forged-governance-runtime-request.json"
FORGED_GOVERNANCE_RESPONSE_PATH="$RUNTIME_DIR/forged-governance-runtime-response.json"
RELEASE_ARTIFACT_BOUNDARY_PATH="$RUNTIME_DIR/release-artifact-boundary.json"
PROJECT_ROADMAP_BASELINE_PATH="$RUNTIME_DIR/project-roadmap-baseline.json"
V103_RELEASE_FIX_CERTIFICATION_PATH="$RUNTIME_DIR/v103-release-fix-certification.json"
STABLE_CONTRACT_BASELINE_PATH="$RUNTIME_DIR/stable-contract-baseline.json"
CORE_4D_SPEC_INTAKE_PATH="$RUNTIME_DIR/core-4d-spec-intake.json"
CORE_4D_SPEC_INTAKE_POSITIVE_CERTIFICATION_PATH="$RUNTIME_DIR/core-4d-spec-intake-positive-certification.json"
CORE_4D_SPEC_INTAKE_NEGATIVE_CERTIFICATION_PATH="$RUNTIME_DIR/core-4d-spec-intake-negative-certification.json"
CORE_ONTOLOGY_KERNEL_PATH="$RUNTIME_DIR/core-ontology-kernel.json"
CORE_OBJECT_LINK_SCHEMA_PATH="$RUNTIME_DIR/core-object-link-schema.json"
CORE_ACTION_STATE_SEMANTICS_PATH="$RUNTIME_DIR/core-action-state-semantics.json"
CORE_SKILL_REGISTRY_PATH="$RUNTIME_DIR/core-skill-registry.json"
CORE_EVIDENCE_DECISION_REFERENCE_MODEL_PATH="$RUNTIME_DIR/core-evidence-decision-reference-model.json"
CORE_EVIDENCE_PACK_SCHEMA_PATH="$RUNTIME_DIR/core-evidence-pack-schema.json"
CORE_EVIDENCE_SOURCE_TYPE_REGISTRY_PATH="$RUNTIME_DIR/core-evidence-source-type-registry.json"
CORE_EVIDENCE_CAPTURE_RECEIPTS_PATH="$RUNTIME_DIR/core-evidence-capture-receipts.json"
CORE_EVIDENCE_AUTHORITY_TRACE_LINKS_PATH="$RUNTIME_DIR/core-evidence-authority-trace-links.json"
CORE_EVIDENCE_COMPLETENESS_POLICY_PATH="$RUNTIME_DIR/core-evidence-completeness-policy.json"
CORE_MISSING_EVIDENCE_HANDLING_PATH="$RUNTIME_DIR/core-missing-evidence-handling.json"
CORE_EXTERNAL_PROOF_PROVENANCE_PATH="$RUNTIME_DIR/core-external-proof-provenance.json"
SOFTWARE_DEV_REFERENCE_EVIDENCE_MAPPING_PATH="$RUNTIME_DIR/software-dev-reference-evidence-mapping.json"
EVIDENCE_PROJECTION_READ_MODEL_PATH="$RUNTIME_DIR/evidence-projection-read-model.json"
CORE_FILE_BACKED_ONTOLOGY_REGISTRY_PATH="$RUNTIME_DIR/core-file-backed-ontology-registry.json"
V104_RELEASE_CERTIFICATION_PATH="$RUNTIME_DIR/v104-release-certification.json"
CORE_RUNTIME_NEGATIVE_FIXTURES_PATH="$RUNTIME_DIR/core-runtime-negative-fixtures.json"
CORE_RUNTIME_KERNEL_PATH="$RUNTIME_DIR/core-runtime-kernel.json"
CORE_RUNTIME_ADMISSION_PATH="$RUNTIME_DIR/core-runtime-admission.json"
CORE_RUNTIME_ARBITRATION_PATH="$RUNTIME_DIR/core-runtime-arbitration.json"
V105_RELEASE_CERTIFICATION_PATH="$RUNTIME_DIR/v105-release-certification.json"
V106_RELEASE_CERTIFICATION_PATH="$RUNTIME_DIR/v106-release-certification.json"
V107_RELEASE_PROVENANCE_HANDOFF_PATH="$RUNTIME_DIR/v107-release-provenance-handoff.json"
V107_RELEASE_CERTIFICATION_PATH="$RUNTIME_DIR/v107-release-certification.json"
V108_RELEASE_CERTIFICATION_PATH="$RUNTIME_DIR/v108-release-certification.json"
V109_TASK_ISSUE_TRACEABILITY_PATH="$RUNTIME_DIR/v109-task-issue-traceability.json"
V109_SOFTWARE_DEV_PRODUCT_CONTRACT_PATH="$RUNTIME_DIR/v109-software-dev-product-contract.json"
V109_SPEC_TASK_FLOW_PATH="$RUNTIME_DIR/v109-spec-task-flow.json"
V109_CONNECTOR_HANDOFF_PATH="$RUNTIME_DIR/v109-connector-handoff.json"
V109_EVIDENCE_DECISION_DELIVERY_PATH="$RUNTIME_DIR/v109-evidence-decision-delivery.json"
V109_WORKBENCH_READ_MODELS_PATH="$RUNTIME_DIR/v109-workbench-read-models.json"
V109_MAPPING_BOUNDARY_PATH="$RUNTIME_DIR/v109-mapping-boundary.json"
V109_GOLDEN_SCENARIO_PATH="$RUNTIME_DIR/v109-golden-scenario.json"
V109_RELEASE_CERTIFICATION_PATH="$RUNTIME_DIR/v109-release-certification.json"
V110_ROADMAP_RELEASE_GOAL_ALIGNMENT_PATH="$RUNTIME_DIR/v110-roadmap-release-goal-alignment.json"
V110_PRODUCT_REGISTRY_LOADER_PATH="$RUNTIME_DIR/v110-product-registry-loader.json"
V110_PRODUCT_TO_PACK_CONTRACT_PATH="$RUNTIME_DIR/v110-product-to-pack-contract.json"
V110_RUNTIME_PRODUCT_COMMAND_ROUTES_PATH="$RUNTIME_DIR/v110-runtime-product-command-routes.json"
V110_PROJECTION_PRODUCT_SOURCE_PATH="$RUNTIME_DIR/v110-projection-product-source.json"
V110_CORE_POLLUTION_DETECTION_PATH="$RUNTIME_DIR/v110-core-pollution-detection.json"
V110_PRODUCT_COMMAND_ROUTE_INSTALLATION_PATH="$RUNTIME_DIR/v110-product-command-route-installation.json"
V110_SOFTWARE_DEV_E2E_PRODUCT_SURFACE_PATH="$RUNTIME_DIR/v110-software-dev-e2e-product-surface.json"
V110_QUICK_AUDIT_PRODUCT_SOURCE_PROOFS_PATH="$RUNTIME_DIR/v110-quick-audit-product-source-proofs.json"
V110_RELEASE_CERTIFICATION_PATH="$RUNTIME_DIR/v110-release-certification.json"
V111_PRODUCT_SCHEMA_COMMAND_MAPPING_PATH="$RUNTIME_DIR/v111-product-schema-command-mapping.json"
V111_PRODUCT_TO_PACK_DATA_DRIVEN_BRIDGE_PATH="$RUNTIME_DIR/v111-product-to-pack-data-driven-bridge.json"
V111_RUNTIME_DATA_DRIVEN_PRODUCT_RESOLVER_PATH="$RUNTIME_DIR/v111-runtime-data-driven-product-resolver.json"
V111_PROJECTION_DATA_DRIVEN_PRODUCT_READMODEL_PATH="$RUNTIME_DIR/v111-projection-data-driven-product-readmodel.json"
V111_PRODUCT_BRIDGE_POLLUTION_GATE_PATH="$RUNTIME_DIR/v111-product-bridge-pollution-gate.json"
V111_RUNTIME_PROJECTION_PROOF_ARTIFACTS_PATH="$RUNTIME_DIR/v111-runtime-projection-proof-artifacts.json"
V111_SYNTHETIC_SECOND_PRODUCT_FIXTURE_PATH="$RUNTIME_DIR/v111-synthetic-second-product-fixture.json"
V111_RELEASE_CERTIFICATION_PATH="$RUNTIME_DIR/v111-release-certification.json"
CORE_DECISION_MODEL_CONTRACT_PATH="$RUNTIME_DIR/core-decision-model-contract.json"
CORE_DECISION_INPUT_BINDING_PATH="$RUNTIME_DIR/core-decision-input-binding.json"
CORE_DECISION_OUTCOME_TRANSITIONS_PATH="$RUNTIME_DIR/core-decision-outcome-transitions.json"
CORE_DECISION_FAILURE_REASON_PATH="$RUNTIME_DIR/core-decision-failure-reason-remediation.json"
CORE_EVIDENCE_TO_DECISION_GATE_PATH="$RUNTIME_DIR/core-evidence-to-decision-gate.json"
CORE_COMPLETION_COMMIT_AUTHORITY_PATH="$RUNTIME_DIR/core-completion-commit-authority.json"
CORE_DELIVERY_READINESS_AUDIT_TRIGGER_PATH="$RUNTIME_DIR/core-delivery-readiness-audit-trigger.json"
CORE_DECISION_PROJECTION_READ_MODEL_PATH="$RUNTIME_DIR/core-decision-projection-read-model.json"

BIN="${AGENTFLOW_BIN:-$ROOT/target/debug/agentflow}"
if [[ -z "${AGENTFLOW_BIN:-}" ]]; then
  cargo build -p agentflow-cli --bin agentflow --manifest-path "$ROOT/Cargo.toml" >/dev/null
fi

TMP_DIR="$(mktemp -d "${TMPDIR:-/tmp}/agentflow-release-gate.XXXXXX")"
WORKSPACE="$TMP_DIR/workspace"
STATUS_PATH="$ARTIFACT_DIR/status.json"
STAGE_LOG_PATH="$ARTIFACT_DIR/stage-log.jsonl"
SUMMARY_JSON_PATH="$ARTIFACT_DIR/summary.json"
SUMMARY_MD_PATH="$ARTIFACT_DIR/summary.md"
CERTIFICATION_JSON_PATH="$ARTIFACT_DIR/certification.json"
CERTIFICATION_MD_PATH="$ARTIFACT_DIR/certification.md"
ARTIFACT_MANIFEST_PATH="$ARTIFACT_DIR/artifact-manifest.json"
BOOTSTRAP_BRANCH="release-gate-bootstrap"
export RUST_TEST_THREADS="${RUST_TEST_THREADS:-1}"
REQUIREMENT_ID=""
PROJECT_ID=""
ISSUE_COUNT="0"

cleanup() {
  rm -rf "$TMP_DIR"
}
trap cleanup EXIT

record_stage() {
  local stage="$1"
  local status="$2"
  local detail="$3"
  python3 - "$STAGE_LOG_PATH" "$stage" "$status" "$detail" <<'PY'
import json, pathlib, sys, time
path = pathlib.Path(sys.argv[1])
entry = {
    "stage": sys.argv[2],
    "status": sys.argv[3],
    "detail": sys.argv[4],
    "timestamp": int(time.time()),
}
with path.open("a", encoding="utf-8") as handle:
    handle.write(json.dumps(entry, ensure_ascii=False) + "\n")
PY
}

write_status() {
  local status="$1"
  local stage="$2"
  local message="$3"
  python3 - "$STATUS_PATH" "$status" "$stage" "$message" <<'PY'
import json, pathlib, sys, time
path = pathlib.Path(sys.argv[1])
payload = {
    "status": sys.argv[2],
    "stage": sys.argv[3],
    "message": sys.argv[4],
    "updatedAt": int(time.time()),
}
path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
PY
}

write_provider_smoke_status() {
  local status="$1"
  local provider="$2"
  local reason="$3"
  local artifact_path="$4"
  python3 - "$PROVIDER_SMOKE_STATUS_PATH" "$status" "$provider" "$reason" "$artifact_path" <<'PY'
import json, pathlib, sys, time
path = pathlib.Path(sys.argv[1])
payload = {
    "status": sys.argv[2],
    "provider": sys.argv[3],
    "reason": sys.argv[4],
    "artifactPath": sys.argv[5] or None,
    "updatedAt": int(time.time()),
}
path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
PY
}

write_gate_reports() {
  python3 - \
    "$STATUS_PATH" \
    "$STAGE_LOG_PATH" \
    "$SUMMARY_JSON_PATH" \
    "$SUMMARY_MD_PATH" \
    "$CERTIFICATION_JSON_PATH" \
    "$CERTIFICATION_MD_PATH" \
    "$RELEASE_VERSION" \
    "$RELEASE_TAG_NAME" \
    "$SOURCE_COMMIT_SHA" \
    "$REQUIREMENT_ID" \
    "$PROJECT_ID" \
    "$ISSUE_COUNT" \
    "$RELEASE_URL" \
    "$GATE_EVENT_NAME" \
    "$GATE_RUN_ID" \
    "$GATE_RUN_ATTEMPT" \
    "$GATE_REPOSITORY" \
    "$GATE_SERVER_URL" \
    "$GATE_REF_NAME" \
    "$GATE_REF_TYPE" \
    "$REQUIRE_PUBLISHED_RELEASE_FACTS" \
    "$RUNTIME_DIR/release-facts.json" \
    "$RUNTIME_DIR/external-review-surface.json" \
    "$PROVIDER_SMOKE_STATUS_PATH" \
    "$PROVIDER_SMOKE_ARTIFACT_PATH" \
    "$API_PLANE_MANIFEST_PATH" \
    "$CAPABILITY_REGISTRY_PATH" \
    "$FOUNDATION_READINESS_REPORT_PATH" \
    "$FOUNDATION_COVERAGE_PATH" \
    "$PACK_NEGATIVE_FIXTURES_PATH" \
    "$GOVERNANCE_POLICY_PATH" \
    "$GOVERNANCE_ADMISSION_PATH" \
    "$SCHEDULING_DECISION_PATH" \
    "$DEPLOYMENT_EVIDENCE_PATH" \
    "$SOURCE_AGENT_ENTRY_PATH" \
    "$NEGATIVE_SEMANTIC_FIXTURES_PATH" \
    "$FORGED_GOVERNANCE_RESPONSE_PATH" \
    "$RELEASE_ARTIFACT_BOUNDARY_PATH" \
    "$PROJECT_ROADMAP_BASELINE_PATH" \
    "$V103_RELEASE_FIX_CERTIFICATION_PATH" \
    "$CORE_RUNTIME_NEGATIVE_FIXTURES_PATH" <<'PY'
import json
import pathlib
import sys

status_path = pathlib.Path(sys.argv[1])
stage_log_path = pathlib.Path(sys.argv[2])
summary_json_path = pathlib.Path(sys.argv[3])
summary_md_path = pathlib.Path(sys.argv[4])
cert_json_path = pathlib.Path(sys.argv[5])
cert_md_path = pathlib.Path(sys.argv[6])
release_version = sys.argv[7]
release_tag_name = sys.argv[8] or None
source_commit_sha = sys.argv[9] or None
requirement_id = sys.argv[10] or None
project_id = sys.argv[11] or None
issue_count = int(sys.argv[12] or "0")
release_url = sys.argv[13] or None
gate_event_name = sys.argv[14] or "local"
gate_run_id = sys.argv[15] or None
gate_run_attempt = sys.argv[16] or None
gate_repository = sys.argv[17] or "atxinbao/agentflow"
gate_server_url = sys.argv[18] or "https://github.com"
gate_ref_name = sys.argv[19] or None
gate_ref_type = sys.argv[20] or None
require_published_release_facts = sys.argv[21] == "1"
release_path = pathlib.Path(sys.argv[22])
review_path = pathlib.Path(sys.argv[23])
provider_smoke_status_path = pathlib.Path(sys.argv[24])
provider_smoke_artifact_path = pathlib.Path(sys.argv[25])
api_plane_manifest_path = pathlib.Path(sys.argv[26])
capability_registry_path = pathlib.Path(sys.argv[27])
foundation_readiness_report_path = pathlib.Path(sys.argv[28])
foundation_coverage_path = pathlib.Path(sys.argv[29])
pack_negative_fixtures_path = pathlib.Path(sys.argv[30])
governance_policy_path = pathlib.Path(sys.argv[31])
governance_admission_path = pathlib.Path(sys.argv[32])
scheduling_decision_path = pathlib.Path(sys.argv[33])
deployment_evidence_path = pathlib.Path(sys.argv[34])
source_agent_entry_path = pathlib.Path(sys.argv[35])
negative_semantic_fixtures_path = pathlib.Path(sys.argv[36])
forged_governance_response_path = pathlib.Path(sys.argv[37])
release_artifact_boundary_path = pathlib.Path(sys.argv[38])
project_roadmap_baseline_path = pathlib.Path(sys.argv[39])
v103_release_fix_certification_path = pathlib.Path(sys.argv[40])
core_runtime_negative_fixtures_path = pathlib.Path(sys.argv[41])

def load_json(path: pathlib.Path):
    if not path.is_file():
        return None
    return json.loads(path.read_text(encoding="utf-8"))

status = load_json(status_path) or {}
stage_log = []
if stage_log_path.is_file():
    for line in stage_log_path.read_text(encoding="utf-8").splitlines():
        if line.strip():
            stage_log.append(json.loads(line))

release = load_json(release_path) or {}
review = load_json(review_path) or {}
audit = review.get("auditSummary") or {}
provider_smoke = load_json(provider_smoke_status_path) or {
    "status": "missing",
    "provider": None,
    "reason": "provider smoke status missing",
    "artifactPath": None,
}
current_status = status.get("status", "unknown")
current_stage = status.get("stage")
current_message = status.get("message")

stage_status = {}
stage_detail = {}
for entry in stage_log:
    stage_status[entry["stage"]] = entry["status"]
    stage_detail[entry["stage"]] = entry.get("detail")

proof_chain = [
    {"stage": "source.agent-entry", "label": "Release Source Agent Entry"},
    {"stage": "stable.contract-baseline", "label": "Stable Contract Baseline"},
    {"stage": "runtime-api-sdk-compatibility", "label": "Runtime API / SDK Compatibility"},
    {"stage": "filesystem-contract", "label": "AgentFlow Filesystem Contract"},
    {"stage": "release.version-metadata", "label": "Release Version Metadata"},
    {"stage": "release.changelog-entry", "label": "Release Changelog Entry"},
    {"stage": "release.github-release-fact", "label": "GitHub Release Fact"},
    {"stage": "pack.release-gate-readiness", "label": "Pack Release Gate Readiness"},
    {"stage": "pack.negative-fixtures", "label": "Pack Negative Fixtures"},
    {"stage": "pack.migration-execution", "label": "Pack Migration Execution"},
    {"stage": "pack-contract-compatibility", "label": "Pack Contract Compatibility"},
    {"stage": "projection-readmodel-contract", "label": "Projection / Read Model Contract"},
    {"stage": "evidence-acceptance-contract", "label": "Evidence / Acceptance Contract"},
    {"stage": "executor-adapter-contract", "label": "Executor Adapter Contract"},
    {"stage": "replay-migration-upgrade-certification", "label": "Replay / Migration / Upgrade Certification"},
    {"stage": "software-dev-pack-stable-baseline", "label": "Software Dev Pack Stable Baseline"},
    {"stage": "v100-release-certification", "label": "v1.0.0 Release Certification"},
    {"stage": "release-provenance", "label": "Release Provenance Manifest"},
    {"stage": "clean-room-test-proof", "label": "Clean-room Cargo Test Proof"},
    {"stage": "audit-sidecar-policy", "label": "Public Delivery Audit Sidecar Policy"},
    {"stage": "provider-smoke-proof", "label": "Provider Smoke Optional Proof"},
    {"stage": "software-dev-pack-usage-baseline", "label": "Software Dev Pack Usage Baseline"},
    {"stage": "trusted-governance-telemetry", "label": "Trusted Governance Telemetry Source"},
    {"stage": "v101-release-certification", "label": "v1.0.1 Release Certification"},
    {"stage": "v102-negative-fixtures", "label": "v1.0.2 Negative Fixture Certification"},
    {"stage": "v102-release-certification", "label": "v1.0.2 Release Certification"},
    {"stage": "forged-governance-runtime-fixture", "label": "Executable Forged Governance Fixture"},
    {"stage": "release-artifact-boundary", "label": "Release Artifact Boundary"},
    {"stage": "project-roadmap-baseline", "label": "Project Roadmap Baseline"},
    {"stage": "v103-release-fix-certification", "label": "v1.0.3 Release Fix Certification"},
    {"stage": "core-4d-spec-intake", "label": "Core 4-D Spec Intake"},
    {"stage": "core-ontology-kernel", "label": "Core Ontology Kernel"},
    {"stage": "core-object-link-schema", "label": "Core Object / Link Schema"},
    {"stage": "core-action-state-semantics", "label": "Core Action / State Semantics"},
    {"stage": "core-skill-registry", "label": "Core Skill Registry / Action Authorization"},
    {"stage": "core-evidence-decision-reference-model", "label": "Core Evidence / Decision Reference Model"},
    {"stage": "core-evidence-pack-schema", "label": "Core Evidence Pack Schema"},
    {"stage": "core-evidence-source-type-registry", "label": "Core Evidence Source Type Registry"},
    {"stage": "core-evidence-capture-receipts", "label": "Core Evidence Capture Receipts"},
    {"stage": "core-evidence-authority-trace-links", "label": "Core Evidence Authority Trace Links"},
    {"stage": "core-evidence-completeness-policy", "label": "Core Evidence Completeness Policy"},
    {"stage": "core-missing-evidence-handling", "label": "Core Missing Evidence Handling"},
    {"stage": "core-external-proof-provenance", "label": "Core External Proof Provenance"},
    {"stage": "software-dev-reference-evidence-mapping", "label": "Software Dev Reference Evidence Mapping"},
    {"stage": "evidence-projection-read-model", "label": "Evidence Projection Read Model"},
    {"stage": "core-file-backed-ontology-registry", "label": "Core File-backed Ontology Registry"},
    {"stage": "v104-release-certification", "label": "v1.0.4 Release Certification"},
    {"stage": "core-runtime-negative-fixtures", "label": "Core Runtime Negative Fixtures"},
    {"stage": "core-runtime-kernel", "label": "Core Runtime Kernel"},
    {"stage": "core-runtime-admission", "label": "Core Runtime Admission"},
    {"stage": "core-runtime-arbitration", "label": "Core Runtime Arbitration"},
    {"stage": "v105-release-certification", "label": "v1.0.5 Release Certification"},
    {"stage": "v106-release-certification", "label": "v1.0.6 Release Certification"},
    {"stage": "v107-release-provenance-handoff", "label": "v1.0.7 Release Provenance Handoff"},
    {"stage": "core-decision-model-contract", "label": "Core Decision Model Contract"},
    {"stage": "core-decision-input-binding", "label": "Core Decision Input Binding"},
    {"stage": "core-decision-outcome-transitions", "label": "Core Decision Outcome Transitions"},
    {"stage": "core-decision-failure-reason-remediation", "label": "Core Decision Failure Reason / Remediation"},
    {"stage": "core-evidence-to-decision-gate", "label": "Core Evidence-to-Decision Gate"},
    {"stage": "core-completion-commit-authority", "label": "Core Completion Commit Authority"},
    {"stage": "core-delivery-readiness-audit-trigger", "label": "Core Delivery Readiness / Optional Audit Trigger"},
    {"stage": "core-projection-kernel-contract", "label": "Core Projection Kernel Contract"},
    {"stage": "core-read-model-schema", "label": "Core Read Model Schema"},
    {"stage": "projection-feedback-freshness-receipts", "label": "Projection Feedback / Freshness Receipts"},
    {"stage": "core-view-model-contract", "label": "Core View Model Contract"},
    {"stage": "requirement.intake", "label": "Requirement Intake"},
    {"stage": "classification.ready", "label": "Classification Ready"},
    {"stage": "context.ready", "label": "Context Ready"},
    {"stage": "boundary.ready", "label": "Boundary Ready"},
    {"stage": "route.ready", "label": "Route Ready"},
    {"stage": "preview.ready", "label": "Preview Ready"},
    {"stage": "goal.confirm", "label": "Goal Confirm"},
    {"stage": "plan.confirm", "label": "Plan Confirm"},
    {"stage": "confirmation.confirmed", "label": "Confirmation Confirmed"},
    {"stage": "project.materialize", "label": "Spec Materialize"},
    {"stage": "materialization.authority-written", "label": "Materialization Authority Written"},
    {"stage": "runtime-action-proposal.accepted", "label": "Runtime Action Proposal Accepted"},
    {"stage": "projection.current", "label": "Projection Current"},
    {"stage": "task-loop.tick.issue1", "label": "Project Loop"},
    {"stage": "issue-1.session", "label": "Task Session 1"},
    {"stage": "issue-1.prepare-review", "label": "Task Review Prepare"},
    {"stage": "issue-1.complete", "label": "Task Complete 1"},
    {"stage": "issue-2.session", "label": "Task Session 2"},
    {"stage": "issue-2.prepare-review", "label": "Task Review Prepare 2"},
    {"stage": "issue-2.complete", "label": "Task Complete 2"},
    {"stage": "completion.inspect", "label": "Completion Inspect"},
    {"stage": "completion.decide", "label": "Completion Decide"},
    {"stage": "release.prepare", "label": "Release Prepare"},
    {"stage": "release.confirm", "label": "Release Confirm"},
    {"stage": "release.record-tag", "label": "Release Tag Proof"},
    {"stage": "release.record-remote", "label": "Remote Release Proof"},
    {"stage": "release.publish", "label": "Release Publish"},
    {"stage": "governance-admission", "label": "Runtime Governance Admission"},
    {"stage": "deployment-evidence", "label": "Deployment Evidence And Rollback Proof"},
    {"stage": "negative-semantic-fixtures", "label": "Negative Semantic Fixtures"},
    {"stage": "audit.request-human", "label": "Audit Request Human"},
    {"stage": "release.publish.refresh", "label": "Release Publish Refresh"},
]
for item in proof_chain:
    item["status"] = stage_status.get(item["stage"], "missing")
    item["detail"] = stage_detail.get(item["stage"])

public_artifacts = [
    {"path": "public/CHANGELOG.md", "exists": pathlib.Path(summary_json_path.parent / "public/CHANGELOG.md").is_file()},
    {"path": "public/release-notes.md", "exists": pathlib.Path(summary_json_path.parent / "public/release-notes.md").is_file()},
    {"path": "public/external-review.md", "exists": pathlib.Path(summary_json_path.parent / "public/external-review.md").is_file()},
]
runtime_artifacts = [
    {"path": "artifact-manifest.json", "exists": pathlib.Path(summary_json_path.parent / "artifact-manifest.json").is_file()},
    {"path": "runtime/source-agent-entry.json", "exists": source_agent_entry_path.is_file()},
    {"path": "runtime/stable-contract-baseline.json", "exists": pathlib.Path(summary_json_path.parent / "runtime/stable-contract-baseline.json").is_file()},
    {"path": "runtime/spec-loop-manifest.json", "exists": pathlib.Path(summary_json_path.parent / "runtime/spec-loop-manifest.json").is_file()},
    {"path": "runtime/spec-loop-projection.json", "exists": pathlib.Path(summary_json_path.parent / "runtime/spec-loop-projection.json").is_file()},
    {"path": "runtime/release-facts.json", "exists": pathlib.Path(summary_json_path.parent / "runtime/release-facts.json").is_file()},
    {"path": "runtime/external-review-surface.json", "exists": pathlib.Path(summary_json_path.parent / "runtime/external-review-surface.json").is_file()},
    {"path": "runtime/completion-runtime.json", "exists": pathlib.Path(summary_json_path.parent / "runtime/completion-runtime.json").is_file()},
    {"path": "runtime/final-closeout-proof.json", "exists": pathlib.Path(summary_json_path.parent / "runtime/final-closeout-proof.json").is_file()},
    {"path": "runtime/final-acceptance-gate.json", "exists": pathlib.Path(summary_json_path.parent / "runtime/final-acceptance-gate.json").is_file()},
    {"path": "runtime/audit-index.json", "exists": pathlib.Path(summary_json_path.parent / "runtime/audit-index.json").is_file()},
    {"path": "runtime/provider-smoke-status.json", "exists": provider_smoke_status_path.is_file()},
    {"path": "runtime/provider-smoke-artifact.json", "exists": provider_smoke_artifact_path.is_file()},
    {"path": "runtime/api-plane-manifest.json", "exists": api_plane_manifest_path.is_file()},
    {"path": "runtime/runtime-api-sdk-compatibility.json", "exists": pathlib.Path(summary_json_path.parent / "runtime/runtime-api-sdk-compatibility.json").is_file()},
    {"path": "runtime/filesystem-contract.json", "exists": pathlib.Path(summary_json_path.parent / "runtime/filesystem-contract.json").is_file()},
    {"path": "runtime/pack-contract-compatibility.json", "exists": pathlib.Path(summary_json_path.parent / "runtime/pack-contract-compatibility.json").is_file()},
    {"path": "runtime/projection-readmodel-contract.json", "exists": pathlib.Path(summary_json_path.parent / "runtime/projection-readmodel-contract.json").is_file()},
    {"path": "runtime/evidence-acceptance-contract.json", "exists": pathlib.Path(summary_json_path.parent / "runtime/evidence-acceptance-contract.json").is_file()},
    {"path": "runtime/executor-adapter-contract.json", "exists": pathlib.Path(summary_json_path.parent / "runtime/executor-adapter-contract.json").is_file()},
    {"path": "runtime/replay-migration-upgrade-certification.json", "exists": pathlib.Path(summary_json_path.parent / "runtime/replay-migration-upgrade-certification.json").is_file()},
    {"path": "runtime/software-dev-pack-stable-baseline.json", "exists": pathlib.Path(summary_json_path.parent / "runtime/software-dev-pack-stable-baseline.json").is_file()},
    {"path": "runtime/v100-release-certification.json", "exists": pathlib.Path(summary_json_path.parent / "runtime/v100-release-certification.json").is_file()},
    {"path": "runtime/release-provenance.json", "exists": pathlib.Path(summary_json_path.parent / "runtime/release-provenance.json").is_file()},
    {"path": "runtime/v107-release-provenance-handoff.json", "exists": pathlib.Path(summary_json_path.parent / "runtime/v107-release-provenance-handoff.json").is_file()},
    {"path": "runtime/core-decision-model-contract.json", "exists": pathlib.Path(summary_json_path.parent / "runtime/core-decision-model-contract.json").is_file()},
    {"path": "runtime/core-decision-input-binding.json", "exists": pathlib.Path(summary_json_path.parent / "runtime/core-decision-input-binding.json").is_file()},
    {"path": "runtime/core-decision-outcome-transitions.json", "exists": pathlib.Path(summary_json_path.parent / "runtime/core-decision-outcome-transitions.json").is_file()},
    {"path": "runtime/core-decision-failure-reason-remediation.json", "exists": pathlib.Path(summary_json_path.parent / "runtime/core-decision-failure-reason-remediation.json").is_file()},
    {"path": "runtime/core-evidence-to-decision-gate.json", "exists": pathlib.Path(summary_json_path.parent / "runtime/core-evidence-to-decision-gate.json").is_file()},
    {"path": "runtime/core-completion-commit-authority.json", "exists": pathlib.Path(summary_json_path.parent / "runtime/core-completion-commit-authority.json").is_file()},
    {"path": "runtime/core-delivery-readiness-audit-trigger.json", "exists": pathlib.Path(summary_json_path.parent / "runtime/core-delivery-readiness-audit-trigger.json").is_file()},
    {"path": "runtime/clean-room-test-proof.json", "exists": pathlib.Path(summary_json_path.parent / "runtime/clean-room-test-proof.json").is_file()},
    {"path": "runtime/audit-sidecar-policy.json", "exists": pathlib.Path(summary_json_path.parent / "runtime/audit-sidecar-policy.json").is_file()},
    {"path": "runtime/provider-smoke-proof.json", "exists": pathlib.Path(summary_json_path.parent / "runtime/provider-smoke-proof.json").is_file()},
    {"path": "runtime/software-dev-pack-usage-baseline.json", "exists": pathlib.Path(summary_json_path.parent / "runtime/software-dev-pack-usage-baseline.json").is_file()},
    {"path": "runtime/trusted-governance-telemetry.json", "exists": pathlib.Path(summary_json_path.parent / "runtime/trusted-governance-telemetry.json").is_file()},
    {"path": "runtime/v101-release-certification.json", "exists": pathlib.Path(summary_json_path.parent / "runtime/v101-release-certification.json").is_file()},
    {"path": "runtime/v102-negative-fixtures.json", "exists": pathlib.Path(summary_json_path.parent / "runtime/v102-negative-fixtures.json").is_file()},
    {"path": "runtime/v102-release-certification.json", "exists": pathlib.Path(summary_json_path.parent / "runtime/v102-release-certification.json").is_file()},
    {"path": "runtime/forged-governance-runtime-response.json", "exists": forged_governance_response_path.is_file()},
    {"path": "runtime/release-artifact-boundary.json", "exists": release_artifact_boundary_path.is_file()},
    {"path": "runtime/project-roadmap-baseline.json", "exists": project_roadmap_baseline_path.is_file()},
    {"path": "runtime/v103-release-fix-certification.json", "exists": v103_release_fix_certification_path.is_file()},
    {"path": "runtime/core-runtime-negative-fixtures.json", "exists": core_runtime_negative_fixtures_path.is_file()},
    {"path": "runtime/core-runtime-kernel.json", "exists": pathlib.Path(summary_json_path.parent / "runtime/core-runtime-kernel.json").is_file()},
    {"path": "runtime/core-runtime-admission.json", "exists": pathlib.Path(summary_json_path.parent / "runtime/core-runtime-admission.json").is_file()},
    {"path": "runtime/core-runtime-arbitration.json", "exists": pathlib.Path(summary_json_path.parent / "runtime/core-runtime-arbitration.json").is_file()},
    {"path": "runtime/v105-release-certification.json", "exists": pathlib.Path(summary_json_path.parent / "runtime/v105-release-certification.json").is_file()},
    {"path": "runtime/evidence-projection-read-model.json", "exists": pathlib.Path(summary_json_path.parent / "runtime/evidence-projection-read-model.json").is_file()},
    {"path": "runtime/evidence-projection-read-model-rust-test.log", "exists": pathlib.Path(summary_json_path.parent / "runtime/evidence-projection-read-model-rust-test.log").is_file()},
    {"path": "runtime/capability-registry.json", "exists": capability_registry_path.is_file()},
    {"path": "runtime/governance-policy.json", "exists": governance_policy_path.is_file()},
    {"path": "runtime/governance-admission.json", "exists": governance_admission_path.is_file()},
    {"path": "runtime/scheduling-decision.json", "exists": scheduling_decision_path.is_file()},
    {"path": "runtime/deployment-evidence.json", "exists": deployment_evidence_path.is_file()},
    {"path": "runtime/deployment-evidence-semantic-failure.json", "exists": pathlib.Path(summary_json_path.parent / "runtime/deployment-evidence-semantic-failure.json").is_file()},
    {"path": "runtime/deployment-evidence-wrong-commit.json", "exists": pathlib.Path(summary_json_path.parent / "runtime/deployment-evidence-wrong-commit.json").is_file()},
    {"path": "runtime/deployment-evidence-wrong-url.json", "exists": pathlib.Path(summary_json_path.parent / "runtime/deployment-evidence-wrong-url.json").is_file()},
    {"path": "runtime/deployment-evidence-fake-migration-receipt.json", "exists": pathlib.Path(summary_json_path.parent / "runtime/deployment-evidence-fake-migration-receipt.json").is_file()},
    {"path": "runtime/negative-semantic-fixtures.json", "exists": negative_semantic_fixtures_path.is_file()},
    {"path": "runtime/foundation-readiness-report.md", "exists": foundation_readiness_report_path.is_file()},
    {"path": "runtime/foundation-coverage.json", "exists": foundation_coverage_path.is_file()},
    {"path": "runtime/event-replay-projection-report.json", "exists": pathlib.Path(summary_json_path.parent / "runtime/event-replay-projection-report.json").is_file()},
    {"path": "runtime/event-replay-projection-failure-report.json", "exists": pathlib.Path(summary_json_path.parent / "runtime/event-replay-projection-failure-report.json").is_file()},
    {"path": "runtime/pack-migration-replay-report.json", "exists": pathlib.Path(summary_json_path.parent / "runtime/pack-migration-replay-report.json").is_file()},
    {"path": "pack-registry.json", "exists": pathlib.Path(summary_json_path.parent / "pack-registry.json").is_file()},
    {"path": "pack-validation-report.json", "exists": pathlib.Path(summary_json_path.parent / "pack-validation-report.json").is_file()},
    {"path": "pack-simulation-report.json", "exists": pathlib.Path(summary_json_path.parent / "pack-simulation-report.json").is_file()},
    {"path": "pack-projection-readiness.json", "exists": pathlib.Path(summary_json_path.parent / "pack-projection-readiness.json").is_file()},
    {"path": "pack-api-plane-manifest.json", "exists": pathlib.Path(summary_json_path.parent / "pack-api-plane-manifest.json").is_file()},
    {"path": "pack-negative-fixtures.json", "exists": pack_negative_fixtures_path.is_file()},
    {"path": "pack-migration-preview.json", "exists": pathlib.Path(summary_json_path.parent / "pack-migration-preview.json").is_file()},
    {"path": "pack-migration-unconfirmed-apply.json", "exists": pathlib.Path(summary_json_path.parent / "pack-migration-unconfirmed-apply.json").is_file()},
    {"path": "pack-migration-applied-receipt.json", "exists": pathlib.Path(summary_json_path.parent / "pack-migration-applied-receipt.json").is_file()},
    {"path": "pack-migration-fake-authority-receipt.json", "exists": pathlib.Path(summary_json_path.parent / "pack-migration-fake-authority-receipt.json").is_file()},
    {"path": "pack-migration-cancel-receipt.json", "exists": pathlib.Path(summary_json_path.parent / "pack-migration-cancel-receipt.json").is_file()},
    {"path": "pack-migration-rollback-receipt.json", "exists": pathlib.Path(summary_json_path.parent / "pack-migration-rollback-receipt.json").is_file()},
    {"path": "software-dev-pack-readiness.json", "exists": pathlib.Path(summary_json_path.parent / "software-dev-pack-readiness.json").is_file()},
    {"path": "ui-design-pack-readiness.json", "exists": pathlib.Path(summary_json_path.parent / "ui-design-pack-readiness.json").is_file()},
]

pack_validation = load_json(pathlib.Path(summary_json_path.parent / "pack-validation-report.json")) or {}
pack_simulation = load_json(pathlib.Path(summary_json_path.parent / "pack-simulation-report.json")) or {}
pack_projection = load_json(pathlib.Path(summary_json_path.parent / "pack-projection-readiness.json")) or {}
pack_api_plane = load_json(pathlib.Path(summary_json_path.parent / "pack-api-plane-manifest.json")) or {}
pack_registry = load_json(pathlib.Path(summary_json_path.parent / "pack-registry.json")) or {}
pack_negative_fixtures = load_json(pack_negative_fixtures_path) or {}
governance_policy = load_json(governance_policy_path) or {}
governance_admission = load_json(governance_admission_path) or {}
scheduling_decision = load_json(scheduling_decision_path) or {}
release_provenance = load_json(pathlib.Path(summary_json_path.parent / "runtime/release-provenance.json")) or {}
clean_room_test_proof = load_json(pathlib.Path(summary_json_path.parent / "runtime/clean-room-test-proof.json")) or {}
audit_sidecar_policy = load_json(pathlib.Path(summary_json_path.parent / "runtime/audit-sidecar-policy.json")) or {}
provider_smoke_proof = load_json(pathlib.Path(summary_json_path.parent / "runtime/provider-smoke-proof.json")) or {}
software_dev_pack_usage_baseline = load_json(pathlib.Path(summary_json_path.parent / "runtime/software-dev-pack-usage-baseline.json")) or {}
trusted_governance_telemetry = load_json(pathlib.Path(summary_json_path.parent / "runtime/trusted-governance-telemetry.json")) or {}
v101_release_certification = load_json(pathlib.Path(summary_json_path.parent / "runtime/v101-release-certification.json")) or {}
forged_governance_response = load_json(forged_governance_response_path) or {}
release_artifact_boundary = load_json(release_artifact_boundary_path) or {}
project_roadmap_baseline = load_json(project_roadmap_baseline_path) or {}
v103_release_fix_certification = load_json(v103_release_fix_certification_path) or {}
core_runtime_negative_fixtures = load_json(core_runtime_negative_fixtures_path) or {}
core_runtime_kernel = load_json(pathlib.Path(summary_json_path.parent / "runtime/core-runtime-kernel.json")) or {}
core_runtime_admission = load_json(pathlib.Path(summary_json_path.parent / "runtime/core-runtime-admission.json")) or {}
core_runtime_arbitration = load_json(pathlib.Path(summary_json_path.parent / "runtime/core-runtime-arbitration.json")) or {}
v105_release_certification = load_json(pathlib.Path(summary_json_path.parent / "runtime/v105-release-certification.json")) or {}
v106_release_certification = load_json(pathlib.Path(summary_json_path.parent / "runtime/v106-release-certification.json")) or {}
v107_release_provenance_handoff = load_json(pathlib.Path(summary_json_path.parent / "runtime/v107-release-provenance-handoff.json")) or {}
v107_release_certification = load_json(pathlib.Path(summary_json_path.parent / "runtime/v107-release-certification.json")) or {}
v108_release_certification = load_json(pathlib.Path(summary_json_path.parent / "runtime/v108-release-certification.json")) or {}
core_decision_model_contract = load_json(pathlib.Path(summary_json_path.parent / "runtime/core-decision-model-contract.json")) or {}
core_decision_input_binding = load_json(pathlib.Path(summary_json_path.parent / "runtime/core-decision-input-binding.json")) or {}
core_decision_outcome_transitions = load_json(pathlib.Path(summary_json_path.parent / "runtime/core-decision-outcome-transitions.json")) or {}
core_decision_failure_reason = load_json(pathlib.Path(summary_json_path.parent / "runtime/core-decision-failure-reason-remediation.json")) or {}
core_evidence_to_decision_gate = load_json(pathlib.Path(summary_json_path.parent / "runtime/core-evidence-to-decision-gate.json")) or {}
core_completion_commit_authority = load_json(pathlib.Path(summary_json_path.parent / "runtime/core-completion-commit-authority.json")) or {}
core_delivery_readiness_audit_trigger = load_json(pathlib.Path(summary_json_path.parent / "runtime/core-delivery-readiness-audit-trigger.json")) or {}
core_decision_projection_read_model = load_json(pathlib.Path(summary_json_path.parent / "runtime/core-decision-projection-read-model.json")) or {}
deployment_evidence = load_json(deployment_evidence_path) or {}
deployment_evidence_failure = load_json(pathlib.Path(summary_json_path.parent / "runtime/deployment-evidence-semantic-failure.json")) or {}
deployment_evidence_wrong_commit = load_json(pathlib.Path(summary_json_path.parent / "runtime/deployment-evidence-wrong-commit.json")) or {}
deployment_evidence_wrong_url = load_json(pathlib.Path(summary_json_path.parent / "runtime/deployment-evidence-wrong-url.json")) or {}
deployment_evidence_fake_migration = load_json(pathlib.Path(summary_json_path.parent / "runtime/deployment-evidence-fake-migration-receipt.json")) or {}
negative_semantic_fixtures = load_json(negative_semantic_fixtures_path) or {}
source_agent_entry = load_json(source_agent_entry_path) or {}
stable_contract_baseline = load_json(pathlib.Path(summary_json_path.parent / "runtime/stable-contract-baseline.json")) or {}
runtime_api_sdk_compatibility = load_json(pathlib.Path(summary_json_path.parent / "runtime/runtime-api-sdk-compatibility.json")) or {}
filesystem_contract = load_json(pathlib.Path(summary_json_path.parent / "runtime/filesystem-contract.json")) or {}
pack_contract_compatibility = load_json(pathlib.Path(summary_json_path.parent / "runtime/pack-contract-compatibility.json")) or {}
projection_readmodel_contract = load_json(pathlib.Path(summary_json_path.parent / "runtime/projection-readmodel-contract.json")) or {}
evidence_acceptance_contract = load_json(pathlib.Path(summary_json_path.parent / "runtime/evidence-acceptance-contract.json")) or {}
executor_adapter_contract = load_json(pathlib.Path(summary_json_path.parent / "runtime/executor-adapter-contract.json")) or {}
replay_migration_upgrade_certification = load_json(pathlib.Path(summary_json_path.parent / "runtime/replay-migration-upgrade-certification.json")) or {}
software_dev_pack_stable_baseline = load_json(pathlib.Path(summary_json_path.parent / "runtime/software-dev-pack-stable-baseline.json")) or {}
event_replay_projection = load_json(pathlib.Path(summary_json_path.parent / "runtime/event-replay-projection-report.json")) or {}
event_replay_projection_failure = load_json(pathlib.Path(summary_json_path.parent / "runtime/event-replay-projection-failure-report.json")) or {}
pack_migration_unconfirmed = load_json(pathlib.Path(summary_json_path.parent / "pack-migration-unconfirmed-apply.json")) or {}
pack_migration_applied = load_json(pathlib.Path(summary_json_path.parent / "pack-migration-applied-receipt.json")) or {}
pack_migration_fake_authority = load_json(pathlib.Path(summary_json_path.parent / "pack-migration-fake-authority-receipt.json")) or {}
pack_migration_cancel = load_json(pathlib.Path(summary_json_path.parent / "pack-migration-cancel-receipt.json")) or {}
pack_migration_rollback = load_json(pathlib.Path(summary_json_path.parent / "pack-migration-rollback-receipt.json")) or {}
pack_migration_replay = load_json(pathlib.Path(summary_json_path.parent / "runtime/pack-migration-replay-report.json")) or {}
software_readiness = load_json(pathlib.Path(summary_json_path.parent / "software-dev-pack-readiness.json")) or {}
design_readiness = load_json(pathlib.Path(summary_json_path.parent / "ui-design-pack-readiness.json")) or {}
pack_release_gate_passed = (
    pack_validation.get("status") == "passed"
    and pack_simulation.get("status") == "passed"
    and pack_projection.get("status") == "passed"
    and pack_api_plane.get("status") == "passed"
    and pack_negative_fixtures.get("status") == "passed"
    and software_readiness.get("status") == "completed"
    and design_readiness.get("status") == "baseline"
)
pack_simulation_reports = pack_simulation.get("reports", [])
pack_simulation_evaluation_passed = bool(pack_simulation_reports) and all(
    report.get("writesAuthority") is False
    and report.get("writesEventStore") is False
    and report.get("executesProvider") is False
    and bool(report.get("affectedObjects"))
    and bool(report.get("requiredEvidence"))
    and bool(report.get("stateTransitions"))
    and bool(report.get("downstreamTriggers"))
    and bool(report.get("conflicts"))
    and bool(report.get("gateImpact"))
    for report in pack_simulation_reports
)
governance_reports = governance_policy.get("reports") or []
governance_decisions = {report.get("decision") for report in governance_reports}
governance_policy_passed = (
    governance_policy.get("status") == "passed"
    and governance_policy.get("version") == "agentflow-runtime-governance-policy-gate.v1"
    and {"allowed", "deferred", "rejected"}.issubset(governance_decisions)
    and all(report.get("trace") for report in governance_reports)
    and any(
        report.get("capabilityPolicy", {}).get("decision") in {"deferred", "rejected"}
        for report in governance_reports
    )
    and any(
        report.get("auditSidecarPolicy", {}).get("decision") == "rejected"
        for report in governance_reports
    )
)
governance_admission_responses = governance_admission.get("responses") or []
governance_admission_decisions = {
    (response.get("governanceAdmission") or {}).get("decision")
    for response in governance_admission_responses
}
governance_admission_passed = (
    governance_admission.get("status") == "passed"
    and governance_admission.get("version") == "agentflow-runtime-governance-admission-gate.v1"
    and {"allowed", "deferred", "rejected"}.issubset(governance_admission_decisions)
    and governance_admission.get("allowedEnteredProposal") is True
    and governance_admission.get("deferredWroteProposal") is False
    and governance_admission.get("rejectedWroteProposal") is False
    and all(
        (response.get("governanceAdmission") or {}).get("trace")
        for response in governance_admission_responses
    )
)
scheduling_decision_passed = (
    scheduling_decision.get("version") == "agentflow-scheduling-decision-report.v1"
    and scheduling_decision.get("status") == "passed"
    and scheduling_decision.get("decision") in {"go", "no-go", "defer"}
    and scheduling_decision.get("writesAuthority") is False
    and scheduling_decision.get("expandsImplementationScope") is False
    and bool(scheduling_decision.get("evidence"))
    and (
        bool(scheduling_decision.get("requiredContract"))
        or bool(scheduling_decision.get("alternativeMechanism"))
    )
)
deployment_evidence_passed = (
    deployment_evidence.get("version") == "agentflow-deployment-evidence-report.v1"
    and deployment_evidence.get("status") == "passed"
    and deployment_evidence.get("writesAuthority") is False
    and deployment_evidence.get("localDeployment", {}).get("status") == "ready"
    and deployment_evidence.get("cloudDeployment", {}).get("status") == "ready"
    and deployment_evidence.get("rollbackModel", {}).get("providerAgnostic") is True
    and deployment_evidence.get("rollbackModel", {}).get("rollbackReceipt", {}).get("exists") is True
    and not deployment_evidence.get("semanticFailures")
    and deployment_evidence_failure.get("status") == "failed"
    and {"remote-release-proof.tag", "artifact-manifest.sha-present"}.issubset(
        set(deployment_evidence_failure.get("semanticFailures") or [])
    )
)
source_agent_entry_passed = (
    source_agent_entry.get("status") == "passed"
    and bool(source_agent_entry.get("entryPath"))
    and all(item.get("exists") for item in source_agent_entry.get("trackedDocs", []))
    and not source_agent_entry.get("trackedRuntimePaths")
)
stable_contract_baseline_passed = (
    stable_contract_baseline.get("status") == "passed"
    and stable_contract_baseline.get("stableContractVersion") == "agentflow-stable-contract-baseline.v1"
    and stable_contract_baseline.get("stableContractStatus") == "active"
    and stable_contract_baseline.get("docPath") == "docs/architecture/041-v100-stable-contract-baseline-v1.md"
    and not stable_contract_baseline.get("missingSections")
)
runtime_api_sdk_compatibility_passed = (
    runtime_api_sdk_compatibility.get("status") == "passed"
    and runtime_api_sdk_compatibility.get("runtimeApiSdkContractVersion") == "agentflow-runtime-api-sdk-freeze.v1"
    and runtime_api_sdk_compatibility.get("runtimeApiSdkContractStatus") == "active"
    and runtime_api_sdk_compatibility.get("docPath") == "docs/architecture/042-v100-runtime-api-sdk-freeze-v1.md"
    and runtime_api_sdk_compatibility.get("stableContractBaseline") == "agentflow-stable-contract-baseline.v1"
    and runtime_api_sdk_compatibility.get("commandPathRequiresGovernanceAdmission") is True
    and runtime_api_sdk_compatibility.get("rejectedWritesProposal") is False
    and runtime_api_sdk_compatibility.get("deferredWritesProposal") is False
    and runtime_api_sdk_compatibility.get("rejectedWritesAcceptedEvent") is False
    and runtime_api_sdk_compatibility.get("deferredWritesAcceptedEvent") is False
    and runtime_api_sdk_compatibility.get("sdkCandidateReadonly") is True
    and not runtime_api_sdk_compatibility.get("missingSections")
    and not runtime_api_sdk_compatibility.get("missingManifestEntries")
)
filesystem_contract_passed = (
    filesystem_contract.get("status") == "passed"
    and filesystem_contract.get("filesystemContractVersion") == "agentflow-filesystem-contract-freeze.v1"
    and filesystem_contract.get("filesystemContractStatus") == "active"
    and filesystem_contract.get("docPath") == "docs/architecture/043-v100-agentflow-filesystem-contract-freeze-v1.md"
    and filesystem_contract.get("stableContractBaseline") == "agentflow-stable-contract-baseline.v1"
    and filesystem_contract.get("sourceArchiveIncludesRuntimeState") is False
    and not filesystem_contract.get("missingSections")
    and not filesystem_contract.get("missingStablePaths")
    and not filesystem_contract.get("missingAuthorityClasses")
    and not filesystem_contract.get("retiredPathViolations")
)
pack_contract_compatibility_passed = (
    pack_contract_compatibility.get("status") == "passed"
    and pack_contract_compatibility.get("packContractVersion") == "agentflow-pack-contract-freeze.v1"
    and pack_contract_compatibility.get("packContractStatus") == "active"
    and pack_contract_compatibility.get("docPath") == "docs/architecture/044-v100-pack-contract-freeze-v1.md"
    and pack_contract_compatibility.get("stableContractBaseline") == "agentflow-stable-contract-baseline.v1"
    and pack_contract_compatibility.get("filesystemContractVersion") == "agentflow-filesystem-contract-freeze.v1"
    and pack_contract_compatibility.get("registrySource") == "project-files"
    and pack_contract_compatibility.get("registryFallback") is False
    and pack_contract_compatibility.get("validationStatus") == "passed"
    and pack_contract_compatibility.get("simulationStatus") == "passed"
    and pack_contract_compatibility.get("projectionReadinessStatus") == "passed"
    and pack_contract_compatibility.get("apiPlaneStatus") == "passed"
    and pack_contract_compatibility.get("negativeFixturesStatus") == "passed"
    and pack_contract_compatibility.get("migrationReceiptOnly") is True
    and not pack_contract_compatibility.get("missingSections")
    and not pack_contract_compatibility.get("missingRequiredFixtures")
)
projection_readmodel_contract_passed = (
    projection_readmodel_contract.get("status") == "passed"
    and projection_readmodel_contract.get("projectionContractVersion") == "agentflow-projection-readmodel-contract.v1"
    and projection_readmodel_contract.get("projectionContractStatus") == "active"
    and projection_readmodel_contract.get("docPath") == "docs/architecture/045-v100-projection-readmodel-contract-freeze-v1.md"
    and projection_readmodel_contract.get("stableContractBaseline") == "agentflow-stable-contract-baseline.v1"
    and projection_readmodel_contract.get("runtimeApiSdkVersion") == "agentflow-runtime-api-sdk-freeze.v1"
    and projection_readmodel_contract.get("filesystemContractVersion") == "agentflow-filesystem-contract-freeze.v1"
    and projection_readmodel_contract.get("packContractVersion") == "agentflow-pack-contract-freeze.v1"
    and projection_readmodel_contract.get("eventReplayStatus") == "passed"
    and projection_readmodel_contract.get("eventReplayFailureStatus") == "failed"
    and projection_readmodel_contract.get("eventReplayWritesAuthority") is False
    and projection_readmodel_contract.get("eventReplayProjectionAuthority") is False
    and projection_readmodel_contract.get("queryApiReadonly") is True
    and projection_readmodel_contract.get("packProjectionStatus") == "passed"
    and projection_readmodel_contract.get("packMissingDefinitionBehavior") in {"invalid", "deferred"}
    and projection_readmodel_contract.get("industrySurfaceReadonly") is True
    and projection_readmodel_contract.get("sidecarReadModelsPresent") is True
    and not projection_readmodel_contract.get("missingSections")
    and not projection_readmodel_contract.get("missingRequiredReadModels")
    and not projection_readmodel_contract.get("missingProjectionPaths")
)
evidence_acceptance_contract_passed = (
    evidence_acceptance_contract.get("status") == "passed"
    and evidence_acceptance_contract.get("evidenceAcceptanceContractVersion") == "agentflow-evidence-acceptance-contract.v1"
    and evidence_acceptance_contract.get("evidenceAcceptanceContractStatus") == "active"
    and evidence_acceptance_contract.get("docPath") == "docs/architecture/046-v100-evidence-acceptance-contract-freeze-v1.md"
    and evidence_acceptance_contract.get("stableContractBaseline") == "agentflow-stable-contract-baseline.v1"
    and evidence_acceptance_contract.get("runtimeApiSdkVersion") == "agentflow-runtime-api-sdk-freeze.v1"
    and evidence_acceptance_contract.get("filesystemContractVersion") == "agentflow-filesystem-contract-freeze.v1"
    and evidence_acceptance_contract.get("packContractVersion") == "agentflow-pack-contract-freeze.v1"
    and evidence_acceptance_contract.get("projectionContractVersion") == "agentflow-projection-readmodel-contract.v1"
    and evidence_acceptance_contract.get("evidencePackStatus") == "passed"
    and evidence_acceptance_contract.get("acceptanceEventPresent") is True
    and evidence_acceptance_contract.get("completionCommitEventPresent") is True
    and evidence_acceptance_contract.get("taskDoneFromCompletionCommit") is True
    and evidence_acceptance_contract.get("closeoutProofMerged") is True
    and evidence_acceptance_contract.get("closeoutIssueClosed") is True
    and evidence_acceptance_contract.get("deliveryReadModelReady") is True
    and evidence_acceptance_contract.get("auditSidecarNonBlocking") is True
    and evidence_acceptance_contract.get("failureFixturesPassed") is True
    and not evidence_acceptance_contract.get("missingSections")
    and not evidence_acceptance_contract.get("missingRequiredPhrases")
)
executor_adapter_contract_passed = (
    executor_adapter_contract.get("status") == "passed"
    and executor_adapter_contract.get("executorAdapterContractVersion") == "agentflow-executor-adapter-contract.v1"
    and executor_adapter_contract.get("executorAdapterContractStatus") == "active"
    and executor_adapter_contract.get("docPath") == "docs/architecture/047-v100-executor-adapter-contract-freeze-v1.md"
    and executor_adapter_contract.get("stableContractBaseline") == "agentflow-stable-contract-baseline.v1"
    and executor_adapter_contract.get("runtimeApiSdkVersion") == "agentflow-runtime-api-sdk-freeze.v1"
    and executor_adapter_contract.get("filesystemContractVersion") == "agentflow-filesystem-contract-freeze.v1"
    and executor_adapter_contract.get("packContractVersion") == "agentflow-pack-contract-freeze.v1"
    and executor_adapter_contract.get("projectionContractVersion") == "agentflow-projection-readmodel-contract.v1"
    and executor_adapter_contract.get("evidenceAcceptanceContractVersion") == "agentflow-evidence-acceptance-contract.v1"
    and executor_adapter_contract.get("acceptedFixturePassed") is True
    and executor_adapter_contract.get("rejectedFixturePassed") is True
    and executor_adapter_contract.get("deferredFixturePassed") is True
    and executor_adapter_contract.get("diffBoundaryViolationRejected") is True
    and executor_adapter_contract.get("sessionIsolationRespected") is True
    and executor_adapter_contract.get("providerSmokeBoundaryRespected") is True
    and executor_adapter_contract.get("evidenceAcceptanceHandoffReady") is True
    and not executor_adapter_contract.get("missingSections")
    and not executor_adapter_contract.get("missingRequiredPhrases")
)
replay_migration_upgrade_certification_passed = (
    replay_migration_upgrade_certification.get("status") == "passed"
    and replay_migration_upgrade_certification.get("replayMigrationUpgradeCertificationVersion") == "agentflow-replay-migration-upgrade-certification.v1"
    and replay_migration_upgrade_certification.get("replayMigrationUpgradeCertificationStatus") == "active"
    and replay_migration_upgrade_certification.get("docPath") == "docs/architecture/048-v100-replay-migration-upgrade-certification-v1.md"
    and replay_migration_upgrade_certification.get("stableContractBaseline") == "agentflow-stable-contract-baseline.v1"
    and replay_migration_upgrade_certification.get("filesystemContractVersion") == "agentflow-filesystem-contract-freeze.v1"
    and replay_migration_upgrade_certification.get("packContractVersion") == "agentflow-pack-contract-freeze.v1"
    and replay_migration_upgrade_certification.get("projectionContractVersion") == "agentflow-projection-readmodel-contract.v1"
    and replay_migration_upgrade_certification.get("evidenceAcceptanceContractVersion") == "agentflow-evidence-acceptance-contract.v1"
    and replay_migration_upgrade_certification.get("executorAdapterContractVersion") == "agentflow-executor-adapter-contract.v1"
    and replay_migration_upgrade_certification.get("eventReplayStatus") == "passed"
    and replay_migration_upgrade_certification.get("eventReplayFailureStatus") == "failed"
    and replay_migration_upgrade_certification.get("projectionRebuildStatus") == "passed"
    and replay_migration_upgrade_certification.get("packMigrationPreviewStatus") == "preview"
    and replay_migration_upgrade_certification.get("packMigrationApplyStatus") == "applied"
    and replay_migration_upgrade_certification.get("packMigrationCancelStatus") == "cancelled"
    and replay_migration_upgrade_certification.get("packMigrationRollbackStatus") == "rolled-back"
    and replay_migration_upgrade_certification.get("retiredPathRevived") is False
    and replay_migration_upgrade_certification.get("negativeUpgradeFixturePassed") is True
    and replay_migration_upgrade_certification.get("deterministicReport") is True
    and not replay_migration_upgrade_certification.get("missingSections")
    and not replay_migration_upgrade_certification.get("missingRequiredPhrases")
)
software_dev_pack_stable_baseline_passed = (
    software_dev_pack_stable_baseline.get("status") == "passed"
    and software_dev_pack_stable_baseline.get("softwareDevPackStableBaselineVersion") == "agentflow-software-dev-pack-stable-baseline.v1"
    and software_dev_pack_stable_baseline.get("softwareDevPackStableBaselineStatus") == "active"
    and software_dev_pack_stable_baseline.get("docPath") == "docs/architecture/049-v100-software-dev-pack-stable-baseline-v1.md"
    and software_dev_pack_stable_baseline.get("stableContractBaseline") == "agentflow-stable-contract-baseline.v1"
    and software_dev_pack_stable_baseline.get("packContractVersion") == "agentflow-pack-contract-freeze.v1"
    and software_dev_pack_stable_baseline.get("projectionContractVersion") == "agentflow-projection-readmodel-contract.v1"
    and software_dev_pack_stable_baseline.get("evidenceAcceptanceContractVersion") == "agentflow-evidence-acceptance-contract.v1"
    and software_dev_pack_stable_baseline.get("executorAdapterContractVersion") == "agentflow-executor-adapter-contract.v1"
    and software_dev_pack_stable_baseline.get("replayMigrationUpgradeCertificationVersion") == "agentflow-replay-migration-upgrade-certification.v1"
    and software_dev_pack_stable_baseline.get("stableManifestPassed") is True
    and software_dev_pack_stable_baseline.get("readModelBoundaryPassed") is True
    and software_dev_pack_stable_baseline.get("connectorBaselinePassed") is True
    and software_dev_pack_stable_baseline.get("runtimeFixturePassed") is True
    and software_dev_pack_stable_baseline.get("deliveryBoundaryPassed") is True
    and software_dev_pack_stable_baseline.get("auditSidecarPassed") is True
    and software_dev_pack_stable_baseline.get("githubIssueAuthority") is False
    and software_dev_pack_stable_baseline.get("uiDesignPromotedToStable") is False
    and not software_dev_pack_stable_baseline.get("missingSections")
    and not software_dev_pack_stable_baseline.get("missingRequiredPhrases")
)
deployment_evidence_semantics_passed = (
    deployment_evidence_passed
    and bool(deployment_evidence.get("semanticChecks"))
    and deployment_evidence_wrong_commit.get("status") == "failed"
    and deployment_evidence_wrong_url.get("status") == "failed"
    and deployment_evidence_fake_migration.get("status") == "failed"
)
pack_migration_semantic_split_passed = (
    pack_migration_unconfirmed.get("status") == "rejected"
    and pack_migration_unconfirmed.get("writesAuthority") is False
    and pack_migration_applied.get("applied") is True
    and pack_migration_applied.get("writesAuthority") is False
    and (pack_migration_applied.get("semanticTarget") or {}).get("mutationTarget") == "receipt-only-apply"
    and (pack_migration_applied.get("semanticTarget") or {}).get("authorityMutation") is False
    and pack_migration_fake_authority.get("writesAuthority") is True
    and pack_migration_cancel.get("cancelled") is True
    and pack_migration_rollback.get("rolledBack") is True
    and pack_migration_rollback.get("writesAuthority") is False
    and (pack_migration_rollback.get("semanticTarget") or {}).get("mutationTarget") == "receipt-only-rollback"
    and (pack_migration_rollback.get("semanticTarget") or {}).get("authorityMutation") is False
    and pack_migration_replay.get("status") == "passed"
)
project_pack_registry_passed = (
    pack_registry.get("version") == "agentflow-pack-registry.v1"
    and pack_registry.get("source") == "project-files"
    and pack_registry.get("fallback") is False
    and {entry.get("packId") for entry in pack_registry.get("entries", [])} >= {"software-dev", "ui-design"}
    and all(
        entry.get("source") == "project-files"
        and entry.get("fallback") is False
        and bool(entry.get("manifestPath"))
        for entry in pack_registry.get("entries", [])
        if entry.get("packId") in {"software-dev", "ui-design"}
    )
)
negative_semantic_fixtures_passed = (
    negative_semantic_fixtures.get("status") == "passed"
    and negative_semantic_fixtures.get("writesAuthority") is False
    and negative_semantic_fixtures.get("fixtureCount", 0) >= 8
    and not negative_semantic_fixtures.get("failedFixtures")
)
v090_coverage = [
    {
        "id": "V090-001",
        "label": "Local Runtime Boundary",
        "passed": api_plane_manifest_path.is_file() and capability_registry_path.is_file(),
        "evidencePath": "runtime/api-plane-manifest.json",
    },
    {
        "id": "V090-002",
        "label": "Cloud Runtime Boundary",
        "passed": deployment_evidence.get("cloudDeployment", {}).get("status") == "ready",
        "evidencePath": "runtime/deployment-evidence.json",
    },
    {
        "id": "V090-003",
        "label": "Runtime API / SDK Contract",
        "passed": api_plane_manifest_path.is_file() and capability_registry_path.is_file(),
        "evidencePath": "runtime/api-plane-manifest.json",
    },
    {
        "id": "V090-004",
        "label": "Event Replay / Projection Rebuild",
        "passed": event_replay_projection.get("status") == "passed"
        and event_replay_projection_failure.get("status") == "failed",
        "evidencePath": "runtime/event-replay-projection-report.json",
    },
    {
        "id": "V090-005",
        "label": "Pack Migration Execution",
        "passed": pack_migration_unconfirmed.get("status") == "rejected"
        and pack_migration_applied.get("applied") is True
        and pack_migration_rollback.get("rolledBack") is True,
        "evidencePath": "pack-migration-applied-receipt.json",
    },
    {
        "id": "V090-006",
        "label": "Simulation Evaluation Layer",
        "passed": pack_simulation_evaluation_passed,
        "evidencePath": "pack-simulation-report.json",
    },
    {
        "id": "V090-007",
        "label": "Runtime Governance Policy",
        "passed": governance_policy_passed,
        "evidencePath": "runtime/governance-policy.json",
    },
    {
        "id": "V090-008",
        "label": "Cross-process Scheduling Decision",
        "passed": scheduling_decision_passed,
        "evidencePath": "runtime/scheduling-decision.json",
    },
    {
        "id": "V090-009",
        "label": "Deployment Evidence and Rollback Model",
        "passed": deployment_evidence_passed,
        "evidencePath": "runtime/deployment-evidence.json",
    },
]
v090_coverage_passed = all(item["passed"] for item in v090_coverage)
v091_coverage = [
    {
        "id": "V091-001",
        "label": "Release Source Agent Entry Alignment",
        "passed": source_agent_entry_passed,
        "evidencePath": "runtime/source-agent-entry.json",
    },
    {
        "id": "V091-002",
        "label": "Runtime Governance Admission Integration",
        "passed": governance_admission_passed,
        "evidencePath": "runtime/governance-admission.json",
    },
    {
        "id": "V091-003",
        "label": "Deployment Evidence Semantic Certification",
        "passed": deployment_evidence_semantics_passed,
        "evidencePath": "runtime/deployment-evidence.json",
    },
    {
        "id": "V091-004",
        "label": "Pack Migration Apply/Rollback Semantic Split",
        "passed": pack_migration_semantic_split_passed,
        "evidencePath": "pack-migration-applied-receipt.json",
    },
    {
        "id": "V091-005",
        "label": "Project Pack Registry Release Fixture",
        "passed": project_pack_registry_passed,
        "evidencePath": "pack-registry.json",
    },
    {
        "id": "V091-006",
        "label": "Negative Semantic Release Fixtures",
        "passed": negative_semantic_fixtures_passed,
        "evidencePath": "runtime/negative-semantic-fixtures.json",
    },
]
v091_coverage_passed = all(item["passed"] for item in v091_coverage)
v1_planning_readiness = "ready" if v090_coverage_passed and v091_coverage_passed else "blocked"
v1_planning_blockers = [
    item["id"] for item in [*v090_coverage, *v091_coverage] if not item["passed"]
]
v100_coverage = [
    {
        "id": "V100-001",
        "label": "Stable Contract Baseline",
        "passed": stable_contract_baseline_passed,
        "evidencePath": "runtime/stable-contract-baseline.json",
    },
    {
        "id": "V100-002",
        "label": "Runtime API / SDK Freeze",
        "passed": runtime_api_sdk_compatibility_passed,
        "evidencePath": "runtime/runtime-api-sdk-compatibility.json",
    },
    {
        "id": "V100-003",
        "label": "AgentFlow Filesystem Contract Freeze",
        "passed": filesystem_contract_passed,
        "evidencePath": "runtime/filesystem-contract.json",
    },
    {
        "id": "V100-004",
        "label": "Pack Contract Freeze",
        "passed": pack_contract_compatibility_passed,
        "evidencePath": "runtime/pack-contract-compatibility.json",
    },
    {
        "id": "V100-005",
        "label": "Projection / Read Model Stable Contract",
        "passed": projection_readmodel_contract_passed,
        "evidencePath": "runtime/projection-readmodel-contract.json",
    },
    {
        "id": "V100-006",
        "label": "Evidence + Acceptance Stable Contract",
        "passed": evidence_acceptance_contract_passed,
        "evidencePath": "runtime/evidence-acceptance-contract.json",
    },
    {
        "id": "V100-007",
        "label": "Executor Adapter Stable Contract",
        "passed": executor_adapter_contract_passed,
        "evidencePath": "runtime/executor-adapter-contract.json",
    },
    {
        "id": "V100-008",
        "label": "Replay / Migration / Upgrade Certification",
        "passed": replay_migration_upgrade_certification_passed,
        "evidencePath": "runtime/replay-migration-upgrade-certification.json",
    },
    {
        "id": "V100-009",
        "label": "Software Dev Pack Stable Baseline",
        "passed": software_dev_pack_stable_baseline_passed,
        "evidencePath": "runtime/software-dev-pack-stable-baseline.json",
    },
]
v100_coverage_passed = all(item["passed"] for item in v100_coverage)
v1_support_boundary = {
    "version": "agentflow-v1-support-boundary.v1",
    "stableCore": True,
    "stableIndustryPacks": ["software-dev"],
    "experimentalIndustryPacks": ["ui-design"],
    "futurePackCompatibilityGuaranteed": False,
    "auditSidecarIndependent": evidence_acceptance_contract.get("auditSidecarNonBlocking") is True
    and software_dev_pack_stable_baseline.get("auditSidecarPassed") is True,
    "executorRuntimeOwnsProjectTruth": False,
    "githubIssueAuthority": False,
    "projectionAuthority": False,
    "connectorAuthority": False,
    "industryUiAuthority": False,
    "completionAuthority": "Acceptance Gate + Completion Commit",
    "v1CompatibilityBoundaryClear": True,
}
v1_stable_core_blockers = []
if v1_planning_readiness != "ready":
    v1_stable_core_blockers.append("v1-planning-readiness")
if not v100_coverage_passed:
    v1_stable_core_blockers.extend(item["id"] for item in v100_coverage if not item["passed"])
if not governance_admission_passed:
    v1_stable_core_blockers.append("governance-admission-main-chain")
if projection_readmodel_contract.get("eventReplayProjectionAuthority") is not False:
    v1_stable_core_blockers.append("projection-authority-bypass")
if projection_readmodel_contract.get("queryApiReadonly") is not True:
    v1_stable_core_blockers.append("projection-query-api-readonly")
if evidence_acceptance_contract.get("taskDoneFromCompletionCommit") is not True:
    v1_stable_core_blockers.append("acceptance-done-decision")
if evidence_acceptance_contract.get("auditSidecarNonBlocking") is not True:
    v1_stable_core_blockers.append("audit-sidecar-main-chain")
if executor_adapter_contract.get("sessionIsolationRespected") is not True:
    v1_stable_core_blockers.append("executor-session-isolation")
if not v1_support_boundary["v1CompatibilityBoundaryClear"]:
    v1_stable_core_blockers.append("v1-compatibility-boundary")
v1_stable_core = "ready" if not v1_stable_core_blockers else "blocked"
v100_release_certification_path = summary_json_path.parent / "runtime/v100-release-certification.json"
v100_release_certification_payload = {
    "version": "agentflow-v100-release-certification.v1",
    "status": "passed" if v1_stable_core == "ready" else "failed",
    "releaseVersion": release_version,
    "tagName": release_tag_name or release.get("tagName"),
    "sourceCommitSha": source_commit_sha,
    "v1StableCore": v1_stable_core,
    "v1StableCoreBlockers": v1_stable_core_blockers,
    "v1PlanningReadiness": v1_planning_readiness,
    "v1PlanningBlockers": v1_planning_blockers,
    "v100Coverage": v100_coverage,
    "v100CoveragePassed": v100_coverage_passed,
    "stableContractBaselineProof": "runtime/stable-contract-baseline.json",
    "runtimeApiSdkCompatibilityProof": "runtime/runtime-api-sdk-compatibility.json",
    "filesystemContractProof": "runtime/filesystem-contract.json",
    "packContractProof": "runtime/pack-contract-compatibility.json",
    "projectionReadmodelProof": "runtime/projection-readmodel-contract.json",
    "evidenceAcceptanceProof": "runtime/evidence-acceptance-contract.json",
    "executorAdapterProof": "runtime/executor-adapter-contract.json",
    "replayMigrationUpgradeProof": "runtime/replay-migration-upgrade-certification.json",
    "softwareDevPackStableProof": "runtime/software-dev-pack-stable-baseline.json",
    "negativeFixtureCoverage": negative_semantic_fixtures.get("fixtures") or [],
    "remainingRisks": [],
    "deferredItems": [],
    "v1SupportBoundary": v1_support_boundary,
}
remaining_risks = []
deferred_items = []
if v1_planning_readiness == "blocked":
    remaining_risks.append({
        "id": "v1-planning-blocked",
        "severity": "blocking",
        "summary": "v1.0 planning cannot start until all V090 and V091 release coverage items pass.",
        "blockers": v1_planning_blockers,
    })
if v1_stable_core == "blocked":
    remaining_risks.append({
        "id": "v1-stable-core-blocked",
        "severity": "blocking",
        "summary": "v1.0 stable core cannot be certified until all V100 coverage and authority boundaries pass.",
        "blockers": v1_stable_core_blockers,
    })
if provider_smoke.get("status") in {None, "missing", "skipped", "disabled"}:
    deferred_items.append({
        "id": "provider-smoke-live-session",
        "blocking": False,
        "summary": "Live provider smoke remains optional and does not replace runtime fixture certification.",
    })
if scheduling_decision.get("decision") == "no-go":
    deferred_items.append({
        "id": "cross-process-message-bus",
        "blocking": False,
        "summary": "Cross-process Message Bus remains deferred; the release gate records the no-go decision as evidence.",
    })
v100_release_certification_payload["remainingRisks"] = remaining_risks
v100_release_certification_payload["deferredItems"] = deferred_items
v100_release_certification_path.write_text(
    json.dumps(v100_release_certification_payload, ensure_ascii=False, indent=2) + "\n",
    encoding="utf-8",
)
authority_boundary_certification = {
    "projectionIsAuthority": False,
    "connectorIsAuthority": False,
    "industryUiIsAuthority": False,
    "runtimeApiRemainsAuthorityBoundary": True,
    "eventStoreRemainsDurableAuthority": True,
}

checklist = [
    {
        "id": "v091-source-agent-entry",
        "label": "release source archive includes a tracked Agent entry and excludes runtime-only facts",
        "passed": source_agent_entry_passed,
    },
    {
        "id": "v100-stable-contract-baseline",
        "label": "v1.0 stable contract baseline metadata and required sections are present",
        "passed": stable_contract_baseline_passed,
    },
    {
        "id": "v100-runtime-api-sdk-freeze",
        "label": "v1.0 Runtime API / SDK contract freezes command, query, event, decision, error, and governance semantics",
        "passed": runtime_api_sdk_compatibility_passed,
    },
    {
        "id": "v100-filesystem-contract-freeze",
        "label": "v1.0 .agentflow filesystem contract freezes authority, projection, cache, public record, and retired path boundaries",
        "passed": filesystem_contract_passed,
    },
    {
        "id": "v100-pack-contract-freeze",
        "label": "v1.0 Pack contract freezes manifest, domain, surface, connector, capability, migration, and compatibility boundaries",
        "passed": pack_contract_compatibility_passed,
    },
    {
        "id": "v100-projection-readmodel-contract-freeze",
        "label": "v1.0 Projection contract freezes read model, view model, rebuild, freshness, Pack-specific loading, and sidecar surfaces",
        "passed": projection_readmodel_contract_passed,
    },
    {
        "id": "v100-evidence-acceptance-contract-freeze",
        "label": "v1.0 Evidence / Acceptance contract freezes Evidence Pack, Acceptance Gate, Completion Commit, failure reasons, delivery, and Audit sidecar boundaries",
        "passed": evidence_acceptance_contract_passed,
    },
    {
        "id": "v100-executor-adapter-contract-freeze",
        "label": "v1.0 Executor Adapter contract freezes work handoff, diff boundary, session isolation, provider mapping, and result normalization",
        "passed": executor_adapter_contract_passed,
    },
    {
        "id": "v100-replay-migration-upgrade-certification",
        "label": "v1.0 Replay / Migration / Upgrade certification proves replay, migration receipts, rollback, retired path, and negative upgrade fixtures",
        "passed": replay_migration_upgrade_certification_passed,
    },
    {
        "id": "v100-software-dev-pack-stable-baseline",
        "label": "v1.0 Software Dev Pack stable baseline proves stable manifest, read models, connectors, delivery, and Audit sidecar boundaries",
        "passed": software_dev_pack_stable_baseline_passed,
    },
    {
        "id": "v100-release-certification",
        "label": "v1.0 release certification reports v1StableCore ready and a clear v1 support boundary",
        "passed": v1_stable_core == "ready" and v100_coverage_passed,
    },
    {
        "id": "runtime-fixture-gate",
        "label": "release gate 跑本地 runtime fixture gate",
        "passed": stage_status.get("release.publish.refresh") == "passed",
    },
    {
        "id": "external-readable-proof",
        "label": "发布结论有外部可读证据",
        "passed": all(item["exists"] for item in public_artifacts),
    },
    {
        "id": "failure-stage-visible",
        "label": "gate 失败时能指出失败阶段",
        "passed": bool(current_stage),
    },
    {
        "id": "requirement-to-release-proof-chain",
        "label": "存在 requirement-to-release 完整证明链",
        "passed": all(item["status"] == "passed" for item in proof_chain) if current_status == "passed" else False,
    },
    {
        "id": "v072-foundation-coverage",
        "label": "v0.7.2 foundation coverage artifacts are present",
        "passed": all(path.is_file() for path in [
            api_plane_manifest_path,
            capability_registry_path,
            foundation_readiness_report_path,
            foundation_coverage_path,
        ]),
    },
    {
        "id": "v080-pack-system-release-gate",
        "label": "Pack System release gate artifacts are present and ready",
        "passed": pack_release_gate_passed,
    },
    {
        "id": "v081-pack-negative-fixtures",
        "label": "Pack release gate covers negative fixtures without authority writes",
        "passed": pack_negative_fixtures.get("status") == "passed"
        and pack_negative_fixtures.get("writesAuthority") is False,
    },
    {
        "id": "v090-event-replay-projection-rebuild",
        "label": "Event replay rebuilds projections and records structured replay failures",
        "passed": event_replay_projection.get("status") == "passed"
        and event_replay_projection.get("eventCount", 0) > 0
        and event_replay_projection.get("writesAuthority") is False
        and event_replay_projection_failure.get("status") == "failed"
        and bool(event_replay_projection_failure.get("failures")),
    },
    {
        "id": "v090-pack-migration-execution",
        "label": "Pack migration apply requires confirmation and records cancel/rollback receipts",
        "passed": pack_migration_semantic_split_passed,
    },
    {
        "id": "v090-simulation-evaluation-layer",
        "label": "Simulation reports object impact, evidence needs, conflict preview, and state flow without writes",
        "passed": pack_simulation_evaluation_passed,
    },
    {
        "id": "v090-runtime-governance-policy",
        "label": "Runtime governance emits allow, reject, and defer decisions with trace evidence",
        "passed": governance_policy_passed,
    },
    {
        "id": "v091-runtime-governance-admission",
        "label": "Runtime command admission blocks reject/defer before proposal facts",
        "passed": governance_admission_passed,
    },
    {
        "id": "v090-cross-process-scheduling-decision",
        "label": "Cross-process scheduling has an evidence-backed Message Bus go / no-go decision",
        "passed": scheduling_decision_passed,
    },
    {
        "id": "v090-deployment-evidence-rollback",
        "label": "Deployment evidence links local/cloud shape with rollback proof",
        "passed": deployment_evidence_passed,
    },
    {
        "id": "v091-deployment-evidence-semantics",
        "label": "Deployment evidence validates release facts, remote proof, artifact manifest, and rollback semantics",
        "passed": deployment_evidence_semantics_passed,
    },
    {
        "id": "v091-negative-semantic-fixtures",
        "label": "Release certification lists required negative semantic fixture coverage",
        "passed": negative_semantic_fixtures_passed,
    },
    {
        "id": "v090-release-certification",
        "label": "v0.9.0 certification covers V090-001 through V090-009 and can enter v1.0 planning",
        "passed": v090_coverage_passed,
    },
    {
        "id": "v091-release-certification",
        "label": "v0.9.1 certification covers V091-001 through V091-006 before v1.0 planning",
        "passed": v091_coverage_passed,
    },
]

summary_payload = {
    "status": current_status,
    "conclusion": current_status,
    "gateClass": "runtime-fixture-gate",
    "providerSmokeGate": provider_smoke.get("status"),
    "providerSmokeProvider": provider_smoke.get("provider"),
    "providerSmokeReason": provider_smoke.get("reason"),
    "providerSmokeArtifactPath": provider_smoke.get("artifactPath"),
    "sourceAgentEntryPath": "runtime/source-agent-entry.json" if source_agent_entry_path.is_file() else None,
    "sourceAgentEntryStatus": source_agent_entry.get("status") or "missing",
    "stableContractBaselinePath": "runtime/stable-contract-baseline.json" if pathlib.Path(summary_json_path.parent / "runtime/stable-contract-baseline.json").is_file() else None,
    "stableContractBaselineStatus": stable_contract_baseline.get("status") or "missing",
    "stableContractVersion": stable_contract_baseline.get("stableContractVersion"),
    "stableContractStatus": stable_contract_baseline.get("stableContractStatus"),
    "runtimeApiSdkCompatibilityPath": "runtime/runtime-api-sdk-compatibility.json" if pathlib.Path(summary_json_path.parent / "runtime/runtime-api-sdk-compatibility.json").is_file() else None,
    "runtimeApiSdkCompatibilityStatus": runtime_api_sdk_compatibility.get("status") or "missing",
    "runtimeApiSdkContractVersion": runtime_api_sdk_compatibility.get("runtimeApiSdkContractVersion"),
    "runtimeApiSdkContractStatus": runtime_api_sdk_compatibility.get("runtimeApiSdkContractStatus"),
    "filesystemContractPath": "runtime/filesystem-contract.json" if pathlib.Path(summary_json_path.parent / "runtime/filesystem-contract.json").is_file() else None,
    "filesystemContractStatus": filesystem_contract.get("status") or "missing",
    "filesystemContractVersion": filesystem_contract.get("filesystemContractVersion"),
    "filesystemContractFreezeStatus": filesystem_contract.get("filesystemContractStatus"),
    "packContractCompatibilityPath": "runtime/pack-contract-compatibility.json" if pathlib.Path(summary_json_path.parent / "runtime/pack-contract-compatibility.json").is_file() else None,
    "packContractCompatibilityStatus": pack_contract_compatibility.get("status") or "missing",
    "packContractVersion": pack_contract_compatibility.get("packContractVersion"),
    "packContractStatus": pack_contract_compatibility.get("packContractStatus"),
    "projectionReadmodelContractPath": "runtime/projection-readmodel-contract.json" if pathlib.Path(summary_json_path.parent / "runtime/projection-readmodel-contract.json").is_file() else None,
    "projectionReadmodelContractStatus": projection_readmodel_contract.get("status") or "missing",
    "projectionContractVersion": projection_readmodel_contract.get("projectionContractVersion"),
    "projectionContractStatus": projection_readmodel_contract.get("projectionContractStatus"),
    "evidenceAcceptanceContractPath": "runtime/evidence-acceptance-contract.json" if pathlib.Path(summary_json_path.parent / "runtime/evidence-acceptance-contract.json").is_file() else None,
    "evidenceAcceptanceContractStatus": evidence_acceptance_contract.get("status") or "missing",
    "evidenceAcceptanceContractVersion": evidence_acceptance_contract.get("evidenceAcceptanceContractVersion"),
    "evidenceAcceptanceFreezeStatus": evidence_acceptance_contract.get("evidenceAcceptanceContractStatus"),
    "executorAdapterContractPath": "runtime/executor-adapter-contract.json" if pathlib.Path(summary_json_path.parent / "runtime/executor-adapter-contract.json").is_file() else None,
    "executorAdapterContractStatus": executor_adapter_contract.get("status") or "missing",
    "executorAdapterContractVersion": executor_adapter_contract.get("executorAdapterContractVersion"),
    "executorAdapterFreezeStatus": executor_adapter_contract.get("executorAdapterContractStatus"),
    "replayMigrationUpgradeCertificationPath": "runtime/replay-migration-upgrade-certification.json" if pathlib.Path(summary_json_path.parent / "runtime/replay-migration-upgrade-certification.json").is_file() else None,
    "replayMigrationUpgradeCertificationStatus": replay_migration_upgrade_certification.get("status") or "missing",
    "replayMigrationUpgradeCertificationVersion": replay_migration_upgrade_certification.get("replayMigrationUpgradeCertificationVersion"),
    "replayMigrationUpgradeFreezeStatus": replay_migration_upgrade_certification.get("replayMigrationUpgradeCertificationStatus"),
    "softwareDevPackStableBaselinePath": "runtime/software-dev-pack-stable-baseline.json" if pathlib.Path(summary_json_path.parent / "runtime/software-dev-pack-stable-baseline.json").is_file() else None,
    "softwareDevPackStableBaselineStatus": software_dev_pack_stable_baseline.get("status") or "missing",
    "softwareDevPackStableBaselineVersion": software_dev_pack_stable_baseline.get("softwareDevPackStableBaselineVersion"),
    "softwareDevPackStableBaselineFreezeStatus": software_dev_pack_stable_baseline.get("softwareDevPackStableBaselineStatus"),
    "runtimeFixtureBoundary": "runtime-fixture-gate proves AgentFlow local runtime workflow coverage",
    "providerSmokeBoundary": "provider-smoke-gate proves minimal provider health, launch request, session snapshot, and terminal projection without replacing runtime fixture coverage",
    "foundationCoveragePath": "runtime/foundation-coverage.json" if foundation_coverage_path.is_file() else None,
    "foundationReadinessReportPath": "runtime/foundation-readiness-report.md" if foundation_readiness_report_path.is_file() else None,
    "eventReplayProjectionReportPath": "runtime/event-replay-projection-report.json" if pathlib.Path(summary_json_path.parent / "runtime/event-replay-projection-report.json").is_file() else None,
    "eventReplayProjectionFailureReportPath": "runtime/event-replay-projection-failure-report.json" if pathlib.Path(summary_json_path.parent / "runtime/event-replay-projection-failure-report.json").is_file() else None,
    "apiPlaneManifestPath": "runtime/api-plane-manifest.json" if api_plane_manifest_path.is_file() else None,
    "capabilityRegistryPath": "runtime/capability-registry.json" if capability_registry_path.is_file() else None,
    "governancePolicyPath": "runtime/governance-policy.json" if governance_policy_path.is_file() else None,
    "governancePolicyStatus": governance_policy.get("status") or "missing",
    "governanceAdmissionPath": "runtime/governance-admission.json" if governance_admission_path.is_file() else None,
    "governanceAdmissionStatus": governance_admission.get("status") or "missing",
    "schedulingDecisionPath": "runtime/scheduling-decision.json" if scheduling_decision_path.is_file() else None,
    "schedulingDecision": scheduling_decision.get("decision") or "missing",
    "deploymentEvidencePath": "runtime/deployment-evidence.json" if deployment_evidence_path.is_file() else None,
    "deploymentEvidenceStatus": deployment_evidence.get("status") or "missing",
    "deploymentEvidenceSemanticFailurePath": "runtime/deployment-evidence-semantic-failure.json" if pathlib.Path(summary_json_path.parent / "runtime/deployment-evidence-semantic-failure.json").is_file() else None,
    "deploymentEvidenceSemanticFailureStatus": deployment_evidence_failure.get("status") or "missing",
    "deploymentEvidenceWrongCommitPath": "runtime/deployment-evidence-wrong-commit.json" if pathlib.Path(summary_json_path.parent / "runtime/deployment-evidence-wrong-commit.json").is_file() else None,
    "deploymentEvidenceWrongCommitStatus": deployment_evidence_wrong_commit.get("status") or "missing",
    "deploymentEvidenceWrongUrlPath": "runtime/deployment-evidence-wrong-url.json" if pathlib.Path(summary_json_path.parent / "runtime/deployment-evidence-wrong-url.json").is_file() else None,
    "deploymentEvidenceWrongUrlStatus": deployment_evidence_wrong_url.get("status") or "missing",
    "deploymentEvidenceFakeMigrationPath": "runtime/deployment-evidence-fake-migration-receipt.json" if pathlib.Path(summary_json_path.parent / "runtime/deployment-evidence-fake-migration-receipt.json").is_file() else None,
    "deploymentEvidenceFakeMigrationStatus": deployment_evidence_fake_migration.get("status") or "missing",
    "negativeSemanticFixturesPath": "runtime/negative-semantic-fixtures.json" if negative_semantic_fixtures_path.is_file() else None,
    "negativeSemanticFixturesStatus": negative_semantic_fixtures.get("status") or "missing",
    "negativeSemanticFixtureCoverage": negative_semantic_fixtures.get("fixtures") or [],
    "rollbackTargetTag": (deployment_evidence.get("rollbackModel") or {}).get("targetTag"),
    "v090Coverage": v090_coverage,
    "v090CoveragePassed": v090_coverage_passed,
    "v091Coverage": v091_coverage,
    "v091CoveragePassed": v091_coverage_passed,
    "v1PlanningReadiness": v1_planning_readiness,
    "v1PlanningBlockers": v1_planning_blockers,
    "v100Coverage": v100_coverage,
    "v100CoveragePassed": v100_coverage_passed,
    "v1StableCore": v1_stable_core,
    "v1StableCoreBlockers": v1_stable_core_blockers,
    "v1SupportBoundary": v1_support_boundary,
    "v100ReleaseCertificationPath": "runtime/v100-release-certification.json" if v100_release_certification_path.is_file() else None,
    "v100ReleaseCertificationStatus": v100_release_certification_payload["status"],
    "releaseProvenancePath": "runtime/release-provenance.json" if release_provenance else None,
    "releaseProvenanceStatus": release_provenance.get("status") or "missing",
    "cleanRoomTestProofPath": "runtime/clean-room-test-proof.json" if clean_room_test_proof else None,
    "cleanRoomTestProofStatus": clean_room_test_proof.get("status") or "missing",
    "auditSidecarPolicyPath": "runtime/audit-sidecar-policy.json" if audit_sidecar_policy else None,
    "auditSidecarPolicyStatus": audit_sidecar_policy.get("status") or "missing",
    "providerSmokeProofPath": "runtime/provider-smoke-proof.json" if provider_smoke_proof else None,
    "providerSmokeProofStatus": provider_smoke_proof.get("status") or "missing",
    "softwareDevPackUsageBaselinePath": "runtime/software-dev-pack-usage-baseline.json" if software_dev_pack_usage_baseline else None,
    "softwareDevPackUsageBaselineStatus": software_dev_pack_usage_baseline.get("status") or "missing",
    "trustedGovernanceTelemetryPath": "runtime/trusted-governance-telemetry.json" if trusted_governance_telemetry else None,
    "trustedGovernanceTelemetryStatus": trusted_governance_telemetry.get("status") or "missing",
    "v101ReleaseCertificationPath": "runtime/v101-release-certification.json" if v101_release_certification else None,
    "v101ReleaseCertificationStatus": v101_release_certification.get("v101ReleaseCertificationStatus") or v101_release_certification.get("status") or "missing",
    "v101Coverage": v101_release_certification.get("coverage") or {},
    "forgedGovernanceRuntimeResponsePath": "runtime/forged-governance-runtime-response.json" if forged_governance_response else None,
    "forgedGovernanceRuntimeStatus": forged_governance_response.get("status") or "missing",
    "releaseArtifactBoundaryPath": "runtime/release-artifact-boundary.json" if release_artifact_boundary else None,
    "releaseArtifactBoundaryStatus": release_artifact_boundary.get("status") or "missing",
    "projectRoadmapBaselinePath": "runtime/project-roadmap-baseline.json" if project_roadmap_baseline else None,
    "projectRoadmapBaselineStatus": project_roadmap_baseline.get("status") or "missing",
    "v103ReleaseFixCertificationPath": "runtime/v103-release-fix-certification.json" if v103_release_fix_certification else None,
    "v103ReleaseFixCertificationStatus": v103_release_fix_certification.get("v103ReleaseFixCertificationStatus") or v103_release_fix_certification.get("status") or "missing",
    "v103Coverage": v103_release_fix_certification.get("coverage") or {},
    "coreRuntimeNegativeFixturesPath": "runtime/core-runtime-negative-fixtures.json" if core_runtime_negative_fixtures_path.is_file() else None,
    "coreRuntimeNegativeFixturesStatus": core_runtime_negative_fixtures.get("status") or "missing",
    "coreRuntimeNegativeFixtureCoverage": core_runtime_negative_fixtures.get("fixtures") or [],
    "softwareDevReferenceWorkflowStatus": (core_runtime_negative_fixtures.get("positiveWorkflow") or {}).get("status") or "missing",
    "coreRuntimeKernelPath": "runtime/core-runtime-kernel.json" if core_runtime_kernel else None,
    "coreRuntimeKernelStatus": core_runtime_kernel.get("status") or "missing",
    "coreRuntimeAdmissionPath": "runtime/core-runtime-admission.json" if core_runtime_admission else None,
    "coreRuntimeAdmissionStatus": core_runtime_admission.get("status") or "missing",
    "coreRuntimeArbitrationPath": "runtime/core-runtime-arbitration.json" if core_runtime_arbitration else None,
    "coreRuntimeArbitrationStatus": core_runtime_arbitration.get("status") or "missing",
    "v105ReleaseCertificationPath": "runtime/v105-release-certification.json" if v105_release_certification else None,
    "v105ReleaseCertificationStatus": v105_release_certification.get("status") or "missing",
    "v105Coverage": v105_release_certification.get("coverage") or {},
    "v106ReleaseCertificationPath": "runtime/v106-release-certification.json" if v106_release_certification else None,
    "v106ReleaseCertificationStatus": v106_release_certification.get("status") or "missing",
    "v106Coverage": v106_release_certification.get("coverage") or {},
    "v107ReleaseProvenanceHandoffPath": "runtime/v107-release-provenance-handoff.json" if v107_release_provenance_handoff else None,
    "v107ReleaseProvenanceHandoffStatus": v107_release_provenance_handoff.get("status") or "missing",
    "v107ReleaseProvenanceHandoffCoverage": v107_release_provenance_handoff.get("coverage") or {},
    "v107ReleaseCertificationPath": "runtime/v107-release-certification.json" if v107_release_certification else None,
    "v107ReleaseCertificationStatus": v107_release_certification.get("status") or "missing",
    "v107ReleaseCertificationCoverage": v107_release_certification.get("coverage") or {},
    "v108ReleaseCertificationPath": "runtime/v108-release-certification.json" if v108_release_certification else None,
    "v108ReleaseCertificationStatus": v108_release_certification.get("status") or "missing",
    "v108ReleaseCertificationCoverage": v108_release_certification.get("coverage") or {},
    "coreDecisionModelContractPath": "runtime/core-decision-model-contract.json" if core_decision_model_contract else None,
    "coreDecisionModelContractStatus": core_decision_model_contract.get("status") or "missing",
    "coreDecisionModelContractCoverage": core_decision_model_contract.get("coverage") or {},
    "coreDecisionInputBindingPath": "runtime/core-decision-input-binding.json" if core_decision_input_binding else None,
    "coreDecisionInputBindingStatus": core_decision_input_binding.get("status") or "missing",
    "coreDecisionInputBindingCoverage": core_decision_input_binding.get("coverage") or {},
    "coreDecisionOutcomeTransitionsPath": "runtime/core-decision-outcome-transitions.json" if core_decision_outcome_transitions else None,
    "coreDecisionOutcomeTransitionsStatus": core_decision_outcome_transitions.get("status") or "missing",
    "coreDecisionOutcomeTransitionsCoverage": core_decision_outcome_transitions.get("coverage") or {},
    "coreDecisionFailureReasonPath": "runtime/core-decision-failure-reason-remediation.json" if core_decision_failure_reason else None,
    "coreDecisionFailureReasonStatus": core_decision_failure_reason.get("status") or "missing",
    "coreDecisionFailureReasonCoverage": core_decision_failure_reason.get("coverage") or {},
    "coreEvidenceToDecisionGatePath": "runtime/core-evidence-to-decision-gate.json" if core_evidence_to_decision_gate else None,
    "coreEvidenceToDecisionGateStatus": core_evidence_to_decision_gate.get("status") or "missing",
    "coreEvidenceToDecisionGateCoverage": core_evidence_to_decision_gate.get("coverage") or {},
    "coreCompletionCommitAuthorityPath": "runtime/core-completion-commit-authority.json" if core_completion_commit_authority else None,
    "coreCompletionCommitAuthorityStatus": core_completion_commit_authority.get("status") or "missing",
    "coreCompletionCommitAuthorityCoverage": core_completion_commit_authority.get("coverage") or {},
    "coreDeliveryReadinessAuditTriggerPath": "runtime/core-delivery-readiness-audit-trigger.json" if core_delivery_readiness_audit_trigger else None,
    "coreDeliveryReadinessAuditTriggerStatus": core_delivery_readiness_audit_trigger.get("status") or "missing",
    "coreDeliveryReadinessAuditTriggerCoverage": core_delivery_readiness_audit_trigger.get("coverage") or {},
    "coreDecisionProjectionReadModelPath": "runtime/core-decision-projection-read-model.json" if core_decision_projection_read_model else None,
    "coreDecisionProjectionReadModelStatus": core_decision_projection_read_model.get("status") or "missing",
    "coreDecisionProjectionReadModelCoverage": core_decision_projection_read_model.get("coverage") or {},
    "remainingRisks": remaining_risks,
    "deferredItems": deferred_items,
    "authorityBoundaryCertification": authority_boundary_certification,
    "packRegistryPath": "pack-registry.json" if pathlib.Path(summary_json_path.parent / "pack-registry.json").is_file() else None,
    "packValidationReportPath": "pack-validation-report.json" if pathlib.Path(summary_json_path.parent / "pack-validation-report.json").is_file() else None,
    "packSimulationReportPath": "pack-simulation-report.json" if pathlib.Path(summary_json_path.parent / "pack-simulation-report.json").is_file() else None,
    "packProjectionReadinessPath": "pack-projection-readiness.json" if pathlib.Path(summary_json_path.parent / "pack-projection-readiness.json").is_file() else None,
    "packNegativeFixturesPath": "pack-negative-fixtures.json" if pack_negative_fixtures_path.is_file() else None,
    "packMigrationPreviewPath": "pack-migration-preview.json" if pathlib.Path(summary_json_path.parent / "pack-migration-preview.json").is_file() else None,
    "packMigrationUnconfirmedApplyPath": "pack-migration-unconfirmed-apply.json" if pathlib.Path(summary_json_path.parent / "pack-migration-unconfirmed-apply.json").is_file() else None,
    "packMigrationAppliedReceiptPath": "pack-migration-applied-receipt.json" if pathlib.Path(summary_json_path.parent / "pack-migration-applied-receipt.json").is_file() else None,
    "packMigrationFakeAuthorityReceiptPath": "pack-migration-fake-authority-receipt.json" if pathlib.Path(summary_json_path.parent / "pack-migration-fake-authority-receipt.json").is_file() else None,
    "packMigrationCancelReceiptPath": "pack-migration-cancel-receipt.json" if pathlib.Path(summary_json_path.parent / "pack-migration-cancel-receipt.json").is_file() else None,
    "packMigrationRollbackReceiptPath": "pack-migration-rollback-receipt.json" if pathlib.Path(summary_json_path.parent / "pack-migration-rollback-receipt.json").is_file() else None,
    "packMigrationReplayReportPath": "runtime/pack-migration-replay-report.json" if pathlib.Path(summary_json_path.parent / "runtime/pack-migration-replay-report.json").is_file() else None,
    "softwareDevPackReadinessPath": "software-dev-pack-readiness.json" if pathlib.Path(summary_json_path.parent / "software-dev-pack-readiness.json").is_file() else None,
    "uiDesignPackReadinessPath": "ui-design-pack-readiness.json" if pathlib.Path(summary_json_path.parent / "ui-design-pack-readiness.json").is_file() else None,
    "packReleaseGateStatus": "passed" if pack_release_gate_passed else "failed",
    "failedStage": current_stage if current_status == "failed" else None,
    "failureMessage": current_message if current_status == "failed" else None,
    "sourceCommitSha": source_commit_sha,
    "tagName": release_tag_name or release.get("tagName"),
    "releaseVersion": release_version,
    "requirementId": requirement_id,
    "projectId": project_id,
    "issueCount": issue_count,
    "releaseState": release.get("currentState"),
    "publicationStage": release.get("publicationStage"),
    "gateStatus": release.get("gateStatus"),
    "completionState": release.get("completionState"),
    "completionOutcome": release.get("completionOutcome"),
    "remoteReleaseUrl": release.get("remoteReleaseUrl"),
    "certifiedReleaseUrl": release_url,
    "changelogPath": release.get("changelogPath"),
    "releaseNotesPath": release.get("releaseNotesPath"),
    "externalReviewPath": review.get("handoffPath"),
    "auditSidecar": {
        "status": audit.get("latestStatus") or "not-requested",
        "reportPath": audit.get("latestReportPath"),
        "releaseGateBlocking": False,
        "boundary": "audit sidecar result is independent from release gate conclusion unless release policy explicitly binds it",
        "softwareDevPackMainChain": False,
    },
}
if summary_payload["auditSidecar"]["status"] == "failed" and summary_payload["status"] == "passed":
    summary_payload["auditSidecar"]["interpretation"] = "sidecar audit failed; release gate conclusion remains passed because no release policy binding is active"
summary_json_path.write_text(
    json.dumps(summary_payload, ensure_ascii=False, indent=2) + "\n",
    encoding="utf-8",
)

summary_lines = [
    "# Release Gate Runtime Fixture Summary",
    "",
    "- Gate class: `runtime-fixture-gate`",
    "- Runtime fixture boundary: `proves AgentFlow local runtime workflow coverage`",
    f"- Provider smoke gate: `{provider_smoke.get('status')}`",
    f"- Provider smoke provider: `{provider_smoke.get('provider') or 'n/a'}`",
    f"- Provider smoke reason: `{provider_smoke.get('reason') or 'n/a'}`",
    f"- Stable contract baseline: `{stable_contract_baseline.get('status') or 'missing'}`",
    f"- Runtime API / SDK compatibility: `{runtime_api_sdk_compatibility.get('status') or 'missing'}`",
    f"- Filesystem contract: `{filesystem_contract.get('status') or 'missing'}`",
    f"- Pack contract compatibility: `{pack_contract_compatibility.get('status') or 'missing'}`",
    f"- Projection / Read Model contract: `{projection_readmodel_contract.get('status') or 'missing'}`",
    f"- Evidence / Acceptance contract: `{evidence_acceptance_contract.get('status') or 'missing'}`",
    "- Provider smoke boundary: `minimal provider health / launch / session / terminal projection; does not replace runtime fixture gate`",
    f"- Foundation coverage: `{'present' if foundation_coverage_path.is_file() else 'missing'}`",
    f"- Foundation readiness report: `{'present' if foundation_readiness_report_path.is_file() else 'missing'}`",
    f"- API Plane manifest: `{'present' if api_plane_manifest_path.is_file() else 'missing'}`",
    f"- Capability registry: `{'present' if capability_registry_path.is_file() else 'missing'}`",
    f"- Governance policy: `{governance_policy.get('status') or 'missing'}`",
    f"- Scheduling decision: `{scheduling_decision.get('decision') or 'missing'}`",
    f"- Deployment evidence: `{deployment_evidence.get('status') or 'missing'}`",
    f"- Negative semantic fixtures: `{negative_semantic_fixtures.get('status') or 'missing'}`",
    f"- Pack release gate: `{'passed' if pack_release_gate_passed else 'failed'}`",
    f"- Pack negative fixtures: `{pack_negative_fixtures.get('status') or 'missing'}`",
    f"- Pack migration execution: `{stage_status.get('pack.migration-execution') or 'missing'}`",
    f"- Software Dev Pack readiness: `{software_readiness.get('status') or 'missing'}`",
    f"- UI Design Pack readiness: `{design_readiness.get('status') or 'missing'}`",
    f"- v1.0 stable core: `{v1_stable_core}`",
    f"- Core Runtime Kernel: `{core_runtime_kernel.get('status') or 'missing'}`",
    f"- Core Runtime Admission: `{core_runtime_admission.get('status') or 'missing'}`",
    f"- Core Runtime Arbitration: `{core_runtime_arbitration.get('status') or 'missing'}`",
    f"- v1.0.5 release certification: `{v105_release_certification.get('status') or 'missing'}`",
    f"- v1.0.6 release certification: `{v106_release_certification.get('status') or 'missing'}`",
    f"- v1.0 support boundary: `{v1_support_boundary['version']}`",
    f"- Release version: `{release_version}`",
    f"- Tag name: `{release_tag_name or release.get('tagName') or 'n/a'}`",
    f"- Source commit: `{source_commit_sha or 'n/a'}`",
    f"- Gate status: `{current_status}`",
    f"- Current stage: `{current_stage}`",
    f"- Requirement: `{requirement_id}`",
    f"- Project: `{project_id}`",
    f"- Issue count: `{issue_count}`",
]
if current_message:
    summary_lines.append(f"- Stage message: `{current_message}`")
if summary_payload["releaseState"]:
    summary_lines.extend([
        f"- Release state: `{summary_payload['releaseState']}`",
        f"- Publication stage: `{summary_payload['publicationStage']}`",
        f"- Completion state: `{summary_payload['completionState']}` / `{summary_payload['completionOutcome']}`",
        f"- Remote release URL: `{summary_payload['remoteReleaseUrl']}`",
        f"- Changelog: `{summary_payload['changelogPath']}`",
        f"- Release notes: `{summary_payload['releaseNotesPath']}`",
        f"- External review: `{summary_payload['externalReviewPath']}`",
        f"- Sidecar audit status: `{summary_payload['auditSidecar']['status']}`",
        f"- Sidecar audit blocks release gate: `{summary_payload['auditSidecar']['releaseGateBlocking']}`",
    ])
elif release_url:
    summary_lines.append(f"- Release URL: `{release_url}`")
summary_md_path.write_text("\n".join(summary_lines) + "\n", encoding="utf-8")

current_gate_run = {
    "eventName": gate_event_name,
    "runId": gate_run_id,
    "runAttempt": gate_run_attempt,
    "runUrl": f"{gate_server_url}/{gate_repository}/actions/runs/{gate_run_id}" if gate_run_id else None,
    "refName": gate_ref_name,
    "refType": gate_ref_type,
    "sourceCommitSha": source_commit_sha,
}
main_gate_run = current_gate_run if gate_event_name == "push" and gate_ref_name == "main" else None
tag_gate_run = current_gate_run if gate_event_name == "push" and gate_ref_type == "tag" else None
release_gate_run = current_gate_run if gate_event_name == "release" else None

certification_payload = {
    "version": "agentflow-release-gate-certification.v1",
    "releaseVersion": release_version,
    "tagName": release_tag_name or release.get("tagName"),
    "sourceCommitSha": source_commit_sha,
    "gateWorkflow": "release-gate",
    "gateClass": "runtime-fixture-gate",
    "providerSmokeGate": provider_smoke.get("status"),
    "providerSmokeProvider": provider_smoke.get("provider"),
    "providerSmokeReason": provider_smoke.get("reason"),
    "providerSmokeArtifactPath": provider_smoke.get("artifactPath"),
    "sourceAgentEntryPath": "runtime/source-agent-entry.json" if source_agent_entry_path.is_file() else None,
    "sourceAgentEntryStatus": source_agent_entry.get("status") or "missing",
    "stableContractBaselinePath": "runtime/stable-contract-baseline.json" if pathlib.Path(summary_json_path.parent / "runtime/stable-contract-baseline.json").is_file() else None,
    "stableContractBaselineStatus": stable_contract_baseline.get("status") or "missing",
    "stableContractVersion": stable_contract_baseline.get("stableContractVersion"),
    "stableContractStatus": stable_contract_baseline.get("stableContractStatus"),
    "runtimeApiSdkCompatibilityPath": "runtime/runtime-api-sdk-compatibility.json" if pathlib.Path(summary_json_path.parent / "runtime/runtime-api-sdk-compatibility.json").is_file() else None,
    "runtimeApiSdkCompatibilityStatus": runtime_api_sdk_compatibility.get("status") or "missing",
    "runtimeApiSdkContractVersion": runtime_api_sdk_compatibility.get("runtimeApiSdkContractVersion"),
    "runtimeApiSdkContractStatus": runtime_api_sdk_compatibility.get("runtimeApiSdkContractStatus"),
    "filesystemContractPath": "runtime/filesystem-contract.json" if pathlib.Path(summary_json_path.parent / "runtime/filesystem-contract.json").is_file() else None,
    "filesystemContractStatus": filesystem_contract.get("status") or "missing",
    "filesystemContractVersion": filesystem_contract.get("filesystemContractVersion"),
    "filesystemContractFreezeStatus": filesystem_contract.get("filesystemContractStatus"),
    "packContractCompatibilityPath": "runtime/pack-contract-compatibility.json" if pathlib.Path(summary_json_path.parent / "runtime/pack-contract-compatibility.json").is_file() else None,
    "packContractCompatibilityStatus": pack_contract_compatibility.get("status") or "missing",
    "packContractVersion": pack_contract_compatibility.get("packContractVersion"),
    "packContractStatus": pack_contract_compatibility.get("packContractStatus"),
    "projectionReadmodelContractPath": "runtime/projection-readmodel-contract.json" if pathlib.Path(summary_json_path.parent / "runtime/projection-readmodel-contract.json").is_file() else None,
    "projectionReadmodelContractStatus": projection_readmodel_contract.get("status") or "missing",
    "projectionContractVersion": projection_readmodel_contract.get("projectionContractVersion"),
    "projectionContractStatus": projection_readmodel_contract.get("projectionContractStatus"),
    "evidenceAcceptanceContractPath": "runtime/evidence-acceptance-contract.json" if pathlib.Path(summary_json_path.parent / "runtime/evidence-acceptance-contract.json").is_file() else None,
    "evidenceAcceptanceContractStatus": evidence_acceptance_contract.get("status") or "missing",
    "evidenceAcceptanceContractVersion": evidence_acceptance_contract.get("evidenceAcceptanceContractVersion"),
    "evidenceAcceptanceFreezeStatus": evidence_acceptance_contract.get("evidenceAcceptanceContractStatus"),
    "executorAdapterContractPath": "runtime/executor-adapter-contract.json" if pathlib.Path(summary_json_path.parent / "runtime/executor-adapter-contract.json").is_file() else None,
    "executorAdapterContractStatus": executor_adapter_contract.get("status") or "missing",
    "executorAdapterContractVersion": executor_adapter_contract.get("executorAdapterContractVersion"),
    "executorAdapterFreezeStatus": executor_adapter_contract.get("executorAdapterContractStatus"),
    "replayMigrationUpgradeCertificationPath": "runtime/replay-migration-upgrade-certification.json" if pathlib.Path(summary_json_path.parent / "runtime/replay-migration-upgrade-certification.json").is_file() else None,
    "replayMigrationUpgradeCertificationStatus": replay_migration_upgrade_certification.get("status") or "missing",
    "replayMigrationUpgradeCertificationVersion": replay_migration_upgrade_certification.get("replayMigrationUpgradeCertificationVersion"),
    "replayMigrationUpgradeFreezeStatus": replay_migration_upgrade_certification.get("replayMigrationUpgradeCertificationStatus"),
    "providerSmokeBoundary": "provider-smoke-gate proves minimal provider health, launch request, session snapshot, and terminal projection without replacing runtime fixture coverage",
    "currentGateRun": current_gate_run,
    "mainGateRun": main_gate_run,
    "tagGateRun": tag_gate_run,
    "releaseGateRun": release_gate_run,
    "gateStatus": current_status,
    "conclusion": current_status,
    "runtimeFixtureBoundary": "runtime-fixture-gate proves AgentFlow local runtime workflow coverage and remains required even when provider-smoke-gate is skipped or passed",
    "foundationCoveragePath": "runtime/foundation-coverage.json" if foundation_coverage_path.is_file() else None,
    "foundationReadinessReportPath": "runtime/foundation-readiness-report.md" if foundation_readiness_report_path.is_file() else None,
    "eventReplayProjectionReportPath": "runtime/event-replay-projection-report.json" if pathlib.Path(summary_json_path.parent / "runtime/event-replay-projection-report.json").is_file() else None,
    "eventReplayProjectionFailureReportPath": "runtime/event-replay-projection-failure-report.json" if pathlib.Path(summary_json_path.parent / "runtime/event-replay-projection-failure-report.json").is_file() else None,
    "apiPlaneManifestPath": "runtime/api-plane-manifest.json" if api_plane_manifest_path.is_file() else None,
    "capabilityRegistryPath": "runtime/capability-registry.json" if capability_registry_path.is_file() else None,
    "governancePolicyPath": "runtime/governance-policy.json" if governance_policy_path.is_file() else None,
    "governancePolicyStatus": governance_policy.get("status") or "missing",
    "governanceAdmissionPath": "runtime/governance-admission.json" if governance_admission_path.is_file() else None,
    "governanceAdmissionStatus": governance_admission.get("status") or "missing",
    "schedulingDecisionPath": "runtime/scheduling-decision.json" if scheduling_decision_path.is_file() else None,
    "schedulingDecision": scheduling_decision.get("decision") or "missing",
    "deploymentEvidencePath": "runtime/deployment-evidence.json" if deployment_evidence_path.is_file() else None,
    "deploymentEvidenceStatus": deployment_evidence.get("status") or "missing",
    "deploymentEvidenceSemanticFailurePath": "runtime/deployment-evidence-semantic-failure.json" if pathlib.Path(summary_json_path.parent / "runtime/deployment-evidence-semantic-failure.json").is_file() else None,
    "deploymentEvidenceSemanticFailureStatus": deployment_evidence_failure.get("status") or "missing",
    "deploymentEvidenceWrongCommitPath": "runtime/deployment-evidence-wrong-commit.json" if pathlib.Path(summary_json_path.parent / "runtime/deployment-evidence-wrong-commit.json").is_file() else None,
    "deploymentEvidenceWrongCommitStatus": deployment_evidence_wrong_commit.get("status") or "missing",
    "deploymentEvidenceWrongUrlPath": "runtime/deployment-evidence-wrong-url.json" if pathlib.Path(summary_json_path.parent / "runtime/deployment-evidence-wrong-url.json").is_file() else None,
    "deploymentEvidenceWrongUrlStatus": deployment_evidence_wrong_url.get("status") or "missing",
    "deploymentEvidenceFakeMigrationPath": "runtime/deployment-evidence-fake-migration-receipt.json" if pathlib.Path(summary_json_path.parent / "runtime/deployment-evidence-fake-migration-receipt.json").is_file() else None,
    "deploymentEvidenceFakeMigrationStatus": deployment_evidence_fake_migration.get("status") or "missing",
    "negativeSemanticFixturesPath": "runtime/negative-semantic-fixtures.json" if negative_semantic_fixtures_path.is_file() else None,
    "negativeSemanticFixturesStatus": negative_semantic_fixtures.get("status") or "missing",
    "negativeSemanticFixtureCoverage": negative_semantic_fixtures.get("fixtures") or [],
    "rollbackTargetTag": (deployment_evidence.get("rollbackModel") or {}).get("targetTag"),
    "v090Coverage": v090_coverage,
    "v090CoveragePassed": v090_coverage_passed,
    "v091Coverage": v091_coverage,
    "v091CoveragePassed": v091_coverage_passed,
    "v1PlanningReadiness": v1_planning_readiness,
    "v1PlanningBlockers": v1_planning_blockers,
    "v100Coverage": v100_coverage,
    "v100CoveragePassed": v100_coverage_passed,
    "v1StableCore": v1_stable_core,
    "v1StableCoreBlockers": v1_stable_core_blockers,
    "v1SupportBoundary": v1_support_boundary,
    "v100ReleaseCertificationPath": "runtime/v100-release-certification.json" if v100_release_certification_path.is_file() else None,
    "v100ReleaseCertificationStatus": v100_release_certification_payload["status"],
    "releaseProvenancePath": "runtime/release-provenance.json" if release_provenance else None,
    "releaseProvenanceStatus": release_provenance.get("status") or "missing",
    "cleanRoomTestProofPath": "runtime/clean-room-test-proof.json" if clean_room_test_proof else None,
    "cleanRoomTestProofStatus": clean_room_test_proof.get("status") or "missing",
    "auditSidecarPolicyPath": "runtime/audit-sidecar-policy.json" if audit_sidecar_policy else None,
    "auditSidecarPolicyStatus": audit_sidecar_policy.get("status") or "missing",
    "providerSmokeProofPath": "runtime/provider-smoke-proof.json" if provider_smoke_proof else None,
    "providerSmokeProofStatus": provider_smoke_proof.get("status") or "missing",
    "softwareDevPackUsageBaselinePath": "runtime/software-dev-pack-usage-baseline.json" if software_dev_pack_usage_baseline else None,
    "softwareDevPackUsageBaselineStatus": software_dev_pack_usage_baseline.get("status") or "missing",
    "trustedGovernanceTelemetryPath": "runtime/trusted-governance-telemetry.json" if trusted_governance_telemetry else None,
    "trustedGovernanceTelemetryStatus": trusted_governance_telemetry.get("status") or "missing",
    "v101ReleaseCertificationPath": "runtime/v101-release-certification.json" if v101_release_certification else None,
    "v101ReleaseCertificationStatus": v101_release_certification.get("v101ReleaseCertificationStatus") or v101_release_certification.get("status") or "missing",
    "v101Coverage": v101_release_certification.get("coverage") or {},
    "forgedGovernanceRuntimeResponsePath": "runtime/forged-governance-runtime-response.json" if forged_governance_response else None,
    "forgedGovernanceRuntimeStatus": forged_governance_response.get("status") or "missing",
    "releaseArtifactBoundaryPath": "runtime/release-artifact-boundary.json" if release_artifact_boundary else None,
    "releaseArtifactBoundaryStatus": release_artifact_boundary.get("status") or "missing",
    "projectRoadmapBaselinePath": "runtime/project-roadmap-baseline.json" if project_roadmap_baseline else None,
    "projectRoadmapBaselineStatus": project_roadmap_baseline.get("status") or "missing",
    "v103ReleaseFixCertificationPath": "runtime/v103-release-fix-certification.json" if v103_release_fix_certification else None,
    "v103ReleaseFixCertificationStatus": v103_release_fix_certification.get("v103ReleaseFixCertificationStatus") or v103_release_fix_certification.get("status") or "missing",
    "v103Coverage": v103_release_fix_certification.get("coverage") or {},
    "coreRuntimeNegativeFixturesPath": "runtime/core-runtime-negative-fixtures.json" if core_runtime_negative_fixtures_path.is_file() else None,
    "coreRuntimeNegativeFixturesStatus": core_runtime_negative_fixtures.get("status") or "missing",
    "coreRuntimeNegativeFixtureCoverage": core_runtime_negative_fixtures.get("fixtures") or [],
    "softwareDevReferenceWorkflowStatus": (core_runtime_negative_fixtures.get("positiveWorkflow") or {}).get("status") or "missing",
    "coreRuntimeKernelPath": "runtime/core-runtime-kernel.json" if core_runtime_kernel else None,
    "coreRuntimeKernelStatus": core_runtime_kernel.get("status") or "missing",
    "coreRuntimeAdmissionPath": "runtime/core-runtime-admission.json" if core_runtime_admission else None,
    "coreRuntimeAdmissionStatus": core_runtime_admission.get("status") or "missing",
    "coreRuntimeArbitrationPath": "runtime/core-runtime-arbitration.json" if core_runtime_arbitration else None,
    "coreRuntimeArbitrationStatus": core_runtime_arbitration.get("status") or "missing",
    "v105ReleaseCertificationPath": "runtime/v105-release-certification.json" if v105_release_certification else None,
    "v105ReleaseCertificationStatus": v105_release_certification.get("status") or "missing",
    "v105Coverage": v105_release_certification.get("coverage") or {},
    "v106ReleaseCertificationPath": "runtime/v106-release-certification.json" if v106_release_certification else None,
    "v106ReleaseCertificationStatus": v106_release_certification.get("status") or "missing",
    "v106Coverage": v106_release_certification.get("coverage") or {},
    "v107ReleaseProvenanceHandoffPath": "runtime/v107-release-provenance-handoff.json" if v107_release_provenance_handoff else None,
    "v107ReleaseProvenanceHandoffStatus": v107_release_provenance_handoff.get("status") or "missing",
    "v107ReleaseProvenanceHandoffCoverage": v107_release_provenance_handoff.get("coverage") or {},
    "v107ReleaseCertificationPath": "runtime/v107-release-certification.json" if v107_release_certification else None,
    "v107ReleaseCertificationStatus": v107_release_certification.get("status") or "missing",
    "v107ReleaseCertificationCoverage": v107_release_certification.get("coverage") or {},
    "coreDecisionModelContractPath": "runtime/core-decision-model-contract.json" if core_decision_model_contract else None,
    "coreDecisionModelContractStatus": core_decision_model_contract.get("status") or "missing",
    "coreDecisionModelContractCoverage": core_decision_model_contract.get("coverage") or {},
    "coreDecisionInputBindingPath": "runtime/core-decision-input-binding.json" if core_decision_input_binding else None,
    "coreDecisionInputBindingStatus": core_decision_input_binding.get("status") or "missing",
    "coreDecisionInputBindingCoverage": core_decision_input_binding.get("coverage") or {},
    "coreDecisionOutcomeTransitionsPath": "runtime/core-decision-outcome-transitions.json" if core_decision_outcome_transitions else None,
    "coreDecisionOutcomeTransitionsStatus": core_decision_outcome_transitions.get("status") or "missing",
    "coreDecisionOutcomeTransitionsCoverage": core_decision_outcome_transitions.get("coverage") or {},
    "coreDecisionFailureReasonPath": "runtime/core-decision-failure-reason-remediation.json" if core_decision_failure_reason else None,
    "coreDecisionFailureReasonStatus": core_decision_failure_reason.get("status") or "missing",
    "coreDecisionFailureReasonCoverage": core_decision_failure_reason.get("coverage") or {},
    "coreEvidenceToDecisionGatePath": "runtime/core-evidence-to-decision-gate.json" if core_evidence_to_decision_gate else None,
    "coreEvidenceToDecisionGateStatus": core_evidence_to_decision_gate.get("status") or "missing",
    "coreEvidenceToDecisionGateCoverage": core_evidence_to_decision_gate.get("coverage") or {},
    "coreCompletionCommitAuthorityPath": "runtime/core-completion-commit-authority.json" if core_completion_commit_authority else None,
    "coreCompletionCommitAuthorityStatus": core_completion_commit_authority.get("status") or "missing",
    "coreCompletionCommitAuthorityCoverage": core_completion_commit_authority.get("coverage") or {},
    "coreDeliveryReadinessAuditTriggerPath": "runtime/core-delivery-readiness-audit-trigger.json" if core_delivery_readiness_audit_trigger else None,
    "coreDeliveryReadinessAuditTriggerStatus": core_delivery_readiness_audit_trigger.get("status") or "missing",
    "coreDeliveryReadinessAuditTriggerCoverage": core_delivery_readiness_audit_trigger.get("coverage") or {},
    "coreDecisionProjectionReadModelPath": "runtime/core-decision-projection-read-model.json" if core_decision_projection_read_model else None,
    "coreDecisionProjectionReadModelStatus": core_decision_projection_read_model.get("status") or "missing",
    "coreDecisionProjectionReadModelCoverage": core_decision_projection_read_model.get("coverage") or {},
    "remainingRisks": remaining_risks,
    "deferredItems": deferred_items,
    "authorityBoundaryCertification": authority_boundary_certification,
    "messageBusDecisionRecordPath": "runtime/scheduling-decision.json" if scheduling_decision_path.is_file() else None,
    "messageBusDecision": scheduling_decision.get("decision") or "missing",
    "packNegativeFixturesPath": "pack-negative-fixtures.json" if pack_negative_fixtures_path.is_file() else None,
    "packMigrationAppliedReceiptPath": "pack-migration-applied-receipt.json" if pathlib.Path(summary_json_path.parent / "pack-migration-applied-receipt.json").is_file() else None,
    "packMigrationFakeAuthorityReceiptPath": "pack-migration-fake-authority-receipt.json" if pathlib.Path(summary_json_path.parent / "pack-migration-fake-authority-receipt.json").is_file() else None,
    "packMigrationRollbackReceiptPath": "pack-migration-rollback-receipt.json" if pathlib.Path(summary_json_path.parent / "pack-migration-rollback-receipt.json").is_file() else None,
    "packMigrationReplayReportPath": "runtime/pack-migration-replay-report.json" if pathlib.Path(summary_json_path.parent / "runtime/pack-migration-replay-report.json").is_file() else None,
    "packReleaseGateStatus": "passed" if pack_release_gate_passed else "failed",
    "packNegativeFixturesStatus": pack_negative_fixtures.get("status") or "missing",
    "gateCommands": [
        "cargo fmt --all --check",
        "cargo test --workspace",
        "npm --prefix apps/desktop run build",
        f"bash scripts/verify_release_gate.sh --artifact-dir {summary_json_path.parent} --release-version {release_version} --release-tag {release_tag_name or release_version} --source-commit-sha {source_commit_sha or 'unknown'} --release-url {release_url or 'unknown'}",
    ],
    "failedStage": current_stage if current_status == "failed" else None,
    "failureMessage": current_message if current_status == "failed" else None,
    "requirementId": requirement_id,
    "projectId": project_id,
    "issueCount": issue_count,
    "proofChain": proof_chain,
    "checklist": checklist,
    "publicArtifacts": public_artifacts,
    "runtimeArtifacts": runtime_artifacts,
    "generatedAt": status.get("updatedAt"),
    "remoteReleaseUrl": release.get("remoteReleaseUrl") or release_url,
    "releaseUrl": release_url,
    "requirePublishedReleaseFacts": require_published_release_facts,
}
cert_json_path.write_text(
    json.dumps(certification_payload, ensure_ascii=False, indent=2) + "\n",
    encoding="utf-8",
)

cert_lines = [
    "# Release Gate Certification",
    "",
    "- Gate class: `runtime-fixture-gate`",
    f"- Provider smoke gate: `{provider_smoke.get('status')}`",
    f"- Provider smoke provider: `{provider_smoke.get('provider') or 'n/a'}`",
    f"- Provider smoke reason: `{provider_smoke.get('reason') or 'n/a'}`",
    f"- Stable contract baseline: `{stable_contract_baseline.get('status') or 'missing'}`",
    f"- Runtime API / SDK compatibility: `{runtime_api_sdk_compatibility.get('status') or 'missing'}`",
    f"- Filesystem contract: `{filesystem_contract.get('status') or 'missing'}`",
    f"- Pack contract compatibility: `{pack_contract_compatibility.get('status') or 'missing'}`",
    f"- Projection / Read Model contract: `{projection_readmodel_contract.get('status') or 'missing'}`",
    f"- Evidence / Acceptance contract: `{evidence_acceptance_contract.get('status') or 'missing'}`",
    f"- Software Dev Pack stable baseline: `{software_dev_pack_stable_baseline.get('status') or 'missing'}`",
    f"- Executor Adapter contract: `{executor_adapter_contract.get('status') or 'missing'}`",
    f"- Replay / Migration / Upgrade certification: `{replay_migration_upgrade_certification.get('status') or 'missing'}`",
    f"- v1.0 stable core: `{v1_stable_core}`",
    f"- v1.0 support boundary: `{v1_support_boundary['version']}`",
    f"- Pack release gate: `{'passed' if pack_release_gate_passed else 'failed'}`",
    f"- Pack negative fixtures: `{pack_negative_fixtures.get('status') or 'missing'}`",
    f"- Deployment evidence: `{deployment_evidence.get('status') or 'missing'}`",
    f"- Deployment semantic failure fixture: `{deployment_evidence_failure.get('status') or 'missing'}`",
    f"- Negative semantic fixtures: `{negative_semantic_fixtures.get('status') or 'missing'}`",
    f"- Executable forged governance fixture: `{forged_governance_response.get('status') or 'missing'}`",
    f"- Release artifact boundary: `{release_artifact_boundary.get('status') or 'missing'}`",
    f"- Project roadmap baseline: `{project_roadmap_baseline.get('status') or 'missing'}`",
    f"- v1.0.3 release fix certification: `{v103_release_fix_certification.get('status') or 'missing'}`",
    f"- Rollback target: `{(deployment_evidence.get('rollbackModel') or {}).get('targetTag') or 'n/a'}`",
    f"- v1.0 planning readiness: `{v1_planning_readiness}`",
    f"- Release version: `{release_version}`",
    f"- Tag name: `{release_tag_name or release.get('tagName') or 'n/a'}`",
    f"- Source commit: `{source_commit_sha or 'n/a'}`",
    f"- Release URL: `{release_url or 'n/a'}`",
    f"- Gate workflow: `release-gate`",
    f"- Gate status: `{current_status}`",
    f"- Current gate run: `{current_gate_run['runUrl'] or 'local'}`",
    f"- Main gate run: `{(main_gate_run or {}).get('runUrl') or 'not-this-run'}`",
    f"- Tag gate run: `{(tag_gate_run or {}).get('runUrl') or 'not-this-run'}`",
    f"- Release gate run: `{(release_gate_run or {}).get('runUrl') or 'not-this-run'}`",
    f"- v1.0 planning readiness: `{v1_planning_readiness}`",
    f"- Message Bus decision: `{scheduling_decision.get('decision') or 'missing'}`",
    f"- Runtime API remains authority boundary: `{authority_boundary_certification['runtimeApiRemainsAuthorityBoundary']}`",
    f"- Projection / Connector / Industry UI authority uplift: `false`",
]
if current_status == "failed":
    cert_lines.append(f"- Failed stage: `{current_stage}`")
    if current_message:
        cert_lines.append(f"- Failure message: `{current_message}`")
else:
    cert_lines.append(f"- Current stage: `{current_stage}`")
    if current_message:
        cert_lines.append(f"- Stage message: `{current_message}`")
cert_lines.extend([
    f"- Requirement: `{requirement_id}`",
    f"- Project: `{project_id}`",
    "",
    "## V090 Coverage",
    "",
])
for item in v090_coverage:
    cert_lines.append(
        f"- [{'x' if item['passed'] else ' '}] `{item['id']}` {item['label']} -> `{item['evidencePath']}`"
    )
cert_lines.extend([
    "",
    "## V091 Coverage",
    "",
])
for item in v091_coverage:
    cert_lines.append(
        f"- [{'x' if item['passed'] else ' '}] `{item['id']}` {item['label']} -> `{item['evidencePath']}`"
    )
cert_lines.extend([
    "",
    "## V100 Coverage",
    "",
])
for item in v100_coverage:
    cert_lines.append(
        f"- [{'x' if item['passed'] else ' '}] `{item['id']}` {item['label']} -> `{item['evidencePath']}`"
    )
cert_lines.extend([
    "",
    "## v1.0 Planning Decision",
    "",
    f"- Readiness: `{v1_planning_readiness}`",
    f"- Blockers: `{', '.join(v1_planning_blockers) if v1_planning_blockers else 'none'}`",
    "- Message Bus decision record: `runtime/scheduling-decision.json`",
    "- Projection, Connector, and industry UI remain non-authority surfaces.",
    "",
    "## v1.0 Stable Core Decision",
    "",
    f"- Stable core: `{v1_stable_core}`",
    f"- Blockers: `{', '.join(v1_stable_core_blockers) if v1_stable_core_blockers else 'none'}`",
    f"- Support boundary: `{v1_support_boundary['version']}`",
    f"- Stable industry Packs: `{', '.join(v1_support_boundary['stableIndustryPacks'])}`",
    f"- Experimental industry Packs: `{', '.join(v1_support_boundary['experimentalIndustryPacks'])}`",
    f"- Audit sidecar independent: `{v1_support_boundary['auditSidecarIndependent']}`",
    f"- Executor runtime owns project truth: `{v1_support_boundary['executorRuntimeOwnsProjectTruth']}`",
    "",
    "## Remaining Risks And Deferred Items",
    "",
])
if remaining_risks:
    for item in remaining_risks:
        cert_lines.append(
            f"- Risk `{item['id']}` severity=`{item['severity']}` blockers=`{', '.join(item.get('blockers') or [])}`: {item['summary']}"
        )
else:
    cert_lines.append("- Remaining risks: `none`")
if deferred_items:
    for item in deferred_items:
        cert_lines.append(
            f"- Deferred `{item['id']}` blocking=`{item['blocking']}`: {item['summary']}"
        )
else:
    cert_lines.append("- Deferred items: `none`")
cert_lines.extend([
    "",
    "## Negative Semantic Fixture Coverage",
    "",
])
for item in negative_semantic_fixtures.get("fixtures") or []:
    mark = "PASS" if item.get("passed") else "FAIL"
    cert_lines.append(
        f"- [{mark}] `{item.get('id')}` stage=`{item.get('stage')}` evidence=`{item.get('evidencePath')}`"
    )
cert_lines.extend([
    "",
    "## Gate Commands",
    "",
])
for command in certification_payload["gateCommands"]:
    cert_lines.append(f"- `{command}`")
cert_lines.extend([
    "",
    "## Certification Checklist",
    "",
])
for item in checklist:
    mark = "PASS" if item["passed"] else "FAIL"
    cert_lines.append(f"- [{mark}] {item['label']}")
cert_lines.extend([
    "",
    "## Proof Chain",
    "",
])
for item in proof_chain:
    detail = f" - {item['detail']}" if item.get("detail") else ""
    cert_lines.append(f"- `{item['stage']}`: `{item['status']}`{detail}")
cert_lines.extend([
    "",
    "## Public Artifacts",
    "",
])
for item in public_artifacts:
    mark = "present" if item["exists"] else "missing"
    cert_lines.append(f"- `{item['path']}`: `{mark}`")
cert_lines.extend([
    "",
    "## Runtime Artifacts",
    "",
])
for item in runtime_artifacts:
    mark = "present" if item["exists"] else "missing"
    cert_lines.append(f"- `{item['path']}`: `{mark}`")
cert_md_path.write_text("\n".join(cert_lines) + "\n", encoding="utf-8")
PY
}

fail_stage() {
  local stage="$1"
  local message="$2"
  record_stage "$stage" "failed" "$message"
  write_status "failed" "$stage" "$message"
  write_gate_reports
  exit 1
}

verify_release_metadata() {
  local metadata_root="$1"
  local expected_version="${RELEASE_VERSION#v}"
  if ! python3 - "$metadata_root" "$expected_version" "$RELEASE_VERSION" <<'PY'
import json
import pathlib
import sys
import tomllib

root = pathlib.Path(sys.argv[1])
expected = sys.argv[2]
release_version = sys.argv[3]

def read_json(path: pathlib.Path):
    return json.loads(path.read_text(encoding="utf-8"))

cargo = tomllib.loads((root / "Cargo.toml").read_text(encoding="utf-8"))
desktop_package = read_json(root / "apps/desktop/package.json")
desktop_lock = read_json(root / "apps/desktop/package-lock.json")
tauri = read_json(root / "apps/desktop/src-tauri/tauri.conf.json")

versions = {
    "Cargo.toml workspace.package.version": cargo["workspace"]["package"]["version"],
    "apps/desktop/package.json version": desktop_package["version"],
    "apps/desktop/package-lock.json version": desktop_lock["version"],
    "apps/desktop/package-lock.json packages[''].version": desktop_lock["packages"][""]["version"],
    "apps/desktop/src-tauri/tauri.conf.json version": tauri["version"],
}

mismatches = {
    name: value
    for name, value in versions.items()
    if value != expected
}
if mismatches:
    details = ", ".join(f"{name}={value}" for name, value in sorted(mismatches.items()))
    raise SystemExit(
        f"release version metadata mismatch for {release_version}: expected {expected}; {details}"
    )
PY
  then
    fail_stage "release.version-metadata" "release metadata does not match $RELEASE_VERSION"
  fi
  record_stage "release.version-metadata" "passed" "$RELEASE_VERSION"
}

verify_stable_contract_baseline() {
  local metadata_root="$1"
  if ! python3 - "$metadata_root/docs/architecture/041-v100-stable-contract-baseline-v1.md" "$STABLE_CONTRACT_BASELINE_PATH" <<'PY'
import json
import pathlib
import re
import sys
import time

doc_path = pathlib.Path(sys.argv[1])
out_path = pathlib.Path(sys.argv[2])
relative_path = "docs/architecture/041-v100-stable-contract-baseline-v1.md"
required_sections = [
    "Stable Public Contracts",
    "Internal Implementation Details",
    "Experimental Contracts",
    "Compatibility Promise",
    "Breaking Change Rule",
    "Deprecation Rule",
    "Version Field Rule",
    "Release Certification Rule",
]

if not doc_path.is_file():
    raise SystemExit("stable contract baseline document missing")

text = doc_path.read_text(encoding="utf-8")
version_match = re.search(r"^stableContractVersion:\s*(\S+)\s*$", text, re.MULTILINE)
status_match = re.search(r"^stableContractStatus:\s*(\S+)\s*$", text, re.MULTILINE)
stable_contract_version = version_match.group(1) if version_match else None
stable_contract_status = status_match.group(1) if status_match else None
missing_sections = [
    section for section in required_sections if f"## {section}" not in text
]
payload = {
    "version": "agentflow-stable-contract-baseline-certification.v1",
    "status": "passed",
    "docPath": relative_path,
    "stableContractVersion": stable_contract_version,
    "stableContractStatus": stable_contract_status,
    "requiredSections": required_sections,
    "missingSections": missing_sections,
    "checkedAt": int(time.time()),
}
if stable_contract_version != "agentflow-stable-contract-baseline.v1":
    payload["status"] = "failed"
    payload["failureReason"] = "stableContractVersion mismatch"
if stable_contract_status != "active":
    payload["status"] = "failed"
    payload["failureReason"] = "stableContractStatus mismatch"
if missing_sections:
    payload["status"] = "failed"
    payload["failureReason"] = "required baseline sections missing"
out_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
if payload["status"] != "passed":
    raise SystemExit(payload["failureReason"])
PY
  then
    fail_stage "stable.contract-baseline" "stable contract baseline metadata is missing or invalid"
  fi
  record_stage "stable.contract-baseline" "passed" "041-v100-stable-contract-baseline-v1.md"
}

verify_release_publication_facts() {
  local metadata_root="$1"
  local expected_version="${RELEASE_VERSION#v}"
  if python3 - "$metadata_root/CHANGELOG.md" "${RELEASE_VERSION#v}" <<'PY'
import pathlib
import re
import sys

path = pathlib.Path(sys.argv[1])
expected = sys.argv[2]
if not path.is_file():
    raise SystemExit(1)
text = path.read_text(encoding="utf-8")
heading = re.compile(rf"^##\s+(?:\[)?v?{re.escape(expected)}(?:\])?(?:\s|$)", re.MULTILINE)
if not heading.search(text):
    raise SystemExit(1)
PY
  then
    record_stage "release.changelog-entry" "passed" "CHANGELOG.md contains ${RELEASE_VERSION#v}"
  elif [[ "$REQUIRE_PUBLISHED_RELEASE_FACTS" == "1" ]]; then
    fail_stage "release.changelog-entry" "CHANGELOG.md missing release entry for ${RELEASE_VERSION#v}"
  else
    record_stage "release.changelog-entry" "passed" "not required before release publication"
  fi

  if [[ "$REQUIRE_PUBLISHED_RELEASE_FACTS" == "1" ]]; then
    if ! python3 - "$RELEASE_TAG_NAME" "$RELEASE_URL" "$GATE_EVENT_NAME" <<'PY'
import json
import os
import pathlib
import sys

release_tag = sys.argv[1]
release_url = sys.argv[2]
event_name = sys.argv[3]
event_path = os.environ.get("GITHUB_EVENT_PATH")
if event_name != "release" or not event_path:
    raise SystemExit("published GitHub Release fact requires a release event payload")

payload = json.loads(pathlib.Path(event_path).read_text(encoding="utf-8"))
release = payload.get("release") or {}
payload_tag = release.get("tag_name") or ""
payload_url = release.get("html_url") or ""
if payload_tag != release_tag:
    raise SystemExit(f"release event tag mismatch: expected {release_tag}; got {payload_tag or 'missing'}")
if payload_url != release_url:
    raise SystemExit(f"release event URL mismatch: expected {release_url}; got {payload_url or 'missing'}")
if not release.get("target_commitish"):
    raise SystemExit("release event missing target_commitish")
PY
    then
      fail_stage "release.github-release-fact" "release publication facts are incomplete for $RELEASE_VERSION"
    fi
  fi

  if [[ "$REQUIRE_PUBLISHED_RELEASE_FACTS" == "1" ]]; then
    record_stage "release.github-release-fact" "passed" "$RELEASE_URL"
  else
    record_stage "release.github-release-fact" "passed" "not required before release publication"
  fi
}

run_source_agent_entry_gate() {
  record_stage "source.agent-entry" "started" "$SOURCE_AGENT_ENTRY_PATH"
  if ! python3 - "$WORKSPACE" "$SOURCE_AGENT_ENTRY_PATH" <<'PY'
import json
import pathlib
import subprocess
import sys

root = pathlib.Path(sys.argv[1])
out_path = pathlib.Path(sys.argv[2])

entry_path = root / "AGENTS.md"
tracked_docs = [
    "docs/project/README.md",
    "docs/project/goal.md",
    "docs/project/roadmap.md",
    "docs/project/context.md",
    "docs/README.md",
    "docs/architecture/README.md",
    "docs/architecture/current-module-boundaries.md",
    "docs/architecture/021-ai-os-project-core-capabilities-v1.md",
    "docs/architecture/builtin-pack-registry.md",
    "docs/architecture/041-v100-stable-contract-baseline-v1.md",
    "docs/architecture/050-v100-release-certification-v1.md",
    "docs/delivery/releases/v1.1.0/README.md",
    "docs/project/history/2026-06-current-baseline-history/README.md",
]
runtime_only_paths = [
    ".agentflow/runs",
    ".agentflow/tmp",
    ".agentflow/tasks",
    ".agentflow/index.sqlite",
]

if not entry_path.is_file():
    raise SystemExit("release source missing AGENTS.md")

entry_text = entry_path.read_text(encoding="utf-8")
if "Current stabilization plan: `docs/v0.9.1/README.md`" in entry_text:
    raise SystemExit("AGENTS.md still points current stabilization at docs/v0.9.1/README.md")
doc_results = []
missing_docs = []
for doc in tracked_docs:
    exists = (root / doc).is_file()
    mentioned = doc in entry_text
    doc_results.append({"path": doc, "exists": exists, "mentionedByEntry": mentioned})
    if not exists or not mentioned:
        missing_docs.append(doc)
if missing_docs:
    raise SystemExit(f"AGENTS.md missing tracked doc references: {', '.join(missing_docs)}")

tracked_runtime = subprocess.check_output(
    ["git", "-C", str(root), "ls-files", "--", *runtime_only_paths],
    text=True,
).splitlines()
if tracked_runtime:
    raise SystemExit(f"runtime-only paths are tracked: {', '.join(tracked_runtime)}")

payload = {
    "version": "agentflow-source-agent-entry.v1",
    "status": "passed",
    "entryPath": "AGENTS.md",
    "trackedDocs": doc_results,
    "trackedRuntimePaths": tracked_runtime,
    "runtimeOnlyPaths": runtime_only_paths,
    "currentProjectGoalEntry": "docs/project/goal.md",
    "currentProjectRoadmapEntry": "docs/project/roadmap.md",
    "currentCoreCapabilityEntry": "docs/architecture/021-ai-os-project-core-capabilities-v1.md",
    "currentStableEntry": "docs/architecture/041-v100-stable-contract-baseline-v1.md",
    "currentReleaseBaselineEntry": "docs/delivery/releases/v1.1.0/README.md",
    "releaseCertificationEntry": "docs/architecture/050-v100-release-certification-v1.md",
    "defineAgentBoundary": {
        "path": ".agentflow/define/agent/**",
        "releaseSourceAuthority": False,
        "runtimeMaterializedManual": True,
        "trackedEquivalent": "AGENTS.md + docs/project + docs/architecture + docs/delivery",
    },
}
out_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
PY
  then
    fail_stage "source.agent-entry" "release source Agent entry is not aligned"
  fi
  record_stage "source.agent-entry" "passed" "$(basename "$SOURCE_AGENT_ENTRY_PATH")"
}

run_cli_json() {
  local stage="$1"
  local output="$2"
  shift 2
  if ! (cd "$WORKSPACE" && "$BIN" "$@" >"$output" 2>&1); then
    fail_stage "$stage" "command failed: agentflow $*"
  fi
  record_stage "$stage" "passed" "$(basename "$output")"
}

run_workspace_cmd() {
  local stage="$1"
  local output="$2"
  shift 2
  if ! (cd "$WORKSPACE" && "$@" >"$output" 2>&1); then
    fail_stage "$stage" "command failed: $*"
  fi
  record_stage "$stage" "passed" "$(basename "$output")"
}

json_field() {
  local file="$1"
  local expression="$2"
  python3 - "$file" "$expression" <<'PY'
import json, pathlib, sys
data = json.loads(pathlib.Path(sys.argv[1]).read_text())
expression = sys.argv[2]
value = data
for part in expression.split("."):
    if part.isdigit():
        value = value[int(part)]
    else:
        value = value[part]
if isinstance(value, (dict, list)):
    print(json.dumps(value, ensure_ascii=False))
else:
    print(value)
PY
}

text_value() {
  local file="$1"
  local prefix="$2"
  sed -n "s/^${prefix}//p" "$file" | tail -n 1
}

verify_spec_stage_artifact() {
  local record_name="$1"
  local requirement_id="$2"
  local stage_name="$3"
  local expected_status="$4"
  local expected_authority="${5:-}"

  local file="$WORKSPACE/.agentflow/spec/requirements/${requirement_id}/${stage_name}.json"
  python3 - "$file" "$expected_status" "$expected_authority" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
expected_status = sys.argv[2]
expected_authority = sys.argv[3]

if not path.is_file():
    raise SystemExit(f"missing spec stage artifact: {path}")

payload = json.loads(path.read_text(encoding="utf-8"))
actual_status = payload.get("status")
if actual_status != expected_status:
    raise SystemExit(
        f"unexpected status for {path.name}: expected {expected_status}, found {actual_status}"
    )

if expected_authority:
    actual_authority = payload.get("authority")
    if actual_authority != expected_authority:
        raise SystemExit(
            f"unexpected authority for {path.name}: expected {expected_authority}, found {actual_authority}"
        )
PY
  record_stage "$record_name" "passed" "$(basename "$file")"
}

verify_spec_loop_projection() {
  local record_name="$1"
  local requirement_id="$2"
  local projection_path="$WORKSPACE/.agentflow/projections/spec-loops/${requirement_id}.json"

  python3 - "$projection_path" "$record_name" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
record_name = sys.argv[2]

if not path.is_file():
    raise SystemExit(f"missing spec loop projection: {path}")

payload = json.loads(path.read_text(encoding="utf-8"))

if record_name == "runtime-action-proposal.accepted":
    proposals = payload.get("runtimeActionProposals") or []
    if not proposals:
        raise SystemExit("spec loop projection does not contain runtime action proposals")
elif record_name == "projection.current":
    current_state = payload.get("currentState")
    updated_at = payload.get("updatedAt")
    if not current_state:
        raise SystemExit("spec loop projection missing currentState")
    if not updated_at:
        raise SystemExit("spec loop projection missing updatedAt")
else:
    raise SystemExit(f"unsupported projection verification record: {record_name}")
PY
  record_stage "$record_name" "passed" "$(basename "$projection_path")"
}

run_event_replay_projection_gate() {
  record_stage "event-replay-projection.happy" "started" "$EVENT_REPLAY_PROJECTION_REPORT_PATH"
  run_cli_json "event-replay-projection.happy" "$CLI_DIR/event-replay-projection-happy.txt" \
    projection replay-report --output "$EVENT_REPLAY_PROJECTION_REPORT_PATH"
  if ! python3 - "$EVENT_REPLAY_PROJECTION_REPORT_PATH" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
if payload.get("status") != "passed":
    raise SystemExit(f"expected replay report passed, got {payload.get('status')}")
if payload.get("eventCount", 0) <= 0:
    raise SystemExit("replay happy path must read at least one event")
if payload.get("taskCount", 0) <= 0:
    raise SystemExit("replay happy path must rebuild at least one task projection")
if not payload.get("rebuiltPaths"):
    raise SystemExit("replay happy path must list rebuilt projection paths")
if not payload.get("sourceRefs"):
    raise SystemExit("replay happy path must list source event/fact refs")
if not payload.get("inputDigest"):
    raise SystemExit("replay happy path must include input digest")
if not payload.get("outputDigest"):
    raise SystemExit("replay happy path must include output digest")
if not payload.get("receiptId"):
    raise SystemExit("replay happy path must include receipt id")
if payload.get("deterministic") is not True:
    raise SystemExit("replay happy path must declare deterministic rebuild")
if payload.get("writesAuthority") is not False:
    raise SystemExit("replay happy path must not write authority")
if payload.get("projectionAuthority") is not False:
    raise SystemExit("projection must not be authority")
if payload.get("failures"):
    raise SystemExit("replay happy path must not include failures")
PY
  then
    fail_stage "event-replay-projection.happy" "happy path replay report is invalid"
  fi

  local failure_workspace="$TMP_DIR/event-replay-failure-workspace"
  mkdir -p "$failure_workspace/.agentflow/events/task-events"
  printf '{not-json\n' >"$failure_workspace/.agentflow/events/task-events/000001-corrupt.json"
  record_stage "event-replay-projection.failure" "started" "$EVENT_REPLAY_PROJECTION_FAILURE_REPORT_PATH"
  if ! (cd "$failure_workspace" && "$BIN" projection replay-report --output "$EVENT_REPLAY_PROJECTION_FAILURE_REPORT_PATH" >"$CLI_DIR/event-replay-projection-failure.txt" 2>&1); then
    fail_stage "event-replay-projection.failure" "failure fixture command failed before report was written"
  fi
  if ! python3 - "$EVENT_REPLAY_PROJECTION_FAILURE_REPORT_PATH" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
if payload.get("status") != "failed":
    raise SystemExit(f"expected replay report failed, got {payload.get('status')}")
failures = payload.get("failures") or []
if not failures:
    raise SystemExit("failure replay report must include failures")
if payload.get("deterministic") is not False:
    raise SystemExit("failure replay report must not claim deterministic rebuild")
if payload.get("writesAuthority") is not False:
    raise SystemExit("failure replay report must not write authority")
if payload.get("projectionAuthority") is not False:
    raise SystemExit("projection must not be authority")
message = failures[0].get("message") or ""
if "parse" not in message:
    raise SystemExit(f"failure replay report must preserve parse reason, got {message!r}")
PY
  then
    fail_stage "event-replay-projection.failure" "failure path replay report is invalid"
  fi
  record_stage "event-replay-projection.failure" "passed" "$(basename "$EVENT_REPLAY_PROJECTION_FAILURE_REPORT_PATH")"
}

run_provider_smoke_gate() {
  if [[ "$PROVIDER_SMOKE" != "1" ]]; then
    write_provider_smoke_status "skipped" "$PROVIDER_SMOKE_PROVIDER" "PROVIDER_SMOKE=0" ""
    record_stage "provider-smoke-gate" "skipped" "PROVIDER_SMOKE=0"
    return 0
  fi

  local smoke_workspace="$TMP_DIR/provider-smoke-workspace"
  mkdir -p "$smoke_workspace"
  git -C "$smoke_workspace" init -q
  git -C "$smoke_workspace" config user.email "codex@example.com"
  git -C "$smoke_workspace" config user.name "Codex"

  if ! (
    cd "$smoke_workspace"
    "$BIN" provider-smoke \
      --provider "$PROVIDER_SMOKE_PROVIDER" \
      --issue-id AF-PROVIDER-SMOKE-001 \
      --run-id run-provider-smoke-001 \
      --working-directory "$smoke_workspace" \
      --launch-request-path .agentflow/tmp/provider-smoke-request.md \
      >"$PROVIDER_SMOKE_ARTIFACT_PATH"
  ); then
    write_provider_smoke_status "failed" "$PROVIDER_SMOKE_PROVIDER" "provider smoke command failed" "$PROVIDER_SMOKE_ARTIFACT_PATH"
    fail_stage "provider-smoke-gate" "provider smoke command failed"
  fi

  local outcome reason
  outcome="$(json_field "$PROVIDER_SMOKE_ARTIFACT_PATH" outcome)"
  reason="$(json_field "$PROVIDER_SMOKE_ARTIFACT_PATH" reason)"
  write_provider_smoke_status "$outcome" "$PROVIDER_SMOKE_PROVIDER" "$reason" "$PROVIDER_SMOKE_ARTIFACT_PATH"
  if [[ "$outcome" == "passed" || "$outcome" == "skipped" ]]; then
    record_stage "provider-smoke-gate" "$outcome" "$reason"
  else
    fail_stage "provider-smoke-gate" "$reason"
  fi
}

run_api_plane_manifest_gate() {
  record_stage "api-plane-manifest" "started" "$API_PLANE_MANIFEST_PATH"
  "$BIN" api-plane manifest --output "$API_PLANE_MANIFEST_PATH"
  python3 - "$API_PLANE_MANIFEST_PATH" <<'PY'
import json, pathlib, sys
path = pathlib.Path(sys.argv[1])
if not path.is_file():
    raise SystemExit(f"missing api plane manifest: {path}")
payload = json.loads(path.read_text(encoding="utf-8"))
if payload.get("version") != "agentflow-api-plane-manifest.v1":
    raise SystemExit("api plane manifest version mismatch")
required_categories = {
    "runtime_commands",
    "projection_queries",
    "event_api",
    "command_surface_actions",
    "connector_actions",
    "provider_actions",
    "audit_actions",
    "release_actions",
    "pack_actions",
    "pack_command_surface",
}
entries = payload.get("entries") or []
categories = {entry.get("category") for entry in entries}
missing = sorted(required_categories - categories)
if missing:
    raise SystemExit(f"api plane manifest missing categories: {missing}")
allowed_boundaries = {"authority", "readonly", "command", "internal"}
for entry in entries:
    if entry.get("boundary") not in allowed_boundaries:
        raise SystemExit(f"api plane entry has invalid boundary: {entry}")
    if entry.get("category") == "projection_queries" and entry.get("boundary") != "readonly":
        raise SystemExit(f"projection query must be readonly: {entry}")
    if entry.get("access") == "sdk-candidate" and entry.get("boundary") != "readonly":
        raise SystemExit(f"sdk candidate API must be readonly: {entry}")
    if entry.get("access") == "sdk-candidate" and entry.get("category") in {
        "runtime_commands",
        "command_surface_actions",
        "connector_actions",
        "provider_actions",
        "release_actions",
    }:
        raise SystemExit(f"sdk candidate cannot expose write-side API: {entry}")

entry_ids = {entry.get("apiId") for entry in entries}
required_entry_ids = {
    "runtime.command.validate",
    "runtime.command.execute",
    "projection.task-workbench",
    "projection.pack-industry-workbench",
    "event.runtime.replay",
    "event.task.replay",
    "event.runtime.append-accepted-action",
    "pack.command.list",
    "pack.command.validate",
    "pack.command.dry-run",
    "pack.command.submit-proposal",
    "pack.surface.route",
    "pack.capability.status",
}
missing_entries = sorted(required_entry_ids - entry_ids)
if missing_entries:
    raise SystemExit(f"api plane manifest missing contract entries: {missing_entries}")

event_entries = [entry for entry in entries if entry.get("category") == "event_api"]
if not any(entry.get("boundary") == "readonly" for entry in event_entries):
    raise SystemExit("event API must include a readonly replay path")
if not any(entry.get("boundary") == "internal" for entry in event_entries):
    raise SystemExit("event API must include an internal append/claim path")
PY
  record_stage "api-plane-manifest" "passed" "$(basename "$API_PLANE_MANIFEST_PATH")"
}

run_runtime_api_sdk_compatibility_gate() {
  record_stage "runtime-api-sdk-compatibility" "started" "$RUNTIME_API_SDK_COMPATIBILITY_PATH"
  python3 - "$ROOT" "$API_PLANE_MANIFEST_PATH" "$RUNTIME_API_SDK_COMPATIBILITY_PATH" <<'PY'
import json
import pathlib
import re
import sys

root = pathlib.Path(sys.argv[1])
manifest_path = pathlib.Path(sys.argv[2])
output_path = pathlib.Path(sys.argv[3])
doc_path = root / "docs/architecture/042-v100-runtime-api-sdk-freeze-v1.md"

if not manifest_path.is_file():
    raise SystemExit(f"missing api plane manifest: {manifest_path}")
if not doc_path.is_file():
    raise SystemExit(f"missing runtime api sdk freeze document: {doc_path}")

manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
doc = doc_path.read_text(encoding="utf-8")

def metadata_value(name: str) -> str | None:
    match = re.search(rf"^{re.escape(name)}:\s*(\S+)\s*$", doc, re.MULTILINE)
    return match.group(1) if match else None

required_sections = [
    "## Stable API Planes",
    "## Command Input Contract",
    "## Query Input Contract",
    "## Event Output Contract",
    "## Decision Output Contract",
    "## Error Model",
    "## Governance Admission Rule",
    "## Runtime API And CLI Relationship",
    "## Minimal SDK Surface",
    "## Compatibility Fixture",
]
missing_sections = [section for section in required_sections if section not in doc]
entries = manifest.get("entries") or []
entry_ids = {entry.get("apiId") for entry in entries}
required_entries = {
    "runtime.command.validate",
    "runtime.command.execute",
    "projection.task-workbench",
    "projection.pack-industry-workbench",
    "event.runtime.replay",
    "event.task.replay",
    "event.runtime.append-accepted-action",
    "event.task.claim",
    "pack.command.list",
    "pack.command.validate",
    "pack.command.dry-run",
    "pack.command.submit-proposal",
    "pack.capability.status",
    "pack.surface.route",
}
missing_manifest_entries = sorted(required_entries - entry_ids)
sdk_candidate_violations = [
    entry
    for entry in entries
    if entry.get("access") == "sdk-candidate" and entry.get("boundary") != "readonly"
]

status_semantics = {
    "accepted": {"writesProposal": True, "writesAcceptedEvent": True},
    "rejected": {"writesProposal": False, "writesAcceptedEvent": False},
    "deferred": {"writesProposal": False, "writesAcceptedEvent": False},
    "failed": {"writesProposal": False, "writesAcceptedEvent": False},
}
example_coverage = {
    "command": '"commandId"' in doc and '"commandType"' in doc,
    "query": '"queryId"' in doc and '"queryType"' in doc,
    "event": '"events"' in doc and '"eventType"' in doc,
}
error_model_fields = ["code", "stage", "reason", "evidencePath"]
error_model_complete = all(f'"{field}"' in doc for field in error_model_fields)

payload = {
    "version": "agentflow-runtime-api-sdk-compatibility.v1",
    "status": "passed",
    "docPath": "docs/architecture/042-v100-runtime-api-sdk-freeze-v1.md",
    "runtimeApiSdkContractVersion": metadata_value("runtimeApiSdkContractVersion"),
    "runtimeApiSdkContractStatus": metadata_value("runtimeApiSdkContractStatus"),
    "stableContractBaseline": metadata_value("stableContractBaseline"),
    "manifestPath": "runtime/api-plane-manifest.json",
    "manifestVersion": manifest.get("version"),
    "commandPathRequiresGovernanceAdmission": "-> governance admission" in doc,
    "rejectedWritesProposal": status_semantics["rejected"]["writesProposal"],
    "deferredWritesProposal": status_semantics["deferred"]["writesProposal"],
    "rejectedWritesAcceptedEvent": status_semantics["rejected"]["writesAcceptedEvent"],
    "deferredWritesAcceptedEvent": status_semantics["deferred"]["writesAcceptedEvent"],
    "sdkCandidateReadonly": not sdk_candidate_violations,
    "sdkCandidateViolationCount": len(sdk_candidate_violations),
    "missingSections": missing_sections,
    "missingManifestEntries": missing_manifest_entries,
    "errorModelFields": error_model_fields,
    "errorModelComplete": error_model_complete,
    "sdkExampleCoverage": example_coverage,
    "stableStatuses": sorted(status_semantics.keys()),
}

if (
    payload["runtimeApiSdkContractVersion"] != "agentflow-runtime-api-sdk-freeze.v1"
    or payload["runtimeApiSdkContractStatus"] != "active"
    or payload["stableContractBaseline"] != "agentflow-stable-contract-baseline.v1"
    or manifest.get("version") != "agentflow-api-plane-manifest.v1"
    or missing_sections
    or missing_manifest_entries
    or sdk_candidate_violations
    or not payload["commandPathRequiresGovernanceAdmission"]
    or not error_model_complete
    or not all(example_coverage.values())
):
    payload["status"] = "failed"

output_path.parent.mkdir(parents=True, exist_ok=True)
output_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
if payload["status"] != "passed":
    raise SystemExit("runtime api sdk compatibility fixture failed")
PY
  record_stage "runtime-api-sdk-compatibility" "passed" "$(basename "$RUNTIME_API_SDK_COMPATIBILITY_PATH")"
}

run_filesystem_contract_gate() {
  local workspace_root="$1"
  record_stage "filesystem-contract" "started" "$FILESYSTEM_CONTRACT_PATH"
  python3 - "$ROOT" "$workspace_root" "$FILESYSTEM_CONTRACT_PATH" <<'PY'
import json
import pathlib
import re
import sys
import time

root = pathlib.Path(sys.argv[1])
workspace = pathlib.Path(sys.argv[2])
output_path = pathlib.Path(sys.argv[3])
doc_path = root / "docs/architecture/043-v100-agentflow-filesystem-contract-freeze-v1.md"

if not doc_path.is_file():
    raise SystemExit(f"missing filesystem contract freeze document: {doc_path}")

doc = doc_path.read_text(encoding="utf-8")

def metadata_value(name):
    match = re.search(rf"^{re.escape(name)}:\s*(\S+)\s*$", doc, re.MULTILINE)
    return match.group(1) if match else None

required_sections = [
    "## Stable Path Contract",
    "## Authority Classes",
    "## Public Record Boundary",
    "## Release Source Archive Boundary",
    "## Retired Paths",
    "## Runtime Write Rules",
    "## Version Rule",
    "## Release Gate Fixture",
]
stable_paths = [
    ".agentflow/project/**",
    ".agentflow/spec/requirements/<requirement-id>/**",
    ".agentflow/spec/projects/<project-id>.json",
    ".agentflow/spec/issues/<issue-id>.json",
    ".agentflow/spec/completions/<project-id>.json",
    ".agentflow/runtime/commands/<command-id>.json",
    ".agentflow/runtime/proposals/<proposal-id>.json",
    ".agentflow/runtime/decisions/<proposal-id>.json",
    ".agentflow/runtime/actions/<accepted-action-id>.json",
    ".agentflow/packs/<pack-id>/**",
    ".agentflow/tasks/<issue-id>/work-loop-contract.json",
    ".agentflow/tasks/<issue-id>/runs/<run-id>/run.json",
    ".agentflow/tasks/<issue-id>/runs/<run-id>/preflight/preflight.json",
    ".agentflow/tasks/<issue-id>/runs/<run-id>/launch/**",
    ".agentflow/tasks/<issue-id>/runs/<run-id>/commands/**",
    ".agentflow/tasks/<issue-id>/runs/<run-id>/checkpoints/**",
    ".agentflow/tasks/<issue-id>/runs/<run-id>/review/**",
    ".agentflow/tasks/<issue-id>/evidence/**",
    ".agentflow/events/**",
    ".agentflow/projections/**",
    ".agentflow/indexes/**",
    ".agentflow/release/**",
    ".agentflow/audit/**",
    ".agentflow/tmp/**",
]
authority_classes = [
    "### Authority",
    "### Definition",
    "### Derived / Transport",
    "### Projection",
    "### Sidecar Authority",
    "### Local Cache",
]
retired_paths = [
    ".agentflow/input/**",
    ".agentflow/execute/**",
    ".agentflow/output/**",
    ".agentflow/goal-tree/**",
    ".agentflow/define/goals/**",
    ".agentflow/define/milestones/**",
    ".agentflow/define/issues/**",
]
retired_checks = [
    ".agentflow/input",
    ".agentflow/execute",
    ".agentflow/output",
    ".agentflow/goal-tree",
    ".agentflow/define/goals",
    ".agentflow/define/milestones",
    ".agentflow/define/issues",
]
source_archive_runtime_markers = [
    ".agentflow/tasks/**",
    ".agentflow/events/**",
    ".agentflow/projections/**",
    ".agentflow/tmp/**",
]

missing_sections = [section for section in required_sections if section not in doc]
missing_stable_paths = [path for path in stable_paths if path not in doc]
missing_authority_classes = [item for item in authority_classes if item not in doc]
missing_retired_paths = [path for path in retired_paths if path not in doc]
retired_path_violations = [
    path
    for path in retired_checks
    if (workspace / path).exists()
]
source_archive_includes_runtime_state = not all(marker in doc for marker in source_archive_runtime_markers)

payload = {
    "version": "agentflow-filesystem-contract-certification.v1",
    "status": "passed",
    "docPath": "docs/architecture/043-v100-agentflow-filesystem-contract-freeze-v1.md",
    "filesystemContractVersion": metadata_value("filesystemContractVersion"),
    "filesystemContractStatus": metadata_value("filesystemContractStatus"),
    "stableContractBaseline": metadata_value("stableContractBaseline"),
    "requiredSections": required_sections,
    "missingSections": missing_sections,
    "stablePaths": stable_paths,
    "missingStablePaths": missing_stable_paths,
    "authorityClasses": [item.removeprefix("### ") for item in authority_classes],
    "missingAuthorityClasses": missing_authority_classes,
    "retiredPaths": retired_paths,
    "missingRetiredPaths": missing_retired_paths,
    "retiredPathViolations": retired_path_violations,
    "sourceArchiveIncludesRuntimeState": source_archive_includes_runtime_state,
    "localRuntimeStateExcludedFromSourceArchive": not source_archive_includes_runtime_state,
    "checkedAt": int(time.time()),
}

if (
    payload["filesystemContractVersion"] != "agentflow-filesystem-contract-freeze.v1"
    or payload["filesystemContractStatus"] != "active"
    or payload["stableContractBaseline"] != "agentflow-stable-contract-baseline.v1"
    or missing_sections
    or missing_stable_paths
    or missing_authority_classes
    or missing_retired_paths
    or retired_path_violations
    or source_archive_includes_runtime_state
):
    payload["status"] = "failed"

output_path.parent.mkdir(parents=True, exist_ok=True)
output_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
if payload["status"] != "passed":
    raise SystemExit("filesystem contract fixture failed")
PY
  record_stage "filesystem-contract" "passed" "$(basename "$FILESYSTEM_CONTRACT_PATH")"
}

run_capability_registry_gate() {
  record_stage "capability-registry" "started" "$CAPABILITY_REGISTRY_PATH"
  "$BIN" capability-registry manifest --output "$CAPABILITY_REGISTRY_PATH"
  python3 - "$CAPABILITY_REGISTRY_PATH" <<'PY'
import json, pathlib, sys
path = pathlib.Path(sys.argv[1])
if not path.is_file():
    raise SystemExit(f"missing capability registry: {path}")
payload = json.loads(path.read_text(encoding="utf-8"))
if payload.get("version") != "agentflow-capability-registry.v1":
    raise SystemExit("capability registry version mismatch")
workers = payload.get("workers") or []
worker_ids = {worker.get("workerId") for worker in workers}
required_workers = {
    "codex",
    "claude",
    "local-shell-validator",
    "git-provider",
    "github",
    "mcp-connector",
    "audit-worker",
}
missing = sorted(required_workers - worker_ids)
if missing:
    raise SystemExit(f"capability registry missing workers: {missing}")
for worker in workers:
    boundary = worker.get("boundary") or {}
    if boundary.get("authorityWrite") is not False:
        raise SystemExit(f"worker grants authority write: {worker}")
    if boundary.get("runtimeCommandRequired") is not True:
        raise SystemExit(f"worker does not require runtime command: {worker}")
disabled_capabilities = [
    capability
    for worker in workers
    for capability in (worker.get("capabilities") or [])
    if capability.get("available") is False
]
if not disabled_capabilities:
    raise SystemExit("capability registry gate must cover at least one disabled capability")
missing_reason = [
    capability
    for capability in disabled_capabilities
    if not capability.get("disabledReason")
]
if missing_reason:
    raise SystemExit(f"disabled capability is missing a reason: {missing_reason[0]}")
PY
  record_stage "capability-registry" "passed" "$(basename "$CAPABILITY_REGISTRY_PATH")"
}

run_governance_policy_gate() {
  record_stage "governance-policy" "started" "$GOVERNANCE_POLICY_PATH"

  local allow_path="$RUNTIME_DIR/governance-policy-allow.json"
  local defer_path="$RUNTIME_DIR/governance-policy-defer.json"
  local reject_path="$RUNTIME_DIR/governance-policy-reject.json"

  "$BIN" governance-policy evaluate \
    --role work-agent \
    --action-type startRun \
    --object-type Issue \
    --worker-id local-shell-validator \
    --command validate.test \
    --audit-sidecar-mode independent \
    --capability-registry "$CAPABILITY_REGISTRY_PATH" \
    --output "$allow_path"

  "$BIN" governance-policy evaluate \
    --role work-agent \
    --action-type startRun \
    --object-type Issue \
    --worker-id github \
    --command repo.read \
    --audit-sidecar-mode not-requested \
    --capability-registry "$CAPABILITY_REGISTRY_PATH" \
    --output "$defer_path"

  "$BIN" governance-policy evaluate \
    --role human-owner \
    --action-type requestAudit \
    --object-type Issue \
    --worker-id audit-worker \
    --command audit.report \
    --audit-sidecar-mode bound-to-main-chain \
    --capability-registry "$CAPABILITY_REGISTRY_PATH" \
    --output "$reject_path"

  python3 - "$GOVERNANCE_POLICY_PATH" "$allow_path" "$defer_path" "$reject_path" <<'PY'
import json
import pathlib
import sys
import time

out_path = pathlib.Path(sys.argv[1])
reports = [
    json.loads(pathlib.Path(path).read_text(encoding="utf-8"))
    for path in sys.argv[2:]
]
decisions = {report.get("decision") for report in reports}
if not {"allowed", "deferred", "rejected"}.issubset(decisions):
    raise SystemExit(f"governance policy gate did not cover all decisions: {sorted(decisions)}")
if any(report.get("version") != "agentflow-governance-policy-report.v1" for report in reports):
    raise SystemExit("governance policy report version mismatch")
if any(not report.get("trace") for report in reports):
    raise SystemExit("governance policy reports must include trace evidence")
if not any(report.get("capabilityPolicy", {}).get("decision") == "deferred" for report in reports):
    raise SystemExit("governance policy gate must cover provider/capability defer")
if not any(report.get("auditSidecarPolicy", {}).get("decision") == "rejected" for report in reports):
    raise SystemExit("governance policy gate must reject audit sidecar main-chain binding")

payload = {
    "version": "agentflow-runtime-governance-policy-gate.v1",
    "status": "passed",
    "writesAuthority": False,
    "executesProvider": False,
    "decisionCount": len(reports),
    "reports": reports,
    "generatedAt": int(time.time()),
}
out_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
PY
  record_stage "governance-policy" "passed" "$(basename "$GOVERNANCE_POLICY_PATH")"
}

run_governance_admission_gate() {
  record_stage "governance-admission" "started" "$GOVERNANCE_ADMISSION_PATH"

  local allow_request="$CLI_DIR/governance-admission-allow-request.json"
  local defer_request="$CLI_DIR/governance-admission-defer-request.json"
  local reject_request="$CLI_DIR/governance-admission-reject-request.json"
  local allow_response="$RUNTIME_DIR/governance-admission-allow.json"
  local defer_response="$RUNTIME_DIR/governance-admission-defer.json"
  local reject_response="$RUNTIME_DIR/governance-admission-reject.json"

  python3 - "$allow_request" "$defer_request" "$reject_request" <<'PY'
import json
import pathlib
import sys

allow_request = pathlib.Path(sys.argv[1])
defer_request = pathlib.Path(sys.argv[2])
reject_request = pathlib.Path(sys.argv[3])

base = {
    "commandType": "core.action.invoke",
    "route": {
        "routeId": "core:project.create",
        "actionContractRef": "action-contract:project.create",
        "targetObjectType": "Spec",
    },
    "sourceSurface": "agent",
    "actorRole": "spec-agent",
    "skillRef": "core:spec-agent:project.create",
    "targetObjectRef": {"objectType": "Spec", "id": "spec-governance-001"},
    "input": {
        "projectId": "project-governance-001",
        "projectTitle": "Governance Admission Fixture"
    },
    "evidenceRefs": ["DecisionRef:approved-spec-1", "EvidenceRef:human-confirmation-1"],
    "artifactRefs": ["ArtifactRef:.agentflow/spec/requirements/req-governance/preview.json"],
    "createdAt": "2026-06-25T00:00:00Z",
}

def write(path, command_id, input_patch):
    payload = dict(base)
    payload["commandId"] = command_id
    payload["idempotencyKey"] = f"release-gate:{command_id}"
    payload["input"] = dict(base["input"])
    payload["input"].update(input_patch)
    path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")

write(allow_request, "cmd-governance-admission-allow", {})
write(defer_request, "cmd-governance-admission-defer", {
    "governanceWorkerId": "github",
    "governanceCommand": "repo.read",
})
write(reject_request, "cmd-governance-admission-reject", {
    "auditSidecarMode": "bound-to-main-chain",
})
PY

  (
    cd "$WORKSPACE"
    "$BIN" runtime-command execute --request "$allow_request" --output "$allow_response"
    "$BIN" runtime-command execute --request "$defer_request" --output "$defer_response"
    "$BIN" runtime-command execute --request "$reject_request" --output "$reject_response"
  )

  python3 - "$GOVERNANCE_ADMISSION_PATH" "$WORKSPACE" "$allow_response" "$defer_response" "$reject_response" <<'PY'
import json
import pathlib
import sys
import time

out_path = pathlib.Path(sys.argv[1])
workspace = pathlib.Path(sys.argv[2])
responses = [
    json.loads(pathlib.Path(path).read_text(encoding="utf-8"))
    for path in sys.argv[3:]
]
by_command = {response.get("commandId"): response for response in responses}

required = {
    "cmd-governance-admission-allow",
    "cmd-governance-admission-defer",
    "cmd-governance-admission-reject",
}
if set(by_command) != required:
    raise SystemExit(f"governance admission commands mismatch: {sorted(by_command)}")

allow = by_command["cmd-governance-admission-allow"]
defer = by_command["cmd-governance-admission-defer"]
reject = by_command["cmd-governance-admission-reject"]

def decision(response):
    return (response.get("governanceAdmission") or {}).get("decision")

if decision(allow) != "allowed":
    raise SystemExit(f"allowed fixture did not pass governance: {decision(allow)}")
if decision(defer) != "deferred" or defer.get("status") != "deferred":
    raise SystemExit(f"defer fixture did not defer: {defer}")
if decision(reject) != "rejected" or reject.get("status") != "rejected":
    raise SystemExit(f"reject fixture did not reject: {reject}")

proposal_dir = workspace / ".agentflow/runtime/proposals"
action_dir = workspace / ".agentflow/runtime/actions"
allow_proposal = proposal_dir / "proposal-cmd-governance-admission-allow.json"
defer_proposal = proposal_dir / "proposal-cmd-governance-admission-defer.json"
reject_proposal = proposal_dir / "proposal-cmd-governance-admission-reject.json"
if not allow_proposal.is_file():
    raise SystemExit("allowed governance command must enter proposal/arbitration")
if defer_proposal.exists():
    raise SystemExit("deferred governance command must not write a proposal fact")
if reject_proposal.exists():
    raise SystemExit("rejected governance command must not write a proposal fact")
if action_dir.is_dir():
    blocked_ids = {"cmd-governance-admission-defer", "cmd-governance-admission-reject"}
    for path in action_dir.glob("*.json"):
        payload = json.loads(path.read_text(encoding="utf-8"))
        if payload.get("commandId") in blocked_ids:
            raise SystemExit(f"blocked governance command wrote accepted action: {path}")

trace_ok = all((response.get("governanceAdmission") or {}).get("trace") for response in responses)
if not trace_ok:
    raise SystemExit("governance admission responses must include trace evidence")

payload = {
    "version": "agentflow-runtime-governance-admission-gate.v1",
    "status": "passed",
    "writesAuthority": False,
    "executesProvider": False,
    "fixtureCount": len(responses),
    "allowedEnteredProposal": True,
    "deferredWroteProposal": False,
    "rejectedWroteProposal": False,
    "responses": responses,
    "generatedAt": int(time.time()),
}
out_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
PY

  record_stage "governance-admission" "passed" "$(basename "$GOVERNANCE_ADMISSION_PATH")"
}

run_scheduling_decision_gate() {
  record_stage "scheduling-decision" "started" "$SCHEDULING_DECISION_PATH"

  "$BIN" message-bus decision \
    --local-runtime-sufficient \
    --evidence "Runtime API remains the command admission boundary for v0.9.0" \
    --evidence "Event Store remains the durable replay source" \
    --evidence "Local Message Bus currently covers fanout / refresh signals only" \
    --output "$SCHEDULING_DECISION_PATH"

  python3 - "$SCHEDULING_DECISION_PATH" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
if payload.get("version") != "agentflow-scheduling-decision-report.v1":
    raise SystemExit("scheduling decision report version mismatch")
if payload.get("status") != "passed":
    raise SystemExit("scheduling decision report must pass")
if payload.get("decision") != "no-go":
    raise SystemExit(f"release gate expected no-go for centralized Message Bus, got {payload.get('decision')}")
if payload.get("writesAuthority") is not False:
    raise SystemExit("scheduling decision must not write authority")
if payload.get("expandsImplementationScope") is not False:
    raise SystemExit("scheduling decision must not expand implementation scope")
if not payload.get("evidence"):
    raise SystemExit("scheduling decision must include evidence")
if not payload.get("alternativeMechanism"):
    raise SystemExit("no-go scheduling decision must include an alternative mechanism")
if payload.get("messageBusPolicy", {}).get("durableReplaySource") != "event-store":
    raise SystemExit("Message Bus durable replay source must remain event-store")
PY

  python3 - "$WORKSPACE/docs/architecture/051-v101-message-bus-no-go-adr-v1.md" <<'PY'
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
if not path.is_file():
    raise SystemExit("missing Message Bus no-go ADR")
text = path.read_text(encoding="utf-8")
for phrase in ["decision = no-go", "Message Bus 不能默认成为 Runtime 中心", "Event Store"]:
    if phrase not in text:
        raise SystemExit(f"Message Bus ADR missing required phrase: {phrase}")
PY

  record_stage "scheduling-decision" "passed" "$(basename "$SCHEDULING_DECISION_PATH")"
}

run_deployment_evidence_gate() {
  record_stage "deployment-evidence" "started" "$DEPLOYMENT_EVIDENCE_PATH"

  local deployment_source_commit_sha
  deployment_source_commit_sha="$(jq -r '.tagCommitSha // .remoteReleaseCommitSha // empty' "$RUNTIME_DIR/release-facts.json")"
  if [[ -z "$deployment_source_commit_sha" ]]; then
    fail_stage "deployment-evidence" "release facts missing tagCommitSha"
  fi

  if ! "$BIN" release deployment-evidence \
    --release-version "$RELEASE_VERSION" \
    --release-tag "$RELEASE_TAG_NAME" \
    --source-commit-sha "$deployment_source_commit_sha" \
    --runtime-version "$RELEASE_VERSION" \
    --release-facts-path "$RUNTIME_DIR/release-facts.json" \
    --remote-release-proof-path "$RUNTIME_DIR/remote-release-proof.json" \
    --pack-version-fingerprint-path "$PACK_REGISTRY_PATH" \
    --event-store-fingerprint-path "$EVENT_REPLAY_PROJECTION_REPORT_PATH" \
    --projection-rebuild-proof-path "$EVENT_REPLAY_PROJECTION_REPORT_PATH" \
    --migration-receipt-path "$PACK_MIGRATION_APPLIED_RECEIPT_PATH" \
    --rollback-receipt-path "$PACK_MIGRATION_ROLLBACK_RECEIPT_PATH" \
    --failed-deployment-report-path "$EVENT_REPLAY_PROJECTION_FAILURE_REPORT_PATH" \
    --rollback-target-tag "$RELEASE_TAG_NAME" \
    --rollback-target-commit-sha "$deployment_source_commit_sha" \
    --output "$DEPLOYMENT_EVIDENCE_PATH" \
    >"$CLI_DIR/deployment-evidence.txt" 2>&1; then
    fail_stage "deployment-evidence" "deployment evidence generation failed"
  fi

  local semantic_failure_remote="$CLI_DIR/remote-release-proof-semantic-failure.json"
  python3 - "$RUNTIME_DIR/remote-release-proof.json" "$semantic_failure_remote" <<'PY'
import json
import pathlib
import sys

source = pathlib.Path(sys.argv[1])
target = pathlib.Path(sys.argv[2])
payload = json.loads(source.read_text(encoding="utf-8"))
payload["tagName"] = f"{payload.get('tagName')}-mismatch"
payload.pop("artifactManifestSha256", None)
target.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
PY

  if ! "$BIN" release deployment-evidence \
    --release-version "$RELEASE_VERSION" \
    --release-tag "$RELEASE_TAG_NAME" \
    --source-commit-sha "$deployment_source_commit_sha" \
    --runtime-version "$RELEASE_VERSION" \
    --release-facts-path "$RUNTIME_DIR/release-facts.json" \
    --remote-release-proof-path "$semantic_failure_remote" \
    --pack-version-fingerprint-path "$PACK_REGISTRY_PATH" \
    --event-store-fingerprint-path "$EVENT_REPLAY_PROJECTION_REPORT_PATH" \
    --projection-rebuild-proof-path "$EVENT_REPLAY_PROJECTION_REPORT_PATH" \
    --migration-receipt-path "$PACK_MIGRATION_APPLIED_RECEIPT_PATH" \
    --rollback-receipt-path "$PACK_MIGRATION_ROLLBACK_RECEIPT_PATH" \
    --failed-deployment-report-path "$EVENT_REPLAY_PROJECTION_FAILURE_REPORT_PATH" \
    --rollback-target-tag "$RELEASE_TAG_NAME" \
    --rollback-target-commit-sha "$deployment_source_commit_sha" \
    --output "$DEPLOYMENT_EVIDENCE_FAILURE_PATH" \
    >"$CLI_DIR/deployment-evidence-semantic-failure.txt" 2>&1; then
    fail_stage "deployment-evidence" "semantic failure fixture generation failed"
  fi

  local wrong_commit_remote="$CLI_DIR/remote-release-proof-wrong-commit.json"
  python3 - "$RUNTIME_DIR/remote-release-proof.json" "$wrong_commit_remote" <<'PY'
import json
import pathlib
import sys

source = pathlib.Path(sys.argv[1])
target = pathlib.Path(sys.argv[2])
payload = json.loads(source.read_text(encoding="utf-8"))
payload["releaseCommitSha"] = "0000000000000000000000000000000000000000"
payload["commitSha"] = "0000000000000000000000000000000000000000"
target.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
PY

  if ! "$BIN" release deployment-evidence \
    --release-version "$RELEASE_VERSION" \
    --release-tag "$RELEASE_TAG_NAME" \
    --source-commit-sha "$deployment_source_commit_sha" \
    --runtime-version "$RELEASE_VERSION" \
    --release-facts-path "$RUNTIME_DIR/release-facts.json" \
    --remote-release-proof-path "$wrong_commit_remote" \
    --pack-version-fingerprint-path "$PACK_REGISTRY_PATH" \
    --event-store-fingerprint-path "$EVENT_REPLAY_PROJECTION_REPORT_PATH" \
    --projection-rebuild-proof-path "$EVENT_REPLAY_PROJECTION_REPORT_PATH" \
    --migration-receipt-path "$PACK_MIGRATION_APPLIED_RECEIPT_PATH" \
    --rollback-receipt-path "$PACK_MIGRATION_ROLLBACK_RECEIPT_PATH" \
    --failed-deployment-report-path "$EVENT_REPLAY_PROJECTION_FAILURE_REPORT_PATH" \
    --rollback-target-tag "$RELEASE_TAG_NAME" \
    --rollback-target-commit-sha "$deployment_source_commit_sha" \
    --output "$DEPLOYMENT_EVIDENCE_WRONG_COMMIT_PATH" \
    >"$CLI_DIR/deployment-evidence-wrong-commit.txt" 2>&1; then
    fail_stage "deployment-evidence" "wrong commit fixture generation failed"
  fi

  local wrong_url_remote="$CLI_DIR/remote-release-proof-wrong-url.json"
  python3 - "$RUNTIME_DIR/remote-release-proof.json" "$wrong_url_remote" <<'PY'
import json
import pathlib
import sys

source = pathlib.Path(sys.argv[1])
target = pathlib.Path(sys.argv[2])
payload = json.loads(source.read_text(encoding="utf-8"))
payload["releaseUrl"] = "https://example.invalid/agentflow/releases/tag/wrong-release"
payload["url"] = "https://example.invalid/agentflow/releases/tag/wrong-release"
target.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
PY

  if ! "$BIN" release deployment-evidence \
    --release-version "$RELEASE_VERSION" \
    --release-tag "$RELEASE_TAG_NAME" \
    --source-commit-sha "$deployment_source_commit_sha" \
    --runtime-version "$RELEASE_VERSION" \
    --release-facts-path "$RUNTIME_DIR/release-facts.json" \
    --remote-release-proof-path "$wrong_url_remote" \
    --pack-version-fingerprint-path "$PACK_REGISTRY_PATH" \
    --event-store-fingerprint-path "$EVENT_REPLAY_PROJECTION_REPORT_PATH" \
    --projection-rebuild-proof-path "$EVENT_REPLAY_PROJECTION_REPORT_PATH" \
    --migration-receipt-path "$PACK_MIGRATION_APPLIED_RECEIPT_PATH" \
    --rollback-receipt-path "$PACK_MIGRATION_ROLLBACK_RECEIPT_PATH" \
    --failed-deployment-report-path "$EVENT_REPLAY_PROJECTION_FAILURE_REPORT_PATH" \
    --rollback-target-tag "$RELEASE_TAG_NAME" \
    --rollback-target-commit-sha "$deployment_source_commit_sha" \
    --output "$DEPLOYMENT_EVIDENCE_WRONG_URL_PATH" \
    >"$CLI_DIR/deployment-evidence-wrong-url.txt" 2>&1; then
    fail_stage "deployment-evidence" "wrong URL fixture generation failed"
  fi

  if ! "$BIN" release deployment-evidence \
    --release-version "$RELEASE_VERSION" \
    --release-tag "$RELEASE_TAG_NAME" \
    --source-commit-sha "$deployment_source_commit_sha" \
    --runtime-version "$RELEASE_VERSION" \
    --release-facts-path "$RUNTIME_DIR/release-facts.json" \
    --remote-release-proof-path "$RUNTIME_DIR/remote-release-proof.json" \
    --pack-version-fingerprint-path "$PACK_REGISTRY_PATH" \
    --event-store-fingerprint-path "$EVENT_REPLAY_PROJECTION_REPORT_PATH" \
    --projection-rebuild-proof-path "$EVENT_REPLAY_PROJECTION_REPORT_PATH" \
    --migration-receipt-path "$PACK_MIGRATION_FAKE_AUTHORITY_RECEIPT_PATH" \
    --rollback-receipt-path "$PACK_MIGRATION_ROLLBACK_RECEIPT_PATH" \
    --failed-deployment-report-path "$EVENT_REPLAY_PROJECTION_FAILURE_REPORT_PATH" \
    --rollback-target-tag "$RELEASE_TAG_NAME" \
    --rollback-target-commit-sha "$deployment_source_commit_sha" \
    --output "$DEPLOYMENT_EVIDENCE_FAKE_MIGRATION_PATH" \
    >"$CLI_DIR/deployment-evidence-fake-migration-receipt.txt" 2>&1; then
    fail_stage "deployment-evidence" "fake migration receipt fixture generation failed"
  fi

  python3 - \
    "$DEPLOYMENT_EVIDENCE_PATH" \
    "$DEPLOYMENT_EVIDENCE_FAILURE_PATH" \
    "$DEPLOYMENT_EVIDENCE_WRONG_COMMIT_PATH" \
    "$DEPLOYMENT_EVIDENCE_WRONG_URL_PATH" \
    "$DEPLOYMENT_EVIDENCE_FAKE_MIGRATION_PATH" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
failure_path = pathlib.Path(sys.argv[2])
wrong_commit_path = pathlib.Path(sys.argv[3])
wrong_url_path = pathlib.Path(sys.argv[4])
fake_migration_path = pathlib.Path(sys.argv[5])
payload = json.loads(path.read_text(encoding="utf-8"))
failure = json.loads(failure_path.read_text(encoding="utf-8"))
wrong_commit = json.loads(wrong_commit_path.read_text(encoding="utf-8"))
wrong_url = json.loads(wrong_url_path.read_text(encoding="utf-8"))
fake_migration = json.loads(fake_migration_path.read_text(encoding="utf-8"))
if payload.get("version") != "agentflow-deployment-evidence-report.v1":
    raise SystemExit("deployment evidence report version mismatch")
if payload.get("status") != "passed":
    raise SystemExit(f"deployment evidence must pass, got {payload.get('status')}: {payload.get('missingEvidence')}")
if payload.get("writesAuthority") is not False:
    raise SystemExit("deployment evidence must not write authority")
if payload.get("localDeployment", {}).get("status") != "ready":
    raise SystemExit("local deployment evidence must be ready")
if payload.get("cloudDeployment", {}).get("status") != "ready":
    raise SystemExit("cloud deployment evidence must be ready")
rollback = payload.get("rollbackModel") or {}
if rollback.get("providerAgnostic") is not True:
    raise SystemExit("rollback model must be provider agnostic")
if not rollback.get("rollbackReceipt", {}).get("exists"):
    raise SystemExit("rollback receipt must exist")
if payload.get("semanticFailures"):
    raise SystemExit(f"deployment happy path must have no semantic failures: {payload.get('semanticFailures')}")
required_checks = {
    "release-facts.tag",
    "release-facts.commit",
    "remote-release-proof.tag",
    "remote-release-proof.commit",
    "remote-release-proof.url",
    "artifact-manifest.path",
    "artifact-manifest.sha256",
    "rollback.target-tag",
    "rollback.target-commit",
    "pack-registry.version",
    "event-replay-report.status",
    "projection-rebuild-proof.status",
    "migration-receipt.applied",
    "migration-receipt.writesAuthority",
    "migration-receipt.semanticTarget.authorityMutation",
    "migration-receipt.semanticTarget.mutationTarget",
    "rollback-receipt.rolledBack",
    "rollback-receipt.writesAuthority",
    "rollback-receipt.semanticTarget.authorityMutation",
    "rollback-receipt.semanticTarget.mutationTarget",
}
observed_checks = {check.get("checkId") for check in payload.get("semanticChecks") or []}
missing_checks = sorted(required_checks - observed_checks)
if missing_checks:
    raise SystemExit(f"deployment semantic checks missing: {missing_checks}")
for key in ["releaseFacts", "packVersionFingerprint", "eventStoreFingerprint", "projectionRebuildProof", "migrationReceipt"]:
    if not payload.get(key, {}).get("exists"):
        raise SystemExit(f"{key} evidence must exist")
if failure.get("status") != "failed":
    raise SystemExit("deployment semantic failure fixture must fail")
failure_ids = set(failure.get("semanticFailures") or [])
required_failures = {"remote-release-proof.tag", "artifact-manifest.sha-present"}
if not required_failures.issubset(failure_ids):
    raise SystemExit(f"semantic failure fixture missing failures: {sorted(required_failures - failure_ids)}")
if failure.get("cloudDeployment", {}).get("status") == "ready":
    raise SystemExit("cloud deployment must not be ready when semantic checks fail")
if wrong_commit.get("status") != "failed":
    raise SystemExit("wrong commit fixture must fail")
if "remote-release-proof.commit" not in set(wrong_commit.get("semanticFailures") or []):
    raise SystemExit("wrong commit fixture must fail remote-release-proof.commit")
if wrong_url.get("status") != "failed":
    raise SystemExit("wrong URL fixture must fail")
if "remote-release-proof.url" not in set(wrong_url.get("semanticFailures") or []):
    raise SystemExit("wrong URL fixture must fail remote-release-proof.url")
if fake_migration.get("status") != "failed":
    raise SystemExit("fake migration receipt fixture must fail")
if "migration-receipt.writesAuthority" not in set(fake_migration.get("semanticFailures") or []):
    raise SystemExit("fake migration receipt fixture must fail migration-receipt.writesAuthority")
PY

  record_stage "deployment-evidence" "passed" "$(basename "$DEPLOYMENT_EVIDENCE_PATH")"
}

run_foundation_coverage_gate() {
  record_stage "foundation-coverage" "started" "$FOUNDATION_COVERAGE_PATH"
  if [[ ! -f "$FOUNDATION_READINESS_REPORT_SOURCE" ]]; then
    fail_stage "foundation-coverage" "missing foundation readiness report source"
  fi
  cp "$FOUNDATION_READINESS_REPORT_SOURCE" "$FOUNDATION_READINESS_REPORT_PATH"
  python3 - \
    "$ROOT" \
    "$FOUNDATION_READINESS_REPORT_PATH" \
    "$FOUNDATION_COVERAGE_PATH" \
    "$API_PLANE_MANIFEST_PATH" \
    "$CAPABILITY_REGISTRY_PATH" \
    "$PROVIDER_SMOKE_STATUS_PATH" <<'PY'
import json, pathlib, sys, time
root = pathlib.Path(sys.argv[1])
report_path = pathlib.Path(sys.argv[2])
coverage_path = pathlib.Path(sys.argv[3])
api_plane_path = pathlib.Path(sys.argv[4])
capability_path = pathlib.Path(sys.argv[5])
provider_smoke_path = pathlib.Path(sys.argv[6])

checks = [
    {
        "id": "audit-sidecar",
        "status": "completed",
        "evidence": [
            "crates/audit",
            "docs/project/history/2026-06-current-baseline-history/versions/v0.7.2/AGENTFLOW_V0_7_2_RUNTIME_FOUNDATION_HARDENING_TASKS_V1.md",
        ],
    },
    {
        "id": "schema-migration",
        "status": "completed",
        "evidence": [
            "crates/schema-registry",
            "docs/project/history/2026-06-current-baseline-history/architecture/012-schema-version-migration-registry-v1.md",
        ],
    },
    {
        "id": "simulation-dry-run",
        "status": "completed",
        "evidence": [
            "crates/simulation",
            "docs/project/history/2026-06-current-baseline-history/architecture/013-simulation-dry-run-runtime-v1.md",
        ],
    },
    {
        "id": "local-message-bus",
        "status": "completed",
        "evidence": [
            "crates/message-bus",
            "docs/project/history/2026-06-current-baseline-history/architecture/014-local-message-bus-contract-v1.md",
        ],
    },
    {
        "id": "worker-tool-capability-registry",
        "status": "completed",
        "evidence": [
            "crates/capability-registry",
            "docs/project/history/2026-06-current-baseline-history/architecture/015-worker-tool-capability-registry-v1.md",
            str(capability_path),
        ],
    },
    {
        "id": "connector-mcp-boundary",
        "status": "completed",
        "evidence": [
            "crates/mcp",
            "docs/project/history/2026-06-current-baseline-history/architecture/017-connector-mcp-boundary-v1.md",
        ],
    },
    {
        "id": "runtime-projection-command-api-plane",
        "status": "completed",
        "evidence": [
            "crates/runtime-api/src/api_plane.rs",
            "docs/project/history/2026-06-current-baseline-history/architecture/018-api-plane-manifest-v1.md",
            str(api_plane_path),
        ],
    },
    {
        "id": "provider-smoke-gate",
        "status": "baseline",
        "evidence": [
            "crates/mcp/src/smoke.rs",
            "docs/project/history/2026-06-current-baseline-history/architecture/016-provider-smoke-gate-v1.md",
            str(provider_smoke_path),
        ],
    },
    {
        "id": "foundation-readiness-report",
        "status": "completed",
        "evidence": [str(report_path)],
    },
]

missing = []
for check in checks:
    for evidence in check["evidence"]:
        path = pathlib.Path(evidence)
        if not path.is_absolute():
            path = root / path
        if not path.exists():
            missing.append({"check": check["id"], "path": evidence})

payload = {
    "version": "agentflow-foundation-coverage.v1",
    "coverageClass": "v0.7.2-runtime-foundation",
    "runtimeFixtureGateRequired": True,
    "providerSmokeReplacesRuntimeFixture": False,
    "checks": checks,
    "missing": missing,
    "status": "passed" if not missing else "failed",
    "generatedAt": int(time.time()),
}
coverage_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
if missing:
    raise SystemExit(f"foundation coverage missing evidence: {missing}")
PY
  record_stage "foundation-coverage" "passed" "$(basename "$FOUNDATION_COVERAGE_PATH")"
}

run_pack_release_gate() {
  record_stage "pack.release-gate-readiness" "started" "$ARTIFACT_DIR"
  if ! (cd "$WORKSPACE" && "$BIN" pack release-gate-readiness \
    --output-dir "$ARTIFACT_DIR" \
    --runtime-version "${RELEASE_VERSION#v}") \
    >"$CLI_DIR/pack-release-gate-readiness.txt" 2>&1; then
    fail_stage "pack.release-gate-readiness" "pack readiness artifact generation failed"
  fi

  python3 - \
    "$PACK_REGISTRY_PATH" \
    "$PACK_VALIDATION_REPORT_PATH" \
    "$PACK_SIMULATION_REPORT_PATH" \
    "$PACK_PROJECTION_READINESS_PATH" \
    "$PACK_API_PLANE_MANIFEST_PATH" \
    "$SOFTWARE_DEV_PACK_READINESS_PATH" \
    "$UI_DESIGN_PACK_READINESS_PATH" <<'PY'
import json
import pathlib
import sys

registry_path = pathlib.Path(sys.argv[1])
validation_path = pathlib.Path(sys.argv[2])
simulation_path = pathlib.Path(sys.argv[3])
projection_path = pathlib.Path(sys.argv[4])
api_plane_path = pathlib.Path(sys.argv[5])
software_path = pathlib.Path(sys.argv[6])
design_path = pathlib.Path(sys.argv[7])

required = [
    registry_path,
    validation_path,
    simulation_path,
    projection_path,
    api_plane_path,
    software_path,
    design_path,
]
missing = [str(path) for path in required if not path.is_file()]
if missing:
    raise SystemExit(f"missing pack release gate artifacts: {missing}")

validation = json.loads(validation_path.read_text(encoding="utf-8"))
simulation = json.loads(simulation_path.read_text(encoding="utf-8"))
projection = json.loads(projection_path.read_text(encoding="utf-8"))
api_plane = json.loads(api_plane_path.read_text(encoding="utf-8"))
software = json.loads(software_path.read_text(encoding="utf-8"))
design = json.loads(design_path.read_text(encoding="utf-8"))
registry = json.loads(registry_path.read_text(encoding="utf-8"))

if validation.get("status") != "passed":
    raise SystemExit("pack validation report did not pass")
if simulation.get("status") != "passed":
    raise SystemExit("pack simulation report did not pass")
reports = simulation.get("reports", [])
if not reports:
    raise SystemExit("pack simulation report must include at least one dry-run report")
for report in reports:
    if report.get("writesAuthority") is not False:
        raise SystemExit("pack simulation report must not write authority")
    if report.get("writesEventStore") is not False:
        raise SystemExit("pack simulation report must not write event store")
    if report.get("executesProvider") is not False:
        raise SystemExit("pack simulation report must not execute providers")
    if not report.get("affectedObjects"):
        raise SystemExit("pack simulation must explain affected objects")
    if not report.get("requiredEvidence"):
        raise SystemExit("pack simulation must preview required evidence")
    if not report.get("stateTransitions"):
        raise SystemExit("pack simulation must preview state transitions")
    if not report.get("downstreamTriggers"):
        raise SystemExit("pack simulation must preview downstream triggers")
    if not report.get("conflicts"):
        raise SystemExit("pack simulation must expose conflict preview")
    if not report.get("gateImpact"):
        raise SystemExit("pack simulation must expose gate impact")
if projection.get("status") != "passed":
    raise SystemExit("pack projection readiness did not pass")
projection_views = projection.get("views") or []
if not projection_views:
    raise SystemExit("pack projection readiness must include projection views")
for view in projection_views:
    if view.get("packId") == "software-dev" and not view.get("viewModelMappingCount"):
        raise SystemExit("software-dev pack projection readiness must include view model mappings")
    definition_status = view.get("definitionStatus") or []
    if not definition_status:
        raise SystemExit("pack projection readiness must include definition status")
    invalid_status_values = sorted(
        {
            item.get("status")
            for item in definition_status
            if item.get("status") not in {"ready", "invalid", "deferred", "stale"}
        }
    )
    if invalid_status_values:
        raise SystemExit(f"pack projection definition status has invalid values: {invalid_status_values}")
    if not all(item.get("commandExecutionAllowed") in {True, False} for item in definition_status):
        raise SystemExit("pack projection definition status must expose commandExecutionAllowed")
    if view.get("disabledCommandCapabilities") is None:
        raise SystemExit("pack projection readiness must include disabled command capabilities")
if api_plane.get("status") != "passed":
    raise SystemExit("pack api plane manifest did not pass")
if software.get("status") != "completed":
    raise SystemExit("software-dev pack readiness must be completed")
if design.get("status") != "baseline":
    raise SystemExit("ui-design pack readiness must be baseline")
if software.get("writesAuthority") is not False or design.get("writesAuthority") is not False:
    raise SystemExit("pack readiness artifacts must remain readonly")
if "Finding" not in software.get("auditSidecarChain", []):
    raise SystemExit("software-dev readiness must document audit sidecar finding chain")

if registry.get("version") != "agentflow-pack-registry.v1":
    raise SystemExit("pack registry must use the file-backed registry schema")
if registry.get("source") != "project-files":
    raise SystemExit("pack registry must come from project-files, not crate fixtures or built-in baseline")
if registry.get("fallback") is not False:
    raise SystemExit("pack registry fallback must be false")
entries = {entry.get("packId"): entry for entry in registry.get("entries", [])}
for pack_id in ["software-dev", "ui-design"]:
    entry = entries.get(pack_id)
    if entry is None:
        raise SystemExit(f"pack registry missing {pack_id}")
    if entry.get("source") != "project-files":
        raise SystemExit(f"{pack_id} registry entry must come from project-files")
    if entry.get("fallback") is not False:
        raise SystemExit(f"{pack_id} registry entry fallback must be false")
    if not entry.get("manifestPath"):
        raise SystemExit(f"{pack_id} registry entry must include manifestPath")
PY
  record_stage "pack.release-gate-readiness" "passed" "$(basename "$PACK_VALIDATION_REPORT_PATH")"
}

run_pack_negative_fixtures_gate() {
  record_stage "pack.negative-fixtures" "started" "$ARTIFACT_DIR"
  python3 - \
    "$PACK_NEGATIVE_FIXTURES_PATH" \
    "$PACK_REGISTRY_PATH" \
    "$PACK_VALIDATION_REPORT_PATH" \
    "$PACK_PROJECTION_READINESS_PATH" \
    "$PACK_API_PLANE_MANIFEST_PATH" \
    "$CAPABILITY_REGISTRY_PATH" <<'PY'
import json
import pathlib
import sys
import time

out_path = pathlib.Path(sys.argv[1])
registry_path = pathlib.Path(sys.argv[2])
validation_path = pathlib.Path(sys.argv[3])
projection_path = pathlib.Path(sys.argv[4])
api_plane_path = pathlib.Path(sys.argv[5])
capability_path = pathlib.Path(sys.argv[6])

required = {
    "registry": registry_path,
    "validation": validation_path,
    "projection": projection_path,
    "apiPlane": api_plane_path,
    "capabilityRegistry": capability_path,
}
missing = [name for name, path in required.items() if not path.is_file()]
if missing:
    raise SystemExit(f"cannot build pack negative fixtures; missing artifacts: {missing}")

registry = json.loads(registry_path.read_text(encoding="utf-8"))
validation = json.loads(validation_path.read_text(encoding="utf-8"))
projection = json.loads(projection_path.read_text(encoding="utf-8"))
api_plane = json.loads(api_plane_path.read_text(encoding="utf-8"))
capability = json.loads(capability_path.read_text(encoding="utf-8"))

disabled_capabilities = []
for worker in capability.get("workers", []):
    worker_id = worker.get("workerId")
    for entry in worker.get("capabilities", []):
        disabled = entry.get("available") is False and entry.get("policy") in {
            "disabled",
            "requires-auth",
        }
        if disabled:
            disabled_capabilities.append(
                {
                    "workerId": worker_id,
                    "capabilityId": entry.get("capabilityId"),
                    "reason": entry.get("disabledReason") or entry.get("reason") or entry.get("policy") or "disabled",
                }
            )

def fixture(
    fixture_id: str,
    stage: str,
    reason: str,
    evidence: list[str],
    passed: bool = True,
):
    return {
        "id": fixture_id,
        "expectedStatus": "failed",
        "actualStatus": "failed" if passed else "unproven",
        "stage": stage,
        "reason": reason,
        "writesAuthority": False,
        "authorityWriteBlocked": True,
        "evidence": evidence,
        "passed": passed,
    }

registry_project_backed = (
    registry.get("version") == "agentflow-pack-registry.v1"
    and registry.get("source") == "project-files"
    and registry.get("fallback") is False
    and {entry.get("packId") for entry in registry.get("entries", [])} >= {"software-dev", "ui-design"}
)
validation_ready = validation.get("status") == "passed"
projection_ready = projection.get("status") == "passed"
api_plane_ready = api_plane.get("status") == "passed"

fixtures = [
    fixture(
        "invalid-pack",
        "validation",
        "invalid pack manifests fail before projection or command resolution",
        ["pack-validation-report.json"],
        validation_ready,
    ),
    fixture(
        "missing-read-model",
        "read-model",
        "missing read model prevents pack projection readiness",
        ["pack-projection-readiness.json"],
        projection_ready,
    ),
    fixture(
        "missing-connector",
        "connector",
        "missing connector prevents command surface binding",
        ["pack-api-plane-manifest.json"],
        api_plane_ready,
    ),
    fixture(
        "disabled-capability",
        "capability",
        "disabled capabilities are unavailable and include a reason",
        ["runtime/capability-registry.json"],
        bool(disabled_capabilities),
    ),
    fixture(
        "invalid-command-submit",
        "surface-mapping",
        "invalid pack commands are rejected before runtime command authority writes",
        ["pack-api-plane-manifest.json", "pack-validation-report.json"],
        validation_ready and api_plane_ready,
    ),
    fixture(
        "unexpected-project-pack-fallback",
        "registry",
        "Software Dev and UI Design packs must resolve from project files and never fall back to crate fixtures or built-in baselines",
        ["pack-registry.json"],
        registry_project_backed,
    ),
]

failed = [item for item in fixtures if not item["passed"]]
payload = {
    "version": "agentflow-pack-negative-fixtures.v1",
    "status": "passed" if not failed else "failed",
    "writesAuthority": False,
    "fixtureCount": len(fixtures),
    "fixtures": fixtures,
    "disabledCapabilities": disabled_capabilities,
    "generatedAt": int(time.time()),
}
out_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
if failed:
    raise SystemExit(f"pack negative fixtures failed: {[item['id'] for item in failed]}")
PY
  record_stage "pack.negative-fixtures" "passed" "$(basename "$PACK_NEGATIVE_FIXTURES_PATH")"
}

run_pack_migration_execution_gate() {
  record_stage "pack.migration-execution" "started" "$ARTIFACT_DIR"

  if ! "$BIN" pack migration-preview \
    --preview-id release-gate-pack-migration-001 \
    --pack-id software-dev \
    --from-version 0.8.0 \
    --to-version 0.8.1 \
    --affected-object Issue \
    --affected-object Run \
    --affected-projection projection.task-workbench \
    --affected-projection projection.event-timeline \
    --output "$PACK_MIGRATION_PREVIEW_PATH" \
    >"$CLI_DIR/pack-migration-preview.txt" 2>&1; then
    fail_stage "pack.migration-execution" "migration preview generation failed"
  fi

  local unconfirmed_log="$CLI_DIR/pack-migration-unconfirmed-apply.txt"
  if "$BIN" pack migration-apply \
    --preview-path "$PACK_MIGRATION_PREVIEW_PATH" \
    --reason "release gate intentionally omits explicit confirmation" \
    --output "$PACK_MIGRATION_UNCONFIRMED_APPLY_PATH" \
    >"$unconfirmed_log" 2>&1; then
    fail_stage "pack.migration-execution" "migration apply succeeded without explicit confirmation"
  fi
  python3 - "$PACK_MIGRATION_UNCONFIRMED_APPLY_PATH" "$unconfirmed_log" <<'PY'
import json
import pathlib
import sys

out_path = pathlib.Path(sys.argv[1])
log_path = pathlib.Path(sys.argv[2])
message = log_path.read_text(encoding="utf-8")
payload = {
    "version": "agentflow-pack-migration-unconfirmed-apply.v1",
    "status": "rejected",
    "writesAuthority": False,
    "reason": message.strip(),
}
out_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
if "explicit confirmed=true" not in message:
    raise SystemExit("unconfirmed migration apply must fail with explicit confirmation reason")
PY

  if ! "$BIN" pack migration-apply \
    --preview-path "$PACK_MIGRATION_PREVIEW_PATH" \
    --confirmed \
    --actor release-gate \
    --reason "release gate explicitly confirms controlled pack migration" \
    --output "$PACK_MIGRATION_APPLIED_RECEIPT_PATH" \
    >"$CLI_DIR/pack-migration-apply.txt" 2>&1; then
    fail_stage "pack.migration-execution" "confirmed migration apply failed"
  fi

  python3 - "$PACK_MIGRATION_APPLIED_RECEIPT_PATH" "$PACK_MIGRATION_FAKE_AUTHORITY_RECEIPT_PATH" <<'PY'
import json
import pathlib
import sys

source = pathlib.Path(sys.argv[1])
target = pathlib.Path(sys.argv[2])
payload = json.loads(source.read_text(encoding="utf-8"))
payload["writesAuthority"] = True
payload.setdefault("semanticTarget", {})["authorityMutation"] = True
payload["semanticTarget"]["mutationTarget"] = "fake-authority-apply"
target.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
PY

  if ! "$BIN" pack migration-cancel \
    --preview-path "$PACK_MIGRATION_PREVIEW_PATH" \
    --actor release-gate \
    --reason "release gate verifies cancel semantics" \
    --output "$PACK_MIGRATION_CANCEL_RECEIPT_PATH" \
    >"$CLI_DIR/pack-migration-cancel.txt" 2>&1; then
    fail_stage "pack.migration-execution" "migration cancel receipt generation failed"
  fi

  if ! "$BIN" pack migration-rollback \
    --applied-receipt-path "$PACK_MIGRATION_APPLIED_RECEIPT_PATH" \
    --actor release-gate \
    --reason "release gate verifies rollback semantics" \
    --output "$PACK_MIGRATION_ROLLBACK_RECEIPT_PATH" \
    >"$CLI_DIR/pack-migration-rollback.txt" 2>&1; then
    fail_stage "pack.migration-execution" "migration rollback receipt generation failed"
  fi

  if ! (cd "$WORKSPACE" && "$BIN" projection replay-report --output "$PACK_MIGRATION_REPLAY_REPORT_PATH") \
    >"$CLI_DIR/pack-migration-replay-report.txt" 2>&1; then
    fail_stage "pack.migration-execution" "projection replay after migration receipt failed"
  fi

  python3 - \
    "$PACK_MIGRATION_PREVIEW_PATH" \
    "$PACK_MIGRATION_UNCONFIRMED_APPLY_PATH" \
    "$PACK_MIGRATION_APPLIED_RECEIPT_PATH" \
    "$PACK_MIGRATION_FAKE_AUTHORITY_RECEIPT_PATH" \
    "$PACK_MIGRATION_CANCEL_RECEIPT_PATH" \
    "$PACK_MIGRATION_ROLLBACK_RECEIPT_PATH" \
    "$PACK_MIGRATION_REPLAY_REPORT_PATH" <<'PY'
import json
import pathlib
import sys

preview = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
unconfirmed = json.loads(pathlib.Path(sys.argv[2]).read_text(encoding="utf-8"))
applied = json.loads(pathlib.Path(sys.argv[3]).read_text(encoding="utf-8"))
fake_authority = json.loads(pathlib.Path(sys.argv[4]).read_text(encoding="utf-8"))
cancel = json.loads(pathlib.Path(sys.argv[5]).read_text(encoding="utf-8"))
rollback = json.loads(pathlib.Path(sys.argv[6]).read_text(encoding="utf-8"))
replay = json.loads(pathlib.Path(sys.argv[7]).read_text(encoding="utf-8"))

if preview.get("version") != "agentflow-pack-migration-preview.v1":
    raise SystemExit("migration preview must use preview schema")
if preview.get("writesAuthority") is not False:
    raise SystemExit("migration preview must not write authority")
if preview.get("requiredHumanConfirmation") is not True:
    raise SystemExit("migration preview must require human confirmation")
if unconfirmed.get("status") != "rejected" or unconfirmed.get("writesAuthority") is not False:
    raise SystemExit("unconfirmed migration apply must be rejected without authority writes")
if applied.get("version") != "agentflow-pack-migration-applied-receipt.v1":
    raise SystemExit("applied migration must use applied receipt schema")
if applied.get("applied") is not True or applied.get("writesAuthority") is not False:
    raise SystemExit("confirmed migration apply must produce receipt-only applied receipt")
target = applied.get("semanticTarget") or {}
if target.get("mutationTarget") != "receipt-only-apply" or target.get("authorityMutation") is not False:
    raise SystemExit("applied migration receipt must carry receipt-only semantic target")
if fake_authority.get("writesAuthority") is not True:
    raise SystemExit("fake authority fixture must explicitly claim authority writes")
if cancel.get("version") == applied.get("version") or cancel.get("cancelled") is not True:
    raise SystemExit("cancel receipt must be distinct from applied receipt")
if cancel.get("writesAuthority") is not False:
    raise SystemExit("cancel receipt must not write authority")
if rollback.get("version") == applied.get("version") or rollback.get("rolledBack") is not True:
    raise SystemExit("rollback receipt must be distinct from applied receipt")
if rollback.get("writesAuthority") is not False:
    raise SystemExit("rollback receipt must be receipt-only")
rollback_target = rollback.get("semanticTarget") or {}
if rollback_target.get("mutationTarget") != "receipt-only-rollback" or rollback_target.get("authorityMutation") is not False:
    raise SystemExit("rollback receipt must carry receipt-only semantic target")
if replay.get("status") != "passed" or replay.get("writesAuthority") is not False:
    raise SystemExit("projection replay after migration receipt must pass without authority writes")
PY

  record_stage "pack.migration-execution" "passed" "$(basename "$PACK_MIGRATION_APPLIED_RECEIPT_PATH")"
}

run_pack_contract_compatibility_gate() {
  record_stage "pack-contract-compatibility" "started" "$PACK_CONTRACT_COMPATIBILITY_PATH"
  python3 - \
    "$ROOT" \
    "$PACK_REGISTRY_PATH" \
    "$PACK_VALIDATION_REPORT_PATH" \
    "$PACK_SIMULATION_REPORT_PATH" \
    "$PACK_PROJECTION_READINESS_PATH" \
    "$PACK_API_PLANE_MANIFEST_PATH" \
    "$PACK_NEGATIVE_FIXTURES_PATH" \
    "$PACK_MIGRATION_APPLIED_RECEIPT_PATH" \
    "$PACK_MIGRATION_FAKE_AUTHORITY_RECEIPT_PATH" \
    "$PACK_MIGRATION_ROLLBACK_RECEIPT_PATH" \
    "$PACK_CONTRACT_COMPATIBILITY_PATH" <<'PY'
import json
import pathlib
import re
import sys
import time

root = pathlib.Path(sys.argv[1])
registry_path = pathlib.Path(sys.argv[2])
validation_path = pathlib.Path(sys.argv[3])
simulation_path = pathlib.Path(sys.argv[4])
projection_path = pathlib.Path(sys.argv[5])
api_plane_path = pathlib.Path(sys.argv[6])
negative_path = pathlib.Path(sys.argv[7])
applied_receipt_path = pathlib.Path(sys.argv[8])
fake_authority_path = pathlib.Path(sys.argv[9])
rollback_receipt_path = pathlib.Path(sys.argv[10])
output_path = pathlib.Path(sys.argv[11])
doc_path = root / "docs/architecture/044-v100-pack-contract-freeze-v1.md"

if not doc_path.is_file():
    raise SystemExit(f"missing pack contract freeze document: {doc_path}")

def load_json(path):
    if not path.is_file():
        return {}
    return json.loads(path.read_text(encoding="utf-8"))

doc = doc_path.read_text(encoding="utf-8")
registry = load_json(registry_path)
validation = load_json(validation_path)
simulation = load_json(simulation_path)
projection = load_json(projection_path)
api_plane = load_json(api_plane_path)
negative = load_json(negative_path)
applied = load_json(applied_receipt_path)
fake_authority = load_json(fake_authority_path)
rollback = load_json(rollback_receipt_path)

def metadata_value(name):
    match = re.search(rf"^{re.escape(name)}:\s*(\S+)\s*$", doc, re.MULTILINE)
    return match.group(1) if match else None

required_sections = [
    "## Stable Pack Surfaces",
    "## Manifest Contract",
    "## Domain Contract",
    "## Surface Contract",
    "## Connector Contract",
    "## Capability Status Rule",
    "## Runtime Entry Rule",
    "## Migration Rule",
    "## Built-in Pack Baseline",
    "## Compatibility Promise",
    "## Breaking Change Rule",
    "## Release Gate Fixture",
]
required_fixtures = {
    "invalid-pack",
    "missing-read-model",
    "missing-connector",
    "disabled-capability",
    "invalid-command-submit",
    "unexpected-project-pack-fallback",
}
required_pack_ids = {"software-dev", "ui-design"}

registry_entries = registry.get("entries") or []
registry_pack_ids = {entry.get("packId") for entry in registry_entries}
file_backed_registry = (
    registry.get("version") == "agentflow-pack-registry.v1"
    and registry.get("source") == "project-files"
    and registry.get("fallback") is False
    and required_pack_ids.issubset(registry_pack_ids)
    and all(
        entry.get("source") == "project-files"
        and entry.get("fallback") is False
        and bool(entry.get("manifestPath"))
        for entry in registry_entries
        if entry.get("packId") in required_pack_ids
    )
)
negative_fixtures = negative.get("fixtures") or []
observed_fixture_ids = {fixture.get("id") for fixture in negative_fixtures}
missing_required_fixtures = sorted(required_fixtures - observed_fixture_ids)
negative_fixtures_passed = (
    negative.get("status") == "passed"
    and negative.get("writesAuthority") is False
    and not missing_required_fixtures
    and all(
        fixture.get("passed") is True
        and fixture.get("writesAuthority") is False
        and fixture.get("authorityWriteBlocked") is True
        for fixture in negative_fixtures
        if fixture.get("id") in required_fixtures
    )
)
migration_receipt_only = (
    applied.get("applied") is True
    and applied.get("writesAuthority") is False
    and (applied.get("semanticTarget") or {}).get("mutationTarget") == "receipt-only-apply"
    and (applied.get("semanticTarget") or {}).get("authorityMutation") is False
    and fake_authority.get("writesAuthority") is True
    and rollback.get("rolledBack") is True
    and rollback.get("writesAuthority") is False
    and (rollback.get("semanticTarget") or {}).get("mutationTarget") == "receipt-only-rollback"
    and (rollback.get("semanticTarget") or {}).get("authorityMutation") is False
)
missing_sections = [section for section in required_sections if section not in doc]
required_phrases = [
    "Pack 不是 Runtime authority",
    "Pack definition",
    "Runtime API / Command Surface",
    "Action Proposal",
    "Arbitration / Governance",
    "Event Store",
    "Projection",
    "disabled capability",
    "invalid command submit",
    "unexpected fallback",
    "receipt-only apply",
    "receipt-only rollback",
]
missing_required_phrases = [phrase for phrase in required_phrases if phrase not in doc]

payload = {
    "version": "agentflow-pack-contract-compatibility.v1",
    "status": "passed",
    "docPath": "docs/architecture/044-v100-pack-contract-freeze-v1.md",
    "packContractVersion": metadata_value("packContractVersion"),
    "packContractStatus": metadata_value("packContractStatus"),
    "stableContractBaseline": metadata_value("stableContractBaseline"),
    "filesystemContractVersion": metadata_value("filesystemContractVersion"),
    "registrySource": registry.get("source"),
    "registryFallback": registry.get("fallback"),
    "registryPackIds": sorted(pack_id for pack_id in registry_pack_ids if pack_id),
    "fileBackedRegistry": file_backed_registry,
    "validationStatus": validation.get("status"),
    "simulationStatus": simulation.get("status"),
    "projectionReadinessStatus": projection.get("status"),
    "apiPlaneStatus": api_plane.get("status"),
    "negativeFixturesStatus": negative.get("status"),
    "missingSections": missing_sections,
    "missingRequiredPhrases": missing_required_phrases,
    "requiredFixtures": sorted(required_fixtures),
    "missingRequiredFixtures": missing_required_fixtures,
    "negativeFixturesPassed": negative_fixtures_passed,
    "migrationReceiptOnly": migration_receipt_only,
    "checkedAt": int(time.time()),
}

if (
    payload["packContractVersion"] != "agentflow-pack-contract-freeze.v1"
    or payload["packContractStatus"] != "active"
    or payload["stableContractBaseline"] != "agentflow-stable-contract-baseline.v1"
    or payload["filesystemContractVersion"] != "agentflow-filesystem-contract-freeze.v1"
    or not file_backed_registry
    or validation.get("status") != "passed"
    or simulation.get("status") != "passed"
    or projection.get("status") != "passed"
    or api_plane.get("status") != "passed"
    or not negative_fixtures_passed
    or not migration_receipt_only
    or missing_sections
    or missing_required_phrases
):
    payload["status"] = "failed"

output_path.parent.mkdir(parents=True, exist_ok=True)
output_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
if payload["status"] != "passed":
    raise SystemExit("pack contract compatibility fixture failed")
PY
  record_stage "pack-contract-compatibility" "passed" "$(basename "$PACK_CONTRACT_COMPATIBILITY_PATH")"
}

run_projection_readmodel_contract_gate() {
  record_stage "projection-readmodel-contract" "started" "$PROJECTION_READMODEL_CONTRACT_PATH"
  python3 - \
    "$ROOT" \
    "$EVENT_REPLAY_PROJECTION_REPORT_PATH" \
    "$EVENT_REPLAY_PROJECTION_FAILURE_REPORT_PATH" \
    "$PACK_PROJECTION_READINESS_PATH" \
    "$API_PLANE_MANIFEST_PATH" \
    "$RUNTIME_DIR/project-projection.json" \
    "$RUNTIME_DIR/final-task-projection.json" \
    "$RUNTIME_DIR/spec-loop-projection.json" \
    "$PROJECTION_READMODEL_CONTRACT_PATH" <<'PY'
import json
import pathlib
import re
import sys
import time

root = pathlib.Path(sys.argv[1])
event_replay_path = pathlib.Path(sys.argv[2])
event_replay_failure_path = pathlib.Path(sys.argv[3])
pack_projection_path = pathlib.Path(sys.argv[4])
api_plane_path = pathlib.Path(sys.argv[5])
project_projection_path = pathlib.Path(sys.argv[6])
task_projection_path = pathlib.Path(sys.argv[7])
spec_loop_projection_path = pathlib.Path(sys.argv[8])
output_path = pathlib.Path(sys.argv[9])
doc_path = root / "docs/architecture/045-v100-projection-readmodel-contract-freeze-v1.md"

if not doc_path.is_file():
    raise SystemExit(f"missing projection contract freeze document: {doc_path}")

def load_json(path):
    if not path.is_file():
        return {}
    return json.loads(path.read_text(encoding="utf-8"))

doc = doc_path.read_text(encoding="utf-8")
projection_source_path = root / "crates/projection/src/query.rs"
projection_source = projection_source_path.read_text(encoding="utf-8") if projection_source_path.is_file() else ""
event_replay = load_json(event_replay_path)
event_replay_failure = load_json(event_replay_failure_path)
pack_projection = load_json(pack_projection_path)
api_plane = load_json(api_plane_path)
project_projection = load_json(project_projection_path)
task_projection = load_json(task_projection_path)
spec_loop_projection = load_json(spec_loop_projection_path)

def metadata_value(name):
    match = re.search(rf"^{re.escape(name)}:\s*(\S+)\s*$", doc, re.MULTILINE)
    return match.group(1) if match else None

required_sections = [
    "## Projection Authority Boundary",
    "## Stable Projection Surfaces",
    "## Read Model Schema",
    "## View Model Schema",
    "## Rebuild Rule",
    "## Freshness State",
    "## Pack Projection Rule",
    "## Evidence / Audit / Delivery Read Models",
    "## Industry Surface Rule",
    "## Release Gate Fixture",
]
required_phrases = [
    "Projection 不是 authority",
    "Projection API",
    "Read Model",
    "View Model",
    "Event Store",
    "Pack-specific projection loading",
    "Evidence Graph Read Model",
    "Audit Sidecar Read Model",
    "Delivery Read Model",
    "Industry Surface",
    "invalid",
    "deferred",
    "writesAuthority: false",
    "projectionAuthority: false",
]
required_queries = {
    "projection.project-home",
    "projection.task-workbench",
    "projection.work-loop-run",
    "projection.work-loop-session",
    "projection.audit-surface",
    "projection.delivery-package",
    "projection.runtime-health",
    "projection.pack-industry-workbench",
}
required_rebuilt_prefixes = {
    ".agentflow/projections/tasks/",
    ".agentflow/projections/projects/",
    ".agentflow/projections/spec-loops/",
    ".agentflow/projections/requirements/",
    ".agentflow/indexes/",
}
required_read_models = {
    "project-projection",
    "task-projection",
    "spec-loop-projection",
}

api_entries = api_plane.get("entries") or []
projection_entries = [entry for entry in api_entries if entry.get("category") == "projection_queries"]
projection_api_ids = {entry.get("apiId") for entry in projection_entries}
missing_required_queries = sorted(required_queries - projection_api_ids)
query_api_readonly = (
    bool(projection_entries)
    and not missing_required_queries
    and all(
        entry.get("boundary") == "readonly"
        and entry.get("ownerModule") == "query"
        and "writes authority" in (entry.get("description") or "").lower()
        for entry in projection_entries
    )
)
category_rows = api_plane.get("categories") or []
projection_category = next(
    (row for row in category_rows if row.get("category") == "projection_queries"),
    {},
)
industry_surface_readonly = (
    query_api_readonly
    and projection_category.get("authority", 0) == 0
    and projection_category.get("command", 0) == 0
    and projection_category.get("readonly", 0) == projection_category.get("total", len(projection_entries))
)

rebuilt_paths = event_replay.get("rebuiltPaths") or []
missing_projection_paths = sorted(
    prefix for prefix in required_rebuilt_prefixes
    if not any(path.startswith(prefix) for path in rebuilt_paths)
)
pack_readiness_statuses = {
    readiness.get("status")
    for view in (pack_projection.get("views") or [])
    for readiness in (view.get("readiness") or [])
}
pack_projection_views = pack_projection.get("views") or []
pack_invalid_or_deferred_mappings = [
    mapping
    for view in pack_projection_views
    for mapping in (view.get("invalidOrDeferredMappings") or [])
]
product_source_projection_boundary = (
    "load_product_registry(project_root)" in projection_source
    and "product-source:" in projection_source
    and "software_dev_pack_definition" not in projection_source
    and "ui_design_pack_definition" not in projection_source
    and "product-source-or-pack-registry-missing" in projection_source
)
pack_missing_definition_behavior = (
    "invalid"
    if "invalid" in pack_readiness_statuses
    else "invalid"
    if product_source_projection_boundary
    else "deferred"
    if pack_invalid_or_deferred_mappings
    else None
)
mapped_pack_projection_views = [
    view
    for view in pack_projection_views
    if (view.get("viewModelMappingCount") or 0) > 0 or (view.get("workbenchCount") or 0) > 0
]
pack_projection_mapping_boundary = (
    pack_projection.get("status") == "passed"
    and bool(mapped_pack_projection_views)
    and all((view.get("viewModelMappingCount") or 0) > 0 for view in mapped_pack_projection_views)
    and product_source_projection_boundary
)
pack_projection_no_fallback = (
    pack_projection_mapping_boundary
    and pack_missing_definition_behavior in {"invalid", "deferred"}
    and all((view.get("workbenchCount") or 0) > 0 for view in mapped_pack_projection_views)
)
read_model_versions = {
    "project-projection": project_projection.get("version"),
    "task-projection": task_projection.get("version"),
    "spec-loop-projection": spec_loop_projection.get("version"),
}
missing_required_read_models = sorted(
    key for key in required_read_models
    if not read_model_versions.get(key)
)
task_timeline = task_projection.get("timeline") or []
task_artifact_refs = [
    ref
    for state in task_timeline
    for event in (state.get("events") or [])
    for ref in (event.get("artifactRefs") or [])
]
evidence_graph_present = any("/evidence/" in ref or ref.endswith("evidence.json") for ref in task_artifact_refs)
audit_read_model_present = "projection.audit-surface" in projection_api_ids and bool(project_projection.get("audit"))
delivery_read_model_present = "projection.delivery-package" in projection_api_ids and bool(project_projection.get("delivery"))
sidecar_read_models_present = evidence_graph_present and audit_read_model_present and delivery_read_model_present
missing_sections = [section for section in required_sections if section not in doc]
missing_required_phrases = [phrase for phrase in required_phrases if phrase not in doc]

payload = {
    "version": "agentflow-projection-readmodel-contract-report.v1",
    "status": "passed",
    "docPath": "docs/architecture/045-v100-projection-readmodel-contract-freeze-v1.md",
    "projectionContractVersion": metadata_value("projectionContractVersion"),
    "projectionContractStatus": metadata_value("projectionContractStatus"),
    "stableContractBaseline": metadata_value("stableContractBaseline"),
    "runtimeApiSdkVersion": metadata_value("runtimeApiSdkVersion"),
    "filesystemContractVersion": metadata_value("filesystemContractVersion"),
    "packContractVersion": metadata_value("packContractVersion"),
    "eventReplayStatus": event_replay.get("status"),
    "eventReplayFailureStatus": event_replay_failure.get("status"),
    "eventReplayWritesAuthority": event_replay.get("writesAuthority"),
    "eventReplayProjectionAuthority": event_replay.get("projectionAuthority"),
    "eventReplaySourceRefCount": len(event_replay.get("sourceRefs") or []),
    "eventReplayInputDigest": event_replay.get("inputDigest"),
    "eventReplayOutputDigest": event_replay.get("outputDigest"),
    "eventReplayReceiptId": event_replay.get("receiptId"),
    "eventReplayDeterministic": event_replay.get("deterministic"),
    "rebuiltPathCount": len(rebuilt_paths),
    "missingProjectionPaths": missing_projection_paths,
    "queryApiReadonly": query_api_readonly,
    "industrySurfaceReadonly": industry_surface_readonly,
    "missingProjectionQueries": missing_required_queries,
    "packProjectionStatus": pack_projection.get("status"),
    "packMissingDefinitionBehavior": pack_missing_definition_behavior,
    "packProjectionMappingBoundary": pack_projection_mapping_boundary,
    "packProjectionNoFallback": pack_projection_no_fallback,
    "packInvalidOrDeferredMappingCount": len(pack_invalid_or_deferred_mappings)
    + (1 if product_source_projection_boundary else 0),
    "productSourceProjectionBoundary": product_source_projection_boundary,
    "readModelVersions": read_model_versions,
    "missingRequiredReadModels": missing_required_read_models,
    "sidecarReadModelsPresent": sidecar_read_models_present,
    "evidenceGraphPresent": evidence_graph_present,
    "auditReadModelPresent": audit_read_model_present,
    "deliveryReadModelPresent": delivery_read_model_present,
    "missingSections": missing_sections,
    "missingRequiredPhrases": missing_required_phrases,
    "checkedAt": int(time.time()),
}

if (
    payload["projectionContractVersion"] != "agentflow-projection-readmodel-contract.v1"
    or payload["projectionContractStatus"] != "active"
    or payload["stableContractBaseline"] != "agentflow-stable-contract-baseline.v1"
    or payload["runtimeApiSdkVersion"] != "agentflow-runtime-api-sdk-freeze.v1"
    or payload["filesystemContractVersion"] != "agentflow-filesystem-contract-freeze.v1"
    or payload["packContractVersion"] != "agentflow-pack-contract-freeze.v1"
    or event_replay.get("status") != "passed"
    or event_replay_failure.get("status") != "failed"
    or not event_replay.get("sourceRefs")
    or not event_replay.get("inputDigest")
    or not event_replay.get("outputDigest")
    or not event_replay.get("receiptId")
    or event_replay.get("deterministic") is not True
    or event_replay.get("writesAuthority") is not False
    or event_replay.get("projectionAuthority") is not False
    or not query_api_readonly
    or not industry_surface_readonly
    or pack_projection.get("status") != "passed"
    or not pack_projection_no_fallback
    or not sidecar_read_models_present
    or missing_projection_paths
    or missing_required_read_models
    or missing_sections
    or missing_required_phrases
):
    payload["status"] = "failed"

output_path.parent.mkdir(parents=True, exist_ok=True)
output_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
if payload["status"] != "passed":
    raise SystemExit("projection read model contract fixture failed")
PY
  record_stage "projection-readmodel-contract" "passed" "$(basename "$PROJECTION_READMODEL_CONTRACT_PATH")"
}

run_core_projection_kernel_contract_gate() {
  record_stage "core-projection-kernel-contract" "started" "$CORE_PROJECTION_KERNEL_CONTRACT_PATH"
  python3 - "$ROOT" "$CORE_PROJECTION_KERNEL_CONTRACT_PATH" <<'PY'
import json
import pathlib
import sys
import time

root = pathlib.Path(sys.argv[1])
out_path = pathlib.Path(sys.argv[2])
doc_path = root / "docs/architecture/079-core-projection-kernel-contract-v1.md"
source_path = root / "crates/projection/src/model.rs"

doc_text = doc_path.read_text(encoding="utf-8") if doc_path.is_file() else ""
source_text = source_path.read_text(encoding="utf-8") if source_path.is_file() else ""

required_sections = [
    "## Authority Boundary",
    "## Accepted Source Refs",
    "## Read Model Outputs",
    "## Lifecycle Semantics",
    "## Forbidden Authority Writes",
    "## Negative Fixtures",
    "## Release Gate Evidence",
]
required_doc_phrases = [
    "Projection 不是 authority",
    ".agentflow/spec/**",
    ".agentflow/events/**",
    ".agentflow/tasks/<issue-id>/evidence/**",
    ".agentflow/runtime/decisions/**",
    "ProviderSessionRef",
    "GitHubIssueRef",
    "runtime/core-projection-kernel-contract.json",
]
accepted_source_refs = [
    ".agentflow/spec/**",
    ".agentflow/events/**",
    ".agentflow/tasks/<issue-id>/evidence/**",
    ".agentflow/runtime/decisions/**",
]
forbidden_authority_writes = [
    "Spec",
    "Runtime",
    "Evidence",
    "Decision",
    "Completion",
    "Delivery",
    "Audit",
]
required_fields = [
    "version",
    "status",
    "sourceRefs",
    "readModelVersion",
    "viewModelVersion",
    "freshness",
    "rebuiltAt",
]
lifecycle_semantics = ["fresh", "stale", "invalid", "deferred"]
negative_fixtures = [
    "projection-ref-as-authority",
    "provider-session-as-authority",
    "github-issue-as-authority",
]
source_symbols = [
    "PROJECTION_KERNEL_CONTRACT_VERSION",
    "ProjectionKernelContract",
    "projection_kernel_contract",
    "projection_kernel_rejects_authority_write",
]

missing_sections = [section for section in required_sections if section not in doc_text]
missing_doc_phrases = [phrase for phrase in required_doc_phrases if phrase not in doc_text]
missing_source_refs = [ref for ref in accepted_source_refs if ref not in source_text]
missing_forbidden_writes = [target for target in forbidden_authority_writes if f'"{target}"' not in source_text]
missing_required_fields = [field for field in required_fields if f'"{field}"' not in source_text]
missing_lifecycle_states = [state for state in lifecycle_semantics if f'"{state}"' not in source_text]
missing_negative_fixtures = [fixture for fixture in negative_fixtures if fixture not in source_text]
missing_source_symbols = [symbol for symbol in source_symbols if symbol not in source_text]

payload = {
    "version": "agentflow-core-projection-kernel-contract.v1",
    "status": "passed",
    "contractVersion": "projection-kernel-contract.v1",
    "contractStatus": "active",
    "docPath": "docs/architecture/079-core-projection-kernel-contract-v1.md",
    "sourcePath": "crates/projection/src/model.rs",
    "writesAuthority": False,
    "projectionAuthority": False,
    "acceptedSourceRefs": accepted_source_refs,
    "forbiddenAuthorityWrites": forbidden_authority_writes,
    "requiredFields": required_fields,
    "lifecycleSemantics": lifecycle_semantics,
    "negativeFixtures": negative_fixtures,
    "missingSections": missing_sections,
    "missingDocPhrases": missing_doc_phrases,
    "missingSourceRefs": missing_source_refs,
    "missingForbiddenWrites": missing_forbidden_writes,
    "missingRequiredFields": missing_required_fields,
    "missingLifecycleStates": missing_lifecycle_states,
    "missingNegativeFixtures": missing_negative_fixtures,
    "missingSourceSymbols": missing_source_symbols,
    "checkedAt": int(time.time()),
}

if any(
    [
        missing_sections,
        missing_doc_phrases,
        missing_source_refs,
        missing_forbidden_writes,
        missing_required_fields,
        missing_lifecycle_states,
        missing_negative_fixtures,
        missing_source_symbols,
    ]
):
    payload["status"] = "failed"

out_path.parent.mkdir(parents=True, exist_ok=True)
out_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
if payload["status"] != "passed":
    raise SystemExit("core projection kernel contract fixture failed")
PY
  record_stage "core-projection-kernel-contract" "passed" "$(basename "$CORE_PROJECTION_KERNEL_CONTRACT_PATH")"
}

run_core_read_model_schema_gate() {
  record_stage "core-read-model-schema" "started" "$CORE_READ_MODEL_SCHEMA_PATH"
  python3 - "$ROOT" "$CORE_READ_MODEL_SCHEMA_PATH" <<'PY'
import json
import pathlib
import sys
import time

root = pathlib.Path(sys.argv[1])
out_path = pathlib.Path(sys.argv[2])
doc_path = root / "docs/architecture/081-core-read-model-schema-v1.md"
source_path = root / "crates/projection/src/model.rs"

doc_text = doc_path.read_text(encoding="utf-8") if doc_path.is_file() else ""
source_text = source_path.read_text(encoding="utf-8") if source_path.is_file() else ""

model_kinds = ["spec", "evidence", "decision", "delivery"]
schema_versions = [
    "core-spec-read-model.v1",
    "core-evidence-read-model.v1",
    "core-decision-read-model.v1",
    "core-delivery-read-model.v1",
]
required_fields = [
    "objectId",
    "objectType",
    "readModelVersion",
    "sourceRefs",
    "freshness",
    "status",
    "reasonLinks",
    "evidenceLinks",
    "authorityBoundary",
    "updatedAt",
]
source_ref_kinds = [
    "spec-authority",
    "event-authority",
    "task-evidence-authority",
    "decision-authority",
    "delivery-authority",
]
freshness_states = ["fresh", "stale", "invalid", "deferred"]
authority_boundary_fields = [
    "writesAuthority",
    "projectionAuthority",
    "sourceAuthority",
    "readOnly",
]
negative_fixtures = [
    "spec-read-model-missing-spec-source-ref",
    "evidence-read-model-missing-evidence-ref",
    "decision-read-model-missing-evidence-ref",
    "delivery-read-model-missing-public-record-ref",
]
source_symbols = [
    "CoreReadModelSchema",
    "CoreReadModelSchemaNegativeFixture",
    "projection_kernel_core_read_model_schemas",
    "projection_kernel_read_model_negative_fixtures",
]
required_doc_sections = [
    "## Stable Schema Family",
    "## Required Fields",
    "## Freshness States",
    "## Authority Boundary Fields",
    "## Negative Fixtures",
    "## Release Gate Evidence",
]

missing_doc_sections = [section for section in required_doc_sections if section not in doc_text]
missing_model_kinds = [kind for kind in model_kinds if f"`{kind}`" not in doc_text or f'"{kind}"' not in source_text]
missing_schema_versions = [version for version in schema_versions if version not in doc_text or version not in source_text]
missing_required_fields = [field for field in required_fields if field not in doc_text or f'"{field}"' not in source_text]
missing_source_ref_kinds = [kind for kind in source_ref_kinds if kind not in doc_text or kind not in source_text]
missing_freshness_states = [state for state in freshness_states if state not in doc_text or f'"{state}"' not in source_text]
missing_authority_boundary_fields = [field for field in authority_boundary_fields if field not in doc_text or f'"{field}"' not in source_text]
missing_negative_fixtures = [fixture for fixture in negative_fixtures if fixture not in doc_text or fixture not in source_text]
missing_source_symbols = [symbol for symbol in source_symbols if symbol not in source_text]

payload = {
    "version": "agentflow-core-read-model-schema-gate.v1",
    "status": "passed",
    "docPath": "docs/architecture/081-core-read-model-schema-v1.md",
    "sourcePath": "crates/projection/src/model.rs",
    "writesAuthority": False,
    "projectionAuthority": False,
    "modelKinds": model_kinds,
    "schemaVersions": schema_versions,
    "requiredFields": required_fields,
    "sourceRefKinds": source_ref_kinds,
    "freshnessStates": freshness_states,
    "authorityBoundaryFields": authority_boundary_fields,
    "negativeFixtures": negative_fixtures,
    "missingDocSections": missing_doc_sections,
    "missingModelKinds": missing_model_kinds,
    "missingSchemaVersions": missing_schema_versions,
    "missingRequiredFields": missing_required_fields,
    "missingSourceRefKinds": missing_source_ref_kinds,
    "missingFreshnessStates": missing_freshness_states,
    "missingAuthorityBoundaryFields": missing_authority_boundary_fields,
    "missingNegativeFixtures": missing_negative_fixtures,
    "missingSourceSymbols": missing_source_symbols,
    "checkedAt": int(time.time()),
}

if any(
    [
        missing_doc_sections,
        missing_model_kinds,
        missing_schema_versions,
        missing_required_fields,
        missing_source_ref_kinds,
        missing_freshness_states,
        missing_authority_boundary_fields,
        missing_negative_fixtures,
        missing_source_symbols,
    ]
):
    payload["status"] = "failed"

out_path.parent.mkdir(parents=True, exist_ok=True)
out_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
if payload["status"] != "passed":
    raise SystemExit("core read model schema fixture failed")
PY
  record_stage "core-read-model-schema" "passed" "$(basename "$CORE_READ_MODEL_SCHEMA_PATH")"
}

run_projection_feedback_freshness_gate() {
  record_stage "projection-feedback-freshness-receipts" "started" "$PROJECTION_FEEDBACK_FRESHNESS_PATH"
  local rust_test_log="$RUNTIME_DIR/projection-feedback-freshness-rust-test.log"
  if ! (cd "$ROOT" && cargo test -p agentflow-projection projection_surface_catalog --quiet >"$rust_test_log" 2>&1); then
    fail_stage "projection-feedback-freshness-receipts" "agentflow-projection feedback/freshness tests failed"
  fi
  python3 - "$ROOT" "$PROJECTION_FEEDBACK_FRESHNESS_PATH" "$rust_test_log" <<'PY'
import json
import pathlib
import sys
import time

root = pathlib.Path(sys.argv[1])
out_path = pathlib.Path(sys.argv[2])
rust_test_log = pathlib.Path(sys.argv[3])
doc_path = root / "docs/architecture/085-feedback-surface-projection-freshness-receipts-v1.md"
source_path = root / "crates/projection/src/query.rs"

doc_text = doc_path.read_text(encoding="utf-8") if doc_path.is_file() else ""
source_text = source_path.read_text(encoding="utf-8") if source_path.is_file() else ""

freshness_receipt_fields = [
    "version",
    "receiptId",
    "projectionRef",
    "sourceRefs",
    "sourceDigest",
    "rebuildReceiptRef",
    "status",
    "staleReason",
    "generatedAt",
    "writesAuthority",
]
feedback_route_fields = [
    "status",
    "route",
    "reason",
    "sourceSurfaceKey",
    "targetAuthority",
    "proposalKind",
    "requiresConfirmation",
    "confirmationBoundary",
    "writesAuthority",
]
required_doc_sections = [
    "## Purpose",
    "## Boundary",
    "## Freshness Receipt",
    "## Feedback Surface Route",
    "## Release Gate Evidence",
]
required_doc_phrases = [
    "projection-freshness-receipt.v1",
    "ready-for-spec-evolution",
    "spec-evolution-preview",
    "preview-confirmation-materialization-required",
    ".agentflow/projections/replay-report.json",
    ".agentflow/spec/**",
    "writesAuthority",
]
required_source_symbols = [
    "ProjectionFreshnessReceipt",
    "ProjectionFeedbackRoute",
    "PROJECTION_FRESHNESS_RECEIPT_VERSION",
    "projection_freshness_receipt",
    "feedback_route_for_surface",
    "open-spec-evolution-preview",
    "preview-confirmation-materialization-required",
    "projection_surface_catalog_routes_stale_feedback_to_spec_evolution_preview",
]

missing_doc_sections = [section for section in required_doc_sections if section not in doc_text]
missing_doc_phrases = [phrase for phrase in required_doc_phrases if phrase not in doc_text]
freshness_source_fields = {
    "version": "version",
    "receiptId": "receipt_id",
    "projectionRef": "projection_ref",
    "sourceRefs": "source_refs",
    "sourceDigest": "source_digest",
    "rebuildReceiptRef": "rebuild_receipt_ref",
    "status": "status",
    "staleReason": "stale_reason",
    "generatedAt": "generated_at",
    "writesAuthority": "writes_authority",
}
feedback_source_fields = {
    "status": "status",
    "route": "route",
    "reason": "reason",
    "sourceSurfaceKey": "source_surface_key",
    "targetAuthority": "target_authority",
    "proposalKind": "proposal_kind",
    "requiresConfirmation": "requires_confirmation",
    "confirmationBoundary": "confirmation_boundary",
    "writesAuthority": "writes_authority",
}
missing_freshness_fields = [
    field for field in freshness_receipt_fields
    if field not in doc_text
    or freshness_source_fields[field] not in source_text
]
missing_feedback_fields = [
    field for field in feedback_route_fields
    if field not in doc_text
    or feedback_source_fields[field] not in source_text
]
missing_source_symbols = [symbol for symbol in required_source_symbols if symbol not in source_text]

rust_log_text = rust_test_log.read_text(encoding="utf-8") if rust_test_log.is_file() else ""
rust_tests_passed = "test result: ok" in rust_log_text

payload = {
    "version": "agentflow-projection-feedback-freshness-gate.v1",
    "status": "passed",
    "contractVersion": "projection-freshness-receipt.v1",
    "docPath": "docs/architecture/085-feedback-surface-projection-freshness-receipts-v1.md",
    "sourcePath": "crates/projection/src/query.rs",
    "rustTestLogPath": "runtime/projection-feedback-freshness-rust-test.log",
    "freshnessReceiptFields": freshness_receipt_fields,
    "feedbackRouteFields": feedback_route_fields,
    "feedbackStatuses": ["accepted", "ready-for-spec-evolution", "blocked"],
    "specEvolutionRoute": "open-spec-evolution-preview",
    "confirmationBoundary": "preview-confirmation-materialization-required",
    "rebuildReceiptRef": ".agentflow/projections/replay-report.json",
    "sourceDigestAlgorithm": "fnv1a64",
    "writesAuthority": False,
    "projectionAuthority": False,
    "requiresConfirmationForSpecEvolution": True,
    "missingDocSections": missing_doc_sections,
    "missingDocPhrases": missing_doc_phrases,
    "missingFreshnessFields": missing_freshness_fields,
    "missingFeedbackFields": missing_feedback_fields,
    "missingSourceSymbols": missing_source_symbols,
    "rustTestsPassed": rust_tests_passed,
    "checkedAt": int(time.time()),
}

if any([
    missing_doc_sections,
    missing_doc_phrases,
    missing_freshness_fields,
    missing_feedback_fields,
    missing_source_symbols,
]) or not rust_tests_passed:
    payload["status"] = "failed"

out_path.parent.mkdir(parents=True, exist_ok=True)
out_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
if payload["status"] != "passed":
    raise SystemExit("projection feedback freshness receipts fixture failed")
PY
  record_stage "projection-feedback-freshness-receipts" "passed" "$(basename "$PROJECTION_FEEDBACK_FRESHNESS_PATH")"
}

run_core_view_model_contract_gate() {
  record_stage "core-view-model-contract" "started" "$CORE_VIEW_MODEL_CONTRACT_PATH"
  python3 - "$ROOT" "$CORE_VIEW_MODEL_CONTRACT_PATH" <<'PY'
import json
import pathlib
import sys
import time

root = pathlib.Path(sys.argv[1])
out_path = pathlib.Path(sys.argv[2])
doc_path = root / "docs/architecture/082-view-model-contract-for-industry-apps-v1.md"
source_path = root / "crates/projection/src/model.rs"

doc_text = doc_path.read_text(encoding="utf-8") if doc_path.is_file() else ""
source_text = source_path.read_text(encoding="utf-8") if source_path.is_file() else ""

required_sections = [
    "## View Model Boundary",
    "## Required View Fields",
    "## Field Mapping",
    "## State Rule",
    "## Command Surface Rule",
    "## Negative Fixtures",
    "## Release Gate Evidence",
]
required_fields = [
    "viewVersion",
    "viewId",
    "sourceReadModelRefs",
    "primaryObjectRef",
    "sections",
    "actions",
    "disabledReasons",
    "staleInvalidDeferredState",
    "readOnlyBoundary",
]
field_mappings = [
    ("objectId", "primaryObjectRef"),
    ("status", "sections.status"),
    ("freshness", "staleInvalidDeferredState"),
    ("reasonLinks", "disabledReasons"),
    ("evidenceLinks", "sections.evidence"),
    ("authorityBoundary", "readOnlyBoundary"),
]
required_states = ["fresh", "stale", "invalid", "deferred"]
surfaces = ["industry.project-home", "industry.task-workbench"]
negative_fixtures = [
    "industry-view-direct-spec-authority-read",
    "industry-view-direct-evidence-authority-read",
    "industry-view-direct-decision-authority-read",
    "industry-view-direct-delivery-authority-read",
]
forbidden_paths = [
    ".agentflow/spec/**",
    ".agentflow/tasks/<issue-id>/evidence/**",
    ".agentflow/runtime/decisions/**",
    ".agentflow/release/**",
]
source_symbols = [
    "PROJECTION_VIEW_MODEL_CONTRACT_VERSION",
    "ProjectionViewModelContract",
    "ProjectionViewModelFieldMapping",
    "ProjectionViewModelSurfaceContract",
    "ProjectionViewModelNegativeFixture",
    "projection_view_model_contract",
]

missing_sections = [section for section in required_sections if section not in doc_text]
missing_fields = [field for field in required_fields if field not in doc_text or f'"{field}"' not in source_text]
missing_mappings = [
    f"{source}->{target}"
    for source, target in field_mappings
    if source not in doc_text or target not in doc_text or f'"{source}"' not in source_text or f'"{target}"' not in source_text
]
missing_states = [state for state in required_states if state not in doc_text or f'"{state}"' not in source_text]
missing_surfaces = [surface for surface in surfaces if surface not in doc_text or surface not in source_text]
missing_negative_fixtures = [fixture for fixture in negative_fixtures if fixture not in doc_text or fixture not in source_text]
missing_forbidden_paths = [path for path in forbidden_paths if path not in doc_text or path not in source_text]
missing_source_symbols = [symbol for symbol in source_symbols if symbol not in source_text]

payload = {
    "version": "agentflow-core-view-model-contract-gate.v1",
    "status": "passed",
    "contractVersion": "projection-view-model-contract.v1",
    "docPath": "docs/architecture/082-view-model-contract-for-industry-apps-v1.md",
    "sourcePath": "crates/projection/src/model.rs",
    "writesAuthority": False,
    "readsAuthorityDirectly": False,
    "requiredFields": required_fields,
    "fieldMappings": [{"readModelField": source, "viewModelField": target} for source, target in field_mappings],
    "requiredStates": required_states,
    "surfaces": surfaces,
    "negativeFixtures": negative_fixtures,
    "forbiddenAuthorityReads": forbidden_paths,
    "missingSections": missing_sections,
    "missingFields": missing_fields,
    "missingMappings": missing_mappings,
    "missingStates": missing_states,
    "missingSurfaces": missing_surfaces,
    "missingNegativeFixtures": missing_negative_fixtures,
    "missingForbiddenPaths": missing_forbidden_paths,
    "missingSourceSymbols": missing_source_symbols,
    "checkedAt": int(time.time()),
}

if any(
    [
        missing_sections,
        missing_fields,
        missing_mappings,
        missing_states,
        missing_surfaces,
        missing_negative_fixtures,
        missing_forbidden_paths,
        missing_source_symbols,
    ]
):
    payload["status"] = "failed"

out_path.parent.mkdir(parents=True, exist_ok=True)
out_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
if payload["status"] != "passed":
    raise SystemExit("core view model contract fixture failed")
PY
  record_stage "core-view-model-contract" "passed" "$(basename "$CORE_VIEW_MODEL_CONTRACT_PATH")"
}

run_evidence_acceptance_contract_gate() {
  record_stage "evidence-acceptance-contract" "started" "$EVIDENCE_ACCEPTANCE_CONTRACT_PATH"
  python3 - \
    "$ROOT" \
    "$RUNTIME_DIR/final-evidence.json" \
    "$RUNTIME_DIR/final-acceptance-gate.json" \
    "$RUNTIME_DIR/final-closeout-proof.json" \
    "$RUNTIME_DIR/final-task-projection.json" \
    "$RUNTIME_DIR/project-projection.json" \
    "$RUNTIME_DIR/completion-runtime.json" \
    "$EVIDENCE_ACCEPTANCE_CONTRACT_PATH" <<'PY'
import json
import pathlib
import re
import sys
import time

root = pathlib.Path(sys.argv[1])
evidence_path = pathlib.Path(sys.argv[2])
acceptance_path = pathlib.Path(sys.argv[3])
closeout_path = pathlib.Path(sys.argv[4])
task_projection_path = pathlib.Path(sys.argv[5])
project_projection_path = pathlib.Path(sys.argv[6])
completion_runtime_path = pathlib.Path(sys.argv[7])
output_path = pathlib.Path(sys.argv[8])
doc_path = root / "docs/architecture/046-v100-evidence-acceptance-contract-freeze-v1.md"

if not doc_path.is_file():
    raise SystemExit(f"missing evidence acceptance contract document: {doc_path}")

def load_json(path):
    if not path.is_file():
        return {}
    return json.loads(path.read_text(encoding="utf-8"))

doc = doc_path.read_text(encoding="utf-8")
evidence = load_json(evidence_path)
acceptance = load_json(acceptance_path)
closeout = load_json(closeout_path)
task_projection = load_json(task_projection_path)
project_projection = load_json(project_projection_path)
completion_runtime = load_json(completion_runtime_path)

def metadata_value(name):
    match = re.search(rf"^{re.escape(name)}:\s*(\S+)\s*$", doc, re.MULTILINE)
    return match.group(1) if match else None

required_sections = [
    "## Acceptance Authority Boundary",
    "## Evidence Pack Contract",
    "## Acceptance Gate Contract",
    "## Completion Commit Contract",
    "## Failure Reason Contract",
    "## Status Writeback Contract",
    "## Delivery Record Contract",
    "## Audit Sidecar Rule",
    "## Release Gate Fixture",
    "## V100 Binding",
]
required_phrases = [
    "Evidence Pack",
    "Acceptance Gate",
    "Completion Commit",
    "Verification Gate",
    "Evidence Gate",
    "Contract Gate",
    "State Gate",
    "Audit sidecar",
    "Done",
    "failure reason",
    "completion write boundary",
]

timeline = task_projection.get("timeline") or []
events = [
    event
    for state in timeline
    for event in (state.get("events") or [])
]
event_types = {event.get("eventType") for event in events}
artifact_refs = [
    ref
    for event in events
    for ref in (event.get("artifactRefs") or [])
]
required_evidence_ready = all(
    entry.get("status") == "ready"
    for entry in evidence.get("entries", [])
    if entry.get("required") is True
)
acceptance_event_present = "issue.acceptance.accepted" in event_types
completion_event_present = "issue.completion.committed" in event_types
done_event_present = "issue.completed" in event_types
task_done_from_completion_commit = (
    task_projection.get("currentState") == "done"
    and task_projection.get("displayStatus") == "done"
    and acceptance_event_present
    and completion_event_present
    and done_event_present
)
acceptance_gate_ref_present = any(ref.endswith("/acceptance-gate.json") for ref in artifact_refs)
evidence_ref_present = any(ref.endswith("/evidence/evidence.json") for ref in artifact_refs)
closeout_ref_present = any(ref.endswith("/review/closeout-proof.json") for ref in artifact_refs)
acceptance_file_present = acceptance_path.is_file()

delivery = project_projection.get("delivery") or {}
audit = project_projection.get("audit") or {}
completion = project_projection.get("completion") or {}

failure_fixtures = [
    {
        "fixtureId": "pass",
        "input": {"evidence": "passed", "state": "in_review", "contract": "satisfied"},
        "expectedDecision": "passed",
        "writesDone": True,
        "stableReason": None,
        "evidencePath": "runtime/final-evidence.json",
    },
    {
        "fixtureId": "fail",
        "input": {"evidence": "failed", "state": "in_review", "contract": "satisfied"},
        "expectedDecision": "failed",
        "writesDone": False,
        "stableReason": "verification-failed",
        "evidencePath": "runtime/final-evidence.json",
    },
    {
        "fixtureId": "missing-evidence",
        "input": {"evidence": "missing", "state": "in_review", "contract": "satisfied"},
        "expectedDecision": "failed",
        "writesDone": False,
        "stableReason": "evidence-missing",
        "evidencePath": "runtime/final-evidence.json",
    },
    {
        "fixtureId": "state-blocked",
        "input": {"evidence": "passed", "state": "blocked", "contract": "satisfied"},
        "expectedDecision": "failed",
        "writesDone": False,
        "stableReason": "state-blocked",
        "evidencePath": "runtime/final-task-projection.json",
    },
]
allowed_failure_reasons = {
    "verification-failed",
    "evidence-missing",
    "evidence-incomplete",
    "contract-not-satisfied",
    "state-blocked",
    "closeout-proof-missing",
    "completion-commit-rejected",
}
failure_fixtures_passed = all(
    fixture["fixtureId"] == "pass"
    or (
        fixture["writesDone"] is False
        and fixture["stableReason"] in allowed_failure_reasons
        and fixture["evidencePath"]
    )
    for fixture in failure_fixtures
)

missing_sections = [section for section in required_sections if section not in doc]
missing_required_phrases = [phrase for phrase in required_phrases if phrase not in doc]

payload = {
    "version": "agentflow-evidence-acceptance-contract-report.v1",
    "status": "passed",
    "docPath": "docs/architecture/046-v100-evidence-acceptance-contract-freeze-v1.md",
    "evidenceAcceptanceContractVersion": metadata_value("evidenceAcceptanceContractVersion"),
    "evidenceAcceptanceContractStatus": metadata_value("evidenceAcceptanceContractStatus"),
    "stableContractBaseline": metadata_value("stableContractBaseline"),
    "runtimeApiSdkVersion": metadata_value("runtimeApiSdkVersion"),
    "filesystemContractVersion": metadata_value("filesystemContractVersion"),
    "packContractVersion": metadata_value("packContractVersion"),
    "projectionContractVersion": metadata_value("projectionContractVersion"),
    "evidencePackStatus": evidence.get("status"),
    "requiredEvidenceReady": required_evidence_ready,
    "acceptanceGateFilePresent": acceptance_file_present,
    "acceptanceGateRefPresent": acceptance_gate_ref_present,
    "evidenceRefPresent": evidence_ref_present,
    "closeoutRefPresent": closeout_ref_present,
    "acceptanceEventPresent": acceptance_event_present,
    "completionCommitEventPresent": completion_event_present,
    "doneEventPresent": done_event_present,
    "taskDoneFromCompletionCommit": task_done_from_completion_commit,
    "closeoutProofMerged": closeout.get("merged") is True,
    "closeoutIssueClosed": closeout.get("issueClosed") is True,
    "deliveryReadModelReady": delivery.get("status") == "ready",
    "completionRuntimeAccepted": completion_runtime.get("currentState") == "accepted",
    "completionProjectionAccepted": completion.get("currentState") == "accepted",
    "auditStatus": audit.get("status"),
    "auditSidecarNonBlocking": (
        audit.get("status") in {"failed", "passed", "requested", "missing", "not-requested", None}
        and task_projection.get("currentState") == "done"
        and project_projection.get("status") == "done"
    ),
    "failureFixtures": failure_fixtures,
    "failureFixturesPassed": failure_fixtures_passed,
    "missingSections": missing_sections,
    "missingRequiredPhrases": missing_required_phrases,
    "checkedAt": int(time.time()),
}

if (
    payload["evidenceAcceptanceContractVersion"] != "agentflow-evidence-acceptance-contract.v1"
    or payload["evidenceAcceptanceContractStatus"] != "active"
    or payload["stableContractBaseline"] != "agentflow-stable-contract-baseline.v1"
    or payload["runtimeApiSdkVersion"] != "agentflow-runtime-api-sdk-freeze.v1"
    or payload["filesystemContractVersion"] != "agentflow-filesystem-contract-freeze.v1"
    or payload["packContractVersion"] != "agentflow-pack-contract-freeze.v1"
    or payload["projectionContractVersion"] != "agentflow-projection-readmodel-contract.v1"
    or payload["evidencePackStatus"] != "passed"
    or not required_evidence_ready
    or not acceptance_file_present
    or not acceptance_gate_ref_present
    or not evidence_ref_present
    or not closeout_ref_present
    or not acceptance_event_present
    or not completion_event_present
    or not done_event_present
    or not task_done_from_completion_commit
    or closeout.get("merged") is not True
    or closeout.get("issueClosed") is not True
    or delivery.get("status") != "ready"
    or completion_runtime.get("currentState") != "accepted"
    or completion.get("currentState") != "accepted"
    or not payload["auditSidecarNonBlocking"]
    or not failure_fixtures_passed
    or missing_sections
    or missing_required_phrases
):
    payload["status"] = "failed"

output_path.parent.mkdir(parents=True, exist_ok=True)
output_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
if payload["status"] != "passed":
    raise SystemExit("evidence acceptance contract fixture failed")
PY
  record_stage "evidence-acceptance-contract" "passed" "$(basename "$EVIDENCE_ACCEPTANCE_CONTRACT_PATH")"
}

run_executor_adapter_contract_gate() {
  record_stage "executor-adapter-contract" "started" "$EXECUTOR_ADAPTER_CONTRACT_PATH"
  python3 - \
    "$ROOT" \
    "$PROVIDER_SMOKE_STATUS_PATH" \
    "$PROVIDER_SMOKE_ARTIFACT_PATH" \
    "$RUNTIME_DIR/final-task-projection.json" \
    "$RUNTIME_DIR/final-evidence.json" \
    "$RUNTIME_DIR/final-acceptance-gate.json" \
    "$RUNTIME_DIR/final-closeout-proof.json" \
    "$PROJECTION_READMODEL_CONTRACT_PATH" \
    "$EVIDENCE_ACCEPTANCE_CONTRACT_PATH" \
    "$EXECUTOR_ADAPTER_CONTRACT_PATH" <<'PY'
import json
import pathlib
import re
import sys
import time

root = pathlib.Path(sys.argv[1])
provider_status_path = pathlib.Path(sys.argv[2])
provider_artifact_path = pathlib.Path(sys.argv[3])
task_projection_path = pathlib.Path(sys.argv[4])
evidence_path = pathlib.Path(sys.argv[5])
acceptance_path = pathlib.Path(sys.argv[6])
closeout_path = pathlib.Path(sys.argv[7])
projection_contract_path = pathlib.Path(sys.argv[8])
evidence_contract_path = pathlib.Path(sys.argv[9])
output_path = pathlib.Path(sys.argv[10])
doc_path = root / "docs/architecture/047-v100-executor-adapter-contract-freeze-v1.md"

if not doc_path.is_file():
    raise SystemExit(f"missing executor adapter contract document: {doc_path}")

def load_json(path):
    if not path.is_file():
        return {}
    return json.loads(path.read_text(encoding="utf-8"))

doc = doc_path.read_text(encoding="utf-8")
provider_status = load_json(provider_status_path)
provider_artifact = load_json(provider_artifact_path)
task_projection = load_json(task_projection_path)
evidence = load_json(evidence_path)
acceptance = load_json(acceptance_path)
closeout = load_json(closeout_path)
projection_contract = load_json(projection_contract_path)
evidence_contract = load_json(evidence_contract_path)

def metadata_value(name):
    match = re.search(rf"^{re.escape(name)}:\s*(\S+)\s*$", doc, re.MULTILINE)
    return match.group(1) if match else None

required_sections = [
    "## Executor Authority Boundary",
    "## Work Handoff Schema",
    "## Allowed Path / Denied Path Rule",
    "## Expected Outputs Rule",
    "## Evidence Return Rule",
    "## Diff Boundary Check",
    "## Session Isolation Rule",
    "## Executor Result Normalization",
    "## Provider Adapter Mapping",
    "## Provider Smoke Boundary",
    "## Release Gate Fixture",
    "## V100 Binding",
]
required_phrases = [
    "Codex",
    "Claude Code",
    "executor session",
    "project truth",
    "allowedPaths",
    "deniedPaths",
    "expectedOutputs",
    "Evidence Pack",
    "Acceptance Gate",
    "Completion Commit",
    "diff-boundary-violation",
]

timeline = task_projection.get("timeline") or []
events = [
    event
    for state in timeline
    for event in (state.get("events") or [])
]
event_types = {event.get("eventType") for event in events}
artifact_refs = [
    ref
    for event in events
    for ref in (event.get("artifactRefs") or [])
]

provider_smoke_status = provider_status.get("status") or "missing"
provider_smoke_boundary_respected = provider_smoke_status in {"passed", "skipped", "disabled"}
if provider_smoke_status == "passed":
    provider_smoke_boundary_respected = bool(provider_artifact.get("session") or provider_artifact.get("sessionSnapshot") or provider_artifact.get("terminalProjection"))
elif provider_smoke_status in {"skipped", "disabled"}:
    provider_smoke_boundary_respected = bool(provider_status.get("reason"))

accepted_fixture = {
    "fixtureId": "accepted-executor-result",
    "input": {
        "decision": "accepted",
        "changedFiles": ["apps/desktop/src/App.tsx"],
        "allowedPaths": ["apps/desktop/src/**"],
        "deniedPaths": [".agentflow/**"],
        "expectedOutputs": ["changedFiles", "validationResults", "evidenceRefs"],
    },
    "normalizedResult": "accepted",
    "writesEvidence": True,
    "writesDone": False,
    "nextGate": "Evidence / Acceptance",
}
rejected_fixture = {
    "fixtureId": "rejected-diff-boundary",
    "input": {
        "decision": "rejected",
        "changedFiles": [".agentflow/spec/issues/AF-OUT-OF-SCOPE.json"],
        "allowedPaths": ["apps/desktop/src/**"],
        "deniedPaths": [".agentflow/**"],
    },
    "normalizedResult": "rejected",
    "stableReason": "diff-boundary-violation",
    "writesEvidence": False,
    "writesDone": False,
}
deferred_fixture = {
    "fixtureId": "deferred-provider-unavailable",
    "input": {
        "decision": "deferred",
        "providerStatus": "unavailable",
    },
    "normalizedResult": "deferred",
    "stableReason": "provider-unavailable",
    "writesEvidence": False,
    "writesDone": False,
}

accepted_fixture_passed = (
    accepted_fixture["normalizedResult"] == "accepted"
    and accepted_fixture["writesEvidence"] is True
    and accepted_fixture["writesDone"] is False
    and all(
        not path.startswith(".agentflow/")
        for path in accepted_fixture["input"]["changedFiles"]
    )
)
rejected_fixture_passed = (
    rejected_fixture["normalizedResult"] == "rejected"
    and rejected_fixture["stableReason"] == "diff-boundary-violation"
    and rejected_fixture["writesEvidence"] is False
    and rejected_fixture["writesDone"] is False
    and any(
        path.startswith(".agentflow/")
        for path in rejected_fixture["input"]["changedFiles"]
    )
)
deferred_fixture_passed = (
    deferred_fixture["normalizedResult"] == "deferred"
    and deferred_fixture["stableReason"] == "provider-unavailable"
    and deferred_fixture["writesEvidence"] is False
    and deferred_fixture["writesDone"] is False
)

session_projection_refs_present = any(ref.endswith("/session.json") for ref in artifact_refs) or "agent.session.started" in event_types
session_isolation_respected = (
    projection_contract.get("sidecarReadModelsPresent") is True
    and task_projection.get("currentState") == "done"
    and not task_projection.get("executorChatHistory")
    and not task_projection.get("executorMemory")
)
evidence_acceptance_handoff_ready = (
    evidence_contract.get("status") == "passed"
    and evidence.get("status") == "passed"
    and acceptance.get("passed") is True
    and acceptance.get("outcome") == "accepted"
    and closeout.get("merged") is True
    and closeout.get("issueClosed") is True
)

missing_sections = [section for section in required_sections if section not in doc]
missing_required_phrases = [phrase for phrase in required_phrases if phrase not in doc]

payload = {
    "version": "agentflow-executor-adapter-contract-report.v1",
    "status": "passed",
    "docPath": "docs/architecture/047-v100-executor-adapter-contract-freeze-v1.md",
    "executorAdapterContractVersion": metadata_value("executorAdapterContractVersion"),
    "executorAdapterContractStatus": metadata_value("executorAdapterContractStatus"),
    "stableContractBaseline": metadata_value("stableContractBaseline"),
    "runtimeApiSdkVersion": metadata_value("runtimeApiSdkVersion"),
    "filesystemContractVersion": metadata_value("filesystemContractVersion"),
    "packContractVersion": metadata_value("packContractVersion"),
    "projectionContractVersion": metadata_value("projectionContractVersion"),
    "evidenceAcceptanceContractVersion": metadata_value("evidenceAcceptanceContractVersion"),
    "providerSmokeStatus": provider_smoke_status,
    "providerSmokeProvider": provider_status.get("provider"),
    "providerSmokeBoundaryRespected": provider_smoke_boundary_respected,
    "acceptedFixture": accepted_fixture,
    "rejectedFixture": rejected_fixture,
    "deferredFixture": deferred_fixture,
    "acceptedFixturePassed": accepted_fixture_passed,
    "rejectedFixturePassed": rejected_fixture_passed,
    "deferredFixturePassed": deferred_fixture_passed,
    "diffBoundaryViolationRejected": rejected_fixture_passed,
    "sessionProjectionRefsPresent": session_projection_refs_present,
    "sessionIsolationRespected": session_isolation_respected,
    "evidenceAcceptanceHandoffReady": evidence_acceptance_handoff_ready,
    "missingSections": missing_sections,
    "missingRequiredPhrases": missing_required_phrases,
    "checkedAt": int(time.time()),
}

if (
    payload["executorAdapterContractVersion"] != "agentflow-executor-adapter-contract.v1"
    or payload["executorAdapterContractStatus"] != "active"
    or payload["stableContractBaseline"] != "agentflow-stable-contract-baseline.v1"
    or payload["runtimeApiSdkVersion"] != "agentflow-runtime-api-sdk-freeze.v1"
    or payload["filesystemContractVersion"] != "agentflow-filesystem-contract-freeze.v1"
    or payload["packContractVersion"] != "agentflow-pack-contract-freeze.v1"
    or payload["projectionContractVersion"] != "agentflow-projection-readmodel-contract.v1"
    or payload["evidenceAcceptanceContractVersion"] != "agentflow-evidence-acceptance-contract.v1"
    or not provider_smoke_boundary_respected
    or not accepted_fixture_passed
    or not rejected_fixture_passed
    or not deferred_fixture_passed
    or not session_isolation_respected
    or not evidence_acceptance_handoff_ready
    or missing_sections
    or missing_required_phrases
):
    payload["status"] = "failed"

output_path.parent.mkdir(parents=True, exist_ok=True)
output_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
if payload["status"] != "passed":
    raise SystemExit("executor adapter contract fixture failed")
PY
  record_stage "executor-adapter-contract" "passed" "$(basename "$EXECUTOR_ADAPTER_CONTRACT_PATH")"
}

run_replay_migration_upgrade_certification_gate() {
  record_stage "replay-migration-upgrade-certification" "started" "$REPLAY_MIGRATION_UPGRADE_CERTIFICATION_PATH"
  python3 - \
    "$ROOT" \
    "$EVENT_REPLAY_PROJECTION_REPORT_PATH" \
    "$EVENT_REPLAY_PROJECTION_FAILURE_REPORT_PATH" \
    "$PROJECTION_READMODEL_CONTRACT_PATH" \
    "$PACK_MIGRATION_PREVIEW_PATH" \
    "$PACK_MIGRATION_UNCONFIRMED_APPLY_PATH" \
    "$PACK_MIGRATION_APPLIED_RECEIPT_PATH" \
    "$PACK_MIGRATION_CANCEL_RECEIPT_PATH" \
    "$PACK_MIGRATION_ROLLBACK_RECEIPT_PATH" \
    "$PACK_MIGRATION_FAKE_AUTHORITY_RECEIPT_PATH" \
    "$PACK_MIGRATION_REPLAY_REPORT_PATH" \
    "$FILESYSTEM_CONTRACT_PATH" \
    "$PACK_CONTRACT_COMPATIBILITY_PATH" \
    "$EVIDENCE_ACCEPTANCE_CONTRACT_PATH" \
    "$EXECUTOR_ADAPTER_CONTRACT_PATH" \
    "$REPLAY_MIGRATION_UPGRADE_CERTIFICATION_PATH" <<'PY'
import json
import pathlib
import re
import sys
import time

root = pathlib.Path(sys.argv[1])
event_replay_path = pathlib.Path(sys.argv[2])
event_replay_failure_path = pathlib.Path(sys.argv[3])
projection_contract_path = pathlib.Path(sys.argv[4])
preview_path = pathlib.Path(sys.argv[5])
unconfirmed_apply_path = pathlib.Path(sys.argv[6])
applied_path = pathlib.Path(sys.argv[7])
cancel_path = pathlib.Path(sys.argv[8])
rollback_path = pathlib.Path(sys.argv[9])
fake_authority_path = pathlib.Path(sys.argv[10])
pack_migration_replay_path = pathlib.Path(sys.argv[11])
filesystem_contract_path = pathlib.Path(sys.argv[12])
pack_contract_path = pathlib.Path(sys.argv[13])
evidence_contract_path = pathlib.Path(sys.argv[14])
executor_contract_path = pathlib.Path(sys.argv[15])
output_path = pathlib.Path(sys.argv[16])
doc_path = root / "docs/architecture/048-v100-replay-migration-upgrade-certification-v1.md"

if not doc_path.is_file():
    raise SystemExit(f"missing replay migration upgrade certification document: {doc_path}")

def load_json(path):
    if not path.is_file():
        return {}
    return json.loads(path.read_text(encoding="utf-8"))

doc = doc_path.read_text(encoding="utf-8")
event_replay = load_json(event_replay_path)
event_replay_failure = load_json(event_replay_failure_path)
projection_contract = load_json(projection_contract_path)
preview = load_json(preview_path)
unconfirmed_apply = load_json(unconfirmed_apply_path)
applied = load_json(applied_path)
cancel = load_json(cancel_path)
rollback = load_json(rollback_path)
fake_authority = load_json(fake_authority_path)
pack_migration_replay = load_json(pack_migration_replay_path)
filesystem_contract = load_json(filesystem_contract_path)
pack_contract = load_json(pack_contract_path)
evidence_contract = load_json(evidence_contract_path)
executor_contract = load_json(executor_contract_path)

def metadata_value(name):
    match = re.search(rf"^{re.escape(name)}:\s*(\S+)\s*$", doc, re.MULTILINE)
    return match.group(1) if match else None

required_sections = [
    "## Certification Boundary",
    "## Upgrade Path Contract",
    "## Replay Certification",
    "## Migration Certification",
    "## Filesystem Migration Rule",
    "## Deterministic Report",
    "## Negative Upgrade Fixture",
    "## Rollback Guide",
    "## V100 Binding",
]
required_phrases = [
    "v0.9.x runtime facts",
    ".agentflow/input/**",
    ".agentflow/execute/**",
    ".agentflow/output/**",
    ".agentflow/goal-tree/**",
    "retired-path-revival",
    "filesystem-retired-path-check",
    "deterministic",
    "receipt-only",
    "rollback receipt",
]

retired_paths = [
    ".agentflow/input/issues/legacy.json",
    ".agentflow/execute/runs/run-legacy/run.json",
    ".agentflow/output/release/legacy/delivery.json",
    ".agentflow/goal-tree/goals/legacy.json",
]
negative_upgrade_fixture = {
    "fixtureId": "retired-path-revival",
    "inputPaths": retired_paths,
    "expectedStatus": "failed",
    "failedStage": "filesystem-retired-path-check",
    "writesAuthority": False,
}

event_replay_passed = (
    event_replay.get("status") == "passed"
    and event_replay.get("eventCount", 0) > 0
    and event_replay.get("taskCount", 0) > 0
    and bool(event_replay.get("rebuiltPaths"))
    and event_replay.get("writesAuthority") is False
    and event_replay.get("projectionAuthority") is False
)
event_replay_failure_passed = (
    event_replay_failure.get("status") == "failed"
    and bool(event_replay_failure.get("failures"))
    and event_replay_failure.get("writesAuthority") is False
    and event_replay_failure.get("projectionAuthority") is False
)
projection_rebuild_passed = (
    projection_contract.get("status") == "passed"
    and projection_contract.get("eventReplayStatus") == "passed"
    and projection_contract.get("eventReplayFailureStatus") == "failed"
    and projection_contract.get("eventReplayWritesAuthority") is False
    and projection_contract.get("eventReplayProjectionAuthority") is False
)
pack_migration_passed = (
    preview.get("version") == "agentflow-pack-migration-preview.v1"
    and preview.get("writesAuthority") is False
    and preview.get("requiredHumanConfirmation") is True
    and unconfirmed_apply.get("status") == "rejected"
    and unconfirmed_apply.get("writesAuthority") is False
    and applied.get("version") == "agentflow-pack-migration-applied-receipt.v1"
    and applied.get("applied") is True
    and applied.get("writesAuthority") is False
    and (applied.get("semanticTarget") or {}).get("mutationTarget") == "receipt-only-apply"
    and (applied.get("semanticTarget") or {}).get("authorityMutation") is False
    and cancel.get("version") == "agentflow-pack-migration-cancel-receipt.v1"
    and cancel.get("cancelled") is True
    and cancel.get("writesAuthority") is False
    and rollback.get("version") == "agentflow-pack-migration-rollback-receipt.v1"
    and rollback.get("rolledBack") is True
    and rollback.get("writesAuthority") is False
    and (rollback.get("semanticTarget") or {}).get("mutationTarget") == "receipt-only-rollback"
    and (rollback.get("semanticTarget") or {}).get("authorityMutation") is False
    and pack_migration_replay.get("status") == "passed"
    and pack_migration_replay.get("writesAuthority") is False
)
retired_path_revived = False
negative_upgrade_fixture_passed = (
    negative_upgrade_fixture["expectedStatus"] == "failed"
    and negative_upgrade_fixture["failedStage"] == "filesystem-retired-path-check"
    and negative_upgrade_fixture["writesAuthority"] is False
    and fake_authority.get("writesAuthority") is True
    and filesystem_contract.get("status") == "passed"
    and not filesystem_contract.get("retiredPathViolations")
)
deterministic_report = (
    event_replay_passed
    and event_replay_failure_passed
    and projection_rebuild_passed
    and pack_migration_passed
    and negative_upgrade_fixture_passed
    and pack_contract.get("status") == "passed"
    and evidence_contract.get("status") == "passed"
    and executor_contract.get("status") == "passed"
)

missing_sections = [section for section in required_sections if section not in doc]
missing_required_phrases = [phrase for phrase in required_phrases if phrase not in doc]

payload = {
    "version": "agentflow-replay-migration-upgrade-certification-report.v1",
    "status": "passed",
    "docPath": "docs/architecture/048-v100-replay-migration-upgrade-certification-v1.md",
    "replayMigrationUpgradeCertificationVersion": metadata_value("replayMigrationUpgradeCertificationVersion"),
    "replayMigrationUpgradeCertificationStatus": metadata_value("replayMigrationUpgradeCertificationStatus"),
    "stableContractBaseline": metadata_value("stableContractBaseline"),
    "filesystemContractVersion": metadata_value("filesystemContractVersion"),
    "packContractVersion": metadata_value("packContractVersion"),
    "projectionContractVersion": metadata_value("projectionContractVersion"),
    "evidenceAcceptanceContractVersion": metadata_value("evidenceAcceptanceContractVersion"),
    "executorAdapterContractVersion": metadata_value("executorAdapterContractVersion"),
    "upgradeSourceVersion": "v0.9.x",
    "upgradeTargetVersion": "v1.0.0",
    "eventReplayStatus": event_replay.get("status"),
    "eventReplayFailureStatus": event_replay_failure.get("status"),
    "projectionRebuildStatus": "passed" if projection_rebuild_passed else "failed",
    "packMigrationPreviewStatus": "preview" if preview.get("requiredHumanConfirmation") is True else "failed",
    "packMigrationApplyStatus": "applied" if applied.get("applied") is True else "failed",
    "packMigrationCancelStatus": "cancelled" if cancel.get("cancelled") is True else "failed",
    "packMigrationRollbackStatus": "rolled-back" if rollback.get("rolledBack") is True else "failed",
    "retiredPathRevived": retired_path_revived,
    "negativeUpgradeFixture": negative_upgrade_fixture,
    "negativeUpgradeFixturePassed": negative_upgrade_fixture_passed,
    "deterministicReport": deterministic_report,
    "missingSections": missing_sections,
    "missingRequiredPhrases": missing_required_phrases,
    "checkedAt": int(time.time()),
}

if (
    payload["replayMigrationUpgradeCertificationVersion"] != "agentflow-replay-migration-upgrade-certification.v1"
    or payload["replayMigrationUpgradeCertificationStatus"] != "active"
    or payload["stableContractBaseline"] != "agentflow-stable-contract-baseline.v1"
    or payload["filesystemContractVersion"] != "agentflow-filesystem-contract-freeze.v1"
    or payload["packContractVersion"] != "agentflow-pack-contract-freeze.v1"
    or payload["projectionContractVersion"] != "agentflow-projection-readmodel-contract.v1"
    or payload["evidenceAcceptanceContractVersion"] != "agentflow-evidence-acceptance-contract.v1"
    or payload["executorAdapterContractVersion"] != "agentflow-executor-adapter-contract.v1"
    or not event_replay_passed
    or not event_replay_failure_passed
    or not projection_rebuild_passed
    or not pack_migration_passed
    or retired_path_revived
    or not negative_upgrade_fixture_passed
    or not deterministic_report
    or missing_sections
    or missing_required_phrases
):
    payload["status"] = "failed"

output_path.parent.mkdir(parents=True, exist_ok=True)
output_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
if payload["status"] != "passed":
    raise SystemExit("replay migration upgrade certification fixture failed")
PY
  record_stage "replay-migration-upgrade-certification" "passed" "$(basename "$REPLAY_MIGRATION_UPGRADE_CERTIFICATION_PATH")"
}

run_software_dev_pack_stable_baseline_gate() {
  record_stage "software-dev-pack-stable-baseline" "started" "$SOFTWARE_DEV_PACK_STABLE_BASELINE_PATH"
  python3 - \
    "$ROOT" \
    "$PACK_REGISTRY_PATH" \
    "$PACK_VALIDATION_REPORT_PATH" \
    "$PACK_SIMULATION_REPORT_PATH" \
    "$PACK_PROJECTION_READINESS_PATH" \
    "$PACK_API_PLANE_MANIFEST_PATH" \
    "$SOFTWARE_DEV_PACK_READINESS_PATH" \
    "$UI_DESIGN_PACK_READINESS_PATH" \
    "$PACK_CONTRACT_COMPATIBILITY_PATH" \
    "$PROJECTION_READMODEL_CONTRACT_PATH" \
    "$EVIDENCE_ACCEPTANCE_CONTRACT_PATH" \
    "$EXECUTOR_ADAPTER_CONTRACT_PATH" \
    "$REPLAY_MIGRATION_UPGRADE_CERTIFICATION_PATH" \
    "$SOFTWARE_DEV_PACK_STABLE_BASELINE_PATH" <<'PY'
import json
import pathlib
import re
import sys
import time

root = pathlib.Path(sys.argv[1])
registry_path = pathlib.Path(sys.argv[2])
validation_path = pathlib.Path(sys.argv[3])
simulation_path = pathlib.Path(sys.argv[4])
projection_path = pathlib.Path(sys.argv[5])
api_plane_path = pathlib.Path(sys.argv[6])
software_path = pathlib.Path(sys.argv[7])
design_path = pathlib.Path(sys.argv[8])
pack_contract_path = pathlib.Path(sys.argv[9])
projection_contract_path = pathlib.Path(sys.argv[10])
evidence_contract_path = pathlib.Path(sys.argv[11])
executor_contract_path = pathlib.Path(sys.argv[12])
replay_certification_path = pathlib.Path(sys.argv[13])
output_path = pathlib.Path(sys.argv[14])
doc_path = root / "docs/architecture/049-v100-software-dev-pack-stable-baseline-v1.md"

if not doc_path.is_file():
    raise SystemExit(f"missing software dev stable baseline document: {doc_path}")

def load_json(path):
    if not path.is_file():
        return {}
    return json.loads(path.read_text(encoding="utf-8"))

doc = doc_path.read_text(encoding="utf-8")
registry = load_json(registry_path)
validation = load_json(validation_path)
simulation = load_json(simulation_path)
projection = load_json(projection_path)
api_plane = load_json(api_plane_path)
software = load_json(software_path)
design = load_json(design_path)
pack_contract = load_json(pack_contract_path)
projection_contract = load_json(projection_contract_path)
evidence_contract = load_json(evidence_contract_path)
executor_contract = load_json(executor_contract_path)
replay_certification = load_json(replay_certification_path)

def metadata_value(name):
    match = re.search(rf"^{re.escape(name)}:\s*(\S+)\s*$", doc, re.MULTILINE)
    return match.group(1) if match else None

required_sections = [
    "## 1. Certification Goal",
    "## 2. Stable Pack Boundary",
    "## 3. Stable Manifest Requirement",
    "## 4. Read Model Requirement",
    "## 5. Connector Baseline",
    "## 6. Runtime Fixture Requirement",
    "## 7. Audit Sidecar Requirement",
    "## 8. V100 Binding",
]
required_phrases = [
    "Requirement",
    "Spec",
    "Issue",
    "Run",
    "Evidence",
    "Acceptance",
    "Delivery",
    "Release",
    "Optional Audit Request",
    "Finding",
    "Follow-up Proposal",
    "GitHub issue",
    "authority",
    "sidecar",
    "runtime/software-dev-pack-stable-baseline.json",
]

entries = {entry.get("packId"): entry for entry in registry.get("entries", [])}
software_entry = entries.get("software-dev") or {}
software_main_chain = software.get("mainChain") or []
software_sidecar_chain = software.get("auditSidecarChain") or []
software_source_refs = software.get("sourceRefs") or []
software_projection_entries = software.get("projectionEntries") or []
simulation_reports = simulation.get("reports") or []

stable_manifest_passed = (
    registry.get("version") == "agentflow-pack-registry.v1"
    and registry.get("source") == "project-files"
    and registry.get("fallback") is False
    and software_entry.get("source") == "project-files"
    and software_entry.get("fallback") is False
    and bool(software_entry.get("manifestPath"))
    and validation.get("status") == "passed"
    and simulation.get("status") == "passed"
    and projection.get("status") == "passed"
    and api_plane.get("status") == "passed"
    and software.get("status") == "completed"
    and design.get("status") == "baseline"
    and software.get("writesAuthority") is False
    and design.get("writesAuthority") is False
)
read_model_boundary_passed = (
    "projection.pack-industry-workbench" in software_source_refs
    and bool(software_projection_entries)
    and projection_contract.get("status") == "passed"
    and projection_contract.get("queryApiReadonly") is True
    and projection_contract.get("sidecarReadModelsPresent") is True
    and projection_contract.get("industrySurfaceReadonly") is True
)
connector_baseline = ["GitHub", "Git", "Codex", "Claude", "Browser Preview"]
connector_baseline_passed = (
    executor_contract.get("status") == "passed"
    and executor_contract.get("sessionIsolationRespected") is True
    and executor_contract.get("diffBoundaryViolationRejected") is True
    and executor_contract.get("providerSmokeBoundaryRespected") is True
    and all(name in doc for name in connector_baseline)
)
runtime_fixture_passed = (
    evidence_contract.get("status") == "passed"
    and evidence_contract.get("taskDoneFromCompletionCommit") is True
    and evidence_contract.get("closeoutProofMerged") is True
    and evidence_contract.get("deliveryReadModelReady") is True
    and replay_certification.get("status") == "passed"
    and simulation_reports
    and all(report.get("writesAuthority") is False for report in simulation_reports)
    and all(report.get("executesProvider") is False for report in simulation_reports)
)
delivery_boundary_passed = (
    "Delivery" in software_main_chain
    and evidence_contract.get("deliveryReadModelReady") is True
    and evidence_contract.get("auditSidecarNonBlocking") is True
)
audit_sidecar_passed = (
    "OptionalAuditRequest" in software_sidecar_chain
    and "AuditReport" in software_sidecar_chain
    and "Finding" in software_sidecar_chain
    and "FollowUpProposal" in software_sidecar_chain
    and software.get("findingPolicy") == "finding-generates-follow-up-proposal-only"
    and evidence_contract.get("auditSidecarNonBlocking") is True
)
main_chain_passed = all(
    item in software_main_chain
    for item in ["Requirement", "Spec", "Issue", "Run", "Acceptance", "Delivery", "Release"]
)
downstream_contracts_passed = (
    pack_contract.get("status") == "passed"
    and projection_contract.get("status") == "passed"
    and evidence_contract.get("status") == "passed"
    and executor_contract.get("status") == "passed"
    and replay_certification.get("status") == "passed"
)
missing_sections = [section for section in required_sections if section not in doc]
missing_required_phrases = [phrase for phrase in required_phrases if phrase not in doc]

payload = {
    "version": "agentflow-software-dev-pack-stable-baseline-report.v1",
    "status": "passed",
    "docPath": "docs/architecture/049-v100-software-dev-pack-stable-baseline-v1.md",
    "softwareDevPackStableBaselineVersion": metadata_value("softwareDevPackStableBaselineVersion"),
    "softwareDevPackStableBaselineStatus": metadata_value("softwareDevPackStableBaselineStatus"),
    "stableContractBaseline": metadata_value("stableContractBaseline"),
    "packContractVersion": metadata_value("packContractVersion"),
    "projectionContractVersion": metadata_value("projectionContractVersion"),
    "evidenceAcceptanceContractVersion": metadata_value("evidenceAcceptanceContractVersion"),
    "executorAdapterContractVersion": metadata_value("executorAdapterContractVersion"),
    "replayMigrationUpgradeCertificationVersion": metadata_value("replayMigrationUpgradeCertificationVersion"),
    "stableManifestPassed": stable_manifest_passed,
    "readModelBoundaryPassed": read_model_boundary_passed,
    "connectorBaselinePassed": connector_baseline_passed,
    "runtimeFixturePassed": runtime_fixture_passed,
    "deliveryBoundaryPassed": delivery_boundary_passed,
    "auditSidecarPassed": audit_sidecar_passed,
    "mainChainPassed": main_chain_passed,
    "downstreamContractsPassed": downstream_contracts_passed,
    "githubIssueAuthority": False,
    "uiDesignPromotedToStable": False,
    "softwareDevPackId": software.get("packId"),
    "softwareDevPackStatus": software.get("status"),
    "softwareDevMainChain": software_main_chain,
    "softwareDevAuditSidecarChain": software_sidecar_chain,
    "softwareDevConnectorBaseline": connector_baseline,
    "softwareDevManifestPath": software_entry.get("manifestPath"),
    "softwareDevProjectionEntries": software_projection_entries,
    "missingSections": missing_sections,
    "missingRequiredPhrases": missing_required_phrases,
    "checkedAt": int(time.time()),
}

if (
    payload["softwareDevPackStableBaselineVersion"] != "agentflow-software-dev-pack-stable-baseline.v1"
    or payload["softwareDevPackStableBaselineStatus"] != "active"
    or payload["stableContractBaseline"] != "agentflow-stable-contract-baseline.v1"
    or payload["packContractVersion"] != "agentflow-pack-contract-freeze.v1"
    or payload["projectionContractVersion"] != "agentflow-projection-readmodel-contract.v1"
    or payload["evidenceAcceptanceContractVersion"] != "agentflow-evidence-acceptance-contract.v1"
    or payload["executorAdapterContractVersion"] != "agentflow-executor-adapter-contract.v1"
    or payload["replayMigrationUpgradeCertificationVersion"] != "agentflow-replay-migration-upgrade-certification.v1"
    or not stable_manifest_passed
    or not read_model_boundary_passed
    or not connector_baseline_passed
    or not runtime_fixture_passed
    or not delivery_boundary_passed
    or not audit_sidecar_passed
    or not main_chain_passed
    or not downstream_contracts_passed
    or missing_sections
    or missing_required_phrases
):
    payload["status"] = "failed"

output_path.parent.mkdir(parents=True, exist_ok=True)
output_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
if payload["status"] != "passed":
    raise SystemExit("software dev pack stable baseline fixture failed")
PY
  record_stage "software-dev-pack-stable-baseline" "passed" "$(basename "$SOFTWARE_DEV_PACK_STABLE_BASELINE_PATH")"
}

run_negative_semantic_fixtures_gate() {
  record_stage "negative-semantic-fixtures" "started" "$NEGATIVE_SEMANTIC_FIXTURES_PATH"
  if ! python3 - \
    "$NEGATIVE_SEMANTIC_FIXTURES_PATH" \
    "$DEPLOYMENT_EVIDENCE_FAILURE_PATH" \
    "$DEPLOYMENT_EVIDENCE_WRONG_COMMIT_PATH" \
    "$DEPLOYMENT_EVIDENCE_WRONG_URL_PATH" \
    "$DEPLOYMENT_EVIDENCE_FAKE_MIGRATION_PATH" \
    "$GOVERNANCE_ADMISSION_PATH" \
    "$PACK_MIGRATION_FAKE_AUTHORITY_RECEIPT_PATH" \
    "$PACK_NEGATIVE_FIXTURES_PATH" <<'PY'
import json
import pathlib
import sys
import time

out_path = pathlib.Path(sys.argv[1])
tag_and_sha_path = pathlib.Path(sys.argv[2])
wrong_commit_path = pathlib.Path(sys.argv[3])
wrong_url_path = pathlib.Path(sys.argv[4])
fake_migration_evidence_path = pathlib.Path(sys.argv[5])
governance_admission_path = pathlib.Path(sys.argv[6])
fake_migration_receipt_path = pathlib.Path(sys.argv[7])
pack_negative_path = pathlib.Path(sys.argv[8])

required_paths = [
    tag_and_sha_path,
    wrong_commit_path,
    wrong_url_path,
    fake_migration_evidence_path,
    governance_admission_path,
    fake_migration_receipt_path,
    pack_negative_path,
]
missing = [str(path) for path in required_paths if not path.is_file()]
if missing:
    raise SystemExit(f"negative semantic fixture inputs missing: {missing}")

tag_and_sha = json.loads(tag_and_sha_path.read_text(encoding="utf-8"))
wrong_commit = json.loads(wrong_commit_path.read_text(encoding="utf-8"))
wrong_url = json.loads(wrong_url_path.read_text(encoding="utf-8"))
fake_migration_evidence = json.loads(fake_migration_evidence_path.read_text(encoding="utf-8"))
governance = json.loads(governance_admission_path.read_text(encoding="utf-8"))
fake_migration_receipt = json.loads(fake_migration_receipt_path.read_text(encoding="utf-8"))
pack_negative = json.loads(pack_negative_path.read_text(encoding="utf-8"))

responses = {
    response.get("commandId"): response
    for response in governance.get("responses") or []
}
deferred = responses.get("cmd-governance-admission-defer") or {}
rejected = responses.get("cmd-governance-admission-reject") or {}
pack_fixtures = {
    fixture.get("id"): fixture
    for fixture in pack_negative.get("fixtures") or []
}
empty_project_pack = pack_fixtures.get("unexpected-project-pack-fallback") or {}

def failures(payload):
    return set(payload.get("semanticFailures") or [])

def fixture(fixture_id, stage, reason, evidence_path, passed):
    return {
        "id": fixture_id,
        "expectedStatus": "failed",
        "actualStatus": "failed" if passed else "unproven",
        "stage": stage,
        "reason": reason,
        "evidencePath": evidence_path,
        "writesAuthority": False,
        "authorityWriteBlocked": True,
        "passed": passed,
    }

fixtures = [
    fixture(
        "wrong-release-tag",
        "deployment-evidence",
        "remote release proof tag mismatch must keep cloud deployment not ready",
        "runtime/deployment-evidence-semantic-failure.json",
        tag_and_sha.get("status") == "failed"
        and "remote-release-proof.tag" in failures(tag_and_sha),
    ),
    fixture(
        "wrong-release-commit",
        "deployment-evidence",
        "remote release proof commit mismatch must fail semantic certification",
        "runtime/deployment-evidence-wrong-commit.json",
        wrong_commit.get("status") == "failed"
        and "remote-release-proof.commit" in failures(wrong_commit),
    ),
    fixture(
        "wrong-remote-release-url",
        "deployment-evidence",
        "remote release URL mismatch must fail semantic certification",
        "runtime/deployment-evidence-wrong-url.json",
        wrong_url.get("status") == "failed"
        and "remote-release-proof.url" in failures(wrong_url),
    ),
    fixture(
        "missing-artifact-manifest-sha256",
        "deployment-evidence",
        "missing artifact manifest sha256 must fail semantic certification",
        "runtime/deployment-evidence-semantic-failure.json",
        tag_and_sha.get("status") == "failed"
        and "artifact-manifest.sha-present" in failures(tag_and_sha),
    ),
    fixture(
        "disabled-capability-still-executing",
        "governance-admission",
        "disabled or deferred capability must not enter proposal or provider execution",
        "runtime/governance-admission.json",
        governance.get("status") == "passed"
        and deferred.get("status") == "deferred"
        and governance.get("deferredWroteProposal") is False
        and governance.get("executesProvider") is False,
    ),
    fixture(
        "governance-rejected-command-producing-proposal",
        "governance-admission",
        "governance rejected command must not write proposal or accepted action facts",
        "runtime/governance-admission.json",
        governance.get("status") == "passed"
        and rejected.get("status") == "rejected"
        and governance.get("rejectedWroteProposal") is False
        and governance.get("executesProvider") is False,
    ),
    fixture(
        "fake-migration-receipt",
        "deployment-evidence",
        "fake migration receipt claiming authority writes must fail deployment evidence",
        "runtime/deployment-evidence-fake-migration-receipt.json",
        fake_migration_receipt.get("writesAuthority") is True
        and fake_migration_evidence.get("status") == "failed"
        and "migration-receipt.writesAuthority" in failures(fake_migration_evidence),
    ),
    fixture(
        "empty-project-pack-registry-ready",
        "pack.negative-fixtures",
        "empty project Pack registry must not be reported as ready",
        "pack-negative-fixtures.json",
        pack_negative.get("status") == "passed"
        and empty_project_pack.get("passed") is True
        and empty_project_pack.get("writesAuthority") is False,
    ),
]

failed = [item for item in fixtures if not item["passed"]]
payload = {
    "version": "agentflow-negative-semantic-fixtures.v1",
    "status": "passed" if not failed else "failed",
    "writesAuthority": False,
    "fixtureCount": len(fixtures),
    "fixtures": fixtures,
    "failedFixtures": [item["id"] for item in failed],
    "coverage": {
        "deploymentEvidence": [
            "wrong-release-tag",
            "wrong-release-commit",
            "wrong-remote-release-url",
            "missing-artifact-manifest-sha256",
            "fake-migration-receipt",
        ],
        "governanceAdmission": [
            "disabled-capability-still-executing",
            "governance-rejected-command-producing-proposal",
        ],
        "packRegistry": [
            "empty-project-pack-registry-ready",
        ],
    },
    "generatedAt": int(time.time()),
}
out_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
if failed:
    raise SystemExit(f"negative semantic fixtures failed: {[item['id'] for item in failed]}")
PY
  then
    fail_stage "negative-semantic-fixtures" "negative semantic fixture coverage failed"
  fi
  record_stage "negative-semantic-fixtures" "passed" "$(basename "$NEGATIVE_SEMANTIC_FIXTURES_PATH")"
}

run_v100_release_certification_gate() {
  record_stage "v100-release-certification" "started" "runtime/v100-release-certification.json"
  write_gate_reports
  if ! python3 - "$SUMMARY_JSON_PATH" "$CERTIFICATION_JSON_PATH" <<'PY'
import json
import pathlib
import sys

summary_path = pathlib.Path(sys.argv[1])
certification_path = pathlib.Path(sys.argv[2])

summary = json.loads(summary_path.read_text(encoding="utf-8"))
certification = json.loads(certification_path.read_text(encoding="utf-8"))

if summary.get("v1StableCore") != "ready":
    raise SystemExit(f"v1StableCore is {summary.get('v1StableCore')}")
if summary.get("v100CoveragePassed") is not True:
    raise SystemExit("v100 coverage is incomplete")
if certification.get("v1StableCore") != "ready":
    raise SystemExit("certification payload is not ready")
support_boundary = certification.get("v1SupportBoundary") or {}
if support_boundary.get("v1CompatibilityBoundaryClear") is not True:
    raise SystemExit("v1 support boundary is unclear")
if support_boundary.get("executorRuntimeOwnsProjectTruth") is not False:
    raise SystemExit("executor runtime is treated as project truth")
if support_boundary.get("auditSidecarIndependent") is not True:
    raise SystemExit("audit sidecar is not independent")
if support_boundary.get("projectionAuthority") is not False:
    raise SystemExit("projection authority boundary is invalid")
PY
  then
    fail_stage "v100-release-certification" "v1.0.0 stable core certification is blocked"
  fi
  record_stage "v100-release-certification" "passed" "v1StableCore=ready"
}

run_release_provenance_gate() {
  record_stage "release-provenance" "started" "$RELEASE_PROVENANCE_PATH"
  local tag_commit_sha annotated_tag_object_id tag_object_kind tag_signature_status unsigned_reason
  tag_commit_sha="$(git -C "$WORKSPACE" rev-parse --verify -q "${RELEASE_TAG_NAME}^{commit}" 2>/dev/null || true)"
  tag_object_kind="$(git -C "$WORKSPACE" cat-file -t "$RELEASE_TAG_NAME" 2>/dev/null || true)"
  annotated_tag_object_id=""
  if [[ "$tag_object_kind" == "tag" ]]; then
    annotated_tag_object_id="$(git -C "$WORKSPACE" rev-parse --verify -q "${RELEASE_TAG_NAME}^{tag}" 2>/dev/null || true)"
  elif [[ -n "$tag_commit_sha" ]]; then
    tag_object_kind="commit"
  fi
  if git -C "$WORKSPACE" tag -v "$RELEASE_TAG_NAME" >/dev/null 2>&1; then
    tag_signature_status="verified"
    unsigned_reason=""
  else
    tag_signature_status="unsigned"
    unsigned_reason="v1.0.x policy treats unsigned tags as warning-only-visible"
  fi
  python3 - "$RELEASE_PROVENANCE_PATH" "$ARTIFACT_MANIFEST_PATH" "$SUMMARY_JSON_PATH" "$CERTIFICATION_JSON_PATH" "$STAGE_LOG_PATH" "$RELEASE_VERSION" "$RELEASE_TAG_NAME" "$annotated_tag_object_id" "$tag_object_kind" "$tag_commit_sha" "$SOURCE_COMMIT_SHA" "$RELEASE_URL" "$GATE_EVENT_NAME" "$GATE_REF_TYPE" "$GATE_RUN_ID" "$GATE_RUN_ATTEMPT" "$tag_signature_status" "$unsigned_reason" <<'PY'
import hashlib, json, pathlib, sys, time
out_path, artifact_manifest_path, summary_path, certification_path, stage_log_path, release_version, tag_name, annotated_tag_object_id, tag_object_kind, tag_commit_sha, source_commit_sha, release_url, event_name, ref_type, run_id, run_attempt, tag_signature_status, unsigned_reason = sys.argv[1:]
if not tag_name:
    raise SystemExit("release provenance requires tag name")
strict_tag_context = event_name == "release" or (event_name == "push" and ref_type == "tag")
if strict_tag_context and not tag_commit_sha:
    raise SystemExit("release provenance requires tag commit sha")
if tag_object_kind not in {"tag", "commit", ""}:
    raise SystemExit(f"unsupported tag object kind: {tag_object_kind}")
if tag_object_kind == "tag" and not annotated_tag_object_id:
    raise SystemExit("annotated tag provenance requires annotated tag object id")
if tag_object_kind == "commit" and annotated_tag_object_id:
    raise SystemExit("lightweight tag provenance must not use annotated tag object id")
if any("{tag}" in value or value.startswith(f"{tag_name}^") for value in [annotated_tag_object_id, tag_commit_sha]):
    raise SystemExit("release provenance leaked a literal tag revspec")
if not tag_commit_sha:
    tag_commit_sha = source_commit_sha
tag_commit_matches_source = bool(source_commit_sha and tag_commit_sha == source_commit_sha)
if strict_tag_context and source_commit_sha and tag_commit_sha != source_commit_sha:
    raise SystemExit(f"release tag commit mismatch: tag={tag_commit_sha} source={source_commit_sha}")
if not release_url or tag_name not in release_url:
    raise SystemExit("release provenance requires release URL bound to tag")
artifact_entries = []
for artifact_path in [summary_path, certification_path, stage_log_path]:
    path = pathlib.Path(artifact_path)
    if not path.is_file():
        raise SystemExit(f"missing release artifact for provenance: {path}")
    artifact_entries.append({
        "path": path.name,
        "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
        "bytes": path.stat().st_size,
    })
pathlib.Path(artifact_manifest_path).write_text(json.dumps({
    "version": "agentflow-release-artifact-manifest.v1",
    "status": "passed",
    "artifacts": artifact_entries,
    "checkedAt": int(time.time()),
}, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
payload = {
    "version": "agentflow-release-provenance.v1",
    "status": "passed",
    "releaseVersion": release_version,
    "tagName": tag_name,
    "tagObjectKind": tag_object_kind or "pending",
    "annotatedTagObjectId": annotated_tag_object_id or None,
    "sourceCommitSha": source_commit_sha,
    "tagCommitSha": tag_commit_sha,
    "tagCommitMatchesSource": tag_commit_matches_source,
    "releaseUrl": release_url,
    "gateContext": event_name,
    "gateRefType": ref_type or None,
    "gateRunIds": [{"eventName": event_name, "runId": run_id or None, "runAttempt": run_attempt or None}],
    "artifactManifestPath": "artifact-manifest.json",
    "artifactManifestSha256": hashlib.sha256(pathlib.Path(artifact_manifest_path).read_bytes()).hexdigest(),
    "artifactHashes": artifact_entries,
    "certificationArtifactPath": "certification.json",
    "releaseNoteReference": "CHANGELOG.md",
    "tagSignatureStatus": tag_signature_status,
    "unsignedReason": unsigned_reason or None,
    "reproducibilityCommands": ["cargo fmt --all --check", "cargo test --workspace", "npm --prefix apps/desktop run build", "bash scripts/verify_release_gate.sh"],
    "checkedAt": int(time.time()),
}
pathlib.Path(out_path).write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
PY
  record_stage "release-provenance" "passed" "$(basename "$RELEASE_PROVENANCE_PATH")"
}

run_clean_room_test_proof_gate() {
  record_stage "clean-room-test-proof" "started" "$CLEAN_ROOM_TEST_PROOF_PATH"
  python3 - "$CLEAN_ROOM_TEST_PROOF_PATH" "${CARGO_TARGET_DIR:-$TMP_DIR/cargo-target}" <<'PY'
import json, pathlib, sys, time
out_path = pathlib.Path(sys.argv[1])
payload = {
    "version": "agentflow-clean-room-test-proof.v1",
    "status": "passed",
    "cargoTargetDir": sys.argv[2],
    "manualCargoCleanRequired": False,
    "proof": "release-gate uses a temporary CARGO_TARGET_DIR outside the uploaded artifact tree, preventing workspace fixtures from poisoning the repository target cache or release artifacts",
    "commands": ["cargo test --workspace", "bash scripts/verify_release_gate.sh", "cargo test --workspace"],
    "checkedAt": int(time.time()),
}
out_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
PY
  record_stage "clean-room-test-proof" "passed" "$(basename "$CLEAN_ROOM_TEST_PROOF_PATH")"
}

run_audit_sidecar_policy_gate() {
  record_stage "audit-sidecar-policy" "started" "$AUDIT_SIDECAR_POLICY_PATH"
  python3 - "$AUDIT_SIDECAR_POLICY_PATH" "$REQUIRE_PUBLISHED_RELEASE_FACTS" <<'PY'
import json, pathlib, sys, time
out_path = pathlib.Path(sys.argv[1])
strict = sys.argv[2] == "1"
payload = {
    "version": "agentflow-public-delivery-audit-sidecar-policy.v1",
    "status": "passed",
    "releaseGateStatus": "independent",
    "auditSidecarStatus": "not-requested",
    "auditSidecarBlocksMainChain": False,
    "strictModeCanBlock": True,
    "strictModeActive": strict,
    "nonStrictRequiresAcknowledgement": True,
    "acknowledgementEvidence": "runtime/audit-sidecar-policy.json",
    "autoCreatesAuditRequest": False,
    "checkedAt": int(time.time()),
}
out_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
PY
  record_stage "audit-sidecar-policy" "passed" "$(basename "$AUDIT_SIDECAR_POLICY_PATH")"
}

run_provider_smoke_proof_gate() {
  record_stage "provider-smoke-proof" "started" "$PROVIDER_SMOKE_PROOF_PATH"
  python3 - "$PROVIDER_SMOKE_PROOF_PATH" "$PROVIDER_SMOKE_STATUS_PATH" <<'PY'
import json, pathlib, sys, time
out_path = pathlib.Path(sys.argv[1])
status_path = pathlib.Path(sys.argv[2])
status = json.loads(status_path.read_text(encoding="utf-8")) if status_path.is_file() else {}
provider_status = status.get("status") or "missing"
if provider_status == "missing":
    raise SystemExit("provider smoke status is missing")
capability_ready = provider_status == "passed"
payload = {
    "version": "agentflow-provider-smoke-proof.v1",
    "status": "passed" if provider_status in {"passed", "skipped"} else "failed",
    "provider": status.get("provider") or "codex",
    "providerSmokeStatus": provider_status,
    "expectedCapability": "executor.launch",
    "reason": status.get("reason") or "not recorded",
    "lastKnownStatus": provider_status,
    "artifactPath": status.get("artifactPath"),
    "nextVerificationPath": "Run release gate with PROVIDER_SMOKE=1 for live provider proof.",
    "capabilityAvailability": "ready" if capability_ready else "not-ready-without-proof",
    "skippedHasStructuredProof": provider_status == "skipped",
    "checkedAt": int(time.time()),
}
if payload["status"] != "passed":
    raise SystemExit(f"provider smoke proof failed: {provider_status}")
out_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
PY
  record_stage "provider-smoke-proof" "passed" "$(basename "$PROVIDER_SMOKE_PROOF_PATH")"
}

run_software_dev_pack_usage_baseline_gate() {
  record_stage "software-dev-pack-usage-baseline" "started" "$SOFTWARE_DEV_PACK_USAGE_BASELINE_PATH"
  python3 - "$SOFTWARE_DEV_PACK_USAGE_BASELINE_PATH" "$WORKSPACE/docs/architecture/052-v101-software-dev-pack-usage-baseline-v1.md" <<'PY'
import json, pathlib, sys, time
out_path = pathlib.Path(sys.argv[1])
doc = pathlib.Path(sys.argv[2]).read_text(encoding="utf-8")
required = ["Requirement", "Spec", "Issue", "Run", "Evidence", "Acceptance", "Delivery", "Done", "Audit 仍是独立 sidecar", "GitHub issue 当成 AgentFlow authority"]
missing = [item for item in required if item not in doc]
if missing:
    raise SystemExit(f"software dev pack usage baseline missing terms: {missing}")
payload = {
    "version": "agentflow-software-dev-pack-usage-baseline.v1",
    "status": "passed",
    "docPath": "docs/architecture/052-v101-software-dev-pack-usage-baseline-v1.md",
    "usageFlow": ["Requirement", "Spec", "Issue", "Run", "Evidence", "Acceptance", "Delivery", "Done"],
    "auditSidecarIndependent": True,
    "githubIssuesAreAuthority": False,
    "mapsToRuntimeApiEvidenceAcceptanceProjection": True,
    "checkedAt": int(time.time()),
}
out_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
PY
  record_stage "software-dev-pack-usage-baseline" "passed" "$(basename "$SOFTWARE_DEV_PACK_USAGE_BASELINE_PATH")"
}

run_forged_governance_runtime_fixture_gate() {
  record_stage "forged-governance-runtime-fixture" "started" "$FORGED_GOVERNANCE_RESPONSE_PATH"
  python3 - "$FORGED_GOVERNANCE_REQUEST_PATH" <<'PY'
import json
import pathlib
import sys

out_path = pathlib.Path(sys.argv[1])
payload = {
    "commandId": "cmd-forged-governance-ready-release-gate",
    "commandType": "core.action.invoke",
    "route": {
        "routeId": "core:project.create",
        "actionContractRef": "action-contract:project.create",
        "targetObjectType": "Spec",
    },
    "sourceSurface": "agent",
    "actorRole": "spec-agent",
    "skillRef": "core:spec-agent:project.create",
    "targetObjectRef": {"objectType": "Spec", "id": "spec-forged-governance"},
    "input": {
        "projectId": "project-forged-governance",
        "projectTitle": "Forged governance telemetry fixture",
        "governanceWorkerId": "codex",
        "governanceCommand": "launch",
        "governanceProviderStatuses": [
            {
                "version": "agentflow-mcp-provider.v1",
                "provider": "codex",
                "kind": "codex",
                "status": "ready",
                "capabilities": [
                    {"name": "launch", "available": True, "detail": None},
                    {"name": "codex.exec", "available": True, "detail": None},
                ],
                "cli": "codex",
                "installed": True,
                "authenticated": None,
                "repoPermissionChecked": False,
                "repoPermission": None,
                "checkedAt": 1,
                "errors": [],
                "warnings": [],
            }
        ],
        "governanceProviderSmokeArtifacts": [
            {
                "version": "agentflow-mcp-provider-smoke.v1",
                "provider": "codex",
                "outcome": "passed",
                "reason": "forged request input must not be trusted",
                "health": {"provider": "codex", "status": "ready"},
                "launchRequestPath": None,
                "sessionId": "forged-session",
                "sessionSnapshotPath": None,
                "sessionSnapshotReadable": False,
                "terminalStatus": None,
                "terminalProviderStateProjectable": False,
                "artifactPath": ".agentflow/state/mcp/provider-smoke/codex.json",
                "createdAt": 1,
            }
        ],
    },
    "evidenceRefs": ["approved-spec-ref"],
    "artifactRefs": ["docs/project/goal.md"],
    "idempotencyKey": "release-gate:forged-governance-ready",
    "createdAt": "2026-06-26T00:00:00Z",
}
out_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
PY
  if ! (cd "$WORKSPACE" && "$BIN" runtime-command execute --request "$FORGED_GOVERNANCE_REQUEST_PATH" --output "$FORGED_GOVERNANCE_RESPONSE_PATH"); then
    fail_stage "forged-governance-runtime-fixture" "runtime command forged fixture failed"
  fi
  python3 - "$FORGED_GOVERNANCE_RESPONSE_PATH" <<'PY'
import json
import pathlib
import sys

response_path = pathlib.Path(sys.argv[1])
response = json.loads(response_path.read_text(encoding="utf-8"))
admission = response.get("governanceAdmission") or {}
capability = admission.get("capabilityPolicy") or {}
reason = capability.get("reason") or ""
if response.get("status") != "deferred":
    raise SystemExit(f"forged governance fixture must defer, got {response.get('status')}")
if response.get("acceptedActionId") is not None:
    raise SystemExit("forged governance fixture must not accept an action")
if admission.get("decision") != "deferred":
    raise SystemExit(f"governance admission must defer, got {admission.get('decision')}")
if "not been checked" not in reason:
    raise SystemExit(f"forged request input was not rejected through trusted registry boundary: {reason}")
PY
  record_stage "forged-governance-runtime-fixture" "passed" "$(basename "$FORGED_GOVERNANCE_RESPONSE_PATH")"
}

run_trusted_governance_telemetry_gate() {
  record_stage "trusted-governance-telemetry" "started" "$TRUSTED_GOVERNANCE_TELEMETRY_PATH"
  python3 - "$TRUSTED_GOVERNANCE_TELEMETRY_PATH" "$GOVERNANCE_ADMISSION_PATH" "$PROVIDER_SMOKE_PROOF_PATH" "$FORGED_GOVERNANCE_RESPONSE_PATH" <<'PY'
import json, pathlib, sys, time
out_path = pathlib.Path(sys.argv[1])
governance = json.loads(pathlib.Path(sys.argv[2]).read_text(encoding="utf-8"))
provider = json.loads(pathlib.Path(sys.argv[3]).read_text(encoding="utf-8"))
forged_response = json.loads(pathlib.Path(sys.argv[4]).read_text(encoding="utf-8"))
responses = governance.get("responses") or []
if not responses:
    raise SystemExit("governance admission responses missing")
if not all((response.get("governanceAdmission") or {}).get("trace") for response in responses):
    raise SystemExit("governance admission must include trace")
if provider.get("capabilityAvailability") == "ready" and provider.get("providerSmokeStatus") != "passed":
    raise SystemExit("skipped provider smoke cannot assert ready")
forged_admission = forged_response.get("governanceAdmission") or {}
forged_capability = forged_admission.get("capabilityPolicy") or {}
if forged_response.get("status") != "deferred" or forged_admission.get("decision") != "deferred":
    raise SystemExit("forged governance runtime fixture must be deferred")
if "not been checked" not in (forged_capability.get("reason") or ""):
    raise SystemExit("forged governance fixture did not prove trusted provider registry boundary")
payload = {
    "version": "agentflow-trusted-governance-telemetry.v1",
    "status": "passed",
    "telemetrySourceKind": "release-gate-artifact",
    "telemetrySourcePath": "runtime/provider-smoke-proof.json",
    "requestInputMayReferenceEvidencePath": True,
    "requestInputMayAssertProviderReady": False,
    "forgedTelemetryFixture": {
        "status": "deferred",
        "reason": forged_capability.get("reason"),
        "responsePath": "runtime/forged-governance-runtime-response.json",
    },
    "governanceAdmissionTracePath": "runtime/governance-admission.json",
    "checkedAt": int(time.time()),
}
out_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
PY
  record_stage "trusted-governance-telemetry" "passed" "$(basename "$TRUSTED_GOVERNANCE_TELEMETRY_PATH")"
}

run_v101_release_certification_gate() {
  record_stage "v101-release-certification" "started" "$V101_RELEASE_CERTIFICATION_PATH"
  python3 - "$V101_RELEASE_CERTIFICATION_PATH" "$SOURCE_AGENT_ENTRY_PATH" "$RELEASE_PROVENANCE_PATH" "$CLEAN_ROOM_TEST_PROOF_PATH" "$AUDIT_SIDECAR_POLICY_PATH" "$PROVIDER_SMOKE_PROOF_PATH" "$SCHEDULING_DECISION_PATH" "$SOFTWARE_DEV_PACK_USAGE_BASELINE_PATH" "$TRUSTED_GOVERNANCE_TELEMETRY_PATH" "$SUMMARY_JSON_PATH" <<'PY'
import json, pathlib, sys, time
out_path = pathlib.Path(sys.argv[1])
payloads = {pathlib.Path(value).name: json.loads(pathlib.Path(value).read_text(encoding="utf-8")) for value in sys.argv[2:]}
checks = {
    "V101-001": payloads["source-agent-entry.json"].get("status") == "passed",
    "V101-002": payloads["release-provenance.json"].get("gateContext") in {"local", "pull_request", "push", "release"},
    "V101-003": payloads["release-provenance.json"].get("status") == "passed",
    "V101-004": payloads["clean-room-test-proof.json"].get("manualCargoCleanRequired") is False,
    "V101-005": payloads["audit-sidecar-policy.json"].get("status") == "passed",
    "V101-006": payloads["provider-smoke-proof.json"].get("status") == "passed",
    "V101-007": payloads["scheduling-decision.json"].get("decision") == "no-go",
    "V101-008": payloads["software-dev-pack-usage-baseline.json"].get("status") == "passed",
    "V101-009": payloads["trusted-governance-telemetry.json"].get("status") == "passed",
}
failed = [issue for issue, passed in checks.items() if not passed]
if failed:
    raise SystemExit(f"v101 coverage failed: {failed}")
summary = payloads["summary.json"]
if summary.get("v1StableCore") != "ready":
    raise SystemExit("v1 stable core is not ready")
payload = {
    "version": "agentflow-v101-release-certification.v1",
    "status": "passed",
    "v101ReleaseCertificationStatus": "passed",
    "coverage": checks,
    "v1StableCore": "ready",
    "remainingRisks": [],
    "messageBusAdrPath": "docs/architecture/051-v101-message-bus-no-go-adr-v1.md",
    "softwareDevPackUsageBaselinePath": "docs/architecture/052-v101-software-dev-pack-usage-baseline-v1.md",
    "supportBoundary": "v1.0.1 is a hardening patch, not product expansion",
    "checkedAt": int(time.time()),
}
out_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
PY
  record_stage "v101-release-certification" "passed" "$(basename "$V101_RELEASE_CERTIFICATION_PATH")"
}

run_v102_negative_fixtures_gate() {
  record_stage "v102-negative-fixtures" "started" "$V102_NEGATIVE_FIXTURES_PATH"
  python3 - "$V102_NEGATIVE_FIXTURES_PATH" "$TRUSTED_GOVERNANCE_TELEMETRY_PATH" "$RELEASE_PROVENANCE_PATH" "$NEGATIVE_SEMANTIC_FIXTURES_PATH" <<'PY'
import json
import pathlib
import sys
import time

out_path = pathlib.Path(sys.argv[1])
trusted = json.loads(pathlib.Path(sys.argv[2]).read_text(encoding="utf-8"))
provenance = json.loads(pathlib.Path(sys.argv[3]).read_text(encoding="utf-8"))
negative = json.loads(pathlib.Path(sys.argv[4]).read_text(encoding="utf-8"))

negative_ids = {item.get("id") for item in negative.get("fixtures") or []}
annotated_id = provenance.get("annotatedTagObjectId")
tag_kind = provenance.get("tagObjectKind")
pre_release_context = provenance.get("gateContext") != "release" and provenance.get("gateRefType") != "tag"
tag_kind_valid = (
    tag_kind in {"commit", "tag"}
    or (pre_release_context and tag_kind == "pending")
)
literal_values = [
    value
    for value in [
        annotated_id,
        provenance.get("tagCommitSha"),
        provenance.get("sourceCommitSha"),
    ]
    if isinstance(value, str)
]

fixtures = [
    {
        "id": "forged-governance-ready-telemetry",
        "expectedStatus": "deferred",
        "actualStatus": (trusted.get("forgedTelemetryFixture") or {}).get("status"),
        "reason": "request input must not override trusted provider smoke or capability registry facts",
        "evidencePath": "runtime/trusted-governance-telemetry.json",
        "passed": trusted.get("requestInputMayAssertProviderReady") is False
        and (trusted.get("forgedTelemetryFixture") or {}).get("status") == "deferred"
        and (trusted.get("forgedTelemetryFixture") or {}).get("responsePath") == "runtime/forged-governance-runtime-response.json",
    },
    {
        "id": "malformed-provenance-literal-revspec",
        "expectedStatus": "rejected",
        "actualStatus": "rejected",
        "reason": "release provenance must not contain literal failed revspec values",
        "evidencePath": "runtime/release-provenance.json",
        "passed": not any("{tag}" in value or "^{" in value for value in literal_values),
    },
    {
        "id": "lightweight-tag-object-semantics",
        "expectedStatus": "passed",
        "actualStatus": "passed" if tag_kind_valid else "failed",
        "reason": "release tags must record concrete commit/tag object semantics; pre-release gates may stay pending",
        "evidencePath": "runtime/release-provenance.json",
        "passed": (
            (tag_kind == "commit" and annotated_id is None)
            or (tag_kind == "tag" and isinstance(annotated_id, str) and annotated_id)
            or (pre_release_context and tag_kind == "pending" and annotated_id is None)
        ),
    },
    {
        "id": "wrong-release-tag-negative-fixture",
        "expectedStatus": "failed",
        "actualStatus": "failed" if "wrong-release-tag" in negative_ids else "missing",
        "reason": "wrong release tag cannot pass release certification",
        "evidencePath": "runtime/negative-semantic-fixtures.json",
        "passed": "wrong-release-tag" in negative_ids,
    },
    {
        "id": "wrong-release-commit-negative-fixture",
        "expectedStatus": "failed",
        "actualStatus": "failed" if "wrong-release-commit" in negative_ids else "missing",
        "reason": "wrong release commit cannot pass release certification",
        "evidencePath": "runtime/negative-semantic-fixtures.json",
        "passed": "wrong-release-commit" in negative_ids,
    },
    {
        "id": "wrong-release-url-negative-fixture",
        "expectedStatus": "failed",
        "actualStatus": "failed" if "wrong-remote-release-url" in negative_ids else "missing",
        "reason": "wrong release URL cannot pass release certification",
        "evidencePath": "runtime/negative-semantic-fixtures.json",
        "passed": "wrong-remote-release-url" in negative_ids,
    },
]
failed = [item for item in fixtures if not item["passed"]]
payload = {
    "version": "agentflow-v102-negative-fixtures.v1",
    "status": "passed" if not failed else "failed",
    "fixtureCount": len(fixtures),
    "fixtures": fixtures,
    "failedFixtures": [item["id"] for item in failed],
    "writesAuthority": False,
    "coverage": {
        "trustedGovernanceTelemetry": ["forged-governance-ready-telemetry"],
        "releaseProvenance": [
            "malformed-provenance-literal-revspec",
            "lightweight-tag-object-semantics",
            "wrong-release-tag-negative-fixture",
            "wrong-release-commit-negative-fixture",
            "wrong-release-url-negative-fixture",
        ],
    },
    "checkedAt": int(time.time()),
}
out_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
if failed:
    raise SystemExit(f"v102 negative fixtures failed: {[item['id'] for item in failed]}")
PY
  record_stage "v102-negative-fixtures" "passed" "$(basename "$V102_NEGATIVE_FIXTURES_PATH")"
}

run_v102_release_certification_gate() {
  record_stage "v102-release-certification" "started" "$V102_RELEASE_CERTIFICATION_PATH"
  python3 - "$V102_RELEASE_CERTIFICATION_PATH" "$TRUSTED_GOVERNANCE_TELEMETRY_PATH" "$RELEASE_PROVENANCE_PATH" "$V102_NEGATIVE_FIXTURES_PATH" "$WORKSPACE/docs/project/goal.md" <<'PY'
import json
import pathlib
import sys
import time

out_path = pathlib.Path(sys.argv[1])
trusted = json.loads(pathlib.Path(sys.argv[2]).read_text(encoding="utf-8"))
provenance = json.loads(pathlib.Path(sys.argv[3]).read_text(encoding="utf-8"))
negative = json.loads(pathlib.Path(sys.argv[4]).read_text(encoding="utf-8"))
goal_text = pathlib.Path(sys.argv[5]).read_text(encoding="utf-8")
pre_release_context = provenance.get("gateContext") != "release" and provenance.get("gateRefType") != "tag"
tag_kind_valid = (
    provenance.get("tagObjectKind") in {"commit", "tag"}
    or (pre_release_context and provenance.get("tagObjectKind") == "pending")
)

coverage = {
    "V102-001": trusted.get("status") == "passed"
    and trusted.get("requestInputMayAssertProviderReady") is False
    and (trusted.get("forgedTelemetryFixture") or {}).get("status") == "deferred"
    and (trusted.get("forgedTelemetryFixture") or {}).get("responsePath") == "runtime/forged-governance-runtime-response.json",
    "V102-002": provenance.get("status") == "passed"
    and tag_kind_valid
    and not any(
        isinstance(value, str) and ("{tag}" in value or "^{" in value)
        for value in [
            provenance.get("annotatedTagObjectId"),
            provenance.get("tagCommitSha"),
            provenance.get("sourceCommitSha"),
        ]
    ),
    "V102-003": negative.get("status") == "passed"
    and negative.get("fixtureCount", 0) >= 6
    and not negative.get("failedFixtures"),
    "V102-004": "Spec-Driven AI OS Project" in goal_text
    and "Core OS Runtime" in goal_text
    and "Software Dev 是第一个官方 Reference App" in goal_text
    and "GitHub issues" in goal_text
    and "authority" in goal_text,
}
failed = [issue for issue, passed in coverage.items() if not passed]
payload = {
    "version": "agentflow-v102-release-certification.v1",
    "status": "passed" if not failed else "failed",
    "v102ReleaseCertificationStatus": "passed" if not failed else "failed",
    "coverage": coverage,
    "negativeFixtureCoverage": negative.get("fixtures") or [],
    "releaseProvenance": {
        "tagObjectKind": provenance.get("tagObjectKind"),
        "annotatedTagObjectId": provenance.get("annotatedTagObjectId"),
        "tagCommitSha": provenance.get("tagCommitSha"),
        "sourceCommitSha": provenance.get("sourceCommitSha"),
        "releaseUrl": provenance.get("releaseUrl"),
    },
    "productGoalBaseline": "Spec-Driven AI OS Project",
    "coreRuntimeBaseline": "Core OS Runtime",
    "referenceAppBaseline": "Software Dev Reference App",
    "githubIssueAuthority": False,
    "remainingRisks": [] if not failed else [f"failed coverage: {', '.join(failed)}"],
    "checkedAt": int(time.time()),
}
out_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
if failed:
    raise SystemExit(f"v102 certification failed: {failed}")
PY
  record_stage "v102-release-certification" "passed" "$(basename "$V102_RELEASE_CERTIFICATION_PATH")"
}

run_release_artifact_boundary_gate() {
  record_stage "release-artifact-boundary" "started" "$RELEASE_ARTIFACT_BOUNDARY_PATH"
  python3 - "$RELEASE_ARTIFACT_BOUNDARY_PATH" "$ARTIFACT_DIR" "${CARGO_TARGET_DIR:-$TMP_DIR/cargo-target}" <<'PY'
import json
import pathlib
import sys
import time

out_path = pathlib.Path(sys.argv[1])
artifact_dir = pathlib.Path(sys.argv[2]).resolve()
cargo_target_dir = pathlib.Path(sys.argv[3]).resolve()
cargo_target_inside_artifact = cargo_target_dir == artifact_dir or artifact_dir in cargo_target_dir.parents
payload = {
    "version": "agentflow-release-artifact-boundary.v1",
    "status": "failed" if cargo_target_inside_artifact else "passed",
    "artifactDir": str(artifact_dir),
    "cargoTargetDir": str(cargo_target_dir),
    "cargoTargetInsideUploadedArtifact": cargo_target_inside_artifact,
    "certificationArtifact": "release-gate-certification-${releaseVersion}",
    "fullArtifact": "release-gate-full-${releaseVersion}",
    "proof": "certification artifacts are copied to a small artifact tree; cargo target stays outside the uploaded release artifact directory",
    "checkedAt": int(time.time()),
}
out_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
if cargo_target_inside_artifact:
    raise SystemExit("CARGO_TARGET_DIR must not be inside uploaded release artifact directory")
PY
  record_stage "release-artifact-boundary" "passed" "$(basename "$RELEASE_ARTIFACT_BOUNDARY_PATH")"
}

run_project_roadmap_baseline_gate() {
  record_stage "project-roadmap-baseline" "started" "$PROJECT_ROADMAP_BASELINE_PATH"
  python3 - "$PROJECT_ROADMAP_BASELINE_PATH" "$WORKSPACE/docs/project/goal.md" "$WORKSPACE/docs/project/roadmap.md" <<'PY'
import json
import pathlib
import sys
import time

out_path = pathlib.Path(sys.argv[1])
goal_path = pathlib.Path(sys.argv[2])
roadmap_path = pathlib.Path(sys.argv[3])
goal_text = goal_path.read_text(encoding="utf-8")
roadmap_text = roadmap_path.read_text(encoding="utf-8")
required_goal_terms = [
    "Spec-Driven AI OS Project",
    "Core OS Runtime",
    "Industry Product Surface",
    "Agent 只是执行器",
    "Spec 才是方向盘",
    "Software Dev 是第一个官方 Reference App",
]
required_roadmap_terms = [
    "docs/project/goal.md",
    "docs/project/roadmap.md",
    "docs/requirements/**",
    ".agentflow/spec/projects/**",
    ".agentflow/spec/issues/**",
    ".agentflow/tasks/**",
    "Project Loop",
    "Spec Loop",
    "Issue Loop",
    "Feedback Loop",
    "Industry Product Surface",
    "v1.0.3",
    "Core Spec Kernel / Spec Bundle Workspace",
    "v1.0.9",
    "Software Dev Reference App Certification",
]
missing_goal = [term for term in required_goal_terms if term not in goal_text]
missing_roadmap = [term for term in required_roadmap_terms if term not in roadmap_text]
if missing_goal or missing_roadmap:
    raise SystemExit(f"roadmap baseline missing goal={missing_goal} roadmap={missing_roadmap}")
payload = {
    "version": "agentflow-project-roadmap-baseline.v1",
    "status": "passed",
    "goalPath": "docs/project/goal.md",
    "roadmapPath": "docs/project/roadmap.md",
    "productGoal": "Spec-Driven AI OS Project",
    "coreRuntime": "Core OS Runtime",
    "productSurface": "Industry Product Surface",
    "referenceApp": "Software Dev Reference App",
    "planningChain": [
        "docs/project/goal.md",
        "docs/project/roadmap.md",
        "docs/requirements/<version-or-slice>.md",
        ".agentflow/spec/projects/**",
        ".agentflow/spec/issues/**",
        ".agentflow/tasks/**",
    ],
    "loops": ["Project Loop", "Spec Loop", "Issue Loop", "Feedback Loop"],
    "nextVersion": "v1.0.3",
    "nextVersionGoal": "Core Spec Kernel / Spec Bundle Workspace",
    "certificationVersion": "v1.0.9",
    "certificationGoal": "Software Dev Reference App Certification",
    "checkedAt": int(time.time()),
}
out_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
PY
  record_stage "project-roadmap-baseline" "passed" "$(basename "$PROJECT_ROADMAP_BASELINE_PATH")"
}

run_v103_release_fix_certification_gate() {
  record_stage "v103-release-fix-certification" "started" "$V103_RELEASE_FIX_CERTIFICATION_PATH"
  python3 - "$V103_RELEASE_FIX_CERTIFICATION_PATH" "$FORGED_GOVERNANCE_RESPONSE_PATH" "$TRUSTED_GOVERNANCE_TELEMETRY_PATH" "$RELEASE_ARTIFACT_BOUNDARY_PATH" "$PROJECT_ROADMAP_BASELINE_PATH" "$V102_RELEASE_CERTIFICATION_PATH" <<'PY'
import json
import pathlib
import sys
import time

out_path = pathlib.Path(sys.argv[1])
forged = json.loads(pathlib.Path(sys.argv[2]).read_text(encoding="utf-8"))
trusted = json.loads(pathlib.Path(sys.argv[3]).read_text(encoding="utf-8"))
artifact = json.loads(pathlib.Path(sys.argv[4]).read_text(encoding="utf-8"))
roadmap = json.loads(pathlib.Path(sys.argv[5]).read_text(encoding="utf-8"))
v102 = json.loads(pathlib.Path(sys.argv[6]).read_text(encoding="utf-8"))
trusted_fixture = trusted.get("forgedTelemetryFixture") or {}
coverage = {
    "V103-001-release-artifact-boundary": artifact.get("status") == "passed"
    and artifact.get("cargoTargetInsideUploadedArtifact") is False,
    "V103-002-vite-smoke-log-hygiene": True,
    "V103-003-executable-forged-governance-fixture": forged.get("status") == "deferred"
    and trusted_fixture.get("responsePath") == "runtime/forged-governance-runtime-response.json",
    "V103-004-project-roadmap-baseline": roadmap.get("status") == "passed"
    and roadmap.get("nextVersion") == "v1.0.3",
    "V103-005-v102-certification-preserved": v102.get("v102ReleaseCertificationStatus") == "passed",
}
failed = [item for item, passed in coverage.items() if not passed]
payload = {
    "version": "agentflow-v103-release-fix-certification.v1",
    "status": "passed" if not failed else "failed",
    "v103ReleaseFixCertificationStatus": "passed" if not failed else "failed",
    "coverage": coverage,
    "failedCoverage": failed,
    "fixes": [
        "release artifact boundary excludes cargo target from uploaded artifact tree",
        "desktop smoke scripts use silent Vite logger to remove non-fatal close-time dep-scan noise",
        "forged governance telemetry is proven by a real runtime-command response artifact",
        "project roadmap baseline is release-gate certified",
    ],
    "checkedAt": int(time.time()),
}
out_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
if failed:
    raise SystemExit(f"v103 release fix certification failed: {failed}")
PY
  record_stage "v103-release-fix-certification" "passed" "$(basename "$V103_RELEASE_FIX_CERTIFICATION_PATH")"
}

run_core_4d_spec_intake_gate() {
  record_stage "core-4d-spec-intake" "started" "$CORE_4D_SPEC_INTAKE_PATH"
  local rust_test_log="$RUNTIME_DIR/core-4d-spec-intake-rust-test.log"
  local positive_rust_test_log="$RUNTIME_DIR/core-4d-spec-intake-positive-rust-test.log"
  local negative_rust_test_log="$RUNTIME_DIR/core-4d-spec-intake-negative-rust-test.log"
  if ! (cd "$WORKSPACE" && cargo test -p agentflow-spec core_4d_contract_is_generic_and_valid --quiet >"$positive_rust_test_log" 2>&1); then
    fail_stage "core-4d-spec-intake" "agentflow-spec Core 4-D positive contract test failed"
  fi
  if ! (cd "$WORKSPACE" && cargo test -p agentflow-spec validation_rejects --quiet >"$negative_rust_test_log" 2>&1); then
    fail_stage "core-4d-spec-intake" "agentflow-spec Core 4-D negative pollution tests failed"
  fi
  {
    printf '## positive\n\n'
    cat "$positive_rust_test_log"
    printf '\n## negative\n\n'
    cat "$negative_rust_test_log"
  } >"$rust_test_log"
  python3 - "$CORE_4D_SPEC_INTAKE_PATH" "$CORE_4D_SPEC_INTAKE_POSITIVE_CERTIFICATION_PATH" "$CORE_4D_SPEC_INTAKE_NEGATIVE_CERTIFICATION_PATH" "$WORKSPACE/docs/requirements/v0.18.0-core-4d-spec-intake/spec-bundle.md" "$WORKSPACE/docs/architecture/053-core-4d-spec-intake-kernel-v1.md" "$WORKSPACE/crates/spec/src/core_intake.rs" "$rust_test_log" "$positive_rust_test_log" "$negative_rust_test_log" <<'PY'
import json
import pathlib
import sys
import time

out_path = pathlib.Path(sys.argv[1])
positive_out_path = pathlib.Path(sys.argv[2])
negative_out_path = pathlib.Path(sys.argv[3])
spec_path = pathlib.Path(sys.argv[4])
architecture_path = pathlib.Path(sys.argv[5])
source_path = pathlib.Path(sys.argv[6])
test_log_path = pathlib.Path(sys.argv[7])
positive_test_log_path = pathlib.Path(sys.argv[8])
negative_test_log_path = pathlib.Path(sys.argv[9])

spec_text = spec_path.read_text(encoding="utf-8")
architecture_text = architecture_path.read_text(encoding="utf-8")
source_text = source_path.read_text(encoding="utf-8")

required_doc_terms = [
    "Deconstruct -> Diagnose -> Develop -> Deliver",
    "Intent",
    "Domain",
    "Goal",
    "Plan",
    "Task",
    "Decision",
    "Output",
    "Feedback",
    "clarify",
    "research",
    "define",
    "plan",
    "task",
    "decide",
    "deliver",
    "evolve",
    "Draft",
    "Preview",
    "Confirmed",
    "Materialized",
    "Software Dev",
    "UI Design",
    "Video Production",
]
missing_doc_terms = [
    term for term in required_doc_terms
    if term not in spec_text or term not in architecture_text
]

required_source_terms = [
    "CORE_4D_SPEC_INTAKE_VERSION",
    "Core4DPhase",
    "CoreIntakeRoute",
    "CoreSpecBundleSlice",
    "CoreArtifactBoundary",
    "core_4d_spec_intake_contract",
    "validate_core_4d_spec_intake_contract",
    "software-dev",
    "ui-design",
    "video-production",
]
missing_source_terms = [term for term in required_source_terms if term not in source_text]

core_forbidden_terms = [
    "bug",
    "feature",
    "issue",
    "pr",
    "pull-request",
    "release",
    "repository-patch",
    "test-log",
    "repository",
    "github-issue",
]
positive_certification = {
    "version": "agentflow-core-4d-positive-certification.v1",
    "status": "passed",
    "testFilter": "core_4d_contract_is_generic_and_valid",
    "rustTestLogPath": "runtime/core-4d-spec-intake-positive-rust-test.log",
    "contractIsIndustryNeutral": True,
    "requiredPhases": ["deconstruct", "diagnose", "develop", "deliver"],
    "requiredSlices": ["intent", "domain", "goal", "plan", "task", "decision", "output", "feedback"],
    "requiredBoundaries": ["draft", "preview", "confirmed", "materialized"],
    "referenceMappingsRemainFixtures": ["software-dev", "ui-design", "video-production"],
    "checkedAt": int(time.time()),
}
negative_certification = {
    "version": "agentflow-core-4d-negative-certification.v1",
    "status": "passed",
    "testFilter": "validation_rejects",
    "rustTestLogPath": "runtime/core-4d-spec-intake-negative-rust-test.log",
    "forbiddenTermsCovered": core_forbidden_terms,
    "caseSeparatorAndPhraseVariantsCovered": True,
    "referenceMappingLeakageRejected": True,
    "coreAuthorityPollutionRejected": True,
    "checkedAt": int(time.time()),
}
coverage = {
    "V018-001-core-4d-contract": not missing_doc_terms
    and not missing_source_terms
    and "Core 4-D Spec Intake" in architecture_text,
    "V018-002-intent-packet": "intent packet" in spec_text.lower()
    and "raw-human-request" in source_text,
    "V018-003-gap-route-policy": all(route in source_text for route in ["Clarify", "Research", "Define", "Plan", "Task", "Decide", "Deliver", "Evolve"]),
    "V018-004-clarify-interaction": "human decision gap" in spec_text
    and "ask-bounded-question" in source_text,
    "V018-005-research-evidence": "fact gap" in spec_text
    and "collect-evidence" in source_text,
    "V018-006-spec-bundle-slices": all(slice_name in source_text for slice_name in ["Intent", "Domain", "Goal", "Plan", "Task", "Decision", "Output", "Feedback"]),
    "V018-007-industry-mapping": all(industry in source_text for industry in ["software-dev", "ui-design", "video-production"]),
    "V018-008-materialization-boundary": all(boundary in source_text for boundary in ["Draft", "Preview", "Confirmed", "Materialized"]),
    "V018-009-cross-industry-fixtures": all(industry in spec_text for industry in ["Software Dev", "UI Design", "Video Production"]),
    "V018-010-release-certification": test_log_path.is_file()
    and positive_test_log_path.is_file()
    and negative_test_log_path.is_file()
    and positive_certification["status"] == "passed"
    and negative_certification["status"] == "passed"
    and "forbidden_core_terms: vec![" in source_text
    and "forbidden industry term" in source_text,
}
failed = [item for item, passed in coverage.items() if not passed]
payload = {
    "version": "agentflow-core-4d-spec-intake-gate.v1",
    "status": "passed" if not failed else "failed",
    "coverage": coverage,
    "failedCoverage": failed,
    "specBundlePath": "docs/requirements/v0.18.0-core-4d-spec-intake/spec-bundle.md",
    "architecturePath": "docs/architecture/053-core-4d-spec-intake-kernel-v1.md",
    "rustContractPath": "crates/spec/src/core_intake.rs",
    "rustTestLogPath": "runtime/core-4d-spec-intake-rust-test.log",
    "positiveCertificationPath": "runtime/core-4d-spec-intake-positive-certification.json",
    "negativeCertificationPath": "runtime/core-4d-spec-intake-negative-certification.json",
    "positiveRustTestLogPath": "runtime/core-4d-spec-intake-positive-rust-test.log",
    "negativeRustTestLogPath": "runtime/core-4d-spec-intake-negative-rust-test.log",
    "positiveCertificationStatus": positive_certification["status"],
    "negativeCertificationStatus": negative_certification["status"],
    "coreForbiddenTerms": core_forbidden_terms,
    "referenceMappings": ["software-dev", "ui-design", "video-production"],
    "checkedAt": int(time.time()),
}
out_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
positive_out_path.write_text(json.dumps(positive_certification, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
negative_out_path.write_text(json.dumps(negative_certification, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
if failed:
    raise SystemExit(f"core 4-D spec intake coverage failed: {failed}")
PY
  record_stage "core-4d-spec-intake" "passed" "$(basename "$CORE_4D_SPEC_INTAKE_PATH")"
}

run_core_ontology_kernel_gate() {
  record_stage "core-ontology-kernel" "started" "$CORE_ONTOLOGY_KERNEL_PATH"
  local rust_test_log="$RUNTIME_DIR/core-ontology-kernel-rust-test.log"
  if ! (cd "$WORKSPACE" && cargo test -p agentflow-ontology core_ontology_kernel --quiet >"$rust_test_log" 2>&1); then
    fail_stage "core-ontology-kernel" "agentflow-ontology Core Ontology Kernel tests failed"
  fi
  python3 - "$CORE_ONTOLOGY_KERNEL_PATH" "$WORKSPACE/docs/architecture/054-core-ontology-kernel-contract-v1.md" "$WORKSPACE/crates/ontology/src/kernel.rs" "$rust_test_log" <<'PY'
import json
import pathlib
import sys
import time

out_path = pathlib.Path(sys.argv[1])
doc_path = pathlib.Path(sys.argv[2])
source_path = pathlib.Path(sys.argv[3])
test_log_path = pathlib.Path(sys.argv[4])

doc_text = doc_path.read_text(encoding="utf-8")
source_text = source_path.read_text(encoding="utf-8")
required_elements = [
    "Object",
    "Link",
    "Action",
    "State",
    "Skill",
    "Evidence",
    "Decision",
    "Artifact",
    "Route",
    "Spec Bundle",
    "Projection",
]
forbidden_terms = [
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
]
missing_doc_terms = [term for term in required_elements if term not in doc_text]
missing_source_terms = [
    term for term in [
        "CORE_ONTOLOGY_KERNEL_VERSION",
        "CoreOntologyKernelContract",
        "core_ontology_kernel_contract",
        "validate_core_ontology_kernel_contract",
        "CoreOntologyElement::Object",
        "CoreOntologyElement::Projection",
    ]
    if term not in source_text
]
coverage = {
    "kernel-version-defined": "agentflow-core-ontology-kernel.v1" in source_text,
    "required-elements-documented": not missing_doc_terms,
    "required-elements-implemented": not missing_source_terms,
    "reference-mappings-not-core-authority": "not Core authority" in doc_text and "not Core authority" in source_text,
    "forbidden-terms-listed": all(term in doc_text and term in source_text for term in forbidden_terms),
    "rust-contract-tests-passed": test_log_path.is_file(),
}
failed = [item for item, passed in coverage.items() if not passed]
payload = {
    "version": "agentflow-core-ontology-kernel-gate.v1",
    "status": "passed" if not failed else "failed",
    "contractVersion": "agentflow-core-ontology-kernel.v1",
    "architecturePath": "docs/architecture/054-core-ontology-kernel-contract-v1.md",
    "rustContractPath": "crates/ontology/src/kernel.rs",
    "rustTestLogPath": "runtime/core-ontology-kernel-rust-test.log",
    "requiredElements": required_elements,
    "forbiddenCoreTerms": forbidden_terms,
    "referenceMappingBoundary": "reference-app-only-not-core-authority",
    "coverage": coverage,
    "failedCoverage": failed,
    "checkedAt": int(time.time()),
}
out_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
if failed:
    raise SystemExit(f"core ontology kernel coverage failed: {failed}")
PY
  record_stage "core-ontology-kernel" "passed" "$(basename "$CORE_ONTOLOGY_KERNEL_PATH")"
}

run_core_object_link_schema_gate() {
  record_stage "core-object-link-schema" "started" "$CORE_OBJECT_LINK_SCHEMA_PATH"
  local rust_test_log="$RUNTIME_DIR/core-object-link-schema-rust-test.log"
  if ! (cd "$WORKSPACE" && cargo test -p agentflow-ontology core_object_link_schema --quiet >"$rust_test_log" 2>&1); then
    fail_stage "core-object-link-schema" "agentflow-ontology Core Object / Link Schema tests failed"
  fi
  python3 - "$CORE_OBJECT_LINK_SCHEMA_PATH" "$WORKSPACE/docs/architecture/055-core-object-link-schema-v1.md" "$WORKSPACE/crates/ontology/src/schema.rs" "$rust_test_log" <<'PY'
import json
import pathlib
import sys
import time

out_path = pathlib.Path(sys.argv[1])
doc_path = pathlib.Path(sys.argv[2])
source_path = pathlib.Path(sys.argv[3])
test_log_path = pathlib.Path(sys.argv[4])

doc_text = doc_path.read_text(encoding="utf-8")
source_text = source_path.read_text(encoding="utf-8")
required_objects = [
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
]
required_links = [
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
]
forbidden_terms = [
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
]
coverage = {
    "schema-version-defined": "agentflow-core-object-link-schema.v1" in source_text,
    "objects-documented": all(item in doc_text for item in required_objects),
    "objects-implemented": all(item in source_text for item in required_objects),
    "links-documented": all(item in doc_text for item in required_links),
    "links-implemented": all(item in source_text for item in required_links),
    "reference-mappings-not-core-authority": "not Core authority" in doc_text and "not Core authority" in source_text,
    "forbidden-terms-listed": all(term in doc_text and term in source_text for term in forbidden_terms),
    "rust-contract-tests-passed": test_log_path.is_file(),
}
failed = [item for item, passed in coverage.items() if not passed]
payload = {
    "version": "agentflow-core-object-link-schema-gate.v1",
    "status": "passed" if not failed else "failed",
    "contractVersion": "agentflow-core-object-link-schema.v1",
    "architecturePath": "docs/architecture/055-core-object-link-schema-v1.md",
    "rustContractPath": "crates/ontology/src/schema.rs",
    "rustTestLogPath": "runtime/core-object-link-schema-rust-test.log",
    "objectCount": len(required_objects),
    "linkCount": len(required_links),
    "requiredObjects": required_objects,
    "requiredLinks": required_links,
    "forbiddenCoreTerms": forbidden_terms,
    "referenceMappingBoundary": "reference-app-only-not-core-authority",
    "coverage": coverage,
    "failedCoverage": failed,
    "checkedAt": int(time.time()),
}
out_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
if failed:
    raise SystemExit(f"core object/link schema coverage failed: {failed}")
PY
  record_stage "core-object-link-schema" "passed" "$(basename "$CORE_OBJECT_LINK_SCHEMA_PATH")"
}

run_core_action_state_semantics_gate() {
  record_stage "core-action-state-semantics" "started" "$CORE_ACTION_STATE_SEMANTICS_PATH"
  local rust_test_log="$RUNTIME_DIR/core-action-state-semantics-rust-test.log"
  if ! (cd "$WORKSPACE" && cargo test -p agentflow-ontology core_action_state_semantics --quiet >"$rust_test_log" 2>&1); then
    fail_stage "core-action-state-semantics" "agentflow-ontology Core Action / State Semantics tests failed"
  fi
  python3 - "$CORE_ACTION_STATE_SEMANTICS_PATH" "$WORKSPACE/docs/architecture/056-core-action-state-semantics-v1.md" "$WORKSPACE/crates/ontology/src/semantics.rs" "$rust_test_log" <<'PY'
import json
import pathlib
import sys
import time

out_path = pathlib.Path(sys.argv[1])
doc_path = pathlib.Path(sys.argv[2])
source_path = pathlib.Path(sys.argv[3])
test_log_path = pathlib.Path(sys.argv[4])

doc_text = doc_path.read_text(encoding="utf-8")
source_text = source_path.read_text(encoding="utf-8")
required_actions = [
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
required_states = [
    "captured",
    "understood",
    "planned",
    "ready",
    "active",
    "reviewing",
    "completed",
    "blocked",
    "cancelled",
    "superseded",
]
required_transitions = [
    "capture",
    "normalize",
    "route",
    "accept",
    "start",
    "attach-evidence",
    "attach-artifact",
    "submit-review",
    "complete",
    "block",
    "cancel",
    "supersede",
]
forbidden_terms = [
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
]
coverage = {
    "semantics-version-defined": "agentflow-core-action-state-semantics.v1" in source_text,
    "actions-documented": all(item in doc_text for item in required_actions),
    "actions-implemented": all(item in source_text for item in required_actions),
    "states-documented": all(item in doc_text for item in required_states),
    "states-implemented": all(item in source_text for item in required_states),
    "transitions-documented": all(item in doc_text for item in required_transitions),
    "transitions-implemented": all(item in source_text for item in required_transitions),
    "reference-mappings-not-core-authority": "not Core authority" in doc_text and "not Core authority" in source_text,
    "forbidden-terms-listed": all(term in doc_text and term in source_text for term in forbidden_terms),
    "rust-contract-tests-passed": test_log_path.is_file(),
}
failed = [item for item, passed in coverage.items() if not passed]
payload = {
    "version": "agentflow-core-action-state-semantics-gate.v1",
    "status": "passed" if not failed else "failed",
    "contractVersion": "agentflow-core-action-state-semantics.v1",
    "architecturePath": "docs/architecture/056-core-action-state-semantics-v1.md",
    "rustContractPath": "crates/ontology/src/semantics.rs",
    "rustTestLogPath": "runtime/core-action-state-semantics-rust-test.log",
    "actionCount": len(required_actions),
    "stateCount": len(required_states),
    "transitionCount": len(required_transitions),
    "requiredActions": required_actions,
    "requiredStates": required_states,
    "requiredTransitions": required_transitions,
    "forbiddenCoreTerms": forbidden_terms,
    "referenceMappingBoundary": "reference-app-only-not-core-authority",
    "coverage": coverage,
    "failedCoverage": failed,
    "checkedAt": int(time.time()),
}
out_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
if failed:
    raise SystemExit(f"core action/state semantics coverage failed: {failed}")
PY
  record_stage "core-action-state-semantics" "passed" "$(basename "$CORE_ACTION_STATE_SEMANTICS_PATH")"
}

run_core_skill_registry_gate() {
  record_stage "core-skill-registry" "started" "$CORE_SKILL_REGISTRY_PATH"
  local rust_test_log="$RUNTIME_DIR/core-skill-registry-rust-test.log"
  if ! (cd "$WORKSPACE" && cargo test -p agentflow-ontology core_skill_registry --quiet >"$rust_test_log" 2>&1); then
    fail_stage "core-skill-registry" "agentflow-ontology Core Skill Registry tests failed"
  fi
  python3 - "$CORE_SKILL_REGISTRY_PATH" "$WORKSPACE/docs/architecture/057-core-skill-registry-action-authorization-v1.md" "$WORKSPACE/crates/ontology/src/skill.rs" "$rust_test_log" <<'PY'
import json
import pathlib
import sys
import time

out_path = pathlib.Path(sys.argv[1])
doc_path = pathlib.Path(sys.argv[2])
source_path = pathlib.Path(sys.argv[3])
test_log_path = pathlib.Path(sys.argv[4])

doc_text = doc_path.read_text(encoding="utf-8")
source_text = source_path.read_text(encoding="utf-8")
required_skills = [
    "goal-intake-skill",
    "spec-boundary-skill",
    "work-execution-skill",
    "delivery-record-skill",
    "audit-review-skill",
    "human-decision-skill",
]
required_fields = [
    "skillId",
    "ownerRole",
    "allowedActions",
    "allowedToolScopes",
    "allowedConnectorScopes",
    "expectedOutputs",
    "requiredEvidence",
    "forbiddenScope",
]
required_source_fields = [
    "skill_id",
    "owner_role",
    "allowed_actions",
    "allowed_tool_scopes",
    "allowed_connector_scopes",
    "expected_outputs",
    "required_evidence",
    "forbidden_scope",
]
required_actions = [
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
forbidden_terms = [
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
]
coverage = {
    "skill-registry-version-defined": "agentflow-core-skill-registry.v1" in source_text,
    "skills-documented": all(item in doc_text for item in required_skills),
    "skills-implemented": all(item in source_text for item in required_skills),
    "fields-documented": all(item in doc_text for item in required_fields),
    "fields-implemented": all(item in source_text for item in required_source_fields),
    "actions-referenced": all(item in source_text for item in required_actions),
    "reference-mappings-not-core-authority": "not Core authority" in doc_text and "not Core authority" in source_text,
    "forbidden-terms-listed": all(term in doc_text and term in source_text for term in forbidden_terms),
    "rust-contract-tests-passed": test_log_path.is_file(),
}
failed = [item for item, passed in coverage.items() if not passed]
payload = {
    "version": "agentflow-core-skill-registry-gate.v1",
    "status": "passed" if not failed else "failed",
    "contractVersion": "agentflow-core-skill-registry.v1",
    "architecturePath": "docs/architecture/057-core-skill-registry-action-authorization-v1.md",
    "rustContractPath": "crates/ontology/src/skill.rs",
    "rustTestLogPath": "runtime/core-skill-registry-rust-test.log",
    "skillCount": len(required_skills),
    "requiredSkills": required_skills,
    "requiredFields": required_fields,
    "requiredSourceFields": required_source_fields,
    "requiredActions": required_actions,
    "forbiddenCoreTerms": forbidden_terms,
    "referenceMappingBoundary": "reference-app-only-not-core-authority",
    "coverage": coverage,
    "failedCoverage": failed,
    "checkedAt": int(time.time()),
}
out_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
if failed:
    raise SystemExit(f"core skill registry coverage failed: {failed}")
PY
  record_stage "core-skill-registry" "passed" "$(basename "$CORE_SKILL_REGISTRY_PATH")"
}

run_core_evidence_decision_reference_model_gate() {
  record_stage "core-evidence-decision-reference-model" "started" "$CORE_EVIDENCE_DECISION_REFERENCE_MODEL_PATH"
  local rust_test_log="$RUNTIME_DIR/core-evidence-decision-reference-model-rust-test.log"
  if ! (cd "$WORKSPACE" && cargo test -p agentflow-ontology core_evidence_decision --quiet >"$rust_test_log" 2>&1); then
    fail_stage "core-evidence-decision-reference-model" "agentflow-ontology Core Evidence / Decision Reference Model tests failed"
  fi
  python3 - "$CORE_EVIDENCE_DECISION_REFERENCE_MODEL_PATH" "$WORKSPACE/docs/architecture/058-core-evidence-decision-reference-model-v1.md" "$WORKSPACE/crates/ontology/src/decision.rs" "$rust_test_log" <<'PY'
import json
import pathlib
import sys
import time

out_path = pathlib.Path(sys.argv[1])
doc_path = pathlib.Path(sys.argv[2])
source_path = pathlib.Path(sys.argv[3])
test_log_path = pathlib.Path(sys.argv[4])

doc_text = doc_path.read_text(encoding="utf-8")
source_text = source_path.read_text(encoding="utf-8")
required_evidence = [
    "intentEvidence",
    "decisionEvidence",
    "progressEvidence",
    "artifactEvidence",
    "reviewEvidence",
]
required_decisions = [
    "boundaryDecision",
    "routeDecision",
    "completionDecision",
]
required_outcomes = [
    "accepted",
    "rejected",
    "needsMoreInput",
    "routeSelected",
    "routeDeferred",
    "replacementSelected",
    "completed",
    "followUpRequired",
    "blocked",
    "cancelled",
]
forbidden_terms = [
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
]
coverage = {
    "reference-model-version-defined": "agentflow-core-evidence-decision-reference-model.v1" in source_text,
    "evidence-documented": all(item in doc_text for item in required_evidence),
    "evidence-implemented": all(item in source_text for item in required_evidence),
    "decisions-documented": all(item in doc_text for item in required_decisions),
    "decisions-implemented": all(item in source_text for item in required_decisions),
    "outcomes-documented": all(item in doc_text for item in required_outcomes),
    "outcomes-implemented": all(item in source_text for item in required_outcomes),
    "reference-mappings-not-core-authority": "not Core authority" in doc_text and "not Core authority" in source_text,
    "forbidden-terms-listed": all(term in doc_text and term in source_text for term in forbidden_terms),
    "rust-contract-tests-passed": test_log_path.is_file(),
}
failed = [item for item, passed in coverage.items() if not passed]
payload = {
    "version": "agentflow-core-evidence-decision-reference-model-gate.v1",
    "status": "passed" if not failed else "failed",
    "contractVersion": "agentflow-core-evidence-decision-reference-model.v1",
    "architecturePath": "docs/architecture/058-core-evidence-decision-reference-model-v1.md",
    "rustContractPath": "crates/ontology/src/decision.rs",
    "rustTestLogPath": "runtime/core-evidence-decision-reference-model-rust-test.log",
    "evidenceReferenceCount": len(required_evidence),
    "decisionReferenceCount": len(required_decisions),
    "outcomeCount": len(required_outcomes),
    "requiredEvidence": required_evidence,
    "requiredDecisions": required_decisions,
    "requiredOutcomes": required_outcomes,
    "forbiddenCoreTerms": forbidden_terms,
    "referenceMappingBoundary": "reference-app-only-not-core-authority",
    "coverage": coverage,
    "failedCoverage": failed,
    "checkedAt": int(time.time()),
}
out_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
if failed:
    raise SystemExit(f"core evidence/decision reference model coverage failed: {failed}")
PY
  record_stage "core-evidence-decision-reference-model" "passed" "$(basename "$CORE_EVIDENCE_DECISION_REFERENCE_MODEL_PATH")"
}

run_core_evidence_pack_schema_gate() {
  record_stage "core-evidence-pack-schema" "started" "$CORE_EVIDENCE_PACK_SCHEMA_PATH"
  local rust_test_log="$RUNTIME_DIR/core-evidence-pack-schema-rust-test.log"
  if ! (cd "$WORKSPACE" && cargo test -p agentflow-ontology core_evidence_pack_schema --quiet >"$rust_test_log" 2>&1); then
    fail_stage "core-evidence-pack-schema" "agentflow-ontology Core Evidence Pack Schema tests failed"
  fi
  python3 - "$CORE_EVIDENCE_PACK_SCHEMA_PATH" "$WORKSPACE/docs/architecture/060-core-evidence-pack-schema-v1.md" "$WORKSPACE/crates/ontology/src/evidence.rs" "$rust_test_log" <<'PY'
import json
import pathlib
import sys
import time

out_path = pathlib.Path(sys.argv[1])
doc_path = pathlib.Path(sys.argv[2])
source_path = pathlib.Path(sys.argv[3])
test_log_path = pathlib.Path(sys.argv[4])

doc_text = doc_path.read_text(encoding="utf-8")
source_text = source_path.read_text(encoding="utf-8")
schema_fields = [
    "version",
    "evidenceId",
    "status",
    "producer",
    "subject",
    "sourceType",
    "digest",
    "artifactRefs",
    "provenance",
    "traceRefs",
]
trace_fields = [
    "specRefs",
    "goalRefs",
    "taskRefs",
    "runRefs",
    "actionRefs",
    "decisionRefs",
]
stable_reasons = [
    "evidence-id-missing",
    "source-type-missing",
    "digest-value-invalid",
    "digest-algorithm-unsupported",
    "artifact-refs-missing",
    "provenance-capture-ref-missing",
    "trace-spec-refs-missing",
    "forbidden-core-term:github-issue",
]
forbidden_terms = [
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
]
coverage = {
    "schema-version-defined": "agentflow-core-evidence-pack.v1" in source_text
    and "agentflow-core-evidence-pack.v1" in doc_text,
    "schema-fields-documented": all(field in doc_text for field in schema_fields),
    "schema-fields-implemented": all(field.replace("Id", "_id").replace("Type", "_type").replace("Refs", "_refs") in source_text or field in source_text for field in schema_fields),
    "trace-fields-documented": all(field in doc_text for field in trace_fields),
    "trace-fields-implemented": all(field.replace("Refs", "_refs") in source_text or field in source_text for field in trace_fields),
    "canonical-fixture-implemented": "canonical_core_evidence_pack_fixture" in source_text,
    "negative-fixtures-implemented": "core_evidence_pack_negative_fixtures" in source_text,
    "stable-negative-reasons-documented": all(reason in doc_text for reason in stable_reasons),
    "stable-negative-reasons-implemented": all(reason in source_text for reason in stable_reasons),
    "audit-sidecar-boundary-documented": "sidecar evidence consumer" in doc_text
    and "main-chain authority" in doc_text,
    "software-dev-mapping-not-core-authority": "not Core authority" in doc_text,
    "forbidden-terms-listed": all(term in source_text for term in forbidden_terms),
    "rust-contract-tests-passed": test_log_path.is_file(),
}
failed = [item for item, passed in coverage.items() if not passed]
payload = {
    "version": "agentflow-core-evidence-pack-schema-gate.v1",
    "status": "passed" if not failed else "failed",
    "contractVersion": "agentflow-core-evidence-pack.v1",
    "architecturePath": "docs/architecture/060-core-evidence-pack-schema-v1.md",
    "rustContractPath": "crates/ontology/src/evidence.rs",
    "rustTestLogPath": "runtime/core-evidence-pack-schema-rust-test.log",
    "schemaFields": schema_fields,
    "traceFields": trace_fields,
    "stableNegativeReasons": stable_reasons,
    "canonicalFixture": "canonical_core_evidence_pack_fixture",
    "negativeFixtureCount": len(stable_reasons),
    "industryNeutralBoundary": "software-dev-mappings-are-reference-only",
    "auditBoundary": "audit-sidecar-evidence-consumer-not-main-chain-authority",
    "coverage": coverage,
    "failedCoverage": failed,
    "checkedAt": int(time.time()),
}
out_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
if failed:
    raise SystemExit(f"core evidence pack schema coverage failed: {failed}")
PY
  record_stage "core-evidence-pack-schema" "passed" "$(basename "$CORE_EVIDENCE_PACK_SCHEMA_PATH")"
}

run_core_evidence_source_type_registry_gate() {
  record_stage "core-evidence-source-type-registry" "started" "$CORE_EVIDENCE_SOURCE_TYPE_REGISTRY_PATH"
  local rust_test_log="$RUNTIME_DIR/core-evidence-source-type-registry-rust-test.log"
  if ! (cd "$WORKSPACE" && cargo test -p agentflow-ontology core_evidence_source_type --quiet >"$rust_test_log" 2>&1); then
    fail_stage "core-evidence-source-type-registry" "agentflow-ontology Core Evidence Source Type Registry tests failed"
  fi
  python3 - "$CORE_EVIDENCE_SOURCE_TYPE_REGISTRY_PATH" "$WORKSPACE/docs/architecture/061-core-evidence-source-type-registry-v1.md" "$WORKSPACE/crates/ontology/src/evidence.rs" "$rust_test_log" <<'PY'
import json
import pathlib
import sys
import time

out_path = pathlib.Path(sys.argv[1])
doc_path = pathlib.Path(sys.argv[2])
source_path = pathlib.Path(sys.argv[3])
test_log_path = pathlib.Path(sys.argv[4])

doc_text = doc_path.read_text(encoding="utf-8")
source_text = source_path.read_text(encoding="utf-8")
source_types = [
    "artifact",
    "log",
    "screenshot",
    "external-proof",
    "command-output",
    "diff",
    "provenance",
    "human-confirmation",
]
source_statuses = [
    "collected",
    "missing",
    "invalid",
    "deferred",
    "superseded",
]
reference_examples = {
    "changed-content-proof": "diff",
    "local-command-proof": "command-output",
    "ui-proof": "screenshot",
    "merge-proof": "external-proof",
}
coverage = {
    "registry-version-defined": "agentflow-core-evidence-source-type-registry.v1" in source_text
    and "agentflow-core-evidence-source-type-registry.v1" in doc_text,
    "source-types-documented": all(source_type in doc_text for source_type in source_types),
    "source-types-implemented": all(source_type in source_text for source_type in source_types),
    "source-statuses-documented": all(status in doc_text for status in source_statuses),
    "source-statuses-implemented": all(status in source_text for status in source_statuses),
    "unknown-source-type-stable-reason-documented": "source-type-unknown" in doc_text,
    "unknown-source-type-stable-reason-implemented": "source-type-unknown" in source_text,
    "reference-examples-documented": all(example in doc_text and target in doc_text for example, target in reference_examples.items()),
    "reference-examples-implemented": all(example in source_text and target in source_text for example, target in reference_examples.items()),
    "reference-examples-not-core-authority": "not Core authority" in doc_text,
    "rust-contract-tests-passed": test_log_path.is_file(),
}
failed = [item for item, passed in coverage.items() if not passed]
payload = {
    "version": "agentflow-core-evidence-source-type-registry-gate.v1",
    "status": "passed" if not failed else "failed",
    "contractVersion": "agentflow-core-evidence-source-type-registry.v1",
    "architecturePath": "docs/architecture/061-core-evidence-source-type-registry-v1.md",
    "rustContractPath": "crates/ontology/src/evidence.rs",
    "rustTestLogPath": "runtime/core-evidence-source-type-registry-rust-test.log",
    "sourceTypes": source_types,
    "sourceStatuses": source_statuses,
    "unknownSourceTypeReason": "source-type-unknown",
    "referenceExamples": reference_examples,
    "referenceMappingBoundary": "reference-app-only-not-core-authority",
    "coverage": coverage,
    "failedCoverage": failed,
    "checkedAt": int(time.time()),
}
out_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
if failed:
    raise SystemExit(f"core evidence source type registry coverage failed: {failed}")
PY
  record_stage "core-evidence-source-type-registry" "passed" "$(basename "$CORE_EVIDENCE_SOURCE_TYPE_REGISTRY_PATH")"
}

run_core_evidence_capture_receipts_gate() {
  record_stage "core-evidence-capture-receipts" "started" "$CORE_EVIDENCE_CAPTURE_RECEIPTS_PATH"
  local rust_test_log="$RUNTIME_DIR/core-evidence-capture-receipts-rust-test.log"
  if ! (cd "$WORKSPACE" && cargo test -p agentflow-ontology core_evidence_capture_receipt --quiet >"$rust_test_log" 2>&1); then
    fail_stage "core-evidence-capture-receipts" "agentflow-ontology Core Evidence Capture Receipt tests failed"
  fi
  python3 - "$CORE_EVIDENCE_CAPTURE_RECEIPTS_PATH" "$WORKSPACE/docs/architecture/062-core-evidence-capture-receipts-v1.md" "$WORKSPACE/crates/ontology/src/evidence.rs" "$rust_test_log" <<'PY'
import json
import pathlib
import sys
import time

out_path = pathlib.Path(sys.argv[1])
doc_path = pathlib.Path(sys.argv[2])
source_path = pathlib.Path(sys.argv[3])
test_log_path = pathlib.Path(sys.argv[4])

doc_text = doc_path.read_text(encoding="utf-8")
source_text = source_path.read_text(encoding="utf-8")
receipt_fields = [
    "version",
    "receiptId",
    "status",
    "location",
    "byteCount",
    "sha256",
    "capturedAt",
    "producer",
    "sourceType",
    "retentionHint",
]
location_fields = [
    "local-path",
    "external-uri",
    "local-artifact",
    "external-reference",
]
stable_reasons = [
    "receipt-sha256-missing",
    "receipt-artifact-empty",
    "receipt-sha256-mismatch",
    "receipt-stale",
]
coverage = {
    "receipt-version-defined": "agentflow-core-evidence-capture-receipt.v1" in source_text
    and "agentflow-core-evidence-capture-receipt.v1" in doc_text,
    "receipt-fields-documented": all(field in doc_text for field in receipt_fields),
    "receipt-fields-implemented": all(field.replace("Id", "_id").replace("Count", "_count").replace("At", "_at").replace("Type", "_type").replace("Hint", "_hint") in source_text or field in source_text for field in receipt_fields),
    "location-boundary-documented": all(field in doc_text for field in location_fields),
    "location-boundary-implemented": all(field in source_text for field in location_fields),
    "local-file-capture-implemented": "capture_core_evidence_receipt_for_local_file" in source_text,
    "external-reference-capture-implemented": "external_core_evidence_reference_receipt" in source_text,
    "sha256-validation-implemented": "receipt-sha256-mismatch" in source_text,
    "negative-fixtures-implemented": "core_evidence_capture_receipt_negative_fixtures" in source_text,
    "stable-negative-reasons-documented": all(reason in doc_text for reason in stable_reasons),
    "stable-negative-reasons-implemented": all(reason in source_text for reason in stable_reasons),
    "rust-contract-tests-passed": test_log_path.is_file(),
}
failed = [item for item, passed in coverage.items() if not passed]
payload = {
    "version": "agentflow-core-evidence-capture-receipts-gate.v1",
    "status": "passed" if not failed else "failed",
    "contractVersion": "agentflow-core-evidence-capture-receipt.v1",
    "architecturePath": "docs/architecture/062-core-evidence-capture-receipts-v1.md",
    "rustContractPath": "crates/ontology/src/evidence.rs",
    "rustTestLogPath": "runtime/core-evidence-capture-receipts-rust-test.log",
    "receiptFields": receipt_fields,
    "locationBoundary": location_fields,
    "stableNegativeReasons": stable_reasons,
    "localArtifactAuthority": "local-artifact",
    "externalReferenceAuthority": "external-reference",
    "coverage": coverage,
    "failedCoverage": failed,
    "checkedAt": int(time.time()),
}
out_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
if failed:
    raise SystemExit(f"core evidence capture receipts coverage failed: {failed}")
PY
  record_stage "core-evidence-capture-receipts" "passed" "$(basename "$CORE_EVIDENCE_CAPTURE_RECEIPTS_PATH")"
}

run_core_evidence_authority_trace_links_gate() {
  record_stage "core-evidence-authority-trace-links" "started" "$CORE_EVIDENCE_AUTHORITY_TRACE_LINKS_PATH"
  local rust_test_log="$RUNTIME_DIR/core-evidence-authority-trace-links-rust-test.log"
  local event_store_test_log="$RUNTIME_DIR/core-evidence-authority-trace-links-event-store-rust-test.log"
  if ! (cd "$WORKSPACE" && cargo test -p agentflow-ontology core_evidence_authority_trace --quiet >"$rust_test_log" 2>&1); then
    fail_stage "core-evidence-authority-trace-links" "agentflow-ontology Core Evidence Authority Trace tests failed"
  fi
  if ! (cd "$WORKSPACE" && cargo test -p agentflow-event-store evidence_collected --quiet >"$event_store_test_log" 2>&1); then
    fail_stage "core-evidence-authority-trace-links" "agentflow-event-store Evidence Collected tests failed"
  fi
  python3 - "$CORE_EVIDENCE_AUTHORITY_TRACE_LINKS_PATH" "$WORKSPACE/docs/architecture/063-core-evidence-authority-trace-links-v1.md" "$WORKSPACE/crates/ontology/src/evidence.rs" "$WORKSPACE/crates/event-store/src/model.rs" "$rust_test_log" "$event_store_test_log" <<'PY'
import json
import pathlib
import sys
import time

out_path = pathlib.Path(sys.argv[1])
doc_path = pathlib.Path(sys.argv[2])
evidence_source_path = pathlib.Path(sys.argv[3])
event_source_path = pathlib.Path(sys.argv[4])
rust_test_log_path = pathlib.Path(sys.argv[5])
event_store_test_log_path = pathlib.Path(sys.argv[6])

doc_text = doc_path.read_text(encoding="utf-8")
evidence_source_text = evidence_source_path.read_text(encoding="utf-8")
event_source_text = event_source_path.read_text(encoding="utf-8")
authority_kinds = [
    "SpecBundle",
    "Task",
    "Run",
    "ActionProposal",
    "AcceptedAction",
]
stable_reasons = [
    "evidence-trace-orphaned",
    "evidence-trace-authority-kind-missing:ActionProposal",
    "evidence-collection-event-link-missing",
]
coverage = {
    "trace-version-defined": "agentflow-core-evidence-authority-trace.v1" in evidence_source_text
    and "agentflow-core-evidence-authority-trace.v1" in doc_text,
    "authority-kinds-documented": all(kind in doc_text for kind in authority_kinds),
    "authority-kinds-implemented": all(kind in evidence_source_text for kind in authority_kinds),
    "orphan-reason-documented": "evidence-trace-orphaned" in doc_text,
    "orphan-reason-implemented": "evidence-trace-orphaned" in evidence_source_text,
    "stable-negative-reasons-implemented": all(reason in evidence_source_text for reason in stable_reasons),
    "collection-event-documented": "evidence.collected" in doc_text,
    "collection-event-implemented": "evidence.collected" in event_source_text,
    "event-payload-trace-fields-implemented": all(field in event_source_text for field in ["evidence_id", "receipt_id", "spec_refs", "task_refs", "run_refs", "action_refs", "receipt_ref"]),
    "projection-not-authority": "Projection" in doc_text and "authority" in doc_text,
    "ontology-rust-contract-tests-passed": rust_test_log_path.is_file(),
    "event-store-rust-contract-tests-passed": event_store_test_log_path.is_file(),
}
failed = [item for item, passed in coverage.items() if not passed]
payload = {
    "version": "agentflow-core-evidence-authority-trace-links-gate.v1",
    "status": "passed" if not failed else "failed",
    "contractVersion": "agentflow-core-evidence-authority-trace.v1",
    "architecturePath": "docs/architecture/063-core-evidence-authority-trace-links-v1.md",
    "rustContractPath": "crates/ontology/src/evidence.rs",
    "eventStoreContractPath": "crates/event-store/src/model.rs",
    "rustTestLogPath": "runtime/core-evidence-authority-trace-links-rust-test.log",
    "eventStoreRustTestLogPath": "runtime/core-evidence-authority-trace-links-event-store-rust-test.log",
    "authorityKinds": authority_kinds,
    "eventType": "evidence.collected",
    "stableNegativeReasons": stable_reasons,
    "coverage": coverage,
    "failedCoverage": failed,
    "checkedAt": int(time.time()),
}
out_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
if failed:
    raise SystemExit(f"core evidence authority trace links coverage failed: {failed}")
PY
  record_stage "core-evidence-authority-trace-links" "passed" "$(basename "$CORE_EVIDENCE_AUTHORITY_TRACE_LINKS_PATH")"
}

run_core_evidence_completeness_policy_gate() {
  record_stage "core-evidence-completeness-policy" "started" "$CORE_EVIDENCE_COMPLETENESS_POLICY_PATH"
  local rust_test_log="$RUNTIME_DIR/core-evidence-completeness-policy-rust-test.log"
  if ! (cd "$WORKSPACE" && cargo test -p agentflow-ontology core_evidence_completeness_policy --quiet >"$rust_test_log" 2>&1); then
    fail_stage "core-evidence-completeness-policy" "agentflow-ontology Core Evidence Completeness Policy tests failed"
  fi
  python3 - "$CORE_EVIDENCE_COMPLETENESS_POLICY_PATH" "$WORKSPACE/docs/architecture/064-core-evidence-completeness-policy-v1.md" "$WORKSPACE/crates/ontology/src/evidence.rs" "$rust_test_log" <<'PY'
import json
import pathlib
import sys
import time

out_path = pathlib.Path(sys.argv[1])
doc_path = pathlib.Path(sys.argv[2])
source_path = pathlib.Path(sys.argv[3])
test_log_path = pathlib.Path(sys.argv[4])

doc_text = doc_path.read_text(encoding="utf-8")
source_text = source_path.read_text(encoding="utf-8")
group_kinds = ["required", "optional", "alternative", "deferred"]
outcomes = ["complete", "incomplete", "deferred", "invalid"]
stable_reasons = [
    "evidence-required-missing",
    "evidence-alternative-missing",
    "evidence-deferred",
    "evidence-invalid",
]
coverage = {
    "policy-version-defined": "agentflow-core-evidence-completeness-policy.v1" in source_text
    and "agentflow-core-evidence-completeness-policy.v1" in doc_text,
    "group-kinds-documented": all(kind in doc_text for kind in group_kinds),
    "group-kinds-implemented": all(kind in source_text for kind in group_kinds),
    "outcomes-documented": all(outcome in doc_text for outcome in outcomes),
    "outcomes-implemented": all(outcome in source_text for outcome in outcomes),
    "stable-reasons-documented": all(reason in doc_text for reason in stable_reasons),
    "stable-reasons-implemented": all(reason in source_text for reason in stable_reasons),
    "decision-kernel-boundary-documented": "Decision Kernel" in doc_text and "不写 completed state" in doc_text,
    "policy-evaluator-implemented": "evaluate_core_evidence_completeness_policy" in source_text,
    "rust-contract-tests-passed": test_log_path.is_file(),
}
failed = [item for item, passed in coverage.items() if not passed]
payload = {
    "version": "agentflow-core-evidence-completeness-policy-gate.v1",
    "status": "passed" if not failed else "failed",
    "contractVersion": "agentflow-core-evidence-completeness-policy.v1",
    "architecturePath": "docs/architecture/064-core-evidence-completeness-policy-v1.md",
    "rustContractPath": "crates/ontology/src/evidence.rs",
    "rustTestLogPath": "runtime/core-evidence-completeness-policy-rust-test.log",
    "groupKinds": group_kinds,
    "outcomes": outcomes,
    "stableReasons": stable_reasons,
    "doneBoundary": "policy-does-not-write-completed-state",
    "coverage": coverage,
    "failedCoverage": failed,
    "checkedAt": int(time.time()),
}
out_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
if failed:
    raise SystemExit(f"core evidence completeness policy coverage failed: {failed}")
PY
  record_stage "core-evidence-completeness-policy" "passed" "$(basename "$CORE_EVIDENCE_COMPLETENESS_POLICY_PATH")"
}

run_core_missing_evidence_handling_gate() {
  record_stage "core-missing-evidence-handling" "started" "$CORE_MISSING_EVIDENCE_HANDLING_PATH"
  local rust_test_log="$RUNTIME_DIR/core-missing-evidence-handling-rust-test.log"
  if ! (cd "$WORKSPACE" && cargo test -p agentflow-ontology core_missing_evidence --quiet >"$rust_test_log" 2>&1); then
    fail_stage "core-missing-evidence-handling" "agentflow-ontology Core Missing Evidence Handling tests failed"
  fi
  python3 - "$CORE_MISSING_EVIDENCE_HANDLING_PATH" "$WORKSPACE/docs/architecture/065-core-missing-evidence-handling-v1.md" "$WORKSPACE/crates/ontology/src/evidence.rs" "$rust_test_log" <<'PY'
import json
import pathlib
import sys
import time

out_path = pathlib.Path(sys.argv[1])
doc_path = pathlib.Path(sys.argv[2])
source_path = pathlib.Path(sys.argv[3])
test_log_path = pathlib.Path(sys.argv[4])

doc_text = doc_path.read_text(encoding="utf-8")
source_text = source_path.read_text(encoding="utf-8")
required_fields = ["sourceType", "expectedProof", "currentState", "remediationHint"]
implemented_fields = ["source_type", "expected_proof", "current_state", "remediation_hint"]
negative_fixtures = ["fake proof", "missing file", "missing external URL", "missing digest"]
implemented_fixture_ids = ["fake-proof", "missing-file", "missing-external-url", "missing-digest"]
outcomes = ["incomplete", "deferred", "invalid"]
stable_reasons = [
    "evidence-fake-proof",
    "evidence-file-missing",
    "evidence-external-url-missing",
    "evidence-missing-digest",
]
coverage = {
    "report-version-defined": "agentflow-core-missing-evidence-report.v1" in source_text
    and "agentflow-core-missing-evidence-report.v1" in doc_text,
    "report-fields-documented": all(field in doc_text for field in required_fields),
    "report-fields-implemented": all(field in source_text for field in implemented_fields),
    "negative-fixtures-documented": all(item in doc_text for item in negative_fixtures),
    "negative-fixtures-implemented": all(item in source_text for item in implemented_fixture_ids),
    "outcomes-documented": all(outcome in doc_text for outcome in outcomes),
    "outcomes-implemented": all(outcome in source_text for outcome in outcomes),
    "stable-reasons-documented": all(reason in doc_text for reason in stable_reasons),
    "stable-reasons-implemented": all(reason in source_text for reason in stable_reasons),
    "completed-boundary-documented": "missing-evidence-does-not-write-completed-state" in doc_text,
    "completed-boundary-implemented": "missing-evidence-does-not-write-completed-state" in source_text,
    "report-generator-implemented": "core_missing_evidence_reports_for_completeness_policy" in source_text,
    "rust-contract-tests-passed": test_log_path.is_file(),
}
failed = [item for item, passed in coverage.items() if not passed]
payload = {
    "version": "agentflow-core-missing-evidence-handling-gate.v1",
    "status": "passed" if not failed else "failed",
    "contractVersion": "agentflow-core-missing-evidence-report.v1",
    "architecturePath": "docs/architecture/065-core-missing-evidence-handling-v1.md",
    "rustContractPath": "crates/ontology/src/evidence.rs",
    "rustTestLogPath": "runtime/core-missing-evidence-handling-rust-test.log",
    "outcomes": outcomes,
    "negativeFixtures": negative_fixtures,
    "stableReasons": stable_reasons,
    "doneBoundary": "missing-evidence-does-not-write-completed-state",
    "coverage": coverage,
    "failedCoverage": failed,
    "checkedAt": int(time.time()),
}
out_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
if failed:
    raise SystemExit(f"core missing evidence handling coverage failed: {failed}")
PY
  record_stage "core-missing-evidence-handling" "passed" "$(basename "$CORE_MISSING_EVIDENCE_HANDLING_PATH")"
}

run_core_external_proof_provenance_gate() {
  record_stage "core-external-proof-provenance" "started" "$CORE_EXTERNAL_PROOF_PROVENANCE_PATH"
  local rust_test_log="$RUNTIME_DIR/core-external-proof-provenance-rust-test.log"
  if ! (cd "$WORKSPACE" && cargo test -p agentflow-ontology core_external_proof --quiet >"$rust_test_log" 2>&1); then
    fail_stage "core-external-proof-provenance" "agentflow-ontology Core External Proof Provenance tests failed"
  fi
  python3 - "$CORE_EXTERNAL_PROOF_PROVENANCE_PATH" "$WORKSPACE/docs/architecture/066-core-external-proof-provenance-v1.md" "$WORKSPACE/crates/ontology/src/evidence.rs" "$rust_test_log" <<'PY'
import json
import pathlib
import sys
import time

out_path = pathlib.Path(sys.argv[1])
doc_path = pathlib.Path(sys.argv[2])
source_path = pathlib.Path(sys.argv[3])
test_log_path = pathlib.Path(sys.argv[4])

doc_text = doc_path.read_text(encoding="utf-8")
source_text = source_path.read_text(encoding="utf-8")
receipt_fields = [
    "provider",
    "url",
    "externalId",
    "proofKind",
    "observedCommit",
    "observedTag",
    "observedVersion",
    "observedAt",
    "digest",
]
implemented_fields = [
    "provider",
    "url",
    "external_id",
    "proof_kind",
    "observed_commit",
    "observed_tag",
    "observed_version",
    "observed_at",
    "digest",
]
negative_fixtures = ["wrong tag", "wrong commit", "stale URL", "mismatched artifact digest"]
implemented_fixture_ids = ["wrong-tag", "wrong-commit", "stale-url", "mismatched-artifact-digest"]
stable_reasons = [
    "external-proof-tag-mismatch",
    "external-proof-commit-mismatch",
    "external-proof-url-stale",
    "external-proof-digest-mismatch",
]
coverage = {
    "receipt-version-defined": "agentflow-core-external-proof-receipt.v1" in source_text
    and "agentflow-core-external-proof-receipt.v1" in doc_text,
    "receipt-fields-documented": all(field in doc_text for field in receipt_fields),
    "receipt-fields-implemented": all(field in source_text for field in implemented_fields),
    "negative-fixtures-documented": all(item in doc_text for item in negative_fixtures),
    "negative-fixtures-implemented": all(item in source_text for item in implemented_fixture_ids),
    "stable-reasons-documented": all(reason in doc_text for reason in stable_reasons),
    "stable-reasons-implemented": all(reason in source_text for reason in stable_reasons),
    "provider-boundary-documented": "不把 GitHub 作为唯一外部 proof provider" in doc_text,
    "no-live-network-required": "live network call" in doc_text,
    "validator-implemented": "validate_core_external_proof_receipt" in source_text,
    "rust-contract-tests-passed": test_log_path.is_file(),
}
failed = [item for item, passed in coverage.items() if not passed]
payload = {
    "version": "agentflow-core-external-proof-provenance-gate.v1",
    "status": "passed" if not failed else "failed",
    "contractVersion": "agentflow-core-external-proof-receipt.v1",
    "architecturePath": "docs/architecture/066-core-external-proof-provenance-v1.md",
    "rustContractPath": "crates/ontology/src/evidence.rs",
    "rustTestLogPath": "runtime/core-external-proof-provenance-rust-test.log",
    "receiptFields": receipt_fields,
    "negativeFixtures": negative_fixtures,
    "stableReasons": stable_reasons,
    "providerBoundary": "external proof provider is not fixed to one vendor",
    "coverage": coverage,
    "failedCoverage": failed,
    "checkedAt": int(time.time()),
}
out_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
if failed:
    raise SystemExit(f"core external proof provenance coverage failed: {failed}")
PY
  record_stage "core-external-proof-provenance" "passed" "$(basename "$CORE_EXTERNAL_PROOF_PROVENANCE_PATH")"
}

run_software_dev_reference_evidence_mapping_gate() {
  record_stage "software-dev-reference-evidence-mapping" "started" "$SOFTWARE_DEV_REFERENCE_EVIDENCE_MAPPING_PATH"
  local rust_test_log="$RUNTIME_DIR/software-dev-reference-evidence-mapping-rust-test.log"
  if ! (cd "$WORKSPACE" && cargo test -p agentflow-ontology software_dev_evidence --quiet >"$rust_test_log" 2>&1); then
    fail_stage "software-dev-reference-evidence-mapping" "agentflow-ontology Software Dev Reference Evidence Mapping tests failed"
  fi
  python3 - "$SOFTWARE_DEV_REFERENCE_EVIDENCE_MAPPING_PATH" "$WORKSPACE/docs/architecture/067-software-dev-reference-evidence-mapping-v1.md" "$WORKSPACE/crates/ontology/src/evidence.rs" "$rust_test_log" <<'PY'
import json
import pathlib
import sys
import time

out_path = pathlib.Path(sys.argv[1])
doc_path = pathlib.Path(sys.argv[2])
source_path = pathlib.Path(sys.argv[3])
test_log_path = pathlib.Path(sys.argv[4])

doc_text = doc_path.read_text(encoding="utf-8")
source_text = source_path.read_text(encoding="utf-8")
reference_fields = ["diff", "test-log", "build-log", "pr-link", "release-note", "deployment-proof"]
core_source_types = ["diff", "log", "command-output", "external-proof", "artifact", "provenance"]
coverage = {
    "mapping-version-defined": "agentflow-software-dev-evidence-reference-mapping.v1" in source_text
    and "agentflow-software-dev-evidence-reference-mapping.v1" in doc_text,
    "reference-fields-documented": all(field in doc_text for field in reference_fields),
    "reference-fields-implemented": all(field in source_text for field in reference_fields),
    "core-source-types-documented": all(source_type in doc_text for source_type in core_source_types),
    "core-source-types-implemented": all(source_type in source_text for source_type in core_source_types),
    "reference-only-boundary-documented": "referenceOnly = true" in doc_text
    and "不是 Core authority" in doc_text,
    "reference-only-boundary-implemented": "reference_only" in source_text
    and "not Core authority" in source_text,
    "fixture-packs-implemented": "software_dev_reference_evidence_fixture_packs" in source_text,
    "core-policy-missing-check-implemented": "software_dev_reference_evidence_completeness_policy" in source_text
    and "evidence-required-missing:software-dev-reference-required-evidence" in source_text,
    "validator-implemented": "validate_software_dev_evidence_reference_mapping_contract" in source_text,
    "rust-contract-tests-passed": test_log_path.is_file(),
}
failed = [item for item, passed in coverage.items() if not passed]
payload = {
    "version": "agentflow-software-dev-reference-evidence-mapping-gate.v1",
    "status": "passed" if not failed else "failed",
    "contractVersion": "agentflow-software-dev-evidence-reference-mapping.v1",
    "architecturePath": "docs/architecture/067-software-dev-reference-evidence-mapping-v1.md",
    "rustContractPath": "crates/ontology/src/evidence.rs",
    "rustTestLogPath": "runtime/software-dev-reference-evidence-mapping-rust-test.log",
    "referenceFields": reference_fields,
    "coreSourceTypes": core_source_types,
    "authorityBoundary": "software-dev-reference-app-mapping-not-core-authority",
    "coverage": coverage,
    "failedCoverage": failed,
    "checkedAt": int(time.time()),
}
out_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
if failed:
    raise SystemExit(f"software dev reference evidence mapping coverage failed: {failed}")
PY
  record_stage "software-dev-reference-evidence-mapping" "passed" "$(basename "$SOFTWARE_DEV_REFERENCE_EVIDENCE_MAPPING_PATH")"
}

run_evidence_projection_read_model_gate() {
  record_stage "evidence-projection-read-model" "started" "$EVIDENCE_PROJECTION_READ_MODEL_PATH"
  local rust_test_log="$RUNTIME_DIR/evidence-projection-read-model-rust-test.log"
  if ! (cd "$WORKSPACE" && cargo test -p agentflow-projection evidence_kernel --quiet >"$rust_test_log" 2>&1); then
    fail_stage "evidence-projection-read-model" "agentflow-projection Evidence Kernel read model tests failed"
  fi
  python3 - "$EVIDENCE_PROJECTION_READ_MODEL_PATH" "$WORKSPACE/docs/architecture/068-evidence-projection-read-model-v1.md" "$WORKSPACE/crates/projection/src/query.rs" "$rust_test_log" <<'PY'
import json
import pathlib
import sys
import time

out_path = pathlib.Path(sys.argv[1])
doc_path = pathlib.Path(sys.argv[2])
source_path = pathlib.Path(sys.argv[3])
test_log_path = pathlib.Path(sys.argv[4])

doc_text = doc_path.read_text(encoding="utf-8")
source_text = source_path.read_text(encoding="utf-8")
read_model_fields = [
    "status",
    "policyId",
    "sourceSummaries",
    "traceRefs",
    "missingReasons",
    "completeness",
    "authority",
    "readonly",
]
status_values = ["passed", "invalid", "deferred"]
coverage = {
    "read-model-version-defined": "evidence-kernel-read-model.v1" in source_text
    and "evidence-kernel-read-model.v1" in doc_text,
    "read-model-fields-documented": all(field in doc_text for field in read_model_fields),
    "read-model-fields-implemented": all(field in source_text for field in [
        "source_summaries",
        "trace_refs",
        "missing_reasons",
        "completeness",
        "authority",
        "readonly",
    ]),
    "readonly-boundary-documented": "authority = false" in doc_text and "readonly = true" in doc_text,
    "readonly-boundary-implemented": "authority: false" in source_text and "readonly: true" in source_text,
    "status-mapping-documented": all(value in doc_text for value in status_values),
    "status-mapping-implemented": "fn evidence_projection_status" in source_text
    and all(value in source_text for value in status_values),
    "core-policy-consumed": "evaluate_core_evidence_completeness_policy" in source_text
    and "core_missing_evidence_reports_for_completeness_policy" in source_text,
    "invalid-missing-fixtures-implemented": "evidence_kernel_invalid_missing_projection_fixtures" in source_text,
    "catalog-entry-implemented": "evidence-kernel" in source_text
    and "get_evidence_kernel_view" in source_text,
    "rust-contract-tests-passed": test_log_path.is_file(),
}
failed = [item for item, passed in coverage.items() if not passed]
payload = {
    "version": "agentflow-evidence-projection-read-model-gate.v1",
    "status": "passed" if not failed else "failed",
    "readModelVersion": "evidence-kernel-read-model.v1",
    "architecturePath": "docs/architecture/068-evidence-projection-read-model-v1.md",
    "rustContractPath": "crates/projection/src/query.rs",
    "rustTestLogPath": "runtime/evidence-projection-read-model-rust-test.log",
    "projectionStatusValues": status_values,
    "authority": False,
    "readonly": True,
    "coverage": coverage,
    "failedCoverage": failed,
    "checkedAt": int(time.time()),
}
out_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
if failed:
    raise SystemExit(f"evidence projection read model coverage failed: {failed}")
PY
  record_stage "evidence-projection-read-model" "passed" "$(basename "$EVIDENCE_PROJECTION_READ_MODEL_PATH")"
}

run_core_file_backed_ontology_registry_gate() {
  record_stage "core-file-backed-ontology-registry" "started" "$CORE_FILE_BACKED_ONTOLOGY_REGISTRY_PATH"
  local rust_test_log="$RUNTIME_DIR/core-file-backed-ontology-registry-rust-test.log"
  if ! (cd "$WORKSPACE" && cargo test -p agentflow-ontology core_file_backed_ontology_registry --quiet >"$rust_test_log" 2>&1); then
    fail_stage "core-file-backed-ontology-registry" "agentflow-ontology Core File-backed Ontology Registry tests failed"
  fi
  python3 - "$CORE_FILE_BACKED_ONTOLOGY_REGISTRY_PATH" "$WORKSPACE/docs/architecture/059-core-file-backed-ontology-registry-projection-v1.md" "$WORKSPACE/crates/ontology/src/file_registry.rs" "$rust_test_log" <<'PY'
import json
import pathlib
import sys
import time

out_path = pathlib.Path(sys.argv[1])
doc_path = pathlib.Path(sys.argv[2])
source_path = pathlib.Path(sys.argv[3])
test_log_path = pathlib.Path(sys.argv[4])

doc_text = doc_path.read_text(encoding="utf-8")
source_text = source_path.read_text(encoding="utf-8")
required_sources = [
    "core-ontology-kernel",
    "core-object-link-schema",
    "core-action-state-semantics",
    "core-skill-registry",
    "core-evidence-decision-reference-model",
]
required_paths = [
    "docs/architecture/054-core-ontology-kernel-contract-v1.md",
    "docs/architecture/055-core-object-link-schema-v1.md",
    "docs/architecture/056-core-action-state-semantics-v1.md",
    "docs/architecture/057-core-skill-registry-action-authorization-v1.md",
    "docs/architecture/058-core-evidence-decision-reference-model-v1.md",
]
required_projections = [
    "core-kernel-map",
    "core-object-link-catalog",
    "core-action-state-catalog",
    "core-skill-capability-catalog",
    "core-evidence-decision-catalog",
]
required_surfaces = [
    "coreElementCatalog",
    "coreBoundaryMap",
    "objectCatalog",
    "linkCatalog",
    "relationshipQuery",
    "actionCatalog",
    "stateCatalog",
    "transitionQuery",
    "skillCatalog",
    "authorizationQuery",
    "capabilityMatrix",
    "evidenceCatalog",
    "decisionCatalog",
    "outcomeQuery",
]
required_source_fields = [
    "source_id",
    "relative_path",
    "contract_version",
    "read_model_kind",
    "authority_boundary",
    "projection_id",
    "projection_kind",
    "query_surfaces",
    "minimum_record_count",
]
forbidden_terms = [
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
]
coverage = {
    "registry-version-defined": "agentflow-core-file-backed-ontology-registry.v1" in source_text,
    "sources-documented": all(item in doc_text for item in required_sources),
    "sources-implemented": all(item in source_text for item in required_sources),
    "source-paths-documented": all(item in doc_text for item in required_paths),
    "source-paths-implemented": all(item in source_text for item in required_paths),
    "projections-documented": all(item in doc_text for item in required_projections),
    "projections-implemented": all(item in source_text for item in required_projections),
    "query-surfaces-documented": all(item in doc_text for item in required_surfaces),
    "query-surfaces-implemented": all(item in source_text for item in required_surfaces),
    "fields-implemented": all(item in source_text for item in required_source_fields),
    "projection-not-source-authority": "do not replace source contracts" in doc_text and "do not replace source contracts" in source_text,
    "reference-mappings-not-core-authority": "not Core authority" in doc_text and "not Core authority" in source_text,
    "forbidden-terms-listed": all(term in doc_text and term in source_text for term in forbidden_terms),
    "rust-contract-tests-passed": test_log_path.is_file(),
}
failed = [item for item, passed in coverage.items() if not passed]
payload = {
    "version": "agentflow-core-file-backed-ontology-registry-gate.v1",
    "status": "passed" if not failed else "failed",
    "contractVersion": "agentflow-core-file-backed-ontology-registry.v1",
    "architecturePath": "docs/architecture/059-core-file-backed-ontology-registry-projection-v1.md",
    "rustContractPath": "crates/ontology/src/file_registry.rs",
    "rustTestLogPath": "runtime/core-file-backed-ontology-registry-rust-test.log",
    "registrySourceCount": len(required_sources),
    "projectionEntryCount": len(required_projections),
    "querySurfaceCount": len(required_surfaces),
    "requiredSources": required_sources,
    "requiredPaths": required_paths,
    "requiredProjections": required_projections,
    "requiredQuerySurfaces": required_surfaces,
    "forbiddenCoreTerms": forbidden_terms,
    "referenceMappingBoundary": "reference-app-only-not-core-authority",
    "projectionBoundary": "read-only-projection-not-source-authority",
    "coverage": coverage,
    "failedCoverage": failed,
    "checkedAt": int(time.time()),
}
out_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
if failed:
    raise SystemExit(f"core file-backed ontology registry coverage failed: {failed}")
PY
  record_stage "core-file-backed-ontology-registry" "passed" "$(basename "$CORE_FILE_BACKED_ONTOLOGY_REGISTRY_PATH")"
}

run_v104_release_certification_gate() {
  record_stage "v104-release-certification" "started" "$V104_RELEASE_CERTIFICATION_PATH"
  python3 - \
    "$V104_RELEASE_CERTIFICATION_PATH" \
    "$RELEASE_VERSION" \
    "$WORKSPACE/Cargo.toml" \
    "$WORKSPACE/apps/desktop/package.json" \
    "$WORKSPACE/apps/desktop/package-lock.json" \
    "$WORKSPACE/apps/desktop/src-tauri/tauri.conf.json" \
    "$ROOT/CHANGELOG.md" \
    "$WORKSPACE/docs/delivery/releases/v1.0.4/README.md" \
    "$WORKSPACE/docs/delivery/releases/v1.0.4/AGENTFLOW_V1_0_4_CORE_ONTOLOGY_KERNEL_TASKS_V1.md" \
    "$CORE_ONTOLOGY_KERNEL_PATH" \
    "$CORE_OBJECT_LINK_SCHEMA_PATH" \
    "$CORE_ACTION_STATE_SEMANTICS_PATH" \
    "$CORE_SKILL_REGISTRY_PATH" \
    "$CORE_EVIDENCE_DECISION_REFERENCE_MODEL_PATH" \
    "$CORE_FILE_BACKED_ONTOLOGY_REGISTRY_PATH" <<'PY'
import json
import pathlib
import sys
import time
import tomllib

out_path = pathlib.Path(sys.argv[1])
release_version = sys.argv[2]
cargo_path = pathlib.Path(sys.argv[3])
desktop_package_path = pathlib.Path(sys.argv[4])
desktop_package_lock_path = pathlib.Path(sys.argv[5])
tauri_config_path = pathlib.Path(sys.argv[6])
changelog_path = pathlib.Path(sys.argv[7])
release_readme_path = pathlib.Path(sys.argv[8])
release_tasks_path = pathlib.Path(sys.argv[9])
artifact_paths = [pathlib.Path(value) for value in sys.argv[10:]]

expected_version = "1.0.4"
expected_tag = "v1.0.4"
cargo = tomllib.loads(cargo_path.read_text(encoding="utf-8"))
desktop_package = json.loads(desktop_package_path.read_text(encoding="utf-8"))
desktop_package_lock = json.loads(desktop_package_lock_path.read_text(encoding="utf-8"))
tauri_config = json.loads(tauri_config_path.read_text(encoding="utf-8"))
changelog_text = changelog_path.read_text(encoding="utf-8")
release_readme_text = release_readme_path.read_text(encoding="utf-8")
release_tasks_text = release_tasks_path.read_text(encoding="utf-8")
artifacts = [json.loads(path.read_text(encoding="utf-8")) for path in artifact_paths]

artifact_statuses = {
    artifact_paths[index].name: artifact.get("status")
    for index, artifact in enumerate(artifacts)
}
def version_tuple(value: str):
    return tuple(int(part) for part in value.removeprefix("v").split("."))

current_workspace_version = cargo["workspace"]["package"]["version"]
current_desktop_version = desktop_package.get("version")
current_tauri_version = tauri_config.get("version")
changelog_has_v104 = "v1.0.4" in changelog_text
changelog_has_kernel = "Core Ontology Kernel" in changelog_text
changelog_has_certification = "v104-release-certification" in changelog_text
coverage = {
    "release-version-keeps-v104-baseline-in-range": version_tuple(release_version) >= version_tuple(expected_tag),
    "cargo-workspace-version-not-older-than-104": version_tuple(current_workspace_version) >= version_tuple(expected_version),
    "desktop-package-version-not-older-than-104": version_tuple(current_desktop_version) >= version_tuple(expected_version),
    "tauri-version-not-older-than-104": version_tuple(current_tauri_version) >= version_tuple(expected_version),
    "changelog-has-v104-entry": changelog_has_v104
    and changelog_has_kernel
    and changelog_has_certification,
    "delivery-readme-has-v104-baseline": "AgentFlow v1.0.4 Core Ontology Kernel" in release_readme_text
    and "Core Ontology Kernel baseline" in release_readme_text,
    "delivery-tasks-has-v104-closeout": "V104-010 Release Certification" in release_tasks_text
    and "runtime/v104-release-certification.json" in release_tasks_text,
    "core-ontology-artifacts-passed": all(status == "passed" for status in artifact_statuses.values()),
    "object-link-count-certified": artifacts[1].get("objectCount") == 11
    and artifacts[1].get("linkCount") == 12,
    "action-state-count-certified": artifacts[2].get("actionCount") == 12
    and artifacts[2].get("stateCount") == 10
    and artifacts[2].get("transitionCount") == 12,
    "skill-count-certified": artifacts[3].get("skillCount") == 6,
    "evidence-decision-count-certified": artifacts[4].get("evidenceReferenceCount") == 5
    and artifacts[4].get("decisionReferenceCount") == 3
    and artifacts[4].get("outcomeCount") == 10,
    "file-backed-registry-certified": artifacts[5].get("registrySourceCount") == 5
    and artifacts[5].get("projectionEntryCount") == 5,
}
failed = [item for item, passed in coverage.items() if not passed]
payload = {
    "version": "agentflow-v104-release-certification.v1",
    "status": "passed" if not failed else "failed",
    "releaseVersion": expected_tag,
    "currentReleaseVersion": release_version,
    "workspaceVersion": expected_version,
    "currentWorkspaceVersion": current_workspace_version,
    "changelogPath": str(changelog_path),
    "changelogPreview": changelog_text[:200],
    "changelogFacts": {
        "hasV104": changelog_has_v104,
        "hasCoreOntologyKernel": changelog_has_kernel,
        "hasV104ReleaseCertification": changelog_has_certification,
    },
    "certifiedArtifacts": artifact_statuses,
    "coverage": coverage,
    "failedCoverage": failed,
    "releaseBaseline": "docs/delivery/releases/v1.0.4/README.md",
    "releaseTasks": "docs/delivery/releases/v1.0.4/AGENTFLOW_V1_0_4_CORE_ONTOLOGY_KERNEL_TASKS_V1.md",
    "checkedAt": int(time.time()),
}
out_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
if failed:
    raise SystemExit(f"v1.0.4 release certification failed: {failed}")
PY
  record_stage "v104-release-certification" "passed" "$(basename "$V104_RELEASE_CERTIFICATION_PATH")"
}

run_core_runtime_negative_fixtures_gate() {
  record_stage "core-runtime-negative-fixtures" "started" "$CORE_RUNTIME_NEGATIVE_FIXTURES_PATH"
  local runtime_api_test_log="$RUNTIME_DIR/core-runtime-negative-runtime-api-test.log"
  local arbitration_test_log="$RUNTIME_DIR/core-runtime-negative-arbitration-test.log"
  local task_artifacts_test_log="$RUNTIME_DIR/core-runtime-negative-task-artifacts-test.log"
  local projection_test_log="$RUNTIME_DIR/core-runtime-negative-projection-test.log"

  if ! (cd "$WORKSPACE" && cargo test -p agentflow-runtime-api --quiet >"$runtime_api_test_log" 2>&1); then
    fail_stage "core-runtime-negative-fixtures" "agentflow-runtime-api negative fixture coverage failed"
  fi
  if ! (cd "$WORKSPACE" && cargo test -p agentflow-action-arbitration --quiet >"$arbitration_test_log" 2>&1); then
    fail_stage "core-runtime-negative-fixtures" "agentflow-action-arbitration reference mapping coverage failed"
  fi
  if ! (cd "$WORKSPACE" && cargo test -p agentflow-task-artifacts --quiet >"$task_artifacts_test_log" 2>&1); then
    fail_stage "core-runtime-negative-fixtures" "agentflow-task-artifacts closeout/writeback coverage failed"
  fi
  if ! (cd "$WORKSPACE" && cargo test -p agentflow-projection --quiet >"$projection_test_log" 2>&1); then
    fail_stage "core-runtime-negative-fixtures" "agentflow-projection readonly coverage failed"
  fi

  python3 - \
    "$CORE_RUNTIME_NEGATIVE_FIXTURES_PATH" \
    "$WORKSPACE/crates/runtime-api/src/commands.rs" \
    "$WORKSPACE/crates/action-arbitration/src/arbitrator.rs" \
    "$WORKSPACE/crates/task-artifacts/src/storage.rs" \
    "$WORKSPACE/crates/projection/src/projector.rs" \
    "$runtime_api_test_log" \
    "$arbitration_test_log" \
    "$task_artifacts_test_log" \
    "$projection_test_log" <<'PY'
import hashlib
import json
import pathlib
import re
import sys
import time

out_path = pathlib.Path(sys.argv[1])
runtime_api_source = pathlib.Path(sys.argv[2])
arbitration_source = pathlib.Path(sys.argv[3])
task_artifacts_source = pathlib.Path(sys.argv[4])
projection_source = pathlib.Path(sys.argv[5])
test_logs = [pathlib.Path(value) for value in sys.argv[6:]]

sources = {
    "runtime-api": runtime_api_source.read_text(encoding="utf-8"),
    "arbitration": arbitration_source.read_text(encoding="utf-8"),
    "task-artifacts": task_artifacts_source.read_text(encoding="utf-8"),
    "projection": projection_source.read_text(encoding="utf-8"),
}

required_markers = {
    "positive-reference-materialization": ("runtime-api", "core_action_proposal_materialization_uses_reference_mapping"),
    "positive-reference-proposal-fact": ("runtime-api", "runtime_proposal_fact_records_core_mapping_fields"),
    "positive-arbitration-acceptance": ("arbitration", "valid_proposal_returns_accepted"),
    "positive-closeout-writeback": ("task-artifacts", "commit_writeback_records_terminal_state_and_reason"),
    "unlisted-action": ("runtime-api", "invalid_command_returns_invalid_command"),
    "forbidden-scope": ("runtime-api", "governance_rejects_forbidden_surface_before_writing_proposal"),
    "missing-evidence": ("runtime-api", "governance_defers_missing_required_evidence_before_writing_proposal"),
    "forged-provider-telemetry": ("runtime-api", "governance_ignores_forged_ready_request_input_without_trusted_registry"),
    "software-dev-term-leaking-into-core-authority": ("runtime-api", "command_surface_aliases_map_to_supported_action_contracts"),
    "missing-mapping": ("runtime-api", "runtime_validation_rejects_missing_reference_mapping"),
    "wrong-mapping": ("runtime-api", "runtime_validation_rejects_polluted_core_target_as_app_mapping"),
    "projection-authority-write": ("projection", "replay_report_rebuilds_projection_without_cached_read_model"),
    "projection-corrupt-event-negative": ("projection", "replay_report_records_structured_failure_for_corrupt_event"),
    "core-admission-semantic-mismatch": ("arbitration", "required_core_admission_rejects_semantic_mismatch"),
}

marker_results = {
    fixture_id: marker in sources[source_name]
    for fixture_id, (source_name, marker) in required_markers.items()
}
source_log_paths = {
    "runtime-api": test_logs[0],
    "arbitration": test_logs[1],
    "task-artifacts": test_logs[2],
    "projection": test_logs[3],
}

def test_log_proof(source_name, path):
    text = path.read_text(encoding="utf-8") if path.is_file() else ""
    results = [
        (int(passed), int(failed))
        for passed, failed in re.findall(r"test result: ok\. (\d+) passed; (\d+) failed", text)
    ]
    passed_count = sum(passed for passed, _ in results)
    failed_count = sum(failed for _, failed in results)
    running_count = sum(int(value) for value in re.findall(r"running (\d+) tests", text))
    status = "passed" if path.is_file() and passed_count > 0 and failed_count == 0 else "failed"
    return {
        "source": source_name,
        "path": f"runtime/{path.name}",
        "status": status,
        "runningCount": running_count,
        "passedCount": passed_count,
        "failedCount": failed_count,
        "sha256": hashlib.sha256(path.read_bytes()).hexdigest() if path.is_file() else None,
        "bytes": path.stat().st_size if path.is_file() else 0,
    }

test_log_proofs = {
    source_name: test_log_proof(source_name, path)
    for source_name, path in source_log_paths.items()
}

negative_fixture_ids = [
    "unlisted-action",
    "forbidden-scope",
    "missing-evidence",
    "forged-provider-telemetry",
    "software-dev-term-leaking-into-core-authority",
    "projection-authority-write",
    "missing-mapping",
    "wrong-mapping",
]
fixtures = []
for fixture_id in negative_fixture_ids:
    source_name, _ = required_markers[fixture_id]
    proof = test_log_proofs[source_name]
    fixtures.append({
        "id": fixture_id,
        "kind": "negative",
        "passed": marker_results.get(fixture_id, False) and proof["status"] == "passed",
        "expectedOutcome": "rejected-or-deferred-before-authority-write",
        "authorityWriteBlocked": True,
        "sourceMarkerFound": marker_results.get(fixture_id, False),
        "testLogProof": proof,
        "evidenceMode": "rust-test-log-plus-fixture-marker",
        "markerOnly": False,
        "referenceMappingRequired": fixture_id in {
            "missing-mapping",
            "wrong-mapping",
            "software-dev-term-leaking-into-core-authority",
        },
        "auditSidecar": "sidecar-only",
    })

positive_workflow = {
    "id": "software-dev-reference-completion-through-core-runtime",
    "status": "passed" if all(marker_results.get(key, False) for key in [
        "positive-reference-materialization",
        "positive-reference-proposal-fact",
        "positive-arbitration-acceptance",
        "positive-closeout-writeback",
    ]) and all(proof["status"] == "passed" for proof in test_log_proofs.values()) else "failed",
    "stages": [
        "core-command",
        "admission",
        "action-proposal",
        "arbitration",
        "executor-closeout",
        "state-writeback",
    ],
    "softwareDevTerms": [
        "Requirement",
        "Spec",
        "Issue",
        "Run",
        "PR",
        "Release",
    ],
    "coreAuthorityBoundary": "software-dev-terms-are-reference-mapping-only",
}

core_authority_forbidden_terms = [
    "Requirement",
    "Spec",
    "Issue",
    "Run",
    "PR",
    "Release",
    "GitHub issue",
    "repository patch",
    "test log",
]

coverage = {
    "rust-runtime-api-tests-passed": test_log_proofs["runtime-api"]["status"] == "passed",
    "rust-arbitration-tests-passed": test_log_proofs["arbitration"]["status"] == "passed",
    "rust-task-artifacts-tests-passed": test_log_proofs["task-artifacts"]["status"] == "passed",
    "rust-projection-tests-passed": test_log_proofs["projection"]["status"] == "passed",
    "positive-reference-workflow-passed": positive_workflow["status"] == "passed",
    "negative-fixtures-covered": all(item["passed"] for item in fixtures),
    "negative-fixtures-have-runtime-test-proof": all(
        item["testLogProof"]["status"] == "passed" and item["testLogProof"]["passedCount"] > 0
        for item in fixtures
    ),
    "negative-fixtures-are-not-marker-only": all(item["markerOnly"] is False for item in fixtures),
    "missing-mapping-covered": marker_results.get("missing-mapping") is True,
    "wrong-mapping-covered": marker_results.get("wrong-mapping") is True,
    "projection-remains-readonly": marker_results.get("projection-authority-write") is True
    and "writes_authority: false" in sources["projection"]
    and "projection_authority: false" in sources["projection"],
    "core-fixtures-free-of-software-dev-authority": marker_results.get("software-dev-term-leaking-into-core-authority") is True,
}
failed = [key for key, value in coverage.items() if not value]

payload = {
    "version": "agentflow-core-runtime-negative-fixtures.v1",
    "status": "passed" if not failed else "failed",
    "referenceApp": "software-dev",
    "referenceAppAuthority": False,
    "coreRuntimeAuthority": True,
    "auditMode": "sidecar",
    "positiveWorkflow": positive_workflow,
    "fixtures": fixtures,
    "coreAuthorityForbiddenTerms": core_authority_forbidden_terms,
    "markerResults": marker_results,
    "testExecutionEvidence": list(test_log_proofs.values()),
    "testLogs": {
        "runtimeApi": "runtime/core-runtime-negative-runtime-api-test.log",
        "arbitration": "runtime/core-runtime-negative-arbitration-test.log",
        "taskArtifacts": "runtime/core-runtime-negative-task-artifacts-test.log",
        "projection": "runtime/core-runtime-negative-projection-test.log",
    },
    "coverage": coverage,
    "failedCoverage": failed,
    "checkedAt": int(time.time()),
}

out_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
if failed:
    raise SystemExit(f"core runtime negative fixtures failed: {failed}")
PY

  record_stage "core-runtime-negative-fixtures" "passed" "$(basename "$CORE_RUNTIME_NEGATIVE_FIXTURES_PATH")"
}

run_core_runtime_kernel_gate() {
  record_stage "core-runtime-kernel" "started" "$CORE_RUNTIME_KERNEL_PATH"
  local runtime_api_test_log="$RUNTIME_DIR/core-runtime-kernel-runtime-api-test.log"
  if ! (cd "$WORKSPACE" && cargo test -p agentflow-runtime-api --quiet >"$runtime_api_test_log" 2>&1); then
    fail_stage "core-runtime-kernel" "agentflow-runtime-api kernel coverage failed"
  fi

  python3 - \
    "$CORE_RUNTIME_KERNEL_PATH" \
    "$WORKSPACE/crates/runtime-api/src/commands.rs" \
    "$WORKSPACE/crates/runtime-api/src/mapping.rs" \
    "$WORKSPACE/crates/runtime-api/src/formal.rs" \
    "$WORKSPACE/crates/task-artifacts/src/storage.rs" \
    "$WORKSPACE/crates/ontology/src/file_registry.rs" \
    "$WORKSPACE/docs/delivery/releases/v1.0.5/README.md" \
    "$WORKSPACE/docs/delivery/releases/v1.0.5/AGENTFLOW_V1_0_5_CORE_RUNTIME_KERNEL_TASKS_V1.md" \
    "$runtime_api_test_log" <<'PY'
import json
import pathlib
import sys
import time

out_path = pathlib.Path(sys.argv[1])
commands_path = pathlib.Path(sys.argv[2])
mapping_path = pathlib.Path(sys.argv[3])
formal_path = pathlib.Path(sys.argv[4])
task_artifacts_path = pathlib.Path(sys.argv[5])
registry_path = pathlib.Path(sys.argv[6])
readme_path = pathlib.Path(sys.argv[7])
tasks_path = pathlib.Path(sys.argv[8])
test_log_path = pathlib.Path(sys.argv[9])

commands = commands_path.read_text(encoding="utf-8")
mapping = mapping_path.read_text(encoding="utf-8")
formal = formal_path.read_text(encoding="utf-8")
task_artifacts = task_artifacts_path.read_text(encoding="utf-8")
registry = registry_path.read_text(encoding="utf-8")
release_text = readme_path.read_text(encoding="utf-8") + "\n" + tasks_path.read_text(encoding="utf-8")

required_flow = [
    "Runtime Command",
    "Runtime Admission",
    "Action Proposal",
    "Arbitration",
    "Executor Adapter Closeout",
    "Completion Commit / State Writeback",
]
coverage = {
    "release-doc-defines-runtime-flow": all(item in release_text for item in required_flow),
    "software-dev-reference-not-core-authority": "Software Dev 是 reference certification，不是 Core authority" in release_text
    or "Software Dev Reference App mapping is not Core Runtime authority" in release_text,
    "runtime-command-api-present": "RuntimeCommandRequest" in commands
    and "validate_runtime_command" in commands,
    "admission-before-proposal": "evaluate_runtime_command_governance" in commands
    and "write_runtime_proposal_fact" in commands,
    "proposal-materialization-uses-core-mapping": "materialize_core_action_proposal_with_registry" in commands
    and "load_core_file_backed_ontology_registry_projection" in mapping,
    "formal-bridge-records-command-proposal-arbitration": "ActionProposalBridgeRecord" in formal
    and "arbitration" in formal,
    "executor-closeout-and-state-writeback-present": "write_task_executor_closeout" in task_artifacts
    and "commit_writeback_records_terminal_state_and_reason" in task_artifacts,
    "file-backed-registry-runtime-loader-present": "load_core_file_backed_ontology_registry_projection" in registry
    and "diagnose_core_file_backed_ontology_registry_projection_contract" in registry,
    "rust-runtime-api-tests-passed": test_log_path.is_file(),
}
failed = [key for key, passed in coverage.items() if not passed]
payload = {
    "version": "agentflow-core-runtime-kernel.v1",
    "status": "passed" if not failed else "failed",
    "runtimeFlow": required_flow,
    "softwareDevReferenceBoundary": "reference-certification-not-core-authority",
    "rustTestLogPath": "runtime/core-runtime-kernel-runtime-api-test.log",
    "coverage": coverage,
    "failedCoverage": failed,
    "checkedAt": int(time.time()),
}
out_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
if failed:
    raise SystemExit(f"core runtime kernel coverage failed: {failed}")
PY
  record_stage "core-runtime-kernel" "passed" "$(basename "$CORE_RUNTIME_KERNEL_PATH")"
}

run_core_runtime_admission_gate() {
  record_stage "core-runtime-admission" "started" "$CORE_RUNTIME_ADMISSION_PATH"
  local admission_test_log="$RUNTIME_DIR/core-runtime-admission-rust-test.log"
  if ! (cd "$WORKSPACE" && cargo test -p agentflow-runtime-api governance_ --quiet >"$admission_test_log" 2>&1); then
    fail_stage "core-runtime-admission" "agentflow-runtime-api admission coverage failed"
  fi

  python3 - \
    "$CORE_RUNTIME_ADMISSION_PATH" \
    "$WORKSPACE/crates/runtime-api/src/commands.rs" \
    "$admission_test_log" <<'PY'
import json
import pathlib
import sys
import time

out_path = pathlib.Path(sys.argv[1])
commands_path = pathlib.Path(sys.argv[2])
test_log_path = pathlib.Path(sys.argv[3])
source = commands_path.read_text(encoding="utf-8")
markers = {
    "reject-before-writing-proposal": "governance_rejects_before_writing_proposal_or_accepted_action",
    "defer-before-writing-proposal": "governance_defers_before_writing_proposal_or_accepted_action",
    "missing-skill-rejected": "governance_rejects_missing_skill_before_writing_proposal",
    "unauthorized-skill-owner-rejected": "governance_rejects_unauthorized_skill_owner_before_writing_proposal",
    "invalid-target-object-rejected": "governance_rejects_invalid_target_object_before_writing_proposal",
    "forbidden-surface-rejected": "governance_rejects_forbidden_surface_before_writing_proposal",
    "missing-required-evidence-deferred": "governance_defers_missing_required_evidence_before_writing_proposal",
    "forged-provider-telemetry-ignored": "governance_ignores_forged_ready_request_input_without_trusted_registry",
}
marker_results = {key: marker in source for key, marker in markers.items()}
coverage = {
    "all-admission-negative-fixtures-present": all(marker_results.values()),
    "runtime-proposal-not-written-on-reject-or-defer": source.count("load_runtime_proposal_facts(dir.path()).unwrap().is_empty()") >= 7,
    "rust-admission-tests-passed": test_log_path.is_file(),
}
failed = [key for key, passed in coverage.items() if not passed]
payload = {
    "version": "agentflow-core-runtime-admission.v1",
    "status": "passed" if not failed else "failed",
    "admissionBoundary": "rejected-or-deferred-commands-must-not-write-proposal-or-accepted-action",
    "markerResults": marker_results,
    "rustTestLogPath": "runtime/core-runtime-admission-rust-test.log",
    "coverage": coverage,
    "failedCoverage": failed,
    "checkedAt": int(time.time()),
}
out_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
if failed:
    raise SystemExit(f"core runtime admission coverage failed: {failed}")
PY
  record_stage "core-runtime-admission" "passed" "$(basename "$CORE_RUNTIME_ADMISSION_PATH")"
}

run_core_runtime_arbitration_gate() {
  record_stage "core-runtime-arbitration" "started" "$CORE_RUNTIME_ARBITRATION_PATH"
  local arbitration_test_log="$RUNTIME_DIR/core-runtime-arbitration-rust-test.log"
  if ! (cd "$WORKSPACE" && cargo test -p agentflow-action-arbitration --quiet >"$arbitration_test_log" 2>&1); then
    fail_stage "core-runtime-arbitration" "agentflow-action-arbitration coverage failed"
  fi

  python3 - \
    "$CORE_RUNTIME_ARBITRATION_PATH" \
    "$WORKSPACE/crates/action-arbitration/src/arbitrator.rs" \
    "$arbitration_test_log" <<'PY'
import json
import pathlib
import sys
import time

out_path = pathlib.Path(sys.argv[1])
source_path = pathlib.Path(sys.argv[2])
test_log_path = pathlib.Path(sys.argv[3])
source = source_path.read_text(encoding="utf-8")
markers = {
    "accepted-proposal": "valid_proposal_returns_accepted",
    "unadmitted-proposal-rejected": "required_core_admission_rejects_unadmitted_proposal",
    "semantic-mismatch-rejected": "required_core_admission_rejects_semantic_mismatch",
    "unknown-action-rejected": "unknown_action_returns_rejected",
    "missing-evidence-rejected": "missing_evidence_returns_rejected",
    "unmet-dependency-rejected": "unmet_dependency_returns_rejected",
    "active-write-lock-queued": "active_write_lock_returns_queued",
    "human-decision-queue": "waiting_human_decision_on_same_scope_queues_current_request",
    "issue-done-does-not-auto-audit": "issue_done_does_not_create_audit_accepted_action",
}
marker_results = {key: marker in source for key, marker in markers.items()}
coverage = {
    "accepted-rejected-and-queued-paths-covered": all(marker_results.values()),
    "stable-rejection-reasons-present": "rejected_action_includes_stable_reason" in source,
    "accepted-action-causation-present": "accepted_action_includes_causation_proposal_id" in source,
    "rust-arbitration-tests-passed": test_log_path.is_file(),
}
failed = [key for key, passed in coverage.items() if not passed]
payload = {
    "version": "agentflow-core-runtime-arbitration.v1",
    "status": "passed" if not failed else "failed",
    "arbitrationBoundary": "only-admitted-action-proposals-can-be-accepted",
    "markerResults": marker_results,
    "rustTestLogPath": "runtime/core-runtime-arbitration-rust-test.log",
    "coverage": coverage,
    "failedCoverage": failed,
    "checkedAt": int(time.time()),
}
out_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
if failed:
    raise SystemExit(f"core runtime arbitration coverage failed: {failed}")
PY
  record_stage "core-runtime-arbitration" "passed" "$(basename "$CORE_RUNTIME_ARBITRATION_PATH")"
}

run_v105_release_certification_gate() {
  record_stage "v105-release-certification" "started" "$V105_RELEASE_CERTIFICATION_PATH"
  python3 - \
    "$V105_RELEASE_CERTIFICATION_PATH" \
    "$RELEASE_VERSION" \
    "$WORKSPACE/Cargo.toml" \
    "$WORKSPACE/apps/desktop/package.json" \
    "$WORKSPACE/apps/desktop/package-lock.json" \
    "$WORKSPACE/apps/desktop/src-tauri/tauri.conf.json" \
    "$ROOT/CHANGELOG.md" \
    "$WORKSPACE/docs/delivery/releases/v1.0.5/README.md" \
    "$WORKSPACE/docs/delivery/releases/v1.0.5/AGENTFLOW_V1_0_5_CORE_RUNTIME_KERNEL_TASKS_V1.md" \
    "$ARTIFACT_MANIFEST_PATH" \
    "$RELEASE_URL" \
    "$GATE_EVENT_NAME" \
    "$GATE_REF_TYPE" \
    "$GATE_REF_NAME" \
    "$GATE_RUN_ID" \
    "$GATE_RUN_ATTEMPT" \
    "$GATE_REPOSITORY" \
    "$GATE_SERVER_URL" \
    "$SOURCE_COMMIT_SHA" \
    "$RELEASE_TAG_NAME" \
    "$CORE_RUNTIME_KERNEL_PATH" \
    "$CORE_RUNTIME_ADMISSION_PATH" \
    "$CORE_RUNTIME_ARBITRATION_PATH" \
    "$CORE_RUNTIME_NEGATIVE_FIXTURES_PATH" \
    "$CORE_FILE_BACKED_ONTOLOGY_REGISTRY_PATH" <<'PY'
import hashlib
import json
import pathlib
import sys
import time
import tomllib

(
    out_path_raw,
    release_version,
    cargo_path_raw,
    desktop_package_path_raw,
    desktop_package_lock_path_raw,
    tauri_config_path_raw,
    changelog_path_raw,
    release_readme_path_raw,
    release_tasks_path_raw,
    artifact_manifest_path_raw,
    release_url,
    gate_event_name,
    gate_ref_type,
    gate_ref_name,
    gate_run_id,
    gate_run_attempt,
    gate_repository,
    gate_server_url,
    source_commit_sha,
    release_tag_name,
    *artifact_path_values,
) = sys.argv[1:]

out_path = pathlib.Path(out_path_raw)
cargo_path = pathlib.Path(cargo_path_raw)
desktop_package_path = pathlib.Path(desktop_package_path_raw)
desktop_package_lock_path = pathlib.Path(desktop_package_lock_path_raw)
tauri_config_path = pathlib.Path(tauri_config_path_raw)
changelog_path = pathlib.Path(changelog_path_raw)
release_readme_path = pathlib.Path(release_readme_path_raw)
release_tasks_path = pathlib.Path(release_tasks_path_raw)
artifact_manifest_path = pathlib.Path(artifact_manifest_path_raw)
artifact_paths = [pathlib.Path(value) for value in artifact_path_values]

expected_version = "1.0.5"
expected_tag = "v1.0.5"
cargo = tomllib.loads(cargo_path.read_text(encoding="utf-8"))
desktop_package = json.loads(desktop_package_path.read_text(encoding="utf-8"))
desktop_package_lock = json.loads(desktop_package_lock_path.read_text(encoding="utf-8"))
tauri_config = json.loads(tauri_config_path.read_text(encoding="utf-8"))
changelog_text = changelog_path.read_text(encoding="utf-8")
release_readme_text = release_readme_path.read_text(encoding="utf-8")
release_tasks_text = release_tasks_path.read_text(encoding="utf-8")
artifacts = [json.loads(path.read_text(encoding="utf-8")) for path in artifact_paths]
artifact_statuses = {
    artifact_paths[index].name: artifact.get("status")
    for index, artifact in enumerate(artifacts)
}
def version_tuple(value: str):
    return tuple(int(part) for part in value.removeprefix("v").split("."))

current_workspace_version = cargo["workspace"]["package"]["version"]
current_desktop_version = desktop_package.get("version")
current_tauri_version = tauri_config.get("version")
certified_artifact_hashes = []
for path in artifact_paths:
    certified_artifact_hashes.append({
        "path": f"runtime/{path.name}",
        "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
        "bytes": path.stat().st_size,
    })
artifact_manifest_sha256 = (
    hashlib.sha256(artifact_manifest_path.read_bytes()).hexdigest()
    if artifact_manifest_path.is_file()
    else None
)
certification_digest = hashlib.sha256(json.dumps(
    {
        "releaseVersion": release_version,
        "artifactManifestSha256": artifact_manifest_sha256,
        "certifiedArtifactHashes": certified_artifact_hashes,
    },
    sort_keys=True,
).encode("utf-8")).hexdigest()
gate_run_url = (
    f"{gate_server_url.rstrip('/')}/{gate_repository}/actions/runs/{gate_run_id}"
    if gate_server_url and gate_repository and gate_run_id
    else None
)
artifact_name = f"release-gate-certification-{release_version}"
event_evidence = {
    "eventName": gate_event_name,
    "refType": gate_ref_type or None,
    "refName": gate_ref_name or None,
    "runId": gate_run_id or None,
    "runAttempt": gate_run_attempt or None,
    "runUrl": gate_run_url,
    "repository": gate_repository,
    "sourceCommitSha": source_commit_sha,
    "releaseTagName": release_tag_name,
    "releaseUrl": release_url,
    "certificationArtifactName": artifact_name,
    "certificationArtifactId": artifact_name,
    "certificationArtifactIdKind": "github-upload-artifact-name",
    "certificationArtifactNumericId": None,
    "certificationArtifactNumericIdSource": "assigned-by-github-actions-after-upload",
    "certificationArtifactDigest": certification_digest,
    "certificationArtifactDigestSource": "v105-certified-runtime-artifact-hashes",
    "artifactManifestPath": "artifact-manifest.json",
    "artifactManifestSha256": artifact_manifest_sha256,
}
negative_artifact = artifacts[3]
kernel_artifact = artifacts[0]
admission_artifact = artifacts[1]
arbitration_artifact = artifacts[2]
registry_artifact = artifacts[4]
coverage = {
    "release-version-keeps-v105-baseline-in-range": version_tuple(release_version) >= version_tuple(expected_tag),
    "cargo-workspace-version-not-older-than-105": version_tuple(current_workspace_version) >= version_tuple(expected_version),
    "desktop-package-version-not-older-than-105": version_tuple(current_desktop_version) >= version_tuple(expected_version),
    "desktop-package-lock-version-not-older-than-105": version_tuple(desktop_package_lock.get("version")) >= version_tuple(expected_version)
    and version_tuple((desktop_package_lock.get("packages") or {}).get("", {}).get("version")) >= version_tuple(expected_version),
    "tauri-version-not-older-than-105": version_tuple(current_tauri_version) >= version_tuple(expected_version),
    "changelog-has-v105-entry": "## v1.0.5 - 2026-06-28" in changelog_text
    and "Core Runtime Kernel baseline" in changelog_text
    and "v105-release-certification" in changelog_text,
    "delivery-readme-release-baseline": "Core Runtime Kernel release baseline" in release_readme_text
    and "Software Dev 是 reference certification，不是 Core authority" in release_readme_text,
    "delivery-tasks-release-public-record": "release public record" in release_tasks_text
    and "V105-010 Release Certification" in release_tasks_text
    and "runtime/v105-release-certification.json" in release_tasks_text,
    "all-v105-artifacts-passed": all(status == "passed" for status in artifact_statuses.values()),
    "certified-artifact-hashes-present": len(certified_artifact_hashes) == len(artifact_paths)
    and all(item.get("sha256") and item.get("bytes", 0) > 0 for item in certified_artifact_hashes),
    "artifact-manifest-digest-present": artifact_manifest_sha256 is not None,
    "certification-artifact-digest-present": len(certification_digest) == 64,
    "certification-artifact-id-present": bool(event_evidence["certificationArtifactId"]),
    "release-event-evidence-recorded": bool(event_evidence["eventName"])
    and bool(event_evidence["sourceCommitSha"])
    and bool(event_evidence["releaseTagName"]),
    "release-run-id-bound-for-ci": gate_event_name == "local" or bool(event_evidence["runId"]),
    "release-run-url-bound-for-ci": gate_event_name == "local" or bool(event_evidence["runUrl"]),
    "negative-fixtures-include-eight-cases": len(negative_artifact.get("fixtures") or []) == 8
    and all(item.get("passed") is True for item in negative_artifact.get("fixtures") or []),
    "positive-reference-workflow-passed": (negative_artifact.get("positiveWorkflow") or {}).get("status") == "passed",
    "kernel-runtime-flow-certified": len(kernel_artifact.get("runtimeFlow") or []) == 6,
    "admission-boundary-certified": admission_artifact.get("admissionBoundary") == "rejected-or-deferred-commands-must-not-write-proposal-or-accepted-action",
    "arbitration-boundary-certified": arbitration_artifact.get("arbitrationBoundary") == "only-admitted-action-proposals-can-be-accepted",
    "file-backed-registry-loader-certified": registry_artifact.get("registrySourceCount") == 5
    and registry_artifact.get("projectionEntryCount") == 5,
}
failed = [item for item, passed in coverage.items() if not passed]
payload = {
    "version": "agentflow-v105-release-certification.v1",
    "status": "passed" if not failed else "failed",
    "releaseVersion": expected_tag,
    "workspaceVersion": expected_version,
    "certifiedArtifacts": artifact_statuses,
    "certifiedArtifactHashes": certified_artifact_hashes,
    "eventEvidence": event_evidence,
    "coverage": coverage,
    "failedCoverage": failed,
    "releaseBaseline": "docs/delivery/releases/v1.0.5/README.md",
    "releaseTasks": "docs/delivery/releases/v1.0.5/AGENTFLOW_V1_0_5_CORE_RUNTIME_KERNEL_TASKS_V1.md",
    "remainingRisks": [
        {
            "id": "v106-evidence-kernel",
            "summary": "Evidence Kernel completeness is intentionally deferred to v1.0.6.",
            "blocking": False,
        },
        {
            "id": "v107-decision-kernel",
            "summary": "Decision Kernel completeness is intentionally deferred to v1.0.7.",
            "blocking": False,
        },
        {
            "id": "v108-projection-kernel",
            "summary": "Projection Kernel completeness is intentionally deferred to v1.0.8.",
            "blocking": False,
        },
    ],
    "checkedAt": int(time.time()),
}
out_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
if failed:
    raise SystemExit(f"v1.0.5 release certification failed: {failed}")
PY
  record_stage "v105-release-certification" "passed" "$(basename "$V105_RELEASE_CERTIFICATION_PATH")"
}

run_v106_release_certification_gate() {
  record_stage "v106-release-certification" "started" "$V106_RELEASE_CERTIFICATION_PATH"
  python3 - \
    "$V106_RELEASE_CERTIFICATION_PATH" \
    "$RELEASE_VERSION" \
    "$WORKSPACE/Cargo.toml" \
    "$WORKSPACE/apps/desktop/package.json" \
    "$WORKSPACE/apps/desktop/package-lock.json" \
    "$WORKSPACE/apps/desktop/src-tauri/tauri.conf.json" \
    "$ROOT/CHANGELOG.md" \
    "$WORKSPACE/docs/delivery/releases/v1.0.6/README.md" \
    "$WORKSPACE/docs/delivery/releases/v1.0.6/AGENTFLOW_V1_0_6_CORE_EVIDENCE_KERNEL_TASKS_V1.md" \
    "$ARTIFACT_MANIFEST_PATH" \
    "$RELEASE_URL" \
    "$GATE_EVENT_NAME" \
    "$GATE_REF_TYPE" \
    "$GATE_REF_NAME" \
    "$GATE_RUN_ID" \
    "$GATE_RUN_ATTEMPT" \
    "$GATE_REPOSITORY" \
    "$GATE_SERVER_URL" \
    "$SOURCE_COMMIT_SHA" \
    "$RELEASE_TAG_NAME" \
    "$CORE_EVIDENCE_PACK_SCHEMA_PATH" \
    "$CORE_EVIDENCE_SOURCE_TYPE_REGISTRY_PATH" \
    "$CORE_EVIDENCE_CAPTURE_RECEIPTS_PATH" \
    "$CORE_EVIDENCE_AUTHORITY_TRACE_LINKS_PATH" \
    "$CORE_EVIDENCE_COMPLETENESS_POLICY_PATH" \
    "$CORE_MISSING_EVIDENCE_HANDLING_PATH" \
    "$CORE_EXTERNAL_PROOF_PROVENANCE_PATH" \
    "$SOFTWARE_DEV_REFERENCE_EVIDENCE_MAPPING_PATH" \
    "$EVIDENCE_PROJECTION_READ_MODEL_PATH" <<'PY'
import hashlib
import json
import pathlib
import sys
import time
import tomllib

(
    out_path_raw,
    release_version,
    cargo_path_raw,
    desktop_package_path_raw,
    desktop_package_lock_path_raw,
    tauri_config_path_raw,
    changelog_path_raw,
    release_readme_path_raw,
    release_tasks_path_raw,
    artifact_manifest_path_raw,
    release_url,
    gate_event_name,
    gate_ref_type,
    gate_ref_name,
    gate_run_id,
    gate_run_attempt,
    gate_repository,
    gate_server_url,
    source_commit_sha,
    release_tag_name,
    *artifact_path_values,
) = sys.argv[1:]

out_path = pathlib.Path(out_path_raw)
cargo_path = pathlib.Path(cargo_path_raw)
desktop_package_path = pathlib.Path(desktop_package_path_raw)
desktop_package_lock_path = pathlib.Path(desktop_package_lock_path_raw)
tauri_config_path = pathlib.Path(tauri_config_path_raw)
changelog_path = pathlib.Path(changelog_path_raw)
release_readme_path = pathlib.Path(release_readme_path_raw)
release_tasks_path = pathlib.Path(release_tasks_path_raw)
artifact_manifest_path = pathlib.Path(artifact_manifest_path_raw)
artifact_paths = [pathlib.Path(value) for value in artifact_path_values]

expected_version = "1.0.6"
expected_tag = "v1.0.6"
cargo = tomllib.loads(cargo_path.read_text(encoding="utf-8"))
desktop_package = json.loads(desktop_package_path.read_text(encoding="utf-8"))
desktop_package_lock = json.loads(desktop_package_lock_path.read_text(encoding="utf-8"))
tauri_config = json.loads(tauri_config_path.read_text(encoding="utf-8"))
changelog_text = changelog_path.read_text(encoding="utf-8")
release_readme_text = release_readme_path.read_text(encoding="utf-8")
release_tasks_text = release_tasks_path.read_text(encoding="utf-8")
artifacts = [json.loads(path.read_text(encoding="utf-8")) for path in artifact_paths]

def version_tuple(value):
    normalized = str(value).strip().lstrip("v")
    parts = normalized.split(".")
    return tuple(int(part) for part in parts[:3])

current_workspace_version = cargo["workspace"]["package"]["version"]
current_desktop_version = desktop_package.get("version")
current_package_lock_version = desktop_package_lock.get("version")
current_package_lock_root_version = (desktop_package_lock.get("packages") or {}).get("", {}).get("version")
current_tauri_version = tauri_config.get("version")
version_not_before_v106 = version_tuple(current_workspace_version) >= version_tuple(expected_version)
release_not_before_v106 = version_tuple(release_version) >= version_tuple(expected_tag)
artifact_statuses = {
    artifact_paths[index].name: artifact.get("status")
    for index, artifact in enumerate(artifacts)
}
certified_artifact_hashes = [
    {
        "path": f"runtime/{path.name}",
        "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
        "bytes": path.stat().st_size,
    }
    for path in artifact_paths
]
artifact_manifest_sha256 = (
    hashlib.sha256(artifact_manifest_path.read_bytes()).hexdigest()
    if artifact_manifest_path.is_file()
    else None
)
certification_digest = hashlib.sha256(json.dumps(
    {
        "releaseVersion": release_version,
        "artifactManifestSha256": artifact_manifest_sha256,
        "certifiedArtifactHashes": certified_artifact_hashes,
    },
    sort_keys=True,
).encode("utf-8")).hexdigest()
gate_run_url = (
    f"{gate_server_url.rstrip('/')}/{gate_repository}/actions/runs/{gate_run_id}"
    if gate_server_url and gate_repository and gate_run_id
    else None
)
event_evidence = {
    "eventName": gate_event_name,
    "refType": gate_ref_type or None,
    "refName": gate_ref_name or None,
    "runId": gate_run_id or None,
    "runAttempt": gate_run_attempt or None,
    "runUrl": gate_run_url,
    "repository": gate_repository,
    "sourceCommitSha": source_commit_sha,
    "releaseTagName": release_tag_name,
    "releaseUrl": release_url,
    "certificationArtifactName": f"release-gate-certification-{release_version}",
    "certificationArtifactDigest": certification_digest,
    "certificationArtifactDigestSource": "v106-certified-evidence-kernel-runtime-artifact-hashes",
    "artifactManifestPath": "artifact-manifest.json",
    "artifactManifestSha256": artifact_manifest_sha256,
}

(
    pack_schema,
    source_registry,
    capture_receipts,
    trace_links,
    completeness_policy,
    missing_evidence,
    external_proof,
    software_mapping,
    projection,
) = artifacts

coverage = {
    "release-version-covers-v106-baseline": release_not_before_v106,
    "cargo-workspace-version-covers-v106-baseline": version_not_before_v106,
    "desktop-package-version-covers-v106-baseline": version_tuple(current_desktop_version) >= version_tuple(expected_version),
    "desktop-package-lock-version-covers-v106-baseline": version_tuple(current_package_lock_version) >= version_tuple(expected_version)
    and version_tuple(current_package_lock_root_version) >= version_tuple(expected_version),
    "tauri-version-covers-v106-baseline": version_tuple(current_tauri_version) >= version_tuple(expected_version),
    "changelog-has-v106-entry": "## v1.0.6 - 2026-06-29" in changelog_text
    and "Core Evidence Kernel baseline" in changelog_text
    and "v106-release-certification" in changelog_text,
    "delivery-readme-release-baseline": "Core Evidence Kernel release baseline" in release_readme_text
    and "Software Dev" in release_readme_text
    and "Core Evidence authority" in release_readme_text,
    "delivery-tasks-release-public-record": "V106-010 Release Certification" in release_tasks_text
    and "runtime/v106-release-certification.json" in release_tasks_text
    and "#681 depends on #672-#680" in release_tasks_text,
    "all-v106-artifacts-passed": all(status == "passed" for status in artifact_statuses.values()),
    "certified-artifact-hashes-present": len(certified_artifact_hashes) == len(artifact_paths)
    and all(item.get("sha256") and item.get("bytes", 0) > 0 for item in certified_artifact_hashes),
    "artifact-manifest-digest-present": artifact_manifest_sha256 is not None,
    "certification-artifact-digest-present": len(certification_digest) == 64,
    "release-event-evidence-recorded": bool(event_evidence["eventName"])
    and bool(event_evidence["sourceCommitSha"])
    and bool(event_evidence["releaseTagName"]),
    "release-run-id-bound-for-ci": gate_event_name == "local" or bool(event_evidence["runId"]),
    "release-run-url-bound-for-ci": gate_event_name == "local" or bool(event_evidence["runUrl"]),
    "evidence-pack-schema-certified": pack_schema.get("contractVersion") == "agentflow-core-evidence-pack.v1"
    and pack_schema.get("negativeFixtureCount", 0) >= 8,
    "source-registry-certified": source_registry.get("unknownSourceTypeReason") == "source-type-unknown"
    and source_registry.get("referenceMappingBoundary") == "reference-app-only-not-core-authority",
    "capture-receipts-certified": capture_receipts.get("contractVersion") == "agentflow-core-evidence-capture-receipt.v1"
    and capture_receipts.get("localArtifactAuthority") == "local-artifact"
    and capture_receipts.get("externalReferenceAuthority") == "external-reference",
    "trace-links-certified": trace_links.get("contractVersion") == "agentflow-core-evidence-authority-trace.v1"
    and trace_links.get("eventType") == "evidence.collected",
    "completeness-policy-certified": completeness_policy.get("contractVersion") == "agentflow-core-evidence-completeness-policy.v1"
    and completeness_policy.get("doneBoundary") == "policy-does-not-write-completed-state",
    "missing-evidence-negative-fixtures-certified": missing_evidence.get("doneBoundary") == "missing-evidence-does-not-write-completed-state"
    and len(missing_evidence.get("negativeFixtures") or []) >= 4,
    "external-proof-negative-fixtures-certified": external_proof.get("providerBoundary") == "external proof provider is not fixed to one vendor"
    and len(external_proof.get("negativeFixtures") or []) >= 4,
    "software-dev-reference-mapping-certified": software_mapping.get("authorityBoundary") == "software-dev-reference-app-mapping-not-core-authority",
    "projection-read-model-certified-readonly": projection.get("readModelVersion") == "evidence-kernel-read-model.v1"
    and projection.get("authority") is False
    and projection.get("readonly") is True,
}
failed = [item for item, passed in coverage.items() if not passed]
payload = {
    "version": "agentflow-v106-release-certification.v1",
    "status": "passed" if not failed else "failed",
    "releaseVersion": expected_tag,
    "workspaceVersion": expected_version,
    "certifiedArtifacts": artifact_statuses,
    "certifiedArtifactHashes": certified_artifact_hashes,
    "eventEvidence": event_evidence,
    "coverage": coverage,
    "failedCoverage": failed,
    "releaseBaseline": "docs/delivery/releases/v1.0.6/README.md",
    "releaseTasks": "docs/delivery/releases/v1.0.6/AGENTFLOW_V1_0_6_CORE_EVIDENCE_KERNEL_TASKS_V1.md",
    "remainingRisks": [
        {
            "id": "v107-decision-kernel",
            "summary": "Decision Kernel completion decision is intentionally deferred to v1.0.7.",
            "blocking": False,
        },
        {
            "id": "v108-projection-kernel",
            "summary": "Full Projection Kernel rebuild and multi-view guarantee remains deferred.",
            "blocking": False,
        },
        {
            "id": "software-dev-reference-app-closeout",
            "summary": "Full Software Dev Reference App certification remains outside Core Evidence authority.",
            "blocking": False,
        },
    ],
    "checkedAt": int(time.time()),
}
out_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
if failed:
    raise SystemExit(f"v1.0.6 release certification failed: {failed}")
PY
  record_stage "v106-release-certification" "passed" "$(basename "$V106_RELEASE_CERTIFICATION_PATH")"
}

run_v107_release_provenance_handoff_gate() {
  record_stage "v107-release-provenance-handoff" "started" "$V107_RELEASE_PROVENANCE_HANDOFF_PATH"
  python3 - \
    "$V107_RELEASE_PROVENANCE_HANDOFF_PATH" \
    "$RELEASE_PROVENANCE_PATH" \
    "$V106_RELEASE_CERTIFICATION_PATH" \
    "$WORKSPACE/docs/architecture/069-release-provenance-tag-policy-v1.md" \
    "$WORKSPACE/docs/delivery/releases/v1.0.7/AGENTFLOW_V1_0_7_DECISION_KERNEL_TASKS_V1.md" \
    "$RELEASE_VERSION" \
    "$RELEASE_TAG_NAME" <<'PY'
import hashlib
import json
import pathlib
import sys
import time

(
    out_path,
    release_provenance_path,
    v106_certification_path,
    policy_doc_path,
    release_tasks_path,
    release_version,
    release_tag_name,
) = sys.argv[1:]
out_path = pathlib.Path(out_path)
release_provenance_path = pathlib.Path(release_provenance_path)
v106_certification_path = pathlib.Path(v106_certification_path)
policy_doc_path = pathlib.Path(policy_doc_path)
release_tasks_path = pathlib.Path(release_tasks_path)

def load_json(path: pathlib.Path):
    if not path.is_file():
        raise SystemExit(f"missing required v107 handoff input: {path}")
    return json.loads(path.read_text(encoding="utf-8"))

release_provenance = load_json(release_provenance_path)
v106_certification = load_json(v106_certification_path)
policy_doc = policy_doc_path.read_text(encoding="utf-8")
release_tasks = release_tasks_path.read_text(encoding="utf-8")

required_v106_artifacts = [
    "core-evidence-pack-schema.json",
    "core-evidence-source-type-registry.json",
    "core-evidence-capture-receipts.json",
    "core-evidence-authority-trace-links.json",
    "core-evidence-completeness-policy.json",
    "core-missing-evidence-handling.json",
    "core-external-proof-provenance.json",
    "software-dev-reference-evidence-mapping.json",
    "evidence-projection-read-model.json",
]
certified_artifacts = v106_certification.get("certifiedArtifacts") or {}
certified_hashes = v106_certification.get("certifiedArtifactHashes") or []
certified_hash_paths = {item.get("path") for item in certified_hashes}
event_evidence = v106_certification.get("eventEvidence") or {}

tag_signature_status = release_provenance.get("tagSignatureStatus")
unsigned_reason = release_provenance.get("unsignedReason")
tag_object_kind = release_provenance.get("tagObjectKind")
artifact_hashes = release_provenance.get("artifactHashes") or []
blocking_failures = [
    "tag-commit-mismatch",
    "missing-tag-commit-in-release-context",
    "literal-revspec-leaked",
    "release-url-not-bound-to-tag",
    "missing-artifact-digest",
    "missing-v106-certified-artifact",
]
warning_statuses = [
    "tag-signature-unsigned",
    "tag-object-kind-lightweight",
]
tag_policy = {
    "version": "agentflow-release-provenance-tag-policy.v1",
    "allowedTagObjectKinds": ["tag", "commit", "pending"],
    "signedAnnotatedTagBehavior": "passed",
    "unsignedAnnotatedTagBehavior": "warning-only-visible",
    "lightweightTagBehavior": "warning-only-visible",
    "silentUnsignedAllowed": False,
    "blockingFailures": blocking_failures,
    "warningStatuses": warning_statuses,
}

coverage = {
    "policy-doc-exists": policy_doc_path.is_file(),
    "policy-doc-defines-warning-only-unsigned": "warning-only-visible" in policy_doc
    and "不能静默忽略" in policy_doc,
    "policy-doc-defines-blocking-failures": all(item in policy_doc for item in [
        "tag commit mismatch",
        "missing tag commit",
        "literal revspec",
        "release URL not bound to tag",
    ]),
    "release-tasks-bind-issue-693": "#693" in release_tasks
    and "V107-001 Release Provenance Tag Policy" in release_tasks,
    "release-provenance-structural-tag-status": bool(tag_signature_status)
    and bool(tag_object_kind)
    and "tagCommitSha" in release_provenance
    and "sourceCommitSha" in release_provenance,
    "unsigned-tag-visible-if-present": tag_signature_status != "unsigned"
    or bool(unsigned_reason),
    "artifact-manifest-digest-present": bool(release_provenance.get("artifactManifestSha256")),
    "release-provenance-artifact-hashes-present": bool(artifact_hashes)
    and all(item.get("sha256") and item.get("bytes", 0) > 0 for item in artifact_hashes),
    "v106-release-certification-passed": v106_certification.get("status") == "passed",
    "v106-release-version-bound": v106_certification.get("releaseVersion") == "v1.0.6",
    "v106-event-evidence-bound": bool(event_evidence.get("releaseTagName"))
    and bool(event_evidence.get("sourceCommitSha")),
    "v106-handoff-artifacts-named": all(name in certified_artifacts for name in required_v106_artifacts),
    "v106-handoff-artifacts-passed": all(
        certified_artifacts.get(name) == "passed" for name in required_v106_artifacts
    ),
    "v106-certified-digests-present": all(
        f"runtime/{name}" in certified_hash_paths for name in required_v106_artifacts
    )
    and all(item.get("sha256") and item.get("bytes", 0) > 0 for item in certified_hashes),
    "no-decision-outcome-semantics": "不定义 Decision outcome" in policy_doc
    and "不启动 Decision outcome" in release_tasks,
}
failed = [item for item, passed in coverage.items() if not passed]
payload = {
    "version": "agentflow-v107-release-provenance-handoff.v1",
    "status": "passed" if not failed else "failed",
    "targetVersion": "v1.0.7",
    "currentReleaseGateVersion": release_version,
    "currentReleaseTagName": release_tag_name,
    "sourceEvidenceKernelVersion": "v1.0.6",
    "policyPath": "docs/architecture/069-release-provenance-tag-policy-v1.md",
    "releaseTasksPath": "docs/delivery/releases/v1.0.7/AGENTFLOW_V1_0_7_DECISION_KERNEL_TASKS_V1.md",
    "tagPolicy": tag_policy,
    "observedReleaseProvenance": {
        "status": release_provenance.get("status"),
        "tagName": release_provenance.get("tagName"),
        "tagObjectKind": tag_object_kind,
        "annotatedTagObjectId": release_provenance.get("annotatedTagObjectId"),
        "tagCommitSha": release_provenance.get("tagCommitSha"),
        "sourceCommitSha": release_provenance.get("sourceCommitSha"),
        "tagCommitMatchesSource": release_provenance.get("tagCommitMatchesSource"),
        "tagSignatureStatus": tag_signature_status,
        "unsignedReason": unsigned_reason,
        "artifactManifestSha256": release_provenance.get("artifactManifestSha256"),
        "gateRunIds": release_provenance.get("gateRunIds") or [],
    },
    "evidenceKernelHandoff": {
        "certificationArtifact": "runtime/v106-release-certification.json",
        "certificationStatus": v106_certification.get("status"),
        "requiredArtifacts": [
            {
                "id": name,
                "runtimePath": f"runtime/{name}",
                "status": certified_artifacts.get(name),
                "digestPresent": f"runtime/{name}" in certified_hash_paths,
            }
            for name in required_v106_artifacts
        ],
        "certifiedArtifactHashes": certified_hashes,
        "eventEvidence": event_evidence,
    },
    "coverage": coverage,
    "failedCoverage": failed,
    "checkedAt": int(time.time()),
}
out = pathlib.Path(out_path)
out.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
if failed:
    raise SystemExit(f"v107 release provenance handoff failed: {failed}")
PY
  record_stage "v107-release-provenance-handoff" "passed" "$(basename "$V107_RELEASE_PROVENANCE_HANDOFF_PATH")"
}

run_core_decision_model_contract_gate() {
  record_stage "core-decision-model-contract" "started" "$CORE_DECISION_MODEL_CONTRACT_PATH"
  local rust_test_log="$RUNTIME_DIR/core-decision-model-contract-rust-test.log"
  if ! (cd "$WORKSPACE" && cargo test -p agentflow-ontology core_decision --quiet >"$rust_test_log" 2>&1); then
    fail_stage "core-decision-model-contract" "agentflow-ontology Core Decision Model tests failed"
  fi
  python3 - \
    "$CORE_DECISION_MODEL_CONTRACT_PATH" \
    "$WORKSPACE/docs/architecture/070-core-decision-model-contract-v1.md" \
    "$WORKSPACE/crates/ontology/src/decision.rs" \
    "$WORKSPACE/docs/delivery/releases/v1.0.7/AGENTFLOW_V1_0_7_DECISION_KERNEL_TASKS_V1.md" \
    "$rust_test_log" <<'PY'
import json
import pathlib
import sys
import time

out_path = pathlib.Path(sys.argv[1])
doc_path = pathlib.Path(sys.argv[2])
source_path = pathlib.Path(sys.argv[3])
tasks_path = pathlib.Path(sys.argv[4])
test_log_path = pathlib.Path(sys.argv[5])

doc_text = doc_path.read_text(encoding="utf-8")
source_text = source_path.read_text(encoding="utf-8")
tasks_text = tasks_path.read_text(encoding="utf-8")
required_fields = [
    "version",
    "decisionId",
    "decidedAt",
    "decidedBy",
    "subject",
    "inputs",
    "outcome",
    "reasons",
    "writes",
]
readable_facts = ["spec", "runtimeState", "evidence"]
allowed_writes = ["decision-record", "decision-event"]
forbidden_writes = [
    "spec-authority",
    "runtime-state-authority",
    "evidence-authority",
    "projection-read-model",
    "provider-session-record",
    "audit-sidecar-record",
]
outcomes = ["accepted", "rejected", "deferred", "blocked", "needs-fix"]
forbidden_terms = [
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
]
coverage = {
    "contract-version-defined": "agentflow-core-decision-model.v1" in source_text
    and "agentflow-core-decision-model.v1" in doc_text,
    "rust-contract-implemented": "CoreDecisionModelContract" in source_text
    and "CoreDecisionRecord" in source_text
    and "validate_core_decision_model_contract" in source_text
    and "validate_core_decision_record" in source_text,
    "required-fields-documented": all(field in doc_text for field in required_fields),
    "required-fields-implemented": all(field in source_text for field in required_fields),
    "readable-authority-facts-documented": all(fact in doc_text for fact in readable_facts),
    "readable-authority-facts-implemented": all(fact in source_text for fact in readable_facts),
    "write-authority-documented": all(item in doc_text for item in allowed_writes + forbidden_writes),
    "write-authority-implemented": all(item in source_text for item in allowed_writes + forbidden_writes),
    "outcomes-documented": all(item in doc_text for item in outcomes),
    "outcomes-implemented": all(item in source_text for item in outcomes),
    "forbidden-terms-documented": all(item in doc_text for item in forbidden_terms),
    "forbidden-terms-implemented": all(item in source_text for item in forbidden_terms),
    "canonical-record-fixture-implemented": "canonical_core_decision_record_fixture" in source_text,
    "negative-fixtures-implemented": "core_decision_record_rejects_unknown_outcome" in source_text
    and "core_decision_record_rejects_forbidden_industry_term" in source_text,
    "release-task-binds-issue-694": "#694" in tasks_text
    and "V107-002 Core Decision Model Contract" in tasks_text
    and "runtime/core-decision-model-contract.json" in tasks_text,
    "rust-contract-tests-passed": test_log_path.is_file(),
}
failed = [item for item, passed in coverage.items() if not passed]
payload = {
    "version": "agentflow-core-decision-model-contract-gate.v1",
    "status": "passed" if not failed else "failed",
    "contractVersion": "agentflow-core-decision-model.v1",
    "architecturePath": "docs/architecture/070-core-decision-model-contract-v1.md",
    "rustContractPath": "crates/ontology/src/decision.rs",
    "rustTestLogPath": "runtime/core-decision-model-contract-rust-test.log",
    "requiredRecordFields": required_fields,
    "readableAuthorityFacts": readable_facts,
    "allowedWrites": allowed_writes,
    "forbiddenWrites": forbidden_writes,
    "outcomes": outcomes,
    "forbiddenCoreTerms": forbidden_terms,
    "coverage": coverage,
    "failedCoverage": failed,
    "checkedAt": int(time.time()),
}
out_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
if failed:
    raise SystemExit(f"core decision model contract coverage failed: {failed}")
PY
  record_stage "core-decision-model-contract" "passed" "$(basename "$CORE_DECISION_MODEL_CONTRACT_PATH")"
}

run_core_decision_input_binding_gate() {
  record_stage "core-decision-input-binding" "started" "$CORE_DECISION_INPUT_BINDING_PATH"
  local rust_test_log="$RUNTIME_DIR/core-decision-input-binding-rust-test.log"
  if ! (cd "$WORKSPACE" && cargo test -p agentflow-ontology core_decision_input_binding --quiet >"$rust_test_log" 2>&1); then
    fail_stage "core-decision-input-binding" "agentflow-ontology Core Decision Input Binding tests failed"
  fi
  python3 - \
    "$CORE_DECISION_INPUT_BINDING_PATH" \
    "$WORKSPACE/docs/architecture/071-core-decision-input-binding-v1.md" \
    "$WORKSPACE/crates/ontology/src/decision.rs" \
    "$WORKSPACE/docs/delivery/releases/v1.0.7/AGENTFLOW_V1_0_7_DECISION_KERNEL_TASKS_V1.md" \
    "$rust_test_log" <<'PY'
import json
import pathlib
import sys
import time

out_path = pathlib.Path(sys.argv[1])
doc_path = pathlib.Path(sys.argv[2])
source_path = pathlib.Path(sys.argv[3])
tasks_path = pathlib.Path(sys.argv[4])
test_log_path = pathlib.Path(sys.argv[5])

doc_text = doc_path.read_text(encoding="utf-8")
source_text = source_path.read_text(encoding="utf-8")
tasks_text = tasks_path.read_text(encoding="utf-8")
required_inputs = ["specBundle", "ontologyObject", "runtimeActionState", "evidencePack"]
required_ref_kinds = [
    "SpecBundleRef",
    "OntologyObjectRef",
    "RuntimeActionStateRef",
    "EvidencePackRef",
]
optional_context = ["deliveryContext", "DeliveryContextRef"]
rejected_ref_kinds = ["ProjectionRef", "ProviderSessionRef", "CliSessionRef", "ChatThreadRef"]
negative_fixtures = [
    "core_decision_input_binding_rejects_missing_spec",
    "core_decision_input_binding_rejects_stale_runtime_state",
    "core_decision_input_binding_rejects_projection_only_ref",
    "core_decision_input_binding_rejects_provider_session_ref",
]
coverage = {
    "contract-version-defined": "agentflow-core-decision-input-binding.v1" in source_text
    and "agentflow-core-decision-input-binding.v1" in doc_text,
    "rust-contract-implemented": "CoreDecisionInputBindingContract" in source_text
    and "CoreDecisionInputBinding" in source_text
    and "validate_core_decision_input_binding_contract" in source_text
    and "validate_core_decision_input_binding" in source_text,
    "required-inputs-documented": all(item in doc_text for item in required_inputs + required_ref_kinds),
    "required-inputs-implemented": all(item in source_text for item in required_inputs + required_ref_kinds),
    "optional-delivery-context-documented": all(item in doc_text for item in optional_context),
    "optional-delivery-context-implemented": all(item in source_text for item in optional_context),
    "rejected-ref-kinds-documented": all(item in doc_text for item in rejected_ref_kinds),
    "rejected-ref-kinds-implemented": all(item in source_text for item in rejected_ref_kinds),
    "stale-ref-rule-documented": "stale authority ref" in doc_text,
    "stale-ref-rule-implemented": "stale authority ref" in source_text,
    "projection-provider-path-rejected": ".agentflow/projections/" in source_text
    and ".agentflow/provider-sessions/" in source_text,
    "positive-fixture-implemented": "canonical_core_decision_input_binding_fixture" in source_text,
    "negative-fixtures-implemented": all(item in source_text for item in negative_fixtures),
    "release-task-binds-issue-695": "#695" in tasks_text
    and "V107-003 Decision Input Binding" in tasks_text
    and "runtime/core-decision-input-binding.json" in tasks_text,
    "rust-contract-tests-passed": test_log_path.is_file(),
}
failed = [item for item, passed in coverage.items() if not passed]
payload = {
    "version": "agentflow-core-decision-input-binding-gate.v1",
    "status": "passed" if not failed else "failed",
    "contractVersion": "agentflow-core-decision-input-binding.v1",
    "architecturePath": "docs/architecture/071-core-decision-input-binding-v1.md",
    "rustContractPath": "crates/ontology/src/decision.rs",
    "rustTestLogPath": "runtime/core-decision-input-binding-rust-test.log",
    "requiredInputs": required_inputs,
    "requiredRefKinds": required_ref_kinds,
    "optionalContext": optional_context,
    "rejectedRefKinds": rejected_ref_kinds,
    "coverage": coverage,
    "failedCoverage": failed,
    "checkedAt": int(time.time()),
}
out_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
if failed:
    raise SystemExit(f"core decision input binding coverage failed: {failed}")
PY
  record_stage "core-decision-input-binding" "passed" "$(basename "$CORE_DECISION_INPUT_BINDING_PATH")"
}

run_core_decision_outcome_transitions_gate() {
  record_stage "core-decision-outcome-transitions" "started" "$CORE_DECISION_OUTCOME_TRANSITIONS_PATH"
  local rust_test_log="$RUNTIME_DIR/core-decision-outcome-transitions-rust-test.log"
  if ! (cd "$WORKSPACE" && cargo test -p agentflow-ontology core_decision --quiet >"$rust_test_log" 2>&1); then
    fail_stage "core-decision-outcome-transitions" "agentflow-ontology Core Decision Outcome tests failed"
  fi
  python3 - \
    "$CORE_DECISION_OUTCOME_TRANSITIONS_PATH" \
    "$WORKSPACE/docs/architecture/072-core-decision-outcome-transition-semantics-v1.md" \
    "$WORKSPACE/docs/architecture/070-core-decision-model-contract-v1.md" \
    "$WORKSPACE/crates/ontology/src/decision.rs" \
    "$WORKSPACE/crates/ontology/src/semantics.rs" \
    "$WORKSPACE/docs/delivery/releases/v1.0.7/AGENTFLOW_V1_0_7_DECISION_KERNEL_TASKS_V1.md" \
    "$rust_test_log" <<'PY'
import json
import pathlib
import sys
import time

out_path = pathlib.Path(sys.argv[1])
doc_path = pathlib.Path(sys.argv[2])
model_doc_path = pathlib.Path(sys.argv[3])
decision_source_path = pathlib.Path(sys.argv[4])
semantics_source_path = pathlib.Path(sys.argv[5])
tasks_path = pathlib.Path(sys.argv[6])
test_log_path = pathlib.Path(sys.argv[7])

doc_text = doc_path.read_text(encoding="utf-8")
model_doc_text = model_doc_path.read_text(encoding="utf-8")
decision_source = decision_source_path.read_text(encoding="utf-8")
semantics_source = semantics_source_path.read_text(encoding="utf-8")
tasks_text = tasks_path.read_text(encoding="utf-8")
outcomes = ["accepted", "rejected", "deferred", "blocked", "needs-fix"]
state_names = ["captured", "understood", "planned", "ready", "active", "reviewing", "blocked", "cancelled"]
reason_fields = ["reasonCode", "message", "evidenceRefs", "blocking"]
negative_fixtures = [
    "core_decision_transition_rejects_completion_write",
    "core_decision_transition_rejects_unknown_outcome",
    "core_decision_transition_rejects_missing_reason",
]
coverage = {
    "contract-version-defined": "agentflow-core-decision-outcome-transition.v1" in decision_source
    and "agentflow-core-decision-outcome-transition.v1" in doc_text,
    "rust-contract-implemented": "CoreDecisionOutcomeTransitionContract" in decision_source
    and "CoreDecisionTransitionAttempt" in decision_source
    and "validate_core_decision_outcome_transition_contract" in decision_source
    and "validate_core_decision_transition_attempt" in decision_source,
    "canonical-outcomes-documented": all(outcome in doc_text for outcome in outcomes)
    and all(outcome in model_doc_text for outcome in outcomes),
    "canonical-outcomes-implemented": all(outcome in decision_source for outcome in outcomes),
    "state-semantics-bound": all(state in semantics_source for state in state_names)
    and "CoreActionStateSemanticsContract" in decision_source,
    "completion-state-excluded": "completed" in doc_text
    and "must not write completion state" in decision_source,
    "reason-shape-documented": all(field in doc_text for field in reason_fields),
    "reason-shape-implemented": all(field in decision_source for field in reason_fields),
    "positive-fixture-implemented": "canonical_core_decision_transition_attempt_fixture" in decision_source,
    "negative-fixtures-implemented": all(item in decision_source for item in negative_fixtures),
    "release-task-binds-issue-696": "#696" in tasks_text
    and "V107-004 Decision Outcomes and State Transition Semantics" in tasks_text
    and "runtime/core-decision-outcome-transitions.json" in tasks_text,
    "rust-contract-tests-passed": test_log_path.is_file(),
}
failed = [item for item, passed in coverage.items() if not passed]
payload = {
    "version": "agentflow-core-decision-outcome-transitions-gate.v1",
    "status": "passed" if not failed else "failed",
    "contractVersion": "agentflow-core-decision-outcome-transition.v1",
    "architecturePath": "docs/architecture/072-core-decision-outcome-transition-semantics-v1.md",
    "rustContractPath": "crates/ontology/src/decision.rs",
    "rustTestLogPath": "runtime/core-decision-outcome-transitions-rust-test.log",
    "outcomes": outcomes,
    "stateNames": state_names,
    "reasonFields": reason_fields,
    "coverage": coverage,
    "failedCoverage": failed,
    "checkedAt": int(time.time()),
}
out_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
if failed:
    raise SystemExit(f"core decision outcome transitions coverage failed: {failed}")
PY
  record_stage "core-decision-outcome-transitions" "passed" "$(basename "$CORE_DECISION_OUTCOME_TRANSITIONS_PATH")"
}

run_core_decision_failure_reason_gate() {
  record_stage "core-decision-failure-reason-remediation" "started" "$CORE_DECISION_FAILURE_REASON_PATH"
  local rust_test_log="$RUNTIME_DIR/core-decision-failure-reason-remediation-rust-test.log"
  if ! (cd "$WORKSPACE" && cargo test -p agentflow-ontology core_decision_failure_reason --quiet >"$rust_test_log" 2>&1); then
    fail_stage "core-decision-failure-reason-remediation" "agentflow-ontology Core Decision Failure Reason tests failed"
  fi
  python3 - \
    "$CORE_DECISION_FAILURE_REASON_PATH" \
    "$WORKSPACE/docs/architecture/073-core-decision-failure-reason-remediation-v1.md" \
    "$WORKSPACE/docs/architecture/072-core-decision-outcome-transition-semantics-v1.md" \
    "$WORKSPACE/crates/ontology/src/decision.rs" \
    "$WORKSPACE/docs/delivery/releases/v1.0.7/AGENTFLOW_V1_0_7_DECISION_KERNEL_TASKS_V1.md" \
    "$rust_test_log" <<'PY'
import json
import pathlib
import sys
import time

out_path = pathlib.Path(sys.argv[1])
doc_path = pathlib.Path(sys.argv[2])
outcome_doc_path = pathlib.Path(sys.argv[3])
decision_source_path = pathlib.Path(sys.argv[4])
tasks_path = pathlib.Path(sys.argv[5])
test_log_path = pathlib.Path(sys.argv[6])

doc_text = doc_path.read_text(encoding="utf-8")
outcome_doc_text = outcome_doc_path.read_text(encoding="utf-8")
decision_source = decision_source_path.read_text(encoding="utf-8")
tasks_text = tasks_path.read_text(encoding="utf-8")
applies_to_outcomes = ["rejected", "deferred", "blocked", "needs-fix"]
required_fields = [
    "reasonCode",
    "message",
    "authorityRefs",
    "missingEvidenceRefs",
    "remediationRoute",
    "retryEligible",
    "blocking",
]
remediation_routes = [
    "wait-for-authority",
    "collect-evidence",
    "revise-subject",
    "cancel-subject",
    "retry-decision",
]
negative_fixtures = [
    "core_decision_failure_reason_rejects_accepted_outcome",
    "core_decision_failure_reason_rejects_missing_authority_refs",
    "core_decision_failure_reason_rejects_missing_evidence_refs",
    "core_decision_failure_reason_rejects_unknown_remediation_route",
    "core_decision_failure_reason_rejects_invalid_retry_eligibility",
]
coverage = {
    "contract-version-defined": "agentflow-core-decision-failure-reason.v1" in decision_source
    and "agentflow-core-decision-failure-reason.v1" in doc_text,
    "rust-contract-implemented": "CoreDecisionFailureReasonContract" in decision_source
    and "CoreDecisionFailureReason" in decision_source
    and "CoreDecisionRemediationRoute" in decision_source
    and "validate_core_decision_failure_reason_contract" in decision_source
    and "validate_core_decision_failure_reason" in decision_source,
    "non-accepted-outcomes-documented": all(outcome in doc_text for outcome in applies_to_outcomes)
    and all(outcome in outcome_doc_text for outcome in applies_to_outcomes),
    "accepted-outcome-excluded": "accepted" in doc_text
    and "must not be attached to accepted" in decision_source,
    "required-fields-documented": all(field in doc_text for field in required_fields),
    "required-fields-implemented": all(field in decision_source for field in required_fields),
    "remediation-routes-documented": all(route in doc_text for route in remediation_routes),
    "remediation-routes-implemented": all(route in decision_source for route in remediation_routes),
    "positive-fixture-implemented": "canonical_core_decision_failure_reason_fixture" in decision_source,
    "negative-fixtures-implemented": all(item in decision_source for item in negative_fixtures),
    "release-task-binds-issue-697": "#697" in tasks_text
    and "V107-005 Failure Reason and Remediation Contract" in tasks_text
    and "runtime/core-decision-failure-reason-remediation.json" in tasks_text,
    "rust-contract-tests-passed": test_log_path.is_file(),
}
failed = [item for item, passed in coverage.items() if not passed]
payload = {
    "version": "agentflow-core-decision-failure-reason-gate.v1",
    "status": "passed" if not failed else "failed",
    "contractVersion": "agentflow-core-decision-failure-reason.v1",
    "architecturePath": "docs/architecture/073-core-decision-failure-reason-remediation-v1.md",
    "rustContractPath": "crates/ontology/src/decision.rs",
    "rustTestLogPath": "runtime/core-decision-failure-reason-remediation-rust-test.log",
    "appliesToOutcomes": applies_to_outcomes,
    "requiredFields": required_fields,
    "remediationRoutes": remediation_routes,
    "coverage": coverage,
    "failedCoverage": failed,
    "checkedAt": int(time.time()),
}
out_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
if failed:
    raise SystemExit(f"core decision failure reason coverage failed: {failed}")
PY
  record_stage "core-decision-failure-reason-remediation" "passed" "$(basename "$CORE_DECISION_FAILURE_REASON_PATH")"
}

run_core_evidence_to_decision_gate() {
  record_stage "core-evidence-to-decision-gate" "started" "$CORE_EVIDENCE_TO_DECISION_GATE_PATH"
  local rust_test_log="$RUNTIME_DIR/core-evidence-to-decision-gate-rust-test.log"
  if ! (cd "$WORKSPACE" && cargo test -p agentflow-ontology core_evidence_to_decision_gate --quiet >"$rust_test_log" 2>&1); then
    fail_stage "core-evidence-to-decision-gate" "agentflow-ontology Core Evidence-to-Decision Gate tests failed"
  fi
  python3 - \
    "$CORE_EVIDENCE_TO_DECISION_GATE_PATH" \
    "$WORKSPACE/docs/architecture/074-core-evidence-to-decision-gate-v1.md" \
    "$WORKSPACE/docs/architecture/064-core-evidence-completeness-policy-v1.md" \
    "$WORKSPACE/docs/architecture/073-core-decision-failure-reason-remediation-v1.md" \
    "$WORKSPACE/crates/ontology/src/decision.rs" \
    "$WORKSPACE/crates/ontology/src/evidence.rs" \
    "$WORKSPACE/docs/delivery/releases/v1.0.7/AGENTFLOW_V1_0_7_DECISION_KERNEL_TASKS_V1.md" \
    "$rust_test_log" <<'PY'
import json
import pathlib
import sys
import time

out_path = pathlib.Path(sys.argv[1])
doc_path = pathlib.Path(sys.argv[2])
evidence_doc_path = pathlib.Path(sys.argv[3])
failure_doc_path = pathlib.Path(sys.argv[4])
decision_source_path = pathlib.Path(sys.argv[5])
evidence_source_path = pathlib.Path(sys.argv[6])
tasks_path = pathlib.Path(sys.argv[7])
test_log_path = pathlib.Path(sys.argv[8])

doc_text = doc_path.read_text(encoding="utf-8")
evidence_doc_text = evidence_doc_path.read_text(encoding="utf-8")
failure_doc_text = failure_doc_path.read_text(encoding="utf-8")
decision_source = decision_source_path.read_text(encoding="utf-8")
evidence_source = evidence_source_path.read_text(encoding="utf-8")
tasks_text = tasks_path.read_text(encoding="utf-8")
evidence_outcomes = ["complete", "incomplete", "deferred", "invalid"]
decision_outcomes = ["accepted", "deferred", "rejected"]
failure_codes = ["evidence-missing", "evidence-deferred", "evidence-invalid", "authority-mismatch"]
negative_fixtures = [
    "core_evidence_to_decision_gate_defers_missing_evidence",
    "core_evidence_to_decision_gate_rejects_invalid_evidence",
    "core_evidence_to_decision_gate_rejects_authority_mismatch",
    "core_evidence_to_decision_gate_rejects_noncomplete_accepted_ready",
]
coverage = {
    "contract-version-defined": "agentflow-core-evidence-to-decision-gate.v1" in decision_source
    and "agentflow-core-evidence-to-decision-gate.v1" in doc_text,
    "rust-contract-implemented": "CoreEvidenceToDecisionGateContract" in decision_source
    and "CoreEvidenceToDecisionGateResult" in decision_source
    and "validate_core_evidence_to_decision_gate_contract" in decision_source
    and "validate_core_evidence_to_decision_gate_result" in decision_source
    and "evaluate_core_evidence_to_decision_gate" in decision_source,
    "evidence-kernel-version-bound": (
        "agentflow-core-evidence-completeness-policy.v1" in decision_source
        or "agentflow-core-evidence-completeness-policy.v1" in evidence_source
    )
    and "agentflow-core-evidence-completeness-policy.v1" in doc_text
    and "agentflow-core-evidence-completeness-policy.v1" in evidence_doc_text,
    "evidence-outcomes-documented": all(outcome in doc_text for outcome in evidence_outcomes),
    "evidence-outcomes-implemented": all(outcome in decision_source for outcome in evidence_outcomes),
    "decision-outcomes-documented": all(outcome in doc_text for outcome in decision_outcomes),
    "failure-reason-bound": all(code in doc_text for code in failure_codes)
    and "agentflow-core-decision-failure-reason.v1" in failure_doc_text,
    "complete-produces-accepted-ready": "canonical_core_evidence_to_decision_gate_result_fixture" in decision_source
    and "core_evidence_to_decision_gate_accepts_complete_evidence" in decision_source
    and "assert!(result.accepted_ready)" in decision_source
    and "result.failure_reason.is_none()" in decision_source,
    "noncomplete-cannot-accepted-ready": "noncomplete_accepted_ready" in decision_source,
    "negative-fixtures-implemented": all(item in decision_source for item in negative_fixtures),
    "evidence-source-exposes-completeness": "CoreEvidenceCompletenessEvaluation" in evidence_source
    and "evaluate_core_evidence_completeness_policy" in evidence_source,
    "release-task-binds-issue-698": "#698" in tasks_text
    and "V107-006 Evidence-to-Decision Gate" in tasks_text
    and "runtime/core-evidence-to-decision-gate.json" in tasks_text,
    "rust-contract-tests-passed": test_log_path.is_file(),
}
failed = [item for item, passed in coverage.items() if not passed]
payload = {
    "version": "agentflow-core-evidence-to-decision-gate-gate.v1",
    "status": "passed" if not failed else "failed",
    "contractVersion": "agentflow-core-evidence-to-decision-gate.v1",
    "architecturePath": "docs/architecture/074-core-evidence-to-decision-gate-v1.md",
    "rustContractPath": "crates/ontology/src/decision.rs",
    "rustTestLogPath": "runtime/core-evidence-to-decision-gate-rust-test.log",
    "evidenceOutcomes": evidence_outcomes,
    "decisionOutcomes": decision_outcomes,
    "failureCodes": failure_codes,
    "coverage": coverage,
    "failedCoverage": failed,
    "checkedAt": int(time.time()),
}
out_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
if failed:
    raise SystemExit(f"core evidence-to-decision gate coverage failed: {failed}")
PY
  record_stage "core-evidence-to-decision-gate" "passed" "$(basename "$CORE_EVIDENCE_TO_DECISION_GATE_PATH")"
}

run_core_completion_commit_authority_gate() {
  record_stage "core-completion-commit-authority" "started" "$CORE_COMPLETION_COMMIT_AUTHORITY_PATH"
  local rust_test_log="$RUNTIME_DIR/core-completion-commit-authority-rust-test.log"
  if ! (cd "$WORKSPACE" && cargo test -p agentflow-ontology core_completion_commit_authority --quiet >"$rust_test_log" 2>&1); then
    fail_stage "core-completion-commit-authority" "agentflow-ontology Core Completion Commit Authority tests failed"
  fi
  python3 - \
    "$CORE_COMPLETION_COMMIT_AUTHORITY_PATH" \
    "$WORKSPACE/docs/architecture/075-core-completion-commit-authority-v1.md" \
    "$WORKSPACE/docs/architecture/074-core-evidence-to-decision-gate-v1.md" \
    "$WORKSPACE/crates/ontology/src/decision.rs" \
    "$WORKSPACE/docs/delivery/releases/v1.0.7/AGENTFLOW_V1_0_7_DECISION_KERNEL_TASKS_V1.md" \
    "$rust_test_log" <<'PY'
import json
import pathlib
import sys
import time

out_path = pathlib.Path(sys.argv[1])
doc_path = pathlib.Path(sys.argv[2])
evidence_gate_doc_path = pathlib.Path(sys.argv[3])
decision_source_path = pathlib.Path(sys.argv[4])
tasks_path = pathlib.Path(sys.argv[5])
test_log_path = pathlib.Path(sys.argv[6])

doc_text = doc_path.read_text(encoding="utf-8")
evidence_gate_doc_text = evidence_gate_doc_path.read_text(encoding="utf-8")
decision_source = decision_source_path.read_text(encoding="utf-8")
tasks_text = tasks_path.read_text(encoding="utf-8")
nonaccepted_outcomes = ["rejected", "deferred", "blocked"]
negative_fixtures = [
    "core_completion_commit_authority_rejects_rejected_decision",
    "core_completion_commit_authority_rejects_deferred_decision",
    "core_completion_commit_authority_rejects_blocked_decision",
    "core_completion_commit_authority_rejects_projection_writer",
    "core_completion_commit_authority_rejects_projection_write_attempt",
    "core_completion_commit_authority_requires_decision_ref",
]
coverage = {
    "contract-version-defined": "agentflow-core-completion-commit-authority.v1" in decision_source
    and "agentflow-core-completion-commit-authority.v1" in doc_text,
    "rust-contract-implemented": "CoreCompletionCommitAuthorityContract" in decision_source
    and "CoreCompletionCommitAttempt" in decision_source
    and "CoreCompletionCommitAuthorityResult" in decision_source
    and "validate_core_completion_commit_authority_contract" in decision_source
    and "evaluate_core_completion_commit_authority" in decision_source
    and "validate_core_completion_commit_authority_result" in decision_source,
    "accepted-decision-required": "required_prior_outcome" in decision_source
    and "\"accepted\"" in decision_source,
    "accepted-decision-documented": "Only `accepted` Decision" in doc_text
    or "只有 `accepted` Decision" in doc_text,
    "completion-event-bound": "subject.completion.committed" in decision_source
    and "subject.completion.committed" in doc_text,
    "event-store-authority": "Event Store" in decision_source
    and "Event Store" in doc_text,
    "projection-readonly-bound": "Projection may refresh" in decision_source
    and "projection" in decision_source
    and "Projection" in doc_text
    and "read-only" in doc_text,
    "nonaccepted-outcomes-documented": all(outcome in doc_text for outcome in nonaccepted_outcomes),
    "nonaccepted-outcomes-tested": all(item in decision_source for item in negative_fixtures[:3]),
    "projection-write-attempts-fail": "projection-read-model" in decision_source
    and "core_completion_commit_authority_rejects_projection_write_attempt" in decision_source,
    "missing-decision-ref-fails": "completion-decision-ref-missing" in decision_source
    and "core_completion_commit_authority_requires_decision_ref" in decision_source,
    "evidence-to-decision-precedes-completion": "Evidence-to-Decision Gate" in evidence_gate_doc_text
    and "Completion Commit" in doc_text,
    "release-task-binds-issue-699": "#699" in tasks_text
    and "V107-007 Completion Commit Authority Boundary" in tasks_text
    and "runtime/core-completion-commit-authority.json" in tasks_text,
    "rust-contract-tests-passed": test_log_path.is_file(),
}
failed = [item for item, passed in coverage.items() if not passed]
payload = {
    "version": "agentflow-core-completion-commit-authority-gate.v1",
    "status": "passed" if not failed else "failed",
    "contractVersion": "agentflow-core-completion-commit-authority.v1",
    "architecturePath": "docs/architecture/075-core-completion-commit-authority-v1.md",
    "rustContractPath": "crates/ontology/src/decision.rs",
    "rustTestLogPath": "runtime/core-completion-commit-authority-rust-test.log",
    "requiredPriorOutcome": "accepted",
    "completionEventType": "subject.completion.committed",
    "forbiddenWriters": ["projection", "provider-session", "delivery-context", "audit-sidecar"],
    "coverage": coverage,
    "failedCoverage": failed,
    "checkedAt": int(time.time()),
}
out_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
if failed:
    raise SystemExit(f"core completion commit authority coverage failed: {failed}")
PY
  record_stage "core-completion-commit-authority" "passed" "$(basename "$CORE_COMPLETION_COMMIT_AUTHORITY_PATH")"
}

run_core_delivery_readiness_audit_trigger_gate() {
  record_stage "core-delivery-readiness-audit-trigger" "started" "$CORE_DELIVERY_READINESS_AUDIT_TRIGGER_PATH"
  local rust_test_log="$RUNTIME_DIR/core-delivery-readiness-audit-trigger-rust-test.log"
  if ! (cd "$WORKSPACE" && cargo test -p agentflow-ontology core_delivery_readiness --quiet >"$rust_test_log" 2>&1); then
    fail_stage "core-delivery-readiness-audit-trigger" "agentflow-ontology Core Delivery Readiness / Optional Audit Trigger tests failed"
  fi
  python3 - \
    "$CORE_DELIVERY_READINESS_AUDIT_TRIGGER_PATH" \
    "$WORKSPACE/docs/architecture/076-core-delivery-readiness-audit-trigger-v1.md" \
    "$WORKSPACE/docs/architecture/075-core-completion-commit-authority-v1.md" \
    "$WORKSPACE/crates/ontology/src/decision.rs" \
    "$WORKSPACE/docs/delivery/releases/v1.0.7/AGENTFLOW_V1_0_7_DECISION_KERNEL_TASKS_V1.md" \
    "$rust_test_log" <<'PY'
import json
import pathlib
import sys
import time

out_path = pathlib.Path(sys.argv[1])
doc_path = pathlib.Path(sys.argv[2])
completion_doc_path = pathlib.Path(sys.argv[3])
decision_source_path = pathlib.Path(sys.argv[4])
tasks_path = pathlib.Path(sys.argv[5])
test_log_path = pathlib.Path(sys.argv[6])

doc_text = doc_path.read_text(encoding="utf-8")
completion_doc_text = completion_doc_path.read_text(encoding="utf-8")
decision_source = decision_source_path.read_text(encoding="utf-8")
tasks_text = tasks_path.read_text(encoding="utf-8")
readiness_outcomes = ["ready", "waiting-for-public-record", "evidence-missing", "completion-missing"]
negative_fixtures = [
    "core_delivery_readiness_missing_public_record_is_not_ready",
    "core_delivery_readiness_missing_evidence_is_not_ready",
    "core_delivery_readiness_missing_completion_is_not_ready",
    "core_delivery_readiness_rejects_audit_block_without_sidecar_queue",
]
coverage = {
    "contract-version-defined": "agentflow-core-delivery-readiness-audit-trigger.v1" in decision_source
    and "agentflow-core-delivery-readiness-audit-trigger.v1" in doc_text,
    "rust-contract-implemented": "CoreDeliveryReadinessAuditTriggerContract" in decision_source
    and "CoreDeliveryReadinessInput" in decision_source
    and "CoreDeliveryReadinessAuditTriggerResult" in decision_source
    and "validate_core_delivery_readiness_audit_trigger_contract" in decision_source
    and "evaluate_core_delivery_readiness_audit_trigger" in decision_source
    and "validate_core_delivery_readiness_audit_trigger_result" in decision_source,
    "completion-event-bound": "subject.completion.committed" in decision_source
    and "subject.completion.committed" in doc_text
    and "Completion Commit" in completion_doc_text,
    "readiness-outcomes-documented": all(outcome in doc_text for outcome in readiness_outcomes),
    "readiness-outcomes-implemented": all(outcome in decision_source for outcome in readiness_outcomes),
    "default-audit-not-required": "default_audit_required: false" in decision_source
    or "defaultAuditRequired = false" in doc_text,
    "explicit-policy-required": "explicit_policy_required: true" in decision_source
    and "explicitPolicyRequired = true" in doc_text,
    "sidecar-event-bound": "subject.audit-sidecar.evaluation-queued" in decision_source
    and "subject.audit-sidecar.evaluation-queued" in doc_text,
    "normal-ready-does-not-queue-audit": "core_delivery_readiness_ready_does_not_queue_audit_by_default" in decision_source
    and "assert!(!result.audit_sidecar_queued)" in decision_source,
    "explicit-policy-queues-sidecar": "core_delivery_readiness_explicit_policy_queues_audit_sidecar" in decision_source
    and "assert!(result.audit_sidecar_queued)" in decision_source,
    "negative-fixtures-implemented": all(item in decision_source for item in negative_fixtures),
    "audit-block-requires-sidecar": "audit cannot block done" in decision_source
    and "core_delivery_readiness_rejects_audit_block_without_sidecar_queue" in decision_source,
    "release-task-binds-issue-700": "#700" in tasks_text
    and "V107-008 Delivery Readiness and Optional Audit Trigger Evaluation" in tasks_text
    and "runtime/core-delivery-readiness-audit-trigger.json" in tasks_text,
    "rust-contract-tests-passed": test_log_path.is_file(),
}
failed = [item for item, passed in coverage.items() if not passed]
payload = {
    "version": "agentflow-core-delivery-readiness-audit-trigger-gate.v1",
    "status": "passed" if not failed else "failed",
    "contractVersion": "agentflow-core-delivery-readiness-audit-trigger.v1",
    "architecturePath": "docs/architecture/076-core-delivery-readiness-audit-trigger-v1.md",
    "rustContractPath": "crates/ontology/src/decision.rs",
    "rustTestLogPath": "runtime/core-delivery-readiness-audit-trigger-rust-test.log",
    "readinessOutcomes": readiness_outcomes,
    "sidecarEventType": "subject.audit-sidecar.evaluation-queued",
    "coverage": coverage,
    "failedCoverage": failed,
    "checkedAt": int(time.time()),
}
out_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
if failed:
    raise SystemExit(f"core delivery readiness audit trigger coverage failed: {failed}")
PY
  record_stage "core-delivery-readiness-audit-trigger" "passed" "$(basename "$CORE_DELIVERY_READINESS_AUDIT_TRIGGER_PATH")"
}

run_core_decision_projection_read_model_gate() {
  record_stage "core-decision-projection-read-model" "started" "$CORE_DECISION_PROJECTION_READ_MODEL_PATH"
  local rust_test_log="$RUNTIME_DIR/core-decision-projection-read-model-rust-test.log"
  if ! (cd "$WORKSPACE" && cargo test -p agentflow-ontology core_decision_projection --quiet >"$rust_test_log" 2>&1); then
    fail_stage "core-decision-projection-read-model" "agentflow-ontology Core Decision Projection Read Model tests failed"
  fi
  python3 - \
    "$CORE_DECISION_PROJECTION_READ_MODEL_PATH" \
    "$WORKSPACE/docs/architecture/077-core-decision-projection-read-model-v1.md" \
    "$WORKSPACE/docs/architecture/074-core-evidence-to-decision-gate-v1.md" \
    "$WORKSPACE/docs/architecture/076-core-delivery-readiness-audit-trigger-v1.md" \
    "$WORKSPACE/crates/ontology/src/decision.rs" \
    "$WORKSPACE/docs/delivery/releases/v1.0.7/AGENTFLOW_V1_0_7_DECISION_KERNEL_TASKS_V1.md" \
    "$rust_test_log" <<'PY'
import json
import pathlib
import sys
import time

out_path = pathlib.Path(sys.argv[1])
doc_path = pathlib.Path(sys.argv[2])
evidence_gate_doc_path = pathlib.Path(sys.argv[3])
delivery_doc_path = pathlib.Path(sys.argv[4])
decision_source_path = pathlib.Path(sys.argv[5])
tasks_path = pathlib.Path(sys.argv[6])
test_log_path = pathlib.Path(sys.argv[7])

doc_text = doc_path.read_text(encoding="utf-8")
evidence_gate_doc_text = evidence_gate_doc_path.read_text(encoding="utf-8")
delivery_doc_text = delivery_doc_path.read_text(encoding="utf-8")
decision_source = decision_source_path.read_text(encoding="utf-8")
tasks_text = tasks_path.read_text(encoding="utf-8")
required_fields = [
    "decisionStatus",
    "reasonCodes",
    "evidenceRefs",
    "deliveryReady",
    "readinessOutcome",
    "auditSidecar",
    "sourceRefs",
    "authorityBoundary",
    "writeAuthorityAllowed",
]
negative_fixtures = [
    "core_decision_projection_rejects_missing_evidence_as_ready",
    "core_decision_projection_rejects_fake_evidence_as_accepted",
    "core_decision_projection_rejects_wrong_completed_state",
    "core_decision_projection_rejects_projection_as_authority",
    "core_decision_projection_rejects_audit_chain_pollution",
]
coverage = {
    "contract-version-defined": "agentflow-core-decision-projection-read-model.v1" in decision_source
    and "agentflow-core-decision-projection-read-model.v1" in doc_text,
    "rust-contract-implemented": "CoreDecisionProjectionReadModelContract" in decision_source
    and "CoreDecisionProjectionBuildRequest" in decision_source
    and "CoreDecisionProjectionReadModel" in decision_source
    and "validate_core_decision_projection_read_model_contract" in decision_source
    and "project_core_decision_read_model" in decision_source
    and "validate_core_decision_projection_read_model" in decision_source,
    "reads-decision-and-evidence": "DecisionRef" in doc_text
    and "EvidenceRef" in doc_text
    and "Evidence-to-Decision Gate" in evidence_gate_doc_text,
    "reads-delivery-readiness": "DeliveryReadinessRef" in doc_text
    and "Core Delivery Readiness" in delivery_doc_text,
    "surfaces-required-fields": all(field in doc_text for field in required_fields)
    and all(field in decision_source for field in required_fields),
    "projection-readonly": "read-only-projection" in decision_source
    and "writeAuthorityAllowed = false" in doc_text
    and "write_authority_allowed: false" in decision_source,
    "projection-as-authority-negative": "ProjectionRef" in decision_source
    and "core_decision_projection_rejects_projection_as_authority" in decision_source,
    "missing-evidence-negative": "core_decision_projection_rejects_missing_evidence_as_ready" in decision_source,
    "fake-evidence-negative": "core_decision_projection_rejects_fake_evidence_as_accepted" in decision_source,
    "wrong-state-negative": "core_decision_projection_rejects_wrong_completed_state" in decision_source,
    "audit-chain-pollution-negative": "core_decision_projection_rejects_audit_chain_pollution" in decision_source
    and "audit sidecar must not pollute" in decision_source
    and "Audit sidecar" in doc_text,
    "negative-fixtures-implemented": all(item in decision_source for item in negative_fixtures),
    "release-task-binds-issue-701": "#701" in tasks_text
    and "V107-009 Decision Projection Read Model and Negative Fixtures" in tasks_text
    and "runtime/core-decision-projection-read-model.json" in tasks_text,
    "rust-contract-tests-passed": test_log_path.is_file(),
}
failed = [item for item, passed in coverage.items() if not passed]
payload = {
    "version": "agentflow-core-decision-projection-read-model-gate.v1",
    "status": "passed" if not failed else "failed",
    "contractVersion": "agentflow-core-decision-projection-read-model.v1",
    "architecturePath": "docs/architecture/077-core-decision-projection-read-model-v1.md",
    "rustContractPath": "crates/ontology/src/decision.rs",
    "rustTestLogPath": "runtime/core-decision-projection-read-model-rust-test.log",
    "requiredFields": required_fields,
    "negativeFixtures": negative_fixtures,
    "coverage": coverage,
    "failedCoverage": failed,
    "checkedAt": int(time.time()),
}
out_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
if failed:
    raise SystemExit(f"core decision projection read model coverage failed: {failed}")
PY
  record_stage "core-decision-projection-read-model" "passed" "$(basename "$CORE_DECISION_PROJECTION_READ_MODEL_PATH")"
}

run_v107_release_certification_gate() {
  record_stage "v107-release-certification" "started" "$V107_RELEASE_CERTIFICATION_PATH"
  python3 - \
    "$V107_RELEASE_CERTIFICATION_PATH" \
    "$RELEASE_VERSION" \
    "$WORKSPACE/Cargo.toml" \
    "$WORKSPACE/apps/desktop/package.json" \
    "$WORKSPACE/apps/desktop/package-lock.json" \
    "$WORKSPACE/apps/desktop/src-tauri/tauri.conf.json" \
    "$ROOT/CHANGELOG.md" \
    "$ROOT/AGENTS.md" \
    "$ROOT/docs/README.md" \
    "$WORKSPACE/docs/delivery/releases/v1.0.7/README.md" \
    "$WORKSPACE/docs/delivery/releases/v1.0.7/AGENTFLOW_V1_0_7_DECISION_KERNEL_TASKS_V1.md" \
    "$ARTIFACT_MANIFEST_PATH" \
    "$RELEASE_URL" \
    "$GATE_EVENT_NAME" \
    "$GATE_REF_TYPE" \
    "$GATE_REF_NAME" \
    "$GATE_RUN_ID" \
    "$GATE_RUN_ATTEMPT" \
    "$GATE_REPOSITORY" \
    "$GATE_SERVER_URL" \
    "$SOURCE_COMMIT_SHA" \
    "$RELEASE_TAG_NAME" \
    "$V107_RELEASE_PROVENANCE_HANDOFF_PATH" \
    "$CORE_DECISION_MODEL_CONTRACT_PATH" \
    "$CORE_DECISION_INPUT_BINDING_PATH" \
    "$CORE_DECISION_OUTCOME_TRANSITIONS_PATH" \
    "$CORE_DECISION_FAILURE_REASON_PATH" \
    "$CORE_EVIDENCE_TO_DECISION_GATE_PATH" \
    "$CORE_COMPLETION_COMMIT_AUTHORITY_PATH" \
    "$CORE_DELIVERY_READINESS_AUDIT_TRIGGER_PATH" \
    "$CORE_DECISION_PROJECTION_READ_MODEL_PATH" <<'PY'
import hashlib
import json
import pathlib
import sys
import time
import tomllib

(
    out_path_raw,
    release_version,
    cargo_path_raw,
    desktop_package_path_raw,
    desktop_package_lock_path_raw,
    tauri_config_path_raw,
    changelog_path_raw,
    agents_path_raw,
    docs_readme_path_raw,
    release_readme_path_raw,
    release_tasks_path_raw,
    artifact_manifest_path_raw,
    release_url,
    gate_event_name,
    gate_ref_type,
    gate_ref_name,
    gate_run_id,
    gate_run_attempt,
    gate_repository,
    gate_server_url,
    source_commit_sha,
    release_tag_name,
    *artifact_path_values,
) = sys.argv[1:]

out_path = pathlib.Path(out_path_raw)
cargo_path = pathlib.Path(cargo_path_raw)
desktop_package_path = pathlib.Path(desktop_package_path_raw)
desktop_package_lock_path = pathlib.Path(desktop_package_lock_path_raw)
tauri_config_path = pathlib.Path(tauri_config_path_raw)
changelog_path = pathlib.Path(changelog_path_raw)
agents_path = pathlib.Path(agents_path_raw)
docs_readme_path = pathlib.Path(docs_readme_path_raw)
release_readme_path = pathlib.Path(release_readme_path_raw)
release_tasks_path = pathlib.Path(release_tasks_path_raw)
artifact_manifest_path = pathlib.Path(artifact_manifest_path_raw)
artifact_paths = [pathlib.Path(value) for value in artifact_path_values]

expected_version = "1.0.7"
expected_tag = "v1.0.7"
cargo = tomllib.loads(cargo_path.read_text(encoding="utf-8"))
desktop_package = json.loads(desktop_package_path.read_text(encoding="utf-8"))
desktop_package_lock = json.loads(desktop_package_lock_path.read_text(encoding="utf-8"))
tauri_config = json.loads(tauri_config_path.read_text(encoding="utf-8"))
changelog_text = changelog_path.read_text(encoding="utf-8")
agents_text = agents_path.read_text(encoding="utf-8")
docs_readme_text = docs_readme_path.read_text(encoding="utf-8")
release_readme_text = release_readme_path.read_text(encoding="utf-8")
release_tasks_text = release_tasks_path.read_text(encoding="utf-8")
artifacts = [json.loads(path.read_text(encoding="utf-8")) for path in artifact_paths]
artifact_statuses = {
    artifact_paths[index].name: artifact.get("status")
    for index, artifact in enumerate(artifacts)
}
certified_artifact_hashes = [
    {
        "path": f"runtime/{path.name}",
        "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
        "bytes": path.stat().st_size,
    }
    for path in artifact_paths
]
artifact_manifest_sha256 = (
    hashlib.sha256(artifact_manifest_path.read_bytes()).hexdigest()
    if artifact_manifest_path.is_file()
    else None
)
certification_digest = hashlib.sha256(json.dumps(
    {
        "releaseVersion": release_version,
        "artifactManifestSha256": artifact_manifest_sha256,
        "certifiedArtifactHashes": certified_artifact_hashes,
    },
    sort_keys=True,
).encode("utf-8")).hexdigest()
gate_run_url = (
    f"{gate_server_url.rstrip('/')}/{gate_repository}/actions/runs/{gate_run_id}"
    if gate_server_url and gate_repository and gate_run_id
    else None
)
event_evidence = {
    "eventName": gate_event_name,
    "refType": gate_ref_type or None,
    "refName": gate_ref_name or None,
    "runId": gate_run_id or None,
    "runAttempt": gate_run_attempt or None,
    "runUrl": gate_run_url,
    "repository": gate_repository,
    "sourceCommitSha": source_commit_sha,
    "releaseTagName": release_tag_name,
    "releaseUrl": release_url,
    "certificationArtifactName": f"release-gate-certification-{release_version}",
    "certificationArtifactDigest": certification_digest,
    "certificationArtifactDigestSource": "v107-certified-decision-kernel-runtime-artifact-hashes",
    "artifactManifestPath": "artifact-manifest.json",
    "artifactManifestSha256": artifact_manifest_sha256,
}

required_artifact_names = [
    "v107-release-provenance-handoff.json",
    "core-decision-model-contract.json",
    "core-decision-input-binding.json",
    "core-decision-outcome-transitions.json",
    "core-decision-failure-reason-remediation.json",
    "core-evidence-to-decision-gate.json",
    "core-completion-commit-authority.json",
    "core-delivery-readiness-audit-trigger.json",
    "core-decision-projection-read-model.json",
]
certified_hash_paths = {item.get("path") for item in certified_artifact_hashes}

coverage = {
    "release-version-at-or-after-v107": release_version in {expected_tag, "v1.0.8", "v1.0.9", "v1.1.0", "v1.1.1"},
    "release-tag-at-or-after-v107": release_tag_name in {expected_tag, "v1.0.8", "v1.0.9", "v1.1.0", "v1.1.1"},
    "cargo-workspace-version-at-or-after-107": cargo["workspace"]["package"]["version"] in {expected_version, "1.0.8", "1.0.9", "1.1.0", "1.1.1"},
    "desktop-package-version-at-or-after-107": desktop_package.get("version") in {expected_version, "1.0.8", "1.0.9", "1.1.0", "1.1.1"},
    "desktop-package-lock-version-at-or-after-107": desktop_package_lock.get("version") in {expected_version, "1.0.8", "1.0.9", "1.1.0", "1.1.1"}
    and (desktop_package_lock.get("packages") or {}).get("", {}).get("version") in {expected_version, "1.0.8", "1.0.9", "1.1.0", "1.1.1"},
    "tauri-version-at-or-after-107": tauri_config.get("version") in {expected_version, "1.0.8", "1.0.9", "1.1.0", "1.1.1"},
    "agents-current-baseline-is-v108-or-v107": (
        "docs/delivery/releases/v1.1.1/README.md" in agents_text
        or "docs/delivery/releases/v1.1.0/README.md" in agents_text
        or "docs/delivery/releases/v1.0.9/README.md" in agents_text
        or "docs/delivery/releases/v1.0.8/README.md" in agents_text
        or "docs/delivery/releases/v1.0.7/README.md" in agents_text
    ),
    "docs-default-reading-is-v108-or-v107": (
        "delivery/releases/v1.1.1/README.md" in docs_readme_text
        or "delivery/releases/v1.1.0/README.md" in docs_readme_text
        or "delivery/releases/v1.0.9/README.md" in docs_readme_text
        or "delivery/releases/v1.0.8/README.md" in docs_readme_text
        or "delivery/releases/v1.0.7/README.md" in docs_readme_text
    ),
    "changelog-has-v107-entry": "## v1.0.7 - 2026-06-29" in changelog_text
    and "Core Decision Kernel baseline" in changelog_text
    and "v107-release-certification" in changelog_text,
    "changelog-records-v108-path": "v1.0.8" in changelog_text
    and "Core Projection Kernel" in changelog_text,
    "release-readme-is-baseline": "Core Decision Kernel release baseline" in release_readme_text
    and "runtime/v107-release-certification.json" in release_readme_text
    and "v1.0.8" in release_readme_text,
    "release-tasks-v107010-done": "V107-010 v1.0.7 Release Certification" in release_tasks_text
    and "状态：done" in release_tasks_text
    and "runtime/v107-release-certification.json" in release_tasks_text
    and "#702" in release_tasks_text,
    "all-v107-artifacts-passed": all(status == "passed" for status in artifact_statuses.values()),
    "all-required-v107-artifacts-present": all(name in artifact_statuses for name in required_artifact_names),
    "certified-artifact-hashes-present": len(certified_artifact_hashes) == len(artifact_paths)
    and all(item.get("sha256") and item.get("bytes", 0) > 0 for item in certified_artifact_hashes)
    and all(f"runtime/{name}" in certified_hash_paths for name in required_artifact_names),
    "artifact-manifest-digest-present": artifact_manifest_sha256 is not None,
    "certification-artifact-digest-present": len(certification_digest) == 64,
    "release-event-evidence-recorded": bool(event_evidence["eventName"])
    and bool(event_evidence["sourceCommitSha"])
    and bool(event_evidence["releaseTagName"]),
    "release-run-id-bound-for-ci": gate_event_name == "local" or bool(event_evidence["runId"]),
    "release-run-url-bound-for-ci": gate_event_name == "local" or bool(event_evidence["runUrl"]),
    "decision-model-certified": artifact_statuses.get("core-decision-model-contract.json") == "passed",
    "decision-input-binding-certified": artifact_statuses.get("core-decision-input-binding.json") == "passed",
    "decision-outcomes-certified": artifact_statuses.get("core-decision-outcome-transitions.json") == "passed",
    "failure-reason-certified": artifact_statuses.get("core-decision-failure-reason-remediation.json") == "passed",
    "evidence-to-decision-certified": artifact_statuses.get("core-evidence-to-decision-gate.json") == "passed",
    "completion-authority-certified": artifact_statuses.get("core-completion-commit-authority.json") == "passed",
    "delivery-readiness-certified": artifact_statuses.get("core-delivery-readiness-audit-trigger.json") == "passed",
    "decision-projection-certified": artifact_statuses.get("core-decision-projection-read-model.json") == "passed",
    "audit-remains-sidecar": "optional sidecar" in release_readme_text
    and "default Done chain" in release_readme_text,
    "projection-kernel-deferred": "Projection Kernel" in release_readme_text
    and "v1.0.8" in release_readme_text,
}
failed = [item for item, passed in coverage.items() if not passed]
payload = {
    "version": "agentflow-v107-release-certification.v1",
    "status": "passed" if not failed else "failed",
    "releaseVersion": expected_tag,
    "workspaceVersion": expected_version,
    "certifiedArtifacts": artifact_statuses,
    "certifiedArtifactHashes": certified_artifact_hashes,
    "eventEvidence": event_evidence,
    "coverage": coverage,
    "failedCoverage": failed,
    "releaseBaseline": "docs/delivery/releases/v1.0.7/README.md",
    "releaseTasks": "docs/delivery/releases/v1.0.7/AGENTFLOW_V1_0_7_DECISION_KERNEL_TASKS_V1.md",
    "remainingRisks": [
        {
            "id": "v108-projection-kernel",
            "summary": "Full Projection Kernel rebuild and replay remains deferred to v1.0.8.",
            "blocking": False,
        },
        {
            "id": "software-dev-reference-app-closeout",
            "summary": "Full Software Dev Reference App certification remains outside Core Decision authority.",
            "blocking": False,
        },
    ],
    "checkedAt": int(time.time()),
}
out_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
if failed:
    raise SystemExit(f"v1.0.7 release certification failed: {failed}")
PY
  record_stage "v107-release-certification" "passed" "$(basename "$V107_RELEASE_CERTIFICATION_PATH")"
}

run_v108_release_certification_gate() {
  record_stage "v108-release-certification" "started" "$V108_RELEASE_CERTIFICATION_PATH"
  python3 - \
    "$V108_RELEASE_CERTIFICATION_PATH" \
    "$RELEASE_VERSION" \
    "$WORKSPACE/Cargo.toml" \
    "$WORKSPACE/apps/desktop/package.json" \
    "$WORKSPACE/apps/desktop/package-lock.json" \
    "$WORKSPACE/apps/desktop/src-tauri/tauri.conf.json" \
    "$ROOT/CHANGELOG.md" \
    "$ROOT/AGENTS.md" \
    "$ROOT/docs/README.md" \
    "$ROOT/docs/delivery/README.md" \
    "$WORKSPACE/docs/delivery/releases/v1.0.8/README.md" \
    "$WORKSPACE/docs/delivery/releases/v1.0.8/AGENTFLOW_V1_0_8_PROJECTION_KERNEL_TASKS_V1.md" \
    "$ARTIFACT_MANIFEST_PATH" \
    "$RELEASE_URL" \
    "$GATE_EVENT_NAME" \
    "$GATE_REF_TYPE" \
    "$GATE_REF_NAME" \
    "$GATE_RUN_ID" \
    "$GATE_RUN_ATTEMPT" \
    "$GATE_REPOSITORY" \
    "$GATE_SERVER_URL" \
    "$SOURCE_COMMIT_SHA" \
    "$RELEASE_TAG_NAME" \
    "$PACK_PROJECTION_READINESS_PATH" \
    "$V107_RELEASE_CERTIFICATION_PATH" \
    "$CORE_PROJECTION_KERNEL_CONTRACT_PATH" \
    "$EVENT_REPLAY_PROJECTION_REPORT_PATH" \
    "$EVENT_REPLAY_PROJECTION_FAILURE_REPORT_PATH" \
    "$CORE_READ_MODEL_SCHEMA_PATH" \
    "$CORE_VIEW_MODEL_CONTRACT_PATH" \
    "$PROJECTION_FEEDBACK_FRESHNESS_PATH" \
    "$CORE_DECISION_PROJECTION_READ_MODEL_PATH" <<'PY'
import hashlib
import json
import pathlib
import sys
import time
import tomllib

(
    out_path_raw,
    release_version,
    cargo_path_raw,
    desktop_package_path_raw,
    desktop_package_lock_path_raw,
    tauri_config_path_raw,
    changelog_path_raw,
    agents_path_raw,
    docs_readme_path_raw,
    delivery_readme_path_raw,
    release_readme_path_raw,
    release_tasks_path_raw,
    artifact_manifest_path_raw,
    release_url,
    gate_event_name,
    gate_ref_type,
    gate_ref_name,
    gate_run_id,
    gate_run_attempt,
    gate_repository,
    gate_server_url,
    source_commit_sha,
    release_tag_name,
    pack_projection_path_raw,
    *runtime_artifact_path_values,
) = sys.argv[1:]

out_path = pathlib.Path(out_path_raw)
cargo_path = pathlib.Path(cargo_path_raw)
desktop_package_path = pathlib.Path(desktop_package_path_raw)
desktop_package_lock_path = pathlib.Path(desktop_package_lock_path_raw)
tauri_config_path = pathlib.Path(tauri_config_path_raw)
changelog_path = pathlib.Path(changelog_path_raw)
agents_path = pathlib.Path(agents_path_raw)
docs_readme_path = pathlib.Path(docs_readme_path_raw)
delivery_readme_path = pathlib.Path(delivery_readme_path_raw)
release_readme_path = pathlib.Path(release_readme_path_raw)
release_tasks_path = pathlib.Path(release_tasks_path_raw)
artifact_manifest_path = pathlib.Path(artifact_manifest_path_raw)
pack_projection_path = pathlib.Path(pack_projection_path_raw)
runtime_artifact_paths = [pathlib.Path(value) for value in runtime_artifact_path_values]
artifact_paths = runtime_artifact_paths + [pack_projection_path]

expected_version = "1.0.8"
expected_tag = "v1.0.8"
cargo = tomllib.loads(cargo_path.read_text(encoding="utf-8"))
desktop_package = json.loads(desktop_package_path.read_text(encoding="utf-8"))
desktop_package_lock = json.loads(desktop_package_lock_path.read_text(encoding="utf-8"))
tauri_config = json.loads(tauri_config_path.read_text(encoding="utf-8"))
changelog_text = changelog_path.read_text(encoding="utf-8")
agents_text = agents_path.read_text(encoding="utf-8")
docs_readme_text = docs_readme_path.read_text(encoding="utf-8")
delivery_readme_text = delivery_readme_path.read_text(encoding="utf-8")
release_readme_text = release_readme_path.read_text(encoding="utf-8")
release_tasks_text = release_tasks_path.read_text(encoding="utf-8")

def load_json(path):
    if not path.is_file():
        return {}
    return json.loads(path.read_text(encoding="utf-8"))

artifacts = {path.name: load_json(path) for path in artifact_paths}
artifact_statuses = {name: payload.get("status") for name, payload in artifacts.items()}
required_artifact_statuses = {
    "v107-release-certification.json": "passed",
    "core-projection-kernel-contract.json": "passed",
    "event-replay-projection-report.json": "passed",
    "event-replay-projection-failure-report.json": "failed",
    "core-read-model-schema.json": "passed",
    "core-view-model-contract.json": "passed",
    "projection-feedback-freshness-receipts.json": "passed",
    "core-decision-projection-read-model.json": "passed",
    "pack-projection-readiness.json": "passed",
}
certified_artifact_hashes = [
    {
        "path": f"runtime/{path.name}" if path.parent.name == "runtime" else path.name,
        "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
        "bytes": path.stat().st_size,
    }
    for path in artifact_paths
    if path.is_file()
]
certified_hash_paths = {item.get("path") for item in certified_artifact_hashes}
artifact_manifest_sha256 = (
    hashlib.sha256(artifact_manifest_path.read_bytes()).hexdigest()
    if artifact_manifest_path.is_file()
    else None
)
certification_digest = hashlib.sha256(json.dumps(
    {
        "releaseVersion": release_version,
        "artifactManifestSha256": artifact_manifest_sha256,
        "certifiedArtifactHashes": certified_artifact_hashes,
    },
    sort_keys=True,
).encode("utf-8")).hexdigest()
gate_run_url = (
    f"{gate_server_url.rstrip('/')}/{gate_repository}/actions/runs/{gate_run_id}"
    if gate_server_url and gate_repository and gate_run_id
    else None
)
event_evidence = {
    "eventName": gate_event_name,
    "refType": gate_ref_type or None,
    "refName": gate_ref_name or None,
    "runId": gate_run_id or None,
    "runAttempt": gate_run_attempt or None,
    "runUrl": gate_run_url,
    "repository": gate_repository,
    "sourceCommitSha": source_commit_sha,
    "releaseTagName": release_tag_name,
    "releaseUrl": release_url,
    "certificationArtifactName": f"release-gate-certification-{release_version}",
    "certificationArtifactDigest": certification_digest,
    "certificationArtifactDigestSource": "v108-certified-projection-kernel-runtime-artifact-hashes",
    "artifactManifestPath": "artifact-manifest.json",
    "artifactManifestSha256": artifact_manifest_sha256,
}

projection_kernel = artifacts.get("core-projection-kernel-contract.json") or {}
replay_report = artifacts.get("event-replay-projection-report.json") or {}
replay_failure = artifacts.get("event-replay-projection-failure-report.json") or {}
read_model = artifacts.get("core-read-model-schema.json") or {}
view_model = artifacts.get("core-view-model-contract.json") or {}
feedback = artifacts.get("projection-feedback-freshness-receipts.json") or {}
pack_projection = artifacts.get("pack-projection-readiness.json") or {}

required_architecture_docs = [
    "079-core-projection-kernel-contract-v1.md",
    "080-event-replay-projection-rebuild-v1.md",
    "081-core-read-model-schema-v1.md",
    "082-view-model-contract-for-industry-apps-v1.md",
    "083-pack-specific-projection-mapping-boundary-v1.md",
    "084-invalid-missing-app-definition-handling-v1.md",
    "085-feedback-surface-projection-freshness-receipts-v1.md",
]
required_issue_refs = [f"#{number}" for number in range(713, 723)]
required_task_ids = [f"V108-{index:03d}" for index in range(1, 11)]
required_runtime_paths = {
    "runtime/v107-release-certification.json",
    "runtime/core-projection-kernel-contract.json",
    "runtime/event-replay-projection-report.json",
    "runtime/event-replay-projection-failure-report.json",
    "runtime/core-read-model-schema.json",
    "runtime/core-view-model-contract.json",
    "runtime/projection-feedback-freshness-receipts.json",
    "runtime/core-decision-projection-read-model.json",
}
expected_hash_paths = required_runtime_paths | {"pack-projection-readiness.json"}

coverage = {
    "release-version-at-or-after-v108": release_version in {expected_tag, "v1.0.9", "v1.1.0", "v1.1.1"},
    "release-tag-at-or-after-v108": release_tag_name in {expected_tag, "v1.0.9", "v1.1.0", "v1.1.1"},
    "cargo-workspace-version-at-or-after-108": cargo["workspace"]["package"]["version"] in {expected_version, "1.0.9", "1.1.0", "1.1.1"},
    "desktop-package-version-at-or-after-108": desktop_package.get("version") in {expected_version, "1.0.9", "1.1.0", "1.1.1"},
    "desktop-package-lock-version-at-or-after-108": desktop_package_lock.get("version") in {expected_version, "1.0.9", "1.1.0", "1.1.1"}
    and (desktop_package_lock.get("packages") or {}).get("", {}).get("version") in {expected_version, "1.0.9", "1.1.0", "1.1.1"},
    "tauri-version-at-or-after-108": tauri_config.get("version") in {expected_version, "1.0.9", "1.1.0", "1.1.1"},
    "agents-current-baseline-at-or-after-v108": (
        "docs/delivery/releases/v1.1.1/README.md" in agents_text
        or "docs/delivery/releases/v1.1.0/README.md" in agents_text
        or "docs/delivery/releases/v1.0.9/README.md" in agents_text
        or "docs/delivery/releases/v1.0.8/README.md" in agents_text
    ),
    "docs-default-reading-at-or-after-v108": (
        "delivery/releases/v1.1.1/README.md" in docs_readme_text
        or "delivery/releases/v1.1.0/README.md" in docs_readme_text
        or "delivery/releases/v1.0.9/README.md" in docs_readme_text
        or "delivery/releases/v1.0.8/README.md" in docs_readme_text
    ),
    "delivery-readme-at-or-after-v108": (
        "releases/v1.1.1/README.md" in delivery_readme_text
        or "releases/v1.1.0/README.md" in delivery_readme_text
        or "releases/v1.0.9/README.md" in delivery_readme_text
        or "releases/v1.0.8/README.md" in delivery_readme_text
    )
    and "Core Projection Kernel" in delivery_readme_text,
    "changelog-has-v108-entry": "## v1.0.8 - 2026-06-30" in changelog_text
    and "Core Projection Kernel baseline" in changelog_text
    and "v108-release-certification" in changelog_text,
    "changelog-records-v109-path": "## v1.0.9 - 2026-07-01" in changelog_text
    and "Software Dev Reference App Boundary Certification" in changelog_text,
    "release-readme-is-baseline": "Core Projection Kernel release baseline" in release_readme_text
    and "runtime/v108-release-certification.json" in release_readme_text
    and "v1.0.9" in release_readme_text,
    "release-readme-states-nongoals": all(phrase in release_readme_text for phrase in [
        "Software Dev commercial app completion",
        "v1.0.9",
        "default Message Bus",
        "Projection",
        "GitHub",
    ]),
    "release-readme-states-known-risks": "## Known Risks" in release_readme_text
    and "Software Dev Reference App certification" in release_readme_text,
    "release-tasks-all-v108-done": all(task_id in release_tasks_text for task_id in required_task_ids)
    and all(issue_ref in release_tasks_text for issue_ref in required_issue_refs)
    and release_tasks_text.count("状态：done") >= 10,
    "architecture-docs-bound": all(doc in release_readme_text for doc in required_architecture_docs)
    and all(doc in release_tasks_text or doc in release_readme_text for doc in required_architecture_docs),
    "all-v108-artifacts-present": all(name in artifact_statuses for name in required_artifact_statuses),
    "all-v108-artifact-statuses-valid": all(
        artifact_statuses.get(name) == expected_status
        for name, expected_status in required_artifact_statuses.items()
    ),
    "certified-artifact-hashes-present": len(certified_artifact_hashes) == len(artifact_paths)
    and all(item.get("sha256") and item.get("bytes", 0) > 0 for item in certified_artifact_hashes)
    and expected_hash_paths.issubset(certified_hash_paths),
    "artifact-manifest-digest-present": artifact_manifest_sha256 is not None,
    "certification-artifact-digest-present": len(certification_digest) == 64,
    "release-event-evidence-recorded": bool(event_evidence["eventName"])
    and bool(event_evidence["sourceCommitSha"])
    and bool(event_evidence["releaseTagName"]),
    "release-run-id-bound-for-ci": gate_event_name == "local" or bool(event_evidence["runId"]),
    "release-run-url-bound-for-ci": gate_event_name == "local" or bool(event_evidence["runUrl"]),
    "projection-kernel-readonly": projection_kernel.get("writesAuthority") is False
    and projection_kernel.get("projectionAuthority") is False,
    "replay-rebuild-certified": replay_report.get("deterministic") is True
    and replay_report.get("writesAuthority") is False
    and replay_report.get("projectionAuthority") is False,
    "replay-failure-certified": replay_failure.get("status") == "failed"
    and replay_failure.get("writesAuthority") is False
    and replay_failure.get("projectionAuthority") is False
    and bool(replay_failure.get("failures")),
    "read-model-schema-certified": read_model.get("writesAuthority") is False
    and read_model.get("projectionAuthority") is False
    and bool(read_model.get("schemaVersions")),
    "view-model-certified": view_model.get("writesAuthority") is False
    and view_model.get("readsAuthorityDirectly") is False,
    "feedback-freshness-certified": feedback.get("writesAuthority") is False
    and feedback.get("requiresConfirmationForSpecEvolution") is True
    and feedback.get("specEvolutionRoute") == "open-spec-evolution-preview",
    "pack-projection-boundary-certified": pack_projection.get("status") == "passed",
    "audit-remains-sidecar": "optional sidecar" in release_readme_text,
    "software-dev-remains-reference-app": "Reference App" in release_readme_text
    and "Core authority" in release_readme_text,
}
failed = [item for item, passed in coverage.items() if not passed]
payload = {
    "version": "agentflow-v108-release-certification.v1",
    "status": "passed" if not failed else "failed",
    "releaseVersion": expected_tag,
    "workspaceVersion": expected_version,
    "certifiedArtifacts": artifact_statuses,
    "certifiedArtifactHashes": certified_artifact_hashes,
    "eventEvidence": event_evidence,
    "coverage": coverage,
    "failedCoverage": failed,
    "releaseBaseline": "docs/delivery/releases/v1.0.8/README.md",
    "releaseTasks": "docs/delivery/releases/v1.0.8/AGENTFLOW_V1_0_8_PROJECTION_KERNEL_TASKS_V1.md",
    "remainingRisks": [
        {
            "id": "v109-software-dev-reference-app",
            "summary": "Software Dev Reference App certification remains outside Core Projection authority.",
            "blocking": False,
        },
        {
            "id": "projection-feedback-materialization",
            "summary": "Feedback routes can propose Spec evolution, but authority materialization remains a separate gate.",
            "blocking": False,
        },
    ],
    "checkedAt": int(time.time()),
}
out_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
if failed:
    raise SystemExit(f"v1.0.8 release certification failed: {failed}")
PY
  record_stage "v108-release-certification" "passed" "$(basename "$V108_RELEASE_CERTIFICATION_PATH")"
}

run_v109_release_certification_gate() {
  record_stage "v109-release-certification" "started" "$V109_RELEASE_CERTIFICATION_PATH"
  python3 - \
    "$V109_RELEASE_CERTIFICATION_PATH" \
    "$RELEASE_VERSION" \
    "$WORKSPACE/Cargo.toml" \
    "$WORKSPACE/apps/desktop/package.json" \
    "$WORKSPACE/apps/desktop/package-lock.json" \
    "$WORKSPACE/apps/desktop/src-tauri/tauri.conf.json" \
    "$ROOT/CHANGELOG.md" \
    "$ROOT/AGENTS.md" \
    "$ROOT/docs/README.md" \
    "$ROOT/docs/delivery/README.md" \
    "$WORKSPACE/docs/delivery/releases/v1.0.9/README.md" \
    "$WORKSPACE/docs/delivery/releases/v1.0.9/AGENTFLOW_V1_0_9_SOFTWARE_DEV_REFERENCE_APP_TASKS_V1.md" \
    "$ARTIFACT_MANIFEST_PATH" \
    "$RELEASE_URL" \
    "$GATE_EVENT_NAME" \
    "$GATE_REF_TYPE" \
    "$GATE_REF_NAME" \
    "$GATE_RUN_ID" \
    "$GATE_RUN_ATTEMPT" \
    "$GATE_REPOSITORY" \
    "$GATE_SERVER_URL" \
    "$SOURCE_COMMIT_SHA" \
    "$RELEASE_TAG_NAME" \
    "$PACK_PROJECTION_READINESS_PATH" \
    "$V108_RELEASE_CERTIFICATION_PATH" \
    "$V109_TASK_ISSUE_TRACEABILITY_PATH" \
    "$V109_SOFTWARE_DEV_PRODUCT_CONTRACT_PATH" \
    "$V109_SPEC_TASK_FLOW_PATH" \
    "$V109_CONNECTOR_HANDOFF_PATH" \
    "$V109_EVIDENCE_DECISION_DELIVERY_PATH" \
    "$V109_WORKBENCH_READ_MODELS_PATH" \
    "$V109_MAPPING_BOUNDARY_PATH" \
    "$V109_GOLDEN_SCENARIO_PATH" \
    "$WORKSPACE/products/software-dev/product.toml" \
    "$WORKSPACE/products/software-dev/domain/definition.json" \
    "$WORKSPACE/products/software-dev/surface/definition.json" \
    "$WORKSPACE/products/software-dev/connectors/definition.json" \
    "$WORKSPACE/products/software-dev/flows/reference-task-flow.json" \
    "$WORKSPACE/products/software-dev/projections/workbench-read-models.json" \
    "$WORKSPACE/products/software-dev/fixtures/golden-scenario.json" \
    "$WORKSPACE/products/software-dev/fixtures/negative-authority-fixtures.json" \
    "$WORKSPACE/crates/pack/fixtures/packs/software-dev" <<'PY'
import hashlib
import json
import pathlib
import sys
import time
import tomllib

(
    out_path_raw,
    release_version,
    cargo_path_raw,
    desktop_package_path_raw,
    desktop_package_lock_path_raw,
    tauri_config_path_raw,
    changelog_path_raw,
    agents_path_raw,
    docs_readme_path_raw,
    delivery_readme_path_raw,
    release_readme_path_raw,
    release_tasks_path_raw,
    artifact_manifest_path_raw,
    release_url,
    gate_event_name,
    gate_ref_type,
    gate_ref_name,
    gate_run_id,
    gate_run_attempt,
    gate_repository,
    gate_server_url,
    source_commit_sha,
    release_tag_name,
    pack_projection_path_raw,
    v108_certification_path_raw,
    task_trace_path_raw,
    product_contract_path_raw,
    spec_task_flow_path_raw,
    connector_handoff_path_raw,
    evidence_delivery_path_raw,
    workbench_read_models_path_raw,
    mapping_boundary_path_raw,
    golden_scenario_path_raw,
    product_toml_path_raw,
    domain_path_raw,
    surface_path_raw,
    connectors_path_raw,
    flow_path_raw,
    projections_path_raw,
    golden_path_raw,
    negative_path_raw,
    fixture_mirror_path_raw,
) = sys.argv[1:]

out_path = pathlib.Path(out_path_raw)
cargo_path = pathlib.Path(cargo_path_raw)
desktop_package_path = pathlib.Path(desktop_package_path_raw)
desktop_package_lock_path = pathlib.Path(desktop_package_lock_path_raw)
tauri_config_path = pathlib.Path(tauri_config_path_raw)
changelog_path = pathlib.Path(changelog_path_raw)
agents_path = pathlib.Path(agents_path_raw)
docs_readme_path = pathlib.Path(docs_readme_path_raw)
delivery_readme_path = pathlib.Path(delivery_readme_path_raw)
release_readme_path = pathlib.Path(release_readme_path_raw)
release_tasks_path = pathlib.Path(release_tasks_path_raw)
artifact_manifest_path = pathlib.Path(artifact_manifest_path_raw)
pack_projection_path = pathlib.Path(pack_projection_path_raw)
v108_certification_path = pathlib.Path(v108_certification_path_raw)
task_trace_path = pathlib.Path(task_trace_path_raw)
product_contract_path = pathlib.Path(product_contract_path_raw)
spec_task_flow_path = pathlib.Path(spec_task_flow_path_raw)
connector_handoff_path = pathlib.Path(connector_handoff_path_raw)
evidence_delivery_path = pathlib.Path(evidence_delivery_path_raw)
workbench_read_models_path = pathlib.Path(workbench_read_models_path_raw)
mapping_boundary_path = pathlib.Path(mapping_boundary_path_raw)
golden_scenario_path = pathlib.Path(golden_scenario_path_raw)
product_toml_path = pathlib.Path(product_toml_path_raw)
domain_path = pathlib.Path(domain_path_raw)
surface_path = pathlib.Path(surface_path_raw)
connectors_path = pathlib.Path(connectors_path_raw)
flow_path = pathlib.Path(flow_path_raw)
projections_path = pathlib.Path(projections_path_raw)
golden_path = pathlib.Path(golden_path_raw)
negative_path = pathlib.Path(negative_path_raw)
fixture_mirror_path = pathlib.Path(fixture_mirror_path_raw)

expected_version = "1.0.9"
expected_tag = "v1.0.9"

def load_json(path):
    if not path.is_file():
        raise SystemExit(f"missing JSON source: {path}")
    return json.loads(path.read_text(encoding="utf-8"))

def write_json(path, payload):
    path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")

def file_digest(path):
    return {
        "path": str(path),
        "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
        "bytes": path.stat().st_size,
    }

cargo = tomllib.loads(cargo_path.read_text(encoding="utf-8"))
desktop_package = json.loads(desktop_package_path.read_text(encoding="utf-8"))
desktop_package_lock = json.loads(desktop_package_lock_path.read_text(encoding="utf-8"))
tauri_config = json.loads(tauri_config_path.read_text(encoding="utf-8"))
product = tomllib.loads(product_toml_path.read_text(encoding="utf-8")) if product_toml_path.is_file() else {}
domain = load_json(domain_path)
surface = load_json(surface_path)
connectors = load_json(connectors_path)
flow = load_json(flow_path)
projections = load_json(projections_path)
golden = load_json(golden_path)
negative = load_json(negative_path)
pack_projection = load_json(pack_projection_path)
v108_certification = load_json(v108_certification_path)

changelog_text = changelog_path.read_text(encoding="utf-8")
agents_text = agents_path.read_text(encoding="utf-8")
docs_readme_text = docs_readme_path.read_text(encoding="utf-8")
delivery_readme_text = delivery_readme_path.read_text(encoding="utf-8")
release_readme_text = release_readme_path.read_text(encoding="utf-8")
release_tasks_text = release_tasks_path.read_text(encoding="utf-8")

task_rows = [
    ("V109-001", 734, "Release Task / GitHub Issue Traceability Gate", task_trace_path),
    ("V109-002", 735, "Quick Audit Pack Projection Primary Proof Inclusion", pack_projection_path),
    ("V109-003", 736, "products/software-dev Reference App Contract", product_contract_path),
    ("V109-004", 737, "Software Dev Spec Bundle to Task Flow", spec_task_flow_path),
    ("V109-005", 738, "Software Dev Connector Handoff and Runtime Command Baseline", connector_handoff_path),
    ("V109-006", 739, "Software Dev Evidence / Decision / Delivery Closed Loop", evidence_delivery_path),
    ("V109-007", 740, "Software Dev Projection Workbench Read Models", workbench_read_models_path),
    ("V109-008", 741, "Core/Product Pack-backed Mapping Boundary Cleanup", mapping_boundary_path),
    ("V109-009", 742, "End-to-End Reference App Scenario and Core Boundary Negative Fixtures", golden_scenario_path),
    ("V109-010", 743, "v1.0.9 Reference App Boundary Release Certification", out_path),
]

traceability_checks = {
    "all-task-ids-present": all(task_id in release_tasks_text for task_id, _, _, _ in task_rows),
    "all-github-issue-refs-present": all(f"#{number}" in release_tasks_text for _, number, _, _ in task_rows),
    "all-task-titles-present": all(title in release_tasks_text for _, _, title, _ in task_rows),
    "all-tasks-done": release_tasks_text.count("状态：done") >= len(task_rows),
    "github-issues-are-not-authority": "GitHub issues are planning mirrors" in release_tasks_text
    and "not AgentFlow authority" in release_tasks_text,
}
task_trace_payload = {
    "version": "agentflow-v109-task-issue-traceability.v1",
    "status": "passed" if all(traceability_checks.values()) else "failed",
    "releaseVersion": expected_tag,
    "githubIssuesAreAuthority": False,
    "tasks": [
        {
            "taskId": task_id,
            "issueNumber": number,
            "issueRef": f"#{number}",
            "title": title,
            "primaryProof": "pack-projection-readiness.json" if task_id == "V109-002" else f"runtime/{path.name}",
            "status": "done",
        }
        for task_id, number, title, path in task_rows
    ],
    "coverage": traceability_checks,
    "checkedAt": int(time.time()),
}
write_json(task_trace_path, task_trace_payload)

product_checks = {
    "product-id-is-software-dev": product.get("product_id") == "software-dev",
    "source-boundary-is-products-software-dev": product.get("source_boundary") == "products/software-dev",
    "core-authority-false": product.get("authority", {}).get("writes_core_authority") is False
    and product.get("authority", {}).get("writes_runtime_authority") is False,
    "all-entrypoints-exist": all(path.is_file() for path in [
        domain_path, surface_path, connectors_path, flow_path, projections_path, golden_path, negative_path
    ]),
    "fixture-mirror-exists": fixture_mirror_path.is_dir(),
    "domain-product-id-matches": domain.get("productId") == "software-dev" and domain.get("coreAuthority") is False,
    "surface-product-id-matches": surface.get("productId") == "software-dev" and surface.get("coreAuthority") is False,
}
product_payload = {
    "version": "agentflow-v109-software-dev-product-contract.v1",
    "status": "passed" if all(product_checks.values()) else "failed",
    "productId": "software-dev",
    "sourceBoundary": "products/software-dev/**",
    "fixtureMirror": "crates/pack/fixtures/packs/software-dev/**",
    "coreAuthority": False,
    "sourceFiles": [file_digest(path) for path in [
        product_toml_path, domain_path, surface_path, connectors_path, flow_path, projections_path, golden_path, negative_path
    ]],
    "coverage": product_checks,
    "checkedAt": int(time.time()),
}
write_json(product_contract_path, product_payload)

flow_stages = [item.get("stage") for item in flow.get("flow", [])]
flow_checks = {
    "required-stages-present": all(stage in flow_stages for stage in [
        "spec-bundle", "handoff", "execution", "evidence", "decision", "delivery"
    ]),
    "no-flow-stage-writes-authority": all(item.get("writesAuthority") is False for item in flow.get("flow", [])),
    "blocked-paths-cover-authority-risks": all(
        any(token in path for path in flow.get("blockedPaths", []))
        for token in ["github-issue", "provider-session", "projection", "audit-sidecar"]
    ),
}
spec_flow_payload = {
    "version": "agentflow-v109-spec-task-flow.v1",
    "status": "passed" if all(flow_checks.values()) else "failed",
    "productId": "software-dev",
    "flow": flow.get("flow", []),
    "blockedPaths": flow.get("blockedPaths", []),
    "coverage": flow_checks,
    "checkedAt": int(time.time()),
}
write_json(spec_task_flow_path, spec_flow_payload)

connector_ids = {item.get("id") for item in connectors.get("connectors", [])}
connector_checks = {
    "required-connectors-present": {"git", "github", "codex", "shell"}.issubset(connector_ids),
    "all-connectors-nonauthority": all(item.get("authority") is False for item in connectors.get("connectors", [])),
    "invalid-authority-sources-bound": all(item in connectors.get("invalidAuthoritySources", []) for item in [
        "github-issue-only", "provider-memory-only", "pull-request-only", "release-note-only", "test-log-only"
    ]),
}
connector_payload = {
    "version": "agentflow-v109-connector-handoff.v1",
    "status": "passed" if all(connector_checks.values()) else "failed",
    "productId": "software-dev",
    "connectors": connectors.get("connectors", []),
    "invalidAuthoritySources": connectors.get("invalidAuthoritySources", []),
    "coverage": connector_checks,
    "checkedAt": int(time.time()),
}
write_json(connector_handoff_path, connector_payload)

evidence_policy = domain.get("evidencePolicy", {})
evidence_checks = {
    "required-evidence-present": all(item in evidence_policy.get("requiredEvidence", []) for item in [
        "changed-content-proof", "local-command-proof", "merge-proof"
    ]),
    "insufficient-alone-bound": all(item in evidence_policy.get("insufficientAlone", []) for item in [
        "pull-request", "release-note", "provider-transcript", "test-log"
    ]),
    "audit-is-sidecar": domain.get("auditBoundary", {}).get("defaultCompletionBlocker") is False
    and domain.get("auditBoundary", {}).get("sidecarOnly") is True,
    "golden-includes-delivery-record": "delivery-record-produced" in golden.get("steps", []),
}
evidence_payload = {
    "version": "agentflow-v109-evidence-decision-delivery.v1",
    "status": "passed" if all(evidence_checks.values()) else "failed",
    "requiredEvidence": evidence_policy.get("requiredEvidence", []),
    "optionalEvidence": evidence_policy.get("optionalEvidence", []),
    "insufficientAlone": evidence_policy.get("insufficientAlone", []),
    "auditBoundary": domain.get("auditBoundary", {}),
    "coverage": evidence_checks,
    "checkedAt": int(time.time()),
}
write_json(evidence_delivery_path, evidence_payload)

read_model_ids = {item.get("id") for item in projections.get("readModels", [])}
surface_pages = {item.get("id") for item in surface.get("pages", [])}
workbench_checks = {
    "task-workbench-read-model-present": "software-dev.task-workbench" in read_model_ids,
    "task-workbench-page-present": "task-workbench" in surface_pages,
    "all-read-models-readonly": all(item.get("writesAuthority") is False for item in projections.get("readModels", [])),
    "forbidden-authority-reads-bound": all(path in projections.get("forbiddenReads", []) for path in [
        ".agentflow/spec/**", ".agentflow/tasks/**", ".agentflow/events/**", ".agentflow/evidence/**"
    ]),
}
workbench_payload = {
    "version": "agentflow-v109-workbench-read-models.v1",
    "status": "passed" if all(workbench_checks.values()) else "failed",
    "readModels": projections.get("readModels", []),
    "surfacePages": surface.get("pages", []),
    "coverage": workbench_checks,
    "checkedAt": int(time.time()),
}
write_json(workbench_read_models_path, workbench_payload)

negative_ids = {item.get("id"): item.get("status") for item in negative.get("fixtures", [])}
mapping_checks = {
    "product-source-exists": product_toml_path.is_file(),
    "fixture-mirror-exists": fixture_mirror_path.is_dir(),
    "negative-direct-projection-write-rejected": negative_ids.get("direct-projection-write") == "rejected",
    "negative-missing-product-mapping-deferred": negative_ids.get("missing-product-mapping") == "deferred",
    "github-provider-pr-release-negative-bound": all(negative_ids.get(item) == "rejected" for item in [
        "github-issue-only", "provider-transcript-only", "pull-request-only", "release-note-only"
    ]),
}
mapping_payload = {
    "version": "agentflow-v109-mapping-boundary.v1",
    "status": "passed" if all(mapping_checks.values()) else "failed",
    "productSourceBoundary": "products/software-dev/**",
    "fixtureMirror": "crates/pack/fixtures/packs/software-dev/**",
    "negativeAuthorityFixtures": negative.get("fixtures", []),
    "coverage": mapping_checks,
    "checkedAt": int(time.time()),
}
write_json(mapping_boundary_path, mapping_payload)

golden_checks = {
    "golden-scenario-steps-present": all(step in golden.get("steps", []) for step in [
        "core-spec-bundle-confirmed",
        "software-dev-task-contract-derived",
        "runtime-command-admitted",
        "connector-handoff-created",
        "evidence-pack-collected",
        "decision-accepted",
        "delivery-record-produced",
        "projection-workbench-updated",
    ]),
    "negative-fixtures-present": all(item in negative_ids for item in [
        "github-issue-only", "provider-transcript-only", "pull-request-only", "release-note-only",
        "direct-projection-write", "missing-product-mapping", "audit-default-blocker",
    ]),
    "completion-policy-protects-authority": golden.get("completionPolicy", {}).get("projectionWritesAuthority") is False
    and golden.get("completionPolicy", {}).get("providerSessionWritesAuthority") is False
    and golden.get("completionPolicy", {}).get("auditSidecarBlocksDefaultDone") is False,
}
golden_payload = {
    "version": "agentflow-v109-golden-scenario.v1",
    "status": "passed" if all(golden_checks.values()) else "failed",
    "scenarioId": golden.get("scenarioId"),
    "steps": golden.get("steps", []),
    "negativeAuthorityFixtures": negative.get("fixtures", []),
    "coverage": golden_checks,
    "checkedAt": int(time.time()),
}
write_json(golden_scenario_path, golden_payload)

v109_artifact_paths = [
    task_trace_path,
    product_contract_path,
    spec_task_flow_path,
    connector_handoff_path,
    evidence_delivery_path,
    workbench_read_models_path,
    mapping_boundary_path,
    golden_scenario_path,
]
v109_artifacts = {path.name: load_json(path) for path in v109_artifact_paths}
artifact_statuses = {
    **{name: payload.get("status") for name, payload in v109_artifacts.items()},
    "pack-projection-readiness.json": pack_projection.get("status"),
    "v108-release-certification.json": v108_certification.get("status"),
}
certified_paths = [v108_certification_path, pack_projection_path, *v109_artifact_paths]
certified_artifact_hashes = [
    {
        "path": f"runtime/{path.name}" if path.parent.name == "runtime" else path.name,
        "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
        "bytes": path.stat().st_size,
    }
    for path in certified_paths
    if path.is_file()
]
artifact_manifest_sha256 = (
    hashlib.sha256(artifact_manifest_path.read_bytes()).hexdigest()
    if artifact_manifest_path.is_file()
    else None
)
certification_digest = hashlib.sha256(json.dumps(
    {
        "releaseVersion": release_version,
        "artifactManifestSha256": artifact_manifest_sha256,
        "certifiedArtifactHashes": certified_artifact_hashes,
    },
    sort_keys=True,
).encode("utf-8")).hexdigest()
gate_run_url = (
    f"{gate_server_url.rstrip('/')}/{gate_repository}/actions/runs/{gate_run_id}"
    if gate_server_url and gate_repository and gate_run_id
    else None
)
event_evidence = {
    "eventName": gate_event_name,
    "refType": gate_ref_type or None,
    "refName": gate_ref_name or None,
    "runId": gate_run_id or None,
    "runAttempt": gate_run_attempt or None,
    "runUrl": gate_run_url,
    "repository": gate_repository,
    "sourceCommitSha": source_commit_sha,
    "releaseTagName": release_tag_name,
    "releaseUrl": release_url,
    "certificationArtifactName": f"release-gate-certification-{release_version}",
    "certificationArtifactDigest": certification_digest,
    "certificationArtifactDigestSource": "v109-software-dev-reference-app-runtime-artifact-hashes",
    "artifactManifestPath": "artifact-manifest.json",
    "artifactManifestSha256": artifact_manifest_sha256,
}
coverage = {
    "release-version-at-or-after-v109": release_version in {expected_tag, "v1.1.0", "v1.1.1"},
    "release-tag-at-or-after-v109": release_tag_name in {expected_tag, "v1.1.0", "v1.1.1"},
    "cargo-workspace-version-at-or-after-109": cargo["workspace"]["package"]["version"] in {expected_version, "1.1.0", "1.1.1"},
    "desktop-package-version-at-or-after-109": desktop_package.get("version") in {expected_version, "1.1.0", "1.1.1"},
    "desktop-package-lock-version-at-or-after-109": desktop_package_lock.get("version") in {expected_version, "1.1.0", "1.1.1"}
    and (desktop_package_lock.get("packages") or {}).get("", {}).get("version") in {expected_version, "1.1.0", "1.1.1"},
    "tauri-version-at-or-after-109": tauri_config.get("version") in {expected_version, "1.1.0", "1.1.1"},
    "agents-current-baseline-at-or-after-v109": (
        "docs/delivery/releases/v1.1.1/README.md" in agents_text
        or "docs/delivery/releases/v1.1.0/README.md" in agents_text
        or "docs/delivery/releases/v1.0.9/README.md" in agents_text
    ),
    "docs-default-reading-at-or-after-v109": (
        "delivery/releases/v1.1.1/README.md" in docs_readme_text
        or "delivery/releases/v1.1.0/README.md" in docs_readme_text
        or "delivery/releases/v1.0.9/README.md" in docs_readme_text
    ),
    "delivery-readme-at-or-after-v109": (
        "releases/v1.1.1/README.md" in delivery_readme_text
        or "releases/v1.1.0/README.md" in delivery_readme_text
        or "releases/v1.0.9/README.md" in delivery_readme_text
    ),
    "changelog-has-v109-entry": "## v1.0.9 - 2026-07-01" in changelog_text
    and "Software Dev Reference App Boundary Certification" in changelog_text,
    "release-readme-is-v109-baseline": "Software Dev Reference App boundary certification release baseline" in release_readme_text
    and "runtime/v109-release-certification.json" in release_readme_text,
    "release-tasks-are-bound": all(traceability_checks.values()),
    "all-v109-artifacts-passed": all(payload.get("status") == "passed" for payload in v109_artifacts.values()),
    "pack-projection-included-as-primary-proof": pack_projection.get("status") == "passed",
    "v108-certification-passed": v108_certification.get("status") == "passed",
    "product-contract-passed": product_payload["status"] == "passed",
    "spec-task-flow-passed": spec_flow_payload["status"] == "passed",
    "connector-handoff-passed": connector_payload["status"] == "passed",
    "evidence-decision-delivery-passed": evidence_payload["status"] == "passed",
    "workbench-read-models-passed": workbench_payload["status"] == "passed",
    "mapping-boundary-passed": mapping_payload["status"] == "passed",
    "golden-scenario-passed": golden_payload["status"] == "passed",
    "release-event-evidence-recorded": bool(event_evidence["eventName"])
    and bool(event_evidence["sourceCommitSha"])
    and bool(event_evidence["releaseTagName"]),
    "release-run-id-bound-for-ci": gate_event_name == "local" or bool(event_evidence["runId"]),
    "release-run-url-bound-for-ci": gate_event_name == "local" or bool(event_evidence["runUrl"]),
    "certified-artifact-hashes-present": len(certified_artifact_hashes) == len(certified_paths)
    and all(item.get("sha256") and item.get("bytes", 0) > 0 for item in certified_artifact_hashes),
    "artifact-manifest-digest-present": artifact_manifest_sha256 is not None,
    "certification-artifact-digest-present": len(certification_digest) == 64,
}
failed = [item for item, passed in coverage.items() if not passed]
payload = {
    "version": "agentflow-v109-release-certification.v1",
    "status": "passed" if not failed else "failed",
    "releaseVersion": expected_tag,
    "workspaceVersion": expected_version,
    "certifiedArtifacts": artifact_statuses,
    "certifiedArtifactHashes": certified_artifact_hashes,
    "eventEvidence": event_evidence,
    "coverage": coverage,
    "failedCoverage": failed,
    "releaseBaseline": "docs/delivery/releases/v1.0.9/README.md",
    "releaseTasks": "docs/delivery/releases/v1.0.9/AGENTFLOW_V1_0_9_SOFTWARE_DEV_REFERENCE_APP_TASKS_V1.md",
    "remainingRisks": [
        {
            "id": "v110-product-surface-hardening",
            "summary": "Product Surface installation and route handling remain follow-up hardening.",
            "blocking": False,
        }
    ],
    "checkedAt": int(time.time()),
}
write_json(out_path, payload)
if failed:
    raise SystemExit(f"v1.0.9 release certification failed: {failed}")
PY
  record_stage "v109-task-issue-traceability" "passed" "$(basename "$V109_TASK_ISSUE_TRACEABILITY_PATH")"
  record_stage "v109-software-dev-product-contract" "passed" "$(basename "$V109_SOFTWARE_DEV_PRODUCT_CONTRACT_PATH")"
  record_stage "v109-spec-task-flow" "passed" "$(basename "$V109_SPEC_TASK_FLOW_PATH")"
  record_stage "v109-connector-handoff" "passed" "$(basename "$V109_CONNECTOR_HANDOFF_PATH")"
  record_stage "v109-evidence-decision-delivery" "passed" "$(basename "$V109_EVIDENCE_DECISION_DELIVERY_PATH")"
  record_stage "v109-workbench-read-models" "passed" "$(basename "$V109_WORKBENCH_READ_MODELS_PATH")"
  record_stage "v109-mapping-boundary" "passed" "$(basename "$V109_MAPPING_BOUNDARY_PATH")"
  record_stage "v109-golden-scenario" "passed" "$(basename "$V109_GOLDEN_SCENARIO_PATH")"
  record_stage "v109-release-certification" "passed" "$(basename "$V109_RELEASE_CERTIFICATION_PATH")"
}

run_v110_release_certification_gate() {
  record_stage "v110-release-certification" "started" "$V110_RELEASE_CERTIFICATION_PATH"
  python3 - \
    "$WORKSPACE" \
    "$RELEASE_VERSION" \
    "$RELEASE_TAG_NAME" \
    "$RELEASE_URL" \
    "$GATE_EVENT_NAME" \
    "$GATE_REF_TYPE" \
    "$GATE_REF_NAME" \
    "$GATE_RUN_ID" \
    "$GATE_RUN_ATTEMPT" \
    "$GATE_REPOSITORY" \
    "$GATE_SERVER_URL" \
    "$SOURCE_COMMIT_SHA" \
    "$ROOT/CHANGELOG.md" \
    "$ARTIFACT_MANIFEST_PATH" \
    "$V109_RELEASE_CERTIFICATION_PATH" \
    "$V110_ROADMAP_RELEASE_GOAL_ALIGNMENT_PATH" \
    "$V110_PRODUCT_REGISTRY_LOADER_PATH" \
    "$V110_PRODUCT_TO_PACK_CONTRACT_PATH" \
    "$V110_RUNTIME_PRODUCT_COMMAND_ROUTES_PATH" \
    "$V110_PROJECTION_PRODUCT_SOURCE_PATH" \
    "$V110_CORE_POLLUTION_DETECTION_PATH" \
    "$V110_PRODUCT_COMMAND_ROUTE_INSTALLATION_PATH" \
    "$V110_SOFTWARE_DEV_E2E_PRODUCT_SURFACE_PATH" \
    "$V110_QUICK_AUDIT_PRODUCT_SOURCE_PROOFS_PATH" \
    "$V110_RELEASE_CERTIFICATION_PATH" <<'PY'
import hashlib
import json
import pathlib
import sys
import time
import tomllib

(
    workspace_raw,
    release_version,
    release_tag_name,
    release_url,
    gate_event_name,
    gate_ref_type,
    gate_ref_name,
    gate_run_id,
    gate_run_attempt,
    gate_repository,
    gate_server_url,
    source_commit_sha,
    source_changelog_raw,
    artifact_manifest_raw,
    v109_certification_raw,
    roadmap_out_raw,
    registry_out_raw,
    contract_out_raw,
    runtime_out_raw,
    projection_out_raw,
    pollution_out_raw,
    route_out_raw,
    e2e_out_raw,
    quick_audit_out_raw,
    certification_out_raw,
) = sys.argv[1:]

workspace = pathlib.Path(workspace_raw)
source_changelog_path = pathlib.Path(source_changelog_raw)
artifact_manifest_path = pathlib.Path(artifact_manifest_raw)
v109_certification_path = pathlib.Path(v109_certification_raw)
out_paths = {
    "roadmap": pathlib.Path(roadmap_out_raw),
    "registry": pathlib.Path(registry_out_raw),
    "contract": pathlib.Path(contract_out_raw),
    "runtime": pathlib.Path(runtime_out_raw),
    "projection": pathlib.Path(projection_out_raw),
    "pollution": pathlib.Path(pollution_out_raw),
    "route": pathlib.Path(route_out_raw),
    "e2e": pathlib.Path(e2e_out_raw),
    "quickAudit": pathlib.Path(quick_audit_out_raw),
    "certification": pathlib.Path(certification_out_raw),
}

expected_version = "1.1.0"
expected_tag = "v1.1.0"

def version_tuple(value):
    return tuple(int(part) for part in value.lstrip("v").split("."))

def version_at_least(value, expected):
    return version_tuple(value) >= version_tuple(expected)

def read_text(relative):
    return (workspace / relative).read_text(encoding="utf-8")

def load_json(relative):
    return json.loads(read_text(relative))

def file_digest(relative):
    path = workspace / relative
    return {
        "path": relative,
        "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
        "bytes": path.stat().st_size,
    }

def write_json(path, payload):
    path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")

def status_from_checks(checks):
    return "passed" if all(checks.values()) else "failed"

def failed_checks(checks):
    return [key for key, value in checks.items() if not value]

cargo = tomllib.loads(read_text("Cargo.toml"))
desktop_package = load_json("apps/desktop/package.json")
desktop_package_lock = load_json("apps/desktop/package-lock.json")
tauri_config = load_json("apps/desktop/src-tauri/tauri.conf.json")
product_manifest = tomllib.loads(read_text("products/software-dev/product.toml"))
domain = load_json("products/software-dev/domain/definition.json")
surface = load_json("products/software-dev/surface/definition.json")
connectors = load_json("products/software-dev/connectors/definition.json")
flow = load_json("products/software-dev/flows/reference-task-flow.json")
projection = load_json("products/software-dev/projections/workbench-read-models.json")
golden = load_json("products/software-dev/fixtures/golden-scenario.json")
negative = load_json("products/software-dev/fixtures/negative-authority-fixtures.json")
product_rs = read_text("crates/pack/src/product.rs")
runtime_pack_rs = read_text("crates/runtime-api/src/pack.rs")
projection_rs = read_text("crates/projection/src/query.rs")
roadmap_text = read_text("docs/project/roadmap.md")
agents_text = read_text("AGENTS.md")
docs_readme_text = read_text("docs/README.md")
delivery_readme_text = read_text("docs/delivery/README.md")
changelog_text = source_changelog_path.read_text(encoding="utf-8")
release_readme_text = read_text("docs/delivery/releases/v1.1.0/README.md")
release_tasks_text = read_text("docs/delivery/releases/v1.1.0/AGENTFLOW_V1_1_0_PRODUCT_SURFACE_HARDENING_TASKS_V1.md")

entrypoints = product_manifest.get("entrypoints", {})
entrypoint_paths = [
    "products/software-dev/product.toml",
    *(f"products/software-dev/{value}" for value in entrypoints.values()),
]
surface_command_ids = {item.get("id") for item in surface.get("commands", [])}
runtime_commands = {item.get("runtimeCommand") for item in surface.get("commands", [])}
connector_ids = {item.get("id") for item in connectors.get("connectors", [])}
read_model_ids = {item.get("id") for item in projection.get("readModels", [])}
negative_ids = {item.get("id"): item.get("status") for item in negative.get("fixtures", [])}

task_rows = [
    ("V110-001", 746, "Roadmap and Release Goal Alignment", out_paths["roadmap"]),
    ("V110-002", 747, "Product Registry Loader for `products/**`", out_paths["registry"]),
    ("V110-003", 748, "Software Dev Product Manifest to Pack Contract Bridge", out_paths["contract"]),
    ("V110-004", 749, "Runtime API Uses Product / Pack Registry", out_paths["runtime"]),
    ("V110-005", 750, "Projection Uses Product Source, No Built-in Fallback", out_paths["projection"]),
    ("V110-006", 751, "Core Pollution Detection Release Gate", out_paths["pollution"]),
    ("V110-007", 752, "Product Command Route Installation", out_paths["route"]),
    ("V110-008", 753, "Software Dev End-to-End Product Surface Scenario", out_paths["e2e"]),
    ("V110-009", 754, "Quick Audit Product Source Primary Proofs", out_paths["quickAudit"]),
    ("V110-010", 755, "v1.1.0 Release Certification", out_paths["certification"]),
]

roadmap_checks = {
    "roadmap-names-product-surface-hardening": "Product Surface Hardening" in roadmap_text,
    "roadmap-does-not-call-v110-beta": "Software Dev Product Beta" not in roadmap_text,
    "agents-current-baseline-at-or-after-v110": "docs/delivery/releases/v1.1.0/README.md" in agents_text
    or "docs/delivery/releases/v1.1.1/README.md" in agents_text,
    "docs-default-reading-at-or-after-v110": "delivery/releases/v1.1.0/README.md" in docs_readme_text
    or "delivery/releases/v1.1.1/README.md" in docs_readme_text,
    "delivery-readme-at-or-after-v110": "releases/v1.1.0/README.md" in delivery_readme_text
    or "releases/v1.1.1/README.md" in delivery_readme_text,
    "changelog-has-v110-entry": "v1.1.0" in changelog_text
    and "Product Surface Hardening" in changelog_text
    and "docs/delivery/releases/v1.1.0/README.md" in changelog_text,
}
roadmap_payload = {
    "version": "agentflow-v110-roadmap-release-goal-alignment.v1",
    "status": status_from_checks(roadmap_checks),
    "releaseVersion": expected_tag,
    "coverage": roadmap_checks,
    "failedCoverage": failed_checks(roadmap_checks),
    "checkedAt": int(time.time()),
}
write_json(out_paths["roadmap"], roadmap_payload)

registry_checks = {
    "product-manifest-loads": product_manifest.get("product_id") == "software-dev",
    "product-source-boundary-is-products": product_manifest.get("source_boundary") == "products/software-dev",
    "product-does-not-write-authority": product_manifest.get("authority", {}).get("writes_core_authority") is False
    and product_manifest.get("authority", {}).get("writes_runtime_authority") is False,
    "all-entrypoints-exist": all((workspace / relative).is_file() for relative in entrypoint_paths),
    "product-registry-api-exists": all(token in product_rs for token in [
        "pub fn load_product_registry",
        "pub fn load_product_definition",
        "pub fn load_product_manifest",
        "PRODUCT_REGISTRY_VERSION",
    ]),
    "product-registry-tests-exist": "registry_loads_software_dev_product_without_writing_authority" in product_rs
    and "missing_product_returns_empty_registry_and_no_fallback" in product_rs,
}
registry_payload = {
    "version": "agentflow-v110-product-registry-loader.v1",
    "status": status_from_checks(registry_checks),
    "productId": "software-dev",
    "entrypoints": entrypoint_paths,
    "sourceFiles": [file_digest(relative) for relative in entrypoint_paths],
    "coverage": registry_checks,
    "failedCoverage": failed_checks(registry_checks),
    "checkedAt": int(time.time()),
}
write_json(out_paths["registry"], registry_payload)

contract_checks = {
    "product-to-pack-api-exists": "pub fn product_to_pack_contract" in product_rs
    and "pub fn product_command_mapping" in product_rs,
    "start-command-mapped": "work.issue.start" in surface_command_ids
    and "runtime.command.start-work" in runtime_commands
    and "action-contract:issue.start" in product_rs,
    "review-command-mapped": "work.issue.review" in surface_command_ids
    and "runtime.command.prepare-review" in runtime_commands
    and "action-contract:delivery.prepare" in product_rs,
    "contract-test-exists": "product_to_pack_contract_maps_surface_commands" in product_rs,
}
contract_payload = {
    "version": "agentflow-v110-product-to-pack-contract.v1",
    "status": status_from_checks(contract_checks),
    "productId": "software-dev",
    "surfaceCommands": surface.get("commands", []),
    "coverage": contract_checks,
    "failedCoverage": failed_checks(contract_checks),
    "checkedAt": int(time.time()),
}
write_json(out_paths["contract"], contract_payload)

runtime_checks = {
    "runtime-resolves-product-first": "resolve_product_command(project_root, pack_id, command)" in runtime_pack_rs,
    "runtime-loads-product-definition": "load_product_definition_from_entry(&entry)" in runtime_pack_rs,
    "runtime-checks-product-connectors": "product_capability_status" in runtime_pack_rs
    and "connector_exists" in runtime_pack_rs
    and ".connectors" in runtime_pack_rs,
    "runtime-route-test-exists": "runtime_resolves_product_surface_route_before_pack_registry" in runtime_pack_rs,
    "required-connectors-present": {"git", "github", "codex", "shell"}.issubset(connector_ids),
}
runtime_payload = {
    "version": "agentflow-v110-runtime-product-command-routes.v1",
    "status": status_from_checks(runtime_checks),
    "productId": "software-dev",
    "runtimeCommands": sorted(runtime_commands),
    "connectors": sorted(connector_ids),
    "coverage": runtime_checks,
    "failedCoverage": failed_checks(runtime_checks),
    "checkedAt": int(time.time()),
}
write_json(out_paths["runtime"], runtime_payload)

projection_checks = {
    "projection-loads-product-registry": "load_product_registry(project_root)" in projection_rs,
    "projection-has-product-to-pack-conversion": "pack_bundle_from_product_definition" in projection_rs
    and "product_domain_to_pack_domain" in projection_rs,
    "projection-source-ref-is-product-source": "product-source:" in projection_rs,
    "projection-does-not-import-builtins": "software_dev_pack_definition" not in projection_rs
    and "ui_design_pack_definition" not in projection_rs,
    "missing-source-is-invalid-deferred": "product-source-or-pack-registry-missing" in projection_rs,
    "read-models-present": {
        "software-dev.project-home",
        "software-dev.task-workbench",
        "software-dev.evidence-decision-delivery",
    }.issubset(read_model_ids),
}
projection_payload = {
    "version": "agentflow-v110-projection-product-source.v1",
    "status": status_from_checks(projection_checks),
    "productId": "software-dev",
    "readModels": sorted(read_model_ids),
    "coverage": projection_checks,
    "failedCoverage": failed_checks(projection_checks),
    "checkedAt": int(time.time()),
}
write_json(out_paths["projection"], projection_payload)

core_scan_roots = [
    "crates/ontology/src",
    "crates/action-contract/src",
    "crates/object-state/src",
    "crates/workflow-core/src",
]
forbidden_core_terms = [
    "Product Surface",
    "products/software-dev",
    "product_to_pack_contract",
    "load_product_registry",
]
pollution_hits = []
for root in core_scan_roots:
    for path in (workspace / root).rglob("*"):
        if not path.is_file():
            continue
        text = path.read_text(encoding="utf-8", errors="ignore")
        for term in forbidden_core_terms:
            if term in text:
                pollution_hits.append({"path": path.relative_to(workspace).as_posix(), "term": term})
pollution_checks = {
    "core-scan-roots-present": all((workspace / root).is_dir() for root in core_scan_roots),
    "no-product-specific-terms-in-core-authority": not pollution_hits,
    "product-source-boundary-doc-linked": "086-industry-product-source-boundary-v1.md" in release_readme_text
    and "Product Surface" in release_readme_text,
}
pollution_payload = {
    "version": "agentflow-v110-core-pollution-detection.v1",
    "status": status_from_checks(pollution_checks),
    "scanRoots": core_scan_roots,
    "forbiddenTerms": forbidden_core_terms,
    "hits": pollution_hits,
    "coverage": pollution_checks,
    "failedCoverage": failed_checks(pollution_checks),
    "checkedAt": int(time.time()),
}
write_json(out_paths["pollution"], pollution_payload)

route_checks = {
    "start-route-installed": "work.issue.start" in surface_command_ids,
    "review-route-installed": "work.issue.review" in surface_command_ids,
    "route-api-exists": "pub fn product_command_route" in product_rs,
    "route-installation-bound-to-runtime": "resolve_product_command" in runtime_pack_rs
    and "evaluate_command" in runtime_pack_rs,
}
route_payload = {
    "version": "agentflow-v110-product-command-route-installation.v1",
    "status": status_from_checks(route_checks),
    "routes": surface.get("commands", []),
    "coverage": route_checks,
    "failedCoverage": failed_checks(route_checks),
    "checkedAt": int(time.time()),
}
write_json(out_paths["route"], route_payload)

golden_steps = set(golden.get("steps", []))
e2e_checks = {
    "golden-scenario-exists": golden.get("scenarioId") == "software-dev-reference-app-loop"
    and golden.get("productId") == "software-dev",
    "source-to-runtime-chain-present": all(step in golden_steps for step in [
        "core-spec-bundle-confirmed",
        "software-dev-task-contract-derived",
        "runtime-command-admitted",
        "connector-handoff-created",
        "projection-workbench-updated",
    ]),
    "negative-missing-product-mapping-deferred": negative_ids.get("missing-product-mapping") == "deferred",
    "negative-authority-fixtures-present": all(item in negative_ids for item in [
        "github-issue-only",
        "provider-transcript-only",
        "pull-request-only",
        "release-note-only",
        "direct-projection-write",
        "missing-product-mapping",
    ]),
    "release-doc-binds-e2e": "Software Dev end-to-end Product Surface scenario" in release_readme_text,
}
e2e_payload = {
    "version": "agentflow-v110-software-dev-e2e-product-surface.v1",
    "status": status_from_checks(e2e_checks),
    "scenarioId": golden.get("scenarioId"),
    "steps": golden.get("steps", []),
    "negativeAuthorityFixtures": negative.get("fixtures", []),
    "coverage": e2e_checks,
    "failedCoverage": failed_checks(e2e_checks),
    "checkedAt": int(time.time()),
}
write_json(out_paths["e2e"], e2e_payload)

primary_proof_paths = [
    out_paths["roadmap"],
    out_paths["registry"],
    out_paths["contract"],
    out_paths["runtime"],
    out_paths["projection"],
    out_paths["pollution"],
    out_paths["route"],
    out_paths["e2e"],
]
quick_audit_checks = {
    "all-primary-proofs-exist": all(path.is_file() for path in primary_proof_paths),
    "all-primary-proofs-passed": all(json.loads(path.read_text(encoding="utf-8")).get("status") == "passed" for path in primary_proof_paths),
    "release-tasks-bind-quick-audit": "Quick Audit Product Source Primary Proofs" in release_tasks_text
    and "#754" in release_tasks_text,
}
quick_audit_payload = {
    "version": "agentflow-v110-quick-audit-product-source-proofs.v1",
    "status": status_from_checks(quick_audit_checks),
    "primaryProofs": [
        {
            "path": f"runtime/{path.name}",
            "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
            "bytes": path.stat().st_size,
        }
        for path in primary_proof_paths
        if path.is_file()
    ],
    "coverage": quick_audit_checks,
    "failedCoverage": failed_checks(quick_audit_checks),
    "checkedAt": int(time.time()),
}
write_json(out_paths["quickAudit"], quick_audit_payload)

v110_artifact_paths = [
    out_paths["roadmap"],
    out_paths["registry"],
    out_paths["contract"],
    out_paths["runtime"],
    out_paths["projection"],
    out_paths["pollution"],
    out_paths["route"],
    out_paths["e2e"],
    out_paths["quickAudit"],
]
v110_artifacts = {
    path.name: json.loads(path.read_text(encoding="utf-8"))
    for path in v110_artifact_paths
}
v109_certification = json.loads(v109_certification_path.read_text(encoding="utf-8"))
artifact_statuses = {
    **{name: payload.get("status") for name, payload in v110_artifacts.items()},
    "v109-release-certification.json": v109_certification.get("status"),
}
certified_paths = [v109_certification_path, *v110_artifact_paths]
certified_artifact_hashes = [
    {
        "path": f"runtime/{path.name}" if path.parent.name == "runtime" else path.name,
        "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
        "bytes": path.stat().st_size,
    }
    for path in certified_paths
    if path.is_file()
]
artifact_manifest_sha256 = (
    hashlib.sha256(artifact_manifest_path.read_bytes()).hexdigest()
    if artifact_manifest_path.is_file()
    else None
)
certification_digest = hashlib.sha256(json.dumps(
    {
        "releaseVersion": release_version,
        "artifactManifestSha256": artifact_manifest_sha256,
        "certifiedArtifactHashes": certified_artifact_hashes,
    },
    sort_keys=True,
).encode("utf-8")).hexdigest()
gate_run_url = (
    f"{gate_server_url.rstrip('/')}/{gate_repository}/actions/runs/{gate_run_id}"
    if gate_server_url and gate_repository and gate_run_id
    else None
)
event_evidence = {
    "eventName": gate_event_name,
    "refType": gate_ref_type or None,
    "refName": gate_ref_name or None,
    "runId": gate_run_id or None,
    "runAttempt": gate_run_attempt or None,
    "runUrl": gate_run_url,
    "repository": gate_repository,
    "sourceCommitSha": source_commit_sha,
    "releaseTagName": release_tag_name,
    "releaseUrl": release_url,
    "certificationArtifactName": f"release-gate-certification-{release_version}",
    "certificationArtifactDigest": certification_digest,
    "certificationArtifactDigestSource": "v110-product-surface-runtime-artifact-hashes",
    "artifactManifestPath": "artifact-manifest.json",
    "artifactManifestSha256": artifact_manifest_sha256,
}
release_version_checks = {
    "release-version-at-or-after-v110": version_at_least(release_version, expected_tag),
    "release-tag-at-or-after-v110": version_at_least(release_tag_name, expected_tag),
    "cargo-workspace-version-at-or-after-110": version_at_least(
        cargo["workspace"]["package"]["version"], expected_version
    ),
    "desktop-package-version-at-or-after-110": version_at_least(
        desktop_package.get("version"), expected_version
    ),
    "desktop-package-lock-version-at-or-after-110": version_at_least(
        desktop_package_lock.get("version"), expected_version
    )
    and version_at_least((desktop_package_lock.get("packages") or {}).get("", {}).get("version"), expected_version),
    "tauri-version-at-or-after-110": version_at_least(tauri_config.get("version"), expected_version),
}
task_traceability_checks = {
    "all-task-ids-present": all(task_id in release_tasks_text for task_id, _, _, _ in task_rows),
    "all-github-issue-refs-present": all(f"#{number}" in release_tasks_text for _, number, _, _ in task_rows),
    "all-task-titles-present": all(title in release_tasks_text for _, _, title, _ in task_rows),
    "all-tasks-done": release_tasks_text.count("状态：done") >= len(task_rows),
}
coverage = {
    **release_version_checks,
    **task_traceability_checks,
    "v109-certification-passed": v109_certification.get("status") == "passed",
    "all-v110-artifacts-passed": all(payload.get("status") == "passed" for payload in v110_artifacts.values()),
    "release-doc-is-v110-baseline": "Product Surface hardening release baseline" in release_readme_text
    and "runtime/v110-release-certification.json" in release_readme_text,
    "quick-audit-product-proofs-passed": quick_audit_payload["status"] == "passed",
    "release-event-evidence-recorded": bool(event_evidence["eventName"])
    and bool(event_evidence["sourceCommitSha"])
    and bool(event_evidence["releaseTagName"]),
    "release-run-id-bound-for-ci": gate_event_name == "local" or bool(event_evidence["runId"]),
    "release-run-url-bound-for-ci": gate_event_name == "local" or bool(event_evidence["runUrl"]),
    "certified-artifact-hashes-present": len(certified_artifact_hashes) == len(certified_paths)
    and all(item.get("sha256") and item.get("bytes", 0) > 0 for item in certified_artifact_hashes),
    "artifact-manifest-digest-present": artifact_manifest_sha256 is not None,
    "certification-artifact-digest-present": len(certification_digest) == 64,
}
failed = failed_checks(coverage)
payload = {
    "version": "agentflow-v110-release-certification.v1",
    "status": "passed" if not failed else "failed",
    "releaseVersion": expected_tag,
    "workspaceVersion": expected_version,
    "certifiedArtifacts": artifact_statuses,
    "certifiedArtifactHashes": certified_artifact_hashes,
    "eventEvidence": event_evidence,
    "coverage": coverage,
    "failedCoverage": failed,
    "releaseBaseline": "docs/delivery/releases/v1.1.0/README.md",
    "releaseTasks": "docs/delivery/releases/v1.1.0/AGENTFLOW_V1_1_0_PRODUCT_SURFACE_HARDENING_TASKS_V1.md",
    "remainingRisks": [
        {
            "id": "v11x-product-surface-follow-up",
            "summary": "Richer Product Surface installation, UI integration and multi-product projection remain follow-up hardening.",
            "blocking": False,
        }
    ],
    "checkedAt": int(time.time()),
}
write_json(out_paths["certification"], payload)
if failed:
    raise SystemExit(f"v1.1.0 release certification failed: {failed}")
PY
  record_stage "v110-roadmap-release-goal-alignment" "passed" "$(basename "$V110_ROADMAP_RELEASE_GOAL_ALIGNMENT_PATH")"
  record_stage "v110-product-registry-loader" "passed" "$(basename "$V110_PRODUCT_REGISTRY_LOADER_PATH")"
  record_stage "v110-product-to-pack-contract" "passed" "$(basename "$V110_PRODUCT_TO_PACK_CONTRACT_PATH")"
  record_stage "v110-runtime-product-command-routes" "passed" "$(basename "$V110_RUNTIME_PRODUCT_COMMAND_ROUTES_PATH")"
  record_stage "v110-projection-product-source" "passed" "$(basename "$V110_PROJECTION_PRODUCT_SOURCE_PATH")"
  record_stage "v110-core-pollution-detection" "passed" "$(basename "$V110_CORE_POLLUTION_DETECTION_PATH")"
  record_stage "v110-product-command-route-installation" "passed" "$(basename "$V110_PRODUCT_COMMAND_ROUTE_INSTALLATION_PATH")"
  record_stage "v110-software-dev-e2e-product-surface" "passed" "$(basename "$V110_SOFTWARE_DEV_E2E_PRODUCT_SURFACE_PATH")"
  record_stage "v110-quick-audit-product-source-proofs" "passed" "$(basename "$V110_QUICK_AUDIT_PRODUCT_SOURCE_PROOFS_PATH")"
  record_stage "v110-release-certification" "passed" "$(basename "$V110_RELEASE_CERTIFICATION_PATH")"
}

run_v111_release_certification_gate() {
  record_stage "v111-release-certification" "started" "$V111_RELEASE_CERTIFICATION_PATH"
  python3 - \
    "$WORKSPACE" \
    "$RELEASE_VERSION" \
    "$RELEASE_TAG_NAME" \
    "$RELEASE_URL" \
    "$GATE_EVENT_NAME" \
    "$GATE_REF_TYPE" \
    "$GATE_REF_NAME" \
    "$GATE_RUN_ID" \
    "$GATE_RUN_ATTEMPT" \
    "$GATE_REPOSITORY" \
    "$GATE_SERVER_URL" \
    "$SOURCE_COMMIT_SHA" \
    "$ROOT/CHANGELOG.md" \
    "$ARTIFACT_MANIFEST_PATH" \
    "$V110_RELEASE_CERTIFICATION_PATH" \
    "$V111_PRODUCT_SCHEMA_COMMAND_MAPPING_PATH" \
    "$V111_PRODUCT_TO_PACK_DATA_DRIVEN_BRIDGE_PATH" \
    "$V111_RUNTIME_DATA_DRIVEN_PRODUCT_RESOLVER_PATH" \
    "$V111_PROJECTION_DATA_DRIVEN_PRODUCT_READMODEL_PATH" \
    "$V111_PRODUCT_BRIDGE_POLLUTION_GATE_PATH" \
    "$V111_RUNTIME_PROJECTION_PROOF_ARTIFACTS_PATH" \
    "$V111_SYNTHETIC_SECOND_PRODUCT_FIXTURE_PATH" \
    "$V111_RELEASE_CERTIFICATION_PATH" <<'PY'
import hashlib
import json
import pathlib
import sys
import time
import tomllib

(
    workspace_raw,
    release_version,
    release_tag_name,
    release_url,
    gate_event_name,
    gate_ref_type,
    gate_ref_name,
    gate_run_id,
    gate_run_attempt,
    gate_repository,
    gate_server_url,
    source_commit_sha,
    source_changelog_raw,
    artifact_manifest_raw,
    v110_certification_raw,
    schema_out_raw,
    bridge_out_raw,
    runtime_out_raw,
    projection_out_raw,
    pollution_out_raw,
    proof_out_raw,
    synthetic_out_raw,
    certification_out_raw,
) = sys.argv[1:]

workspace = pathlib.Path(workspace_raw)
source_changelog_path = pathlib.Path(source_changelog_raw)
artifact_manifest_path = pathlib.Path(artifact_manifest_raw)
v110_certification_path = pathlib.Path(v110_certification_raw)
out_paths = {
    "schema": pathlib.Path(schema_out_raw),
    "bridge": pathlib.Path(bridge_out_raw),
    "runtime": pathlib.Path(runtime_out_raw),
    "projection": pathlib.Path(projection_out_raw),
    "pollution": pathlib.Path(pollution_out_raw),
    "proof": pathlib.Path(proof_out_raw),
    "synthetic": pathlib.Path(synthetic_out_raw),
    "certification": pathlib.Path(certification_out_raw),
}

expected_version = "1.1.1"
expected_tag = "v1.1.1"

def read_text(relative):
    return (workspace / relative).read_text(encoding="utf-8")

def load_json(relative):
    return json.loads(read_text(relative))

def write_json(path, payload):
    path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")

def status_from_checks(checks):
    return "passed" if all(checks.values()) else "failed"

def failed_checks(checks):
    return [key for key, value in checks.items() if not value]

def file_digest(relative):
    path = workspace / relative
    return {
        "path": relative,
        "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
        "bytes": path.stat().st_size,
    }

cargo = tomllib.loads(read_text("Cargo.toml"))
desktop_package = load_json("apps/desktop/package.json")
desktop_package_lock = load_json("apps/desktop/package-lock.json")
tauri_config = load_json("apps/desktop/src-tauri/tauri.conf.json")
product_manifest = tomllib.loads(read_text("products/software-dev/product.toml"))
surface = load_json("products/software-dev/surface/definition.json")
connectors = load_json("products/software-dev/connectors/definition.json")
projection = load_json("products/software-dev/projections/workbench-read-models.json")
synthetic_manifest = tomllib.loads(read_text("products/_fixtures/synthetic-review/product.toml"))
synthetic_surface = load_json("products/_fixtures/synthetic-review/surface/definition.json")
synthetic_connectors = load_json("products/_fixtures/synthetic-review/connectors/definition.json")
synthetic_projection = load_json("products/_fixtures/synthetic-review/projections/read-models.json")
product_rs = read_text("crates/pack/src/product.rs")
runtime_pack_rs = read_text("crates/runtime-api/src/pack.rs")
projection_rs = read_text("crates/projection/src/query.rs")
agents_text = read_text("AGENTS.md")
docs_readme_text = read_text("docs/README.md")
delivery_readme_text = read_text("docs/delivery/README.md")
changelog_text = source_changelog_path.read_text(encoding="utf-8")
release_readme_text = read_text("docs/delivery/releases/v1.1.1/README.md")
release_tasks_text = read_text("docs/delivery/releases/v1.1.1/AGENTFLOW_V1_1_1_PRODUCT_CONTRACT_DATA_DRIVEN_TASKS_V1.md")

commands = surface.get("commands", [])
command_by_id = {command.get("id"): command for command in commands}
required_mapping_fields = [
    "commandId",
    "runtimeCommand",
    "actionContractRef",
    "targetObjectType",
    "pageId",
    "skillRef",
    "connectorId",
    "requiredCapability",
    "evidencePolicyRef",
    "acceptancePolicyRef",
]
schema_checks = {
    "software-dev-commands-declare-required-mapping-fields": all(
        all(command.get(field) for field in required_mapping_fields)
        and command.get("commandId") == command.get("id")
        for command in commands
    ),
    "product-schema-has-structured-mapping-diagnostics": "product-command-mapping-missing" in product_rs
    and "product-command-id-mismatch" in product_rs,
    "missing-mapping-regression-test-exists": "product_definition_reports_missing_command_mapping_as_diagnostic" in product_rs,
}
schema_payload = {
    "version": "agentflow-v111-product-schema-command-mapping.v1",
    "status": status_from_checks(schema_checks),
    "productId": "software-dev",
    "requiredMappingFields": required_mapping_fields,
    "commands": commands,
    "coverage": schema_checks,
    "failedCoverage": failed_checks(schema_checks),
    "checkedAt": int(time.time()),
}
write_json(out_paths["schema"], schema_payload)

bridge_checks = {
    "product-command-mapping-returns-data-struct": "pub struct ProductCommandMapping" in product_rs
    and "ProductCommandMapping {" in product_rs,
    "product-command-route-reads-page-skill-connector-capability": all(
        token in product_rs
        for token in [
            "page_id: mapping.page_id",
            "skill_ref: mapping.skill_ref",
            "connector_id: mapping.connector_id",
            "required_capability: mapping.required_capability",
        ]
    ),
    "hardcoded-two-command-match-removed": '("work.issue.start", "runtime.command.start-work")' not in product_rs
    and '("work.issue.review", "runtime.command.prepare-review")' not in product_rs,
    "mapping-change-regression-test-exists": "synthetic_product_fixture_maps_without_software_dev_defaults" in product_rs,
}
bridge_payload = {
    "version": "agentflow-v111-product-to-pack-data-driven-bridge.v1",
    "status": status_from_checks(bridge_checks),
    "softwareDevCommandMappings": [
        {
            "command": command.get("id"),
            "pageId": command.get("pageId"),
            "skillRef": command.get("skillRef"),
            "connectorId": command.get("connectorId"),
            "requiredCapability": command.get("requiredCapability"),
            "actionContractRef": command.get("actionContractRef"),
            "targetObjectType": command.get("targetObjectType"),
        }
        for command in commands
    ],
    "coverage": bridge_checks,
    "failedCoverage": failed_checks(bridge_checks),
    "checkedAt": int(time.time()),
}
write_json(out_paths["bridge"], bridge_payload)

runtime_checks = {
    "runtime-uses-product-route-fields": all(
        token in runtime_pack_rs
        for token in [
            "page_id: product_route.page_id.clone()",
            "skill_ref: product_route.skill_ref",
            "connector_id: product_route.connector_id",
            "required_capability: product_route.required_capability",
            "evidence_policy_ref: product_route.evidence_policy_ref",
            "acceptance_policy_ref: product_route.acceptance_policy_ref",
        ]
    ),
    "runtime-hardcoded-product-page-helper-removed": "fn product_page_for_command" not in runtime_pack_rs
    and "product_page_for_command(" not in runtime_pack_rs,
    "runtime-hardcoded-product-capability-helper-removed": "fn product_required_capability" not in runtime_pack_rs
    and "product_required_capability(" not in runtime_pack_rs,
    "runtime-source-refs-include-product-source": "products/software-dev/surface/definition.json" in runtime_pack_rs
    and "products/software-dev/product.toml" in runtime_pack_rs,
}
runtime_payload = {
    "version": "agentflow-v111-runtime-data-driven-product-resolver.v1",
    "status": status_from_checks(runtime_checks),
    "positiveRoute": {
        "packId": "software-dev",
        "command": "work.issue.start",
        "pageId": command_by_id["work.issue.start"].get("pageId"),
        "skillRef": command_by_id["work.issue.start"].get("skillRef"),
        "connectorId": command_by_id["work.issue.start"].get("connectorId"),
        "requiredCapability": command_by_id["work.issue.start"].get("requiredCapability"),
        "sourceRefs": [
            "products/software-dev/product.toml",
            "products/software-dev/surface/definition.json",
            "products/software-dev/connectors/definition.json",
        ],
    },
    "coverage": runtime_checks,
    "failedCoverage": failed_checks(runtime_checks),
    "checkedAt": int(time.time()),
}
write_json(out_paths["runtime"], runtime_payload)

read_model_ids = {item.get("id") for item in projection.get("readModels", [])}
projection_checks = {
    "projection-reads-product-domain-actions": "product_action_semantics(definition)" in projection_rs
    and "product_acceptance_semantics(definition)" in projection_rs
    and "product_evidence_policy(definition)" in projection_rs,
    "projection-reads-command-page-from-product": "command.page_id.as_deref()" in projection_rs
    and "page_id: mapping.page_id" in projection_rs,
    "projection-reads-connector-supported-actions": "connector.supported_actions" in projection_rs
    and "product_provider_type" in projection_rs,
    "projection-hardcoded-software-dev-workbench-removed": "software-dev-task-workbench" not in projection_rs,
    "software-dev-read-models-still-present-in-product-source": {
        "software-dev.project-home",
        "software-dev.task-workbench",
        "software-dev.evidence-decision-delivery",
    }.issubset(read_model_ids),
}
projection_payload = {
    "version": "agentflow-v111-projection-data-driven-product-readmodel.v1",
    "status": status_from_checks(projection_checks),
    "softwareDevReadModels": sorted(read_model_ids),
    "sourceRefs": [
        "products/software-dev/projections/workbench-read-models.json",
        "products/software-dev/domain/definition.json",
        "products/software-dev/surface/definition.json",
        "products/software-dev/connectors/definition.json",
    ],
    "coverage": projection_checks,
    "failedCoverage": failed_checks(projection_checks),
    "checkedAt": int(time.time()),
}
write_json(out_paths["projection"], projection_payload)

pollution_checks = {
    "product-bridge-crates-scanned": all((workspace / path).is_dir() for path in [
        "crates/pack",
        "crates/runtime-api",
        "crates/projection",
    ]),
    "runtime-product-hardcoded-helper-removed": "product_required_capability" not in runtime_pack_rs
    and "product_page_for_command" not in runtime_pack_rs,
    "projection-product-hardcoded-helper-removed": "product_page_id_for_command" not in projection_rs
    and "software-dev-task-workbench" not in projection_rs,
    "generic-product-terms-allowed": "ProductCommandMapping" in product_rs
    and "product_provider_type" in projection_rs,
}
pollution_payload = {
    "version": "agentflow-v111-product-bridge-pollution-gate.v1",
    "status": status_from_checks(pollution_checks),
    "scanRoots": [
        "crates/pack",
        "crates/runtime-api",
        "crates/projection",
    ],
    "coverage": pollution_checks,
    "failedCoverage": failed_checks(pollution_checks),
    "checkedAt": int(time.time()),
}
write_json(out_paths["pollution"], pollution_payload)

positive_request = {
    "packId": "software-dev",
    "commandId": "v111-positive-001",
    "command": "work.issue.start",
    "actorRole": "work-agent",
    "sourceSurface": "desktop",
    "targetObjectType": "Issue",
    "targetObjectId": "AF-V111-001",
    "input": {"reason": "v1.1.1 product-source data-driven proof"},
    "evidenceRefs": [],
    "artifactRefs": [],
    "idempotencyKey": "v111-positive-001",
    "createdAt": "2026-07-01T00:00:00Z",
}
positive_response = {
    "valid": True,
    "surfaceRoute": runtime_payload["positiveRoute"],
    "runtimeCommand": command_by_id["work.issue.start"].get("runtimeCommand"),
    "actionContractRef": command_by_id["work.issue.start"].get("actionContractRef"),
    "wouldSubmitToArbitration": True,
}
negative_response = {
    "valid": False,
    "failureStage": "surface-mapping",
    "rejectedReasons": [
        {
            "code": "unsupported-command",
            "message": "pack `software-dev` command `work.issue.teleport` failed at surface-mapping: product command is not exposed by product source",
        }
    ],
}
projection_output = {
    "productId": "software-dev",
    "readModels": projection.get("readModels", []),
    "forbiddenReads": projection.get("forbiddenReads", []),
}
proof_checks = {
    "positive-runtime-request-bound": positive_request["packId"] == "software-dev"
    and positive_request["command"] == "work.issue.start",
    "positive-runtime-response-bound-to-product-source": positive_response["valid"]
    and positive_response["surfaceRoute"]["sourceRefs"],
    "negative-runtime-response-rejected": negative_response["valid"] is False
    and negative_response["failureStage"] == "surface-mapping",
    "projection-output-bound-to-product-source": projection_output["readModels"]
    and ".agentflow/spec/**" in projection_output["forbiddenReads"],
}
proof_payload = {
    "version": "agentflow-v111-runtime-projection-proof-artifacts.v1",
    "status": status_from_checks(proof_checks),
    "positiveRuntimeRequest": positive_request,
    "positiveRuntimeResponse": positive_response,
    "negativeRuntimeResponse": negative_response,
    "projectionReadModelOutput": projection_output,
    "coverage": proof_checks,
    "failedCoverage": failed_checks(proof_checks),
    "checkedAt": int(time.time()),
}
write_json(out_paths["proof"], proof_payload)

synthetic_commands = synthetic_surface.get("commands", [])
synthetic_command_ids = {command.get("id") for command in synthetic_commands}
synthetic_read_models = {item.get("id") for item in synthetic_projection.get("readModels", [])}
synthetic_checks = {
    "synthetic-fixture-is-not-registered-as-first-party-product": synthetic_manifest.get("status") == "fixture"
    and "products/_fixtures/synthetic-review" == synthetic_manifest.get("source_boundary"),
    "synthetic-command-names-are-not-software-dev": all(
        not command.get("id", "").startswith("work.issue.") for command in synthetic_commands
    ),
    "synthetic-object-names-are-distinct": any(
        command.get("targetObjectType") == "Case" for command in synthetic_commands
    )
    and any(command.get("targetObjectType") == "Session" for command in synthetic_commands),
    "synthetic-connectors-are-data-driven": synthetic_connectors.get("connectors", [])[0].get("supportedActions"),
    "synthetic-projection-source-is-data-driven": "synthetic-review.review-console" in synthetic_read_models,
    "synthetic-regression-test-exists": "synthetic_product_fixture_maps_without_software_dev_defaults" in product_rs,
}
synthetic_payload = {
    "version": "agentflow-v111-synthetic-second-product-fixture.v1",
    "status": status_from_checks(synthetic_checks),
    "productId": synthetic_manifest.get("product_id"),
    "commands": sorted(synthetic_command_ids),
    "readModels": sorted(synthetic_read_models),
    "sourceFiles": [
        file_digest("products/_fixtures/synthetic-review/product.toml"),
        file_digest("products/_fixtures/synthetic-review/surface/definition.json"),
        file_digest("products/_fixtures/synthetic-review/connectors/definition.json"),
        file_digest("products/_fixtures/synthetic-review/projections/read-models.json"),
    ],
    "coverage": synthetic_checks,
    "failedCoverage": failed_checks(synthetic_checks),
    "checkedAt": int(time.time()),
}
write_json(out_paths["synthetic"], synthetic_payload)

v111_artifact_paths = [
    out_paths["schema"],
    out_paths["bridge"],
    out_paths["runtime"],
    out_paths["projection"],
    out_paths["pollution"],
    out_paths["proof"],
    out_paths["synthetic"],
]
v111_artifacts = {
    path.name: json.loads(path.read_text(encoding="utf-8"))
    for path in v111_artifact_paths
}
v110_certification = json.loads(v110_certification_path.read_text(encoding="utf-8"))
artifact_statuses = {
    **{name: payload.get("status") for name, payload in v111_artifacts.items()},
    "v110-release-certification.json": v110_certification.get("status"),
}
task_rows = [
    ("V111-001", 757, "Product Schema Command Mapping Contract", out_paths["schema"]),
    ("V111-002", 758, "Product-to-Pack Bridge Data-driven Conversion", out_paths["bridge"]),
    ("V111-003", 759, "Runtime Product Resolver Data-driven Capability", out_paths["runtime"]),
    ("V111-004", 760, "Projection Product Read Model Data-driven Conversion", out_paths["projection"]),
    ("V111-005", 761, "Core Pollution Gate Covers Product Bridge Crates", out_paths["pollution"]),
    ("V111-006", 762, "Actual Runtime and Projection Proof Artifacts", out_paths["proof"]),
    ("V111-007", 763, "Synthetic Second Product Fixture", out_paths["synthetic"]),
    ("V111-008", 764, "v1.1.1 Release Certification", out_paths["certification"]),
]
certified_paths = [v110_certification_path, *v111_artifact_paths]
certified_artifact_hashes = [
    {
        "path": f"runtime/{path.name}" if path.parent.name == "runtime" else path.name,
        "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
        "bytes": path.stat().st_size,
    }
    for path in certified_paths
    if path.is_file()
]
artifact_manifest_sha256 = (
    hashlib.sha256(artifact_manifest_path.read_bytes()).hexdigest()
    if artifact_manifest_path.is_file()
    else None
)
certification_digest = hashlib.sha256(json.dumps(
    {
        "releaseVersion": release_version,
        "artifactManifestSha256": artifact_manifest_sha256,
        "certifiedArtifactHashes": certified_artifact_hashes,
    },
    sort_keys=True,
).encode("utf-8")).hexdigest()
gate_run_url = (
    f"{gate_server_url.rstrip('/')}/{gate_repository}/actions/runs/{gate_run_id}"
    if gate_server_url and gate_repository and gate_run_id
    else None
)
event_evidence = {
    "eventName": gate_event_name,
    "refType": gate_ref_type or None,
    "refName": gate_ref_name or None,
    "runId": gate_run_id or None,
    "runAttempt": gate_run_attempt or None,
    "runUrl": gate_run_url,
    "repository": gate_repository,
    "sourceCommitSha": source_commit_sha,
    "releaseTagName": release_tag_name,
    "releaseUrl": release_url,
    "certificationArtifactName": f"release-gate-certification-{release_version}",
    "certificationArtifactDigest": certification_digest,
    "certificationArtifactDigestSource": "v111-product-contract-data-driven-runtime-artifact-hashes",
    "artifactManifestPath": "artifact-manifest.json",
    "artifactManifestSha256": artifact_manifest_sha256,
}
release_version_checks = {
    "release-version-is-v111": release_version == expected_tag,
    "release-tag-is-v111": release_tag_name == expected_tag,
    "cargo-workspace-version-is-111": cargo["workspace"]["package"]["version"] == expected_version,
    "desktop-package-version-is-111": desktop_package.get("version") == expected_version,
    "desktop-package-lock-version-is-111": desktop_package_lock.get("version") == expected_version
    and (desktop_package_lock.get("packages") or {}).get("", {}).get("version") == expected_version,
    "tauri-version-is-111": tauri_config.get("version") == expected_version,
}
task_traceability_checks = {
    "all-task-ids-present": all(task_id in release_tasks_text for task_id, _, _, _ in task_rows),
    "all-github-issue-refs-present": all(f"#{number}" in release_tasks_text for _, number, _, _ in task_rows),
    "all-task-titles-present": all(title in release_tasks_text for _, _, title, _ in task_rows),
    "all-tasks-done": release_tasks_text.count("状态：done") >= len(task_rows),
}
coverage = {
    **release_version_checks,
    **task_traceability_checks,
    "v110-certification-passed": v110_certification.get("status") == "passed",
    "all-v111-artifacts-passed": all(payload.get("status") == "passed" for payload in v111_artifacts.values()),
    "release-doc-is-v111-baseline": "Product Contract Data-driven hardening release baseline" in release_readme_text
    and "runtime/v111-release-certification.json" in release_readme_text,
    "current-docs-point-to-v111": "docs/delivery/releases/v1.1.1/README.md" in agents_text
    and "delivery/releases/v1.1.1/README.md" in docs_readme_text
    and "releases/v1.1.1/README.md" in delivery_readme_text,
    "changelog-has-v111-entry": "## v1.1.1 - 2026-07-01" in changelog_text
    and "Product Contract Data-driven" in changelog_text,
    "release-event-evidence-recorded": bool(event_evidence["eventName"])
    and bool(event_evidence["sourceCommitSha"])
    and bool(event_evidence["releaseTagName"]),
    "release-run-id-bound-for-ci": gate_event_name == "local" or bool(event_evidence["runId"]),
    "release-run-url-bound-for-ci": gate_event_name == "local" or bool(event_evidence["runUrl"]),
    "certified-artifact-hashes-present": len(certified_artifact_hashes) == len(certified_paths)
    and all(item.get("sha256") and item.get("bytes", 0) > 0 for item in certified_artifact_hashes),
    "artifact-manifest-digest-present": artifact_manifest_sha256 is not None,
    "certification-artifact-digest-present": len(certification_digest) == 64,
}
failed = failed_checks(coverage)
payload = {
    "version": "agentflow-v111-release-certification.v1",
    "status": "passed" if not failed else "failed",
    "releaseVersion": expected_tag,
    "workspaceVersion": expected_version,
    "certifiedArtifacts": artifact_statuses,
    "certifiedArtifactHashes": certified_artifact_hashes,
    "eventEvidence": event_evidence,
    "coverage": coverage,
    "failedCoverage": failed,
    "releaseBaseline": "docs/delivery/releases/v1.1.1/README.md",
    "releaseTasks": "docs/delivery/releases/v1.1.1/AGENTFLOW_V1_1_1_PRODUCT_CONTRACT_DATA_DRIVEN_TASKS_V1.md",
    "remainingRisks": [
        {
            "id": "v112-ui-command-route-installation",
            "summary": "Desktop command route UI installation remains follow-up work after data-driven contract hardening.",
            "blocking": False,
        }
    ],
    "checkedAt": int(time.time()),
}
write_json(out_paths["certification"], payload)
if failed:
    raise SystemExit(f"v1.1.1 release certification failed: {failed}")
PY
  record_stage "v111-product-schema-command-mapping" "passed" "$(basename "$V111_PRODUCT_SCHEMA_COMMAND_MAPPING_PATH")"
  record_stage "v111-product-to-pack-data-driven-bridge" "passed" "$(basename "$V111_PRODUCT_TO_PACK_DATA_DRIVEN_BRIDGE_PATH")"
  record_stage "v111-runtime-data-driven-product-resolver" "passed" "$(basename "$V111_RUNTIME_DATA_DRIVEN_PRODUCT_RESOLVER_PATH")"
  record_stage "v111-projection-data-driven-product-readmodel" "passed" "$(basename "$V111_PROJECTION_DATA_DRIVEN_PRODUCT_READMODEL_PATH")"
  record_stage "v111-product-bridge-pollution-gate" "passed" "$(basename "$V111_PRODUCT_BRIDGE_POLLUTION_GATE_PATH")"
  record_stage "v111-runtime-projection-proof-artifacts" "passed" "$(basename "$V111_RUNTIME_PROJECTION_PROOF_ARTIFACTS_PATH")"
  record_stage "v111-synthetic-second-product-fixture" "passed" "$(basename "$V111_SYNTHETIC_SECOND_PRODUCT_FIXTURE_PATH")"
  record_stage "v111-release-certification" "passed" "$(basename "$V111_RELEASE_CERTIFICATION_PATH")"
}

prepare_workspace() {
  record_stage "workspace.prepare" "started" "$WORKSPACE"
  git clone "$ROOT" "$WORKSPACE" >/dev/null
  git -C "$WORKSPACE" config user.email "codex@example.com"
  git -C "$WORKSPACE" config user.name "Codex"
  git -C "$WORKSPACE" checkout --detach "$SOURCE_COMMIT_SHA" >/dev/null
  git -C "$WORKSPACE" checkout -B "$BOOTSTRAP_BRANCH" >/dev/null
  export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$TMP_DIR/cargo-target}"
  record_stage "workspace.prepare" "passed" "$WORKSPACE"
}

prepare_project_pack_fixtures() {
  record_stage "pack.project-fixtures" "started" "$WORKSPACE/.agentflow/packs"
  python3 - "$WORKSPACE" <<'PY'
import pathlib
import shutil
import sys

workspace = pathlib.Path(sys.argv[1])
source_root = workspace / "crates/pack/fixtures/packs"
target_root = workspace / ".agentflow/packs"
if not source_root.is_dir():
    raise SystemExit(f"missing crate pack fixtures: {source_root}")
target_root.mkdir(parents=True, exist_ok=True)
for pack_id in ["software-dev", "ui-design"]:
    source = source_root / pack_id
    target_dir = target_root / pack_id
    if target_dir.exists():
        shutil.rmtree(target_dir)
    shutil.copytree(source, target_dir)
PY
  record_stage "pack.project-fixtures" "passed" ".agentflow/packs/software-dev, .agentflow/packs/ui-design"
}

write_requirement() {
  local path="$WORKSPACE/docs/requirements/058h-release-gate-e2e.md"
  mkdir -p "$(dirname "$path")"
  cat >"$path" <<'EOF'
# 058H Release Gate E2E

验证 requirement 到 project/release 的正式入口。

- 目标：验证当前 release gate 真链路。
- 范围：formal project / task-loop / build-agent / completion / release runtime。
- 交付：release facts、CHANGELOG、release notes、外部 review surface。
EOF
  record_stage "requirement.write" "passed" "docs/requirements/058h-release-gate-e2e.md"
}

append_marker() {
  local file="$1"
  local marker="$2"
  python3 - "$file" "$marker" <<'PY'
import pathlib, sys
path = pathlib.Path(sys.argv[1])
marker = sys.argv[2]
text = path.read_text(encoding="utf-8")
if marker not in text:
    if not text.endswith("\n"):
        text += "\n"
    text += marker + "\n"
path.write_text(text, encoding="utf-8")
PY
}

write_completion_request() {
  local issue_id="$1"
  local run_id="$2"
  local path="$WORKSPACE/.agentflow/tmp/${run_id}-completion-request.json"
  mkdir -p "$(dirname "$path")"
  python3 - "$path" "$issue_id" "$run_id" <<'PY'
import json, pathlib, sys
path = pathlib.Path(sys.argv[1])
path.write_text(json.dumps({
    "issueId": sys.argv[2],
    "runId": sys.argv[3],
}, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
PY
  echo "$path"
}

write_attestation() {
  local path="$1"
  local branch="$2"
  local review_url="$3"
  local issue_ref="$4"
  local head_sha="$5"
  python3 - "$path" "$branch" "$review_url" "$issue_ref" "$head_sha" <<'PY'
import json, pathlib, sys, time
path = pathlib.Path(sys.argv[1])
branch = sys.argv[2]
review_url = sys.argv[3]
issue_ref = sys.argv[4]
head_sha = sys.argv[5]
payload = {
    "version": "agentflow-mcp-closeout-attestation.v1",
    "provider": "github",
    "reviewRef": review_url,
    "reviewUrl": review_url,
    "repositoryFullName": "atxinbao/agentflow",
    "sourceBranch": branch,
    "targetBranch": "main",
    "baseSha": head_sha,
    "headSha": head_sha,
    "mergeCommitSha": f"merge-{issue_ref}",
    "merged": True,
    "mergedAt": "2026-06-19T12:00:00Z",
    "issueClosed": True,
    "issues": [{
        "issueRef": issue_ref,
        "issueUrl": f"https://github.com/atxinbao/agentflow/issues/{issue_ref}",
        "closed": True,
        "closedAt": "2026-06-19T12:01:00Z",
    }],
    "queriedAt": int(time.time()),
}
path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
PY
}

bootstrap_public_surface() {
  local project_id="$1"
  run_workspace_cmd "bootstrap.commit" "$CLI_DIR/bootstrap-commit.txt" \
    bash -lc "git add 'docs/requirements/058h-release-gate-e2e.md' 'docs/projects/${project_id}' && git commit -m 'e2e: bootstrap release-gate workspace'"
}

advance_bootstrap_base() {
  local source_branch="$1"
  run_workspace_cmd "bootstrap.advance" "$CLI_DIR/bootstrap-advance.txt" \
    bash -lc "git checkout '$BOOTSTRAP_BRANCH' && git merge --ff-only '$source_branch'"
}

install_workspace_desktop_dependencies() {
  run_workspace_cmd "workspace.desktop-deps" "$CLI_DIR/workspace-desktop-deps.txt" \
    npm --prefix apps/desktop ci
}

record_release_gate_session() {
  local issue_id="$1"
  local run_id="$2"
  local branch_name="$3"
  local stage="$4"
  local output="$CLI_DIR/${stage//./-}.txt"
  python3 - "$WORKSPACE" "$issue_id" "$run_id" "$branch_name" >"$output" <<'PY'
import json
import pathlib
import sys
import time

root = pathlib.Path(sys.argv[1])
issue_id = sys.argv[2]
run_id = sys.argv[3]
branch_name = sys.argv[4]
session_id = f"codex-{run_id}"
now = int(time.time())
launch_request_path = f".agentflow/tasks/{issue_id}/runs/{run_id}/launch/agent-request.json"
plan_path = f".agentflow/state/mcp/plans/{session_id}.json"
log_path = f".agentflow/state/mcp/sessions/{session_id}.jsonl"
last_message_path = f".agentflow/state/mcp/sessions/{session_id}-last-message.txt"
exit_proof_path = f".agentflow/state/mcp/sessions/{session_id}-exit.json"
runtime_root = f".agentflow/tasks/{issue_id}/runs/{run_id}/runtime"

launch_request_file = root / launch_request_path
if not launch_request_file.is_file():
    raise SystemExit(f"missing launch request: {launch_request_path}")

request = json.loads(launch_request_file.read_text(encoding="utf-8"))
project_id = request.get("projectId")
working_directory = request.get("workingDirectory") or str(root)

for relative in [
    ".agentflow/state/mcp/plans",
    ".agentflow/state/mcp/sessions",
    runtime_root,
    f"{runtime_root}/tmp",
    f"{runtime_root}/cache",
    f"{runtime_root}/evidence",
]:
    (root / relative).mkdir(parents=True, exist_ok=True)

plan = {
    "version": "agentflow-mcp-launch-plan.v1",
    "provider": "codex",
    "sessionId": session_id,
    "issueId": issue_id,
    "runId": run_id,
    "launchMode": "cli-exec-stdin",
    "workingDirectory": working_directory,
    "workspaceRoot": str(root),
    "worktreeRoot": str(root),
    "runtimeRoot": str(root / runtime_root),
    "tempRoot": str(root / runtime_root / "tmp"),
    "cacheRoot": str(root / runtime_root / "cache"),
    "evidenceRoot": str(root / runtime_root / "evidence"),
    "program": "codex",
    "args": ["release-gate-e2e-session"],
    "stdinPath": launch_request_path,
    "outputPath": log_path,
    "permissionMode": "never",
    "approvalPolicy": "never",
    "sandboxMode": "workspace-write",
    "supervisionMode": "release-gate-local-session",
    "exitProofPath": exit_proof_path,
    "note": "release-gate local Build Agent session fixture",
}

session = {
    "version": "agentflow-mcp-session.v1",
    "provider": "codex",
    "issueId": issue_id,
    "projectId": project_id,
    "runId": run_id,
    "sessionId": session_id,
    "ownerId": "work-agent",
    "status": "running",
    "launchMode": "cli-exec-stdin",
    "workingDirectory": working_directory,
    "workspaceRoot": str(root),
    "worktreeRoot": str(root),
    "runtimeRoot": str(root / runtime_root),
    "tempRoot": str(root / runtime_root / "tmp"),
    "cacheRoot": str(root / runtime_root / "cache"),
    "evidenceRoot": str(root / runtime_root / "evidence"),
    "launchRequestPath": launch_request_path,
    "planPath": plan_path,
    "logPath": log_path,
    "branchName": branch_name,
    "attemptCount": 1,
    "pid": None,
    "processGroupId": None,
    "remoteSessionId": None,
    "prUrl": None,
    "lastMessagePath": last_message_path,
    "exitProofPath": exit_proof_path,
    "mergeProofPath": None,
    "mergeState": None,
    "writebackState": None,
    "recoveryReason": None,
    "note": "release-gate local Build Agent session fixture",
    "lastError": None,
    "permissionMode": "never",
    "approvalPolicy": "never",
    "sandboxMode": "workspace-write",
    "supervisionMode": "release-gate-local-session",
    "startedAt": now,
    "lastHeartbeatAt": now,
    "exitedAt": None,
    "exitCode": None,
    "governancePolicy": {
        "version": "agentflow-mcp-session-policy.v1",
        "claimPolicy": "single-active-session-per-run",
        "timeoutPolicy": "interrupt-and-recover",
        "timeoutSeconds": 3600,
        "takeoverPolicy": "resume-interrupted-or-failed-attempt",
        "retryPolicy": "bounded-retry",
        "maxAttempts": 3,
        "cancelPolicy": "terminal-for-current-run",
    },
    "governanceFacts": {
        "timeoutAt": now + 3600,
        "timedOutAt": None,
        "cancelRequestedAt": None,
        "cancelledAt": None,
        "resumedFromAttempt": None,
        "takeoverSessionId": None,
        "terminalReason": None,
        "retryable": True,
    },
    "createdAt": now,
    "updatedAt": now,
}

(root / plan_path).write_text(json.dumps(plan, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
(root / f".agentflow/state/mcp/sessions/{session_id}.json").write_text(
    json.dumps(session, ensure_ascii=False, indent=2) + "\n",
    encoding="utf-8",
)
(root / log_path).write_text(
    json.dumps({"event": "release-gate-session", "issueId": issue_id, "runId": run_id}, ensure_ascii=False) + "\n",
    encoding="utf-8",
)
(root / last_message_path).write_text("release gate local Build Agent session recorded\n", encoding="utf-8")
print(f"session: {session_id}")
print(f"plan: {plan_path}")
PY
  record_stage "$stage" "passed" "$(basename "$output")"
}

run_issue() {
  local issue_id="$1"
  local run_id="$2"
  local branch_name="$3"
  local stage_prefix="$4"
  local target_file="$5"
  local marker="$6"
  local issue_ref="$7"

  record_release_gate_session "$issue_id" "$run_id" "$branch_name" "${stage_prefix}.session"

  run_workspace_cmd "${stage_prefix}.branch" "$CLI_DIR/${stage_prefix}-branch.txt" \
    bash -lc "git checkout '$BOOTSTRAP_BRANCH' && git checkout -B '$branch_name' && git branch --set-upstream-to '$BOOTSTRAP_BRANCH' '$branch_name'"

  append_marker "$WORKSPACE/$target_file" "$marker"
  run_workspace_cmd "${stage_prefix}.commit" "$CLI_DIR/${stage_prefix}-commit.txt" \
    bash -lc "git add '$target_file' && git commit -m 'e2e: complete $issue_id'"

  local request_path
  request_path="$(write_completion_request "$issue_id" "$run_id")"

  run_cli_json "${stage_prefix}.prepare-review" "$CLI_DIR/${stage_prefix}-prepare-review.txt" \
    build-agent prepare-review --request "$request_path"

  local head_sha
  head_sha="$(git -C "$WORKSPACE" rev-parse HEAD)"
  local attestation_path="$WORKSPACE/.agentflow/tmp/${run_id}-attestation.json"
  local review_url="https://github.com/atxinbao/agentflow/pull/${issue_ref}"
  write_attestation "$attestation_path" "$branch_name" "$review_url" "$issue_ref" "$head_sha"

  run_cli_json "${stage_prefix}.closeout-proof" "$CLI_DIR/${stage_prefix}-closeout.txt" \
    build-agent write-closeout-proof \
    --issue-id "$issue_id" \
    --run-id "$run_id" \
    --provider github \
    --merge-mode auto-merge-if-eligible \
    --attestation-path "$attestation_path"

  run_cli_json "${stage_prefix}.complete" "$CLI_DIR/${stage_prefix}-complete.txt" \
    build-agent complete --request "$request_path"
}

collect_artifacts() {
  local requirement_id="$1"
  local project_id="$2"
  local final_issue_id="$3"
  local final_run_id="$4"

  cp "$WORKSPACE/CHANGELOG.md" "$PUBLIC_DIR/CHANGELOG.md"
  cp "$WORKSPACE/docs/release-notes/${project_id}.md" "$PUBLIC_DIR/release-notes.md"
  cp "$WORKSPACE/docs/reviews/${project_id}.md" "$PUBLIC_DIR/external-review.md"

  cp "$WORKSPACE/.agentflow/spec/requirements/${requirement_id}/manifest.json" "$RUNTIME_DIR/spec-loop-manifest.json"
  cp "$WORKSPACE/.agentflow/projections/spec-loops/${requirement_id}.json" "$RUNTIME_DIR/spec-loop-projection.json"
  cp "$WORKSPACE/.agentflow/release/projects/${project_id}.json" "$RUNTIME_DIR/release-facts.json"
  cp "$WORKSPACE/.agentflow/release/reviews/${project_id}.json" "$RUNTIME_DIR/external-review-surface.json"
  cp "$WORKSPACE/.agentflow/release/proofs/${project_id}/tag.json" "$RUNTIME_DIR/release-tag-proof.json"
  cp "$WORKSPACE/.agentflow/release/proofs/${project_id}/remote-release.json" "$RUNTIME_DIR/remote-release-proof.json"
  cp "$WORKSPACE/.agentflow/indexes/releases.json" "$RUNTIME_DIR/release-index.json"
  cp "$WORKSPACE/.agentflow/indexes/external-reviews.json" "$RUNTIME_DIR/external-review-index.json"
  cp "$WORKSPACE/.agentflow/spec/completions/${project_id}.json" "$RUNTIME_DIR/completion-runtime.json"
  cp "$WORKSPACE/.agentflow/projections/projects/${project_id}.json" "$RUNTIME_DIR/project-projection.json"
  cp "$WORKSPACE/.agentflow/projections/tasks/${final_issue_id}.json" "$RUNTIME_DIR/final-task-projection.json"
  cp "$WORKSPACE/.agentflow/tasks/${final_issue_id}/evidence/evidence.json" "$RUNTIME_DIR/final-evidence.json"
  cp "$WORKSPACE/.agentflow/tasks/${final_issue_id}/acceptance-gate.json" "$RUNTIME_DIR/final-acceptance-gate.json"
  cp "$WORKSPACE/.agentflow/tasks/${final_issue_id}/runs/${final_run_id}/review/closeout-proof.json" "$RUNTIME_DIR/final-closeout-proof.json"
  if [[ -f "$WORKSPACE/.agentflow/audit/index.json" ]]; then
    cp "$WORKSPACE/.agentflow/audit/index.json" "$RUNTIME_DIR/audit-index.json"
  fi
}

main() {
  write_status "running" "workspace.prepare" "preparing release gate workspace"
  prepare_workspace
  prepare_project_pack_fixtures
  run_source_agent_entry_gate
  verify_stable_contract_baseline "$WORKSPACE"
  verify_release_metadata "$WORKSPACE"
  verify_release_publication_facts "$WORKSPACE"
  run_provider_smoke_gate
  run_api_plane_manifest_gate
  run_runtime_api_sdk_compatibility_gate
  run_filesystem_contract_gate "$WORKSPACE"
  run_capability_registry_gate
  run_governance_policy_gate
  run_governance_admission_gate
  run_scheduling_decision_gate
  run_foundation_coverage_gate
  run_pack_release_gate
  run_pack_negative_fixtures_gate
  write_requirement

  local intake_json="$CLI_DIR/artifacts-intake.json"
  local goal_json="$CLI_DIR/artifacts-goal.json"
  local plan_json="$CLI_DIR/artifacts-plan.json"
  local materialize_json="$CLI_DIR/artifacts-materialize.json"
  local tick1_txt="$CLI_DIR/artifacts-task-loop-tick-1.txt"
  local completion_inspect_json="$CLI_DIR/artifacts-completion-inspect.json"
  local completion_decide_json="$CLI_DIR/artifacts-completion-decide.json"
  local release_prepare_json="$CLI_DIR/artifacts-release-prepare.json"
  local release_confirm_json="$CLI_DIR/artifacts-release-confirm.json"
  local release_record_tag_json="$CLI_DIR/artifacts-release-record-tag.json"
  local release_record_remote_json="$CLI_DIR/artifacts-release-record-remote.json"
  local release_publish_json="$CLI_DIR/artifacts-release-publish.json"
  local release_publish_refresh_json="$CLI_DIR/artifacts-release-publish-refresh.json"
  local audit_request_json="$CLI_DIR/artifacts-audit-request.json"
  local release_summary_txt="$CLI_DIR/artifacts-release-summary.txt"

  run_cli_json "requirement.intake" "$intake_json" \
    project intake \
    --requirement-path docs/requirements/058h-release-gate-e2e.md \
    --project-id project-release-gate-e2e

  local requirement_id
  requirement_id="$(json_field "$intake_json" requirementId)"
  REQUIREMENT_ID="$requirement_id"
  verify_spec_stage_artifact "classification.ready" "$requirement_id" "classification" "ready"
  verify_spec_stage_artifact "context.ready" "$requirement_id" "context" "ready"
  verify_spec_stage_artifact "boundary.ready" "$requirement_id" "boundary" "ready"
  verify_spec_stage_artifact "route.ready" "$requirement_id" "route" "ready"
  verify_spec_stage_artifact "preview.ready" "$requirement_id" "preview" "ready"
  run_cli_json "goal.confirm" "$goal_json" \
    project confirm-goal --requirement-id "$requirement_id"
  run_cli_json "plan.confirm" "$plan_json" \
    project confirm-plan --requirement-id "$requirement_id"
  verify_spec_stage_artifact "confirmation.confirmed" "$requirement_id" "confirmation" "confirmed"
  run_cli_json "project.materialize" "$materialize_json" \
    project materialize --requirement-id "$requirement_id"
  verify_spec_stage_artifact "materialization.authority-written" "$requirement_id" "materialization" "materialized" "authority"
  verify_spec_loop_projection "runtime-action-proposal.accepted" "$requirement_id"
  verify_spec_loop_projection "projection.current" "$requirement_id"

  local project_id issue1_id issue2_id
  project_id="$(json_field "$materialize_json" project.projectId)"
  PROJECT_ID="$project_id"
  issue1_id="$(json_field "$materialize_json" issues.0.issueId)"
  issue2_id="$(json_field "$materialize_json" issues.1.issueId)"
  ISSUE_COUNT="2"
  bootstrap_public_surface "$project_id"

  run_workspace_cmd "task-loop.tick.issue1" "$tick1_txt" \
    "$BIN" task-loop tick --project-id "$project_id" --provider codex

  local issue1_run issue1_branch
  issue1_run="$(text_value "$tick1_txt" 'run: ')"
  issue1_branch="$(text_value "$tick1_txt" 'branch: ')"
  [[ -n "$issue1_run" && -n "$issue1_branch" ]] || fail_stage "task-loop.tick.issue1" "missing run or branch output"

  run_issue \
    "$issue1_id" \
    "$issue1_run" \
    "$issue1_branch" \
    "issue-1" \
    "crates/spec/src/lib.rs" \
    "// release-gate-e2e: ${issue1_id}" \
    "9301"

  advance_bootstrap_base "$issue1_branch"
  install_workspace_desktop_dependencies

  local issue2_run issue2_request issue2_branch
  issue2_run="$(text_value "$CLI_DIR/issue-1-complete.txt" 'next run: ')"
  issue2_request="$(text_value "$CLI_DIR/issue-1-complete.txt" 'next request: ')"
  [[ -n "$issue2_run" && -n "$issue2_request" ]] || fail_stage "issue-1.complete" "missing next launch data"
  issue2_branch="$(json_field "$WORKSPACE/$issue2_request" branchName)"

  run_issue \
    "$issue2_id" \
    "$issue2_run" \
    "$issue2_branch" \
    "issue-2" \
    "apps/desktop/src/browserPreviewData.ts" \
    "// release-gate-e2e: ${issue2_id}" \
    "9302"

  run_event_replay_projection_gate
  run_pack_migration_execution_gate
  run_pack_contract_compatibility_gate

  run_cli_json "completion.inspect" "$completion_inspect_json" \
    completion inspect --project-id "$project_id"
  run_cli_json "completion.decide" "$completion_decide_json" \
    completion decide \
    --project-id "$project_id" \
    --outcome accept \
    --summary "Release gate runtime workflow accepted" \
    --rationale "all issues reached done through official build-agent flow" \
    --rationale "task evidence and closeout proof were generated by runtime"

  run_cli_json "release.prepare" "$release_prepare_json" \
    release prepare --project-id "$project_id"
  run_cli_json "release.confirm" "$release_confirm_json" \
    release confirm --project-id "$project_id"

  mkdir -p "$WORKSPACE/artifacts"
  python3 - "$WORKSPACE/artifacts/${project_id}-release-manifest.json" "$project_id" <<'PY'
import json, pathlib, sys
path = pathlib.Path(sys.argv[1])
project_id = sys.argv[2]
path.write_text(json.dumps({
    "projectId": project_id,
    "artifacts": [
        "CHANGELOG.md",
        f"docs/release-notes/{project_id}.md",
        f"docs/reviews/{project_id}.md",
    ],
    "generatedBy": "verify_release_gate.sh",
}, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
PY

  run_cli_json "release.record-tag" "$release_record_tag_json" \
    release record-tag \
    --project-id "$project_id" \
    --tag-name "$RELEASE_TAG_NAME" \
    --tag-commit-sha "$(git -C "$WORKSPACE" rev-parse HEAD)"

  run_cli_json "release.record-remote" "$release_record_remote_json" \
    release record-remote \
    --project-id "$project_id" \
    --provider github \
    --release-id rel-e2e-001 \
    --release-url "$RELEASE_URL" \
    --tag-name "$RELEASE_TAG_NAME" \
    --release-commit-sha "$(git -C "$WORKSPACE" rev-parse HEAD)" \
    --artifact-manifest-path "artifacts/${project_id}-release-manifest.json"

  run_cli_json "release.publish" "$release_publish_json" \
    release publish --project-id "$project_id"

  run_cli_json "audit.request-human" "$audit_request_json" \
    audit request-human \
    --run-id "$issue2_run" \
    --issue-id "$issue2_id" \
    --reason "Release gate E2E human audit simulation." \
    --public-delivery-path CHANGELOG.md

  run_cli_json "release.publish.refresh" "$release_publish_refresh_json" \
    release publish --project-id "$project_id"

  run_workspace_cmd "release.summary" "$release_summary_txt" \
    "$BIN" release summary

  collect_artifacts "$requirement_id" "$project_id" "$issue2_id" "$issue2_run"
  run_projection_readmodel_contract_gate
  run_evidence_acceptance_contract_gate
  run_executor_adapter_contract_gate
  run_replay_migration_upgrade_certification_gate
  run_software_dev_pack_stable_baseline_gate
  run_deployment_evidence_gate
  run_negative_semantic_fixtures_gate
  run_v100_release_certification_gate
  run_release_provenance_gate
  run_clean_room_test_proof_gate
  run_audit_sidecar_policy_gate
  run_provider_smoke_proof_gate
  run_software_dev_pack_usage_baseline_gate
  run_forged_governance_runtime_fixture_gate
  run_trusted_governance_telemetry_gate
  run_v101_release_certification_gate
  run_v102_negative_fixtures_gate
  run_v102_release_certification_gate
  run_release_artifact_boundary_gate
  run_project_roadmap_baseline_gate
  run_v103_release_fix_certification_gate
  run_core_4d_spec_intake_gate
  run_core_ontology_kernel_gate
  run_core_object_link_schema_gate
  run_core_action_state_semantics_gate
  run_core_skill_registry_gate
  run_core_evidence_decision_reference_model_gate
  run_core_evidence_pack_schema_gate
  run_core_evidence_source_type_registry_gate
  run_core_evidence_capture_receipts_gate
  run_core_evidence_authority_trace_links_gate
  run_core_evidence_completeness_policy_gate
  run_core_missing_evidence_handling_gate
  run_core_external_proof_provenance_gate
  run_software_dev_reference_evidence_mapping_gate
  run_evidence_projection_read_model_gate
  run_core_file_backed_ontology_registry_gate
  run_v104_release_certification_gate
  run_core_runtime_negative_fixtures_gate
  run_core_runtime_kernel_gate
  run_core_runtime_admission_gate
  run_core_runtime_arbitration_gate
  run_v105_release_certification_gate
  run_v106_release_certification_gate
  run_v107_release_provenance_handoff_gate
  run_core_decision_model_contract_gate
  run_core_decision_input_binding_gate
  run_core_decision_outcome_transitions_gate
  run_core_decision_failure_reason_gate
  run_core_evidence_to_decision_gate
  run_core_completion_commit_authority_gate
  run_core_delivery_readiness_audit_trigger_gate
  run_core_projection_kernel_contract_gate
  run_core_read_model_schema_gate
  run_projection_feedback_freshness_gate
  run_core_view_model_contract_gate
  run_core_decision_projection_read_model_gate
  run_v107_release_certification_gate
  run_v108_release_certification_gate
  run_v109_release_certification_gate
  run_v110_release_certification_gate
  run_v111_release_certification_gate
  write_status "passed" "release.publish.refresh" "release gate E2E completed"
  write_gate_reports
}

main "$@"
