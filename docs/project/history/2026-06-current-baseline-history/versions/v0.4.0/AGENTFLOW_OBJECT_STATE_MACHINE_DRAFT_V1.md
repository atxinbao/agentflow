# AgentFlow Object State Machine Draft V1

日期：2026-06-19
执行者：Codex
用途：Next Version Development Preflight / 下一版本开发前置规格草案
状态：Architecture Draft / 非执行需求 / 不进入当前 v0.3.0 审计 / 不授权 Build Agent 执行

关联架构基线：

- [AGENTFLOW_AGENT_PROJECT_OS_ARCHITECTURE_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_AGENT_PROJECT_OS_ARCHITECTURE_V1.md)
- [AGENTFLOW_ONTOLOGY_SCHEMA_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_ONTOLOGY_SCHEMA_DRAFT_V1.md)
- [AGENTFLOW_ACTION_CONTRACT_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_ACTION_CONTRACT_DRAFT_V1.md)
- [AGENTFLOW_AGENT_ROLE_POLICY_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_AGENT_ROLE_POLICY_DRAFT_V1.md)
- [AGENTFLOW_ARCHITECTURE_DECISION_RECORD_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_ARCHITECTURE_DECISION_RECORD_DRAFT_V1.md)

说明：本文件定义 AgentFlow Project OS 的核心对象状态机草案。它不创建 `.agentflow/spec/**` 任务事实，不授权执行，也不替代正式 SPEC。

## 1. Purpose

Object State Machine 定义项目对象的生命周期。

它回答：

- `Requirement` 如何从人类意图变成可分析需求；
- `Spec` 如何从草案变成已确认边界；
- `Issue` 如何从可执行契约进入执行、交付和关闭；
- `Run` 如何记录 Agent 的一次执行；
- `Audit` 如何保持独立；
- `Finding` 如何驱动修复而不污染 Build 主链路。

核心结论：

```text
状态不是 Agent 直接写入的字段
状态是 Runtime 根据事件和状态机推导出的事实
```

## 2. Core Rule

AgentFlow 的状态变化必须经过统一路径：

```text
Action Proposal
→ Agent Action Arbitration
→ Action Contract Validation
→ Object State Machine Transition
→ Event Store Append
→ Projection Rebuild
```

任何 Agent、UI、脚本、Connector 都不应该直接把对象状态改成最终值。

状态变化必须满足：

- 当前对象类型有注册状态机；
- 当前状态允许该 Action；
- Agent Role Policy 允许发起该 Action；
- Action Contract 的 guard、evidence、effect 都通过；
- Event Store 成功追加事实事件；
- Projection 根据事件重建读模型。

## 3. Shared Transition Schema

每个对象状态机应使用同一套最小结构。

```text
stateMachineId
objectType
version
initialState
states
terminalStates
transitions
invariants
projectionHints
```

每个 transition 至少包含：

```text
from
actionType
to
guards
requiredEvidence
emittedEvents
linkEffects
rolePolicy
```

建议结构：

```json
{
  "stateMachineId": "issue.state-machine",
  "objectType": "Issue",
  "version": "1.0.0-draft",
  "initialState": "proposed",
  "terminalStates": ["cancelled", "superseded"],
  "transitions": [
    {
      "from": "ready",
      "actionType": "startRun",
      "to": "running",
      "guards": ["issueNotBlocked", "noActiveRunLock"],
      "requiredEvidence": [],
      "emittedEvents": ["RunStarted", "IssueStateChanged"],
      "linkEffects": ["Run executes Issue"],
      "rolePolicy": ["BuildAgent.canExecute.startRun"]
    }
  ]
}
```

## 4. State Vocabulary

状态命名使用 `lowerCamelCase`。

禁止使用含糊状态：

```text
ok
finished
completeMaybe
waiting
processing
```

建议使用能表达对象责任的状态：

```text
captured
classified
needsClarification
approved
ready
blocked
running
reviewReady
done
cancelled
superseded
```

注意：`done` 不一定是硬终态。
如果审计或人类决策要求返工，`done` 可以通过授权 Action 回到 `reopened` 或派生新的修复 Issue。

## 5. Requirement State Machine

`Requirement` 表示进入 AgentFlow 后的标准化需求对象。

它不是原始聊天记录。原始人类输入需要先经过 Intake Normalizer。

### 5.1 States

| state | meaning |
| --- | --- |
| `captured` | 已捕获原始意图 |
| `normalized` | 已清洗、去噪、补齐上下文指针 |
| `classified` | 已完成需求分类 |
| `needsClarification` | 需要人类补充信息 |
| `accepted` | 已接受，可进入 Spec Loop |
| `rejected` | 不进入后续流程 |
| `superseded` | 被新需求替代 |

### 5.2 Transitions

| from | actionType | to | guard |
| --- | --- | --- | --- |
| none | `captureRequirement` | `captured` | 有人类输入或 Connector 输入 |
| `captured` | `normalizeIntake` | `normalized` | 输入可解析 |
| `normalized` | `classifyRequirement` | `classified` | 分类结果完整 |
| `captured` / `normalized` / `classified` | `requestClarification` | `needsClarification` | 存在阻断性歧义 |
| `needsClarification` | `answerClarification` | `normalized` | 人类已补充信息 |
| `classified` | `acceptRequirement` | `accepted` | 边界清楚且允许进入 Spec Loop |
| `classified` / `needsClarification` | `rejectRequirement` | `rejected` | 不属于项目范围或不应执行 |
| `accepted` / `classified` | `supersedeRequirement` | `superseded` | 新需求替代旧需求 |

### 5.3 Invariants

- `Requirement` 不直接生成可执行任务。
- `Requirement.accepted` 只表示可以进入 Spec Loop。
- `Requirement.rejected` 不得继续派生 `Spec`。
- `Requirement.superseded` 必须链接到替代它的新 `Requirement`。

## 6. Spec State Machine

`Spec` 表示经过 Spec Loop 形成的需求边界、范围、验收和任务拆分依据。

### 6.1 States

| state | meaning |
| --- | --- |
| `drafting` | Spec Agent 正在生成草案 |
| `draftReady` | 草案可预览 |
| `awaitingConfirmation` | 等待人类确认 |
| `changeRequested` | 人类要求调整 |
| `approved` | 已确认，可派生工作契约 |
| `cancelled` | 已取消 |
| `superseded` | 被新版 Spec 替代 |

### 6.2 Transitions

| from | actionType | to | guard |
| --- | --- | --- | --- |
| none | `draftSpec` | `drafting` | Requirement 已 accepted |
| `drafting` | `prepareSpecPreview` | `draftReady` | 草案字段完整 |
| `draftReady` | `requestHumanConfirmation` | `awaitingConfirmation` | Preview 可读 |
| `awaitingConfirmation` | `approveSpec` | `approved` | 人类明确确认 |
| `awaitingConfirmation` | `requestSpecChanges` | `changeRequested` | 人类给出调整意见 |
| `changeRequested` | `reviseSpec` | `drafting` | 调整意见可执行 |
| `drafting` / `draftReady` / `awaitingConfirmation` / `changeRequested` | `cancelSpec` | `cancelled` | 人类取消或需求失效 |
| `approved` | `supersedeSpec` | `superseded` | 新版 Spec 替代 |

### 6.3 Invariants

- `Spec.approved` 是派生 Issue 的前置条件。
- 未确认的 Spec Preview 不能授权 Build Agent 执行。
- `Spec.cancelled` 不得继续派生新 Issue。
- `Spec.superseded` 必须保留旧版与新版之间的 link，供事件回放和审计解释。

## 7. Issue State Machine

`Issue` 是可执行工作契约。

它不是普通待办。它必须来自已确认的 Spec、修复 Finding、或明确授权的工作包。

### 7.1 States

| state | meaning |
| --- | --- |
| `proposed` | 已生成但尚未激活 |
| `ready` | 可执行 |
| `blocked` | 依赖或边界阻塞 |
| `running` | 有活跃 Run 正在执行 |
| `reviewReady` | 交付材料已准备好，等待接收或后续处理 |
| `done` | 工作契约已完成 |
| `reopened` | 完成后被授权重开 |
| `cancelled` | 已取消 |
| `superseded` | 被替代 |

### 7.2 Transitions

| from | actionType | to | guard |
| --- | --- | --- | --- |
| none | `createIssue` | `proposed` | 来源对象合法 |
| `proposed` | `activateIssue` | `ready` | 依赖完整且边界清晰 |
| `ready` | `startRun` | `running` | 无活跃 Run lock |
| `ready` / `running` | `blockIssue` | `blocked` | 发现阻塞条件 |
| `blocked` | `unblockIssue` | `ready` | 阻塞已解除 |
| `running` | `submitDelivery` | `reviewReady` | 必需 evidence 已提交 |
| `reviewReady` | `markIssueDone` | `done` | 验收条件满足 |
| `done` | `reopenIssue` | `reopened` | 人类或授权 Finding 要求返工 |
| `reopened` | `activateIssue` | `ready` | 返工边界清楚 |
| `proposed` / `ready` / `blocked` / `reopened` | `cancelIssue` | `cancelled` | 取消理由已记录 |
| `proposed` / `ready` / `blocked` / `done` | `supersedeIssue` | `superseded` | 新 Issue 替代旧 Issue |

### 7.3 Invariants

- `Issue` 的可执行性来自 Contract，不来自聊天消息。
- 默认一个 `Issue` 同一时间只能有一个 active `Run`。
- 多 Agent 并发必须由 Arbitration 显式分配对象锁或子任务锁。
- `Issue.done` 不自动创建 Audit。
- `Issue.done` 可以成为独立 Audit 的输入，但不能绕过 Audit Issue 或人类审计请求。

## 8. Run State Machine

`Run` 表示 Agent 对一个 Issue 的一次执行尝试。

它记录执行过程，不等同于 Issue 本身。

### 8.1 States

| state | meaning |
| --- | --- |
| `queued` | 已排队 |
| `started` | 已开始执行 |
| `awaitingInput` | 等待必要输入 |
| `paused` | 被暂停 |
| `blocked` | 执行中遇到阻塞 |
| `failed` | 本次尝试失败，可重试或关闭 |
| `completed` | 本次执行完成 |
| `cancelled` | 本次执行取消 |

### 8.2 Transitions

| from | actionType | to | guard |
| --- | --- | --- | --- |
| none | `enqueueRun` | `queued` | Issue 可执行 |
| `queued` | `startRun` | `started` | 已获得执行锁 |
| `started` | `requestRunInput` | `awaitingInput` | 缺少必要人类输入 |
| `awaitingInput` | `provideRunInput` | `started` | 输入已补齐 |
| `started` | `pauseRun` | `paused` | 人类或 Coordinator 暂停 |
| `paused` | `resumeRun` | `started` | 暂停原因解除 |
| `started` | `blockRun` | `blocked` | 依赖、环境或权限阻塞 |
| `blocked` | `resumeRun` | `started` | 阻塞解除 |
| `started` / `blocked` | `failRun` | `failed` | 失败证据已记录 |
| `failed` | `retryRun` | `queued` | 重试策略允许 |
| `started` | `completeRun` | `completed` | 交付和 evidence 完整 |
| `queued` / `started` / `awaitingInput` / `paused` / `blocked` / `failed` | `cancelRun` | `cancelled` | 取消原因已记录 |

### 8.3 Invariants

- `Run.completed` 不等于 `Issue.done`。
- `Run` 必须链接到一个 `Issue`。
- `Run` 必须产出执行日志或 evidence。
- `Run.failed` 必须记录失败原因、复现信息或阻塞证据。
- `Run.cancelled` 必须释放它持有的执行锁。

## 9. Audit State Machine

`Audit` 是独立审计流程。

它不能被 Build Done 自动触发，也不能由 Build Agent 顺手写审计结论。

### 9.1 States

| state | meaning |
| --- | --- |
| `requested` | 已收到独立审计请求 |
| `accepted` | 审计范围已确认 |
| `running` | Audit Agent 正在审计 |
| `reportReady` | 审计报告已生成 |
| `closed` | 审计流程已关闭 |
| `cancelled` | 审计已取消 |

### 9.2 Transitions

| from | actionType | to | guard |
| --- | --- | --- | --- |
| none | `requestAudit` | `requested` | 来源是独立 Audit Issue 或明确人类审计请求 |
| `requested` | `acceptAudit` | `accepted` | 范围、对象、证据入口已明确 |
| `accepted` | `startAudit` | `running` | Audit Agent Role 合法 |
| `running` | `submitAuditReport` | `reportReady` | 报告和 findings 已记录 |
| `reportReady` | `closeAudit` | `closed` | 人类或治理规则关闭 |
| `requested` / `accepted` / `running` | `cancelAudit` | `cancelled` | 取消原因已记录 |

### 9.3 Invariants

- `Issue.done` 不自动转成 `Audit.requested`。
- `Audit` 读取 Build 证据，但不修改 Build 交付事实。
- Audit Agent 可以创建 `Finding`，但不能直接改 Build Agent 的完成事实。
- Audit 结论必须通过事件写入，Projection 负责展示。

## 10. Finding State Machine

`Finding` 表示审计、评审或验证发现的问题。

它可以要求修复，但修复仍应通过 Issue Contract 执行。

### 10.1 States

| state | meaning |
| --- | --- |
| `open` | 发现已创建 |
| `triaged` | 已分类和定级 |
| `fixRequired` | 需要修复 |
| `notActionable` | 不需要行动或不成立 |
| `fixLinked` | 已链接修复 Issue |
| `resolved` | 修复交付已提交 |
| `verified` | 修复已验证 |
| `closed` | Finding 已关闭 |
| `reopened` | Finding 被重开 |

### 10.2 Transitions

| from | actionType | to | guard |
| --- | --- | --- | --- |
| none | `createFinding` | `open` | 来源 Audit 或 Review 合法 |
| `open` | `triageFinding` | `triaged` | 分类、影响、证据完整 |
| `triaged` | `requireFix` | `fixRequired` | 需要修复 |
| `triaged` | `dismissFinding` | `notActionable` | 证据不足或无需行动 |
| `fixRequired` | `linkFixIssue` | `fixLinked` | 已创建或关联修复 Issue |
| `fixLinked` | `markFindingResolved` | `resolved` | 修复 Issue 已交付 |
| `resolved` | `verifyFindingFix` | `verified` | 验证证据通过 |
| `verified` / `notActionable` | `closeFinding` | `closed` | 关闭理由已记录 |
| `closed` / `resolved` / `verified` | `reopenFinding` | `reopened` | 新证据或验证失败 |
| `reopened` | `triageFinding` | `triaged` | 已重新评估 |

### 10.3 Invariants

- `Finding` 不直接修改 `Issue.done`。
- `Finding.fixRequired` 可以派生修复 Issue。
- 修复 Issue 的执行仍走 `Issue` 和 `Run` 状态机。
- `Finding.notActionable` 必须保留判断依据。
- `Finding.closed` 必须可追溯到验证证据或关闭决策。

## 11. Cross-object Rules

对象状态机不能孤立工作。关键跨对象规则如下：

| source | rule |
| --- | --- |
| `Requirement.accepted` | 可以启动 `Spec.drafting` |
| `Spec.approved` | 可以派生 `Issue.proposed` |
| `Issue.ready` | 可以创建或启动 `Run` |
| `Run.completed` | 可以推动 `Issue.reviewReady`，但不能直接保证 `Issue.done` |
| `Issue.done` | 可以作为 Audit 输入，但不自动创建 Audit |
| `Audit.reportReady` | 可以创建 `Finding.open` |
| `Finding.fixRequired` | 可以派生修复 `Issue.proposed` |
| `Finding.verified` | 可以关闭 Finding，但不改写原 Build 事实 |

最重要的边界：

```text
Build completion and Audit are separate flows
Finding drives new work through Issue, not by mutating old facts
```

## 12. Multi-agent Execution Rule

多 Agent 执行必须建立在状态机和仲裁之上。

默认规则：

- 一个 `Issue` 同一时间只有一个 active `Run`；
- 一个 `Run` 只能由一个 owner Agent Role 主执行；
- Reviewer、Auditor、Coordinator 可以并行读取，但不能抢占写权；
- 并发写入必须先获得对象锁或 action scope；
- 冲突 Action Proposal 必须由 Arbitration 决定 accept、reject、queue 或 requireHumanDecision。

多 Agent 不应该被理解为“大家同时改项目”。正确模型是：

```text
多个 Agent 同时提交 Action Proposal
Runtime 按对象状态、角色权限、依赖和锁顺序仲裁
Event Store 只记录被接受的事实
```

## 13. Event Emission

所有状态变化都必须发出事件。

建议事件命名：

```text
RequirementStateChanged
SpecStateChanged
IssueStateChanged
RunStateChanged
AuditStateChanged
FindingStateChanged
```

事件必须包含：

```text
eventId
eventType
objectId
objectType
fromState
toState
actionType
actorRole
causationId
correlationId
evidenceRefs
timestamp
ontologyVersion
```

状态事件不是命令。
状态事件只记录已经通过仲裁的事实。

## 14. Projection Output

Projection Surface 不应该重新发明状态。

它只能从 Event Store 和 Object State Machines 推导读模型：

| read model | input |
| --- | --- |
| Requirement Intake View | Requirement state + clarification links |
| Spec Preview View | Spec state + confirmation decision |
| Project Home | Issue states + dependency links + latest Run |
| Task Workbench | active Issue + active Run + evidence |
| Audit Surface | Audit state + Findings + evidence graph |
| Delivery Package | Issue done state + artifacts + evidence |

UI 命令必须回到 Runtime API：

```text
Projection Surface command
→ Runtime API
→ Action Proposal
→ Arbitration
→ Event Store
```

## 15. Validation Checklist

正式实现前，每个状态机定义必须通过这些检查：

- 所有状态名称唯一；
- `initialState` 存在于 `states`；
- `terminalStates` 存在于 `states`；
- 每个 transition 的 `from` 和 `to` 都存在；
- 每个 transition 的 `actionType` 已注册；
- 每个 `actionType` 有 Action Contract；
- 每个 `rolePolicy` 已注册；
- 需要 evidence 的 transition 明确 evidence 类型；
- 会创建 link 的 transition 明确 link type；
- 跨对象状态变化必须通过事件关联，不允许隐式级联写状态。

## 16. Non-goals

本草案不定义：

- 当前 v0.3.0 审计任务；
- `.agentflow/spec/**` 事实文件；
- 数据库表结构；
- 具体前端页面字段；
- 具体 CLI 命令；
- 具体 Build Agent 执行授权；
- 现有项目状态迁移脚本。

## 17. Next

第五份前置草案已经形成：

```text
AGENTFLOW_ARCHITECTURE_DECISION_RECORD_DRAFT_V1.md
```

它用于沉淀这些不可反复摇摆的架构决策：

- Event Store 是事实权威；
- Agent 不直接改状态；
- UI 不直接写事实；
- Audit 独立于 Build Done；
- Object State Machine 是多 Agent 执行的状态基础；
- Action Arbitration 是多 Agent 写入的唯一入口；
- Project Ontology 是行业输入标准化层。
