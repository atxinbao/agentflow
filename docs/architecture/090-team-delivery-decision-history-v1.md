# Team Delivery / Decision History v1

更新日期：2026-07-05  
执行者：Codex

## Purpose

`v1.2.1` 的团队工作流需要一个只读 history view，让团队成员能看懂：

- 哪些 decision 被接受、拒绝或仍待处理；
- 每次 decision 的原因、失败理由和后续修复建议；
- 哪些 delivery 已经形成公开记录；
- 反馈如何回到下一轮 Spec evolution；
- Audit 只是 sidecar，不是默认阻断链。

## Runtime API

Runtime API 暴露只读查询：

```text
team_delivery_decision_history_view(projectRoot, projectId)
```

版本：

```text
agentflow-team-delivery-decision-history.v1
```

API Plane:

```text
projection.team-delivery-decision-history
```

## Source Boundary

该 view 只能读取 projection / read model：

```text
.agentflow/projections/projects/<project-id>.json
.agentflow/projections/tasks/<issue-id>.json
```

它不能直接读取或写入 authority source：

```text
.agentflow/spec/**
.agentflow/events/**
.agentflow/tasks/**
.agentflow/runtime/**
docs/project/**
```

## Entries

| Entry | Meaning |
| --- | --- |
| `decision` | acceptance / completion decision，包含 outcome、reason、remediation |
| `delivery` | public delivery record，包含 PR、merge commit、changelog、release notes |
| `audit-sidecar` | 可选审计侧车记录，不参与默认 delivery 阻断 |

## Feedback Hook

History view 必须暴露 feedback route：

```text
feedback-loop/spec-evolution
```

反馈不能直接改 Spec authority。它只能作为下一轮 Spec Loop 的输入，并且需要人类确认。

## State Semantics

| State | Meaning |
| --- | --- |
| `ready` | project projection 和所有 task projections 均可读 |
| `deferred` | project projection 可读，但部分 task projection 缺失 |
| `invalid` | project projection 缺失或 project projection 自身有 blockers |

## Authority Rule

Team Delivery / Decision History View 必须始终：

```text
readonly = true
authority = false
```

存在 project projection 时：

```text
projectionBacked = true
```

缺失 project projection 时：

```text
projectionBacked = false
status = invalid
```

## Acceptance

- Runtime API 返回结构化 team delivery / decision history view；
- API plane 暴露 `projection.team-delivery-decision-history`；
- Desktop Tauri 暴露只读命令；
- view 能解释 accepted / rejected / delivered 以及原因；
- view 暴露 feedback route，能回到 Feedback Loop / Spec evolution；
- Audit sidecar 保持 optional，不能默认阻断 delivery chain；
- 缺失 task projection 时返回 `deferred` 和 blockers，不假装 ready。
