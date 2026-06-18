# Architecture Docs

创建日期：2026-06-18
执行者：Codex

## Purpose

`docs/architecture/` 负责定义 AgentFlow 的技术底座。

这里不直接写当前迭代需求，也不直接生成实现任务。
它回答的是：

- Project 的底层运行时如何组织？
- Agent、Workflow、Event、Projection 之间是什么关系？
- 哪些模块是长期地基，哪些模块是可替换层？
- Desktop / CLI / 外部 Provider 应该读取或写入什么？

## 文档范围

| 文档 | 作用 |
| --- | --- |
| [001-project-operating-system-v1.md](001-project-operating-system-v1.md) | 定义 AgentFlow 的总蓝图、authority、永久层与可替换层 |
| [002-agent-capability-matrix-v1.md](002-agent-capability-matrix-v1.md) | 定义 Goal / Spec / Work / Audit / Delivery 的角色、职责、技能与 handoff |
| [003-workflow-schema-v1.md](003-workflow-schema-v1.md) | 定义 Project / Work / Audit / Delivery 四类流程的统一 schema |
| [004-event-and-projection-model-v1.md](004-event-and-projection-model-v1.md) | 定义事件、状态、投影和 UI 读模型 |
| [005-public-delivery-standard-v1.md](005-public-delivery-standard-v1.md) | 定义任务级与版本级公开交付模板、边界和 projection 输出 |
| [current-module-boundaries.md](current-module-boundaries.md) | 当前 crates 和目录边界的事实快照 |
| [mcp-provider-adapter.md](mcp-provider-adapter.md) | 外部 provider / MCP 适配层边界 |

## 默认阅读顺序

1. [001-project-operating-system-v1.md](001-project-operating-system-v1.md)
2. [002-agent-capability-matrix-v1.md](002-agent-capability-matrix-v1.md)
3. [003-workflow-schema-v1.md](003-workflow-schema-v1.md)
4. [004-event-and-projection-model-v1.md](004-event-and-projection-model-v1.md)
5. [005-public-delivery-standard-v1.md](005-public-delivery-standard-v1.md)
6. [current-module-boundaries.md](current-module-boundaries.md)
7. [mcp-provider-adapter.md](mcp-provider-adapter.md)

## 规则

- `docs/architecture/` 只定义技术底座，不直接等同于当前迭代需求。
- `docs/architecture/` 不能绕过 `docs/requirements/` 直接授权开发。
- `docs/product/` 负责说明产品方向。
- `docs/foundation/` 负责说明领域模型和基础能力切片。
- `docs/requirements/` 负责当前版本的可执行需求。
- 当 `architecture`、`foundation` 与 `requirements` 有冲突时，必须先回到产品与架构边界重新确认，再继续开发。
