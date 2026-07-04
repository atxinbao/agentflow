# AgentFlow v1.1.8 Recovery / Resume / Failure Handling

更新日期：2026-07-04
执行者：Codex

## Release Baseline

`v1.1.8` 是 Recovery / Resume / Failure Handling release baseline。

这一版把 `v1.1.7` 的 executor readability surface 继续收紧成可恢复、可重放、可诊断的执行闭环：

```text
Executor run / handoff / boundary / command / evidence / closeout facts
-> explicit evidence graph state
-> resume receipt
-> failed command recovery receipt
-> lifecycle closeout receipt
-> projection rebuild receipt
-> workspace health report
-> Desktop executor flow read model
```

## Scope

`v1.1.8` 收口以下内容：

1. Release closeout and certification metadata hardening。
2. Evidence graph completion proof tightening。
3. Desktop executor flow frontend invocation。
4. Run resume contract。
5. Failed command recovery。
6. Interrupted executor session closeout。
7. Duplicate command / idempotency handling。
8. Stale projection rebuild recovery。
9. Workspace health check。
10. v1.1.8 recovery release certification。

## Release Gate Artifacts

`v1.1.8` release gate must produce:

- `runtime/v118-release-closeout-certification-metadata-hardening.json`
- `runtime/v118-evidence-graph-completion-proof-tightening.json`
- `runtime/v118-desktop-executor-flow-frontend-invocation.json`
- `runtime/v118-run-resume-contract.json`
- `runtime/v118-failed-command-recovery.json`
- `runtime/v118-interrupted-executor-session-closeout.json`
- `runtime/v118-duplicate-command-idempotency-handling.json`
- `runtime/v118-stale-projection-rebuild-recovery.json`
- `runtime/v118-workspace-health-check.json`
- `runtime/v118-release-certification.json`

## Certification Metadata

The v1.1.8 certification record must include:

```text
releaseVersion
releaseTag
sourceCommit
workflowRunId
artifactNames
primaryProofs
```

These fields are release-gate metadata. They are not runtime authority, but they bind the published tag to the proof artifacts.

## GitHub Traceability

Task traceability is recorded in:

- [AGENTFLOW_V1_1_8_RECOVERY_RESUME_FAILURE_HANDLING_TASKS_V1.md](AGENTFLOW_V1_1_8_RECOVERY_RESUME_FAILURE_HANDLING_TASKS_V1.md)

## Authority Rules

- `.agentflow/spec/issues/**` remains the executable issue authority.
- `.agentflow/tasks/<issue-id>/**` remains local runtime fact storage.
- Executor session remains transport.
- Resume, recovery and lifecycle receipts are runtime facts, not task authority.
- Desktop reads executor flow through Runtime API and does not read authority files directly.
- Evidence graph `complete` means all required facts are present and accepted.
- Failed or stale facts must be visible as `failed` or `stale`, not silently folded into `partial`.
- Audit remains optional sidecar and is not reintroduced as the default delivery blocker.

## Non-goals

- This release does not add provider process supervision.
- This release does not add cloud recovery.
- This release does not add message bus semantics.
- This release does not make GitHub issues task authority.
- This release does not make Audit a default blocker.

## Next Version

`v1.1.9` can continue hardening Runtime surface behavior after recovery and failure handling is certified.
