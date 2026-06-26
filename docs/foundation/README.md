# AgentFlow Foundation

更新日期：2026-06-26
执行者：Codex

## Purpose

本目录承载 AgentFlow AI OS Project 的底层通用能力设计。

当前底层框架是：

```text
AI OS Project
= Core Runtime
+ Industry AgentFlow Product
```

展开后：

```text
Core Runtime
= Spec Kernel
+ Ontology Kernel
+ Runtime Kernel
+ Evidence Kernel
+ Decision Kernel
+ Projection Kernel
```

```text
Industry AgentFlow Product
= Domain Pack
+ Surface Pack
+ Connector Pack
```

## Current Foundation Documents

| 文档 | 作用 |
| --- | --- |
| [021-ai-os-project-core-capabilities-v1.md](021-ai-os-project-core-capabilities-v1.md) | 定义 `AI OS Project = Core Runtime + Industry AgentFlow Product`，并明确 6 个 Kernel、12 个通用能力、docs/ 与 .agentflow/ 平面边界和行业 Pack 接入方式 |
| [agentflow-filesystem-workflow-architecture-v1.md](agentflow-filesystem-workflow-architecture-v1.md) | 定义 AgentFlow filesystem-first workflow 架构基准、CodeFlow / DesignFlow 分层和未来 Eve adapter 边界 |

## Historical Foundation Docs

旧 foundation 切片已移入：

```text
docs/archive/2026-06-current-baseline-history/foundation/
```

这些文档保留历史上下文，但不再作为当前 foundation 入口。

## Rules

- Foundation 文档必须引用对应的 `docs/product/` 基线。
- Foundation 文档不能直接混入当前版本实现。
- Foundation 文档确认后，才能进一步拆成 `docs/requirements/**` 下的 confirmed Spec Bundle。
- Foundation 文档不自动授权代码实现。
- Foundation 文档不自动写 `.agentflow/` 运行态数据。
- Foundation 文档不自动生成 GitHub issue。
- Foundation 文档不自动进入 Work Loop。
