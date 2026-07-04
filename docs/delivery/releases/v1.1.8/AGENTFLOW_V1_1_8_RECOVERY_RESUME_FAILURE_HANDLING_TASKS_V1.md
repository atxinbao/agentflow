# AgentFlow v1.1.8 Recovery / Resume / Failure Handling Tasks

更新日期：2026-07-04
执行者：Codex

This document records the public delivery traceability for `v1.1.8`.

## Task Traceability

| Task | GitHub Issue | Title | Status | Primary proof |
| --- | --- | --- | --- | --- |
| V118-001 | #830 | Release Closeout and Certification Metadata Hardening | done | `runtime/v118-release-closeout-certification-metadata-hardening.json` |
| V118-002 | #831 | Evidence Graph Completion Proof Tightening | done | `runtime/v118-evidence-graph-completion-proof-tightening.json` |
| V118-003 | #832 | Desktop Executor Flow Frontend Invocation | done | `runtime/v118-desktop-executor-flow-frontend-invocation.json` |
| V118-004 | #833 | Run Resume Contract | done | `runtime/v118-run-resume-contract.json` |
| V118-005 | #834 | Failed Command Recovery | done | `runtime/v118-failed-command-recovery.json` |
| V118-006 | #835 | Interrupted Executor Session Closeout | done | `runtime/v118-interrupted-executor-session-closeout.json` |
| V118-007 | #836 | Duplicate Command / Idempotency Handling | done | `runtime/v118-duplicate-command-idempotency-handling.json` |
| V118-008 | #837 | Stale Projection Rebuild Recovery | done | `runtime/v118-stale-projection-rebuild-recovery.json` |
| V118-009 | #838 | Workspace Health Check | done | `runtime/v118-workspace-health-check.json` |
| V118-010 | #839 | v1.1.8 Recovery Release Certification | done | `runtime/v118-release-certification.json` |

## Dependency Order

```text
#830
-> #831
-> #832
-> #833
-> #834
-> #835
-> #836
-> #837
-> #838
-> #839
```

## Certified Boundary

`v1.1.8` keeps the executor authority model from `v1.1.7`:

- Spec Issue remains the task authority.
- Executor session remains transport.
- Runtime receipts are local facts.
- Desktop reads executor flow through Runtime API.
- Evidence graph completion is explicit and must not treat partial proof as complete.
- Recovery and resume paths must preserve the original failed or interrupted evidence.

## Release Gate Artifacts

The release certification requires the following files:

```text
runtime/v118-release-closeout-certification-metadata-hardening.json
runtime/v118-evidence-graph-completion-proof-tightening.json
runtime/v118-desktop-executor-flow-frontend-invocation.json
runtime/v118-run-resume-contract.json
runtime/v118-failed-command-recovery.json
runtime/v118-interrupted-executor-session-closeout.json
runtime/v118-duplicate-command-idempotency-handling.json
runtime/v118-stale-projection-rebuild-recovery.json
runtime/v118-workspace-health-check.json
runtime/v118-release-certification.json
```
