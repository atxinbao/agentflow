# AgentFlow Product Baseline

更新日期：2026-06-26
执行者：Codex

## Purpose

本目录定义 AgentFlow 当前产品目标和产品边界。

当前产品目标是：

```text
AgentFlow = Spec-Driven Software Dev Workflow
```

AgentFlow 不是 Agent Runner，也不是 Spec 文档生成器，而是面向软件开发团队的 Spec 驱动 Agent 工作流产品。

## Current Product Formula

```text
AgentFlow AI OS Project
= Spec-Driven Core Runtime
+ Industry AgentFlow Product
```

当前唯一商业产品主线：

```text
AgentFlow Software Dev Product
= Core Runtime
+ Software Dev Domain Pack
+ Software Dev Surface Pack
+ Software Dev Connector Pack
```

## Current Documents

| 文档 | 作用 |
| --- | --- |
| [006-spec-driven-software-dev-product-goal-v1.md](006-spec-driven-software-dev-product-goal-v1.md) | 当前项目 goal：Spec-Driven Software Dev Workflow |
| [design-system.md](design-system.md) | 定义当前桌面客户端设计基线和前端 Foundation 规则 |
| [../industries/software-dev/README.md](../industries/software-dev/README.md) | Software Dev 行业产品合同 |

## Historical Product Docs

旧 Project Operating Model 文档已移入：

```text
docs/archive/2026-06-current-baseline-history/product/
```

历史文档可用于理解演进过程，但不再作为当前产品入口。

## Boundary

AgentFlow 当前做：

- Software Dev 的 Spec-Driven Workflow；
- Software Dev 的行业产品合同；
- 用 Spec Bundle 统领 PRD、技术方案、issues、证据、验收、交付和反馈；
- 用 Core Runtime 约束 Agent 执行；
- 用 Software Dev Pack 定义行业对象、页面和连接器。

AgentFlow 当前不做：

- 通用多行业商业平台；
- Pack marketplace；
- 把 Agent Runner 作为产品核心；
- 把 Spec 文档生成器作为产品核心；
- 把 GitHub issues 当作 AgentFlow authority；
- 让 Audit 回到主业务链路。
