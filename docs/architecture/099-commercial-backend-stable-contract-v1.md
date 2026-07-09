# 099 Commercial Backend Stable Contract V1

更新日期：2026-07-10
执行者：Codex

## Purpose

本文档定义 `v1.3.0` 的 Commercial Backend Stable Contract。它冻结的是
generic Paid Report Flow 后端合同，不是某个具体行业产品。

```text
Product / Order / Entitlement / Run / Artifact / Evidence / Decision / Delivery / Feedback
```

这些对象可以被未来 Product surface、Pack、行业 SKU 或客户交付界面读取，但 Core
Runtime 不拥有具体行业文案、价格、模型提示词、支付 provider 执行或最终报告生成。

## Machine-readable Baseline

稳定合同的机器基线由 Runtime API 生成：

```text
agentflow-commercial-backend-stable-contract.v1
```

release gate 产物路径：

```text
runtime/v130-commercial-backend-stable-contract.json
```

该 JSON 必须列出：

- 每个稳定对象；
- 对象版本字符串；
- required fields；
- optional/defaulted fields；
- status values；
- error / decision states；
- migration / version bump policy。

## Stable Object Groups

| Group | Objects |
| --- | --- |
| Product | Product Definition, Product Instance |
| Runtime | Runtime Proposal Handoff, Runtime Admission Receipt |
| Order | Order Intent, Input Snapshot, Order Record |
| Entitlement | Entitlement Authorization |
| Run | Run Contract, Order-to-Run Admission, Run Execution Receipt |
| Artifact | Report Artifact |
| Evidence | Evidence Pack |
| Decision | Decision Record |
| Delivery | Delivery Package Projection, Customer Delivery Access, Access Receipt |
| Feedback | Feedback Loop Projection, Commercial Policy Record |

## Stable Error / Decision States

The stable model must preserve these states after `v1.3.0`:

```text
invalid
deferred
blocked
accepted
revoked
expired
refunded
repair-needed
delivery-ready
```

Blocked / invalid / denied paths must expose machine-readable `failureReasons` or
`blockedReason`. A product surface may translate those reasons, but it must not erase
the underlying fact.

## Migration Rule

After `v1.3.0`, any backward-incompatible change to this backend contract requires:

1. a new version string or explicit migration;
2. a release proof artifact;
3. an updated release gate check;
4. documentation of Product / Pack compatibility impact.

## Non-goals

This contract does not include:

- concrete Bazi / legal / contract / feng shui / study abroad / diligence / naming SKU;
- model-provider-specific final report generation;
- public commercial launch;
- cloud multi-tenant launch;
- full customer account system;
- production payment checkout / charge / refund execution;
- Product console redesign beyond proof/read-model hooks.
