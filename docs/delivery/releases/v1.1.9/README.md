# AgentFlow v1.1.9 Software Dev Reference App Beta Certification

更新日期：2026-07-04
执行者：Codex

## Release Baseline

`v1.1.9` 是 Software Dev reference app beta certification release baseline。

这一版把 `v1.1.8` 的 recovery / resume / failure handling 继续推进到一个可内测的软件开发参考应用闭环：

```text
Project
-> Intake
-> Task authority
-> Executor run
-> Evidence
-> Decision
-> Delivery
-> Retry / feedback
-> Desktop read model
```

## Scope

`v1.1.9` 收口以下内容：

1. Release certification metadata top-level contract。
2. Recovery idempotency receipt path hardening。
3. Projection rebuild positive recovery proof。
4. Workspace health provider / skill smoke boundary。
5. Software Dev Reference App beta scope alignment。
6. End-to-end golden scenario: Project -> Intake -> Tasks。
7. Executor Run -> Evidence -> Decision -> Delivery golden path。
8. Failure / Retry / Feedback beta scenario。
9. Desktop beta readiness UI smoke proof。
10. v1.1.9 beta release certification。

## Release Gate Artifacts

`v1.1.9` release gate must produce:

- `runtime/v119-release-certification-metadata-top-level-contract.json`
- `runtime/v119-recovery-idempotency-receipt-path-hardening.json`
- `runtime/v119-projection-rebuild-positive-recovery-proof.json`
- `runtime/v119-workspace-health-provider-skill-smoke-boundary.json`
- `runtime/v119-software-dev-reference-app-beta-scope-alignment.json`
- `runtime/v119-e2e-project-intake-tasks-golden-scenario.json`
- `runtime/v119-executor-run-evidence-decision-delivery-golden-path.json`
- `runtime/v119-failure-retry-feedback-beta-scenario.json`
- `runtime/v119-desktop-beta-readiness-ui-smoke-proof.json`
- `runtime/v119-beta-release-certification.json`

## Certification Metadata

The v1.1.9 certification record must expose release-gate metadata at the top level:

```text
releaseVersion
releaseTag
sourceCommit
workflowRunId
artifactNames
primaryProofs
```

These fields are release-gate metadata, not runtime authority.

## Beta Boundary

`v1.1.9` certifies the Software Dev reference app beta.

It is:

- a beta baseline for the Software Dev reference app;
- a proof that Domain Pack, Surface Pack, Connector Pack and Desktop can participate in one readable software development flow;
- a proof that recovery, retry, evidence, decision and delivery facts can be projected for a small development task.

It is not Core GA and not public commercial launch.

## GitHub Traceability

Task traceability is recorded in:

- [AGENTFLOW_V1_1_9_SOFTWARE_DEV_REFERENCE_APP_BETA_CERTIFICATION_TASKS_V1.md](AGENTFLOW_V1_1_9_SOFTWARE_DEV_REFERENCE_APP_BETA_CERTIFICATION_TASKS_V1.md)

## Authority Rules

- `.agentflow/spec/issues/**` remains the executable issue authority.
- `.agentflow/tasks/<issue-id>/**` remains local runtime fact storage.
- Runtime receipts are local facts and must be idempotent by receipt path, idempotency key and payload hash.
- Projection freshness must be derived from event replay, not from stale cached read models.
- Provider / skill readiness requires smoke evidence. A configured provider string is not readiness proof.
- Desktop reads executor flow through Runtime API and does not read authority files directly.
- Audit remains optional sidecar and is not reintroduced as the default delivery blocker.

## Non-goals

- This release does not certify public commercial launch.
- This release does not make GitHub issues task authority.
- This release does not add cloud runtime recovery.
- This release does not add provider process supervision.
- This release does not make Audit a default blocker.

## Next Version

`v1.2.0` can build on this beta baseline to improve product onboarding and first-run experience.
