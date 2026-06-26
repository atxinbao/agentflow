# AgentFlow Architecture Decision Record Draft V1

日期：2026-06-19
执行者：Codex
用途：Next Version Development Preflight / 下一版本开发前置架构决策记录
状态：Architecture Draft / 非执行需求 / 不进入当前 v0.3.0 审计 / 不授权 Build Agent 执行

关联架构基线：

- [AGENTFLOW_AGENT_PROJECT_OS_ARCHITECTURE_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_AGENT_PROJECT_OS_ARCHITECTURE_V1.md)
- [AGENTFLOW_ONTOLOGY_SCHEMA_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_ONTOLOGY_SCHEMA_DRAFT_V1.md)
- [AGENTFLOW_ACTION_CONTRACT_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_ACTION_CONTRACT_DRAFT_V1.md)
- [AGENTFLOW_AGENT_ROLE_POLICY_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_AGENT_ROLE_POLICY_DRAFT_V1.md)
- [AGENTFLOW_OBJECT_STATE_MACHINE_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_OBJECT_STATE_MACHINE_DRAFT_V1.md)

说明：本文件沉淀 AgentFlow Agent Project OS 的关键架构决策。它不创建 `.agentflow/spec/**` 任务事实，不授权执行，也不替代正式 SPEC。

## 1. Purpose

本文件记录下一版本前置架构中不应反复摇摆的决策。

它不是产品需求文档，也不是执行 issue。
它的作用是让后续 SPEC、实现、评审和审计有一个稳定的架构判断基线。

ADR 条目使用同一结构：

```text
Decision
Rationale
Consequences
Rejected Alternatives
Status
```

## 2. Decision Index

| id | decision | status |
| --- | --- | --- |
| `ADR-001` | AgentFlow 是 Agent Project OS，不只是 Agent workflow tool | accepted-draft |
| `ADR-002` | 行业产品是独立壳，底层 Runtime 复用 | accepted-draft |
| `ADR-003` | Project Ontology Layer 是 Runtime 前置定义层 | accepted-draft |
| `ADR-004` | Runtime Core 主链路固定为 Spec Loop → Contract → Build Loop → Arbitration → Event Store → Projection | accepted-draft |
| `ADR-005` | Event Store 是事实权威 | accepted-draft |
| `ADR-006` | Agent 不直接改状态，只提交 Action Proposal | accepted-draft |
| `ADR-007` | Action Arbitration 是多 Agent 写入唯一入口 | accepted-draft |
| `ADR-008` | Object State Machine 是项目状态基础 | accepted-draft |
| `ADR-009` | Audit 独立于 Build Done | accepted-draft |
| `ADR-010` | Projection Surface 只读事实并发起命令回流 | accepted-draft |
| `ADR-011` | Agent Role 是能力合约，不是 prompt 文案 | accepted-draft |
| `ADR-012` | Message Bus 是调度设施，不是事实源 | accepted-draft |

## 3. ADR-001: AgentFlow Is Agent Project OS

### Decision

AgentFlow 的目标定位为 Agent Project OS，而不是普通 Agent workflow tool。

### Rationale

普通 workflow tool 只描述任务顺序。
Agent Project OS 必须定义项目世界：

- 有哪些对象；
- 对象之间如何关联；
- 哪些动作合法；
- 哪些 Agent 能做什么；
- 状态如何变化；
- 事实如何留痕；
- 输出如何投影给行业客户端。

没有这个 OS 层，AgentFlow 只能管理流程，不能稳定管理跨 Agent、跨行业、跨版本的项目事实。

### Consequences

- 下一版本设计必须优先定义 Runtime Core 和 Project Ontology；
- UI、CLI、行业客户端都不能成为事实权威；
- 所有执行必须回到 Runtime 的对象、动作、事件、投影模型。

### Rejected Alternatives

- 只做任务流编排；
- 只做多 Agent 对话协调；
- 只做项目管理 UI；
- 把行业需求直接硬编码进一个统一客户端。

### Status

`accepted-draft`

## 4. ADR-002: Industry Product Shells Are Separate

### Decision

行业产品应独立设计，底层 Agent Project Runtime 复用。

标准形态：

```text
Industry AgentFlow Product
= Industry Product Shell
+ Domain Pack
+ Surface Pack
+ Connector Pack
+ Projection Surface
+ Agent Project Runtime
```

### Rationale

不同行业现场差异很大。

软件开发、前端设计、视频制作、金融运营的对象、页面、证据、工具和交付物都不同。把所有行业塞进同一个客户端，会让产品变成低质量通用后台。

正确边界是：

```text
底层 Runtime 稳定
行业客户端分开设计
输入输出通过标准协议接入 Runtime
```

### Consequences

- Runtime API / SDK 必须稳定；
- 行业壳不能直接写 Runtime 事实；
- Domain Pack、Surface Pack、Connector Pack 必须有清晰边界；
- 一个行业产品失败不应污染底层 Runtime 架构。

### Rejected Alternatives

- 单一超级客户端适配所有行业；
- 每个行业都复制一套 Runtime；
- 直接把行业对象写进核心代码，不做 pack 边界。

### Status

`accepted-draft`

## 5. ADR-003: Project Ontology Layer Is Required

### Decision

Project Ontology Layer 是 Runtime Core 的前置定义层。

它定义：

- Object Types；
- Link Types；
- Action Types；
- Function Types；
- State Machines；
- Agent Role Policy；
- Versioning / Migration；
- Evaluation / Simulation。

### Rationale

Agent 要能可靠执行，必须先知道项目世界里什么东西存在、什么关系合法、什么动作会造成什么影响。

如果没有 Project Ontology，Agent 行为只能靠 prompt 和上下文猜测。这个模型无法支撑多 Agent 并发、行业扩展、审计追踪和事件回放。

### Consequences

- Runtime Core 只读取标准化后的定义；
- 行业输入必须先转换为 Project Ontology 能理解的对象、链接和动作；
- Ontology 定义必须注册、版本化、可迁移；
- 后续实现需要提供 Ontology Registry。

### Rejected Alternatives

- 用 prompt 隐式描述对象和动作；
- 用 UI 字段充当项目模型；
- 让每个 Agent 自己理解行业上下文；
- 把所有行业术语直接写进核心 Runtime。

### Status

`accepted-draft`

## 6. ADR-004: Runtime Core Main Chain Is Fixed

### Decision

AgentFlow Runtime Core 的主链路固定为：

```text
Spec Loop
→ Contract
→ Build Loop
→ Agent Action Arbitration
→ Event Store
→ Projection
```

### Rationale

这条链路表达了项目从需求到执行再到输出的最小闭环：

- Spec Loop 负责理解需求；
- Contract 负责锁定边界；
- Build Loop 负责推进交付；
- Agent Action Arbitration 负责合法性、冲突和并发；
- Event Store 负责记录事实；
- Projection 负责把事实变成可读状态。

Message Bus、Artifact Store、Cache、Connector、UI 都是支撑设施，不应成为主链路。

### Consequences

- 新模块必须说明自己处于主链路还是支撑设施；
- 执行动作不能绕过 Contract 和 Arbitration；
- Projection 不能反向直接写事实；
- Build Loop 不能直接结束审计。

### Rejected Alternatives

- 把 Message Bus 放在架构中心；
- 让 UI 页面驱动核心状态；
- 让 Build Agent 直接写最终完成事实；
- 把 Spec、Build、Audit 混成一条不可拆流程。

### Status

`accepted-draft`

## 7. ADR-005: Event Store Is Fact Authority

### Decision

Event Store 是 AgentFlow 的唯一事实权威。

所有被接受的状态变化、动作结果、人类决策、证据链接都必须以事件记录。

### Rationale

AgentFlow 需要支持：

- 多 Agent 执行；
- 审计；
- 回放；
- Projection 重建；
- 版本迁移；
- 跨行业客户端读取。

这些能力都要求事实不可被 UI、缓存或临时状态覆盖。Event Store 必须成为 append-only 的事实层。

### Consequences

- UI 只读 Projection；
- Message Bus 不保存事实；
- 状态从事件推导；
- 任何直接写状态的实现都需要被拒绝；
- 修复历史不能覆盖旧事实，只能追加新事件。

### Rejected Alternatives

- 以数据库当前行状态为事实权威；
- 以任务 JSON 文件当前值为唯一事实；
- 以 UI 当前状态为事实；
- 以 Agent 回答文本为事实。

### Status

`accepted-draft`

## 8. ADR-006: Agents Propose Actions, Runtime Changes State

### Decision

Agent 不直接改状态。
Agent 只能提交 Action Proposal。
Runtime 根据 Action Contract、State Machine、Role Policy 和 Arbitration 结果决定是否写入事实事件。

### Rationale

Agent 的能力来自推理和工具调用，但项目状态必须可控、可审计、可回放。

如果 Agent 可以直接写最终状态，就无法区分：

- 它想做什么；
- 它是否有权限；
- 当前状态是否允许；
- 证据是否充分；
- 是否与其他 Agent 冲突。

### Consequences

- 所有写动作必须有 Action Contract；
- 所有状态变化必须有 State Machine transition；
- Agent 输出和事实写入必须分离；
- Build Agent、Audit Agent、Spec Agent 的写入能力必须受 Role Policy 限制。

### Rejected Alternatives

- Agent 直接写 issue done；
- Agent 直接写 audit pass；
- Agent 直接改 project status；
- Agent 通过自然语言声明事实成立。

### Status

`accepted-draft`

## 9. ADR-007: Action Arbitration Is The Multi-agent Write Gate

### Decision

Agent Action Arbitration 是多 Agent 写入的唯一入口。

它负责决定 Action Proposal 的结果：

```text
accept
reject
queue
requireHumanDecision
cancel
supersede
```

### Rationale

多 Agent 的难点不是“能否同时运行”，而是“同时写入时谁能改变项目事实”。

Arbitration 必须处理：

- 对象锁；
- 依赖顺序；
- Role Policy；
- Action Contract；
- State Machine；
- Evidence；
- 冲突动作；
- 人类确认。

### Consequences

- Build Loop 和 Event Store 之间必须有 Arbitration；
- 多 Agent 并发不能绕过对象锁；
- 冲突写入要明确进入 rejected、queued 或 requireHumanDecision；
- Arbitration 的结果必须可追踪。

### Rejected Alternatives

- 多 Agent 直接并发写文件；
- 让最后写入者获胜；
- 让 Coordinator Agent 靠 prompt 人工调停；
- 用 Message Bus 顺序替代权限和状态校验。

### Status

`accepted-draft`

## 10. ADR-008: Object State Machine Is The State Base

### Decision

AgentFlow 的项目状态必须拆成对象状态机。

核心对象包括：

```text
Requirement
Spec
Issue
Run
Audit
Finding
Decision
Evidence
Artifact
```

### Rationale

一条线性流程无法表达真实项目。

错误模型：

```text
Requirement → Spec → Issue → Run → Done
```

正确模型：

```text
Object State Machines
+ Link Types
+ Action Types
+ Event Store
```

不同对象有不同生命周期。Audit、Finding、Run、Issue 不能被塞进同一个状态字段。

### Consequences

- 后续实现必须先定义对象状态机；
- `Run.completed` 不等于 `Issue.done`；
- `Issue.done` 不等于 `Audit.closed`；
- Finding 修复要通过新的或关联的 Issue 执行；
- Projection 负责合成用户可读的整体状态。

### Rejected Alternatives

- 一个全局 project status 代表所有事实；
- 一个 issue status 代表执行、审计、交付全部状态；
- Build Done 自动带出 Audit Done；
- Finding 直接回写旧 Issue 的完成事实。

### Status

`accepted-draft`

## 11. ADR-009: Audit Is Independent From Build Done

### Decision

Audit 是独立流程，不是 Build Done 的默认尾巴。

Build Agent Done writeback 不得自动创建 audit request，不得写 audit report，不得创建 finding，不得标记审计通过。

### Rationale

Build 和 Audit 的职责不同：

- Build 负责按 Issue Contract 交付；
- Audit 负责独立检查 evidence、traceability、risk、regression；
- Finding 通过修复 Issue 回流，而不是修改 Build Agent 的交付事实。

把 Audit 并入 Build Done，会破坏审计独立性，也会让同一个 Agent 同时成为交付者和审计者。

### Consequences

- Audit 只能从独立 Audit Issue 或明确人类审计请求开始；
- Build 输出可以成为 Audit 输入；
- Audit Finding 可以派生修复 Issue；
- Audit Surface 只展示独立审计状态和 findings；
- UI 不应暗示 Build 完成会自动审计。

### Rejected Alternatives

- Build 完成后自动创建 audit request；
- Build Agent 顺手写审计报告；
- Audit pass 作为 Issue done 的一部分；
- 在同一 Codex thread 中同时执行交付和审计。

### Status

`accepted-draft`

## 12. ADR-010: Projection Surface Is Read-first

### Decision

Projection Surface 只读取事实投影，并通过 Runtime API 发起命令回流。

标准路径：

```text
Event Store
→ Projection
→ Read Model / View Model
→ Industry Surface
→ Runtime API command
→ Action Proposal
```

### Rationale

行业客户端需要不同现场呈现方式，但不应该拥有事实写入权。

Projection 解决的是“给人和行业客户端看什么”。
Runtime API 解决的是“下一步命令如何合法进入系统”。

### Consequences

- UI 不直接修改 Event Store；
- UI 不直接改对象状态；
- View Model 可以行业化；
- Command Surface 必须把操作转成 Action Proposal；
- Projection 可以重建，不能成为不可替代事实源。

### Rejected Alternatives

- UI 本地状态直接代表项目事实；
- 客户端直接写任务文件；
- 不同行业客户端各自实现状态逻辑；
- 让 Read Model 反向驱动 Event Store。

### Status

`accepted-draft`

## 13. ADR-011: Agent Role Is Capability Contract

### Decision

Agent Role 是能力合约，不是 prompt 文案。

每个 Agent Role 必须定义：

```text
canRead
canWrite
canExecute
mustProduce
cannotDo
toolScope
objectScope
handoffRules
```

### Rationale

Prompt 可以描述行为倾向，但不能作为权限系统。

AgentFlow 需要能回答：

- 这个 Agent 能读什么；
- 能写什么；
- 能执行什么；
- 必须产出什么证据；
- 绝对不能做什么；
- 什么时候必须交给其他角色。

这些都必须进入 Role Policy，供 Arbitration 使用。

### Consequences

- Build Agent、Spec Agent、Audit Agent 的权限必须分开；
- Role Policy 是 Action Arbitration 的输入；
- prompt 不能绕过 Role Policy；
- 后续 SPEC 必须定义核心 Agent Role Policy。

### Rejected Alternatives

- 只靠系统提示词约束 Agent；
- 让 Agent 自己判断能不能执行；
- 把权限散落在不同文档；
- 让 UI 按按钮来决定 Agent 权限。

### Status

`accepted-draft`

## 14. ADR-012: Message Bus Is Infrastructure, Not Fact Source

### Decision

Message Bus 是调度和唤醒设施，不是事实源，也不是 Runtime Core 的主链路中心。

### Rationale

事件驱动需要消息机制，但消息不是事实。
Message Bus 可以传递 command、event notification、wake signal、projection rebuild signal，但事实仍然必须在 Event Store。

如果把 Message Bus 当事实源，系统会失去 replay、audit、reconciliation 和 projection rebuild 的稳定基础。

### Consequences

- Message Bus 可以异步化执行；
- Message Bus 失败不应丢失事实；
- Event Store append 成功后才能广播事实通知；
- 消费者必须能从 Event Store 重建状态；
- 任何只存在于消息队列里的状态都不能作为项目事实。

### Rejected Alternatives

- 用消息队列保存项目状态；
- 用消息消费顺序代表事实顺序；
- 不落 Event Store，只靠实时事件通知驱动 UI；
- 把 Message Bus 画成核心架构中心。

### Status

`accepted-draft`

## 15. Decision Dependency Map

这些决策之间有明确依赖：

```text
ADR-001 Agent Project OS
→ ADR-003 Project Ontology
→ ADR-004 Runtime Core Main Chain
→ ADR-006 Action Proposal Model
→ ADR-007 Action Arbitration
→ ADR-005 Event Store
→ ADR-010 Projection Surface
```

并行支撑关系：

```text
ADR-002 Industry Product Shells
→ ADR-010 Projection Surface

ADR-008 Object State Machine
→ ADR-006 Action Proposal Model
→ ADR-007 Action Arbitration

ADR-009 Audit Independence
→ ADR-011 Agent Role Policy
→ ADR-007 Action Arbitration

ADR-012 Message Bus Boundary
→ ADR-005 Event Store
```

## 16. Architecture Guardrails

后续 SPEC 或实现如果违反以下规则，应视为架构偏离：

- 让 Agent 直接改对象状态；
- 让 UI 直接写事实；
- 让 Build Done 自动触发 Audit；
- 让 Audit Agent 修改 Build 交付事实；
- 让 Message Bus 成为事实源；
- 用 prompt 替代 Role Policy；
- 用一个全局状态字段表达所有对象生命周期；
- 把行业产品现场硬编码进 Runtime Core；
- 绕过 Project Ontology 直接执行行业输入；
- 绕过 Action Arbitration 并发写入。

## 17. Non-goals

本 ADR 不定义：

- 当前 v0.3.0 审计任务；
- `.agentflow/spec/**` 任务事实；
- 具体代码实现；
- 数据库 schema；
- UI 页面设计；
- 当前 Build Agent 执行授权；
- 正式 issue 拆分。

## 18. Next

这 6 份前置文档已经形成下一版本架构基线：

```text
1. Agent Project OS Architecture
2. Ontology Schema
3. Action Contract
4. Agent Role Policy
5. Object State Machine
6. Architecture Decision Record
```

下一步应该进入正式 SPEC 前的收敛动作：

- 合并重复概念；
- 标准化术语；
- 决定哪些内容进入下一版本 MVP 范围；
- 生成 SPEC Draft Preview；
- 等人类确认后，再写 `.agentflow/spec/**`。

收敛草案见：

- [AGENTFLOW_NEXT_VERSION_SPEC_CONVERGENCE_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_NEXT_VERSION_SPEC_CONVERGENCE_DRAFT_V1.md)
- [AGENTFLOW_RUNTIME_FOUNDATION_TECHNICAL_SUPPORT_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_RUNTIME_FOUNDATION_TECHNICAL_SUPPORT_DRAFT_V1.md)
