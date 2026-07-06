# License / Entitlement Boundary v1

创建日期：2026-07-06  
执行者：Codex

## Purpose

本文定义 AgentFlow 商业产品层里的 License / Entitlement 边界。

它回答三个问题：

- 产品访问资格如何表达；
- paid-only flow 在没有有效授权时如何被阻断；
- 为什么这不等于支付、账户或账单系统。

## Authority Boundary

License / Entitlement 是商业产品层合同，不是 Core Runtime authority。

它可以影响商业 product surface 的可用性。

它不能绕过 Core Runtime 的 Spec、Runtime Action、Evidence、Decision、Projection 和 Completion 合同。

## Concepts

| Concept | Meaning | Authority |
| --- | --- | --- |
| License | 产品或能力包的使用资格记录 | Commercial Product Layer |
| Entitlement | 某个能力、报告、quota 或 paid-only flow 的访问资格 | Commercial Product Layer |
| Usage Limit | 可用次数、额度或窗口期 | Commercial Product Layer |
| Product Access | Desktop / product surface 展示的可用性状态 | Projection read model |
| Paid-only Flow | 需要有效 entitlement 才能提交的商业流程 | Product command admission |

## Entitlement States

| State | Meaning | Paid-only submit |
| --- | --- | --- |
| `active` | 当前授权有效，可使用对应能力 | allowed |
| `trial` | 试用授权有效，但必须展示试用边界 | allowed-with-trial-boundary |
| `expired` | 授权已过期 | rejected |
| `disabled` | 授权被关闭或不存在 | rejected |
| `deferred` | 授权来源暂不可用，不能确认资格 | deferred |
| `unknown` | 数据缺失或 read model 不完整 | invalid |

## Product Access Read Model

Product surface 只能读取投影，不直接读取商业 authority。

Read model 最小字段：

```json
{
  "version": "agentflow-product-access-read-model.v1",
  "productId": "software-dev",
  "status": "active | trial | expired | disabled | deferred | unknown",
  "licenseId": "local-license-or-null",
  "entitlements": [
    {
      "id": "paid-report",
      "state": "disabled",
      "usageLimit": null,
      "remainingUsage": null,
      "paidOnly": true,
      "submitPolicy": "rejected"
    }
  ],
  "blockers": [
    {
      "code": "entitlement-disabled",
      "message": "paid-only flow requires active entitlement"
    }
  ]
}
```

## Paid-only Flow Rule

`disabled`、`expired`、`unknown` 都不能提交 paid-only flow。

`deferred` 也不能提交 paid-only flow，但结果是 deferred，不是 rejected。

规则：

```text
active   -> submit allowed
trial    -> submit allowed with trial boundary
expired  -> submit rejected
disabled -> submit rejected
deferred -> submit deferred
unknown  -> submit invalid
```

## Runtime Boundary

商业产品层可以在 Runtime command admission 前读取 Product Access Read Model。

如果 entitlement 不满足 paid-only flow，Product surface 必须停止在 product-layer admission。

它不能：

- 直接写 Core Runtime accepted action；
- 直接写 `.agentflow/spec/**`；
- 直接写 `.agentflow/tasks/**`；
- 用 GitHub issue、PR 或 provider session 代替授权状态；
- 把 disabled entitlement 降级成 ready。

## Non-goals

`v1.2.2` 不实现：

- payment provider；
- checkout；
- billing account；
- customer account；
- subscription renewal；
- refund workflow；
- license enforcement service；
- cloud entitlement server。

## Testable Fixtures

Release proof 至少要覆盖：

| Fixture | Expected result |
| --- | --- |
| active entitlement + paid-only flow | allowed |
| trial entitlement + paid-only flow | allowed-with-trial-boundary |
| expired entitlement + paid-only flow | rejected |
| disabled entitlement + paid-only flow | rejected |
| deferred entitlement + paid-only flow | deferred |
| unknown entitlement + paid-only flow | invalid |

## Release Proof Requirement

Release gate 必须生成：

```text
runtime/v121-license-entitlement-boundary.json
```

并证明：

- License / Entitlement / Usage Limit / Product Access / Paid-only Flow 被定义；
- entitlement 状态覆盖 active / trial / expired / disabled / deferred / unknown；
- disabled entitlement cannot submit paid-only flows；
- deferred entitlement 不被当作 ready；
- Product Access 是 read model，不是 Core Runtime authority；
- 不依赖 payment provider integration。

