# Commercial Workflow Shapes v1

创建日期：2026-07-06  
执行者：Codex

## Purpose

本文定义 AgentFlow 商业产品层首先支持的两种工作流形态：

- Paid Report Flow；
- Managed Project Flow。

它们是 Product surface 上的商业交付形态，不是新的 Core Runtime。

`v1.2.2` 只定义合同，不实现 payment、customer account、cloud tenant 或新的行业产品。

## Authority Boundary

两种 flow 都必须复用 Core Runtime 的通用事实：

```text
Spec
Evidence
Decision
Delivery
Projection
Completion
```

商业 flow 可以编排 product-layer read model 和 Runtime command proposal。

商业 flow 不能直接写 Core Runtime authority：

```text
.agentflow/spec/**
.agentflow/events/**
.agentflow/tasks/**
accepted Runtime action
Evidence
Decision
Completion
```

## Paid Report Flow

Paid Report Flow 是一次性交付形态。

适用于：

- 用户提交一个明确输入；
- product layer 确认 paid feature / entitlement；
- Runtime 执行受控分析或生成；
- 输出一份可交付报告；
- 用户可以反馈结果。

标准阶段：

```text
input
-> product access check
-> order intent
-> controlled run
-> evidence
-> decision
-> report delivery
-> feedback
```

### Paid Report Mapping

| Flow step | Core mapping | Product responsibility |
| --- | --- | --- |
| input | Spec seed / Runtime command proposal | 收集报告输入 |
| product access check | Product-layer read model | 检查 paid feature / entitlement |
| order intent | Product-layer fact | 记录购买或交付意图，不等于 payment |
| controlled run | Runtime Action | 受控执行 |
| evidence | Evidence | 保存验证证据 |
| decision | Decision | 判断交付是否成立 |
| report delivery | Delivery | 生成客户可读报告 |
| feedback | Projection / feedback route | 收集后续反馈 |

## Managed Project Flow

Managed Project Flow 是长周期项目交付形态。

适用于：

- 用户有一个 goal；
- Spec Agent 将 goal 拆成 requirement / spec / tasks；
- Work Agent 按任务链执行；
- Runtime 逐步产生 evidence 和 decision；
- Delivery Agent 汇总项目交付；
- Audit Agent 可选审计。

标准阶段：

```text
goal
-> spec
-> tasks
-> execution
-> evidence
-> decision
-> delivery
-> feedback
```

### Managed Project Mapping

| Flow step | Core mapping | Product responsibility |
| --- | --- | --- |
| goal | Product / Project intent | 记录目标和项目边界 |
| spec | Spec | 生成可确认需求 |
| tasks | Runtime command proposals | 派生任务链 |
| execution | Runtime Action | 受控执行 |
| evidence | Evidence | 保存每个任务证据 |
| decision | Decision | 判断任务和项目是否可完成 |
| delivery | Delivery | 汇总项目交付记录 |
| feedback | Projection / feedback route | 收集返工或后续需求 |

## Shared Runtime Rule

Paid Report Flow 和 Managed Project Flow 都必须经过：

```text
Product surface
-> paid feature / entitlement read model when needed
-> Runtime command proposal
-> Core Runtime admission
-> Evidence
-> Decision
-> Delivery
-> Projection
```

它们不能因为是商业 flow 就绕过 Runtime command admission。

## Difference

| Dimension | Paid Report Flow | Managed Project Flow |
| --- | --- | --- |
| Delivery shape | 一次性报告 | 多任务项目交付 |
| Input | 明确输入或报告请求 | Goal / requirement |
| Task structure | 通常是单次 controlled run | 多个 tasks 和状态流 |
| Entitlement | 通常需要 paid feature | 可能需要产品级 entitlement |
| Output | Report delivery | Project delivery summary |
| Feedback | 对报告反馈 | 对项目结果、任务或交付反馈 |

## Non-goals

`v1.2.2` 不实现：

- payment checkout；
- order payment lifecycle；
- customer account；
- managed service operations；
- cloud project collaboration；
- new industry product；
- automatic refund；
- commercial SLA。

## Release Proof Requirement

Release gate 必须生成：

```text
runtime/v121-commercial-workflow-shapes.json
```

并证明：

- Paid Report Flow 和 Managed Project Flow 都被定义；
- Paid Report Flow 覆盖 input / order intent / controlled run / delivery / feedback；
- Managed Project Flow 覆盖 goal / spec / tasks / execution / evidence / decision / delivery / feedback；
- 两种 flow 都映射到 Spec / Evidence / Decision / Delivery；
- 两种 flow 都复用 Core Runtime 和 Product surfaces；
- 本 release 不新增 industry product，也不实现 payment。
