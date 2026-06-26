# AgentFlow Ontology Schema Draft V1

日期：2026-06-19
执行者：Codex
用途：Next Version Development Preflight / 下一版本开发前置规格草案
状态：Architecture Draft / 非执行需求 / 不进入当前 v0.3.0 审计 / 不授权 Build Agent 执行

关联架构基线：

- [AGENTFLOW_AGENT_PROJECT_OS_ARCHITECTURE_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_AGENT_PROJECT_OS_ARCHITECTURE_V1.md)
- [AGENTFLOW_OBJECT_STATE_MACHINE_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_OBJECT_STATE_MACHINE_DRAFT_V1.md)
- [AGENTFLOW_ARCHITECTURE_DECISION_RECORD_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_ARCHITECTURE_DECISION_RECORD_DRAFT_V1.md)

说明：本文件定义 Project Ontology Layer 的 schema 草案。它不创建 `.agentflow/spec/**` 任务事实，不授权执行，也不替代正式 SPEC。

## 1. Purpose

Ontology Schema 定义 AgentFlow 项目世界的通用语言。

它回答：

- 项目世界里有哪些对象；
- 对象之间可以建立哪些关系；
- 对象有哪些状态机；
- 定义如何注册、版本化、迁移；
- Runtime 如何读取这些定义供 Action Arbitration 使用。

## 2. Design Principle

核心原则：

```text
行业输入可变
Project Ontology 必须稳定
Runtime Core 只读取标准化后的定义
```

Ontology Schema 不是普通数据库 schema。它是 Runtime 可读取的项目世界定义。

## 3. Definition Bundle

一个 Project Ontology definition bundle 至少包含：

```text
ontologyId
namespace
version
status
objectTypes
linkTypes
stateMachines
actionTypeRefs
rolePolicyRefs
compatibility
migration
```

建议结构：

```json
{
  "ontologyId": "agentflow.project-os",
  "namespace": "agentflow.core",
  "version": "1.0.0-draft",
  "status": "draft",
  "objectTypes": [],
  "linkTypes": [],
  "stateMachines": [],
  "actionTypeRefs": [],
  "rolePolicyRefs": [],
  "compatibility": {
    "replayFromVersion": "1.0.0-draft"
  },
  "migration": {
    "strategy": "explicit"
  }
}
```

## 4. Ontology Registry Record

所有定义必须进入注册表。

最小字段：

```text
id
namespace
kind
version
status
owner
createdAt
updatedAt
compatibility
deprecation
```

`kind` 候选值：

```text
objectType
linkType
stateMachine
actionType
functionType
agentRolePolicy
projectionModel
```

`status` 候选值：

```text
draft
active
deprecated
retired
```

## 5. Object Type Schema

Object Type 定义项目世界里的对象。

最小字段：

```text
id
name
description
properties
requiredProperties
stateMachineRef
allowedLinkTypes
allowedActionTypes
projectionHints
```

建议结构：

```json
{
  "id": "issue",
  "name": "Issue",
  "description": "Executable work contract derived from approved spec or work package.",
  "properties": {
    "title": "string",
    "status": "string",
    "priority": "string",
    "ownerRole": "string"
  },
  "requiredProperties": ["title", "status"],
  "stateMachineRef": "issue.state-machine",
  "allowedLinkTypes": ["decomposesTo", "blocks", "executes", "proves"],
  "allowedActionTypes": ["startRun", "submitEvidence", "markDone", "requestReview"],
  "projectionHints": ["issueIndex", "taskWorkbench"]
}
```

## 6. Core Object Types

第一版建议定义这些核心对象：

| objectType | purpose |
| --- | --- |
| `Requirement` | 原始意图进入标准化后的需求对象 |
| `Spec` | 被确认的需求说明与边界定义 |
| `WorkPackage` | 从 Spec 派生出的工作包 |
| `Project` | 跨 Issue 的项目聚合 |
| `Issue` | 可执行工作契约 |
| `Run` | Agent 执行一次 Issue 的运行记录 |
| `Evidence` | 运行证据、日志、截图、测试结果 |
| `Artifact` | 产物、文档、代码变更、交付文件 |
| `Decision` | 人类确认、接受、退回、继续等决策 |
| `AuditFinding` | 审计发现 |
| `AgentRole` | Agent 角色定义 |

## 7. Link Type Schema

Link Type 定义对象之间的合法关系。

最小字段：

```text
id
sourceObjectType
targetObjectType
cardinality
description
allowedActions
projectionHints
```

建议结构：

```json
{
  "id": "proves",
  "sourceObjectType": "Evidence",
  "targetObjectType": "Issue",
  "cardinality": "many-to-one",
  "description": "Evidence proves a specific executable issue or acceptance criterion.",
  "allowedActions": ["submitEvidence", "verifyEvidence"],
  "projectionHints": ["evidenceGraph", "deliveryPackage"]
}
```

## 8. Core Link Types

第一版建议定义：

| linkType | source | target | purpose |
| --- | --- | --- | --- |
| `derivesFrom` | `Spec` | `Requirement` | Spec 来自需求 |
| `decomposesTo` | `Spec` | `WorkPackage` | Spec 拆为工作包 |
| `contains` | `Project` | `Issue` | Project 包含 Issue |
| `blocks` | `Issue` | `Issue` | Issue 依赖阻塞 |
| `executes` | `Run` | `Issue` | Run 执行 Issue |
| `produces` | `Run` | `Artifact` | Run 产生交付产物 |
| `proves` | `Evidence` | `Issue` | Evidence 证明 Issue |
| `reviews` | `AuditFinding` | `Evidence` | Finding 审查 Evidence |
| `decides` | `Decision` | `WorkPackage` | Decision 影响工作包 |
| `requiresFix` | `AuditFinding` | `Issue` | Finding 触发修复 |

## 9. State Machine Schema

State Machine 定义对象状态如何转移。

最小字段：

```text
id
objectType
states
initialState
terminalStates
transitions
```

Transition 最小字段：

```text
from
to
actionType
precondition
requiredEvidence
emittedEvents
```

建议结构：

```json
{
  "id": "issue.state-machine",
  "objectType": "Issue",
  "initialState": "ready",
  "terminalStates": ["accepted", "cancelled"],
  "states": ["ready", "inProgress", "evidenceSubmitted", "verified", "deliveryReady", "accepted", "reworkRequired", "cancelled"],
  "transitions": [
    {
      "from": "ready",
      "to": "inProgress",
      "actionType": "startRun",
      "precondition": "issue.hasApprovedContract",
      "requiredEvidence": [],
      "emittedEvents": ["RunStarted", "IssueStateChanged"]
    }
  ]
}
```

## 10. Core State Machines

第一版建议拆分：

```text
requirement.state-machine
spec.state-machine
work-package.state-machine
issue.state-machine
run.state-machine
audit.state-machine
finding.state-machine
decision.state-machine
```

不要用一条线性流程表达所有对象。

错误模型：

```text
Requirement → Spec → Issue → Run → Done
```

目标模型：

```text
Object State Machines + Link Types + Action Types
```

## 11. Compatibility And Migration

Ontology Schema 必须支持版本演进。

每个版本必须说明：

```text
breakingChanges
compatibleFrom
eventReplayCompatibility
migrationRequired
migrationFunction
deprecatedDefinitions
```

迁移原则：

```text
旧事件不能失效
旧项目必须可读
新 Projection 必须能识别旧 definition version
破坏性变更必须显式迁移
```

## 12. Validation Rules

Ontology definition bundle 必须通过这些校验：

- 所有 `objectTypes` 必须有唯一 `id`
- 所有 `linkTypes.sourceObjectType` 和 `targetObjectType` 必须存在
- 所有 `stateMachine.objectType` 必须存在
- 所有 transition 引用的 `actionType` 必须存在
- `terminalStates` 必须包含在 `states` 内
- `allowedActionTypes` 必须引用已注册 Action Type
- deprecated definition 不得被新 Contract 默认引用

## 13. Non-goals

本草案不定义：

- 数据库表结构
- UI 页面字段
- 具体 Rust 类型
- 当前 v0.3.0 执行任务
- `.agentflow/spec/**` 事实文件

## 14. Next

下一步应与 `action.contract` 和 `agent-role.policy` 对齐，形成：

```text
Ontology Schema
Action Contract
Agent Role Policy
State Machine Draft
```

状态机细节见：

- [AGENTFLOW_OBJECT_STATE_MACHINE_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_OBJECT_STATE_MACHINE_DRAFT_V1.md)
