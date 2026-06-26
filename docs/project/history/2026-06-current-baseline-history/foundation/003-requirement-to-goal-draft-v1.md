# 003 - Requirement To Goal Draft V1

创建日期：2026-06-17
执行者：Codex

## Source Foundation

本文从以下文档派生：

- [../product/004-requirement-to-goal-flow-v1.md](../product/004-requirement-to-goal-flow-v1.md)
- [001-goal-agent-project-brain-v1.md](001-goal-agent-project-brain-v1.md)
- [002-project-brain-document-store-v1.md](002-project-brain-document-store-v1.md)

## Purpose

本文定义用户需求如何生成 Goal Draft。

Requirement 是用户输入。Goal Draft 是 Goal Agent 对项目目标的结构化草稿。Goal Draft 不是可执行任务，也不是用户已确认事实。

## Input

用户添加 Project 时可以输入：

```text
projectTitle
projectPath
rawRequirement
optionalContext
```

`rawRequirement` 可以是：

- 产品需求
- 设计需求
- 技术需求
- 修复需求
- 审计需求
- 理解项目需求
- 混合需求

## Intake Result

建议形成：

```text
RequirementIntakeResult
```

字段：

```text
requirementId
projectId
rawText
detectedIntent
detectedScope
detectedNonGoals
detectedDeliverables
detectedConstraints
missingInformation
clarificationQuestions
confidence
nextAction
```

## Intent Types

```text
product
design
technical
repair
audit
understanding
mixed
unknown
```

Intent 只决定 Goal Draft 模板，不授权执行。

## Goal Draft

建议形成：

```text
GoalDraft
```

字段：

```text
goalDraftId
projectId
sourceRequirementId
title
intentType
outcome
targetUser
expectedDeliverables
scope
nonGoals
successCriteria
constraints
openQuestions
riskHints
confidence
status
```

## Goal Draft Status

```text
needs-clarification
ready-for-review
confirmed
rejected
split-required
```

## Clarification Rule

只允许问影响项目方向的问题。

允许：

- 这个项目最终要交付什么？
- 目标用户是谁？
- 哪些范围明确不做？
- 是否只需要设计，不需要实现？
- 是否需要交付代码、文档、测试、部署？
- 成功标准是什么？

不允许：

- 不提前询问低层实现细节。
- 不把澄清问题变成任务清单。
- 不从澄清阶段生成 Issue。

## Preview Output

Requirement To Goal Draft 的输出是：

```text
Goal Draft Preview
Clarification Questions
Recommended Next Action
```

推荐 nextAction：

```text
ask-human
review-goal
split-project
reject
```

## Confirmation Boundary

用户确认 Goal Draft 之前：

- 不写 GOAL.md。
- 不写 PLAN.md。
- 不写 DECISIONS.md。
- 不写 `.agentflow/` runtime data。
- 不生成 Issue。
- 不进入 Work Loop。

用户确认后，才可以进入：

```text
Goal Draft
-> GOAL.md write proposal
-> Plan Draft
```

## Acceptance Criteria

- [ ] Requirement 与 Goal Draft 区分明确。
- [ ] Intent 类型明确。
- [ ] GoalDraft 字段明确。
- [ ] 澄清规则明确。
- [ ] Preview 输出明确。
- [ ] 用户确认前不写入。
- [ ] 不生成 Issue。
- [ ] 不进入 Work Loop。
