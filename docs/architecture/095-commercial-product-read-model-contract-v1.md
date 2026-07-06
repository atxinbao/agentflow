# Commercial Product Read Model Contract v1

创建日期：2026-07-06  
执行者：Codex

## Purpose

本文定义 AgentFlow 商业产品层给 Desktop / Product surface 使用的只读投影合同。

Commercial Product Read Model 只回答一个问题：

```text
这个商业 flow 当前能不能显示、能不能提交、为什么不能提交。
```

它不是 Core Runtime authority，不能替代 Spec、Runtime Action、Evidence、Decision、Projection 或 Completion。

## Version

```text
agentflow-commercial-product-read-model.v1
```

## Authority Boundary

Commercial Product Read Model 是 Product-layer projection。

它可以被 Desktop / Product surface 读取，用来展示商业状态、禁用入口、解释不可用原因。

它不能：

- 直接写 `.agentflow/spec/**`；
- 直接写 `.agentflow/events/**`；
- 直接写 `.agentflow/tasks/**`；
- 直接写 Runtime Action；
- 直接写 Evidence；
- 直接写 Decision；
- 直接写 Completion；
- 把 commercial availability 当作 Core Runtime admission。

合法路径：

```text
Product commercial facts
-> Commercial Product Read Model projection
-> Desktop / Product surface display
-> Runtime command proposal
-> Core Runtime admission
```

## Non-goals

本文不定义：

- account system；
- payment checkout；
- refund engine；
- cloud entitlement server。

## Read Model Fields

最小字段：

```json
{
  "version": "agentflow-commercial-product-read-model.v1",
  "productId": "software-dev",
  "flowType": "paid-report-flow | managed-project-flow",
  "entitlementState": "active | trial | expired | disabled | deferred | unknown | missing",
  "paidFeatureState": "free | paid | deferred | unavailable | unknown",
  "deliveryPromise": "report | project-delivery | none",
  "availability": "available | rejected | deferred | invalid",
  "unavailableReason": "none | missing-product | disabled-entitlement | deferred-entitlement | unknown-flow-type | paid-feature-unavailable | entitlement-expired",
  "commandPolicy": "allowed-to-propose | blocked-before-runtime",
  "projectionOnly": true,
  "coreAuthority": false
}
```

## Flow Semantics

### Paid Report Flow

`paid-report-flow` 是一次性交付报告形态。

Read model 必须表达：

- flow type 是 `paid-report-flow`；
- delivery promise 是 `report`；
- paid feature / entitlement 不满足时，提交在 Runtime 前被阻断；
- Product surface 只能展示状态和解释原因。

### Managed Project Flow

`managed-project-flow` 是长周期项目交付形态。

Read model 必须表达：

- flow type 是 `managed-project-flow`；
- delivery promise 是 `project-delivery`；
- entitlement 可以影响 Product surface 的入口，但不能成为 Runtime authority；
- Runtime command proposal 仍必须经过 Core Runtime admission。

## Availability Rules

| Case | Expected availability | Command policy |
| --- | --- | --- |
| active entitlement + paid report flow | available | allowed-to-propose |
| trial entitlement + managed project flow | available | allowed-to-propose |
| missing product | invalid | blocked-before-runtime |
| disabled entitlement | rejected | blocked-before-runtime |
| deferred entitlement | deferred | blocked-before-runtime |
| unknown flow type | invalid | blocked-before-runtime |
| paid feature unavailable | rejected | blocked-before-runtime |
| expired entitlement | rejected | blocked-before-runtime |

## Negative Fixtures

Release gate 必须覆盖以下负例：

| Fixture | Reason |
| --- | --- |
| missing Product | 不能生成可提交 commercial action |
| disabled entitlement | 不能把 disabled entitlement 降级成 ready |
| deferred entitlement | 不能把 deferred entitlement 当作 ready |
| unknown flow type | 不能提交未知 flow |

所有负例都必须停在 Product-layer admission，不能进入 Core Runtime command admission。

## Desktop / Product Surface Rule

Desktop / Product surface 只能消费 Commercial Product Read Model projection。

它可以：

- 展示 flow type；
- 展示 entitlement state；
- 展示 paid feature state；
- 展示 delivery promise；
- 展示 availability；
- 展示 unavailable reason；
- 禁用不可提交的入口。

它不能：

- 根据 read model 直接写 Runtime action；
- 根据 read model 直接写 `.agentflow/spec/**`；
- 根据 read model 直接写 `.agentflow/tasks/**`；
- 把 commercial read model 当成 Core authority。

## Release Proof Requirement

Release gate 必须生成：

```text
runtime/v123-commercial-product-read-model-contract.json
```

并证明：

- `agentflow-commercial-product-read-model.v1` 已定义；
- flow type 覆盖 `paid-report-flow` 和 `managed-project-flow`；
- 字段覆盖 entitlement state、paid feature state、delivery promise、availability 和 unavailable reason；
- missing Product、disabled entitlement、deferred entitlement、unknown flow type 都会在 Runtime 前阻断；
- Product surface 只消费 projection，不拥有 Core Runtime authority；
- read model 不能被当作 Core Runtime admission。
