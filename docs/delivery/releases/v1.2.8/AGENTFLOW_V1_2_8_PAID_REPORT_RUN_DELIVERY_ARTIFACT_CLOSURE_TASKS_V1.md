# AgentFlow v1.2.8 Paid Report Run and Delivery Artifact Closure Tasks

更新日期：2026-07-09
执行者：Codex

This document records the public delivery traceability for `v1.2.8`.

## Task Traceability

| Task | GitHub Issue | Title | Status | Primary proof |
| --- | --- | --- | --- | --- |
| V128-001 | #967 | Release Provenance and Tag Policy Repair | done | `runtime/v128-release-provenance-tag-policy-repair.json` |
| V128-002 | #968 | Project-unique Product Instance Identity | done | `runtime/v128-project-unique-product-instance-identity.json` |
| V128-003 | #969 | Paid Report Input Snapshot and Order Intent Contract | done | `runtime/v128-paid-report-input-snapshot-order-intent-contract.json` |
| V128-004 | #970 | Paid Report Run Execution Receipt | done | `runtime/v128-paid-report-run-execution-receipt.json` |
| V128-005 | #971 | Report Artifact Schema and Storage Boundary | done | `runtime/v128-report-artifact-schema-storage-boundary.json` |
| V128-006 | #972 | Report Generation Evidence Capture | done | `runtime/v128-report-generation-evidence-capture.json` |
| V128-007 | #973 | Decision Gate for Report Delivery | done | `runtime/v128-decision-gate-report-delivery.json` |
| V128-008 | #974 | Delivery Package Projection and Download Contract | done | `runtime/v128-delivery-package-projection-download-contract.json` |
| V128-009 | #975 | Feedback and Repair Request Loop | done | `runtime/v128-feedback-repair-request-loop.json` |
| V128-010 | #976 | v1.2.8 Release Certification | done | `runtime/v128-release-certification.json` |

## Dependency Order

```text
#967
-> #968
-> #969
-> #970
-> #971
-> #972
-> #973
-> #974
-> #975
-> #976
```

## Certified Boundary

`v1.2.8` keeps the v1.2.7 project-scoped Paid Report Runtime handoff baseline, then closes the generic report delivery artifact boundary:

- Software Dev remains the Managed Project Flow Reference App；
- Paid Report remains a generic runtime delivery flow；
- project/workspace identity is part of the product instance id；
- input snapshot and order intent are required before run execution；
- run execution receipt records success and blocked states；
- report artifact schema is generic and project-scoped；
- generation evidence must satisfy the product evidence policy；
- decision gate controls delivery readiness；
- delivery package projection is read-only；
- feedback/repair creates controlled follow-up routes instead of mutating delivered artifacts；
- release certification records all primary proofs.

## Release Gate Artifacts

The release certification uses the standard release-gate artifact bundle for `v1.2.8`.

The final certification artifact must include:

```text
certificationKind
releaseVersion
releaseTag
sourceCommit
workflowRunId
artifactNames
primaryProofs
runtimeReleaseCertificationPath = runtime/v128-release-certification.json
tagPolicy
```

## V128 Primary Proof Alignment

V128 issue traceability is certified by these primary proof paths:

```text
#967 -> runtime/v128-release-provenance-tag-policy-repair.json
#968 -> runtime/v128-project-unique-product-instance-identity.json
#969 -> runtime/v128-paid-report-input-snapshot-order-intent-contract.json
#970 -> runtime/v128-paid-report-run-execution-receipt.json
#971 -> runtime/v128-report-artifact-schema-storage-boundary.json
#972 -> runtime/v128-report-generation-evidence-capture.json
#973 -> runtime/v128-decision-gate-report-delivery.json
#974 -> runtime/v128-delivery-package-projection-download-contract.json
#975 -> runtime/v128-feedback-repair-request-loop.json
#976 -> runtime/v128-release-certification.json
```
