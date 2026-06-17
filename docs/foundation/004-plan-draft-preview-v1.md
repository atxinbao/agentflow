# 004 - Plan Draft Preview V1

创建日期：2026-06-17
执行者：Codex

## Source Foundation

本文从以下文档派生：

- [../product/005-goal-to-project-loop-flow-v1.md](../product/005-goal-to-project-loop-flow-v1.md)
- [003-requirement-to-goal-draft-v1.md](003-requirement-to-goal-draft-v1.md)

## Purpose

本文定义 Confirmed Goal 如何生成 Plan Draft Preview。

Plan Draft 是项目路径草案，不是执行授权。它负责把目标拆成阶段、策略和候选 Issue Contract，但用户确认前不进入 Project Loop。

## Input

Plan Draft 只能从 Confirmed Goal 生成。

必需输入：

```text
projectId
confirmedGoal
scope
nonGoals
successCriteria
constraints
```

可选输入：

```text
existingPlan
projectFiles
architectureHints
designReferences
technicalConstraints
```

## Plan Draft

建议形成：

```text
PlanDraft
```

字段：

```text
planDraftId
projectId
sourceGoalId
planType
stagePlan
milestoneDrafts
issueContractDrafts
validationStrategy
evidenceStrategy
humanConfirmationPoints
riskList
blockers
nextRecommendedAction
status
```

## Plan Type

```text
product
design
technical
repair
audit
mixed
```

## Stage Plan

Stage Plan 描述项目如何推进。

示例：

```text
1. Goal confirmation
2. Product plan
3. Design plan
4. Technical plan
5. Work execution
6. Audit
7. Delivery
8. Goal recheck
```

项目大小不同，阶段数量可以不同，但必须保持 Goal -> Plan -> Work -> Audit -> Delivery -> Recheck 的方向。

## Milestone Draft

Milestone Draft 是阶段草案。

字段：

```text
milestoneId
title
goal
dependsOn
expectedOutputs
validationNeed
evidenceNeed
```

Milestone Draft 不执行，不承载任务状态。

## Issue Contract Draft

Issue Contract Draft 是候选执行合同。

字段：

```text
issueDraftId
title
goal
scope
nonGoals
dependencies
acceptanceCriteria
validationCommands
evidenceRequirements
boundary
riskLevel
suggestedAgentRole
```

Issue Contract Draft 不是 SpecIssue。用户确认前不能进入 Work Loop。

## Human Confirmation Points

必须标出：

- scope change
- high-risk issue
- project split
- plan structure change
- external dependency
- delivery acceptance

## Preview Output

Plan Draft Preview 输出：

```text
Plan Summary
Milestone Drafts
Issue Contract Drafts
Risks / Blockers
Human Confirmation Points
Next Recommended Action
```

## Confirmation Boundary

用户确认前：

- 不写 PLAN.md。
- 不写 SpecProject。
- 不写 SpecIssue。
- 不生成 Task Workflow。
- 不进入 Work Loop。

用户确认后：

```text
Plan Draft
-> PLAN.md write proposal
-> DECISIONS.md entry
-> SpecProject / SpecIssue materialization proposal
```

## Acceptance Criteria

- [ ] Plan Draft 只能从 Confirmed Goal 生成。
- [ ] PlanDraft 字段明确。
- [ ] Milestone Draft 与 Issue Contract Draft 区分明确。
- [ ] Issue Contract Draft 不等同于可执行 Issue。
- [ ] 用户确认前不写入。
- [ ] 不进入 Work Loop。
- [ ] Human Confirmation Points 明确。
