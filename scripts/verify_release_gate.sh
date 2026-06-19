#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ARTIFACT_DIR="$ROOT/artifacts/release-gate-e2e"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --artifact-dir)
      ARTIFACT_DIR="$2"
      shift 2
      ;;
    *)
      echo "unknown argument: $1" >&2
      exit 1
      ;;
  esac
done

mkdir -p "$ARTIFACT_DIR"
ARTIFACT_DIR="$(cd "$ARTIFACT_DIR" && pwd)"

BIN="${AGENTFLOW_BIN:-$ROOT/target/debug/agentflow}"
if [[ -z "${AGENTFLOW_BIN:-}" ]]; then
  cargo build -p agentflow-cli --bin agentflow --manifest-path "$ROOT/Cargo.toml" >/dev/null
fi

TMP_DIR="$(mktemp -d "${TMPDIR:-/tmp}/agentflow-release-gate.XXXXXX")"
trap 'rm -rf "$TMP_DIR"' EXIT

WORKSPACE="$TMP_DIR/workspace"
mkdir -p "$WORKSPACE/docs/requirements"
cd "$WORKSPACE"
git init -q

REQUIREMENT_FILE="docs/requirements/058h-release-gate-e2e.md"
cat > "$REQUIREMENT_FILE" <<'EOF'
# 058H Release Gate E2E

验证 requirement 到 project/release 的正式入口。

- 目标：验证 v0.3.0 stable release gate。
- 范围：formal project/completion/release runtime。
- 交付：release facts、CHANGELOG、release notes。
EOF

run_json() {
  local output="$1"
  shift
  "$BIN" "$@" > "$output"
}

run_json artifacts-intake.json \
  project intake \
  --requirement-path "$REQUIREMENT_FILE" \
  --project-id project-release-gate-e2e

REQUIREMENT_ID="$(python3 - <<'PY'
import json, pathlib
data = json.loads(pathlib.Path("artifacts-intake.json").read_text())
print(data["requirementId"])
PY
)"

run_json artifacts-goal.json project confirm-goal --requirement-id "$REQUIREMENT_ID"
run_json artifacts-plan.json project confirm-plan --requirement-id "$REQUIREMENT_ID"
run_json artifacts-materialize.json project materialize --requirement-id "$REQUIREMENT_ID"

python3 - <<'PY'
import json
import pathlib
import time

root = pathlib.Path(".")
materialized = json.loads((root / "artifacts-materialize.json").read_text())
project_id = materialized["project"]["projectId"]
now = int(time.time())
projection_dir = root / ".agentflow" / "projections" / "tasks"
projection_dir.mkdir(parents=True, exist_ok=True)

for index, issue in enumerate(materialized["issues"], start=1):
    issue_path = root / issue["system"]["path"]
    issue_doc = json.loads(issue_path.read_text())
    issue_doc["status"] = "done"
    issue_doc.setdefault("system", {})["updatedAt"] = now + index
    issue_path.write_text(json.dumps(issue_doc, ensure_ascii=False, indent=2) + "\n")

    issue_id = issue["issueId"]
    run_id = f"run-{index:03d}"
    expected_outputs = issue.get("expectedOutputs", {})
    issue_root = root / ".agentflow" / "tasks" / issue_id
    evidence_dir = issue_root / "evidence"
    review_dir = issue_root / "runs" / run_id / "review"
    evidence_dir.mkdir(parents=True, exist_ok=True)
    review_dir.mkdir(parents=True, exist_ok=True)
    (evidence_dir / "evidence.json").write_text(
        json.dumps(
            {
                "version": "task-evidence.v1",
                "issueId": issue_id,
                "runId": run_id,
                "status": "ready",
                "summary": "release gate e2e local verification passed",
                "runPath": f".agentflow/tasks/{issue_id}/runs/{run_id}/run.json",
                "commandPaths": [],
                "validationPath": f".agentflow/tasks/{issue_id}/runs/{run_id}/validation.json",
                "createdAt": now + index,
            },
            ensure_ascii=False,
            indent=2,
        )
        + "\n"
    )
    (review_dir / "closeout-proof.json").write_text(
        json.dumps(
            {
                "merged": True,
                "issueClosed": True,
                "publicDeliveryWritten": True,
                "prUrl": f"https://github.com/example/agentflow/pull/{index}",
                "mergeCommitSha": f"merge-058h-{index:03d}",
                "changelogPath": "CHANGELOG.md",
                "releaseNotesPath": f"docs/release-notes/{project_id}.md",
            },
            ensure_ascii=False,
            indent=2,
        )
        + "\n"
    )
    projection_payload = {
        "issueId": issue_id,
        "projectId": project_id,
        "currentState": "done",
        "displayStatus": "done",
        "publicDelivery": {
            "prUrl": f"https://github.com/example/agentflow/pull/{index}",
            "mergeCommit": f"merge-058h-{index:03d}",
            "changelogPath": "CHANGELOG.md",
            "releaseNotesUrl": f"docs/release-notes/{project_id}.md",
        },
        "delivery": {
            "status": "ready",
            "evidenceStatus": "ready",
            "evidencePath": expected_outputs.get("evidencePath"),
            "prUrl": f"https://github.com/example/agentflow/pull/{index}",
            "mergeCommit": f"merge-058h-{index:03d}",
            "publicRecordPath": "CHANGELOG.md",
        },
        "updatedAt": now + index,
    }
    (projection_dir / f"{issue_id}.json").write_text(
        json.dumps(projection_payload, ensure_ascii=False, indent=2) + "\n"
    )
PY

"$BIN" projection rebuild > artifacts-projection.txt
run_json artifacts-completion-inspect.json completion inspect --project-id project-release-gate-e2e
run_json artifacts-completion-decide.json \
  completion decide \
  --project-id project-release-gate-e2e \
  --outcome accept \
  --summary "Release gate fixture accepted" \
  --rationale "all issues done" \
  --rationale "projection rebuilt"
"$BIN" projection rebuild > artifacts-projection-after-completion.txt
run_json artifacts-release-prepare.json release prepare --project-id project-release-gate-e2e
run_json artifacts-release-confirm.json release confirm --project-id project-release-gate-e2e
mkdir -p artifacts
cat > artifacts/project-release-gate-e2e-release-manifest.json <<'EOF'
{
  "projectId": "project-release-gate-e2e",
  "artifacts": [
    "CHANGELOG.md",
    "docs/release-notes/project-release-gate-e2e.md"
  ],
  "generatedBy": "verify_release_gate.sh"
}
EOF
run_json artifacts-release-record-tag.json \
  release record-tag \
  --project-id project-release-gate-e2e \
  --tag-name v0.3.1-e2e \
  --tag-commit-sha e2e-tag-commit-001
run_json artifacts-release-record-remote.json \
  release record-remote \
  --project-id project-release-gate-e2e \
  --provider github \
  --release-id rel-e2e-001 \
  --release-url https://github.com/example/agentflow/releases/tag/v0.3.1-e2e \
  --tag-name v0.3.1-e2e \
  --release-commit-sha e2e-tag-commit-001 \
  --artifact-manifest-path artifacts/project-release-gate-e2e-release-manifest.json
run_json artifacts-release-publish.json release publish --project-id project-release-gate-e2e
"$BIN" release summary > artifacts-release-summary.txt

ARTIFACT_DIR_PY="$ARTIFACT_DIR" python3 - <<'PY'
import json
import os
import pathlib
import shutil

root = pathlib.Path(".")
artifact_dir = pathlib.Path(os.environ["ARTIFACT_DIR_PY"])
cli_dir = artifact_dir / "cli"
public_dir = artifact_dir / "public"
runtime_dir = artifact_dir / "runtime"

for directory in (cli_dir, public_dir, runtime_dir):
    directory.mkdir(parents=True, exist_ok=True)

for path in root.glob("artifacts-*"):
    shutil.copy2(path, cli_dir / path.name)

release = json.loads((root / "artifacts-release-publish.json").read_text())
project_id = release["projectId"]

public_paths = {
    root / "CHANGELOG.md": public_dir / "CHANGELOG.md",
    root / "docs" / "release-notes" / f"{project_id}.md": public_dir / "release-notes.md",
    root / "docs" / "reviews" / f"{project_id}.md": public_dir / "external-review.md",
}
runtime_paths = {
    root / ".agentflow" / "release" / "projects" / f"{project_id}.json": runtime_dir / "release-facts.json",
    root / ".agentflow" / "release" / "reviews" / f"{project_id}.json": runtime_dir / "external-review-surface.json",
    root / ".agentflow" / "release" / "proofs" / project_id / "tag.json": runtime_dir / "release-tag-proof.json",
    root / ".agentflow" / "release" / "proofs" / project_id / "remote-release.json": runtime_dir / "remote-release-proof.json",
    root / ".agentflow" / "indexes" / "releases.json": runtime_dir / "release-index.json",
    root / ".agentflow" / "indexes" / "external-reviews.json": runtime_dir / "external-review-index.json",
}

for source, destination in {**public_paths, **runtime_paths}.items():
    shutil.copy2(source, destination)

release_facts = json.loads((runtime_dir / "release-facts.json").read_text())
external_review = json.loads((runtime_dir / "external-review-surface.json").read_text())
materialized = json.loads((root / "artifacts-materialize.json").read_text())
summary = {
    "requirementId": materialized["project"]["sourceRequirementId"],
    "projectId": project_id,
    "issueCount": len(materialized["issues"]),
    "releaseState": release_facts["currentState"],
    "publicationStage": release_facts["publicationStage"],
    "gateStatus": release_facts["gateStatus"],
    "completionState": release_facts["completionState"],
    "completionOutcome": release_facts["completionOutcome"],
    "tagName": release_facts.get("tagName"),
    "remoteReleaseUrl": release_facts.get("remoteReleaseUrl"),
    "changelogPath": release_facts["changelogPath"],
    "releaseNotesPath": release_facts["releaseNotesPath"],
    "externalReviewPath": external_review["handoffPath"],
    "entryCount": release_facts["entryCount"],
    "latestEventId": release_facts["latestEventId"],
}
summary_path = artifact_dir / "summary.json"
summary_path.write_text(json.dumps(summary, ensure_ascii=False, indent=2) + "\n")

markdown = f"""# Release Gate E2E Summary

- Requirement: `{summary["requirementId"]}`
- Project: `{summary["projectId"]}`
- Issues: `{summary["issueCount"]}`
- Completion: `{summary["completionState"]}` / `{summary["completionOutcome"]}`
- Release: `{summary["releaseState"]}`
- Publication Stage: `{summary["publicationStage"]}`
- Gate: `{summary["gateStatus"]}`
- Tag: `{summary["tagName"]}`
- Remote Release: `{summary["remoteReleaseUrl"]}`
- Entries: `{summary["entryCount"]}`
- Latest Event: `{summary["latestEventId"]}`
- Changelog: `{summary["changelogPath"]}`
- Release Notes: `{summary["releaseNotesPath"]}`
- External Review: `{summary["externalReviewPath"]}`
"""
(artifact_dir / "summary.md").write_text(markdown)
PY

echo "release gate e2e: ok"
echo "artifact dir: $ARTIFACT_DIR"
echo "summary: $ARTIFACT_DIR/summary.md"
