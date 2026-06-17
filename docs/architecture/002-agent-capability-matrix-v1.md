# 002 - Agent Capability Matrix V1

创建日期：2026-06-18
执行者：Codex

## Purpose

本文定义 AgentFlow 的角色、职责、技能和 handoff 基线。

目标是让 Workflow Runtime 能明确回答：

- 当前阶段应该由哪个角色接手？
- 这个角色能做什么、不能做什么？
- 它需要哪些技能和上下文？
- 什么时候是 handoff，什么时候只是工具调用？

## 关键原则

1. Role 是角色，不是一次具体会话。
2. Skill 是能力包，不是 provider。
3. Tool 是技能执行时可调用的工具。
4. Session 是某次执行实例。
5. Provider 是外部执行载体，不等于角色本身。

## Role Overview

| 角色 | 作用 |
| --- | --- |
| Goal Agent | 维护项目方向、阶段判断和目标回看 |
| Spec Agent | 把 Goal / Plan 转成结构化合同 |
| Work Agent | 执行单条 issue |
| Audit Agent | 判断执行结果是否可信 |
| Delivery Agent | 整理对外可接收交付 |
| Specialist | 为上层角色提供垂直能力补充 |

## 角色职责表

| 角色 | 核心职责 | 主要输入 | 主要输出 | 禁止事项 |
| --- | --- | --- | --- | --- |
| Goal Agent | 项目方向、阶段判断、Goal Recheck | Requirement、GOAL/PLAN/DECISIONS、Project 状态、Audit / Delivery 摘要 | Goal Draft、Goal Update、Project Health、Next Decision Proposal | 不拆 issue、不执行任务、不做审计通过结论 |
| Spec Agent | 规划拆解、合同生成、执行边界定义 | Goal / Plan、用户输入、约束、项目上下文 | SpecProject、SpecIssue、workflowRef、acceptance / evidence boundary | 不执行代码、不直接进入 Work Flow |
| Work Agent | 执行单任务、验证、留证据 | SpecIssue、Panel Context、Validation Plan、Allowed Paths | 改动结果、run facts、validation output、evidence、handoff summary | 不改 Goal / Plan、不扩大 scope、不做最终审计 |
| Audit Agent | 审计结果可信度、边界和证据检查 | Goal、SpecIssue、Work Result、Evidence、Validation Output | Audit Result、Findings、Evidence Gap、Repair Recommendation | 不修代码、不执行任务、不改 Goal / Plan |
| Delivery Agent | 交付整理、对外摘要、变更汇总 | Goal / Plan、Completed Issues、Audit Result、Evidence Index | Delivery Summary、CHANGELOG Draft、Release Notes Draft、Next Recommendation | 不决定项目方向、不替代 Audit 做通过判断 |
| Specialist | 提供前端、后端、测试、设计、文档等局部能力 | 上层角色 handoff 的局部任务 | 局部执行结果 | 不独立驱动 Project，不拥有阶段 authority |

## Runtime 兼容说明

运行时 authority 使用：

- `goal-agent`
- `spec-agent`
- `work-agent`
- `audit-agent`
- `delivery-agent`
- `specialist`
- `system`

当前 provider-facing 执行兼容别名：

- `build-agent` = `work-agent`

规则：

- Workflow / Runtime / Projection 必须使用运行时角色名；
- provider session 可以继续使用 `build-agent` 作为执行入口；
- dispatcher 必须把 `build-agent` 解析成运行时角色 `work-agent`；
- provider 名称不能反向定义 workflow authority。

## Skill Taxonomy

### Brain Skills

给 Goal Agent 使用：

- requirement-understanding
- goal-drafting
- scope-delta-analysis
- project-health-check
- next-stage-proposal

### Contract Skills

给 Spec Agent 使用：

- requirement-intake
- plan-drafting
- milestone-splitting
- issue-decomposition
- acceptance-authoring
- workflow-binding

### Execution Skills

给 Work Agent 使用：

- panel-context-reading
- implementation
- test-design
- sandbox-verification
- pr-preparation
- merge-followup

### Judgment Skills

给 Audit Agent 使用：

- evidence-review
- boundary-check
- validation-check
- risk-finding
- repair-recommendation

### Delivery Skills

给 Delivery Agent 使用：

- delivery-summary
- change-aggregation
- release-note-drafting
- handoff-packaging
- completion-hinting

## Role -> Flow 映射

| Flow / Stage | 主角色 | 说明 |
| --- | --- | --- |
| Goal / Project Brain | Goal Agent | 目标、方向、阶段建议 |
| Spec / Contract | Spec Agent | 结构化 project / issue 合同 |
| Work Flow | Work Agent | 单任务执行与验证 |
| Audit Flow | Audit Agent | 结果可信度审计 |
| Delivery Flow | Delivery Agent | 交付摘要与公开记录整理 |
| Specialist Subtask | Specialist | 由上层角色按需调用 |

## Handoff 模式

AgentFlow 只允许两种 handoff：

### 1. Ownership Transfer

表示当前阶段结束，后续 authority 交给另一个角色。

例子：

- Goal Agent -> Spec Agent
- Spec Agent -> Work Agent
- Work Agent -> Audit Agent
- Audit Agent -> Delivery Agent
- Delivery Agent -> Goal Agent

### 2. Bounded Capability Call

表示当前角色仍然持有 authority，只把一段局部能力交给 specialist。

例子：

- Work Agent -> Frontend Specialist
- Work Agent -> Test Specialist
- Spec Agent -> Design Specialist

## Handoff 规则表

| from | to | 模式 | 必带 payload | 期望结果 |
| --- | --- | --- | --- | --- |
| Goal Agent | Spec Agent | ownership-transfer | goalRef、planRef、decisionRef、projectContext | 生成可确认合同草案 |
| Spec Agent | Work Agent | ownership-transfer | specIssueRef、workflowRef、validationPlan、boundary | 启动单任务执行 |
| Work Agent | Audit Agent | ownership-transfer | runRef、evidenceRef、validationRef、diffSummary | 进入审计 |
| Audit Agent | Delivery Agent | ownership-transfer | auditResultRef、evidenceIndex、acceptedScope | 进入交付整理 |
| Delivery Agent | Goal Agent | ownership-transfer | deliverySummaryRef、completionHint、openRisks | 目标回看 |
| 任意主角色 | Specialist | bounded-capability-call | scoped task、allowed files、expected output | 返回局部结果 |

## Tool / Context 依赖

| 角色 | 必需上下文 | 常见工具 |
| --- | --- | --- |
| Goal Agent | GOAL / PLAN / DECISIONS / Project Snapshot | projection、state |
| Spec Agent | Goal / Plan / 项目上下文 / requirements | spec storage、panel summary |
| Work Agent | SpecIssue、Panel Context、Validation Plan | panel、local shell、git、provider session |
| Audit Agent | Work Result、Evidence、Validation、Goal / Spec | audit facts、projection、task artifacts |
| Delivery Agent | Audit Result、Evidence Index、Project Context | release、projection、public delivery writer |

## Session 与 Provider 关系

角色不等于 provider，会话也不等于角色。

正确关系是：

```text
Workflow Stage
-> Role Assignment
-> Skill Pack Selection
-> Dispatcher
-> Provider Session
```

也就是说：

- Workflow 先选角色；
- 角色加载技能包；
- Dispatcher 再决定怎么发起 session；
- Provider 只是具体执行载体。

## 不做事项

- 不让一个角色跨越多个主流程同时持有 authority。
- 不让 provider 直接决定 handoff。
- 不让 Specialist 独立接管 Project Flow。
- 不让 Audit Agent 修代码。
- 不让 Delivery Agent 直接判断项目方向。
