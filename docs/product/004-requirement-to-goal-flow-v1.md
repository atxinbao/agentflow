# 004 - Requirement To Goal Flow V1

创建日期：2026-06-17
执行者：Codex

## Purpose

本文定义用户输入如何被转成项目 Goal。

用户输入的是需求，不是任务。AgentFlow 必须先判断需求意图，再形成 Goal Draft，而不是直接生成执行任务。

## Flow

```text
User Requirement
-> Requirement Intake
-> Requirement Classification
-> Goal Draft
-> Clarification
-> Goal Confirmation
```

## User Requirement

用户可能输入：

- 我要做一个电商 APP。
- 我要做一个前端设计。
- 我要修复一个 BUG。
- 我要重构代码。
- 我要审计当前交付。
- 我要理解一个项目。

这些输入都先被视为 Requirement。

## Requirement Intake

Spec Agent 接收需求并提取：

- 原始文本
- 项目名称候选
- 项目目录候选
- 用户期望结果
- 显性范围
- 显性非目标
- 约束
- 不确定点

## Requirement Classification

Spec Agent 判断需求类型：

```text
product
design
technical
repair
audit
understanding
mixed
```

Classification 只决定进入哪种 Goal Draft 模板，不决定直接执行。

## Goal Draft

Goal Agent 生成 Goal Draft。

### Goal Draft Fields

```text
goalId
sourceRequirement
intentType
projectTitle
outcome
targetUser
expectedDeliverables
scope
nonGoals
successCriteria
constraints
openQuestions
confidence
status
```

### Status

```text
needs-clarification
ready-for-review
confirmed
rejected
```

## Clarification

如果 Goal 不清楚，Goal Agent 只问影响项目方向的问题。

Allowed:

- 目标用户是谁？
- 最终交付是什么？
- 是否需要设计、代码、测试、部署？
- 哪些范围明确不做？
- 成功标准是什么？

Not Allowed:

- 不问低层实现细节。
- 不提前生成执行任务。
- 不假设用户已经确认范围。

## Goal Confirmation

Goal Draft 需要用户确认。

用户可以：

- confirm
- revise
- reject
- split project

确认后才能进入 Plan Draft。

## Examples

### Full Product

Input:

```text
我要做一个电商 APP。
```

Goal Draft:

```text
构建一个可用的电商应用，覆盖商品浏览、购物车、订单和基础管理能力。
```

Plan likely needs:

- Product
- Design
- Frontend
- Backend
- Test
- Delivery

### Design-only

Input:

```text
我只想做前端设计稿。
```

Goal Draft:

```text
为指定产品范围产出前端页面设计方案，不进入代码实现。
```

Plan likely needs:

- Design research
- UI structure
- Interaction states
- Design handoff

### Repair

Input:

```text
我要修复登录页按钮错位。
```

Goal Draft:

```text
修复登录页按钮错位问题，并保持现有交互和视觉风格不变。
```

Plan likely needs:

- Reproduce
- Fix
- Verify
- Delivery summary

## Boundary

```text
Requirement 不是 Goal。
Goal 不是 Plan。
Plan 不是 Issue。
Issue 不是执行授权，除非用户确认且进入 Project Loop。
```

## Output Gate

Requirement To Goal Flow 的最终输出是：

- GOAL.md Draft
- Clarification Questions
- Goal Confirmation Request

它不输出：

- 可执行 Issue
- 运行状态
- 代码改动
- Audit 结论
- Delivery Report
