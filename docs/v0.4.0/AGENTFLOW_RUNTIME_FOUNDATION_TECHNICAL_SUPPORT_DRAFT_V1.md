# AgentFlow Runtime Foundation Technical Support Draft V1

日期：2026-06-19
执行者：Codex
用途：Next Version Development Preflight / Runtime Foundation 技术支撑草案
状态：Architecture Draft / 非执行需求 / 不进入当前 v0.3.0 审计 / 不授权 Build Agent 执行

关联架构基线：

- [AGENTFLOW_NEXT_VERSION_SPEC_CONVERGENCE_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_NEXT_VERSION_SPEC_CONVERGENCE_DRAFT_V1.md)
- [AGENTFLOW_AGENT_PROJECT_OS_ARCHITECTURE_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_AGENT_PROJECT_OS_ARCHITECTURE_V1.md)
- [AGENTFLOW_ARCHITECTURE_DECISION_RECORD_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_ARCHITECTURE_DECISION_RECORD_DRAFT_V1.md)
- [AGENTFLOW_ONTOLOGY_SCHEMA_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_ONTOLOGY_SCHEMA_DRAFT_V1.md)
- [AGENTFLOW_ACTION_CONTRACT_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_ACTION_CONTRACT_DRAFT_V1.md)
- [AGENTFLOW_AGENT_ROLE_POLICY_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_AGENT_ROLE_POLICY_DRAFT_V1.md)
- [AGENTFLOW_OBJECT_STATE_MACHINE_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_OBJECT_STATE_MACHINE_DRAFT_V1.md)

说明：本文件只把下一版本 Runtime Foundation 的技术支撑和 issues 落到根目录文档里。它不创建 `.agentflow/spec/**` 任务事实，不写 `docs/requirements/**`，不授权 Build Agent 执行。

## 1. Conclusion

下一版本的技术推进顺序应该是：

```text
Ontology Registry
→ Action Proposal / Action Contract
→ Agent Role Policy
→ Object State Machine
→ Action Arbitration
→ Event Store integration
→ Projection integration
```

第一步不是 Message Bus。
第一步是让 Runtime 先知道“项目世界里有什么对象、动作、状态和权限”。

## 2. Current Technical Base

当前 AgentFlow 已经有这些可复用底座：

| existing module | current role | next-version use |
| --- | --- | --- |
| `crates/spec` | spec project / issue contract | 继续作为正式 task contract fact source |
| `crates/event-store` | append-only task event stream、sequence、claim lease | 升级为 Runtime accepted events 的事实层 |
| `crates/workflow-core` | workflow definition / role / skill pack | 兼容旧 workflow，并给 state machine 迁移提供参考 |
| `crates/workflow-runtime` | guard / action / transition / checkpoint | 扩展或旁路为 Action Runtime / Arbitration 执行层 |
| `crates/task-loop` | issue scheduling and launch request | 后续改为通过 Action Proposal 启动 |
| `crates/agent-dispatcher` | provider session claim and launch | 继续作为 BuildAgent provider bridge |
| `crates/task-artifacts` | run / evidence / checkpoint artifacts | 继续作为 Run / Evidence / Artifact 的本地存储 |
| `crates/projection` | task/project/requirement projections | 升级为 ontology-aware read model builder |
| `crates/audit` | independent audit artifacts | 继续保持独立 Audit/Finding fact surface |
| `crates/state` | gates / indexes / health | 继续做聚合状态，不成为事实源 |
| `apps/desktop/src-tauri` | local Runtime command/query bridge | 后续只走 Runtime API，不直接写事实 |

## 3. Proposed Crate Shape

### 3.1 Add `crates/ontology`

职责：

- 定义 `ObjectTypeDefinition`
- 定义 `LinkTypeDefinition`
- 定义 `StateMachineDefinition`
- 定义 `OntologyBundle`
- 定义 `OntologyRegistry`
- 校验定义完整性
- 提供内置 `agentflow.core@v1-draft`

不负责：

- 写事件；
- 执行动作；
- 启动 Agent；
- 生成 Projection；
- 写 `.agentflow/spec/**`。

### 3.2 Add `crates/action-contract`

职责：

- 定义 `ActionTypeDefinition`
- 定义 `ActionContract`
- 定义 `ActionProposal`
- 定义 `ActionEffect`
- 定义 `RequiredEvidence`
- 校验 action input / precondition / effect / evidence

不负责：

- 判定 Agent 权限；
- 抢锁；
- 写 Event Store；
- 调用 provider。

### 3.3 Add `crates/role-policy`

职责：

- 定义 `AgentRolePolicy`
- 定义 `canRead / canWrite / canExecute / mustProduce / cannotDo`
- 提供 `SpecAgent / BuildAgent / AuditAgent / ReviewAgent / CoordinatorAgent / HumanOwner`
- 为 Arbitration 提供权限判断

不负责：

- prompt 生成；
- provider 能力探测；
- 直接调度 session。

### 3.4 Add `crates/action-arbitration`

职责：

- 接收 `ActionProposal`
- 读取 Ontology / Action Contract / Role Policy / State Machine / Event Store
- 校验对象状态、动作合法性、角色权限、证据、锁和依赖
- 返回 `ActionAccepted / ActionRejected / ActionQueued / HumanDecisionRequired`
- 对 accepted action 追加 Event Store 事件

不负责：

- 直接执行代码；
- 直接启动 provider；
- 直接写 UI projection；
- 直接写 audit report。

### 3.5 Reuse `crates/event-store`

MVP 不替换 Event Store。

需要扩展：

- event envelope 增加 `ontologyVersion`
- event envelope 增加 `actionType`
- event envelope 增加 `actionProposalId`
- accepted action 统一写入 `RuntimeActionAccepted` 或对象状态事件

### 3.6 Reuse `crates/projection`

Projection 需要从：

```text
spec contracts + task events
```

升级为：

```text
ontology definitions + event store + spec/audit/task artifacts
```

MVP 只做 read model，不做 UI 重构。

## 4. Runtime Flow

MVP 主链路：

```text
Runtime Command API
→ ActionProposal
→ ActionArbitration
→ OntologyRegistry
→ ActionContract
→ RolePolicy
→ ObjectStateMachine
→ EventStore.append
→ Projection.rebuild
```

关键规则：

- Command API 不直接改状态；
- Agent 不直接改状态；
- UI 不直接改状态；
- Action Proposal 不等于事实；
- 只有 accepted action 才能进入 Event Store；
- Projection 只读，不写事实。

## 5. Storage Boundary

MVP 本地存储建议：

| path | purpose |
| --- | --- |
| `.agentflow/ontology/**` | 后续正式实现的 ontology definitions |
| `.agentflow/actions/**` | 后续正式实现的 action contract definitions |
| `.agentflow/roles/**` | 后续正式实现的 role policy definitions |
| `.agentflow/events/**` | accepted runtime events |
| `.agentflow/projections/**` | read models |
| `.agentflow/tasks/**` | run / evidence / artifact |
| `.agentflow/audit/**` | independent audit facts |
| `.agentflow/spec/**` | confirmed task contracts |

注意：本文件不创建这些路径。
这些路径只有进入正式 SPEC 并确认后才能由后续 issue 实现。

## 6. Out Of Scope For MVP

第一版不做：

- 完整 Message Bus；
- 云端部署；
- 多租户；
- 多行业客户端；
- Domain Pack SDK；
- Surface Pack SDK；
- Connector Pack 市场；
- 分布式锁；
- 多 Agent 同时写同一对象；
- 自动冲突合并；
- 自动审计触发；
- Build Agent 写 audit report；
- 直接修改当前 v0.3.0 审计流程。

## 7. Issue Set

以下 issues 只写在本文档中，不写入 `.agentflow/spec/**`。

| issue | title | depends on | first executable |
| --- | --- | --- | --- |
| `AF-OS-001` | 收敛 Project Ontology Registry 与核心对象/关系定义 | none | yes |
| `AF-OS-002` | 定义 Action Type、Action Contract 与 Action Proposal schema | `AF-OS-001` | no |
| `AF-OS-003` | 定义 Agent Role Policy 与角色能力矩阵 | `AF-OS-001` | no |
| `AF-OS-004` | 定义 Requirement / Spec / Issue / Run / Audit / Finding 状态机 | `AF-OS-001`, `AF-OS-002` | no |
| `AF-OS-005` | 定义基础 Action Arbitration 与对象锁规则 | `AF-OS-002`, `AF-OS-003`, `AF-OS-004` | no |
| `AF-OS-006` | 定义 Event Store envelope、append 规则和 replay 边界 | `AF-OS-005` | no |
| `AF-OS-007` | 定义 Projection Read Models 与 Runtime Query API | `AF-OS-006` | no |
| `AF-OS-008` | 定义 Runtime Command API 与 UI/CLI 命令回流规则 | `AF-OS-005`, `AF-OS-007` | no |
| `AF-OS-009` | 对齐旧 workflow/event/capability 文档并形成迁移说明 | `AF-OS-006`, `AF-OS-007` | no |
| `AF-OS-010` | 完成 Runtime Foundation 集成验证与 SPEC closeout | `AF-OS-008`, `AF-OS-009` | no |

## 8. AF-OS-001

### Title

收敛 Project Ontology Registry 与核心对象/关系定义

### Goal

建立 Runtime Foundation 的共同语言，让后续 Action、Role、State、Event、Projection 都引用同一套对象和关系定义。

### Technical Scope

建议后续实现新增：

```text
crates/ontology
```

核心模型：

```text
OntologyBundle
OntologyRegistry
OntologyDefinitionRecord
ObjectTypeDefinition
LinkTypeDefinition
OntologyValidationReport
```

核心对象：

```text
Requirement
Spec
Project
Issue
Run
Evidence
Artifact
Decision
Audit
Finding
```

核心关系：

```text
derivesFrom
decomposesTo
contains
blocks
executes
produces
proves
reviews
requiresFix
decides
```

### Reuse

复用：

- `crates/spec` 的 SpecIssue / SpecProject 作为现有 contract 输入；
- `AGENTFLOW_ONTOLOGY_SCHEMA_DRAFT_V1.md` 的 schema；
- `AGENTFLOW_OBJECT_STATE_MACHINE_DRAFT_V1.md` 的对象列表。

### Deliverables

- Ontology Registry schema；
- core object definitions；
- core link definitions；
- validation checklist；
- 与现有 `spec` 模型的映射说明。

详细技术设计见：

- [AGENTFLOW_AF_OS_001_ONTOLOGY_REGISTRY_TECHNICAL_DESIGN_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_AF_OS_001_ONTOLOGY_REGISTRY_TECHNICAL_DESIGN_DRAFT_V1.md)

### Acceptance Criteria

- 每个核心对象都有唯一 `id`、`version`、`status`；
- 每个 link 的 source / target object type 存在；
- Ontology bundle 可被 Runtime 读取；
- deprecated / draft / active 状态语义清楚；
- 不需要 Action Contract 也能独立校验对象和关系。

### Forbidden

- 不实现 Action Arbitration；
- 不写 `.agentflow/spec/**`；
- 不改 Build Agent 执行链路；
- 不触发 Audit；
- 不改前端 UI。

## 9. AF-OS-002

详细技术设计：

- [AGENTFLOW_AF_OS_002_ACTION_CONTRACT_TECHNICAL_DESIGN_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_AF_OS_002_ACTION_CONTRACT_TECHNICAL_DESIGN_DRAFT_V1.md)

### Title

定义 Action Type、Action Contract 与 Action Proposal schema

### Goal

让 Runtime 能用统一格式接收“想做什么”，并知道动作的输入、前置条件、效果和证据要求。

### Technical Scope

建议后续实现新增：

```text
crates/action-contract
```

核心模型：

```text
ActionTypeDefinition
ActionContract
ActionProposal
ActionInputSchema
ActionPrecondition
ActionEffect
RequiredEvidence
RollbackRule
```

核心动作：

```text
submitRequirement
normalizeIntake
classifyRequirement
draftSpec
approveSpec
createIssue
activateIssue
startRun
submitEvidence
submitDelivery
markIssueDone
requestAudit
createFinding
linkFixIssue
```

### Dependencies

- `AF-OS-001`

### Reuse

复用：

- `AGENTFLOW_ACTION_CONTRACT_DRAFT_V1.md`
- `crates/workflow-runtime` 的 guard / action 概念
- `crates/event-store` 的 idempotency 思路

### Deliverables

- Action Contract schema；
- Action Proposal schema；
- core action definitions；
- action validation report；
- action 到 object type / state machine 的引用规则。

### Acceptance Criteria

- 每个写事实的动作都有 Action Contract；
- Action Proposal 不直接等于事实；
- 每个 Action Contract 声明 target object type；
- 每个 Action Contract 声明 required evidence；
- 每个 Action Contract 声明 accepted 后的 expected event。

### Forbidden

- 不做权限判断；
- 不写 Event Store；
- 不启动 Agent；
- 不改 Projection。

## 10. AF-OS-003

详细技术设计：

- [AGENTFLOW_AF_OS_003_AGENT_ROLE_POLICY_TECHNICAL_DESIGN_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_AF_OS_003_AGENT_ROLE_POLICY_TECHNICAL_DESIGN_DRAFT_V1.md)

### Title

定义 Agent Role Policy 与角色能力矩阵

### Goal

把 Agent 角色从 prompt 文案升级成 Runtime 可仲裁的能力合约。

### Technical Scope

建议后续实现新增：

```text
crates/role-policy
```

核心模型：

```text
AgentRolePolicy
RoleCapability
ObjectScope
ActionScope
ToolScope
HandoffRule
RolePolicyValidationReport
```

核心角色：

```text
SpecAgent
BuildAgent
AuditAgent
ReviewAgent
CoordinatorAgent
HumanOwner
```

### Dependencies

- `AF-OS-001`

### Reuse

复用：

- `AGENTFLOW_AGENT_ROLE_POLICY_DRAFT_V1.md`
- `docs/architecture/002-agent-capability-matrix-v1.md`
- `crates/spec` 中 `BuildAgent / AuditAgent` 的现有映射
- `crates/agent-dispatcher` 的 provider role binding

### Deliverables

- Role Policy schema；
- core role definitions；
- Build/Audit separation rules；
- role-action matrix；
- role-object matrix。

### Acceptance Criteria

- BuildAgent 不能 `draftSpec`、`approveSpec`、`createFinding`、`passAudit`；
- AuditAgent 不能修改 Build 交付事实；
- HumanOwner 可以确认、拒绝、裁决和重开；
- Role Policy 可被 Arbitration 读取；
- prompt 不能覆盖 Role Policy。

### Forbidden

- 不把 prompt 当权限系统；
- 不改 provider session 启动逻辑；
- 不放宽 Build Agent 边界；
- 不自动触发 Audit。

## 11. AF-OS-004

详细技术设计：

- [AGENTFLOW_AF_OS_004_OBJECT_STATE_MACHINE_TECHNICAL_DESIGN_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_AF_OS_004_OBJECT_STATE_MACHINE_TECHNICAL_DESIGN_DRAFT_V1.md)

### Title

定义 Requirement / Spec / Issue / Run / Audit / Finding 状态机

### Goal

把项目状态拆成对象生命周期，避免用一个 issue status 表达所有事实。

### Technical Scope

建议后续实现放在：

```text
crates/ontology
crates/action-contract
```

或后续单独新增：

```text
crates/object-state
```

核心状态机：

```text
requirement.state-machine
spec.state-machine
issue.state-machine
run.state-machine
audit.state-machine
finding.state-machine
```

### Dependencies

- `AF-OS-001`
- `AF-OS-002`

### Reuse

复用：

- `AGENTFLOW_OBJECT_STATE_MACHINE_DRAFT_V1.md`
- `crates/workflow-core` 的 state / transition 校验经验
- `docs/architecture/003-workflow-schema-v1.md` 的 pause/resume/retry/cancel 语义

### Deliverables

- object state machine schema；
- 6 个核心状态机定义；
- transition guard / actionType 引用；
- terminal state 规则；
- cross-object transition rules。

### Acceptance Criteria

- `Run.completed` 不等于 `Issue.done`；
- `Issue.done` 不自动创建 Audit；
- `Finding.fixRequired` 只能派生或链接修复 Issue；
- 每个 transition 引用已注册 Action Type；
- 状态事件可被 Event Store 记录。

### Forbidden

- 不继续扩大 workflow schema；
- 不把 Audit 合并进 Build；
- 不让 Agent 直接写最终状态；
- 不改现有 task-loop 行为。

## 12. AF-OS-005

详细技术设计：

- [AGENTFLOW_AF_OS_005_ACTION_ARBITRATION_TECHNICAL_DESIGN_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_AF_OS_005_ACTION_ARBITRATION_TECHNICAL_DESIGN_DRAFT_V1.md)

### Title

定义基础 Action Arbitration 与对象锁规则

### Goal

建立 Runtime 写入事实前的唯一仲裁入口。

### Technical Scope

建议后续实现新增：

```text
crates/action-arbitration
```

核心模型：

```text
ArbitrationRequest
ArbitrationDecision
ArbitrationContext
ObjectLock
ActionAccepted
ActionRejected
HumanDecisionRequired
```

MVP 仲裁检查：

```text
actionType exists
objectType exists
object state allows action
role canExecute action
required evidence exists
object lock available
dependencies satisfied
```

### Dependencies

- `AF-OS-002`
- `AF-OS-003`
- `AF-OS-004`

### Reuse

复用：

- `crates/event-store` 的 lock / claim lease 思路；
- `crates/task-loop` 的 dependency guard；
- `crates/workflow-runtime` 的 guard result；
- `crates/state` 的 gates / blockers 聚合。

### Deliverables

- Arbitration schema；
- decision enum；
- object lock rules；
- basic dependency checks；
- rejected reason taxonomy；
- accepted action event draft shape。

### Acceptance Criteria

- 所有写事实动作必须经过 Arbitration；
- accepted action 才能 append event；
- rejected action 必须给出 reason；
- Human decision required 必须可表达；
- 同一对象默认只允许一个 active write lock。

### Forbidden

- 不做分布式锁；
- 不做自动冲突合并；
- 不做跨项目事务；
- 不直接启动 provider；
- 不直接生成 Projection。

## 13. AF-OS-006

详细技术设计：

- [AGENTFLOW_AF_OS_006_EVENT_STORE_INTEGRATION_TECHNICAL_DESIGN_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_AF_OS_006_EVENT_STORE_INTEGRATION_TECHNICAL_DESIGN_DRAFT_V1.md)

### Title

定义 Event Store envelope、append 规则和 replay 边界

### Goal

把 accepted action 转换为可回放、可投影、可审计的事实事件。

### Technical Scope

建议后续扩展：

```text
crates/event-store
```

新增或明确字段：

```text
ontologyVersion
actionProposalId
actionType
objectType
objectId
fromState
toState
evidenceRefs
decision
```

### Dependencies

- `AF-OS-005`

### Reuse

复用：

- 现有 `TaskEvent`
- 现有 `ReplayFilter`
- 现有 `idempotencyKey`
- 现有 event store lock
- 现有 consumer / dead-letter 机制

### Deliverables

- Runtime event envelope；
- accepted action append rule；
- replay compatibility rule；
- event taxonomy；
- migration mapping from task-event.v2。

### Acceptance Criteria

- Event Store 仍是事实权威；
- 所有 accepted action 有 causation / correlation；
- replay 不调用 provider；
- replay 不改 spec contract；
- old task events 能被兼容读取。

### Forbidden

- 不替换 Event Store；
- 不引入 Message Bus 作为事实源；
- 不覆盖历史事件；
- 不让 Projection 写回事件。

## 14. AF-OS-007

详细技术设计：

- [AGENTFLOW_AF_OS_007_PROJECTION_READ_MODELS_TECHNICAL_DESIGN_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_AF_OS_007_PROJECTION_READ_MODELS_TECHNICAL_DESIGN_DRAFT_V1.md)

### Title

定义 Projection Read Models 与 Runtime Query API

### Goal

让 UI / CLI / 行业客户端只读 Projection，不直接读写 Runtime 事实。

### Technical Scope

建议后续扩展：

```text
crates/projection
apps/desktop/src-tauri/src/commands/projection.rs
```

核心 read models：

```text
RequirementIntakeView
SpecPreviewView
ProjectHomeView
TaskWorkbenchView
AuditSurfaceView
DeliveryPackageView
```

### Dependencies

- `AF-OS-006`

### Reuse

复用：

- 现有 task projection；
- 现有 project projection；
- 现有 requirement preview projection；
- 现有 audit / delivery summary projection。

### Deliverables

- Projection read model schema；
- Query API shape；
- ontology-aware projection mapping；
- compatibility mapping to current Desktop views。

### Acceptance Criteria

- Projection 只读 Event Store 和定义层；
- UI 不直接写事实；
- read model 能表达 current / past / future / exception；
- AuditSurfaceView 保持独立；
- DeliveryPackageView 不触发 Audit。

### Forbidden

- 不重做 Desktop UI；
- 不写 command side；
- 不直接读 legacy input/execute/output 作为权威；
- 不把 Projection 当事实源。

## 15. AF-OS-008

详细技术设计：

- [AGENTFLOW_AF_OS_008_RUNTIME_COMMAND_API_TECHNICAL_DESIGN_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_AF_OS_008_RUNTIME_COMMAND_API_TECHNICAL_DESIGN_DRAFT_V1.md)

### Title

定义 Runtime Command API 与 UI/CLI 命令回流规则

### Goal

把 UI / CLI / Agent 的写意图统一转成 Action Proposal，而不是直接写状态。

### Technical Scope

建议后续新增或扩展：

```text
crates/runtime-api
apps/desktop/src-tauri/src/commands/runtime.rs
crates/cli
```

核心 Command API：

```text
submitRequirement
approveSpec
createIssue
startRun
submitEvidence
submitDelivery
markIssueDone
requestAudit
createFinding
linkFixIssue
```

### Dependencies

- `AF-OS-005`
- `AF-OS-007`

### Reuse

复用：

- Desktop Tauri commands；
- CLI command patterns；
- `task-loop` start issue / schedule issue capability；
- `audit` request paths。

### Deliverables

- Command API boundary；
- Command to ActionProposal mapping；
- UI/CLI forbidden direct-write list；
- Runtime response model。

### Acceptance Criteria

- Command API 只生成 Action Proposal；
- Action Proposal 必须走 Arbitration；
- Command response 返回 accepted/rejected/humanDecisionRequired；
- UI 不直接改 `.agentflow/spec/**` 或 `.agentflow/events/**`；
- CLI 不绕过 Runtime。

### Forbidden

- 不重写所有 Tauri commands；
- 不迁移全部 CLI；
- 不启动 Build Agent；
- 不自动创建 Audit。

## 16. AF-OS-009

详细技术设计：

- [AGENTFLOW_AF_OS_009_MIGRATION_ALIGNMENT_TECHNICAL_DESIGN_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_AF_OS_009_MIGRATION_ALIGNMENT_TECHNICAL_DESIGN_DRAFT_V1.md)

### Title

对齐旧 workflow/event/capability 文档并形成迁移说明

### Goal

把旧文档里的 Workflow / Capability / Event / Projection 术语迁移到 Runtime Foundation 语言，避免下一版出现两套模型。

### Technical Scope

需要对齐：

```text
docs/architecture/001-project-operating-system-v1.md
docs/architecture/002-agent-capability-matrix-v1.md
docs/architecture/003-workflow-schema-v1.md
docs/architecture/004-event-and-projection-model-v1.md
docs/architecture/current-module-boundaries.md
```

### Dependencies

- `AF-OS-006`
- `AF-OS-007`

### Reuse

复用：

- `AGENTFLOW_NEXT_VERSION_SPEC_CONVERGENCE_DRAFT_V1.md` 的 terminology mapping；
- `AGENTFLOW_ARCHITECTURE_DECISION_RECORD_DRAFT_V1.md` 的 guardrails。

### Deliverables

- old-to-new terminology map；
- module responsibility migration map；
- compatibility notes；
- deprecated concept list；
- next-version documentation update plan。

### Acceptance Criteria

- `Work Agent` 映射到 `BuildAgent` 或兼容别名；
- `Workflow State` 映射到 Object State Machine / Projection State；
- `AuditFinding` 映射到 `Finding`；
- Event model 与 new envelope 不冲突；
- current module boundaries 不被推翻，只被升级。

### Forbidden

- 不删除旧文档；
- 不改源码；
- 不写正式 requirements；
- 不写 `.agentflow/spec/**`。

## 17. AF-OS-010

详细技术设计：

- [AGENTFLOW_AF_OS_010_RUNTIME_FOUNDATION_CLOSEOUT_TECHNICAL_DESIGN_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_AF_OS_010_RUNTIME_FOUNDATION_CLOSEOUT_TECHNICAL_DESIGN_DRAFT_V1.md)

### Title

完成 Runtime Foundation 集成验证与 SPEC closeout

### Goal

在正式进入 Build Agent 前，确认 Runtime Foundation 的 issue 设计、依赖、边界和验收是闭合的。

### Technical Scope

验证对象：

```text
Ontology definitions
Action contracts
Role policies
State machines
Arbitration rules
Event envelope
Projection read models
Runtime API boundary
Migration notes
```

### Dependencies

- `AF-OS-008`
- `AF-OS-009`

### Reuse

复用：

- `crates/acceptance` 的 acceptance 思路；
- `checks/agentflow-readiness.sh` 的 readiness 风格；
- `verification.md` 的本地验证记录方式。

### Deliverables

- integration checklist；
- validation plan；
- issue dependency sanity check；
- spec closeout summary；
- explicit Build Agent entry recommendation。

### Acceptance Criteria

- 所有 issue 依赖闭合；
- 第一条可执行 issue 明确；
- Build/Audit 边界没有混入；
- `.agentflow/spec/**` 写入仍需人类确认；
- Runtime Foundation MVP 范围没有膨胀到行业客户端。

### Forbidden

- 不执行 Build Agent；
- 不跑项目构建；
- 不创建 PR；
- 不关闭当前 v0.3.0 审计；
- 不把 closeout 当作实现完成。

## 18. Recommended Execution Order

推荐顺序：

```text
AF-OS-001
→ AF-OS-002
→ AF-OS-003
→ AF-OS-004
→ AF-OS-005
→ AF-OS-006
→ AF-OS-007
→ AF-OS-008
→ AF-OS-009
→ AF-OS-010
```

可并行但不建议第一版并行：

```text
AF-OS-002 and AF-OS-003 can be drafted after AF-OS-001
```

原因：

- Action Contract 和 Role Policy 都依赖 Ontology；
- Arbitration 必须等 Action / Role / State 三者齐；
- Event Store integration 必须等 Arbitration 决定 accepted action；
- Projection 必须等 Event envelope 稳定。

## 19. Confirmation Boundary

本文档只是工程边界草案。

下一步如果进入正式 SPEC，必须先由人类确认，再写：

```text
docs/requirements/<requirement-id>.md
.agentflow/spec/projects/<project-id>.json
.agentflow/spec/issues/<issue-id>.json
```

未确认前：

- 不写 `.agentflow/spec/**`；
- 不写 `docs/requirements/**`；
- 不执行 Build Agent；
- 不跑构建；
- 不改源码。
