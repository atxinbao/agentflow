# Paid Report Flow State Machine

更新日期：2026-07-10
执行者：Codex

## Purpose

本文定义 generic Paid Report backend lifecycle 的稳定状态机。

它只管通用商业后端生命周期，不定义任何具体行业 SKU、报告文案、价格、
模型提示词、支付 provider 执行或 public launch 行为。

## Contract Version

```text
agentflow-paid-report-flow-state-machine.v1
```

Release proof:

```text
artifacts/release-gate-<version>-e2e/runtime/v130-paid-report-flow-state-machine.json
```

## State List

状态机必须覆盖：

```text
draft-order
order-ready
authorized
admitted
running
artifact-ready
evidence-complete
accepted
delivery-ready
feedback-needed
repair-requested
rerun-needs-authorization
refunded
expired
closed
```

## Authority Boundary

Core Runtime owns:

- generic Paid Report lifecycle states;
- transition legality;
- machine-readable failure reasons;
- whether a transition may write accepted authority;
- whether a transition may write delivery-ready authority.

Product / Pack / SKU layers own:

- concrete domain copy;
- concrete report template;
- pricing;
- provider generation;
- concrete payment provider checkout / charge / refund execution.

## Required Positive Chain

```text
draft-order
-> order-ready
-> authorized
-> admitted
-> running
-> artifact-ready
-> evidence-complete
-> accepted
-> delivery-ready
```

Feedback / commercial follow-up branches:

```text
delivery-ready -> feedback-needed -> repair-requested -> rerun-needs-authorization
delivery-ready -> refunded
delivery-ready -> expired
delivery-ready -> closed
```

## Contract Bindings

Transitions bind to existing stable backend objects:

| State Area | Required Contract |
| --- | --- |
| Order | `PaidReportOrderIntent`, `PaidReportInputSnapshot`, `PaidReportOrderRecord` |
| Entitlement | `PaidReportEntitlementAuthorization` |
| Run admission | `PaidReportOrderToRunAdmission`, `PaidReportRunContract` |
| Run execution | `PaidReportRunExecutionReceipt` |
| Artifact | `PaidReportArtifact` |
| Evidence | `PaidReportEvidencePack` |
| Decision | `PaidReportDecisionRecord` |
| Delivery | `PaidReportDeliveryPackageProjection`, `PaidReportCustomerDeliveryAccessProjection`, `PaidReportAccessReceipt` |
| Feedback | `PaidReportFeedbackLoopProjection`, `PaidReportCommercialPolicyRecord` |

## Negative Fixture Rule

Invalid transitions must:

- produce machine-readable `failureReasons`;
- set `writesAuthority = false`;
- set `writesAcceptedAuthority = false`;
- set `writesDeliveryReadyAuthority = false`;
- never create `accepted` or `delivery-ready` authority.

Required invalid examples include:

```text
draft-order -> accepted
order-ready -> delivery-ready
authorized -> running
artifact-ready -> delivery-ready
refunded -> delivery-ready
expired -> delivery-ready
```

## Release Gate

`scripts/verify_release_gate.sh` must run:

```text
cargo run -p agentflow-runtime-api --example v130_paid_report_flow_state_machine_proofs -- \
  <runtime-dir>/v130-paid-report-flow-state-machine.json
```

The gate fails if:

- required states are missing;
- positive fixtures are missing;
- negative fixtures are missing;
- invalid transitions can write authority;
- invalid transitions lack machine-readable failure reasons;
- contract bindings do not cover Order / Entitlement / Run / Artifact / Evidence / Decision / Delivery / Feedback.
