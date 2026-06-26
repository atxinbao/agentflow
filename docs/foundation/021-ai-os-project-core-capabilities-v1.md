# 021 - AI OS Project Core Capabilities V1

创建日期：2026-06-26
执行者：Codex
状态：Foundation design draft / 非执行需求

## Purpose

本文定义 AgentFlow 作为 AI OS Project 的底层通用能力。

当前产品目标已经收敛为：

```text
AgentFlow = Spec-Driven Software Dev Workflow
```

但底层能力不能只为 Software Dev 写死。AgentFlow Core 应该是一套可被不同行业壳复用的项目运行时能力。

## Bottom Framework Formula

AgentFlow 的底层框架用这个公式表达：

```text
AI OS Project
= Core Runtime
+ Industry AgentFlow Product
```

展开后：

```text
AI OS Project
= Core Runtime
+ (Domain Pack + Surface Pack + Connector Pack)
```

AgentFlow 的完整表达是：

```text
AgentFlow AI OS Project
= Spec-Driven Core Runtime
+ Industry AgentFlow Product
```

其中：

```text
Spec-Driven Core Runtime
= Spec Kernel
+ Ontology Kernel
+ Runtime Kernel
+ Evidence Kernel
+ Decision Kernel
+ Projection Kernel
```

```text
Industry AgentFlow Product
= Domain Pack
+ Surface Pack
+ Connector Pack
```

行业可以不同，页面可以不同，外部工具可以不同；底层 Runtime 只关心：

```text
这个对象是什么？
这个动作是否合法？
谁能执行？
需要什么证据？
验收条件是什么？
结果写到哪里？
```

## Source Baseline

本文基于以下项目基线：

- `docs/product/006-spec-driven-software-dev-product-goal-v1.md`
- `docs/foundation/agentflow-filesystem-workflow-architecture-v1.md`
- `docs/architecture/049-v100-software-dev-pack-stable-baseline-v1.md`
- `docs/architecture/052-v101-software-dev-pack-usage-baseline-v1.md`

参考方法：

- Palantir Ontology：Object / Link / Action / Function；
- Spec-Driven Development：先定义规范，再执行；
- filesystem-first agent workflow：目录结构表达协议；
- durable workflow：执行可恢复、可重放；
- CQRS / Event Sourcing：写入事实和读取投影分离；
- controller pattern：desired state 推动 actual state；
- provenance model：谁在什么活动中生成了什么产物。

## Top-level Boundary

AgentFlow 至少有两个基础目录平面：

```text
docs/       = Human / Project / Third-party Knowledge Plane
.agentflow/ = Agent Runtime Control Plane
```

### docs/

`docs/` 面向人类团队、第三方集成方和 Spec Builder。

它负责解释项目：

- 产品目标；
- Spec-Driven 方法；
- 行业领域定义；
- 已确认的 Spec Bundle；
- 架构设计；
- 验证规则；
- 版本计划、发布审计和修复计划。

`docs/` 是可读的知识层和确认层，不是 Agent 执行状态机。

### .agentflow/

`.agentflow/` 面向 Agent、Runtime、Projection、Decision Gate 和 Audit Agent。

它负责执行项目：

- Agent 手册、角色和技能；
- 可执行 spec project / issue contract；
- task run；
- event stream；
- evidence；
- projection；
- audit facts；
- runtime state。

`.agentflow/` 是机器可执行的控制层和事实层，不承载长篇产品解释。

## Core Loop

AgentFlow AI OS Project 的通用主链是：

```text
Intent
-> Spec Bundle
-> Route
-> Ontology / Action Contract
-> Runtime Command
-> Admission / Arbitration
-> Agent Execution
-> Event Store
-> Evidence
-> Decision
-> Delivery
-> Feedback
-> Spec Evolution
```

大白话：

```text
先听懂人要什么
-> 写成可确认的说明书
-> 判断该走哪种流程
-> 翻译成系统能执行的对象和动作
-> 发正式执行命令
-> 检查谁能做、能不能做、会不会冲突
-> 让 Agent 干活
-> 记录发生过的事
-> 保存证据
-> 判断是否真的完成
-> 交付给人
-> 收反馈
-> 更新下一版说明书
```

## Minimal Kernel

AgentFlow Core 最小内核由 6 个 Kernel 组成。

| Kernel | 职责 | 不负责 |
| --- | --- | --- |
| Spec Kernel | 定义需求、目标、路线、范围、验收和输出 | 不直接执行任务 |
| Ontology Kernel | 定义项目世界里的 Object / Link / Action / State / Capability | 不承载 UI 文案 |
| Runtime Kernel | 接收 command，执行 admission、arbitration、run lifecycle 和 action dispatch | 不把外部系统当 authority |
| Evidence Kernel | 记录命令、输出、artifact、diff、日志、截图、证明和 provenance | 不替代 Decision 决策 |
| Decision Kernel | 根据 Spec + Evidence 判断 accepted / rejected / deferred / blocked | 不执行实现任务 |
| Projection Kernel | 生成 UI / API / 第三方可读 read model 和 view model | 不写 authority facts |

一句话：

```text
Spec 管方向。
Ontology 管项目世界。
Runtime 管动作。
Evidence 管证明。
Decision 管验收判定。
Projection 管读取。
```

## Twelve Core Capabilities

### 1. Project Identity

定义一个 Project 的身份、目标、范围、状态和生命周期。

最低字段：

- project id；
- project name；
- product/domain target；
- current goal；
- lifecycle state；
- active spec bundle；
- current release / delivery state。

### 2. Filesystem Contract

固定 `docs/` 与 `.agentflow/` 的职责边界。

规则：

- `docs/` 保存人类可读的产品目标、领域定义、Spec Bundle、架构和验证说明；
- `.agentflow/` 保存机器可执行合同、运行事实、事件、证据、投影和审计事实；
- GitHub issue、PR、provider session、外部工具状态都不能成为 AgentFlow authority。

### 3. Spec Bundle

Spec 不是单一 Markdown 文档，而是需求操作合同。

标准结构：

```text
Spec Bundle
├── Intent Slice
├── Domain Slice
├── Product Slice
├── Plan Slice
├── Task Slice
├── Decision Slice
├── Delivery Slice
└── Feedback Slice
```

PRD、技术方案、issues、验收和交付都从 Spec Bundle 派生。

### 4. Requirement Router

判断人类输入属于哪类需求，并决定 route。

基础类型：

| Type | Route |
| --- | --- |
| question | answer-only |
| research | research |
| feature | product + plan + tasks |
| bug | reproduce + fix |
| audit | audit sidecar |
| release | release certification |
| design-only | design flow |
| maintenance | cleanup / migration |

Router 的第一职责不是生成 issue，而是防止错误需求进入错误流程。

### 5. Project Ontology

定义项目世界里的对象和关系。

最低元素：

- Object Type；
- Link Type；
- State Machine；
- Action Type；
- Capability；
- Role Policy；
- Version / Migration rule。

Software Dev 的对象可以是：

```text
Requirement / Goal / PRD / Spec / Architecture Plan / Issue / Run / Evidence / Decision / PR / Release / Feedback
```

视频制作的对象可以是：

```text
Brief / Script / Shot / Asset / Cut / Render / Review / Delivery
```

底层 Runtime 不应该写死这些行业对象。

### 6. Action Contract

每个 Action 必须有可验证语义。

最低字段：

- action id；
- target object type；
- input schema；
- precondition；
- effect；
- required role / capability；
- required evidence；
- acceptance impact；
- cancel / rollback rule；
- emitted events。

没有 Action Contract 的动作不能进入 Runtime Command。

### 7. Runtime API

提供统一 command / query / event 入口。

规则：

- command 会尝试改变状态；
- query 只读 projection；
- event 是事实记录；
- Runtime API 是 `.agentflow/` authority 写入入口；
- Agent 和 connector 不应该绕过 Runtime API 直接写核心事实。

### 8. Execution Loop

管理 Agent 执行过程。

最低流程：

```text
issue contract
-> handoff
-> preflight
-> run
-> verification
-> evidence pack
-> acceptance decision
-> completion writeback
```

执行器可以是 Codex、Claude Code、MCP tool、local shell 或其他 provider。

但执行器只是 worker，不是项目事实源。

### 9. Event Store

记录已经发生的事实。

规则：

- Event Store 是事实权威；
- Event 必须 append-only；
- Event 必须包含 source、actor、time、target、payload、schema version；
- Replay 应该能重建 projection；
- Event 不能被 UI 直接手写。

### 10. Evidence Store

保存可审计证据。

证据类型：

- command output；
- exit code；
- test log；
- build log；
- diff summary；
- artifact manifest；
- screenshot；
- browser proof；
- release provenance；
- decision record。

Evidence 证明发生过什么，但不自动等于 Done。

### 11. Decision Gate

根据 Spec + Evidence 做完成判断。

Decision Gate 至少包含：

- Verification Gate：验证命令是否通过；
- Evidence Gate：证据是否完整；
- Contract Gate：是否满足 Spec / Issue 合同；
- Boundary Gate：是否越权或越界；
- State Gate：状态是否允许 Done；
- Decision result：accepted / rejected / deferred / blocked。

只有 Decision Gate 通过，才允许 Completion Commit。

### 12. Projection Surface

生成只读输出面。

输出包括：

- Project Home；
- Spec Workbench；
- Task Workbench；
- Evidence Graph；
- Decision Surface；
- Delivery Surface；
- Audit Sidecar Surface；
- Feedback Loop。

Projection 只读，不是 authority。

## Industry Pack Model

行业能力通过 Pack 接到底层 Core。

```text
Industry AgentFlow Product
= Domain Pack + Surface Pack + Connector Pack
```

### Domain Pack

定义行业世界：

- object types；
- link types；
- action types；
- state machines；
- domain rules；
- acceptance defaults。

### Surface Pack

定义行业现场：

- workbench；
- read models；
- view models；
- commands shown to humans；
- delivery views；
- feedback surfaces。

### Connector Pack

定义外部系统：

- provider；
- tool；
- API；
- MCP server；
- auth / capability status；
- artifact import / export；
- evidence adapter。

Connector Pack 不能直接写 authority facts。

## Software Dev First Product Mapping

当前商业产品只聚焦 Software Dev。

Software Dev 应该使用底层 Core，而不是绕过底层 Core。

| Core capability | Software Dev mapping |
| --- | --- |
| Spec Bundle | PRD / technical plan / issues / decision / delivery |
| Ontology | Requirement / Spec / Issue / Run / PR / Release |
| Action Contract | create issue / run tests / open PR / complete delivery |
| Runtime API | start run / prepare review / write proof / complete |
| Evidence Store | diff / test log / build log / PR link / release note |
| Decision Gate | tests pass + scope respected + evidence complete + delivery ready |
| Projection | Project Home / Task Workbench / Evidence Graph / Delivery Surface |

Audit 仍然是 sidecar，不进入主业务链。

## Authority Rules

必须固定以下权威关系：

| Item | Authority |
| --- | --- |
| Product goal | `docs/product/**` |
| Human-readable Spec Bundle | `docs/requirements/**` |
| Executable project / issue contract | `.agentflow/spec/**` |
| Runtime facts | `.agentflow/events/**` |
| Task run and evidence | `.agentflow/tasks/**` |
| UI read model | `.agentflow/projections/**` |
| Audit findings | `.agentflow/audit/**` |
| GitHub issue / PR | mirror / delivery / external reference only |
| Codex / Claude session | executor transcript only |

## Non-goals

本文不做：

- 不定义新的行业商业目标；
- 不启动视频制作、金融、制造或运营行业壳；
- 不引入默认 Message Bus；
- 不定义云端多租户平台；
- 不授权代码实现；
- 不生成 `.agentflow/spec/**`；
- 不创建 GitHub issue；
- 不改变 v1.0.x release hardening 任务。

## Next Decomposition

后续应该从本文拆出 6 份更具体的 foundation 文档：

```text
022-spec-kernel-contract-v1.md
023-ontology-kernel-contract-v1.md
024-runtime-kernel-contract-v1.md
025-evidence-kernel-contract-v1.md
026-decision-kernel-contract-v1.md
027-projection-kernel-contract-v1.md
```

这些文档确认后，才能进一步转成 `docs/requirements/**` 的当前版本需求切片。
