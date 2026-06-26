# 017 - Project Status Mutation Boundary V1

创建日期：2026-06-17
执行者：Codex

## Source Foundation

本文从以下文档派生：

- [../product/005-goal-to-project-loop-flow-v1.md](../product/005-goal-to-project-loop-flow-v1.md)
- [015-project-completion-evaluation-v1.md](015-project-completion-evaluation-v1.md)
- [016-project-completion-decision-v1.md](016-project-completion-decision-v1.md)

## Purpose

本文定义 Project Status Mutation Boundary 的边界。

Project Status Mutation Boundary 是 Project Completion Decision 之后的状态写入边界。它定义什么条件下可以提出 Project status mutation，什么条件下可以执行 Project status mutation，以及写入前后必须保留哪些证据。

它不是当前实现任务。

它回答：

- 哪些 ProjectCompletionDecision 可以进入 status mutation proposal？
- 哪些 Project status transition 被允许？
- 哪些状态变化必须被阻止？
- 谁能提出 mutation？
- 谁能执行 mutation？
- 写入前必须重新检查哪些事实？
- 写入后必须产生什么 evidence？
- 如何防止绕过 Completion Decision 直接写 Project completed？

## Position In Flow

```text
Project Completion Evaluation
-> Completion Candidate
-> Project Completion Decision
-> Project Status Mutation Boundary
-> Project Status Mutation Writer
```

Project Status Mutation Boundary 位于 Project Completion Decision 之后。

本文只定义边界，不实现 writer。

## Status Vocabulary

Project status 的 canonical 值建议继续保持：

```text
draft
active
paused
completed
canceled
```

说明：

- `completed` 是事实源状态值。
- `done` 可以作为 UI 或口语表达，但不建议作为新写入的 canonical status。
- `archived` 不作为完成状态写入目标。
- `canceled` 表示取消，不表示完成。

## Inputs

Project Status Mutation Boundary 读取：

```text
ProjectCompletionDecision
ProjectCompletionEvaluation
ProjectCompletionCandidate
docs/projects/<project-id>/GOAL.md
docs/projects/<project-id>/PLAN.md
docs/projects/<project-id>/DECISIONS.md
.agentflow/spec/projects/<project-id>.json
.agentflow/spec/issues/<issue-id>.json
AuditResult[]
DeliveryReport[]
WorkLoopRun[]
```

可选读取：

```text
Project policy
Human acceptance notes
Goal recheck summaries
Deferred work register
Risk acceptance notes
```

## Mutation Boundary Object

建议对象：

```text
ProjectStatusMutationBoundary
```

字段：

```text
boundaryId
projectId
sourceDecisionId
sourceEvaluationId
currentStatus
targetStatus
transitionType
eligibilityStatus
eligibilityChecks
mutationProposal
blockedReasons
requiredEvidence
writerRequired
createdAt
readonly
```

## Transition Type

`transitionType` 取值：

```text
complete-project
pause-project
resume-project
cancel-project
reject-mutation
blocked
```

本文重点定义 `complete-project`。

其他 transition 可以复用同一边界模型，但需要后续独立文档进一步细化。

## Eligibility Status

`eligibilityStatus` 取值：

```text
eligible
not-eligible
needs-human-confirmation
needs-goal-recheck
needs-more-work
blocked
inconclusive
```

### eligible

表示可以生成 ProjectStatusMutationProposal。

仍然不表示已经写入 Project status。

### not-eligible

表示当前决策不能进入 status mutation。

### needs-human-confirmation

表示仍缺少 human acceptance 或风险接受记录。

### needs-goal-recheck

表示必须先回到 Goal Agent / Project Brain。

### needs-more-work

表示必须继续 Project Loop。

### blocked

表示存在阻塞条件。

### inconclusive

表示输入事实不足或互相冲突。

## Completion Mutation Eligibility

Project status 可以被建议从 `active` 或 `paused` 变为 `completed`，必须同时满足：

- 存在 ProjectCompletionDecision。
- `decisionStatus = accepted` 或 `accepted-with-warnings`。
- `decisionType = accept-completion` 或 `accept-with-warnings`。
- `statusMutationRecommended = true`。
- `nextRecommendedAction = propose-project-status-mutation`。
- sourceEvaluation 仍然有效。
- sourceCandidate 仍然有效。
- Project 当前 status 允许完成。
- 没有 active run。
- 没有 active lease。
- 没有 blocking audit finding。
- 没有 blocking deferred work。
- 没有 missing required delivery。
- 没有 missing required evidence。
- 没有 unresolved high risk。
- 没有 required Goal Recheck。
- 没有用户拒绝完成。

## Current Status Rules

允许提出 completion mutation 的 current status：

```text
active
paused
```

不允许直接提出 completion mutation 的 current status：

```text
draft
completed
canceled
```

规则：

- `draft -> completed` 不允许。
- `canceled -> completed` 不允许。
- `completed -> completed` 不需要 mutation。
- `paused -> completed` 只允许在 completion decision 明确接受时提出。

## Target Status Rules

completion mutation 的 target status 必须是：

```text
completed
```

不允许在 completion mutation 中写：

```text
done
archived
closed
released
deployed
```

release / deploy / archive 必须由后续独立流程定义。

## Eligibility Check Model

建议对象：

```text
ProjectStatusMutationEligibilityCheck
```

字段：

```text
name
status
summary
blocking
source
recommendedFix
```

`status` 取值：

```text
pass
warn
fail
incomplete
not-applicable
```

## Required Eligibility Checks

至少需要以下 checks：

| Check | Pass 条件 | Block 条件 |
| --- | --- | --- |
| completion-decision | accepted / accepted-with-warnings | 无 decision 或 decision rejected |
| decision-action | propose-project-status-mutation | 仍需 goal recheck / project loop / human |
| current-status | active / paused | draft / completed / canceled |
| target-status | completed | target 非 completed |
| goal | Goal Verdict pass 或已接受 warning | Goal missing / drift unresolved |
| plan | Plan Verdict pass 或已接受 warning | required stage 未完成 |
| issues | required issues 完成且无 active run / lease | missing issue / active run / active lease |
| audit | audit passed 或 non-blocking warning accepted | blocking finding |
| delivery | required delivery present | delivery missing |
| evidence | required evidence present | evidence missing |
| deferred-work | non-blocking deferred accepted | blocking deferred work |
| risk | risk accepted 或 pass | high risk unresolved |
| policy | project policy 允许 status mutation | policy blocked |

## Mutation Proposal Object

建议对象：

```text
ProjectStatusMutationProposal
```

字段：

```text
proposalId
projectId
sourceDecisionId
currentStatus
targetStatus
transitionType
reason
evidence
warnings
acceptedDeferredWork
acceptedRisks
requiredWriter
expiresAt
createdAt
readonly
```

Mutation Proposal 是写入前的提案。

它不是状态写入。

## Writer Boundary

未来 Project Status Mutation Writer 必须：

- 读取 ProjectStatusMutationProposal。
- 重新读取 ProjectCompletionDecision。
- 重新读取 Project 当前 status。
- 重新检查 active run / active lease。
- 重新检查 blocking audit finding。
- 重新检查 required evidence。
- 写入 Project status 前后生成 evidence。
- 保证写入幂等。
- 防止跳过 Completion Decision。

本文不实现 writer。

## Mutation Evidence

未来 writer 成功后必须产生：

```text
ProjectStatusMutationEvidence
```

建议字段：

```text
evidenceId
projectId
proposalId
decisionId
previousStatus
newStatus
mutationReason
checksSnapshot
writtenBy
writtenAt
changedFiles
rollbackGuidance
```

Mutation Evidence 必须能说明：

- 为什么可以写 `completed`。
- 基于哪个 Completion Decision。
- 写入前状态是什么。
- 写入后状态是什么。
- 哪些 checks 通过。
- 是否存在 accepted warning。
- 是否存在 deferred work。

## Blocked Conditions

Project status mutation 必须 blocked，如果：

- 没有 ProjectCompletionDecision。
- decisionStatus 不是 accepted / accepted-with-warnings。
- statusMutationRecommended 不是 true。
- nextRecommendedAction 不是 propose-project-status-mutation。
- current status 不允许 transition。
- target status 不是 completed。
- sourceEvaluation 过期。
- sourceCandidate 过期。
- required Goal Recheck 未完成。
- required human acceptance 缺失。
- active run 存在。
- active lease 存在。
- blocking audit finding 存在。
- blocking deferred work 存在。
- missing delivery 存在。
- missing evidence 存在。
- high risk unresolved。
- 用户拒绝完成。
- Project facts 互相冲突。

## Allowed Outputs

V1 允许输出：

```text
ProjectStatusMutationBoundary
ProjectStatusMutationEligibilityCheck[]
ProjectStatusMutationProposal
BlockedReasons
RequiredEvidence
```

不允许输出：

```text
Project status write
Issue status write
Project archive
Release
Deploy
Remote PR
```

## Write Boundary

Project Status Mutation Boundary V1 是边界定义。

允许未来写入的逻辑区域建议为：

```text
.agentflow/completion/status-mutations/proposals/**
.agentflow/completion/status-mutations/evidence/**
```

V1 文档只定义逻辑区域，不要求当前实现立即采用该目录结构。

本文不允许写：

```text
.agentflow/spec/**
.agentflow/workloop/**
.agentflow/audit/**
.agentflow/delivery/**
.agentflow/projections/**
docs/product/**
docs/foundation/**
docs/requirements/**
```

本文不修改 Project status。

本文不修改 Issue status。

## Relationship With Completion Decision

Completion Decision 是 Project status mutation 的唯一合法入口。

没有 Completion Decision，不允许 completion mutation。

Completion Decision 被 rejected、cancelled、blocked 时，不允许 mutation。

Completion Decision needs-goal-recheck 时，不允许 mutation。

## Relationship With Project Loop

Project Loop 可以读取 Mutation Boundary 结果：

- 如果 eligible，可以提示进入 writer。
- 如果 blocked，继续 Project Loop 或请求修复。
- 如果 needs-goal-recheck，交给 Goal Agent。
- 如果 needs-human-confirmation，等待用户确认。

Project Loop 不能直接写 Project status。

## Relationship With Goal Agent

Goal Agent 可以读取 Mutation Boundary，用于：

- 判断完成状态是否可写。
- 判断 Goal Recheck 是否仍 required。
- 判断 rejected / blocked 原因。
- 生成项目级总结。

Goal Agent 不应直接写 Project status。

## Relationship With Delivery And Audit

Delivery Report 和 Audit Result 仍是输入证据。

它们不能直接触发 Project status mutation。

必须经过：

```text
Completion Evaluation
-> Completion Decision
-> Status Mutation Boundary
```

## Not In Scope

本文不定义：

- Project Status Mutation Writer 实现。
- Project archive。
- Project release。
- Deploy / publish。
- Issue status mutation。
- Goal Recheck writer。
- Audit rerun。
- Delivery rerun。
- Remote PR / GitHub / Linear。
- Model call orchestration。
- Desktop write UI。

## Acceptance Criteria

- [ ] Project Status Mutation Boundary 的位置和职责明确。
- [ ] Project status canonical vocabulary 明确。
- [ ] `completed` 与 `done` 的边界明确。
- [ ] completion mutation eligibility 明确。
- [ ] current status / target status 规则明确。
- [ ] eligibility checks 明确。
- [ ] ProjectStatusMutationProposal 是提案，不是状态写入。
- [ ] writer boundary 明确。
- [ ] mutation evidence 要求明确。
- [ ] Completion Decision 是唯一合法入口。
- [ ] Audit / Delivery 不能直接触发 Project status mutation。
- [ ] V1 不修改 Project / Issue 状态。
- [ ] V1 不创建 archive / release / deploy / PR。
