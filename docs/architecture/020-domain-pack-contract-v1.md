# Domain Pack Contract V1

日期：2026-06-23
执行者：Codex

## Purpose

`Domain Pack Contract` 定义 Pack 如何表达行业对象世界。

它回答：

```text
一个行业 Pack 有哪些对象？
对象之间如何关联？
对象状态如何变化？
哪些动作可执行？
验收语义、证据规则和审计提示如何表达？
Domain Pack 能否直接写事件？
```

本合同只定义 Domain Pack 的可执行定义边界。

它不实现行业客户端 UI，不绕过 Runtime authority，也不把 Audit 放回主业务链路。

## Domain Path

Domain Pack 固定放在：

```text
.agentflow/packs/<pack-id>/domain/
```

Domain 入口文件后续可以由 manifest 指向，但 Domain schema 的逻辑模型必须至少表达：

```text
object types
link types
state machines
action semantics
acceptance semantics
evidence policy
audit trigger hints
migration compatibility
```

## Runtime Boundary

Domain Pack 是定义，不是事件源。

它不能直接写：

```text
.agentflow/events/**
.agentflow/spec/**
.agentflow/tasks/**
.agentflow/audit/**
```

Domain Pack 输出的 action semantics 只能被这些 Runtime 层读取：

```text
Action Contract
Action Arbitration
Simulation / Dry-run
Projection
Command Surface
```

真正执行仍然必须经过：

```text
Runtime API / Command Surface
-> Action Proposal
-> Arbitration
-> Event Store
-> Projection
```

## Software Dev Domain

Software Dev Pack 的初始对象世界：

```text
Requirement
Spec
Issue
Run
PullRequest
Release
Evidence
Finding
```

它的主链是：

```text
Requirement
-> Spec
-> Issue
-> Run
-> Evidence / PullRequest
-> Release
```

Audit 仍然是 sidecar，不进入 Software Dev 主业务链路。

## UI Design Domain

UI Design Pack 的初始对象世界：

```text
ProductBrief
Prd
Direction
Wireframe
HiFi
DesignSystem
Page
Handoff
Evidence
```

它的主链是：

```text
ProductBrief
-> Prd
-> Direction
-> Wireframe
-> HiFi
-> Handoff
```

UI Design Pack 不能复用 Software Dev 的 `Issue / Run` 作为唯一业务对象。

## Schema Anchor

`agentflow-pack` 提供 Domain Pack 的最小 schema：

```text
PackDomainDefinition
PackDomainValidationReport
software_dev_domain_definition()
ui_design_domain_definition()
validate_domain_definition()
```

校验规则至少保证：

- `writesEvents` 必须为 `false`；
- object type 必须声明；
- link / state / action / acceptance 必须引用已声明 object type；
- action semantics 必须包含 `contractRef`、`arbitrationRef`、`simulationRef`；
- Software Dev 与 UI Design 的 domain 差异能被 schema 表达。

## Non-goals

本合同不做：

- Surface Pack；
- Connector Pack；
- Pack loader；
- Pack marketplace；
- 远程行业客户端；
- 自动生成行业 UI；
- 直接写 Runtime authority。

