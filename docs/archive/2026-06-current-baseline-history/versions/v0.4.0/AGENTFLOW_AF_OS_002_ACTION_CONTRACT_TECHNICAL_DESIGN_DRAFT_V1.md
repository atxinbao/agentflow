# AF-OS-002 Action Contract Technical Design Draft V1

日期：2026-06-19
执行者：Codex
用途：Next Version Development Preflight / AF-OS-002 技术设计草案
状态：Architecture Draft / 非执行需求 / 不进入当前 v0.3.0 审计 / 不授权 Build Agent 执行

关联文档：

- [AGENTFLOW_RUNTIME_FOUNDATION_TECHNICAL_SUPPORT_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_RUNTIME_FOUNDATION_TECHNICAL_SUPPORT_DRAFT_V1.md)
- [AGENTFLOW_ACTION_CONTRACT_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_ACTION_CONTRACT_DRAFT_V1.md)
- [AGENTFLOW_AF_OS_001_ONTOLOGY_REGISTRY_TECHNICAL_DESIGN_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_AF_OS_001_ONTOLOGY_REGISTRY_TECHNICAL_DESIGN_DRAFT_V1.md)
- [AGENTFLOW_OBJECT_STATE_MACHINE_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_OBJECT_STATE_MACHINE_DRAFT_V1.md)
- [AGENTFLOW_AGENT_ROLE_POLICY_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_AGENT_ROLE_POLICY_DRAFT_V1.md)
- [AGENTFLOW_NEXT_VERSION_SPEC_CONVERGENCE_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_NEXT_VERSION_SPEC_CONVERGENCE_DRAFT_V1.md)

说明：本文件只展开 `AF-OS-002` 的技术设计。它不创建 `.agentflow/spec/**` 任务事实，不写 `docs/requirements/**`，不修改源码，也不授权 Build Agent 执行。

## 1. Conclusion

`AF-OS-002` 的目标是把所有写入意图统一成 `Action Proposal`，再用 `Action Contract` 判断这个意图是否结构合法。

核心规则：

```text
Agent / UI / CLI 不直接写项目状态
Agent / UI / CLI 只提交 Action Proposal
Action Contract 定义动作的输入、目标、前置条件、效果和证据要求
后续 Arbitration 决定是否接受 Proposal
Event Store 只记录 accepted action 后产生的事实事件
```

这一层不是权限系统，也不是事件存储。它是 Runtime 能理解“你想做什么”的标准动作语言。

## 2. Problem

当前 AgentFlow 已经有 Spec、Issue、Task、Evidence、Audit 等流程事实，但“写动作”的语义仍然分散在：

- 人类对话；
- Spec Agent / Build Agent / Audit Agent handoff 文案；
- workflow transition；
- task loop 命令；
- projection 和 desktop UI 的状态展示；
- `.agentflow/spec/**` 与 `.agentflow/tasks/**` 的文件结构。

如果没有 Action Contract，系统会继续遇到这些问题：

- Agent 可以用自然语言声称状态完成，但 Runtime 无法判定动作是否合规；
- UI / CLI / Agent 对同一状态变化使用不同输入格式；
- required evidence 只存在于文档约定，无法被底层统一校验；
- 多 Agent 并发时无法先判断动作目标和冲突范围；
- Build Done、Audit Request、Finding Fix 之间的边界容易被写穿；
- 行业客户端无法复用统一动作模型。

## 3. Scope

### 3.1 In Scope

`AF-OS-002` 应覆盖：

- `crates/action-contract` crate 设计；
- Action Type definition；
- Action Contract schema；
- Action Proposal schema；
- Action input schema；
- Action precondition model；
- Action effect model；
- required evidence model；
- expected event model；
- proposal validation report；
- core action definitions；
- 与 `AF-OS-001` Ontology object / link definitions 的引用关系；
- 与后续 State Machine / Role Policy / Arbitration / Event Store 的接口边界。

### 3.2 Out Of Scope

`AF-OS-002` 不做：

- Agent Role Policy 判权；
- Action Arbitration 接受或拒绝；
- Object lock；
- Event Store append；
- Projection update；
- Runtime Command API；
- Desktop UI；
- Build Agent 执行；
- Audit Agent 执行；
- Message Bus；
- 云端调度。

## 4. Proposed Crate

建议新增：

```text
crates/action-contract
```

建议模块：

```text
crates/action-contract/src/lib.rs
crates/action-contract/src/model.rs
crates/action-contract/src/registry.rs
crates/action-contract/src/validation.rs
crates/action-contract/src/core.rs
crates/action-contract/src/report.rs
```

职责划分：

| module | responsibility |
| --- | --- |
| `model.rs` | Action Type / Contract / Proposal / Validation schema |
| `registry.rs` | action contract lookup and version selection |
| `validation.rs` | proposal structure validation against contract |
| `core.rs` | built-in `agentflow.actions.core@v1-draft` definitions |
| `report.rs` | validation report and error taxonomy |
| `lib.rs` | public exports |

## 5. Dependency On AF-OS-001

`AF-OS-002` 必须依赖 Ontology Registry。

原因：

```text
Action Contract 不能自己发明对象类型
Action Contract 的 targetObjectType 必须来自 Ontology
Action Contract 的 expected links 必须来自 Ontology
Action Proposal 的 targetObjectRef 必须能被 Ontology 校验
```

第一版只需要读 `AF-OS-001` 提供的 core object / link definitions，不需要动态读 `.agentflow/ontology/**`。

## 6. Core Model

### 6.1 ActionTypeDefinition

`ActionTypeDefinition` 是动作的基础定义。

建议字段：

```text
id
namespace
version
status
name
description
category
targetMode
targetObjectType
createsObjectType
contractRef
```

`category` 候选：

```text
intake
spec
planning
execution
evidence
delivery
audit
finding
decision
```

`targetMode` 候选：

```text
existingObject
createObject
linkObjects
recordDecision
```

说明：

- `existingObject` 表示动作作用于已有对象，例如 `markIssueDone`；
- `createObject` 表示动作创建对象，例如 `submitRequirement`；
- `linkObjects` 表示动作建立对象关系，例如 `linkFixIssue`；
- `recordDecision` 表示动作记录人类确认、拒绝或裁决。

### 6.2 ActionContract

`ActionContract` 定义动作是否合法。

建议字段：

```text
id
actionType
namespace
version
status
target
inputSchema
preconditions
stateTransitionRef
effects
requiredEvidence
expectedEvents
expectedLinks
idempotency
conflictScopeHint
approvalHint
rollbackHint
simulationHint
```

注意：

- `approvalHint` 只是声明这个动作可能需要人类确认；
- 是否满足人类确认由后续 Arbitration 判断；
- `conflictScopeHint` 只是描述冲突范围；
- 是否抢锁由后续 Arbitration 判断；
- `stateTransitionRef` 只引用状态机；
- 状态机 legality 由 `AF-OS-004` 具体实现。

### 6.3 ActionProposal

`ActionProposal` 是 Agent / UI / CLI 提交给 Runtime 的动作意图。

建议字段：

```text
proposalId
idempotencyKey
actionType
actorRole
sourceSurface
targetObjectRef
input
evidenceRefs
artifactRefs
reason
expectedEffects
ontologyVersion
contractVersion
createdAt
```

`sourceSurface` 候选：

```text
conversation
desktop
cli
sdk
agent
system
```

`targetObjectRef` 在 `createObject` 动作中可以为空，但必须由 input 提供创建对象所需字段。

### 6.4 ActionInputSchema

MVP 不需要完整 JSON Schema 引擎，但必须有结构化字段定义。

建议字段：

```text
fields
requiredFields
allowAdditionalFields
```

字段定义：

```text
name
valueType
required
description
enumValues
objectTypeRef
linkTypeRef
```

`valueType` 候选：

```text
string
number
boolean
objectRef
objectRefList
evidenceRef
evidenceRefList
artifactRef
artifactRefList
timestamp
enum
structuredObject
```

### 6.5 ActionPrecondition

建议字段：

```text
id
kind
description
expression
requiredState
requiredLink
requiredEvidenceType
```

`kind` 候选：

```text
targetExists
targetStateIs
linkExists
linkAbsent
dependencySatisfied
evidenceExists
humanDecisionExists
```

注意：`AF-OS-002` 可以定义 precondition，但不负责读取真实项目状态。真实状态检查由后续 Arbitration 结合 Event Store / Projection 完成。

### 6.6 ActionEffect

建议字段：

```text
id
kind
description
objectType
linkType
stateTransitionRef
eventType
```

`kind` 候选：

```text
createObject
changeState
attachEvidence
attachArtifact
createLink
recordDecision
emitEvent
```

### 6.7 RequiredEvidence

建议字段：

```text
evidenceType
required
minCount
acceptedRefKind
description
```

`evidenceType` 第一版候选：

```text
requirementIntakeResult
humanConfirmation
specDraftPreview
specApproval
implementationSummary
verificationLog
artifactSummary
auditReport
findingRecord
fixEvidence
```

### 6.8 ExpectedEvent

建议字段：

```text
eventType
objectType
required
payloadFields
```

`AF-OS-002` 只定义 expected event，不写 Event Store。

## 7. Core Action Set

第一版 core actions 必须对齐 `AF-OS-001` 的 10 个核心对象：

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

明确不把以下对象作为 MVP action target：

```text
WorkPackage
Delivery
DeliveryPackage
AuditFinding
```

原因：

- `WorkPackage` 已在 `AF-OS-001` 暂缓；
- `Delivery` 暂作为 Projection / DeliveryPackage view；
- `AuditFinding` 统一命名为 `Finding`；
- 第一版需要稳定核心链路，不能扩成完整产品对象图。

建议 core actions：

| action type | target mode | target / creates | purpose |
| --- | --- | --- | --- |
| `submitRequirement` | `createObject` | creates `Requirement` | 记录原始需求输入 |
| `normalizeRequirement` | `existingObject` | `Requirement` | 清洗和标准化需求 |
| `classifyRequirement` | `existingObject` | `Requirement` | 分类需求类型和边界 |
| `draftSpec` | `existingObject` | `Requirement` -> creates `Spec` | 生成 SPEC 草案 |
| `approveSpec` | `existingObject` | `Spec` | 人类确认 SPEC |
| `createProject` | `existingObject` | `Spec` -> creates `Project` | 从 SPEC 创建项目聚合 |
| `createIssue` | `existingObject` | `Project` -> creates `Issue` | 从项目派生可执行 Issue |
| `activateIssue` | `existingObject` | `Issue` | 标记 Issue 可执行 |
| `startRun` | `existingObject` | `Issue` -> creates `Run` | 启动一次 Agent Run |
| `submitEvidence` | `existingObject` | `Run` -> creates `Evidence` | 提交执行证据 |
| `submitArtifact` | `existingObject` | `Run` -> creates `Artifact` | 提交产物引用 |
| `markIssueDone` | `existingObject` | `Issue` | 标记 Issue 完成 |
| `recordDecision` | `recordDecision` | creates `Decision` | 记录人类确认、拒绝、裁决 |
| `requestAudit` | `existingObject` | `Issue` -> creates `Audit` | 发起独立审计 |
| `createFinding` | `existingObject` | `Audit` -> creates `Finding` | 创建审计发现 |
| `linkFixIssue` | `linkObjects` | `Finding` -> `Issue` | 将 Finding 连接到修复 Issue |

## 8. Action Categories

### 8.1 Intake Actions

```text
submitRequirement
normalizeRequirement
classifyRequirement
```

这些动作只处理需求对象，不创建 Issue，不授权 Build Agent。

### 8.2 Spec Actions

```text
draftSpec
approveSpec
```

`approveSpec` 必须要求 `humanConfirmation` 证据。

### 8.3 Planning Actions

```text
createProject
createIssue
activateIssue
```

这些动作只生成或激活任务契约，不代表执行完成。

### 8.4 Execution Actions

```text
startRun
submitEvidence
submitArtifact
markIssueDone
```

`markIssueDone` 必须要求 verification / artifact / acceptance mapping 相关证据。

### 8.5 Audit Actions

```text
requestAudit
createFinding
linkFixIssue
```

`requestAudit` 必须保持独立触发。
`markIssueDone` 不允许自动触发 `requestAudit`。

## 9. Proposal Validation Pipeline

`AF-OS-002` 的 validation 只做结构合法性检查。

建议顺序：

```text
1. action type exists
2. action contract version exists
3. proposal actionType matches contract
4. target mode is valid
5. target object type exists in Ontology
6. created object type exists in Ontology
7. input fields match ActionInputSchema
8. required input fields are present
9. evidence ref shape is valid
10. expected links reference valid Ontology link types
11. expected events are declared
12. idempotency key shape is valid
```

不在本 issue 检查：

```text
actor role permission
object lock availability
live object state
dependency completion
evidence file existence
event append success
projection update success
```

这些分别属于：

- `AF-OS-003` Role Policy；
- `AF-OS-004` Object State Machine；
- `AF-OS-005` Action Arbitration；
- `AF-OS-006` Event Store；
- `AF-OS-007` Projection。

## 10. Validation Report

建议模型：

```text
ActionProposalValidationReport
  proposalId
  actionType
  contractVersion
  status
  errors
  warnings
  normalizedProposal
```

`status` 候选：

```text
valid
invalid
unsupported
versionMismatch
```

错误类型：

```text
unknownActionType
unknownContractVersion
contractRetired
invalidTargetMode
unknownTargetObjectType
unknownCreatedObjectType
invalidInputField
missingRequiredInput
unknownEvidenceRefKind
unknownExpectedLinkType
missingExpectedEvent
invalidIdempotencyKey
```

注意：这里不返回 `ActionAccepted` 或 `ActionRejected`。
那是 Arbitration 的结果，不是 Contract Validation 的结果。

## 11. Core Contract Examples

### 11.1 approveSpec

```text
actionType: approveSpec
targetMode: existingObject
targetObjectType: Spec
requiredEvidence:
  - humanConfirmation
expectedEvents:
  - SpecApproved
expectedLinks:
  - Decision accepts Spec
stateTransitionRef:
  - Spec.draft -> Spec.approved
```

### 11.2 markIssueDone

```text
actionType: markIssueDone
targetMode: existingObject
targetObjectType: Issue
requiredEvidence:
  - implementationSummary
  - verificationLog
  - artifactSummary
expectedEvents:
  - IssueMarkedDone
  - EvidenceLinked
stateTransitionRef:
  - Issue.inProgress -> Issue.done
forbiddenSideEffects:
  - requestAudit
```

### 11.3 requestAudit

```text
actionType: requestAudit
targetMode: existingObject
targetObjectType: Issue
requiredEvidence:
  - humanConfirmation
expectedEvents:
  - AuditRequested
expectedLinks:
  - Audit reviews Evidence
stateTransitionRef:
  - Audit.none -> Audit.requested
```

### 11.4 createFinding

```text
actionType: createFinding
targetMode: existingObject
targetObjectType: Audit
createsObjectType: Finding
requiredEvidence:
  - auditReport
expectedEvents:
  - FindingCreated
expectedLinks:
  - Finding reviews Evidence
```

## 12. Idempotency Rule

每个 Action Proposal 必须带 `idempotencyKey`。

建议组成：

```text
sourceSurface
actorRole
actionType
targetObjectRef
stableInputHash
```

目的：

- 避免同一 Agent 重试时重复创建对象；
- 为 Event Store append 做前置准备；
- 为 Arbitration 判断重复 proposal 提供依据。

`AF-OS-002` 只校验 key 结构，不检查历史重复。

## 13. Versioning

每个 proposal 必须携带：

```text
ontologyVersion
contractVersion
```

规则：

- proposal 的 `targetObjectType` 必须存在于对应 `ontologyVersion`；
- proposal 的 `actionType` 必须存在于对应 `contractVersion`；
- retired contract 不接受新 proposal；
- deprecated contract 可以产生 warning；
- version mismatch 不能自动降级。

## 14. Storage Strategy

第一版建议只提供 built-in core action registry。

推荐阶段：

```text
Phase 1: built-in core action contracts in Rust
Phase 2: export/read JSON action contract bundle
Phase 3: future .agentflow/action-contracts/** persisted definitions
```

当前阶段不写 `.agentflow/**`。

## 15. Public API Sketch

后续实现可以提供：

```text
core_action_contract_bundle() -> ActionContractBundle
core_action_contract_registry() -> ActionContractRegistry
validate_action_contract_bundle(bundle, ontology_registry) -> ActionContractValidationReport
validate_action_proposal(proposal, contract_registry, ontology_registry) -> ActionProposalValidationReport
get_action_contract(action_type, version) -> Option<ActionContract>
list_action_types() -> Vec<ActionTypeDefinition>
```

这些 API 不应接触：

```text
Event Store
Projection
Task Loop
Agent Dispatcher
Desktop UI
GitHub / GitLab
```

## 16. Test Plan

后续实现时建议测试：

1. core action contract bundle validates；
2. unknown action type fails；
3. retired contract rejects proposal；
4. proposal with missing required input fails；
5. proposal with unknown target object type fails；
6. proposal with unknown created object type fails；
7. proposal with invalid expected link type fails；
8. `approveSpec` requires `humanConfirmation`；
9. `markIssueDone` requires verification and artifact evidence；
10. `markIssueDone` does not include `requestAudit` side effect；
11. `requestAudit` is an explicit independent action；
12. `createFinding` creates `Finding`, not `AuditFinding`；
13. `WorkPackage` target is rejected in MVP core contracts；
14. idempotency key shape is required；
15. contract version mismatch returns validation error。

## 17. Acceptance Criteria

`AF-OS-002` 完成时应满足：

- 定义 `crates/action-contract` 的模型边界；
- 定义 Action Type / Action Contract / Action Proposal schema；
- core action set 对齐 `AF-OS-001` 的 10 个核心对象；
- 不引入 `WorkPackage` / `Delivery` / `DeliveryPackage` / `AuditFinding` 作为 MVP action target；
- 每个 core action 声明 target mode；
- 每个 core action 声明 target object 或 creates object；
- 每个写事实动作声明 required evidence；
- 每个写事实动作声明 expected event；
- `markIssueDone` 不自动触发 `requestAudit`；
- validation report 不输出 `ActionAccepted`；
- 不做权限、锁、事件写入、投影更新。

## 18. Risks

| risk | mitigation |
| --- | --- |
| Action Contract 过早承担权限判断 | 只保留 `approvalHint` / `conflictScopeHint`，真正判断放到 Arbitration |
| 动作集合扩大成完整产品工作流 | 只覆盖 10 个核心对象和主链路动作 |
| 与 State Machine 职责重叠 | 本 issue 只引用 transition，不判断 live transition |
| 与 Event Store 职责重叠 | 本 issue 只定义 expected event，不 append event |
| 与 UI 需求混淆 | 不定义任何页面、按钮或交互 |

## 19. Next

`AF-OS-002` 之后，最自然的下一份技术设计是：

```text
AF-OS-003 Agent Role Policy Technical Design Draft
```

原因：

- Action Contract 回答“能做什么动作”；
- Role Policy 回答“谁能提出这个动作”；
- State Machine 回答“当前状态能不能做”；
- Arbitration 把三者合并成最终接受或拒绝。
