# AgentFlow v1.1.7 Evidence / Decision / Delivery User Readability

更新日期：2026-07-04
执行者：Codex

## Release Baseline

`v1.1.7` 是 Evidence / Decision / Delivery User Readability release baseline。

这一版把 `v1.1.6` 已经收进 Runtime 的 executor 事实转换成用户可读的状态面：

```text
Executor handoff / diff / evidence / closeout facts
-> executor flow read model
-> evidence graph
-> decision reasons
-> delivery package
-> repair paths
-> portable diagnostics
```

## Scope

`v1.1.7` 收口以下内容：

1. 下一版发布权威与 roadmap 对齐。
2. Executor surface path validation hardening。
3. Desktop executor flow read model and action visibility。
4. Evidence graph user-readable projection。
5. Decision reason and remediation projection。
6. Delivery package readability contract。
7. Failure / needs-fix / deferred repair paths。
8. Portable vs local diagnostic path boundary。
9. Release certification schema hardening。
10. v1.1.7 release certification。

## Release Gate Artifacts

`v1.1.7` release gate must produce:

- `runtime/v117-next-release-planning-surface-contract.json`
- `runtime/v117-executor-surface-path-validation-hardening.json`
- `runtime/v117-desktop-executor-flow-read-model.json`
- `runtime/v117-evidence-graph-user-readable-projection.json`
- `runtime/v117-decision-reason-remediation-projection.json`
- `runtime/v117-delivery-package-readability-contract.json`
- `runtime/v117-failure-needs-fix-deferred-repair-paths.json`
- `runtime/v117-portable-local-diagnostic-boundary.json`
- `runtime/v117-release-certification-schema-hardening.json`
- `runtime/v117-release-certification.json`

## GitHub Traceability

Task traceability is recorded in:

- [AGENTFLOW_V1_1_7_EVIDENCE_DECISION_DELIVERY_READABILITY_TASKS_V1.md](AGENTFLOW_V1_1_7_EVIDENCE_DECISION_DELIVERY_READABILITY_TASKS_V1.md)

## Authority Rules

- `.agentflow/spec/issues/**` remains the executable issue authority.
- `.agentflow/tasks/<issue-id>/**` remains local runtime fact storage.
- The executor flow read model is projection only; it does not become authority.
- Evidence graph, decision projection and delivery package must preserve source refs back to runtime facts.
- Local absolute paths are diagnostics only and must be marked local-only.
- Audit remains optional sidecar and is not reintroduced as the default delivery blocker.

## Non-goals

- This release does not add direct provider process launch from Desktop.
- This release does not add commercial paid report flow.
- This release does not make Audit a default blocker.
- This release does not make GitHub issues task authority.

## Next Version

`v1.1.8` can improve Recovery / Resume / Failure Handling after Evidence / Decision / Delivery readability is certified.
