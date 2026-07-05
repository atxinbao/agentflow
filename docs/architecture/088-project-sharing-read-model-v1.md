# Project Sharing Read Model v1

更新日期：2026-07-05  
执行者：Codex

## Purpose

`v1.2.1` 的项目共享能力只提供本地、轻量、只读的共享摘要。

它回答：

- 这个项目属于哪个 Product；
- 当前 Goal / Roadmap / Task 状态是什么；
- 最近决策和最近交付是否可读；
- 团队反馈是否已经进入 projection；
- 当 projection 缺失或过期时，UI 应该显示 invalid / deferred，而不是隐藏问题。

## Runtime API

Runtime API 暴露只读查询：

```text
project_sharing_read_model(projectRoot, projectId)
```

版本：

```text
agentflow-project-sharing-read-model.v1
```

## Source Boundary

Project Sharing Read Model 只能读取 projection / read model：

```text
.agentflow/projections/projects/<project-id>.json
.agentflow/projections/workspace-state.json
```

它不能让 Desktop 或 SDK 直接读取这些 authority source：

```text
.agentflow/spec/**
.agentflow/events/**
.agentflow/tasks/**
docs/project/**
```

Runtime 内部可以通过 projection crate 读取 read model，但 UI client 只能消费 sharing view。

## Fields

| Field | Meaning |
| --- | --- |
| `product` | active Product 摘要；缺失时为 `deferred` |
| `goal` | Goal 摘要和状态 |
| `roadmap` | Roadmap / plan 摘要和状态 |
| `tasks` | issue 总数、完成数、当前数、未来数和阻断数 |
| `latestDecision` | 最近 completion / decision 摘要 |
| `latestDelivery` | 最近 delivery / release 摘要 |
| `feedback` | external review / feedback 摘要 |
| `sourceProjectionRefs` | 只读 projection 来源 |
| `blockers` | invalid / deferred 的人类可读原因 |

## State Semantics

| State | Meaning |
| --- | --- |
| `ready` | project projection 和 workspace projection 都可读 |
| `deferred` | project projection 可读，但 product / workspace projection 暂不可用 |
| `invalid` | project projection 缺失或 project projection 自身有 blockers |

## Authority Rule

Project Sharing Read Model 必须始终：

```text
readonly = true
authority = false
projectionBacked = true
```

如果 project projection 缺失：

```text
readonly = true
authority = false
projectionBacked = false
status = invalid
```

## Acceptance

- Runtime API 返回结构化 project sharing read model；
- API plane 暴露 `projection.project-sharing`，且是 readonly projection query；
- Desktop Tauri 暴露只读命令；
- 缺失 project projection 时返回 `invalid` 和 blockers；
- 缺失 workspace / product projection 时返回 `deferred`，不假装 ready；
- UI client 不需要直接读取 authority 文件。
