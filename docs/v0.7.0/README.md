# AgentFlow v0.7.0 Projection Surface & Project OS Console

日期：2026-06-21
执行者：Codex
状态：Version Planning Draft / 开发前置文档 / 不授权 Build Agent 执行

## 1. Purpose

本目录沉淀 AgentFlow `v0.7.0` 的版本目标和开发任务规划。

`v0.7.0` 的核心目标是：

```text
Projection Surface & Project OS Console
```

大白话：

```text
v0.4.0 到 v0.6.1 把底层事实链、Spec Loop、Work Loop、验收和完成写回打稳。
v0.7.0 要把这些事实变成用户真正能用的项目控制台。
```

它不是再补一轮底层 runtime。

它要回答：

- 当前项目在哪个阶段；
- 当前任务为什么停在这里；
- 下一步该做什么；
- Spec / Work / Acceptance / Delivery / Audit 的状态各是什么；
- 哪些事实是 authority；
- 哪些只是只读投影；
- 用户操作如何通过 Command Surface 回流 Runtime API。

## 2. Baseline

`v0.7.0` 建立在以下前提上：

- `v0.4.0` Runtime Foundation 已提供 Ontology、Action Contract、Arbitration、Event Store、Projection、Runtime API；
- `v0.5.0` Spec Loop 已提供需求到 SPEC / Issue / Runtime Action Proposal 的主链；
- `v0.6.0` Work Loop 已提供受控执行、preflight、lock、queue、session、evidence、projection 和 Done writeback；
- `v0.6.1` 必须先收口 release closeout、Acceptance Gate、Completion Commit 和 Audit separation。

如果 `v0.6.1` 未完成，`v0.7.0` 只能停留在规划层。

原因很简单：

```text
Console 只能展示稳定事实。
底层验收和完成写入不稳定时，Console 会把不稳定状态产品化。
```

## 3. Main Chain

`v0.7.0` 的主链是：

```text
Event Store / Spec Facts / Task Facts / Audit Facts
-> Projection Read Models
-> View Models
-> Project OS Console
-> Command Surface
-> Runtime API
-> Action Proposal
```

关键规则：

- Projection 只读；
- View Model 是页面状态，不是事实源；
- UI 不直接写 `.agentflow/**`；
- Command Surface 只提交 command / proposal；
- Runtime 接受后才进入 Event Store；
- Audit Surface 只展示独立审计事实，不执行审计。

## 4. Scope

`v0.7.0` 包含：

- Projection Surface contract；
- Project OS Console information architecture；
- Projection Query API；
- Project Home；
- Spec Workbench；
- Task Workbench；
- Event Timeline；
- Evidence Graph；
- Acceptance / Delivery Surface；
- Audit read-only surface；
- Command Surface；
- Advanced Runtime Diagnostics；
- Desktop view model and browser preview regression。

## 5. Non-goals

`v0.7.0` 不包含：

- Domain Pack / Surface Pack / Connector Pack；
- 多行业产品壳；
- 云端 Runtime；
- Message Bus 中心化；
- Eve / Vercel adapter；
- Pack 市场；
- 自动审计；
- 大规模多 Agent 调度；
- 让 UI 直接修改事实源。

## 6. Reading Order

1. [AGENTFLOW_V0_7_0_PROJECTION_SURFACE_OS_CONSOLE_TASKS_V1.md](AGENTFLOW_V0_7_0_PROJECTION_SURFACE_OS_CONSOLE_TASKS_V1.md)
2. [../architecture/011-projection-surface-console-ia-v1.md](../architecture/011-projection-surface-console-ia-v1.md)

## 7. Development Entry

第一条可执行任务应该是：

```text
V070-001 Projection Surface Contract and Console IA
```

原因：

- 没有 Projection Surface contract，页面会各自解释事实；
- 没有 Console IA，Project Home、Task Workbench、Audit Surface 会互相抢职责；
- 没有 Command Surface 边界，UI 很容易退回直接改状态。

实现边界：

- Projection Surface contract 和 Console IA 以 [../architecture/011-projection-surface-console-ia-v1.md](../architecture/011-projection-surface-console-ia-v1.md) 为准；
- 后续 V070 issues 必须引用该文档，不允许页面绕过 projection / runtime-api 边界。

## 8. Completion Standard

`v0.7.0` 完成时，必须满足：

- Project Home 能解释项目阶段、下一步和阻塞原因；
- Spec Workbench 能展示 Spec Loop 状态和确认链；
- Task Workbench 能展示 issue / run / session / verification / evidence / acceptance / delivery；
- Event Timeline 能解释状态为什么变成现在这样；
- Evidence Graph 能追溯 requirement -> spec -> issue -> run -> evidence -> acceptance -> delivery；
- Audit Surface 只读独立审计事实；
- Command Surface 所有操作都回流 Runtime API；
- UI 不能直接写事实源；
- Projection / View Model 不是 authority；
- 软件开发场景可以从需求、拆解、执行、验收、交付、审计阅读完成闭环。

## 9. Boundary

本目录只是 `v0.7.0` 的开发前置规划。

它不代表：

- 已写入正式 `docs/requirements/**`；
- 已写入 `.agentflow/spec/**`；
- 已授权 Build Agent 执行；
- 已创建 GitHub issue；
- 已进入实现阶段。

进入正式执行前，仍需要按 AgentFlow 当前规则生成 requirement 和 spec issue 合同。
