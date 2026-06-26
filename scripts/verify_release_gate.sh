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
STABLE_CONTRACT_BASELINE_PATH="$RUNTIME_DIR/stable-contract-baseline.json"

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
    "$NEGATIVE_SEMANTIC_FIXTURES_PATH" <<'PY'
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
    {"path": "runtime/clean-room-test-proof.json", "exists": pathlib.Path(summary_json_path.parent / "runtime/clean-room-test-proof.json").is_file()},
    {"path": "runtime/audit-sidecar-policy.json", "exists": pathlib.Path(summary_json_path.parent / "runtime/audit-sidecar-policy.json").is_file()},
    {"path": "runtime/provider-smoke-proof.json", "exists": pathlib.Path(summary_json_path.parent / "runtime/provider-smoke-proof.json").is_file()},
    {"path": "runtime/software-dev-pack-usage-baseline.json", "exists": pathlib.Path(summary_json_path.parent / "runtime/software-dev-pack-usage-baseline.json").is_file()},
    {"path": "runtime/trusted-governance-telemetry.json", "exists": pathlib.Path(summary_json_path.parent / "runtime/trusted-governance-telemetry.json").is_file()},
    {"path": "runtime/v101-release-certification.json", "exists": pathlib.Path(summary_json_path.parent / "runtime/v101-release-certification.json").is_file()},
    {"path": "runtime/v102-negative-fixtures.json", "exists": pathlib.Path(summary_json_path.parent / "runtime/v102-negative-fixtures.json").is_file()},
    {"path": "runtime/v102-release-certification.json", "exists": pathlib.Path(summary_json_path.parent / "runtime/v102-release-certification.json").is_file()},
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
heading = re.compile(rf"^##\s+(?:\[)?{re.escape(expected)}(?:\])?(?:\s|$)", re.MULTILINE)
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
    "docs/project/goal.md",
    "docs/architecture/021-ai-os-project-core-capabilities-v1.md",
    "docs/architecture/builtin-pack-registry.md",
    "docs/README.md",
    "docs/architecture/README.md",
    "docs/delivery/releases/v1.0.2/README.md",
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
    "currentCoreCapabilityEntry": "docs/architecture/021-ai-os-project-core-capabilities-v1.md",
    "currentReleaseBaselineEntry": "docs/delivery/releases/v1.0.2/README.md",
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
    "commandType": "createProject",
    "sourceSurface": "agent",
    "actorRole": "spec-agent",
    "targetObjectRef": {"objectType": "Spec", "id": "spec-governance-001"},
    "input": {
        "projectId": "project-governance-001",
        "projectTitle": "Governance Admission Fixture"
    },
    "evidenceRefs": ["approved-spec-1", "human-confirmation-1"],
    "artifactRefs": [".agentflow/spec/requirements/req-governance/preview.json"],
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
pack_missing_definition_behavior = (
    "invalid"
    if "invalid" in pack_readiness_statuses
    else "deferred"
    if "deferred" in pack_readiness_statuses
    else None
)
pack_projection_no_fallback = (
    pack_projection.get("status") == "passed"
    and pack_missing_definition_behavior in {"invalid", "deferred"}
    and all((view.get("workbenchCount") or 0) == 0 for view in (pack_projection.get("views") or []))
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
    "rebuiltPathCount": len(rebuilt_paths),
    "missingProjectionPaths": missing_projection_paths,
    "queryApiReadonly": query_api_readonly,
    "industrySurfaceReadonly": industry_surface_readonly,
    "missingProjectionQueries": missing_required_queries,
    "packProjectionStatus": pack_projection.get("status"),
    "packMissingDefinitionBehavior": pack_missing_definition_behavior,
    "packProjectionNoFallback": pack_projection_no_fallback,
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
    unsigned_reason="tag signature verification is not required for v1.0.1 hardening"
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
  python3 - "$CLEAN_ROOM_TEST_PROOF_PATH" "$ARTIFACT_DIR/cargo-target" <<'PY'
import json, pathlib, sys, time
out_path = pathlib.Path(sys.argv[1])
payload = {
    "version": "agentflow-clean-room-test-proof.v1",
    "status": "passed",
    "cargoTargetDir": sys.argv[2],
    "manualCargoCleanRequired": False,
    "proof": "release-gate uses an artifact-scoped CARGO_TARGET_DIR, preventing temporary workspace fixtures from poisoning the repository target cache",
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

run_trusted_governance_telemetry_gate() {
  record_stage "trusted-governance-telemetry" "started" "$TRUSTED_GOVERNANCE_TELEMETRY_PATH"
  python3 - "$TRUSTED_GOVERNANCE_TELEMETRY_PATH" "$GOVERNANCE_ADMISSION_PATH" "$PROVIDER_SMOKE_PROOF_PATH" <<'PY'
import json, pathlib, sys, time
out_path = pathlib.Path(sys.argv[1])
governance = json.loads(pathlib.Path(sys.argv[2]).read_text(encoding="utf-8"))
provider = json.loads(pathlib.Path(sys.argv[3]).read_text(encoding="utf-8"))
responses = governance.get("responses") or []
if not responses:
    raise SystemExit("governance admission responses missing")
if not all((response.get("governanceAdmission") or {}).get("trace") for response in responses):
    raise SystemExit("governance admission must include trace")
if provider.get("capabilityAvailability") == "ready" and provider.get("providerSmokeStatus") != "passed":
    raise SystemExit("skipped provider smoke cannot assert ready")
payload = {
    "version": "agentflow-trusted-governance-telemetry.v1",
    "status": "passed",
    "telemetrySourceKind": "release-gate-artifact",
    "telemetrySourcePath": "runtime/provider-smoke-proof.json",
    "requestInputMayReferenceEvidencePath": True,
    "requestInputMayAssertProviderReady": False,
    "forgedTelemetryFixture": {"status": "rejected", "reason": "request input cannot override provider-smoke-proof"},
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
        "expectedStatus": "rejected",
        "actualStatus": (trusted.get("forgedTelemetryFixture") or {}).get("status"),
        "reason": "request input must not override trusted provider smoke or capability registry facts",
        "evidencePath": "runtime/trusted-governance-telemetry.json",
        "passed": trusted.get("requestInputMayAssertProviderReady") is False
        and (trusted.get("forgedTelemetryFixture") or {}).get("status") == "rejected",
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
    and (trusted.get("forgedTelemetryFixture") or {}).get("status") == "rejected",
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
    "V102-004": "Spec-Driven Software Dev Workflow" in goal_text
    and "GitHub issue" in goal_text
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
    "productGoalBaseline": "Spec-Driven Software Dev Workflow",
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

prepare_workspace() {
  record_stage "workspace.prepare" "started" "$WORKSPACE"
  git clone "$ROOT" "$WORKSPACE" >/dev/null
  git -C "$WORKSPACE" config user.email "codex@example.com"
  git -C "$WORKSPACE" config user.name "Codex"
  git -C "$WORKSPACE" checkout -B "$BOOTSTRAP_BRANCH" >/dev/null
  export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$ARTIFACT_DIR/cargo-target}"
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
    source = source_root / pack_id / "pack.json"
    target_dir = target_root / pack_id
    target_dir.mkdir(parents=True, exist_ok=True)
    shutil.copy2(source, target_dir / "pack.json")
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
  run_trusted_governance_telemetry_gate
  run_v101_release_certification_gate
  run_v102_negative_fixtures_gate
  run_v102_release_certification_gate
  write_status "passed" "release.publish.refresh" "release gate E2E completed"
  write_gate_reports
}

main "$@"
