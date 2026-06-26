# Industry AgentFlow Products

更新日期：2026-06-26
执行者：Codex

## Purpose

本目录定义行业 AgentFlow Product 的标准文档结构。

AgentFlow 的上层公式是：

```text
AI OS Project
= Core Runtime
+ Industry AgentFlow Product
```

其中行业产品必须按同一套目录说明：

```text
Industry AgentFlow Product
= Domain Pack
+ Surface Pack
+ Connector Pack
+ Spec Workflow
+ Evidence / Decision Rules
+ Delivery Model
```

## Standard Directory

每个行业目录必须包含：

```text
<industry>/
  README.md
  product-goal.md
  domain-pack.md
  surface-pack.md
  connector-pack.md
  spec-workflow.md
  evidence-and-decision.md
  delivery-model.md
  examples/
    README.md
```

## Current Industry

| 行业 | 状态 | 入口 |
| --- | --- | --- |
| Software Dev | 当前唯一商业产品主线 | [software-dev/README.md](software-dev/README.md) |

## Boundary

`docs/project/industry/**` 是人类可读的行业产品合同。

未来机器可执行合同应落在：

```text
.agentflow/packs/<industry>/
```

不要把 `docs/project/industry/**` 直接当作 Runtime authority，也不要让 GitHub issues 替代行业 Pack 合同。
