# AgentFlow v1.1.9 Software Dev Reference App Beta Certification Tasks

更新日期：2026-07-04
执行者：Codex

This document records the public delivery traceability for `v1.1.9`.

## Task Traceability

| Task | GitHub Issue | Title | Status | Primary proof |
| --- | --- | --- | --- | --- |
| V119-001 | #841 | Release Certification Metadata Top-level Contract | done | `runtime/v119-release-certification-metadata-top-level-contract.json` |
| V119-002 | #842 | Recovery Idempotency Receipt Path Hardening | done | `runtime/v119-recovery-idempotency-receipt-path-hardening.json` |
| V119-003 | #843 | Projection Rebuild Positive Recovery Proof | done | `runtime/v119-projection-rebuild-positive-recovery-proof.json` |
| V119-004 | #844 | Workspace Health Provider / Skill Smoke Boundary | done | `runtime/v119-workspace-health-provider-skill-smoke-boundary.json` |
| V119-005 | #845 | Software Dev Reference App Beta Scope Alignment | done | `runtime/v119-software-dev-reference-app-beta-scope-alignment.json` |
| V119-006 | #846 | End-to-end Golden Scenario: Project -> Intake -> Tasks | done | `runtime/v119-e2e-project-intake-tasks-golden-scenario.json` |
| V119-007 | #847 | Executor Run -> Evidence -> Decision -> Delivery Golden Path | done | `runtime/v119-executor-run-evidence-decision-delivery-golden-path.json` |
| V119-008 | #848 | Failure / Retry / Feedback Beta Scenario | done | `runtime/v119-failure-retry-feedback-beta-scenario.json` |
| V119-009 | #849 | Desktop Beta Readiness UI Smoke Proof | done | `runtime/v119-desktop-beta-readiness-ui-smoke-proof.json` |
| V119-010 | #850 | v1.1.9 Beta Release Certification | done | `runtime/v119-beta-release-certification.json` |

## Dependency Order

```text
#841
-> #842
-> #843
-> #844
-> #845
-> #846
-> #847
-> #848
-> #849
-> #850
```

## Certified Boundary

`v1.1.9` keeps the executor authority model from `v1.1.8` and adds beta readiness proof:

- Spec Issue remains the task authority.
- Executor session remains transport.
- Runtime receipts are local facts.
- Provider / skill smoke evidence is required before workspace health can report ready.
- Projection recovery must pass through event replay.
- Desktop reads executor flow through Runtime API.
- Software Dev reference app beta is not Core GA and not public commercial launch.

## Release Gate Artifacts

The release certification requires the following files:

```text
runtime/v119-release-certification-metadata-top-level-contract.json
runtime/v119-recovery-idempotency-receipt-path-hardening.json
runtime/v119-projection-rebuild-positive-recovery-proof.json
runtime/v119-workspace-health-provider-skill-smoke-boundary.json
runtime/v119-software-dev-reference-app-beta-scope-alignment.json
runtime/v119-e2e-project-intake-tasks-golden-scenario.json
runtime/v119-executor-run-evidence-decision-delivery-golden-path.json
runtime/v119-failure-retry-feedback-beta-scenario.json
runtime/v119-desktop-beta-readiness-ui-smoke-proof.json
runtime/v119-beta-release-certification.json
```
