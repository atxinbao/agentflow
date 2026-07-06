# Paid Feature Boundary v1

创建日期：2026-07-06  
执行者：Codex

## Purpose

本文定义 AgentFlow 商业产品层里的 paid feature boundary。

它回答三个问题：

- 哪些产品能力可以标记为 free、paid、deferred 或 unavailable；
- 缺少 entitlement 时，paid-only action 如何在执行前被阻断；
- UI 如何解释某个命令为什么不可用。

## Authority Boundary

Paid feature boundary 是 Product-layer read model。

它不是 Core Runtime authority。

它不能直接写：

```text
.agentflow/spec/**
.agentflow/events/**
.agentflow/tasks/**
Core Runtime accepted action
Evidence
Decision
Completion
```

它只能在 Runtime command admission 之前给 product surface 提供可读状态。

## Concepts

| Concept | Meaning | Authority |
| --- | --- | --- |
| Feature | 产品能力、命令入口、报告入口或 workspace action | Commercial Product Layer |
| Feature Tier | `free` / `paid` / `deferred` / `unavailable` | Product-layer read model |
| Feature Access | 当前用户或 workspace 对某能力的可用性 | Product-layer read model |
| Upgrade Required | 需要有效 entitlement 才能提交 | Product-layer read model |
| Availability Reason | UI 给用户展示的不可用原因 | Product-layer read model |
| Runtime Admission | Core Runtime 最终命令准入 | Core Runtime |

## Feature Tiers

| Tier | Meaning | Submit policy |
| --- | --- | --- |
| `free` | 不需要 paid entitlement 的能力 | allowed |
| `paid` | 需要 active 或 trial entitlement 的能力 | entitlement-gated |
| `deferred` | 产品层无法确定资格，必须稍后重试 | deferred |
| `unavailable` | 当前版本、workspace 或产品层禁用 | rejected |

## Paid Feature Read Model

Desktop 或 Product surface 只能读取 paid feature projection。

最小字段：

```json
{
  "version": "agentflow-paid-feature-read-model.v1",
  "productId": "software-dev",
  "features": [
    {
      "id": "paid-report",
      "label": "Paid report",
      "tier": "paid",
      "entitlementId": "paid-report",
      "entitlementState": "disabled",
      "available": false,
      "submitPolicy": "rejected",
      "runtimeAdmissionAllowed": false,
      "reason": {
        "code": "upgrade-required",
        "message": "paid-report requires active entitlement"
      }
    }
  ]
}
```

## Submit Rule

Paid feature boundary 必须先消费 License / Entitlement read model。

规则：

```text
free + any entitlement state       -> submit allowed
paid + active entitlement          -> submit allowed
paid + trial entitlement           -> submit allowed with trial boundary
paid + expired entitlement         -> submit rejected
paid + disabled entitlement        -> submit rejected
paid + unknown entitlement         -> submit invalid
paid + deferred entitlement        -> submit deferred
deferred feature                   -> submit deferred
unavailable feature                -> submit rejected
```

`rejected`、`invalid`、`deferred` 都不能进入 Core Runtime command admission。

只有 `allowed` 和 `allowed-with-trial-boundary` 可以继续生成 Runtime command proposal。

即使 paid feature status 是 `allowed`，Core Runtime 仍必须重新执行自己的 command admission。

## UI Explanation Rule

当 command 不可用时，UI 必须能展示人能看懂的原因。

示例：

| Reason code | UI meaning |
| --- | --- |
| `upgrade-required` | 这个能力需要有效授权 |
| `entitlement-expired` | 授权已过期 |
| `entitlement-deferred` | 授权状态暂时不可确认 |
| `feature-unavailable` | 当前版本不开放这个能力 |
| `feature-unknown` | 功能状态不完整，不能提交 |

UI 只能展示、解释和禁用入口。

UI 不能绕过 Product-layer read model 去直接提交 Runtime action。

## Non-goals

`v1.2.2` 不实现：

- checkout；
- payment provider；
- billing account；
- subscription management；
- customer account；
- cloud entitlement server；
- paid report generation；
- automatic upgrade flow。

## Testable Fixtures

Release proof 至少要覆盖：

| Feature tier | Entitlement state | Expected submit policy | Runtime admission |
| --- | --- | --- | --- |
| free | disabled | allowed | allowed-to-propose |
| paid | active | allowed | allowed-to-propose |
| paid | trial | allowed-with-trial-boundary | allowed-to-propose |
| paid | expired | rejected | blocked-before-runtime |
| paid | disabled | rejected | blocked-before-runtime |
| paid | deferred | deferred | blocked-before-runtime |
| paid | unknown | invalid | blocked-before-runtime |
| deferred | deferred | deferred | blocked-before-runtime |
| unavailable | active | rejected | blocked-before-runtime |

## Release Proof Requirement

Release gate 必须生成：

```text
runtime/v121-paid-feature-boundary.json
```

并证明：

- paid feature boundary 是 Product-layer read model；
- feature tiers 覆盖 free / paid / deferred / unavailable；
- paid-only action 缺少 entitlement 时会在 Runtime admission 前被阻断；
- UI 可以解释 unavailable / deferred / upgrade-required 原因；
- paid feature status 不会绕过 Core Runtime command admission；
- 不依赖 payment provider integration。
