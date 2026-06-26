# 006 - Spec-Driven Software Dev Product Goal V1

创建日期：2026-06-26
执行者：Codex

## Purpose

本文固定 AgentFlow 当前产品目标。

AgentFlow 当前不以“通用多行业 Agent OS 商业产品”为开发目标。底层 Project OS 能力仍然保留，但当前商业产品目标收敛为：

```text
AgentFlow = Spec-Driven Software Dev Workflow
```

一句话：

```text
AgentFlow 不是 Agent Runner，也不是 Spec 文档生成器，而是面向软件开发团队的 Spec 驱动 Agent 工作流产品。
```

## Product Goal

AgentFlow 要解决的问题是：

```text
AI Agent 可以快速写代码，但软件开发流程、任务边界、证据、验收、交付和反馈很容易失控。
```

AgentFlow 的产品目标是：

```text
让团队用 Spec 作为方向盘，驱动 Codex / Claude Code / 其他 coding agents 完成可控、可追踪、可验收、可交付的软件开发。
```

## Core Method

AgentFlow 采用 Spec-Driven Development，但它不只约束代码生成。

AgentFlow 的 SDD 范围是：

```text
Spec-Driven Software Dev Workflow
```

也就是：

- Spec 约束需求理解；
- Spec 约束计划和任务拆分；
- Spec 约束 Agent 执行边界；
- Spec 约束验证和证据；
- Spec 约束 Acceptance；
- Spec 约束 Delivery；
- Spec 约束 Feedback 如何回流到下一轮工作。

## Main Product Loop

AgentFlow 的主产品闭环是：

```text
Intent
-> Spec Bundle
-> Route
-> Domain-specific Plan / Tasks / Artifacts
-> Agent Execution
-> Evidence
-> Acceptance
-> Delivery
-> Feedback
-> Spec Evolution
```

### Plain Meaning

一个团队把需求交给 AgentFlow。

AgentFlow 先生成可确认的 Spec Bundle。

Spec Bundle 派生产品目标、技术方案、任务、执行边界和验收标准。

Build Agent 按任务执行。

系统自动收集 Evidence。

Acceptance Gate 根据 Spec 判断能不能 Done。

Delivery 输出 PR、release、changelog 或 handoff。

Feedback 回流到下一轮 Spec。

## Spec Bundle

Spec 不是单一 Markdown 文档。

Spec 是 AgentFlow 的需求操作合同。它可以包含多个 slice：

```text
Spec Bundle
├── Intent
│   ├── human input
│   ├── request type
│   ├── goal
│   ├── non-goals
│   └── confirmation state
│
├── Domain Context
│   ├── domain terms
│   ├── object model
│   ├── business constraints
│   └── domain rules
│
├── Product Slice
│   ├── PRD / Goal
│   ├── user scenario
│   ├── scope
│   └── success criteria
│
├── Plan Slice
│   ├── technical approach
│   ├── architecture boundary
│   ├── data / API / module impact
│   └── risks
│
├── Task Slice
│   ├── issues
│   ├── dependencies
│   ├── allowed paths
│   ├── forbidden paths
│   └── execution pipeline
│
├── Acceptance Slice
│   ├── acceptance criteria
│   ├── validation commands
│   ├── evidence policy
│   └── Done decision rule
│
└── Output Slice
    ├── delivery package
    ├── feedback rule
    ├── audit trigger rule
    └── spec evolution rule
```

PRD、技术方案、issues、验收和交付都从 Spec Bundle 派生。

## Request Routing

不同需求必须进入不同流程。

Spec Builder 的第一职责不是生成 issue，而是决定该需求应该走什么 route。

| Request type | Route | Output |
| --- | --- | --- |
| question | answer-only | explanation / recommendation |
| research | research | research findings / decision proposal |
| feature | product + plan + tasks | Product Slice / Plan Slice / Task Slice |
| bug | reproduce + fix | bug issue / regression evidence |
| audit | audit sidecar | audit report / finding |
| release | release certification | release proof / closeout |
| design-only | design flow | design artifact / handoff |
| maintenance | cleanup / migration | maintenance issues / migration evidence |

## Software Dev Domain

当前商业产品只聚焦 Software Dev。

Software Dev Domain Pack 应优先定义：

```text
Requirement
Goal / PRD
Spec
Architecture Plan
Issue
Run
Evidence
Acceptance
PR
Release
Audit Finding
Feedback
```

Software Dev Connector Pack 应优先围绕：

- Git；
- GitHub；
- Codex；
- Claude Code；
- local shell；
- test / build / lint；
- browser preview；
- release notes。

Software Dev Surface Pack 应优先围绕：

- Spec Workbench；
- Project Home；
- Task Workbench；
- Evidence Graph；
- Acceptance Gate；
- Delivery Surface；
- Feedback Loop；
- Audit Sidecar Surface。

## Agent Role Principle

Agent 只是执行器。

Spec 才是方向盘。

AgentFlow 可以有多个 Agent role，但任何 Agent 都不能越过 Spec：

- Spec Agent 生成和维护 Spec Bundle；
- Architecture / Specialist Agent 补充 Plan Slice；
- Build Agent 执行 Task Slice；
- Verification Agent 或验证流程生成 Evidence；
- Acceptance Gate 判断 Done；
- Delivery Agent 整理交付；
- Audit Agent 独立复查，不进入主链。

Agent role 不拥有项目事实。

项目事实来自 Spec、Event、Evidence、Acceptance 和 Delivery 记录。

## Output Closed Loop

输出不能只等于代码。

每次执行后必须形成四类结果：

```text
Artifact
Evidence
Decision
Feedback
```

| Output | Meaning |
| --- | --- |
| Artifact | code, docs, PR, release, design artifact |
| Evidence | tests, build output, diff, screenshot, log, proof |
| Decision | accepted, rejected, deferred, blocked |
| Feedback | user feedback, bug, new requirement, spec revision |

只有 Artifact 没有 Evidence，不算完成。

只有 Evidence 没有 Acceptance，不算 Done。

只有 Delivery 没有 Feedback，不算闭环。

## Product Boundaries

当前阶段不做：

- 通用多行业商业平台；
- Pack marketplace；
- 视频制作行业壳；
- 金融 / 制造 / 运营行业壳；
- 默认中心化 Message Bus；
- 多租户云平台优先；
- 把 GitHub issues 作为 AgentFlow authority；
- 把 executor session 当成项目事实源；
- 把 Audit 放回主业务链。

可以保留为底层能力或未来扩展的是：

- Domain Pack；
- Surface Pack；
- Connector Pack；
- Runtime API / SDK；
- Event / Projection；
- Audit sidecar；
- Software Dev 之外的实验性样例。

## Product Decision Rule

后续所有产品、工程和 UI 决策都必须先回答：

```text
它是否增强 Spec-Driven Software Dev Workflow？
```

如果答案是否定的，默认不进入当前产品主线。

如果答案是肯定的，还必须说明它增强哪一段：

```text
Intent
Spec Bundle
Route
Plan / Tasks / Artifacts
Agent Execution
Evidence
Acceptance
Delivery
Feedback
Spec Evolution
```

## Relationship To Versions

本文不调整现有版本规划。

本文只固定产品目标，供后续版本、requirements、spec issues 和 UI 设计引用。

后续版本可以围绕本文拆分需求，但不能把本文本身当成可执行 issue。
