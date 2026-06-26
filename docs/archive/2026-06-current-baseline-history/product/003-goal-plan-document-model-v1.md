# 003 - Goal / Plan Document Model V1

创建日期：2026-06-17
执行者：Codex

## Purpose

本文定义 Project 的核心大脑文档。

Goal / Plan 文档是项目方向层，不是执行记录，也不是一次性需求草稿。它们由 Goal Agent 持续维护，用于帮助人类和 Agent 从项目大局判断方向。

## Document Set

每个 Project 至少包含：

```text
GOAL.md
PLAN.md
DECISIONS.md
```

可选扩展：

```text
PROJECT_HEALTH.md
DELIVERY.md
EVIDENCE.md
```

## GOAL.md

GOAL.md 定义项目为什么做、要达成什么。

### Required Sections

```md
# Goal

## Project
- Name:
- Path:
- Owner:

## Outcome
<项目最终要达成的结果。>

## Target User
<谁会使用或受益。>

## Scope
- <scope item>

## Non-goals
- <non-goal item>

## Success Criteria
- [ ] <success criterion>

## Constraints
- <constraint item>

## Current Goal Judgment
<Goal Agent 对当前目标是否清晰、稳定、可执行的判断。>
```

### Rules

- Goal 必须描述最终结果，不描述具体实现步骤。
- Goal 可以被迭代，但必须留下决策记录。
- Goal 不直接生成执行任务。
- Goal 变更需要用户确认。

## PLAN.md

PLAN.md 定义项目如何推进。

### Required Sections

```md
# Plan

## Source Goal
- Goal:
- Last confirmed:

## Stage Plan
1. <stage one>
2. <stage two>
3. <stage three>

## Current Stage
- <current stage>

## Next Recommended Step
- <next step>

## Issue Generation Strategy
- <strategy>

## Validation Strategy
- <strategy>

## Evidence Strategy
- <strategy>

## Risks / Blockers
- <risk or blocker>

## Plan Judgment
<Goal Agent / Spec Agent 对计划是否仍然适配 Goal 的判断。>
```

### Rules

- Plan 是项目路径，不是执行日志。
- Plan 可以包含阶段和任务草案，但不等同于已确认 Issue。
- Plan 变更需要记录原因。
- Plan 不能绕过用户确认直接进入执行。

## DECISIONS.md

DECISIONS.md 记录项目关键判断和用户确认。

### Required Sections

```md
# Decisions

## Decision Log

### YYYY-MM-DD - <Decision Title>
- Context:
- Decision:
- Alternatives:
- Impact:
- Confirmed by:
```

### Decision Types

- Goal confirmation
- Plan confirmation
- Scope change
- Non-goal confirmation
- Risk acceptance
- Pause / resume
- Delivery acceptance
- Next-stage approval

### Rules

- 任何 Goal / Plan 的关键变化都必须记录。
- 用户确认必须可追溯。
- Decision Log 不承载执行细节。

## PROJECT_HEALTH.md

PROJECT_HEALTH.md 是可选文档，用于 Goal Agent 定期回看项目状态。

### Sections

```md
# Project Health

## Goal Alignment
- <alignment judgment>

## Plan Fit
- <fit judgment>

## Execution Drift
- <drift judgment>

## Evidence Gaps
- <gap>

## Risks
- <risk>

## Recommendation
- continue / adjust / pause / deliver
```

## Relationship To Structured Facts

人类可读文档：

```text
GOAL.md
PLAN.md
DECISIONS.md
```

机器可读事实：

```text
SpecProject
SpecIssue
Task Workflow
Evidence
Audit Report
Delivery Report
```

Mapping:

| Human Document | Structured Layer |
| --- | --- |
| GOAL.md | SpecProject.goal / scope / nonGoals / successCriteria |
| PLAN.md | SpecProject.milestones / issue generation strategy |
| DECISIONS.md | project decision log / confirmation records |
| Issue Contract sections | SpecIssue |
| Evidence references | Evidence index |
| Delivery summary | Delivery report |

## Invariants

```text
Goal / Plan 是项目大脑。
Spec / Issue 是执行合同。
Task Workflow 是执行状态机。
Evidence 是结果证明。
Audit 是可信判断。
Delivery 是人类交付。
```
