# AgentFlow v1.1.7 Evidence / Decision / Delivery User Readability Tasks

更新日期：2026-07-04
执行者：Codex

This document records the public delivery traceability for `v1.1.7`.

## Task Traceability

| Task | GitHub Issue | Title | Status | Primary proof |
| --- | --- | --- | --- | --- |
| V117-001 | #819 | Next Release Planning and Surface Contract Alignment | done | `runtime/v117-next-release-planning-surface-contract.json` |
| V117-002 | #820 | Executor Surface Path Validation Hardening | done | `runtime/v117-executor-surface-path-validation-hardening.json` |
| V117-003 | #821 | Desktop Executor Flow Read Model and Action Visibility | done | `runtime/v117-desktop-executor-flow-read-model.json` |
| V117-004 | #822 | Evidence Graph User-readable Projection | done | `runtime/v117-evidence-graph-user-readable-projection.json` |
| V117-005 | #823 | Decision Reason and Remediation Projection | done | `runtime/v117-decision-reason-remediation-projection.json` |
| V117-006 | #824 | Delivery Package Readability Contract | done | `runtime/v117-delivery-package-readability-contract.json` |
| V117-007 | #825 | Failure / Needs-fix / Deferred Repair Paths | done | `runtime/v117-failure-needs-fix-deferred-repair-paths.json` |
| V117-008 | #826 | Portable vs Local Diagnostic Path Boundary | done | `runtime/v117-portable-local-diagnostic-boundary.json` |
| V117-009 | #827 | Release Certification Schema Hardening | done | `runtime/v117-release-certification-schema-hardening.json` |
| V117-010 | #828 | v1.1.7 Release Certification | done | `runtime/v117-release-certification.json` |

## Dependency Order

```text
#819
-> #820
-> #821
-> #822
-> #823
-> #824
-> #825
-> #826
-> #827
-> #828
```

## Certified Boundary

`v1.1.7` keeps the executor authority model from `v1.1.6`:

- Spec Issue remains the task authority.
- Executor session remains transport.
- Evidence / Decision / Delivery surfaces are projections.
- Audit remains optional sidecar.
- Local diagnostic paths are not portable delivery facts.

## Release Gate Artifacts

The release certification requires the following files:

```text
runtime/v117-next-release-planning-surface-contract.json
runtime/v117-executor-surface-path-validation-hardening.json
runtime/v117-desktop-executor-flow-read-model.json
runtime/v117-evidence-graph-user-readable-projection.json
runtime/v117-decision-reason-remediation-projection.json
runtime/v117-delivery-package-readability-contract.json
runtime/v117-failure-needs-fix-deferred-repair-paths.json
runtime/v117-portable-local-diagnostic-boundary.json
runtime/v117-release-certification-schema-hardening.json
runtime/v117-release-certification.json
```
