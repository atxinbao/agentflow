# 007 - Project Loop Entry From Confirmed Plan V1

创建日期：2026-06-17
执行者：Codex

## Source Foundation

本文从以下文档派生：

- [../product/005-goal-to-project-loop-flow-v1.md](../product/005-goal-to-project-loop-flow-v1.md)
- [004-plan-draft-preview-v1.md](004-plan-draft-preview-v1.md)
- [006-project-brain-confirmation-gate-v1.md](006-project-brain-confirmation-gate-v1.md)

## Purpose

本文定义 Confirmed Goal / Confirmed Plan 如何进入 Project Loop。

Project Loop Entry 是从项目大脑层进入结构化项目循环的入口。它只负责把已确认的 Goal / Plan materialize 成项目循环可读取的结构化事实，不负责执行任务。

## Position In Flow

```text
Requirement
-> Goal Draft Preview
-> Goal Confirmation
-> Plan Draft Preview
-> Plan Confirmation
-> Project Loop Entry
-> Project Loop
```

Project Loop Entry 之后，Project Loop 可以调度阶段、Issue、审计和交付；但本文不定义 Work Loop 如何执行单个 Issue。

## Entry Conditions

进入 Project Loop 前必须满足：

- Confirmed Goal exists.
- Confirmed Plan exists.
- DECISIONS.md 已记录 Goal confirmation。
- DECISIONS.md 已记录 Plan confirmation。
- Scope / Non-goals / Success Criteria 已明确。
- Plan Draft 中的 Milestone Draft 已确认。
- Plan Draft 中的 Issue Contract Draft 已确认或标记为 deferred。
- Human Confirmation Points 已记录。
- 没有阻塞性的 open questions。

## Inputs

Project Loop Entry 读取：

```text
docs/projects/<project-id>/GOAL.md
docs/projects/<project-id>/PLAN.md
docs/projects/<project-id>/DECISIONS.md
```

可选读取：

```text
docs/projects/<project-id>/PROJECT_HEALTH.md
docs/projects/<project-id>/EVIDENCE.md
docs/projects/<project-id>/DELIVERY.md
```

## Materialization Proposal

Project Loop Entry 先生成 materialization proposal，而不是直接写入执行事实。

建议对象：

```text
ProjectLoopEntryProposal
```

字段：

```text
proposalId
projectId
sourceGoalPath
sourcePlanPath
sourceDecisionsPath
proposedSpecProject
proposedSpecIssues
proposedMilestones
deferredIssueDrafts
humanConfirmationPoints
blockedReasons
nextAction
status
```

## Proposal Status

```text
draft
ready-for-review
needs-revision
confirmed
rejected
blocked
```

## Proposed Structured Facts

确认后可写入的结构化事实包括：

```text
.agentflow/spec/projects/<project-id>.json
.agentflow/spec/issues/<issue-id>.json
```

如果未来 Project Loop 需要 projection seed，可由 spec / events 派生，不在本文定义。

## SpecProject Mapping

GOAL.md / PLAN.md 映射到 SpecProject：

| Source | SpecProject Field |
| --- | --- |
| Project ID | id |
| Project title | title |
| Goal outcome | goal |
| Scope | scope |
| Non-goals | nonGoals |
| Success criteria | successCriteria |
| Stage plan | milestones |
| Current stage | activeMilestone |
| Human confirmation points | gates |

## SpecIssue Mapping

Issue Contract Draft 映射到 SpecIssue：

| Source | SpecIssue Field |
| --- | --- |
| issueDraftId | id |
| title | title |
| goal | goal |
| scope | scope |
| nonGoals | nonGoals |
| dependencies | dependencies |
| acceptanceCriteria | acceptanceCriteria |
| validationCommands | validationCommands |
| evidenceRequirements | evidenceRequirements |
| boundary | boundary |
| riskLevel | riskLevel |
| suggestedAgentRole | requiredAgentRole |

## Deferred Issues

Deferred Issue Draft 不进入初始 Project Loop。

Deferred 原因必须记录：

- scope not confirmed
- dependency missing
- high risk needs human approval
- future stage
- blocked by external input

Deferred issue 不能进入 Work Loop。

## Confirmation Gate

Project Loop Entry 写入前必须经过确认。

确认后允许：

- 写 SpecProject。
- 写 SpecIssue。
- 写 DECISIONS.md 的 Project Loop entry confirmation。

确认后仍不允许：

- 不执行 Work Loop。
- 不自动选择 eligible Issue。
- 不自动启动 Agent。
- 不生成 Audit Report。
- 不生成 Delivery Report。

## Blocked Conditions

Project Loop Entry 必须 blocked，如果：

- GOAL.md 缺失。
- PLAN.md 缺失。
- DECISIONS.md 缺失。
- Goal 未确认。
- Plan 未确认。
- Scope 和 Non-goals 冲突。
- Issue Contract Draft 缺少验收标准。
- Issue Contract Draft 缺少边界。
- Human Confirmation Point 未处理。

## Output

V1 输出：

```text
ProjectLoopEntryProposal
ProjectLoopEntryStatus
BlockedReasons
NextRecommendedAction
```

确认后输出：

```text
SpecProject
SpecIssue
Decision entry
```

## Not In Scope

本文不定义：

- Work Loop 执行。
- Task Workflow YAML runtime。
- Queue preflight。
- Lease。
- Checkpoint。
- Patch。
- Audit execution。
- Delivery generation。
- Desktop write UI。
- Model call。
- Remote PR / GitHub / Linear。

## Acceptance Criteria

- [ ] Entry conditions 明确。
- [ ] ProjectLoopEntryProposal 字段明确。
- [ ] GOAL.md / PLAN.md 到 SpecProject 的映射明确。
- [ ] Issue Contract Draft 到 SpecIssue 的映射明确。
- [ ] Deferred Issue 规则明确。
- [ ] 确认前不写结构化事实。
- [ ] 确认后只允许写 SpecProject / SpecIssue / Decision entry。
- [ ] 不进入 Work Loop。
- [ ] 不自动启动 Agent。
- [ ] Blocked conditions 明确。
