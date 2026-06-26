# Project Definition 初始化合同

创建日期：2026-05-21
最近压缩：2026-05-22
执行者：Codex

## 定位

本文档是 AgentFlow 当前阶段的 AEP Project Definition。它覆盖 Flow 0 和 Human Planning，但不授权代码实现、不创建 Linear issue、不启动 Agent run。

## Source Goal

[GOAL.md](../../GOAL.md)：闭源、免费、本地优先的 AI 研发执行工具，完成本地需求、执行、验证、审查和 evidence 闭环。

## Flow 0 当前输出

| Flow | 文档落点 | 状态 |
| --- | --- | --- |
| 0.1 Intent | [0.1 questions](0.1-project-initialization-questions.md), [product requirements](../product/product-requirements.md) | established |
| 0.2 Reference / Rules | [0.2 blueprint](0.2-reference-reading-blueprint.md), [ADR](../architecture/architecture-decisions.md) | established |
| 0.3 Map / Archive | [0.3 map](0.3-project-map-and-archive.md), [ROADMAP](../../ROADMAP.md), [construction plan](../planning/construction-plan.md) | established |

## `/goal` 初始化协议

当前阶段把 AEP 第一阶段收束为 `AEP Goal Initialization Protocol v0`：

```text
/goal
-> ProjectGoal
-> ProjectDefinition
-> Bootstrap Artifacts
-> ScopeState
-> GoalReadiness
-> IssueContract
```

新增本地事实源：

- `.agentflow/project-definition.json`
- `.agentflow/scope-state.json`
- `.agentflow/bootstrap/*`

`agentflow goal check` 是第一阶段机械门禁；未通过前，不进入可执行 issue。

## Scope

- 产品目标与非目标。
- AEP / Linear 参考取舍。
- 产品、设计、架构、MVP 规格。
- 第一候选施工包。
- 验证摘要和 append-only 记录。

## Non-Goals

- 不写 Rust / Tauri / React 代码。
- 不创建远程仓库、PR、Linear Project 或 Linear Issue。
- 不实现 SaaS、支付、账号、云同步、移动执行端。
- 不提供海外 AI 代理、支付中转或模型转发。

## 技术定案

| 层 | 定案 |
| --- | --- |
| Core / CLI | Rust |
| Desktop | Tauri 2 |
| UI | React + TypeScript |
| Fact Source | Markdown + JSON |
| Index | SQLite |
| Analytics / Mobile | 后置 |

## 第一候选施工包

`Goal Compiler + Core/CLI Bootstrap v0`

范围：Rust workspace、`agentflow-core`、`agentflow-cli`、`agentflow init --from-goal`、`goal bootstrap`、`goal check`、`goal.json`、`project-definition.json`、`scope-state.json`、Flow 0 文件输出、离线模板版 `agentflow plan`。

状态：`draft / not authorized`

## 验证

文档阶段：`git diff --check` + `rg` 锚点检查。
实现阶段：`cargo fmt --check`、`cargo test`、CLI smoke、fixture parse、evidence report 生成。
