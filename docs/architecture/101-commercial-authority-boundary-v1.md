# Commercial Authority Boundary

更新日期：2026-07-10
执行者：Codex

## Purpose

本文冻结 generic Paid Report commercial backend authority boundary。

它回答一个问题：

```text
哪些组件可以创建 / 更新商业后端 authority facts？
哪些 projection、view、sidecar 永远只能读？
```

## Contract Version

```text
agentflow-commercial-authority-boundary.v1
```

Release proof:

```text
artifacts/release-gate-<version>-e2e/runtime/v130-commercial-authority-boundary.json
```

## Writable Authority Map

| Authority Area | Stable Object | Writer |
| --- | --- | --- |
| Order | `PaidReportOrderRecord` | Order Runtime |
| Entitlement | `PaidReportEntitlementAuthorization` | Entitlement Runtime |
| Run admission | `PaidReportOrderToRunAdmission` | Runtime Admission |
| Run | `PaidReportRunExecutionReceipt` | Execution Runtime |
| Artifact | `PaidReportArtifact` | Artifact Runtime |
| Evidence | `PaidReportEvidencePack` | Evidence Runtime |
| Decision | `PaidReportDecisionRecord` | Decision Runtime |
| Access receipt | `PaidReportAccessReceipt` | Access Runtime |
| Commercial policy | `PaidReportCommercialPolicyRecord` | Commercial Policy Runtime |

## Read-only Surfaces

这些 surface 永远不能写商业 authority：

```text
Projection
Customer View
Download View
Synthetic Release Fixture
Release Sidecar
```

Read-only projection objects:

```text
PaidReportDeliveryPackageProjection
PaidReportCustomerDeliveryAccessProjection
PaidReportFeedbackLoopProjection
```

它们可以展示 delivery / customer access / feedback 状态，但不能创建或更新
Order、Entitlement、Run、Artifact、Evidence、Decision、Access Receipt 或 Policy
authority facts。

## Release Sidecar Rule

Synthetic release fixtures and release sidecars are evidence only.

They cannot satisfy live release authority. Live release authority must come
from published GitHub release provenance and matching source commit.

## Negative Fixtures

Release gate must reject:

```text
projection-writing-authority
customer-view-writing-authority
download-view-writing-authority
synthetic-release-sidecar-promoted-as-authority
release-sidecar-promoted-as-authority
```

Every negative fixture must be machine-readable and set:

```text
canWriteAuthority = false
status = failed-as-expected
failureReason != ""
```

## Release Gate

`scripts/verify_release_gate.sh` must run:

```text
cargo run -p agentflow-runtime-api --example v130_commercial_authority_boundary_proofs -- \
  <runtime-dir>/v130-commercial-authority-boundary.json
```

The gate fails if:

- any required authority object is missing from the map;
- a projection-only object can create, update, or write authority;
- Projection / Customer View / Download View is not listed as read-only;
- synthetic release fixture or release sidecar can be promoted to live authority;
- any negative fixture lacks a failure reason.
