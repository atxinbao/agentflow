# Release Certification Primary Proof Package V1

更新日期：2026-06-30
执行者：Codex

## Purpose

`release-gate-certification-<version>` is the small quick-audit package for a release.
It must be self-contained enough for a reviewer to inspect the current release's
primary runtime proofs without downloading the full gate artifact.

## Problem

`v1.0.7` certified the Decision Kernel through runtime artifacts, but the small
certification package only carried summary certification files and older primary
proof families. The full artifact still had the Decision Kernel files, but the
quick-audit artifact did not.

That is not acceptable for release audit. A certification package that names a
kernel baseline must carry the primary JSON and log proofs for that baseline.

## Contract

For each release version, the certification package must include:

- generic release-gate status files;
- release certification JSON / Markdown;
- artifact manifest and stage log;
- browser / console smoke status;
- all current-version primary proof JSON files;
- all current-version primary proof test logs;
- the current-version release certification JSON.

For `v1.0.7`, this means the package includes the Decision Kernel primary proofs:

- `runtime/v107-release-provenance-handoff.json`;
- `runtime/core-decision-model-contract.json`;
- `runtime/core-decision-model-contract-rust-test.log`;
- `runtime/core-decision-input-binding.json`;
- `runtime/core-decision-input-binding-rust-test.log`;
- `runtime/core-decision-outcome-transitions.json`;
- `runtime/core-decision-outcome-transitions-rust-test.log`;
- `runtime/core-decision-failure-reason-remediation.json`;
- `runtime/core-decision-failure-reason-remediation-rust-test.log`;
- `runtime/core-evidence-to-decision-gate.json`;
- `runtime/core-evidence-to-decision-gate-rust-test.log`;
- `runtime/core-completion-commit-authority.json`;
- `runtime/core-completion-commit-authority-rust-test.log`;
- `runtime/core-delivery-readiness-audit-trigger.json`;
- `runtime/core-delivery-readiness-audit-trigger-rust-test.log`;
- `runtime/core-decision-projection-read-model.json`;
- `runtime/core-decision-projection-read-model-rust-test.log`;
- `runtime/v107-release-certification.json`.

## Negative Check

The release workflow must fail when a required current-version primary proof is
missing from the small certification package.

The workflow also keeps a negative check that deletes one Decision Kernel proof
from a temporary copy and confirms that the same proof-presence validator rejects
the broken package.

## Boundary

This contract only controls artifact packaging and proof presence. It does not
change release semantics, authority writes, Projection Kernel behavior, or Audit
sidecar policy.
