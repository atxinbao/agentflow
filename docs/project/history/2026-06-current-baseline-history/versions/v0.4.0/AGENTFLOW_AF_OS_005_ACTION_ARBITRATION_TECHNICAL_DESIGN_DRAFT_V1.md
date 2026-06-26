# AF-OS-005 Action Arbitration Technical Design Draft V1

日期：2026-06-20
执行者：Codex
用途：Next Version Development Preflight / AF-OS-005 技术设计草案
状态：Architecture Draft / 非执行需求 / 不进入当前 v0.3.0 审计 / 不授权 Build Agent 执行

关联文档：

- [AGENTFLOW_RUNTIME_FOUNDATION_TECHNICAL_SUPPORT_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_RUNTIME_FOUNDATION_TECHNICAL_SUPPORT_DRAFT_V1.md)
- [AGENTFLOW_AF_OS_002_ACTION_CONTRACT_TECHNICAL_DESIGN_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_AF_OS_002_ACTION_CONTRACT_TECHNICAL_DESIGN_DRAFT_V1.md)
- [AGENTFLOW_AF_OS_003_AGENT_ROLE_POLICY_TECHNICAL_DESIGN_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_AF_OS_003_AGENT_ROLE_POLICY_TECHNICAL_DESIGN_DRAFT_V1.md)
- [AGENTFLOW_AF_OS_004_OBJECT_STATE_MACHINE_TECHNICAL_DESIGN_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_AF_OS_004_OBJECT_STATE_MACHINE_TECHNICAL_DESIGN_DRAFT_V1.md)
- [AGENTFLOW_ARCHITECTURE_DECISION_RECORD_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_ARCHITECTURE_DECISION_RECORD_DRAFT_V1.md)

说明：本文件只展开 `AF-OS-005` 的技术设计。它不创建 `.agentflow/spec/**` 任务事实，不写 `docs/requirements/**`，不修改源码，也不授权 Build Agent 执行。

## 1. Conclusion

`AF-OS-005` 是 Runtime 写事实前的唯一仲裁入口。

核心规则：

```text
Action Proposal 先进入 Arbitration
Arbitration 同时读取 Action Contract / Role Policy / State Machine / Evidence / Dependency / Lock
只有 accepted action 才能进入 Event Store append
rejected action 必须给出稳定 reason
humanDecisionRequired 必须被显式表达
```

没有 Arbitration，多 Agent 只会变成多个写入口。那不是 OS，是混乱。

## 2. Scope

### 2.1 In Scope

`AF-OS-005` 应覆盖：

- `crates/action-arbitration` crate 设计；
- ArbitrationRequest schema；
- ArbitrationContext schema；
- ArbitrationDecision schema；
- object lock schema；
- rejected reason taxonomy；
- human decision required response；
- dependency check boundary；
- accepted action event draft shape；
- 与 Event Store append 的接口边界。

### 2.2 Out Of Scope

`AF-OS-005` 不做：

- 分布式锁；
- 跨项目事务；
- 自动冲突合并；
- Event Store append implementation；
- Projection rebuild；
- provider 启动；
- Build Agent 执行；
- Audit Agent 执行。

## 3. Proposed Crate

建议新增：

```text
crates/action-arbitration
```

建议模块：

```text
crates/action-arbitration/src/lib.rs
crates/action-arbitration/src/model.rs
crates/action-arbitration/src/arbitrator.rs
crates/action-arbitration/src/locks.rs
crates/action-arbitration/src/reasons.rs
crates/action-arbitration/src/report.rs
```

## 4. Dependencies

依赖：

```text
AF-OS-002 Action Contract
AF-OS-003 Agent Role Policy
AF-OS-004 Object State Machine
```

运行时读取：

```text
current object state
existing evidence refs
dependency status
active object locks
latest definition versions
```

MVP 可以从现有 projection / state index 读取当前状态，但事实权威仍然是 Event Store。

## 5. Core Model

### 5.1 ArbitrationRequest

建议字段：

```text
requestId
proposal
definitionVersions
requestedAt
```

`proposal` 来自 `AF-OS-002` 的 `ActionProposal`。

### 5.2 ArbitrationContext

建议字段：

```text
ontologyRegistry
actionContractRegistry
rolePolicyRegistry
stateMachineRegistry
currentObjectState
evidenceIndex
dependencyIndex
objectLockIndex
```

### 5.3 ArbitrationDecision

建议字段：

```text
decisionId
requestId
proposalId
status
acceptedAction
rejectedReasons
requiredHumanDecision
lockPlan
wouldEmitEvents
createdAt
```

`status` 候选：

```text
accepted
rejected
humanDecisionRequired
queued
conflictDetected
```

MVP 建议先支持：

```text
accepted
rejected
humanDecisionRequired
```

### 5.4 ObjectLock

建议字段：

```text
lockId
objectType
objectId
lockKind
ownerProposalId
ownerRole
expiresAt
reason
```

`lockKind` 候选：

```text
write
runExecution
auditReview
decisionPending
```

默认规则：

```text
同一对象同一时间只允许一个 active write lock
Issue 默认只允许一个 active runExecution lock
Review/Audit 可以读同一对象，但不能持有 Build write lock
```

## 6. Arbitration Pipeline

建议顺序：

```text
1. validate action proposal structure
2. load action contract
3. validate target object type
4. validate role policy
5. load current object state
6. validate state transition
7. validate required evidence refs exist
8. validate dependencies satisfied
9. validate object lock availability
10. determine human decision requirement
11. prepare accepted action envelope
```

任何一步失败都必须产生稳定 reason。

## 7. Rejected Reason Taxonomy

建议 reason：

```text
unknownActionType
invalidActionProposal
unknownActorRole
roleCannotExecuteAction
roleCannotAccessObject
unknownTargetObject
invalidObjectState
missingRequiredEvidence
dependencyNotSatisfied
objectLockUnavailable
humanDecisionMissing
definitionVersionMismatch
conflictDetected
```

这些 reason 必须可投影到 UI 和 CLI。

## 8. Human Decision Required

当 action 合法但缺少人类裁决时，返回：

```text
humanDecisionRequired
```

建议字段：

```text
decisionKind
targetObjectRef
question
allowedResponses
requiredEvidenceType
```

例子：

```text
approveSpec requires HumanOwner decision
requestAudit may require HumanOwner decision
reopenIssue may require HumanOwner decision
```

Human decision 本身也必须回到 Runtime API，形成新的 Action Proposal。

## 9. Accepted Action Shape

Arbitration accepted 后输出：

```text
AcceptedAction
  acceptedActionId
  proposalId
  actionType
  actorRole
  targetObjectRef
  fromState
  toState
  evidenceRefs
  artifactRefs
  expectedEvents
  lockPlan
  definitionVersions
```

`AcceptedAction` 不是事件。
它是 Event Store append 的输入。

## 10. Lock Rules

MVP lock 规则：

| object | rule |
| --- | --- |
| `Issue` | one active `runExecution` lock |
| `Run` | one active `write` lock |
| `Audit` | one active `auditReview` lock |
| `Finding` | one active `write` lock |
| `Spec` | human confirmation blocks conflicting spec write |

禁止：

```text
distributed lock
cross-project transaction
automatic merge
silent lock stealing
```

## 11. Public API Sketch

后续实现可以提供：

```text
arbitrate_action(request, context) -> ArbitrationDecision
check_object_lock(target, lock_kind, context) -> LockDecision
build_accepted_action(proposal, contract, state_transition, context) -> AcceptedAction
explain_rejection(decision) -> RejectionExplanation
```

这些 API 不应接触：

```text
Provider Session
Desktop UI rendering
GitHub / GitLab
Projection write
```

## 12. Test Plan

后续实现时建议测试：

1. valid proposal returns accepted；
2. unknown action returns rejected；
3. BuildAgent cannot `createFinding`；
4. AuditAgent cannot `markIssueDone`；
5. invalid object state returns rejected；
6. missing evidence returns rejected；
7. unmet dependency returns rejected；
8. active write lock returns rejected；
9. `approveSpec` without human decision returns humanDecisionRequired；
10. accepted action includes causation proposal id；
11. rejected action includes stable reason；
12. `Issue.done` does not create Audit accepted action；
13. lock release is not hidden inside decision。

## 13. Acceptance Criteria

`AF-OS-005` 完成时应满足：

- 所有写事实动作必须经过 Arbitration；
- accepted action 才能进入 Event Store append；
- rejected action 必须给出 reason；
- Human decision required 可以表达；
- 同一对象默认只有一个 active write lock；
- Build / Audit 边界被 Role Policy 强制执行；
- state transition 由 Object State Machine 决定；
- Arbitration 不启动 provider、不写 Projection。

## 14. Risks

| risk | mitigation |
| --- | --- |
| Arbitration 变成业务逻辑大杂烩 | 只组合已有 registry 和当前状态，不定义新业务对象 |
| 过早做分布式锁 | MVP 只做本地对象锁规则 |
| Human decision 被绕过 | 需要人类裁决时只返回 `humanDecisionRequired` |
| rejected reason 不稳定 | reason taxonomy 固化为可测试 enum |

## 15. Next

`AF-OS-005` 之后才能进入：

```text
AF-OS-006 Event Store Integration
```

因为 Event Store 只能记录已经通过仲裁的事实，不能记录未经仲裁的 Agent 意图作为最终状态。
