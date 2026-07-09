# AgentFlow v1.2.9 Paid Report Commercial Order and Access Closure Tasks

更新日期：2026-07-09
执行者：Codex

This document records the public delivery traceability for `v1.2.9`.

## Task Traceability

| Task | GitHub Issue | Title | Status | Primary proof |
| --- | --- | --- | --- | --- |
| V129-001 | #979 | Release Provenance Facts Commit Alignment | done | `runtime/v129-release-provenance-facts-commit-alignment.json` |
| V129-002 | #980 | Annotated Tag Kind Certification Repair | done | `runtime/v129-annotated-tag-kind-certification-repair.json` |
| V129-003 | #981 | Paid Report Order Record Contract | done | `runtime/v129-paid-report-order-record-contract.json` |
| V129-004 | #982 | Payment / Entitlement Authorization Boundary | done | `runtime/v129-payment-entitlement-authorization-boundary.json` |
| V129-005 | #983 | Order-to-Run Admission Gate | done | `runtime/v129-order-to-run-admission-gate.json` |
| V129-006 | #984 | Customer Delivery Access Projection | done | `runtime/v129-customer-delivery-access-projection.json` |
| V129-007 | #985 | Report Download Token / Access Receipt | done | `runtime/v129-report-download-token-access-receipt.json` |
| V129-008 | #986 | Refund / Repair / Rerun Policy Contract | done | `runtime/v129-refund-repair-rerun-policy-contract.json` |
| V129-009 | #987 | Commercial Negative Fixtures | done | `runtime/v129-commercial-negative-fixtures.json` |
| V129-010 | #988 | v1.2.9 Release Certification | done | `runtime/v129-release-certification.json` |

## Dependency Order

```text
#979
-> #980
-> #981
-> #982
-> #983
-> #984
-> #985
-> #986
-> #987
-> #988
```

## Certified Boundary

`v1.2.9` keeps the v1.2.8 generic Paid Report run / delivery artifact closure baseline, then closes the commercial order and access boundary:

- Software Dev remains the Managed Project Flow Reference App；
- Paid Report remains a generic commercial flow, not a concrete industry SKU；
- Order Record binds request / Product Instance / order intent / input snapshot / offer metadata；
- Entitlement authorization proves paid / waived / deferred / refunded / missing states without provider checkout；
- Order-to-Run admission requires valid order, authorization, input snapshot and runtime receipt；
- Customer delivery access is read-only projection and does not write authority；
- Access receipt records allowed / expired / revoked states；
- Refund / repair / rerun policy never mutates delivered artifact in place；
- Negative fixtures reject stale release facts, unknown tag kind, fake paid state, mismatched order/run facts, missing accepted decision and expired access token。

## Release Gate Artifacts

The release certification uses the standard release-gate artifact bundle for `v1.2.9`.

The final certification artifact must include:

```text
certificationKind
releaseVersion
releaseTag
sourceCommit
workflowRunId
artifactNames
primaryProofs
runtimeReleaseCertificationPath = runtime/v129-release-certification.json
commercialBoundary
negativeFixtureSummary
```

## V129 Primary Proof Alignment

V129 issue traceability is certified by these primary proof paths:

```text
#979 -> runtime/v129-release-provenance-facts-commit-alignment.json
#980 -> runtime/v129-annotated-tag-kind-certification-repair.json
#981 -> runtime/v129-paid-report-order-record-contract.json
#982 -> runtime/v129-payment-entitlement-authorization-boundary.json
#983 -> runtime/v129-order-to-run-admission-gate.json
#984 -> runtime/v129-customer-delivery-access-projection.json
#985 -> runtime/v129-report-download-token-access-receipt.json
#986 -> runtime/v129-refund-repair-rerun-policy-contract.json
#987 -> runtime/v129-commercial-negative-fixtures.json
#988 -> runtime/v129-release-certification.json
```
