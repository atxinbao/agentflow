# AgentFlow Agent Project OS Architecture v1

日期：2026-06-19
执行者：Codex
用途：Next Version Development Preflight / 下一版本开发前置架构文档
状态：Architecture Draft / 非执行需求 / 不进入当前 v0.3.0 审计 / 不授权 Build Agent 执行

参考图：

![AgentFlow Agent Project OS God View v2](/Users/mac/Downloads/agentflow-agent-project-os-god-view-v2-zh.png)

说明：本文件沉淀在项目根目录，用作后续正式 SPEC 之前的架构基线。它不创建 `.agentflow/spec/**` 任务事实，不授权 Build Agent 执行，也不替代当前版本审计流程。

## 1. 结论

AgentFlow 的目标不应该只是一个 Agent 工作流系统，而应该是一套 Agent Project OS。

它的核心不是“让多个 Agent 自动干活”，而是先定义一个稳定的项目世界：

- 有哪些项目对象
- 对象之间如何关联
- 哪些动作合法
- 哪些 Agent 可以执行动作
- 动作发生后必须留下什么事实和证据
- 事实如何被投影成不同的行业客户端

因此，AgentFlow Agent Project OS 的主线是：

```text
Industry Input Boundary
→ Standardization Boundary
→ Runtime Core
→ Projection Surface Output
```

更具体地说：

```text
行业输入
→ Project Ontology 标准化
→ Runtime Core 执行与仲裁
→ Event Store 写事实
→ Projection Surface 输出与命令回流
```

## 2. Core Principles

### 2.1 输入可变，核心不变

不同行业的现场完全不同。

软件开发、视频制作、前端设计、金融运营、制造现场，它们的对象、页面、工具、交付物都不同。AgentFlow 不应该把所有行业硬塞进同一个客户端壳。

正确模型是：

```text
底层 Runtime 复用
行业产品壳分开设计
Domain Pack / Surface Pack / Connector Pack 分开适配
```

### 2.2 行业输入必须先标准化

Runtime Core 不直接理解行业现场。

外部输入必须先被翻译成 AgentFlow 的通用项目语言：

```text
Object Types
Link Types
Action Semantics
State Machines
Agent Role Policy
Version / Migration
Simulation Rules
```

这层就是 Project Ontology Layer。

### 2.3 Agent 不直接改状态

多 Agent 执行的核心规则是：

```text
Agent proposes Action
Runtime validates Action
Ontology decides legality
Event Store records accepted facts
Projection updates output surfaces
```

Agent 只提交 Action Proposal。
状态变化必须由 Action Type + State Machine + Capability Policy 决定。

### 2.4 Event Store 是事实权威

Event Store 是唯一事实权威。

Message Bus 只负责唤醒和调度。
Projection 只负责展示。
Output Surface 只负责读取投影、打包证据、发起合法命令。

### 2.5 输出不是终点

输出层不是静态结果页面。

输出层是 Projection Surface：

```text
read
deliver
audit
decide
command
feedback
```

人类下一步操作也必须回到 Runtime API，再进入 Spec Loop 或 Arbitration。

## 3. System Boundaries

## 3.1 Industry Input Boundary

职责：收集行业现场输入，但不定义 Runtime Core。

输入来源包括：

- Human Intent：需求、问题、目标、确认、反馈
- Domain Pack：行业对象、术语、业务规则
- Surface Pack：现场页面状态、交互语义
- Connector Pack：外部系统能力、工具、数据源
- Governance + Facts：人类授权边界、历史事件、当前状态
- Runtime API / SDK：Command、Query、Event 的标准入口

关键规则：

```text
行业现场可以变化
输入结构可以变化
但进入 Runtime 前必须被标准化
```

## 3.2 Standardization Boundary

职责：把行业输入翻译成 AgentFlow 通用项目语言。

主要组成：

- Ontology Registry
- Object / Link Types
- Action Semantics
- State Machines
- Agent Role Policy
- Version / Migration
- Simulation Rules
- Executable Definitions

输出结果：

```text
Project Ontology = 通用项目语言
```

## 3.3 Runtime Core

职责：执行 AgentFlow 的内部稳定主链路。

主链路：

```text
Spec Loop
→ Contract
→ Build Loop
→ Agent Action Arbitration
→ Event Store
→ Projection
```

Runtime Core 不应该跟行业客户端耦合。它只读取标准化后的 Project Ontology，并按定义执行。

## 3.4 Projection Surface Output

职责：把 Runtime Core 的事实和状态投影给行业客户端。

输出层包括：

- Projection API
- Read Models
- View Models
- Industry Workbench
- Delivery Package
- Evidence Graph
- Audit Surface
- Command Surface
- Feedback Loop

关键规则：

```text
Output Surface 不写事实
Output Surface 不绕过 Runtime
Output Surface 发起的命令必须回到 Runtime API
```

## 4. Project Ontology Layer

Project Ontology Layer 是 AgentFlow 从工作流系统升级为 Project OS 的关键。

它不是说明文档，也不是普通数据库 schema。它是 Runtime 可以读取、验证和执行的项目世界定义层。

### 4.1 Ontology Registry

所有定义都必须注册。

最小字段：

```text
namespace
id
version
status
owner
compatibility
deprecation policy
```

它解决的问题：

- 定义不散落
- 定义可版本化
- 旧项目可迁移
- Event replay 有兼容依据

### 4.2 Object Types

定义项目世界里有什么对象。

候选对象：

```text
Requirement
Spec
Project
Issue
Task
Run
Evidence
Artifact
Decision
AuditFinding
AgentRole
```

### 4.3 Link Types

定义对象之间如何关联。

候选关系：

```text
derivesFrom
decomposesTo
blocks
supersedes
executes
produces
proves
reviews
requiresFix
```

### 4.4 Action Semantics

每个 Action 不能只写名字，必须定义可执行语义。

最小结构：

```text
action id
input schema
target object
precondition
effect
required evidence
cancel rule
rollback or compensation rule
emitted events
```

示例：

```text
approveSpec
startRun
submitEvidence
markDone
requestAudit
acceptDelivery
requestFix
reopenIssue
createFollowUp
```

### 4.5 Object State Machines

Project OS 不应该用一条线性流程描述所有对象。

应该拆成对象状态机：

```text
Requirement State Machine
Spec State Machine
Work Package State Machine
Issue State Machine
Run State Machine
Audit State Machine
Finding State Machine
```

关键变化：

```text
不是 Requirement → Spec → Issue → Run → Done
而是 Object State Machine + Link Types + Action Types
```

### 4.6 Agent Role Contracts

Agent Role 不是 prompt。Agent Role 必须变成能力合约。

最小结构：

```text
canRead
canWrite
canExecute
mustProduce
cannotDo
allowedTools
requiredEvidence
handoffRules
```

示例：

```text
Spec Agent
  canExecute: classifyRequirement / draftSpec / generateWorkPackage
  cannotDo: implementCode / writeAuditReport

Build Agent
  canExecute: startRun / submitEvidence / requestReview
  cannotDo: createAuditFinding / bypassSpec

Audit Agent
  canExecute: inspectEvidence / createFinding / passAudit
  cannotDo: modifyDelivery / executeBuild
```

### 4.7 Capability Policy

Capability Policy 是 Agent Action Arbitration 的依据。

它定义：

```text
role → allowed actions
role → object scope
role → tool scope
role → approval gate
role → required evidence
```

### 4.8 Versioning / Migration

Project Ontology 会升级。

必须定义：

```text
schema version
definition version
event replay compatibility
migration rule
deprecation rule
legacy object mapping
```

否则旧项目无法 replay，新 Runtime 也无法安全读取旧事件。

### 4.9 Evaluation / Simulation

真实写入前必须支持 simulation。

Simulation 回答：

```text
这个 Action 如果执行，会改哪些对象？
需要哪些证据？
会触发哪些状态变化？
是否和其他 Agent 的 Action 冲突？
是否违反 Capability Policy？
```

## 5. Runtime Core

Runtime Core 是内部稳定主链路。

```text
Spec Loop
→ Contract
→ Build Loop
→ Agent Action Arbitration
→ Event Store
→ Projection
```

### 5.1 Spec Loop

职责：理解输入、分类需求、生成 preview。

它处理：

```text
Raw Input
Requirement Intake
Requirement Classified
Route Decided
Spec Draft
Spec Approved
Work Package Created
```

它不直接执行代码，也不绕过确认。

### 5.2 Contract

职责：把意图固化成可执行边界。

Contract 包含：

```text
scope
allowed surface
acceptance criteria
required evidence
agent role authorization
execution pipeline
```

### 5.3 Build Loop

职责：让 Agent 执行已授权的工作。

Build Loop 不直接写最终状态。
它产出 Action Proposal 和 Evidence。

### 5.4 Agent Action Arbitration

职责：多 Agent 动作仲裁。

它检查：

```text
precondition
capability policy
object lock
dependency
state machine transition
evidence requirement
conflict
```

输出：

```text
ActionAccepted
ActionRejected
ConflictDetected
Blocked
EvidenceRequired
```

只有 ActionAccepted 才能进入 Event Store。

### 5.5 Event Store

职责：事实权威。

它记录：

```text
ActionSubmitted
ActionAccepted
ActionRejected
StateChanged
EvidenceAttached
FindingCreated
DecisionRecorded
```

### 5.6 Projection

职责：从 Event Store + Project Ontology 生成可读状态。

Projection 不写事实。
Projection 生成 Read Models 和 View Models。

## 6. Projection Surface Output

输出层要从“展示结果”升级为 Projection Surface。

### 6.1 Projection API

提供：

```text
query read model
subscribe state changes
fetch evidence graph
export delivery package
```

### 6.2 Read Models

Read Models 是从事实投影出来的查询模型。

示例：

```text
project status
issue index
run status
evidence index
audit finding index
delivery readiness
```

### 6.3 View Models

View Models 是行业客户端可直接使用的页面状态。

示例：

```text
software project workbench
video production timeline
frontend design board
operations case queue
```

### 6.4 Delivery Package

Delivery Package 不只是交付物列表。

应该包含：

```text
artifact
evidence
decision log
verification result
accepted scope
definition version
```

### 6.5 Evidence Graph

Evidence Graph 是可审计证据链。

示例：

```text
requirement
→ spec
→ issue
→ run
→ evidence
→ finding
→ decision
```

### 6.6 Audit Surface

Audit Surface 是独立审计视角。

它可以展示：

```text
finding
risk
proof
traceability
evidence map
```

但它不能修改交付事实。

### 6.7 Command Surface

Command Surface 允许人类继续驱动项目。

示例 command：

```text
acceptDelivery
requestFix
requestAudit
reopenIssue
createFollowUp
approveSpec
rejectDelivery
```

关键规则：

```text
UI command 不直接改状态
Command → Runtime API → Spec Loop / Arbitration
ActionAccepted 后才进入 Event Store
```

## 7. Industry Product Model

行业产品不是插件列表。

正确模型是：

```text
Industry AgentFlow Product
= Industry Product Shell
+ Domain Pack
+ Surface Pack
+ Connector Pack
+ Projection Surface
```

底层复用：

```text
Agent Project Runtime
Project Ontology Layer
Agent Action Arbitration
Event Store
Projection API
Command API
```

行业差异放在：

```text
Domain Pack
Surface Pack
Connector Pack
Output Surface
```

## 8. Deployment Model

AgentFlow Agent Project OS 可以支持多种部署形态。

```text
Local Runtime
Cloud Runtime
Team Runtime
API / SDK
Connector / MCP
Projection API
Command API
Import / Export
```

部署原则：

```text
Runtime Core 保持一致
Project Ontology 可版本化迁移
Event Store 可 replay
行业产品通过 API 接入
```

## 9. Non-goals

当前架构不做以下事情：

- 不把所有行业塞进同一个客户端壳
- 不让 Agent 直接写最终状态
- 不让 UI 直接修改事实
- 不把 Audit 作为 Build Done 的默认尾巴
- 不把 Message Bus 当事实源
- 不把 prompt 当 Agent Role Contract
- 不把 Project Ontology 当普通说明文档

## 10. Next Work

下一步不应该继续扩图，而应该开始落可执行定义规范。

建议先形成四份规格草案和一份架构决策记录：

```text
ontology.schema
action.contract
agent-role.policy
object.state-machine
architecture.decision-record
```

当前前置草案文件：

- [AGENTFLOW_ONTOLOGY_SCHEMA_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_ONTOLOGY_SCHEMA_DRAFT_V1.md)
- [AGENTFLOW_ACTION_CONTRACT_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_ACTION_CONTRACT_DRAFT_V1.md)
- [AGENTFLOW_AGENT_ROLE_POLICY_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_AGENT_ROLE_POLICY_DRAFT_V1.md)
- [AGENTFLOW_OBJECT_STATE_MACHINE_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_OBJECT_STATE_MACHINE_DRAFT_V1.md)
- [AGENTFLOW_ARCHITECTURE_DECISION_RECORD_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_ARCHITECTURE_DECISION_RECORD_DRAFT_V1.md)
- [AGENTFLOW_NEXT_VERSION_SPEC_CONVERGENCE_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_NEXT_VERSION_SPEC_CONVERGENCE_DRAFT_V1.md)

状态机草案覆盖：

```text
requirement.state-machine
spec.state-machine
issue.state-machine
run.state-machine
audit.state-machine
finding.state-machine
```

推荐下一阶段交付物：

```text
1. Project Ontology schema draft
2. Action Contract schema draft
3. Agent Role Policy draft
4. Object State Machine draft
5. Architecture decision record
```

这些完成后，再根据 [AGENTFLOW_NEXT_VERSION_SPEC_CONVERGENCE_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_NEXT_VERSION_SPEC_CONVERGENCE_DRAFT_V1.md) 决定是否进入 AgentFlow 正式 SPEC。
