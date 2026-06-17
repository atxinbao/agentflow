# 002 - Project Lifecycle V1

创建日期：2026-06-17
执行者：Codex

## Purpose

本文定义 AgentFlow 的 Project 生命周期。

Project 是 AgentFlow 的顶层用户入口。所有需求、计划、执行、审计和交付都围绕 Project 展开。

## Lifecycle

```text
1. Intake
2. Goal Draft
3. Plan Draft
4. Confirmed Project
5. Work Loop
6. Audit / Delivery
7. Goal Recheck
```

## 1. Intake

用户添加项目并输入最小信息。

### Minimum Input

- Project name
- Project path / repository
- Initial requirement

### Behavior

- 系统不直接生成可执行任务。
- Spec Agent 先接收输入。
- Goal Agent 判断需求是否足够形成 Goal Draft。

### Output

- Requirement Intake Result
- Clarification Questions, if needed
- Goal Draft Candidate

## 2. Goal Draft

Goal Agent 将用户输入转成项目目标草案。

### Goal Draft Includes

- 项目目标
- 用户 / 受众
- 预期交付结果
- 范围
- 非目标
- 成功标准
- 约束
- 未决问题

### Gate

Goal Draft 需要用户确认或补充。

### Output

- GOAL.md Draft
- Goal Confirmation Request

## 3. Plan Draft

Spec Agent 根据确认方向生成计划草案。

### Plan Draft Includes

- 阶段拆解
- Milestone 草案
- Issue Contract 草案
- 验证策略
- 证据策略
- 人工确认点
- 风险和阻塞

### Gate

Plan Draft 需要用户确认后才能写入结构化项目事实。

### Output

- PLAN.md Draft
- DECISIONS.md Draft Entry
- SpecProject Draft
- SpecIssue Drafts

## 4. Confirmed Project

用户确认 Goal / Plan / Scope 后，Project 进入可执行状态。

### Confirmed Project Means

- 项目目标已确认。
- 项目范围已确认。
- 计划已确认。
- 至少一个可执行 Issue Contract 已生成。
- 执行边界已明确。

### Output

- GOAL.md
- PLAN.md
- DECISIONS.md
- SpecProject
- SpecIssue

## 5. Work Loop

Work Agent 按确认后的 Issue Contract 执行。

### Rules

- 一次只执行一个被确认的 Issue。
- 不从原始需求直接执行。
- 不绕过 Issue Contract。
- 不扩大范围。
- 不隐藏验证失败。

### Output

- Work Result
- Validation Output
- Evidence
- Handoff Summary

## 6. Audit / Delivery

Audit Agent 和 Delivery Agent 分别处理可信判断和人类交付。

### Audit

Audit Agent 检查：

- 是否满足 Issue Contract。
- 是否满足项目 Goal。
- Evidence 是否完整。
- 是否存在越界。

### Delivery

Delivery Agent 整理：

- 完成内容
- 验证结果
- Evidence 索引
- 已知限制
- 后续建议

### Output

- Audit Report
- DELIVERY.md
- Next Stage Recommendation

## 7. Goal Recheck

Delivery 之后，项目回到 Goal Agent。

### Goal Agent Rechecks

- 项目目标是否已经达成？
- 是否需要继续下一阶段？
- 是否需要调整 Goal？
- 是否需要调整 Plan？
- 是否存在新的风险或范围变化？

### Possible Outcomes

- Continue current plan
- Adjust plan
- Request scope confirmation
- Create next-stage draft
- Pause project
- Mark delivery accepted

## Lifecycle Invariant

```text
原始需求不能直接进入执行。
Goal / Plan 必须先形成草案。
用户确认后才能进入 Project Loop。
Delivery 后必须回到 Goal Recheck。
```

## Project Size

### Large Project

例如电商 APP：

- 多个 Milestone
- 多个专业能力
- 多轮 Goal Recheck
- 多阶段 Delivery

### Small Project

例如一个按钮 BUG：

- 一个 Goal
- 一个 Plan
- 一个 Issue
- 一次 Audit
- 一个 Delivery Summary

生命周期不变，只是深度不同。
