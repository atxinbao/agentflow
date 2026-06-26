# AF-OS-006 Event Store Integration Technical Design Draft V1

日期：2026-06-20
执行者：Codex
用途：Next Version Development Preflight / AF-OS-006 技术设计草案
状态：Architecture Draft / 非执行需求 / 不进入当前 v0.3.0 审计 / 不授权 Build Agent 执行

关联文档：

- [AGENTFLOW_RUNTIME_FOUNDATION_TECHNICAL_SUPPORT_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_RUNTIME_FOUNDATION_TECHNICAL_SUPPORT_DRAFT_V1.md)
- [AGENTFLOW_AF_OS_005_ACTION_ARBITRATION_TECHNICAL_DESIGN_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_AF_OS_005_ACTION_ARBITRATION_TECHNICAL_DESIGN_DRAFT_V1.md)
- [AGENTFLOW_ARCHITECTURE_DECISION_RECORD_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_ARCHITECTURE_DECISION_RECORD_DRAFT_V1.md)

说明：本文件只展开 `AF-OS-006` 的技术设计。它不创建 `.agentflow/spec/**` 任务事实，不写 `docs/requirements/**`，不修改源码，也不授权 Build Agent 执行。

## 1. Conclusion

`AF-OS-006` 的目标是把 `AcceptedAction` 转换成可回放、可投影、可审计的 Runtime Event。

核心规则：

```text
Event Store 是事实权威
Action Proposal 不是事实
AcceptedAction 是 append input
Runtime Event 是 append-only fact
Projection 只能读事件，不能写事件
Replay 不能调用 provider
```

## 2. Scope

### 2.1 In Scope

`AF-OS-006` 应覆盖：

- Runtime event envelope；
- accepted action append rule；
- causation / correlation rule；
- idempotency rule；
- event taxonomy；
- replay compatibility rule；
- task-event.v2 compatibility mapping；
- Event Store append boundary。

### 2.2 Out Of Scope

`AF-OS-006` 不做：

- 替换现有 Event Store；
- 引入 Message Bus 作为事实源；
- 覆盖历史事件；
- Projection 写回事件；
- provider 调用；
- Build Agent 执行；
- Audit Agent 执行。

## 3. Existing Base

复用现有：

```text
crates/event-store
TaskEvent
ReplayFilter
idempotencyKey
event store lock
consumer / dead-letter pattern
```

本 issue 是扩展 envelope 和 append contract，不是重写存储。

## 4. Runtime Event Envelope

建议新增或明确字段：

```text
eventId
eventType
schemaVersion
ontologyVersion
actionContractVersion
rolePolicyVersion
stateMachineVersion
actionProposalId
acceptedActionId
actionType
actorRole
objectType
objectId
fromState
toState
evidenceRefs
artifactRefs
decision
causationId
correlationId
idempotencyKey
occurredAt
recordedAt
payload
```

字段含义：

- `causationId` 指向触发本事件的 proposal / accepted action / prior event；
- `correlationId` 串联一次需求、Issue、Run 或 Audit 链路；
- `ontologyVersion` 等定义版本用于 replay compatibility；
- `payload` 只保存事件必要事实，不保存 UI 派生状态。

## 5. Event Taxonomy

MVP event types：

```text
RequirementSubmitted
RequirementNormalized
RequirementClassified
SpecDrafted
SpecApproved
ProjectCreated
IssueCreated
IssueActivated
RunStarted
EvidenceSubmitted
ArtifactSubmitted
IssueMarkedDone
DecisionRecorded
AuditRequested
FindingCreated
FixIssueLinked
ObjectStateChanged
ActionRejectedRecorded
```

注意：

- `ActionRejectedRecorded` 只用于记录需要审计的拒绝事实；
- 普通 rejected decision 可以不进入 Event Store，除非产品需要追踪；
- `ObjectStateChanged` 可作为通用状态事件，但不能替代具体领域事件。

## 6. Append Rules

append 必须满足：

```text
1. acceptedActionId exists
2. actionProposalId exists
3. actionType exists
4. objectType exists
5. objectId exists when target is existing object
6. idempotencyKey is present
7. causationId is present
8. correlationId is present
9. expected event type matches Action Contract
10. state transition matches Object State Machine when from/to state exists
```

禁止：

```text
append event from raw Agent message
append event from UI direct write
append event without accepted action
overwrite old event
delete old event as correction
```

修复错误只能追加 compensating event 或新的 corrected event。

## 7. Replay Rules

Replay 必须遵守：

```text
replay reads events only
replay does not call provider
replay does not modify spec contract
replay does not create new actions
replay respects definition versions
replay can emit projection rebuild output
```

Replay 遇到旧事件：

```text
如果能映射到 Runtime Event Envelope，正常读取
如果不能映射，进入 compatibility warning
如果事件损坏，进入 dead-letter / invalid event report
```

## 8. Compatibility Mapping

旧 task event 映射建议：

| old concept | new mapping |
| --- | --- |
| `TaskEvent` | `RuntimeEventEnvelope` |
| task issue id | `objectType=Issue`, `objectId` |
| run id | `objectType=Run`, `objectId` |
| status transition | `ObjectStateChanged` |
| evidence path | `evidenceRefs` |
| artifact path | `artifactRefs` |
| idempotency key | `idempotencyKey` |

规则：

```text
旧事件可读
旧事件不回写
旧事件不强制迁移
Projection 可以兼容读取旧 envelope 和新 envelope
```

## 9. Public API Sketch

后续实现可以提供：

```text
append_accepted_action_event(accepted_action, append_context) -> AppendResult
build_runtime_event_envelope(accepted_action, event_type) -> RuntimeEventEnvelope
validate_runtime_event(event) -> EventValidationReport
replay_runtime_events(filter) -> RuntimeReplayResult
map_task_event_to_runtime_event(task_event) -> CompatibilityRuntimeEvent
```

这些 API 不应接触：

```text
Provider Session
Desktop UI rendering
Action Arbitration decision logic
Projection writeback
```

## 10. Test Plan

后续实现时建议测试：

1. accepted action appends Runtime Event；
2. missing acceptedActionId fails；
3. missing idempotencyKey fails；
4. duplicate idempotencyKey is handled deterministically；
5. event has causationId and correlationId；
6. expected event mismatch fails；
7. replay does not call provider；
8. old TaskEvent maps to compatibility event；
9. old TaskEvent remains append-only；
10. Projection cannot append event；
11. correction appends new event rather than overwriting old event；
12. invalid old event enters compatibility warning。

## 11. Acceptance Criteria

`AF-OS-006` 完成时应满足：

- Event Store 仍是事实权威；
- accepted action 才能 append Runtime Event；
- Runtime Event 包含 causation / correlation；
- Runtime Event 包含 definition version；
- replay 不调用 provider；
- replay 不改 spec contract；
- old task events 能兼容读取；
- Message Bus 不成为事实源；
- Projection 不写回事件。

## 12. Risks

| risk | mitigation |
| --- | --- |
| 重写 Event Store 导致范围失控 | 只扩展 envelope 和 append rule |
| Message Bus 被误认为事实源 | 明确 Message Bus 只是传输基础设施 |
| Replay 产生副作用 | 测试固定 replay read-only |
| 旧事件迁移破坏历史 | MVP 兼容读取，不批量改写 |

## 13. Next

`AF-OS-006` 之后进入：

```text
AF-OS-007 Projection Read Models
```

Projection 只能从 Event Store 和定义层推导 UI / CLI / 行业客户端需要看的读模型。
