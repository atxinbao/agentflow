# 019 - Project Terminal Boundaries V1

创建日期：2026-06-17
执行者：Codex

## Source Foundation

本文从以下文档派生：

- [../product/005-goal-to-project-loop-flow-v1.md](../product/005-goal-to-project-loop-flow-v1.md)
- [015-project-completion-evaluation-v1.md](015-project-completion-evaluation-v1.md)
- [016-project-completion-decision-v1.md](016-project-completion-decision-v1.md)
- [017-project-status-mutation-boundary-v1.md](017-project-status-mutation-boundary-v1.md)
- [018-project-status-mutation-writer-v1.md](018-project-status-mutation-writer-v1.md)

## Purpose

本文一次性收口 Project 完成后的终态边界。

它覆盖：

- Archive / 归档边界。
- Retention / 证据保留边界。
- Cancel / 取消边界。
- Reopen / 重开边界。
- Delete / 删除禁区。
- Release / Deploy 边界。
- Follow-up Project / 后续项目边界。
- Historical Readonly View / 历史只读视图边界。
- Terminal Projection / 终态投影边界。

本文不是实现任务。

本文不再把这些边界拆成多个后续文档。

## Position In Flow

```text
Project Completion Evaluation
-> Project Completion Decision
-> Project Status Mutation Boundary
-> Project Status Mutation Writer
-> Project Terminal Boundaries
```

Project Terminal Boundaries 位于 Project status 写入之后或取消路径之后。

它定义终态项目如何被展示、保留、隐藏、引用和后续衍生。

## Terminal Vocabulary

Project canonical status 继续保持：

```text
draft
active
paused
completed
canceled
```

终态相关概念：

```text
completed = 项目完成，事实源状态。
canceled = 项目取消，事实源状态。
archived = 展示层归档标记，不是 canonical status。
retained = 证据保留状态，不是 Project status。
reopened = 重开动作结果，不建议作为 Project status。
deleted = V1 禁止。
```

规则：

- `archived` 不作为新写入的 Project status。
- `deleted` 不作为正常工作流动作。
- `completed` 与 `canceled` 互斥。
- `completed` 不代表已 release / deploy。
- `canceled` 不代表 evidence 可以删除。

## Boundary Inventory

本文定义以下边界：

| Boundary | 作用 | 是否写 Project status |
| --- | --- | --- |
| Archive Boundary | 控制终态项目是否从默认列表隐藏 | 否 |
| Retention Boundary | 控制完成或取消后的证据保留 | 否 |
| Cancel Boundary | 控制项目取消条件和证据 | 是，未来 writer |
| Reopen Boundary | 控制 completed / canceled 后是否允许继续 | 默认否 |
| Delete Boundary | 控制破坏性删除禁区 | 否，V1 禁止 |
| Release / Deploy Boundary | 控制项目完成与外部发布的边界 | 否 |
| Follow-up Project Boundary | 控制后续项目如何从终态项目派生 | 否，生成提案 |
| Historical Readonly Boundary | 控制终态项目只读展示 | 否 |
| Terminal Projection Boundary | 控制终态派生视图 | 否 |

## Common Inputs

终态边界可读取：

```text
.agentflow/spec/projects/<project-id>.json
.agentflow/spec/issues/<issue-id>.json
ProjectStatusMutationEvidence
ProjectStatusMutationLedgerEntry
ProjectCompletionDecision
ProjectCompletionEvaluation
AuditResult[]
DeliveryReport[]
docs/projects/<project-id>/GOAL.md
docs/projects/<project-id>/PLAN.md
docs/projects/<project-id>/DECISIONS.md
```

可选读取：

```text
human archive notes
cancellation notes
follow-up project proposals
release notes
retention policy
```

## Archive Boundary

Archive 是展示层隐藏或降噪动作。

Archive 不等于：

- Project completed。
- Project canceled。
- Project deleted。
- evidence deletion。
- release / deploy。

Archive 允许作用于：

```text
completed
canceled
```

Archive 不建议作用于：

```text
draft
active
paused
```

Archive 只能产生：

```text
ProjectArchiveMarker
ProjectArchiveDecision
ProjectArchiveProjection
```

Archive 不允许修改：

```text
Project.status
Issue.status
AuditResult
DeliveryReport
ProjectCompletionDecision
ProjectStatusMutationEvidence
```

### ProjectArchiveMarker

建议字段：

```text
archiveId
projectId
archiveReason
archivedBy
archivedAt
sourceStatus
readonly
```

Archive Marker 只影响默认视图。

它不能删除事实源。

## Retention Boundary

Retention 定义终态项目必须保留的事实。

完成或取消后必须保留：

```text
GOAL.md
PLAN.md
DECISIONS.md
Project spec
Issue specs
AuditResult
DeliveryReport
ProjectCompletionEvaluation
ProjectCompletionDecision
ProjectStatusMutationEvidence
Status mutation ledger
Accepted warnings
Accepted deferred work
Accepted risks
Cancellation reason
Follow-up project links
```

允许清理的只应是可再生成缓存：

```text
UI cache
temporary projections
search index cache
preview cache
```

不允许清理：

```text
source facts
completion evidence
audit evidence
delivery evidence
human decisions
status mutation evidence
ledger
```

Retention 不写 Project status。

## Cancel Boundary

Cancel 表示项目被明确取消。

Cancel 只能从以下状态提出：

```text
draft
active
paused
```

不允许：

```text
completed -> canceled
canceled -> canceled
```

Cancel 必须记录：

```text
ProjectCancellationDecision
```

建议字段：

```text
cancellationId
projectId
currentStatus
reason
cancelledBy
cancelledAt
blockingWork
preservedEvidence
followupRecommendation
readonly
```

Cancel 可以建议未来 writer 写：

```text
Project.status = canceled
```

但必须使用独立 cancellation writer。

不能复用 completion writer。

Cancel 不能删除已有 evidence。

Cancel 不能把 completed 项目改成 canceled。

## Reopen Boundary

Reopen 表示终态项目被重新拉回执行。

V1 默认不建议重开原 Project。

推荐路径：

```text
completed project
-> Follow-up Project Proposal
-> new Project
```

只有在极少数情况下允许提出 Reopen Proposal：

- Project 刚刚误标 completed。
- status mutation evidence 指向错误 decision。
- completion facts 被证明不成立。
- 用户明确要求恢复原项目而不是创建 follow-up。

Reopen 必须生成：

```text
ProjectReopenProposal
```

建议字段：

```text
reopenProposalId
projectId
sourceStatus
reason
requiredEvidence
recommendedPath
humanConfirmationRequired
readonly
```

`recommendedPath` 取值：

```text
create-follow-up-project
repair-status-mutation
manual-review
blocked
```

Reopen 不直接写 Project status。

Reopen writer 如未来需要，必须独立定义。

## Delete Boundary

V1 禁止 Project hard delete。

不允许删除：

```text
Project spec
Issue spec
GOAL.md
PLAN.md
DECISIONS.md
AuditResult
DeliveryReport
CompletionEvaluation
CompletionDecision
StatusMutationEvidence
Ledger
```

如果用户想“删除项目”，V1 应提供替代路径：

```text
draft project -> cancel
completed project -> archive
canceled project -> archive
wrong project -> mark canceled with reason
```

Hard delete 必须留到独立 destructive operation policy。

本文不定义该 policy。

## Release / Deploy Boundary

Project completed 不等于 release / deploy。

Release / deploy 是外部动作，不属于 Project status mutation。

完成后可以生成：

```text
ReleaseReadinessNote
DeployReadinessNote
```

但不能自动执行：

```text
GitHub Release
tag
deploy
publish
remote PR
production change
```

Release / deploy 必须由后续独立需求或用户明确指令触发。

Project Completion Evidence 可以作为 release input，但不能替代 release approval。

## Follow-up Project Boundary

Follow-up Project 是终态项目继续演进的默认路径。

适用场景：

- completed 项目有 deferred work。
- completed 项目产生新需求。
- canceled 项目拆出新的可行方向。
- audit / delivery 发现后续优化项。
- 用户要扩展原目标。

Follow-up 只能生成：

```text
FollowupProjectProposal
```

建议字段：

```text
proposalId
sourceProjectId
sourceStatus
reason
goalDraft
scopeDraft
nonGoalsDraft
sourceEvidence
recommendedPriority
readonly
```

Follow-up proposal 不自动创建 Project。

仍然必须回到：

```text
Requirement Intake
-> Goal Draft
-> Plan Draft
-> Confirmation Gate
```

## Historical Readonly Boundary

终态项目默认进入历史只读模式。

历史只读模式允许：

```text
view goal
view plan
view issues
view audit
view delivery
view completion evidence
view accepted warnings
view deferred work
view follow-up proposals
```

历史只读模式不允许：

```text
run
verify
review
audit rerun
delivery rerun
status mutation
edit facts
delete evidence
```

如果需要继续工作，必须创建 follow-up project 或进入 reopen proposal。

## Terminal Projection Boundary

终态投影用于 UI 和查询。

允许生成：

```text
ProjectTerminalSnapshot
```

建议字段：

```text
projectId
status
archiveState
retentionState
completionDecision
statusMutationEvidence
acceptedWarnings
acceptedDeferredWork
acceptedRisks
followupProjectProposals
readonly
```

Terminal Snapshot 是派生数据。

它不能替代源事实。

它可以被删除并重新生成。

## Terminal State Matrix

| Project status | 默认视图 | 允许动作 | 禁止动作 |
| --- | --- | --- | --- |
| draft | 工作区 | edit draft / cancel | archive / complete |
| active | 工作区 | project loop / pause / cancel | archive / delete |
| paused | 工作区 | resume / cancel / complete if decision accepted | delete |
| completed | 历史只读 | archive / follow-up proposal | run / delete / cancel |
| canceled | 历史只读 | archive / follow-up proposal | run / complete / delete |

## Blocked Conditions

终态边界必须 blocked，如果：

- Project status 无法读取。
- Project status 与 evidence 冲突。
- archive 尝试修改 Project status。
- retention 尝试删除源事实。
- cancel 尝试取消 completed 项目。
- reopen 尝试绕过 follow-up proposal。
- delete 尝试删除源事实。
- release / deploy 缺少明确用户授权。
- follow-up 尝试直接创建 Project 且未经过 confirmation gate。
- historical view 尝试执行 run / verify / review。

## Write Boundary

本文是边界总文档。

允许未来逻辑写入区域建议：

```text
.agentflow/completion/archive/**
.agentflow/completion/retention/**
.agentflow/completion/cancellation/**
.agentflow/completion/reopen/**
.agentflow/completion/followups/**
.agentflow/projections/terminal/**
```

V1 文档只定义逻辑区域，不要求当前实现立即采用。

本文不允许写：

```text
.agentflow/spec/**
.agentflow/workloop/**
.agentflow/audit/**
.agentflow/delivery/**
docs/product/**
docs/foundation/**
docs/requirements/**
source code
remote systems
```

## Relationship With Previous Foundation Docs

关系链路：

```text
015 Completion Evaluation
  生成完成候选

016 Completion Decision
  确认或拒绝完成候选

017 Status Mutation Boundary
  判断是否允许提出状态写入

018 Status Mutation Writer
  定义未来唯一状态写入器

019 Terminal Boundaries
  定义完成或取消后的归档、保留、重开、删除禁区和后续项目
```

019 不替代 015-018。

019 是完成后生命周期的总边界。

## Remaining Boundary Position

完成本文后，Project terminal lifecycle 的主要边界已经一次性收口。

后续不建议继续按：

```text
archive
retention
cancel
reopen
delete
release
follow-up
```

逐个拆文档。

如果未来要进入实现，应从以下方向拆当前版本需求：

- Project terminal snapshot reader。
- Project archive marker writer。
- Project cancellation decision writer。
- Follow-up project proposal preview。
- Historical readonly Desktop view。

这些是实现切片，不是新的产品边界定义。

## Not In Scope

本文不定义：

- 当前代码实现。
- Project hard delete。
- Release / deploy 执行。
- Remote PR / GitHub / Linear。
- CI / remote automation。
- Model call orchestration。
- Desktop write UI。
- Source code repair。
- Audit rerun。
- Delivery rerun。

## Acceptance Criteria

- [ ] Archive / Retention / Cancel / Reopen / Delete / Release / Follow-up 边界一次性定义。
- [ ] `archived` 不作为 canonical Project status。
- [ ] completed 与 canceled 的边界明确。
- [ ] hard delete 在 V1 禁止。
- [ ] completed 不等于 release / deploy。
- [ ] 终态项目默认历史只读。
- [ ] Follow-up Project 是默认继续演进路径。
- [ ] terminal projection 是派生数据，不替代源事实。
- [ ] 本文不继续拆多个后续边界文档。
- [ ] 本文不实现代码。
- [ ] 本文不写 `.agentflow/` 当前运行态数据。
