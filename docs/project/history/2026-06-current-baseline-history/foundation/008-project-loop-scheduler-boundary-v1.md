# 008 - Project Loop Scheduler Boundary V1

创建日期：2026-06-17
执行者：Codex

## Source Foundation

本文从以下文档派生：

- [../product/005-goal-to-project-loop-flow-v1.md](../product/005-goal-to-project-loop-flow-v1.md)
- [007-project-loop-entry-from-confirmed-plan-v1.md](007-project-loop-entry-from-confirmed-plan-v1.md)

## Purpose

本文定义 Project Loop Scheduler 的边界。

Project Loop Scheduler 负责从 Project 视角读取已确认的结构化事实，判断项目当前应该进入哪一步，并输出下一步建议。它不执行 Issue，不做 Queue Preflight，不持有 Lease，不调用模型。

## Position In Flow

```text
Project Loop Entry
-> SpecProject / SpecIssue
-> Project Loop Scheduler
-> Scheduler Snapshot
-> Next Recommended Action
```

Scheduler 是项目级只读决策层。

它回答：

- 当前 Project 是否可以继续推进？
- 当前 Project 是否缺 Goal / Plan / Spec？
- 当前 Project 应该先处理哪个阶段？
- 是否存在可进入 Issue Preflight 的候选？
- 是否应该进入 Audit / Delivery / Goal Recheck？
- 是否存在必须等待用户确认的问题？

## Inputs

Project Loop Scheduler 读取：

```text
.agentflow/spec/projects/<project-id>.json
.agentflow/spec/issues/<issue-id>.json
docs/projects/<project-id>/GOAL.md
docs/projects/<project-id>/PLAN.md
docs/projects/<project-id>/DECISIONS.md
```

可选读取：

```text
.agentflow/events/**
.agentflow/projections/**
docs/projects/<project-id>/PROJECT_HEALTH.md
docs/projects/<project-id>/EVIDENCE.md
docs/projects/<project-id>/DELIVERY.md
```

## Scheduler Snapshot

建议对象：

```text
ProjectLoopSchedulerSnapshot
```

字段：

```text
projectId
projectTitle
projectStatus
goalStatus
planStatus
activeStage
stageStatuses
issueSummary
candidateIssue
blockedReasons
humanConfirmationNeeded
auditNeeded
deliveryNeeded
goalRecheckNeeded
nextRecommendedAction
readonly
```

## Stage Status

建议对象：

```text
ProjectStageStatus
```

字段：

```text
stageId
title
status
dependsOn
issueIds
doneIssueCount
totalIssueCount
blockedIssueCount
evidenceStatus
nextAction
```

### Stage Status Values

```text
not-started
ready
working
blocked
needs-audit
needs-delivery
done
deferred
```

## Issue Summary

建议对象：

```text
ProjectIssueSummary
```

字段：

```text
total
backlog
todo
inProgress
inReview
done
blocked
deferred
cancelled
highRisk
missingEvidence
```

## Candidate Issue

Scheduler 可以推荐候选 Issue，但不能把它提升为可执行状态。

候选条件：

- Issue 属于当前 Project。
- Issue 属于当前 active stage，或被明确标记为 project-level。
- Issue 没有未完成 dependencies。
- Issue Contract 字段完整。
- Issue 没有 unresolved human confirmation point。
- Issue 不处于 done / cancelled / deferred。

候选输出：

```text
candidateIssue
candidateReason
requiredNextGate
```

`requiredNextGate` 可以是：

```text
issue-preflight
human-confirmation
audit
delivery
goal-recheck
blocked
```

## Next Recommended Action

允许输出：

```text
initialize-project-brain
confirm-goal
confirm-plan
materialize-project-loop-entry
run-issue-preflight
wait-human-confirmation
request-audit
prepare-delivery
run-goal-recheck
blocked
```

不允许输出成直接执行命令。

## Human Confirmation Needed

Scheduler 必须标记需要人工确认，如果：

- Goal / Plan 未确认。
- Scope change 待确认。
- high-risk issue 待确认。
- Delivery acceptance 待确认。
- Project split 待确认。
- Deferred issue 需要恢复。

## Audit Needed

Scheduler 可以判断需要 Audit，如果：

- 当前阶段全部 required issues 完成。
- Evidence 已存在但未审计。
- Delivery 前需要可信检查。
- 用户显式请求 audit。

Scheduler 不生成 Audit Report。

## Delivery Needed

Scheduler 可以判断需要 Delivery，如果：

- Audit 已通过。
- 当前阶段有可交付结果。
- 用户显式请求交付摘要。

Scheduler 不生成 Delivery Report。

## Goal Recheck Needed

Scheduler 可以判断需要 Goal Recheck，如果：

- Delivery 已完成。
- Project 发生 scope change。
- Plan 与实际执行出现偏移。
- 用户新增需求影响项目方向。
- Audit 发现目标或范围不一致。

Scheduler 不修改 GOAL.md / PLAN.md。

## Readonly Boundary

Project Loop Scheduler V1 只读。

允许：

- 读取 Goal / Plan / Decisions。
- 读取 SpecProject / SpecIssue。
- 读取事件和 projection。
- 输出 Scheduler Snapshot。
- 输出下一步建议。

不允许：

- 不写 `.agentflow/`。
- 不写 `docs/projects/**`。
- 不修改 Issue 状态。
- 不做 Queue Preflight。
- 不 acquire Lease。
- 不启动 Work Agent。
- 不生成 Audit Report。
- 不生成 Delivery Report。
- 不调用模型。

## Relationship To Work Loop

Project Loop Scheduler 只决定项目下一步应该走向哪个 gate。

Work Loop 只执行已通过后续 gate 的单个 Issue。

```text
Scheduler 推荐 issue-preflight。
Issue Preflight 判断是否可执行。
Work Loop 执行单个 confirmed issue。
```

Scheduler 不替代 Issue Preflight。

## Blocked Conditions

Scheduler 必须返回 blocked，如果：

- SpecProject 缺失。
- SpecIssue 缺失或与 Project 不匹配。
- Goal / Plan confirmation 缺失。
- 当前阶段 dependencies 未满足。
- Issue Contract 不完整。
- Human confirmation point 未处理。
- 数据读取失败或结构损坏。

## Acceptance Criteria

- [ ] Scheduler 定位为 Project 级只读决策层。
- [ ] ProjectLoopSchedulerSnapshot 字段明确。
- [ ] Stage status 明确。
- [ ] Issue summary 明确。
- [ ] Candidate Issue 只是推荐，不是执行授权。
- [ ] Next Recommended Action 不等同于执行命令。
- [ ] Human confirmation / Audit / Delivery / Goal Recheck 判断边界明确。
- [ ] Scheduler 不写 `.agentflow/`。
- [ ] Scheduler 不写 `docs/projects/**`。
- [ ] Scheduler 不做 Queue Preflight。
- [ ] Scheduler 不启动 Work Loop。
