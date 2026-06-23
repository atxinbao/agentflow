# Surface Pack Contract V1

日期：2026-06-23
执行者：Codex

## Purpose

`Surface Pack Contract` 定义 Pack 如何表达行业客户端的页面、工作台、视图模型、命令入口和状态展示规则。

它回答：

```text
一个行业 Pack 需要哪些页面？
哪些页面是主流程，哪些页面是 sidecar？
页面读取哪些 Projection？
页面能发起哪些 Command？
空态、加载态、错误态如何声明？
Surface Pack 能否直接写 authority？
```

本合同只定义行业客户端呈现和交互表面。

它不实现前端 UI，不写 `.agentflow/**` 事实，也不让 Pack 绕过 Runtime API / Command Surface。

## Surface Path

Surface Pack 固定放在：

```text
.agentflow/packs/<pack-id>/surface/
```

Surface 入口文件后续可以由 manifest 指向，但 Surface schema 的逻辑模型必须至少表达：

```text
page registry
workbench registry
view model mapping
command entry mapping
read model dependencies
navigation rules
empty / loading / error state
sidecar surfaces
```

## Runtime Boundary

Surface Pack 是展示定义，不是事实源。

它不能直接写：

```text
.agentflow/spec/**
.agentflow/events/**
.agentflow/projections/**
.agentflow/tasks/**
.agentflow/audit/**
```

Surface Pack 只能做两件事：

```text
Read Projection
Send Runtime Command / Action Proposal
```

正式写入仍然必须经过：

```text
Runtime API / Command Surface
-> Action Proposal
-> Arbitration
-> Event Store
-> Projection
```

## Software Dev Surface

Software Dev Pack 的主 Surface：

```text
Project Home
Spec Workbench
Task Workbench
Acceptance
Delivery
Event Timeline
Evidence Graph
```

Software Dev Pack 的 sidecar Surface：

```text
Audit Surface
Finding Review
Follow-up Proposal
```

Audit Surface 是 sidecar，只能辅助验收、复核和后续建议，不能重新进入主业务链路，也不能成为普通任务 Done 的阻塞条件。

## UI Design Surface

UI Design Pack 的主 Surface：

```text
Design Home
Brief Intake
Direction Board
Wireframe Board
HiFi Review
Design System
Handoff Surface
```

UI Design Surface 必须能独立表达设计现场。

它不能复用 Software Dev 的 `Task Workbench` 来伪装设计流程，也不能把设计对象强行映射成代码任务。

## Schema Anchor

`agentflow-pack` 提供 Surface Pack 的最小 schema：

```text
PackSurfaceDefinition
PackSurfaceValidationReport
software_dev_surface_definition()
ui_design_surface_definition()
validate_surface_definition()
```

校验规则至少保证：

- `writesAuthority` 必须为 `false`；
- page registry 必须声明；
- view model mapping 必须指向 Projection；
- read model dependency 必须指向 Projection；
- workbench、command、navigation、sidecar 必须引用已声明 page；
- sidecar surface 必须引用 `sidecar` 类型页面；
- sidecar surface 不能阻塞主业务链路；
- Software Dev Audit Surface 必须是 sidecar；
- UI Design Surface 必须有独立设计页面，不能用 Task Workbench 伪装。

## Non-goals

本合同不做：

- 完整前端 UI；
- Pack loader；
- Connector Pack；
- Pack simulation；
- 远程行业客户端；
- 自动生成页面；
- 直接写 Runtime authority。
