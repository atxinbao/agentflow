# Customer Delivery Backend Contract V1

更新日期：2026-07-10
执行者：Codex

## Purpose

本文定义 `v1.3.0` 的 Customer Delivery Backend contract。

它回答一个问题：

```text
当 paid report 已经有订单、授权、决策、artifact 和 access receipt 后，
客户交付后端如何稳定投影下载、过期、撤销、退款、修复、重跑和反馈状态？
```

## Authority Boundary

Customer Delivery Backend 可以读取：

- order fact；
- entitlement authorization fact；
- decision fact；
- report artifact fact；
- access receipt fact；
- expiry / revocation / refund / repair / rerun / feedback state；
- source refs。

Customer Delivery Backend 可以投影：

- customer delivery read model；
- download visibility；
- access handle availability；
- `nextAction`；
- failure reasons。

Customer Delivery Backend 不能：

- 创建 payment checkout；
- 执行 charge / refund；
- 改写 decision authority；
- 改写 artifact；
- 把 repair / rerun 直接当成下载授权；
- 绕过 revoked / expired / refunded 状态。

## Machine-readable Contract

Release gate proof:

```text
runtime/v130-customer-delivery-backend-contract.json
```

Contract version:

```text
agentflow-customer-delivery-backend-contract.v1
```

Required bindings:

```text
orderId
entitlementAuthorizationRef
decisionId
reportArtifactRef
accessReceiptRef
expiryState
revocationState
refundState
repairState
rerunState
feedbackState
sourceRefs
```

## Stable States

```text
accessible
expired
revoked
refunded
repair-needed
rerun-needed
blocked
```

## Positive Fixture

`accepted-authorized` must prove:

- order exists；
- entitlement authorization exists；
- decision exists；
- artifact exists；
- access receipt exists；
- `accessStatus = accessible`；
- `nextAction = show-download`；
- download access is visible；
- access handle is generated；
- no failure reasons are present。

## Negative Fixtures

Required negative access fixtures:

```text
expired
revoked
refunded
repair-needed
rerun-needed
```

Each negative fixture must prove:

- status is `failed-as-expected`；
- download access is not visible；
- access handle is not generated；
- `nextAction` is explicit；
- failure reasons are present；
- source refs are present。

Expected `nextAction` mapping:

| Fixture | nextAction |
| --- | --- |
| `expired` | `renew-access` |
| `revoked` | `contact-support` |
| `refunded` | `show-refund-policy` |
| `repair-needed` | `create-repair-proposal` |
| `rerun-needed` | `request-new-authorization` |

## Release Gate

`scripts/verify_release_gate.sh` must run:

```text
cargo run -p agentflow-runtime-api --example v130_customer_delivery_backend_contract_proofs -- \
  <runtime-dir>/v130-customer-delivery-backend-contract.json
```

The gate fails if:

- any required binding is missing；
- stable state list is incomplete；
- accepted fixture does not expose download access；
- accepted fixture lacks access receipt；
- negative fixture exposes download access；
- negative fixture generates access handle；
- negative fixture lacks `nextAction` or failure reasons。

## Non-goals

This contract does not implement:

- concrete paid-report industry SKU；
- customer account system；
- public commercial launch；
- cloud multi-tenant launch；
- production payment checkout / charge / refund execution；
- model/provider-specific final report generation。
