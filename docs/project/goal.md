# 006 - Spec-Driven AI OS Project Goal V3

创建日期：2026-06-26
更新日期：2026-06-30
执行者：Codex

## Purpose

本文固定 AgentFlow 当前项目目标。

AgentFlow 不是单一行业工具，不是 Agent Runner，也不是 Spec 文档生成器。

AgentFlow 的目标是：

```text
AgentFlow = Spec-Driven AI OS Project
```

一句话：

```text
AgentFlow 是用 Spec 驱动项目对象、动作、证据、验收和投影的 AI 项目操作系统。
```

## Bottom Formula

AgentFlow 的底层框架是：

```text
AgentFlow AI OS Project
= Core OS Runtime
+ Industry Product Surface
```

其中：

```text
Core OS Runtime
= Spec Kernel
+ Ontology Kernel
+ Runtime Kernel
+ Evidence Kernel
+ Decision Kernel
+ Projection Kernel
```

```text
Industry Product Surface
= Paid Report Flow
or Managed Project Flow
or Assistant / Ops Flow
```

每个 Industry Product Surface 由三类内置 Pack 支撑：

```text
Industry Product Surface
= Domain Pack
+ Surface Pack
+ Connector Pack
```

Core OS Runtime 只提供通用项目运行能力。

Industry Product Surface 定义具体行业的输入、对象、页面或报告、工具和交付方式。

行业层不一定是完整 App。它可以是：

- Paid Report Flow：用户输入少量信息并付费，AgentFlow 内部完成一次受控 Run，交付一份报告；
- Managed Project Flow：用户确认目标，AgentFlow 管理多轮任务、证据、验收和交付；
- Assistant / Ops Flow：用户授权持续监控、提醒、执行和反馈。

Software Dev 是第一个官方 Reference App，不是 Core OS 的内核目标。

## Product Goal

AgentFlow 要解决的问题是：

```text
AI Agent 可以快速执行任务，但项目目标、对象、动作边界、证据、验收、交付和反馈很容易失控。
```

AgentFlow 的产品目标是：

```text
让团队用 Spec 作为方向盘，驱动 Codex / Claude Code / 其他 agents 完成可控、可追踪、可验收、可交付的项目工作。
```

商业产品层面的目标是：

```text
AgentFlow 不先卖 Agent。
AgentFlow 卖的是可交付结果。
```

对用户来说，最清晰的付费路径是：

```text
输入信息
-> 支付
-> AgentFlow 内部完成一次受控 Run
-> 交付可验收结果
```

所以 AgentFlow 的真实产品入口可以是一次性服务，而不一定是工作台：

```text
Paid Report Flow
= Product Order
+ One-shot AgentFlow Run
+ Report Delivery
```

例如八字合盘报告、合同审查报告、商业尽调摘要、留学申请评估、装修方案报告、品牌命名报告，都可以复用同一套 Core OS Runtime，只更换 Domain Pack、Surface Pack 和 Connector Pack。

Software Dev 是第一个官方 Reference App，因为它最容易验证完整闭环：

```text
Requirement
-> Spec
-> Issue
-> Run
-> Evidence
-> Decision
-> Delivery
-> Feedback
```

但这些 Software Dev 词汇属于 Software Dev App / Pack，不能直接成为 Core OS 的唯一模型。

## Core Method

AgentFlow 采用 Spec-Driven Development，但它不只约束代码生成。

AgentFlow 的 SDD 范围是：

```text
Spec-Driven Project Workflow
```

也就是：

- Spec 约束意图理解；
- Spec 约束对象、动作、计划和任务拆分；
- Spec 约束 Agent 执行边界；
- Spec 约束验证和证据；
- Spec 约束 Decision；
- Spec 约束 Delivery；
- Spec 约束 Feedback 如何回流到下一轮工作。

## Main Product Loop

AgentFlow 的通用主闭环是：

```text
Intent
-> Spec Bundle
-> Route
-> Domain-specific Plan / Tasks / Artifacts
-> Agent Execution
-> Evidence
-> Decision
-> Delivery
-> Feedback
-> Spec Evolution
```

### Plain Meaning

一个团队把需求交给 AgentFlow。

AgentFlow 先生成可确认的 Spec Bundle。

Spec Bundle 派生项目目标、方案、任务、执行边界和验收标准。

Agent 按任务执行。

系统自动收集 Evidence。

Decision Gate 根据 Spec 判断能不能 Done。

Delivery 输出行业对应的交付物。

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
│   ├── tasks
│   ├── dependencies
│   ├── allowed surfaces
│   ├── forbidden surfaces
│   └── execution pipeline
│
├── Decision Slice
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

PRD、技术方案、任务、验收和交付都从 Spec Bundle 派生。

Core OS 只定义 slice 合同。

Industry Product Surface 负责把 slice 映射成行业语言。例如 Software Dev Managed Project Flow 可以把 Task Slice 映射成 issue，把 Output Slice 映射成 PR / release / changelog。

Industry Product Surface 也可以把 slice 映射成报告型语言。例如 Paid Report Flow 可以把 Product Slice 映射成订单目标，把 Task Slice 映射成一次性内部 Run，把 Output Slice 映射成报告交付、退款边界和反馈入口。

## Request Routing

不同需求必须进入不同流程。

Spec Builder 的第一职责不是生成任务，而是决定该需求应该走什么 route。

| Request type | Route | Output |
| --- | --- | --- |
| question | answer-only | explanation / recommendation |
| research | research | research findings / decision proposal |
| feature | product + plan + tasks | Product Slice / Plan Slice / Task Slice |
| bug | reproduce + fix | bug or defect task / regression evidence |
| audit | audit sidecar | audit report / finding |
| release | release certification | release proof / closeout |
| design-only | design flow | design artifact / handoff |
| maintenance | cleanup / migration | maintenance tasks / migration evidence |

## Core / App Boundary

Core OS Runtime 只能直接认识通用概念：

```text
Object
Link
Action
Run
Artifact
Evidence
Decision
Projection
Route
Spec Bundle
```

Core OS Runtime 不能把这些 Software Dev 概念写死为唯一模型：

```text
Issue
PR
Release
Bug
Patch
Test Log
Architecture Plan
```

这些概念应由 Software Dev App 通过 Domain Pack、Surface Pack 和 Connector Pack 声明。

Core OS Runtime 也不能把报告型商业对象写死为唯一模型：

```text
Order
Payment
Refund
Customer
Report Template
Report Delivery
```

这些概念属于 Paid Report Flow 的 Product Surface，可以通过 Surface Pack、Connector Pack 和商业系统连接声明。Core 只需要知道这次 Flow 是否被授权执行、证据是否齐全、Decision 是否允许交付。

## First Reference App: Software Dev

Software Dev 是第一个官方 Reference App。

Software Dev Domain Pack 应优先定义：

```text
Requirement
Goal / PRD
Spec
Architecture Plan
Issue
Run
Evidence
Decision
PR
Release
Audit Finding
Feedback
```

Software Dev Action mapping 应优先定义：

```text
createIssue
startRun
writePatch
runValidation
submitEvidence
prepareDelivery
openPR
recordRelease
requestFix
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
- Decision Gate；
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
- Decision Gate 判断 Done；
- Delivery Agent 整理交付；
- Audit Agent 独立复查，不进入主链。

Agent role 不拥有项目事实。

项目事实来自 Spec、Event、Evidence、Decision 和 Delivery 记录。

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

只有 Evidence 没有 Decision，不算 Done。

只有 Delivery 没有 Feedback，不算闭环。

## Product Boundaries

当前阶段不做：

- Pack marketplace；
- 视频制作行业壳；
- 金融 / 制造 / 运营行业壳；
- 把 Paid Report Flow 写死为唯一商业形态；
- 把 Order / Payment / Refund 写进 Core OS Runtime；
- 默认中心化 Message Bus；
- 多租户云平台优先；
- 把 GitHub issues 作为 AgentFlow authority；
- 把 executor session 当成项目事实源；
- 把 Audit 放回主业务链；
- 把 Software Dev 写死进 Core OS Runtime。

可以保留为底层能力或未来扩展的是：

- Domain Pack；
- Surface Pack；
- Connector Pack；
- Paid Report Flow；
- Managed Project Flow；
- Assistant / Ops Flow；
- Runtime API / SDK；
- Event / Projection；
- Audit sidecar；
- Software Dev 之外的实验性样例。

## Product Decision Rule

后续所有产品、工程和 UI 决策都必须先回答：

```text
它是否增强 Core OS Runtime，或者增强某个 Industry Product Surface 在 Core 上的闭环交付能力？
```

如果答案是否定的，默认不进入当前产品主线。

如果答案是肯定的，还必须说明它增强哪一段：

```text
Intent
Spec Bundle
Route
Domain App Mapping
Agent Execution
Evidence
Decision
Delivery
Feedback
Spec Evolution
```

如果是商业产品能力，还必须说明它卖的是什么结果：

```text
Paid Report Flow    卖一份可验收报告
Managed Project Flow 卖一次可追踪项目交付
Assistant / Ops Flow 卖持续托管执行结果
```

## Relationship To Versions

本文调整后续版本目标的表达方式。

`v1.0.3` 到 `v1.0.8` 应优先收敛 Core OS Runtime 的 6 个 Kernel。

`v1.0.9` 使用 Software Dev Reference App 认证 Core 能力。

`v1.1.0` 之后再进入 Software Dev Product Beta。

本文只固定目标，供后续 roadmap、requirements、spec issues 和 UI 设计引用。

后续版本可以围绕本文拆分需求，但不能把本文本身当成可执行 issue。
