# AgentFlow v1.2.7 Paid Report Runtime Handoff Closure Tasks

更新日期：2026-07-08
执行者：Codex

This document records the public delivery traceability for `v1.2.7`.

## Task Traceability

| Task | GitHub Issue | Title | Status | Primary proof |
| --- | --- | --- | --- | --- |
| V127-001 | #956 | Next Release Planning Alignment and Roadmap Refresh | done | `runtime/v127-next-release-planning-alignment.json` |
| V127-002 | #957 | Product Flow Source Boundary for Paid Report vs Software Dev | done | `runtime/v127-product-flow-source-boundary.json` |
| V127-003 | #958 | Project-scoped Paid Report Instance Resolver | done | `runtime/v127-project-paid-report-instance-resolver.json` |
| V127-004 | #959 | Project-scoped Paid Report Preflight and Handoff API | done | `runtime/v127-project-paid-report-preflight-handoff-api.json` |
| V127-005 | #960 | Desktop Paid Report Preflight Project Root Bridge | done | `runtime/v127-desktop-paid-report-project-root-bridge.json` |
| V127-006 | #961 | Golden Path Source Semantics Certification | done | `runtime/v127-golden-path-source-semantics.json` |
| V127-007 | #962 | Runtime Proposal Admission Receipt for Paid Report Handoff | done | `runtime/v127-runtime-proposal-admission-receipt.json` |
| V127-008 | #963 | Paid Report Run Contract Boundary | done | `runtime/v127-paid-report-run-contract-boundary.json` |
| V127-009 | #964 | Paid Report Evidence Decision Delivery Projection Contract | done | `runtime/v127-paid-report-evidence-decision-delivery-projection-contract.json` |
| V127-010 | #965 | v1.2.7 Release Certification | done | `runtime/v127-release-certification.json` |

## Dependency Order

```text
#956
-> #957
-> #958
-> #959
-> #960
-> #961
-> #962
-> #963
-> #964
-> #965
```

## Certified Boundary

`v1.2.7` keeps the v1.2.6 project-scoped commercial product instance baseline, then closes the generic Paid Report Runtime handoff boundary:

- Software Dev remains the Managed Project Flow Reference App；
- Paid Report remains a generic backend handoff flow；
- concrete paid report SKU names do not become Core authority；
- project root is used to resolve Paid Report product instance and handoff；
- allowed preflight creates Runtime proposal handoff only；
- Runtime admission receipt is required before run contract；
- run contract cannot start directly from preflight；
- delivery projection reads evidence / decision and never writes authority；
- release certification records all primary proofs.

## Release Gate Artifacts

The release certification uses the standard release-gate artifact bundle for `v1.2.7`.

The final certification artifact must include:

```text
certificationKind
releaseVersion
releaseTag
sourceCommit
workflowRunId
artifactNames
primaryProofs
runtimeReleaseCertificationPath = runtime/v127-release-certification.json
```

## V127 Primary Proof Alignment

V127 issue traceability is certified by these primary proof paths:

```text
#956 -> runtime/v127-next-release-planning-alignment.json
#957 -> runtime/v127-product-flow-source-boundary.json
#958 -> runtime/v127-project-paid-report-instance-resolver.json
#959 -> runtime/v127-project-paid-report-preflight-handoff-api.json
#960 -> runtime/v127-desktop-paid-report-project-root-bridge.json
#961 -> runtime/v127-golden-path-source-semantics.json
#962 -> runtime/v127-runtime-proposal-admission-receipt.json
#963 -> runtime/v127-paid-report-run-contract-boundary.json
#964 -> runtime/v127-paid-report-evidence-decision-delivery-projection-contract.json
#965 -> runtime/v127-release-certification.json
```
