# AgentFlow Core

更新日期：2026-06-26
执行者：Codex

## Purpose

本目录承载 AgentFlow AI OS Project 的底层 Core 能力和稳定架构合同。

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

## Current Core Documents

| 文档 | 作用 |
| --- | --- |
| [021-ai-os-project-core-capabilities-v1.md](021-ai-os-project-core-capabilities-v1.md) | 定义 `AI OS Project = Core Runtime + Industry AgentFlow Product`，并明确 6 个 Kernel、12 个通用能力、docs/ 与 .agentflow/ 平面边界和行业 Pack 接入方式 |
| [agentflow-filesystem-workflow-architecture-v1.md](agentflow-filesystem-workflow-architecture-v1.md) | 定义 AgentFlow filesystem-first workflow 架构基准、CodeFlow / DesignFlow 分层和未来 Eve adapter 边界 |
| [architecture/README.md](architecture/README.md) | v1.x 稳定架构合同和运行时边界 |

## Historical Core Docs

旧 foundation 切片已移入：

```text
docs/archive/2026-06-current-baseline-history/foundation/
```

这些文档保留历史上下文，但不再作为当前 Core 入口。

## Rules

- Core 文档必须引用对应的 `docs/product/` 基线。
- Core 文档不能直接混入当前版本实现。
- Core 文档确认后，才能进一步拆成 `docs/requirements/**` 下的 confirmed Spec Bundle。
- Core 文档不自动授权代码实现。
- Core 文档不自动写 `.agentflow/` 运行态数据。
- Core 文档不自动生成 GitHub issue。
- Core 文档不自动进入 Work Loop。
