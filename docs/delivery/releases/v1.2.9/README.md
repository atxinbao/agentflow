# AgentFlow v1.2.9 Paid Report Commercial Order and Access Closure

更新日期：2026-07-09
执行者：Codex

## Release Baseline

`v1.2.9` 是 Paid Report Commercial Order and Access Closure release baseline。

这一版建立在 `v1.2.8` Paid Report Run and Delivery Artifact Closure 之上，把 Paid Report 从运行与交付产物闭环推进到商业订单、授权、客户访问和修复/退款策略边界：

```text
release provenance / facts commit alignment
-> annotated / lightweight tag kind certification
-> paid report order record
-> payment / entitlement authorization boundary
-> order-to-run admission gate
-> customer delivery access projection
-> report download token / access receipt
-> refund / repair / rerun policy contract
-> commercial negative fixtures
-> v1.2.9 release certification
```

## Scope

`v1.2.9` 收口以下内容：

1. 修复 published release certification 中真实 release provenance 与 synthetic project release facts 混用的问题。
2. 明确 annotated tag 与 lightweight tag 的 tag kind、tag object id 和 peeled commit sha 记录规则。
3. 定义 Paid Report Order Record，绑定 customer request、Product Instance、order intent、input snapshot 和 offer metadata。
4. 定义 payment / entitlement authorization 边界，只证明运行授权，不实现 provider checkout。
5. 定义 Order-to-Run Admission Gate，Runtime 不能只凭 input snapshot 接收 report run。
6. 定义 Customer Delivery Access Projection，只读展示 accessible / blocked / expired / repair-needed 状态。
7. 定义 Report Download Token / Access Receipt，记录客户可访问交付包的访问事实。
8. 定义 Refund / Repair / Rerun Policy Contract，不原地修改已交付 artifact。
9. 增加商业负向 fixtures，防止假 paid 状态、stale release facts、mismatch order/run 和 unauthorized delivery access 通过认证。
10. 生成 v1.2.9 release certification，记录 primary proofs、tag、source commit、workflow run 和非目标。

## Certified Boundary

`v1.2.9` 认证的是 generic Paid Report Flow 的 commercial order / access backend boundary，不是商业公开上线。

这一版确认：

- Software Dev 仍是 Managed Project Flow Reference App；
- Paid Report 仍是 generic flow，不是具体行业 SKU；
- Core Runtime 只认识 Product Instance / Order / Entitlement / Run / Artifact / Evidence / Decision / Delivery / Feedback；
- payment provider checkout 不属于 Core Runtime authority；
- Order-to-Run 必须通过 order record、authorization、input snapshot 和 runtime receipt；
- Customer delivery access 是 read-only projection，不能写 delivery / decision / entitlement authority；
- access receipt 可以记录 allowed / expired / revoked；
- refund request 是 commercial policy state，不是 provider payment execution；
- repair / rerun 只能生成受控 follow-up proposal，不能原地修改已交付 artifact。

## Non-goals

`v1.2.9` 不包含：

- concrete paid-report industry SKU；
- model/provider-specific final report generation；
- public commercial launch；
- cloud multi-tenant launch；
- full customer account system；
- payment provider checkout / charge / refund execution；
- Product console redesign。

## Primary Proof Index

| Proof | Path / URL | Purpose |
| --- | --- | --- |
| Release provenance facts alignment | `runtime/v129-release-provenance-facts-commit-alignment.json` | proves synthetic project release facts cannot satisfy published release certification |
| Annotated tag kind certification | `runtime/v129-annotated-tag-kind-certification-repair.json` | proves tag object id / peeled commit / lightweight tag semantics |
| Paid report order record | `runtime/v129-paid-report-order-record-contract.json` | proves order record schema and non-runnable missing input cases |
| Payment / entitlement authorization | `runtime/v129-payment-entitlement-authorization-boundary.json` | proves paid / waived / deferred / refunded / missing authorization states |
| Order-to-run admission | `runtime/v129-order-to-run-admission-gate.json` | proves accepted and blocked admission paths |
| Customer delivery access | `runtime/v129-customer-delivery-access-projection.json` | proves read-only accessible / blocked access projection |
| Report access receipt | `runtime/v129-report-download-token-access-receipt.json` | proves allowed / expired / revoked access receipts |
| Refund / repair / rerun policy | `runtime/v129-refund-repair-rerun-policy-contract.json` | proves policy outcomes without artifact mutation |
| Commercial negative fixtures | `runtime/v129-commercial-negative-fixtures.json` | proves negative cases fail with machine-readable reason codes |
| v1.2.9 release certification | `runtime/v129-release-certification.json` | final release certification for v1.2.9 |

## Release Gate

`scripts/verify_release_gate.sh` must run the v1.2.9 proof chain after v1.2.8:

```text
run_v129_commercial_order_access_closure_gate
run_v129_release_certification_gate
```

## GitHub Traceability

This release closes GitHub issues `#979` through `#988`.
