# 005 - Goal To Project Loop Flow V1

创建日期：2026-06-17
执行者：Codex

## Purpose

本文定义 Goal / Plan 确认后如何进入 Project Loop。

Project Loop 是项目从计划进入执行、审计、交付和回看的循环。它不从原始需求启动，只从确认后的 Goal / Plan 启动。

## Flow

```text
Confirmed Goal
-> Plan Draft
-> Plan Confirmation
-> SpecProject / SpecIssue
-> Work Loop
-> Audit
-> Delivery
-> Goal Recheck
```

## Confirmed Goal

Confirmed Goal 表示：

- 项目目标已确认。
- 项目范围已确认。
- 非目标已确认。
- 成功标准已确认。
- 关键约束已确认。

Confirmed Goal 不等于可执行任务。

## Plan Draft

Spec Agent 根据 Confirmed Goal 生成 Plan Draft。

### Plan Draft Includes

- Stage Plan
- Milestone Drafts
- Issue Contract Drafts
- Validation Strategy
- Evidence Strategy
- Human Confirmation Points
- Risk / Blocker List

### Human Confirmation Points

必须确认：

- Scope change
- High-risk issue
- Goal change
- Plan structure change
- Delivery acceptance

## Plan Confirmation

用户确认 Plan 后，系统才可以生成结构化项目事实。

### Output

- SpecProject
- SpecIssue
- Task workflow inputs
- Initial projection

### Not Output

- 不执行任务。
- 不写代码。
- 不生成 Audit 结论。
- 不生成 Delivery Report。

## Work Loop

Work Loop 只消费确认后的 Issue Contract。

### Work Loop Rules

- Issue 必须属于当前 Project。
- Issue 必须来自确认后的 Plan / Spec。
- Work Agent 不能扩大 Scope。
- Work Agent 必须产出验证结果和 Evidence。

## Audit

Audit Agent 在 Work Result 后运行。

### Audit Checks

- Goal alignment
- Issue Contract compliance
- Boundary compliance
- Evidence completeness
- Validation credibility
- Delivery readiness

Audit 不是修复动作。

## Delivery

Delivery Agent 把完成结果整理成人类可读交付。

### Delivery Includes

- What changed
- What is complete
- What is incomplete
- Evidence index
- Validation summary
- Known limitations
- Next recommendation

Delivery 不等于项目结束。

## Goal Recheck

Delivery 后回到 Goal Agent。

Goal Agent 判断：

- Goal 是否达成？
- Plan 是否继续有效？
- 是否需要下一阶段？
- 是否需要 Scope Change？
- 是否需要暂停？
- 是否可以接受交付？

### Goal Recheck Output

- Continue recommendation
- Adjust plan draft
- New stage proposal
- Pause proposal
- Delivery acceptance proposal

## Project Loop State

Project 可以处于：

```text
intake
goal-draft
plan-draft
confirmed
working
auditing
delivering
goal-recheck
paused
accepted
```

这些是项目级语义，不替代 Issue 状态机。

## Relationship To Task Workflow

Project Loop 是项目编排层。

Task Workflow 是单个 Issue 的执行状态机。

```text
Project Loop 决定现在应该推进哪个阶段。
Task Workflow 决定单个 Issue 如何执行。
```

二者不能混用。

## Boundary

不允许：

- 从 raw requirement 直接进入 Work Loop。
- 从 Plan Draft 直接执行。
- Work Agent 修改 Goal / Plan。
- Audit Agent 执行修复。
- Delivery Agent 判断项目方向。
- Goal Agent 跳过用户确认写入可执行任务。

允许：

- Goal Agent 生成下一阶段建议。
- Spec Agent 生成可确认的 Plan Draft。
- Work Agent 执行确认后的 Issue。
- Audit Agent 判断结果可信度。
- Delivery Agent 整理交付。
