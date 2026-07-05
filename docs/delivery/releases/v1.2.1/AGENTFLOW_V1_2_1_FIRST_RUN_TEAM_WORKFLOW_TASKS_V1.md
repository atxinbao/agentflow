# AgentFlow v1.2.1 First-run Execution Closure and Team Workflow Boundary Tasks

更新日期：2026-07-05
执行者：Codex

This document records the public delivery traceability for `v1.2.1`.

## Task Traceability

| Task | GitHub Issue | Title | Status | Primary proof |
| --- | --- | --- | --- | --- |
| V121-001 | #863 | Desktop First-run Runtime Command Invocation | done | `run_first_run_product_onboarding` Tauri command |
| V121-002 | #864 | Desktop Onboarding Readiness Read Model Binding | done | readiness binding in Desktop onboarding surface |
| V121-003 | #865 | Guided Sample Actual Run Receipt | done | guided sample run receipt and Runtime API tests |
| V121-004 | #866 | Guided Sample Evidence / Decision / Delivery Proof | done | evidence / decision / delivery paths in guided sample |
| V121-005 | #867 | First-run Failure / Retry UI State | done | deterministic failure / retry onboarding receipt |
| V121-006 | #868 | Team Workflow Boundary Contract | done | `docs/architecture/087-team-workflow-boundary-contract-v1.md` |
| V121-007 | #869 | Project Sharing Read Model | done | `project_sharing_read_model(projectRoot, projectId)` |
| V121-008 | #870 | Role / Permission / Handoff View | done | `role_permission_handoff_view(projectRoot, projectId)` |
| V121-009 | #871 | Team-readable Delivery and Decision History | done | `team_delivery_decision_history_view(projectRoot, projectId)` |
| V121-010 | #872 | v1.2.1 Release Certification | done | this delivery baseline and release-gate certification |

## Dependency Order

```text
#863
-> #864
-> #865
-> #866
-> #867
-> #868
-> #869
-> #870
-> #871
-> #872
```

## Certified Boundary

`v1.2.1` keeps the `v1.2.0` first-run onboarding baseline and adds local team workflow read models:

- first-run Runtime command path is callable from Desktop;
- readiness is projection-backed and visible to users;
- guided sample success and failure both produce explicit receipts;
- team workflow does not add cloud user accounts or multi-tenant behavior;
- project sharing is readonly and projection-backed;
- role / permission / handoff is readonly and exposes current owner;
- delivery / decision history explains accepted, rejected, delivered and feedback routes;
- Audit remains optional sidecar.

## Release Gate Artifacts

The release certification uses the standard release-gate artifact bundle for `v1.2.1`.

The small certification artifact must include:

```text
releaseVersion
releaseTag
sourceCommit
workflowRunId
artifactNames
primaryProofs
```

## Excluded Work

The following remain outside `v1.2.1`:

- cloud multi-tenant collaboration；
- public commercial launch；
- payment / billing；
- new industry Product；
- remote organization administration。
