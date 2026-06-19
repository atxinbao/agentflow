# AF-OS-003 Agent Role Policy Technical Design Draft V1

日期：2026-06-20
执行者：Codex
用途：Next Version Development Preflight / AF-OS-003 技术设计草案
状态：Architecture Draft / 非执行需求 / 不进入当前 v0.3.0 审计 / 不授权 Build Agent 执行

关联文档：

- [AGENTFLOW_RUNTIME_FOUNDATION_TECHNICAL_SUPPORT_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_RUNTIME_FOUNDATION_TECHNICAL_SUPPORT_DRAFT_V1.md)
- [AGENTFLOW_AGENT_ROLE_POLICY_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_AGENT_ROLE_POLICY_DRAFT_V1.md)
- [AGENTFLOW_AF_OS_001_ONTOLOGY_REGISTRY_TECHNICAL_DESIGN_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_AF_OS_001_ONTOLOGY_REGISTRY_TECHNICAL_DESIGN_DRAFT_V1.md)
- [AGENTFLOW_AF_OS_002_ACTION_CONTRACT_TECHNICAL_DESIGN_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_AF_OS_002_ACTION_CONTRACT_TECHNICAL_DESIGN_DRAFT_V1.md)
- [AGENTFLOW_ARCHITECTURE_DECISION_RECORD_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_ARCHITECTURE_DECISION_RECORD_DRAFT_V1.md)

说明：本文件只展开 `AF-OS-003` 的技术设计。它不创建 `.agentflow/spec/**` 任务事实，不写 `docs/requirements/**`，不修改源码，也不授权 Build Agent 执行。

## 1. Conclusion

`AF-OS-003` 的目标是把 Agent Role 从 prompt 文案升级成 Runtime 可读取、可校验、可仲裁的能力合约。

核心规则：

```text
Prompt 只能指导行为
Role Policy 才能定义权限边界
Action Proposal 必须用 Role Policy 判断 actorRole 是否允许提出动作
Role Policy 不能被 prompt、UI、CLI 或 handoff 文案覆盖
```

这一步完成后，后续 Arbitration 才能回答“谁可以对哪个对象提出哪个动作”。

## 2. Scope

### 2.1 In Scope

`AF-OS-003` 应覆盖：

- `crates/role-policy` crate 设计；
- Agent Role Policy schema；
- role capability schema；
- object scope schema；
- action scope schema；
- tool scope schema；
- handoff rule schema；
- evidence obligation schema；
- core role definitions；
- role-action matrix；
- role-object matrix；
- Build / Audit separation hard rules。

### 2.2 Out Of Scope

`AF-OS-003` 不做：

- prompt 模板重写；
- provider session 启动逻辑修改；
- Build Agent 执行；
- Audit Agent 执行；
- Action Arbitration；
- Event Store append；
- Projection update。

## 3. Proposed Crate

建议新增：

```text
crates/role-policy
```

建议模块：

```text
crates/role-policy/src/lib.rs
crates/role-policy/src/model.rs
crates/role-policy/src/registry.rs
crates/role-policy/src/validation.rs
crates/role-policy/src/core.rs
crates/role-policy/src/report.rs
```

职责划分：

| module | responsibility |
| --- | --- |
| `model.rs` | role / capability / scope / handoff schema |
| `registry.rs` | role lookup and policy version selection |
| `validation.rs` | role policy and role-action validation |
| `core.rs` | built-in `agentflow.roles.core@v1-draft` definitions |
| `report.rs` | validation report and denied reason taxonomy |
| `lib.rs` | public exports |

## 4. Dependencies

`AF-OS-003` 依赖：

```text
AF-OS-001 Ontology Registry
AF-OS-002 Action Contract
```

原因：

- Role Policy 的 object scope 必须引用 Ontology object types；
- Role Policy 的 canExecute 必须引用 Action Contract action types；
- required evidence 必须引用 Action Contract 定义的 evidence type；
- Arbitration 后续会同时读取 Action Contract、Role Policy 和 State Machine。

## 5. Core Model

### 5.1 AgentRolePolicyBundle

建议字段：

```text
bundleId
namespace
version
status
roles
roleActionMatrix
roleObjectMatrix
toolScopes
handoffRules
compatibility
```

### 5.2 AgentRolePolicy

建议字段：

```text
roleId
name
description
status
canRead
canWrite
canExecute
mustProduce
cannotDo
objectScopes
toolScopes
handoffRules
approvalGates
requiredEvidence
```

`status` 候选：

```text
active
deprecated
retired
```

### 5.3 RoleCapability

建议字段：

```text
actionType
mode
objectType
scope
requiresHandoff
requiresHumanApproval
requiredEvidence
```

`mode` 候选：

```text
read
propose
execute
review
decide
```

注意：`execute` 不等于直接写事实。它只表示 role 可以提出该 action proposal。

### 5.4 ObjectScope

建议字段：

```text
scopeId
objectType
scopeKind
description
```

`scopeKind` 候选：

```text
assignedIssue
currentRun
referencedEvidence
ownedFinding
approvedSpec
projectWideRead
humanDecisionTarget
```

### 5.5 ToolScope

建议字段：

```text
toolScopeId
allowedToolKinds
forbiddenToolKinds
requiresEvidenceCapture
```

MVP 工具种类：

```text
readDocs
inspectContext
filesystem
localBuild
localTest
browserSmoke
readEvidence
inspectDiff
generateReport
inspectState
```

### 5.6 HandoffRule

建议字段：

```text
handoffId
fromRole
toRole
targetObjectType
allowedActions
requiredInputs
expectedOutputs
boundaryNotes
```

Handoff 是 Runtime 可检查的对象化边界，不是聊天约定。

## 6. Core Roles

MVP core roles：

```text
SpecAgent
BuildAgent
AuditAgent
ReviewAgent
CoordinatorAgent
HumanOwner
```

### 6.1 SpecAgent

允许：

```text
submitRequirement
normalizeRequirement
classifyRequirement
draftSpec
```

禁止：

```text
startRun
submitEvidence
markIssueDone
requestAudit
createFinding
linkFixIssue
```

必须产出：

```text
requirementIntakeResult
specDraftPreview
boundaryNotes
```

### 6.2 BuildAgent

允许：

```text
startRun
submitEvidence
submitArtifact
markIssueDone
```

禁止：

```text
draftSpec
approveSpec
requestAudit
createFinding
linkFixIssue
recordAuditDecision
```

必须产出：

```text
implementationSummary
verificationLog
artifactSummary
```

硬规则：

```text
BuildAgent 只处理 assignedIssue 和 currentRun
BuildAgent 不创建 Audit 事实
BuildAgent 不写 Finding
BuildAgent 不代表 HumanOwner 接受交付
```

### 6.3 AuditAgent

允许：

```text
createFinding
linkFixIssue
submitEvidence
submitArtifact
```

禁止：

```text
implementCode
markIssueDone
rewriteBuildEvidence
modifyDeliveryFact
approveSpec
```

必须产出：

```text
auditReport
evidenceMap
findingRecord
```

硬规则：

```text
AuditAgent 可以读取 Build evidence
AuditAgent 不能修改 Build delivery fact
AuditAgent 的 Finding 通过 Issue 回流，不改写旧事实
```

### 6.4 ReviewAgent

允许：

```text
submitEvidence
recordDecision
```

禁止：

```text
implementCode
draftSpec
createFinding
requestAudit
```

用途：

```text
验证 evidence 是否满足 action contract
输出 reviewDecision 或 missingEvidenceList
```

### 6.5 CoordinatorAgent

允许：

```text
recordDecision
linkFixIssue
```

禁止：

```text
bypassArbitration
overrideHumanApproval
writeAcceptedFactDirectly
```

用途：

```text
处理依赖、冲突、排队、分配和人类裁决请求
```

### 6.6 HumanOwner

允许：

```text
approveSpec
recordDecision
requestAudit
linkFixIssue
```

禁止：

```text
directWriteEventStore
directMutateProjection
```

规则：

```text
HumanOwner 的决定也必须变成 Action Proposal
HumanOwner 的决定进入 Event Store 后才成为项目事实
```

## 7. Role Matrices

### 7.1 Role Action Matrix

MVP 必须生成一个矩阵：

```text
roleId × actionType × allowed/denied × reason
```

### 7.2 Role Object Matrix

MVP 必须生成一个矩阵：

```text
roleId × objectType × read/write/propose scope
```

### 7.3 Role Evidence Matrix

MVP 必须生成一个矩阵：

```text
roleId × evidenceType × mustProduce/canRead/cannotProduce
```

## 8. Validation Pipeline

`AF-OS-003` validation 建议顺序：

```text
1. role exists
2. role policy version exists
3. action type exists in Action Contract registry
4. object type exists in Ontology registry
5. role canExecute includes action type
6. role object scope includes target object type
7. role cannotDo does not deny action
8. required handoff exists if needed
9. required evidence obligations are declared
10. tool scope is known
```

本 issue 只输出 role policy validation report，不输出 Arbitration decision。

## 9. Public API Sketch

后续实现可以提供：

```text
core_role_policy_bundle() -> AgentRolePolicyBundle
core_role_policy_registry() -> RolePolicyRegistry
validate_role_policy_bundle(bundle, ontology_registry, action_registry) -> RolePolicyValidationReport
can_role_propose_action(role_id, action_type, object_type) -> RoleCapabilityDecision
get_role_policy(role_id, version) -> Option<AgentRolePolicy>
list_core_roles() -> Vec<AgentRolePolicy>
```

这些 API 不应接触：

```text
Event Store
Projection
Task Loop
Provider Session
Desktop UI
```

## 10. Test Plan

后续实现时建议测试：

1. core role policy bundle validates；
2. unknown role fails；
3. role references unknown action type fails；
4. role references unknown object type fails；
5. BuildAgent cannot `draftSpec`；
6. BuildAgent cannot `approveSpec`；
7. BuildAgent cannot `requestAudit`；
8. BuildAgent cannot `createFinding`；
9. AuditAgent cannot `markIssueDone`；
10. AuditAgent cannot rewrite Build evidence；
11. HumanOwner can `approveSpec`；
12. HumanOwner can `requestAudit`；
13. prompt cannot override `cannotDo`；
14. missing handoff rule fails when required；
15. role-action matrix exports deterministically。

## 11. Acceptance Criteria

`AF-OS-003` 完成时应满足：

- 定义 `crates/role-policy` 的模型边界；
- 定义 core roles；
- 定义 role-action matrix；
- 定义 role-object matrix；
- BuildAgent / AuditAgent 分离规则可机读；
- HumanOwner 决策必须走 Action Proposal；
- prompt 不能覆盖 Role Policy；
- Role Policy 可被 `AF-OS-005` Arbitration 读取。

## 12. Risks

| risk | mitigation |
| --- | --- |
| 把 prompt 当权限系统 | 明确 prompt 不参与最终权限判断 |
| 角色权限过细导致实现膨胀 | MVP 只覆盖 core roles 和 core actions |
| HumanOwner 被当成直接写状态特权 | HumanOwner 也必须提交 Action Proposal |
| Build/Audit 边界被放宽 | 用 `cannotDo` 和测试固定硬边界 |

## 13. Next

`AF-OS-003` 之后需要与 `AF-OS-004` 状态机合并进入 Arbitration：

```text
Action Contract 回答动作是否合法
Role Policy 回答角色是否允许提出动作
Object State Machine 回答当前状态是否允许动作
Arbitration 统一给出 accepted/rejected/humanDecisionRequired
```
