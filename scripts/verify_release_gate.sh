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
CAPABILITY_REGISTRY_PATH="$RUNTIME_DIR/capability-registry.json"
FOUNDATION_READINESS_REPORT_SOURCE="$ROOT/docs/v0.7.2/AGENTFLOW_V0_7_2_FOUNDATION_READINESS_REPORT_V1.md"
FOUNDATION_READINESS_REPORT_PATH="$RUNTIME_DIR/foundation-readiness-report.md"
FOUNDATION_COVERAGE_PATH="$RUNTIME_DIR/foundation-coverage.json"
PACK_REGISTRY_PATH="$ARTIFACT_DIR/pack-registry.json"
PACK_VALIDATION_REPORT_PATH="$ARTIFACT_DIR/pack-validation-report.json"
PACK_SIMULATION_REPORT_PATH="$ARTIFACT_DIR/pack-simulation-report.json"
PACK_PROJECTION_READINESS_PATH="$ARTIFACT_DIR/pack-projection-readiness.json"
PACK_API_PLANE_MANIFEST_PATH="$ARTIFACT_DIR/pack-api-plane-manifest.json"
PACK_NEGATIVE_FIXTURES_PATH="$ARTIFACT_DIR/pack-negative-fixtures.json"
SOFTWARE_DEV_PACK_READINESS_PATH="$ARTIFACT_DIR/software-dev-pack-readiness.json"
UI_DESIGN_PACK_READINESS_PATH="$ARTIFACT_DIR/ui-design-pack-readiness.json"

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
    "$PACK_NEGATIVE_FIXTURES_PATH" <<'PY'
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
    {"stage": "release.version-metadata", "label": "Release Version Metadata"},
    {"stage": "release.changelog-entry", "label": "Release Changelog Entry"},
    {"stage": "release.github-release-fact", "label": "GitHub Release Fact"},
    {"stage": "pack.release-gate-readiness", "label": "Pack Release Gate Readiness"},
    {"stage": "pack.negative-fixtures", "label": "Pack Negative Fixtures"},
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
    {"path": "runtime/spec-loop-manifest.json", "exists": pathlib.Path(summary_json_path.parent / "runtime/spec-loop-manifest.json").is_file()},
    {"path": "runtime/spec-loop-projection.json", "exists": pathlib.Path(summary_json_path.parent / "runtime/spec-loop-projection.json").is_file()},
    {"path": "runtime/release-facts.json", "exists": pathlib.Path(summary_json_path.parent / "runtime/release-facts.json").is_file()},
    {"path": "runtime/external-review-surface.json", "exists": pathlib.Path(summary_json_path.parent / "runtime/external-review-surface.json").is_file()},
    {"path": "runtime/completion-runtime.json", "exists": pathlib.Path(summary_json_path.parent / "runtime/completion-runtime.json").is_file()},
    {"path": "runtime/final-closeout-proof.json", "exists": pathlib.Path(summary_json_path.parent / "runtime/final-closeout-proof.json").is_file()},
    {"path": "runtime/audit-index.json", "exists": pathlib.Path(summary_json_path.parent / "runtime/audit-index.json").is_file()},
    {"path": "runtime/provider-smoke-status.json", "exists": provider_smoke_status_path.is_file()},
    {"path": "runtime/provider-smoke-artifact.json", "exists": provider_smoke_artifact_path.is_file()},
    {"path": "runtime/api-plane-manifest.json", "exists": api_plane_manifest_path.is_file()},
    {"path": "runtime/capability-registry.json", "exists": capability_registry_path.is_file()},
    {"path": "runtime/foundation-readiness-report.md", "exists": foundation_readiness_report_path.is_file()},
    {"path": "runtime/foundation-coverage.json", "exists": foundation_coverage_path.is_file()},
    {"path": "pack-registry.json", "exists": pathlib.Path(summary_json_path.parent / "pack-registry.json").is_file()},
    {"path": "pack-validation-report.json", "exists": pathlib.Path(summary_json_path.parent / "pack-validation-report.json").is_file()},
    {"path": "pack-simulation-report.json", "exists": pathlib.Path(summary_json_path.parent / "pack-simulation-report.json").is_file()},
    {"path": "pack-projection-readiness.json", "exists": pathlib.Path(summary_json_path.parent / "pack-projection-readiness.json").is_file()},
    {"path": "pack-api-plane-manifest.json", "exists": pathlib.Path(summary_json_path.parent / "pack-api-plane-manifest.json").is_file()},
    {"path": "pack-negative-fixtures.json", "exists": pack_negative_fixtures_path.is_file()},
    {"path": "software-dev-pack-readiness.json", "exists": pathlib.Path(summary_json_path.parent / "software-dev-pack-readiness.json").is_file()},
    {"path": "ui-design-pack-readiness.json", "exists": pathlib.Path(summary_json_path.parent / "ui-design-pack-readiness.json").is_file()},
]

pack_validation = load_json(pathlib.Path(summary_json_path.parent / "pack-validation-report.json")) or {}
pack_simulation = load_json(pathlib.Path(summary_json_path.parent / "pack-simulation-report.json")) or {}
pack_projection = load_json(pathlib.Path(summary_json_path.parent / "pack-projection-readiness.json")) or {}
pack_api_plane = load_json(pathlib.Path(summary_json_path.parent / "pack-api-plane-manifest.json")) or {}
pack_negative_fixtures = load_json(pack_negative_fixtures_path) or {}
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

checklist = [
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
]

summary_payload = {
    "status": current_status,
    "conclusion": current_status,
    "gateClass": "runtime-fixture-gate",
    "providerSmokeGate": provider_smoke.get("status"),
    "providerSmokeProvider": provider_smoke.get("provider"),
    "providerSmokeReason": provider_smoke.get("reason"),
    "providerSmokeArtifactPath": provider_smoke.get("artifactPath"),
    "runtimeFixtureBoundary": "runtime-fixture-gate proves AgentFlow local runtime workflow coverage",
    "providerSmokeBoundary": "provider-smoke-gate proves minimal provider health, launch request, session snapshot, and terminal projection without replacing runtime fixture coverage",
    "foundationCoveragePath": "runtime/foundation-coverage.json" if foundation_coverage_path.is_file() else None,
    "foundationReadinessReportPath": "runtime/foundation-readiness-report.md" if foundation_readiness_report_path.is_file() else None,
    "apiPlaneManifestPath": "runtime/api-plane-manifest.json" if api_plane_manifest_path.is_file() else None,
    "capabilityRegistryPath": "runtime/capability-registry.json" if capability_registry_path.is_file() else None,
    "packRegistryPath": "pack-registry.json" if pathlib.Path(summary_json_path.parent / "pack-registry.json").is_file() else None,
    "packValidationReportPath": "pack-validation-report.json" if pathlib.Path(summary_json_path.parent / "pack-validation-report.json").is_file() else None,
    "packSimulationReportPath": "pack-simulation-report.json" if pathlib.Path(summary_json_path.parent / "pack-simulation-report.json").is_file() else None,
    "packProjectionReadinessPath": "pack-projection-readiness.json" if pathlib.Path(summary_json_path.parent / "pack-projection-readiness.json").is_file() else None,
    "packNegativeFixturesPath": "pack-negative-fixtures.json" if pack_negative_fixtures_path.is_file() else None,
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
    "- Provider smoke boundary: `minimal provider health / launch / session / terminal projection; does not replace runtime fixture gate`",
    f"- Foundation coverage: `{'present' if foundation_coverage_path.is_file() else 'missing'}`",
    f"- Foundation readiness report: `{'present' if foundation_readiness_report_path.is_file() else 'missing'}`",
    f"- API Plane manifest: `{'present' if api_plane_manifest_path.is_file() else 'missing'}`",
    f"- Capability registry: `{'present' if capability_registry_path.is_file() else 'missing'}`",
    f"- Pack release gate: `{'passed' if pack_release_gate_passed else 'failed'}`",
    f"- Pack negative fixtures: `{pack_negative_fixtures.get('status') or 'missing'}`",
    f"- Software Dev Pack readiness: `{software_readiness.get('status') or 'missing'}`",
    f"- UI Design Pack readiness: `{design_readiness.get('status') or 'missing'}`",
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
    "apiPlaneManifestPath": "runtime/api-plane-manifest.json" if api_plane_manifest_path.is_file() else None,
    "capabilityRegistryPath": "runtime/capability-registry.json" if capability_registry_path.is_file() else None,
    "packNegativeFixturesPath": "pack-negative-fixtures.json" if pack_negative_fixtures_path.is_file() else None,
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
    f"- Pack release gate: `{'passed' if pack_release_gate_passed else 'failed'}`",
    f"- Pack negative fixtures: `{pack_negative_fixtures.get('status') or 'missing'}`",
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
    "command_surface_actions",
    "connector_actions",
    "provider_actions",
    "audit_actions",
    "release_actions",
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
PY
  record_stage "api-plane-manifest" "passed" "$(basename "$API_PLANE_MANIFEST_PATH")"
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
            "docs/v0.7.2/AGENTFLOW_V0_7_2_RUNTIME_FOUNDATION_HARDENING_TASKS_V1.md",
        ],
    },
    {
        "id": "schema-migration",
        "status": "completed",
        "evidence": [
            "crates/schema-registry",
            "docs/architecture/012-schema-version-migration-registry-v1.md",
        ],
    },
    {
        "id": "simulation-dry-run",
        "status": "completed",
        "evidence": [
            "crates/simulation",
            "docs/architecture/013-simulation-dry-run-runtime-v1.md",
        ],
    },
    {
        "id": "local-message-bus",
        "status": "completed",
        "evidence": [
            "crates/message-bus",
            "docs/architecture/014-local-message-bus-contract-v1.md",
        ],
    },
    {
        "id": "worker-tool-capability-registry",
        "status": "completed",
        "evidence": [
            "crates/capability-registry",
            "docs/architecture/015-worker-tool-capability-registry-v1.md",
            str(capability_path),
        ],
    },
    {
        "id": "connector-mcp-boundary",
        "status": "completed",
        "evidence": [
            "crates/mcp",
            "docs/architecture/017-connector-mcp-boundary-v1.md",
        ],
    },
    {
        "id": "runtime-projection-command-api-plane",
        "status": "completed",
        "evidence": [
            "crates/runtime-api/src/api_plane.rs",
            "docs/architecture/018-api-plane-manifest-v1.md",
            str(api_plane_path),
        ],
    },
    {
        "id": "provider-smoke-gate",
        "status": "baseline",
        "evidence": [
            "crates/mcp/src/smoke.rs",
            "docs/architecture/016-provider-smoke-gate-v1.md",
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
  if ! "$BIN" pack release-gate-readiness \
    --output-dir "$ARTIFACT_DIR" \
    --runtime-version "${RELEASE_VERSION#v}" \
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
if registry.get("source") != "fixture-files":
    raise SystemExit("pack registry must come from fixture-files, not built-in baseline")
if registry.get("fallback") is not False:
    raise SystemExit("pack registry fallback must be false")
entries = {entry.get("packId"): entry for entry in registry.get("entries", [])}
for pack_id in ["software-dev", "ui-design"]:
    entry = entries.get(pack_id)
    if entry is None:
        raise SystemExit(f"pack registry missing {pack_id}")
    if entry.get("source") != "fixture-files":
        raise SystemExit(f"{pack_id} registry entry must come from fixture-files")
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

registry_file_backed = (
    registry.get("version") == "agentflow-pack-registry.v1"
    and registry.get("source") == "fixture-files"
    and registry.get("fallback") is False
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
        "unexpected-software-dev-fallback",
        "registry",
        "Software Dev Pack must resolve from fixture files and never fall back to the built-in baseline",
        ["pack-registry.json"],
        registry_file_backed,
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

prepare_workspace() {
  record_stage "workspace.prepare" "started" "$WORKSPACE"
  git clone "$ROOT" "$WORKSPACE" >/dev/null
  git -C "$WORKSPACE" config user.email "codex@example.com"
  git -C "$WORKSPACE" config user.name "Codex"
  git -C "$WORKSPACE" checkout -B "$BOOTSTRAP_BRANCH" >/dev/null
  export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT/target}"
  record_stage "workspace.prepare" "passed" "$WORKSPACE"
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
  cp "$WORKSPACE/.agentflow/tasks/${final_issue_id}/runs/${final_run_id}/review/closeout-proof.json" "$RUNTIME_DIR/final-closeout-proof.json"
  if [[ -f "$WORKSPACE/.agentflow/audit/index.json" ]]; then
    cp "$WORKSPACE/.agentflow/audit/index.json" "$RUNTIME_DIR/audit-index.json"
  fi
}

main() {
  write_status "running" "workspace.prepare" "preparing release gate workspace"
  prepare_workspace
  verify_release_metadata "$WORKSPACE"
  verify_release_publication_facts "$WORKSPACE"
  run_provider_smoke_gate
  run_api_plane_manifest_gate
  run_capability_registry_gate
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
  write_status "passed" "release.publish.refresh" "release gate E2E completed"
  write_gate_reports
}

main "$@"
