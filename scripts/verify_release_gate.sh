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
    "$RUNTIME_DIR/external-review-surface.json" <<'PY'
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
]

checklist = [
    {
        "id": "real-runtime-e2e",
        "label": "release gate 跑真实 runtime E2E",
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
]

summary_payload = {
    "status": current_status,
    "conclusion": current_status,
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
    "auditStatus": audit.get("latestStatus"),
    "auditReportPath": audit.get("latestReportPath"),
}
summary_json_path.write_text(
    json.dumps(summary_payload, ensure_ascii=False, indent=2) + "\n",
    encoding="utf-8",
)

summary_lines = [
    "# Release Gate E2E Summary",
    "",
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
        f"- Audit status: `{summary_payload['auditStatus'] or 'not-requested'}`",
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
    "currentGateRun": current_gate_run,
    "mainGateRun": main_gate_run,
    "tagGateRun": tag_gate_run,
    "releaseGateRun": release_gate_run,
    "gateStatus": current_status,
    "conclusion": current_status,
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
