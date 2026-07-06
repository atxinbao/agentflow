# Commercial Product Layer Boundary v1

创建日期：2026-07-06  
执行者：Codex

## Purpose

本文定义 AgentFlow 在 `v1.2.2` 引入商业产品层前必须遵守的边界。

商业产品层可以描述产品、订单、授权、使用量、交付、退款和客户反馈。

它不能把这些商业对象写进 Core OS Runtime authority。

## Product Layer Responsibilities

商业产品层可以拥有这些概念：

| Concept | Responsibility |
| --- | --- |
| Product | 商品、套餐、能力包、Reference App 暴露方式 |
| Order | 购买意图、订单状态、订单与交付的关联 |
| License | 使用资格、授权范围、到期和撤销记录 |
| Usage | 用量记录、配额消耗、报告生成次数 |
| Delivery | 面向客户的交付记录和可读摘要 |
| Refund | 退款请求、退款结果、客户可见解释 |
| Customer Feedback | 客户反馈、满意度、问题回流入口 |

这些概念属于商业产品层，不属于 Core Runtime 的通用任务事实源。

## Core Runtime Boundary

Core OS Runtime 仍只拥有这些行业无关能力：

```text
Spec
Ontology
Runtime Action
Evidence
Decision
Projection
Completion
```

Core Runtime 可以接收商业产品层提交的 Runtime command。

Core Runtime 不能直接拥有：

```text
Product
Order
Payment
Refund
Customer
License entitlement
Usage billing
Paid report
```

## Authority Rule

商业产品层只能通过 Runtime API、Projection API 和 Evidence / Decision 合同与 Core 交互。

合法路径：

```text
Commercial Product Surface
-> Runtime API command
-> Core Runtime admission
-> Evidence / Decision
-> Projection read model
-> Public delivery record
```

非法路径：

```text
Commercial Product Surface
-> direct write Core authority
-> direct write .agentflow/spec/**
-> direct write .agentflow/events/**
-> direct write .agentflow/tasks/**
```

## Software Dev Reference App Boundary

Software Dev Reference App 是当前产品 surface 的一个示例。

它不是商业平台本身。

它可以证明：

- 产品 surface 可以消费 Core Runtime；
- 商业产品层可以在 Core 之上定义；
- Reference App 可以展示交付和决策历史。

它不能证明：

- 支付已经实现；
- 订单、退款、授权已经实现；
- public commercial launch 已经完成；
- customer account 或 cloud tenant 已经存在。

## v1.2.2 Non-goals

`v1.2.2` 不做这些事：

- payment processing；
- paid checkout；
- refund workflow；
- license enforcement；
- customer account management；
- cloud multi-tenant workspace；
- public commercial launch；
- a new industry product；
- commercial SLA。

## Release Proof Requirement

Release gate 必须生成：

```text
runtime/v121-commercial-boundary-contract.json
```

并证明：

- 本文档存在并被 release proof 索引；
- Product / Order / License / Usage / Delivery / Refund / Customer Feedback 被定义为商业产品层概念；
- Core Runtime 仍保持 Spec / Ontology / Runtime / Evidence / Decision / Projection / Completion；
- Software Dev Reference App 只作为 product surface 示例；
- payment / refund / license enforcement / public launch 没有被实现或声称已完成。

