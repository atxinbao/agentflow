# AgentFlow Agent Role Policy Draft V1

日期：2026-06-19
执行者：Codex
用途：Next Version Development Preflight / 下一版本开发前置规格草案
状态：Architecture Draft / 非执行需求 / 不进入当前 v0.3.0 审计 / 不授权 Build Agent 执行

关联架构基线：

- [AGENTFLOW_AGENT_PROJECT_OS_ARCHITECTURE_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_AGENT_PROJECT_OS_ARCHITECTURE_V1.md)
- [AGENTFLOW_ACTION_CONTRACT_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_ACTION_CONTRACT_DRAFT_V1.md)
- [AGENTFLOW_ONTOLOGY_SCHEMA_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_ONTOLOGY_SCHEMA_DRAFT_V1.md)
- [AGENTFLOW_OBJECT_STATE_MACHINE_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_OBJECT_STATE_MACHINE_DRAFT_V1.md)
- [AGENTFLOW_ARCHITECTURE_DECISION_RECORD_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_ARCHITECTURE_DECISION_RECORD_DRAFT_V1.md)

说明：本文件定义 Agent Role Policy 草案。它不创建 `.agentflow/spec/**` 任务事实，不授权执行，也不替代正式 SPEC。

## 1. Purpose

Agent Role Policy 定义每个 Agent Role 的能力边界。

核心判断：

```text
Agent Role 不是 prompt
Agent Role 是能力合约
```

每个 Agent 必须明确：

```text
canRead
canWrite
canExecute
mustProduce
cannotDo
allowedTools
handoffRules
approvalGates
```

## 2. Role Policy Schema

最小结构：

```text
roleId
name
description
canRead
canWrite
canExecute
mustProduce
cannotDo
allowedTools
objectScope
approvalGates
handoffRules
requiredEvidence
```

建议结构：

```json
{
  "roleId": "BuildAgent",
  "name": "Build Agent",
  "description": "Executes approved spec issues within authorized contract boundaries.",
  "canRead": ["Spec", "Issue", "Run", "Evidence"],
  "canWrite": ["Run", "Evidence", "Artifact"],
  "canExecute": ["startRun", "submitEvidence", "requestReview", "markDeliveryReady"],
  "mustProduce": ["verificationLog", "artifactSummary", "evidenceRefs"],
  "cannotDo": ["draftSpec", "approveSpec", "createFinding", "passAudit"],
  "allowedTools": ["filesystem", "localBuild", "localTest"],
  "objectScope": ["assignedIssue", "currentRun"],
  "approvalGates": ["contractRequired"],
  "handoffRules": ["mustUseCurrentIssueContract"],
  "requiredEvidence": ["localVerification"]
}
```

## 3. Core Roles

第一版建议定义：

```text
SpecAgent
BuildAgent
AuditAgent
ReviewAgent
CoordinatorAgent
HumanOwner
```

## 4. Spec Agent Policy

职责：

```text
理解输入
分类需求
生成 Spec Draft Preview
生成 Work Package Preview
等待人类确认
```

`canExecute`：

```text
classifyRequirement
draftSpec
generateWorkPackage
routeRequirement
requestClarification
```

`mustProduce`：

```text
requirementClassification
specDraftPreview
workPackagePreview
boundaryNotes
```

`cannotDo`：

```text
implementCode
startRun
submitEvidence
createFinding
passAudit
```

关键规则：

```text
Spec Agent 可以生成 preview
Spec Agent 不能绕过人类确认写执行事实
```

## 5. Build Agent Policy

职责：

```text
执行已授权 Issue
产出实现结果
提交证据
请求验证
```

`canExecute`：

```text
startRun
submitEvidence
requestReview
markDeliveryReady
```

`mustProduce`：

```text
runLog
artifactSummary
verificationLog
evidenceRefs
deliverySummary
```

`cannotDo`：

```text
draftSpec
approveSpec
requestAuditByDefault
createFinding
writeAuditReport
bypassContract
```

关键规则：

```text
Build Agent 只认当前 Issue Contract
Build Agent 不创建 Audit 事实
Build Agent 不直接标记人类接受
```

## 6. Audit Agent Policy

职责：

```text
独立审计证据
创建 Finding
输出审计判断
```

`canExecute`：

```text
inspectEvidence
createFinding
classifyRisk
passAudit
requestFixFromFinding
```

`mustProduce`：

```text
auditReport
evidenceMap
findingList
traceability
auditDecision
```

`cannotDo`：

```text
implementCode
modifyDelivery
markIssueDone
acceptDelivery
rewriteBuildEvidence
```

关键规则：

```text
Audit 是独立流程
Audit 不修改 Build 交付事实
Audit Finding 通过 Link Type 回流到 Issue
```

## 7. Review Agent Policy

职责：

```text
验证证据
判断交付是否满足 Contract
输出 Review Decision
```

`canExecute`：

```text
verifyEvidence
requestMoreEvidence
approveDeliveryReady
rejectDeliveryReady
```

`mustProduce`：

```text
reviewDecision
verificationMapping
missingEvidenceList
```

`cannotDo`：

```text
implementCode
changeSpec
createAuditFinding
acceptHumanDelivery
```

## 8. Coordinator Agent Policy

职责：

```text
处理依赖
分配 Issue
识别冲突
协调多 Agent
```

`canExecute`：

```text
assignIssue
resolveDependency
detectConflict
queueAction
requestHumanDecision
```

`mustProduce`：

```text
dependencyDecision
conflictSummary
assignmentRecord
```

`cannotDo`：

```text
overrideHumanApproval
bypassArbitration
writeAcceptedFactDirectly
```

## 9. Human Owner Policy

职责：

```text
确认 Spec
接受交付
退回交付
批准审计
做最终业务决策
```

`canExecute`：

```text
approveSpec
rejectSpec
acceptDelivery
requestFix
requestAudit
createFollowUp
approveMigration
```

关键规则：

```text
Human command 也必须变成 Action
UI 不直接改状态
Human decision 进入 Event Store 后才成为事实
```

## 10. Tool Scope

Agent Role Policy 必须约束工具范围。

示例：

```text
SpecAgent:
  allowedTools: readDocs, inspectContext, generatePreview

BuildAgent:
  allowedTools: filesystem, localBuild, localTest, browserSmoke

AuditAgent:
  allowedTools: readEvidence, inspectDiff, generateAuditReport

ReviewAgent:
  allowedTools: readEvidence, runVerification, compareAcceptance

CoordinatorAgent:
  allowedTools: inspectState, detectDependency, enqueueAction
```

## 11. Object Scope

每个 Agent 只能操作授权对象。

示例：

```text
BuildAgent:
  objectScope:
    - assignedIssue
    - currentRun
    - evidenceOwnedByCurrentRun

AuditAgent:
  objectScope:
    - auditIssue
    - referencedEvidence
    - findingsOwnedByAudit
```

## 12. Handoff Rules

Handoff 必须是对象化事实，不是聊天约定。

最小字段：

```text
handoffId
fromRole
toRole
targetObjectRef
allowedActions
requiredInputs
expectedOutputs
boundaryNotes
definitionVersion
```

## 13. Policy Validation

Runtime 在接受 Action Proposal 前必须检查：

```text
role exists
role can execute action
role can access target object
role can use requested tool
role has required handoff
role can produce required evidence
role is not blocked by cannotDo
```

## 14. Non-goals

本草案不定义：

- prompt 模板
- 当前 v0.3.0 执行任务
- Build Agent 当前 issue handoff
- 审计报告格式
- `.agentflow/spec/**` 事实文件

## 15. Next

下一步应将 Role Policy 和 Action Contract 做成矩阵：

```text
role × actionType
role × objectType
role × evidenceType
role × toolScope
```

这张矩阵会成为 Agent Action Arbitration 的权限输入。
