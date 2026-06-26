# 018 - Project Status Mutation Writer V1

创建日期：2026-06-17
执行者：Codex

## Source Foundation

本文从以下文档派生：

- [../product/005-goal-to-project-loop-flow-v1.md](../product/005-goal-to-project-loop-flow-v1.md)
- [016-project-completion-decision-v1.md](016-project-completion-decision-v1.md)
- [017-project-status-mutation-boundary-v1.md](017-project-status-mutation-boundary-v1.md)

## Purpose

本文定义 Project Status Mutation Writer 的边界。

Project Status Mutation Writer 是真正写入 Project status 的唯一 writer。它只接受 ProjectStatusMutationProposal，并在写入前重新校验 Completion Decision、Project 当前状态、active run / lease、blocking audit、delivery、evidence、risk 等事实。

本文仍是 foundation 规划文档，不实现 writer。

它回答：

- writer 可以写哪些字段？
- writer 绝不能写哪些内容？
- 写入前必须重新检查哪些事实？
- 重复执行同一个 proposal 如何保持幂等？
- 写入失败时如何保留 evidence？
- 写入成功后如何留下可审计记录？
- 如何防止绕过 ProjectCompletionDecision 直接写 `completed`？

## Position In Flow

```text
Project Completion Evaluation
-> Completion Candidate
-> Project Completion Decision
-> Project Status Mutation Boundary
-> ProjectStatusMutationProposal
-> Project Status Mutation Writer
-> ProjectStatusMutationEvidence
```

Writer 是 Project status mutation 的唯一写入口。

## Writer Authority

Project Status Mutation Writer 只能处理已通过边界检查的 proposal。

合法入口必须是：

```text
ProjectStatusMutationProposal
```

不允许直接从以下对象写入 Project status：

```text
WorkLoopRun
AuditResult
DeliveryReport
ProjectCompletionEvaluation
ProjectCompletionCandidate
ProjectCompletionDecision
HumanAcceptanceNote
```

这些对象只能作为 writer 的输入证据。

## Inputs

Writer 必须读取：

```text
ProjectStatusMutationProposal
ProjectCompletionDecision
ProjectCompletionEvaluation
ProjectCompletionCandidate
ProjectStatusMutationBoundary
.agentflow/spec/projects/<project-id>.json
.agentflow/spec/issues/<issue-id>.json
AuditResult[]
DeliveryReport[]
WorkLoopRun[]
```

Writer 可以读取：

```text
docs/projects/<project-id>/GOAL.md
docs/projects/<project-id>/PLAN.md
docs/projects/<project-id>/DECISIONS.md
human acceptance notes
goal recheck summaries
deferred work register
risk acceptance notes
```

## Write Targets

未来 writer 允许写入的逻辑目标：

```text
.agentflow/spec/projects/<project-id>.json
.agentflow/completion/status-mutations/evidence/<mutation-id>.json
.agentflow/completion/status-mutations/ledger.jsonl
```

对 Project spec 的写入范围必须限制为：

```text
status
statusUpdatedAt
completedAt
updatedAt
completionDecisionId
statusMutationEvidenceId
```

如果当前 Project schema 没有这些字段，writer 不能临时扩大写入面。

必须先由 schema migration / model contract 独立定义。

## Forbidden Write Targets

Writer 不允许写：

```text
.agentflow/workloop/**
.agentflow/audit/**
.agentflow/delivery/**
.agentflow/projections/**
docs/product/**
docs/foundation/**
docs/requirements/**
source code files
remote GitHub / Linear / PR / release / deploy
```

Writer 不允许修改 Issue status。

Writer 不允许生成新的 Issue。

Writer 不允许触发 run / verify / review / audit / delivery。

## Write Request Object

建议对象：

```text
ProjectStatusMutationWriteRequest
```

字段：

```text
requestId
proposalId
projectId
requestedBy
requestedAt
expectedCurrentStatus
targetStatus
transitionType
dryRun
readonly
```

说明：

- `dryRun = true` 时只返回 write plan，不写事实源。
- `dryRun = false` 时仍必须通过全部 pre-write checks。
- `readonly = true` 表示本对象本身不可直接编辑。

## Write Result Object

建议对象：

```text
ProjectStatusMutationWriteResult
```

字段：

```text
resultId
requestId
proposalId
projectId
previousStatus
newStatus
writeStatus
idempotencyStatus
checksSnapshot
evidencePath
changedFiles
blockedReasons
rollbackGuidance
createdAt
readonly
```

## Write Status

`writeStatus` 取值：

```text
planned
applied
already-applied
blocked
failed
rolled-back
inconclusive
```

### planned

表示 dry-run 成功，生成写入计划。

### applied

表示本次 writer 成功写入 Project status。

### already-applied

表示目标状态已经存在，且对应 evidence 可验证。

### blocked

表示写入前检查阻止执行。

### failed

表示写入过程中失败。

### rolled-back

表示写入失败后已恢复到 previousStatus。

### inconclusive

表示无法确认写入结果。

## Idempotency Status

`idempotencyStatus` 取值：

```text
new-write
same-proposal-already-applied
target-status-already-present
conflicting-proposal
missing-evidence
unsafe-to-repeat
```

规则：

- 同一个 proposal 重复执行，结果必须稳定。
- 如果 Project 已是 targetStatus，且 evidence 匹配同一个 proposal，则返回 `already-applied`。
- 如果 Project 已是 targetStatus，但 evidence 缺失，则不得静默成功。
- 如果 Project 已是 targetStatus，但来源 proposal 不同，则必须 blocked。
- 如果 proposal 已过期，则必须 blocked。

## Pre-write Checks

Writer 写入前必须重新执行以下检查：

| Check | Pass 条件 | Block 条件 |
| --- | --- | --- |
| proposal | proposal 存在且未过期 | proposal 缺失 / 过期 |
| proposal-status | targetStatus = completed | target 非 completed |
| decision | source decision accepted | decision 缺失 / rejected / blocked |
| boundary | boundary eligible | boundary blocked / inconclusive |
| current-status | 当前 status 等于 expectedCurrentStatus | status 已变化且无法幂等确认 |
| active-run | 无 active run | active run 存在 |
| active-lease | 无 active lease | active lease 存在 |
| audit | 无 blocking audit finding | blocking audit finding |
| delivery | required delivery 存在 | delivery missing |
| evidence | required evidence 存在 | evidence missing |
| risk | risk 已通过或已接受 | high risk unresolved |
| deferred-work | 无 blocking deferred work | blocking deferred work |
| human | required human acceptance 已记录 | human acceptance missing |

任何 blocking check 失败时，writer 不能写 Project status。

## Write Sequence

未来 writer 的逻辑顺序必须是：

```text
1. Load ProjectStatusMutationWriteRequest
2. Load ProjectStatusMutationProposal
3. Load ProjectCompletionDecision
4. Load Project current facts
5. Re-run pre-write checks
6. Compute idempotency status
7. Build write plan
8. Write Project status in the allowed field set
9. Write ProjectStatusMutationEvidence
10. Append status mutation ledger entry
11. Return ProjectStatusMutationWriteResult
```

如果第 8 步成功但第 9 步失败，writer 必须进入 recovery path。

不能在没有 evidence 的情况下把写入当成完成。

## Atomicity Boundary

本地文件系统写入无法假设天然事务。

Writer 必须采用可恢复策略：

- 写入前记录 previousStatus。
- 先生成 pending evidence draft。
- 再写 Project status。
- 再 finalize evidence。
- 最后 append ledger。

如果中间失败，必须能判断：

- status 是否已写。
- evidence 是否完整。
- ledger 是否追加。
- 是否需要 rollback 或 recovery。

## Evidence Object

建议对象：

```text
ProjectStatusMutationEvidence
```

字段：

```text
evidenceId
projectId
proposalId
decisionId
boundaryId
previousStatus
newStatus
transitionType
mutationReason
checksSnapshot
acceptedWarnings
acceptedDeferredWork
acceptedRisks
changedFiles
writerVersion
writtenBy
writtenAt
rollbackGuidance
```

Evidence 必须能回答：

- 为什么可以把 Project 写成 `completed`？
- 哪个 Completion Decision 授权了写入？
- 哪个 proposal 被执行？
- 写入前状态是什么？
- 写入后状态是什么？
- 哪些 checks 通过？
- 哪些 warning / deferred work / risk 被接受？
- 写了哪些文件？

## Ledger Entry

建议对象：

```text
ProjectStatusMutationLedgerEntry
```

字段：

```text
entryId
projectId
proposalId
decisionId
previousStatus
newStatus
writeStatus
evidencePath
createdAt
```

Ledger 只追加，不覆盖。

Ledger 用于快速审计 writer 是否执行过。

## Rollback Boundary

rollback 不是默认自动执行。

Writer 只允许在以下情况下自动 rollback：

- status 写入刚刚由同一个 writer 完成。
- previousStatus 已明确记录。
- evidence finalize 失败。
- Project 文件没有被其他进程修改。
- rollback 不会覆盖其他合法写入。

其他情况必须输出：

```text
writeStatus = failed
rollbackGuidance
```

并等待独立 recovery / repair 流程。

## Recovery Path

如果发现 Project 已经是 targetStatus，但 evidence 缺失：

- 不能静默成功。
- 不能重复写 status。
- 必须输出 `missing-evidence`。
- 可以建议生成 recovery evidence。
- recovery evidence 必须独立标记为 reconstructed。

如果发现 Project status 与 proposal 冲突：

- 必须 blocked。
- 必须要求重新运行 Completion Evaluation / Decision / Boundary。

## Concurrency Guard

Writer 必须防止并发写 Project status。

未来实现可以使用本地 writer guard：

```text
.agentflow/completion/status-mutations/locks/<project-id>.json
```

该路径只是逻辑建议，不要求当前实现。

锁只能保护 status mutation writer。

不能替代 Work Loop lease。

## Dry Run Behavior

Writer 必须支持 dry-run。

dry-run 输出：

```text
ProjectStatusMutationWritePlan
preWriteChecks
idempotencyStatus
wouldChangeFiles
requiredEvidence
blockedReasons
```

dry-run 不允许写：

```text
.agentflow/**
docs/**
source code
```

## Failure Modes

Writer 必须区分：

```text
preflight-blocked
proposal-expired
decision-invalid
current-status-changed
write-failed
evidence-write-failed
ledger-write-failed
rollback-failed
inconclusive
```

每一种失败都必须返回 blockedReasons 或 rollbackGuidance。

## Relationship With Status Mutation Boundary

Boundary 负责判断是否可以提出 mutation。

Writer 负责执行已通过的 mutation。

Writer 必须重新运行 boundary 的关键 checks，不能盲信旧 proposal。

## Relationship With Completion Decision

Completion Decision 是 writer 的授权来源。

没有 accepted / accepted-with-warnings decision，不允许 writer。

decision 被取消、拒绝、过期或需要 Goal Recheck 时，writer 必须 blocked。

## Relationship With Project Loop

Project Loop 可以调用 writer 或提示调用 writer。

Project Loop 不能绕过 writer 直接写 Project status。

writer 完成后，Project Loop 可以读取 evidence 判断项目是否已经完成。

## Relationship With Desktop

Desktop 可以读取：

```text
ProjectStatusMutationWriteResult
ProjectStatusMutationEvidence
ProjectStatusMutationLedgerEntry
```

Desktop 不应直接执行 writer。

Desktop 不应直接写 Project status。

## Not In Scope

本文不定义：

- 当前代码实现。
- Desktop 写入 UI。
- Issue status writer。
- Project archive writer。
- Release writer。
- Deploy writer。
- Remote PR / GitHub / Linear。
- CI / remote automation。
- Model call orchestration。
- Recovery writer 实现。

## Acceptance Criteria

- [ ] Project Status Mutation Writer 的唯一写入口明确。
- [ ] writer inputs 明确。
- [ ] writer allowed write targets 明确。
- [ ] writer forbidden write targets 明确。
- [ ] Project status 写入字段范围明确。
- [ ] pre-write checks 明确。
- [ ] 幂等规则明确。
- [ ] dry-run 行为明确。
- [ ] evidence 对象明确。
- [ ] ledger entry 明确。
- [ ] rollback boundary 明确。
- [ ] recovery path 明确。
- [ ] concurrency guard 边界明确。
- [ ] writer 不绕过 Completion Decision。
- [ ] 本文不实现 writer。
