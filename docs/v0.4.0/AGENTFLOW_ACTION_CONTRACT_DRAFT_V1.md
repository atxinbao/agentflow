# AgentFlow Action Contract Draft V1

日期：2026-06-19
执行者：Codex
用途：Next Version Development Preflight / 下一版本开发前置规格草案
状态：Architecture Draft / 非执行需求 / 不进入当前 v0.3.0 审计 / 不授权 Build Agent 执行

关联架构基线：

- [AGENTFLOW_AGENT_PROJECT_OS_ARCHITECTURE_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_AGENT_PROJECT_OS_ARCHITECTURE_V1.md)
- [AGENTFLOW_ONTOLOGY_SCHEMA_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_ONTOLOGY_SCHEMA_DRAFT_V1.md)
- [AGENTFLOW_OBJECT_STATE_MACHINE_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_OBJECT_STATE_MACHINE_DRAFT_V1.md)
- [AGENTFLOW_ARCHITECTURE_DECISION_RECORD_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_ARCHITECTURE_DECISION_RECORD_DRAFT_V1.md)

说明：本文件定义 Action Contract 草案。它不创建 `.agentflow/spec/**` 任务事实，不授权执行，也不替代正式 SPEC。

## 1. Purpose

Action Contract 定义 Agent 能提交什么动作，以及 Runtime 如何判断动作是否合法。

核心规则：

```text
Agent 不直接改状态
Agent 只提交 Action Proposal
Runtime 根据 Action Contract 仲裁
Event Store 只记录通过仲裁的事实
```

## 2. Action Lifecycle

建议动作生命周期：

```text
ActionProposed
→ ActionValidated
→ ActionAccepted / ActionRejected
→ EventsCommitted
→ ProjectionUpdated
```

如果需要模拟：

```text
ActionProposed
→ ActionSimulated
→ ActionValidated
→ ActionAccepted / ActionRejected
```

## 3. Action Contract Schema

每个 Action Type 必须定义：

```text
id
name
description
targetObjectType
inputSchema
preconditions
capabilityRequirements
stateTransition
effects
requiredEvidence
conflictPolicy
approvalPolicy
cancelPolicy
simulation
emittedEvents
projectionHints
```

建议结构：

```json
{
  "id": "submitEvidence",
  "name": "Submit Evidence",
  "targetObjectType": "Run",
  "inputSchema": {
    "runId": "string",
    "evidenceRefs": "array"
  },
  "preconditions": [
    "run.status == inProgress"
  ],
  "capabilityRequirements": {
    "roleCanExecute": "submitEvidence",
    "objectScope": "assignedRun"
  },
  "stateTransition": {
    "objectType": "Run",
    "from": "inProgress",
    "to": "evidenceSubmitted"
  },
  "effects": [
    "attachEvidenceToRun",
    "updateIssueEvidenceIndex"
  ],
  "requiredEvidence": [
    "verificationLog"
  ],
  "conflictPolicy": {
    "lock": "runId",
    "mode": "rejectOnConflict"
  },
  "approvalPolicy": {
    "humanApprovalRequired": false
  },
  "cancelPolicy": {
    "cancelAction": "withdrawEvidence",
    "compensationRequired": false
  },
  "simulation": {
    "enabled": true
  },
  "emittedEvents": [
    "EvidenceAttached",
    "RunStateChanged"
  ],
  "projectionHints": [
    "deliveryPackage",
    "evidenceGraph"
  ]
}
```

## 4. Action Proposal

Agent 提交的是 Action Proposal，不是状态写入。

最小字段：

```text
proposalId
actionType
actorRole
targetObjectRef
input
evidenceRefs
reason
expectedEffects
definitionVersion
createdAt
```

建议结构：

```json
{
  "proposalId": "proposal-001",
  "actionType": "markDone",
  "actorRole": "BuildAgent",
  "targetObjectRef": {
    "objectType": "Issue",
    "id": "AF-CLIENT-001"
  },
  "input": {
    "summary": "Implementation complete with local verification."
  },
  "evidenceRefs": ["evidence-run-001"],
  "reason": "Acceptance criteria satisfied.",
  "expectedEffects": ["IssueStateChanged"],
  "definitionVersion": "1.0.0-draft",
  "createdAt": "2026-06-19T00:00:00Z"
}
```

## 5. Arbitration Checklist

Agent Action Arbitration 必须检查：

```text
action type exists
input schema valid
target object exists
target object state matches precondition
actor role can execute action
actor role can access target object
required evidence exists
dependency is satisfied
object lock is available
conflict policy passes
approval policy passes
simulation passes
```

只有全部通过，才能输出：

```text
ActionAccepted
```

否则输出：

```text
ActionRejected
ConflictDetected
EvidenceRequired
ApprovalRequired
Blocked
```

## 6. Core Action Types

第一版建议定义这些核心动作：

| actionType | target | purpose |
| --- | --- | --- |
| `classifyRequirement` | `Requirement` | 分类输入 |
| `draftSpec` | `Requirement` | 生成 Spec Draft |
| `approveSpec` | `Spec` | 人类确认 Spec |
| `createWorkPackage` | `Spec` | 从 Spec 派生工作包 |
| `createIssue` | `WorkPackage` | 生成可执行 Issue |
| `startRun` | `Issue` | 启动 Agent Run |
| `submitEvidence` | `Run` | 提交执行证据 |
| `requestReview` | `Run` | 请求验证 |
| `verifyEvidence` | `Evidence` | 验证证据 |
| `markDeliveryReady` | `Issue` | 标记交付就绪 |
| `acceptDelivery` | `DeliveryPackage` | 人类接受交付 |
| `requestFix` | `DeliveryPackage` | 要求修复 |
| `requestAudit` | `WorkPackage` | 发起独立审计 |
| `createFinding` | `Audit` | 创建审计发现 |
| `resolveFinding` | `AuditFinding` | 处理审计发现 |
| `reopenIssue` | `Issue` | 重开 Issue |
| `createFollowUp` | `Decision` | 创建后续工作 |

## 7. State Transition Rule

Action Contract 不允许隐式改状态。

每个会改变状态的 Action 必须显式声明：

```text
objectType
from
to
stateMachineRef
```

示例：

```text
markDeliveryReady:
  objectType: Issue
  from: verified
  to: deliveryReady
  stateMachineRef: issue.state-machine
```

## 8. Evidence Rule

所有关键 Action 必须定义证据要求。

例如：

```text
submitEvidence:
  requiredEvidence: verificationLog

markDeliveryReady:
  requiredEvidence:
    - verificationLog
    - artifactSummary
    - acceptanceMapping

acceptDelivery:
  requiredEvidence:
    - humanDecision
```

## 9. Conflict Policy

多 Agent 场景必须定义冲突策略。

候选策略：

```text
rejectOnConflict
queueOnConflict
mergeIfCommutative
requireCoordinator
requireHumanDecision
```

冲突类型：

```text
sameObjectWrite
stateTransitionRace
dependencyNotSatisfied
evidenceMismatch
roleBoundaryViolation
definitionVersionMismatch
```

## 10. Simulation Rule

执行前应支持 dry-run。

Simulation 输出：

```text
wouldChangeObjects
wouldEmitEvents
requiredEvidence
blockedBy
conflicts
approvalRequired
projectionImpact
```

## 11. Emitted Events

ActionAccepted 后必须写事件。

事件建议：

```text
ActionSubmitted
ActionAccepted
ActionRejected
StateChanged
EvidenceAttached
ArtifactProduced
DecisionRecorded
FindingCreated
ConflictDetected
```

## 12. Non-goals

本草案不定义：

- 具体事件存储实现
- 当前 v0.3.0 任务
- Build Agent 执行授权
- GitHub / PR 操作细节
- `.agentflow/spec/**` 事实文件

## 13. Next

下一步应与 `agent-role.policy` 对齐，确保每个 Action 都能回答：

```text
谁可以做？
对哪个对象做？
在什么状态下做？
做完写什么事件？
必须留下什么证据？
```
