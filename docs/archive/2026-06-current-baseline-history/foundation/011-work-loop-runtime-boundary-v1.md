# 011 - Work Loop Runtime Boundary V1

创建日期：2026-06-17
执行者：Codex

## Source Foundation

本文从以下文档派生：

- [../product/005-goal-to-project-loop-flow-v1.md](../product/005-goal-to-project-loop-flow-v1.md)
- [009-issue-preflight-boundary-v1.md](009-issue-preflight-boundary-v1.md)
- [010-work-loop-entry-proposal-v1.md](010-work-loop-entry-proposal-v1.md)

## Purpose

本文定义 Work Loop Runtime 的边界。

Work Loop Runtime 是 AgentFlow 从项目计划进入真实执行的第一层 runtime。它负责承接已确认的 Work Loop Entry Proposal，并把单个 Issue 的执行过程拆成可审计的运行记录。

它必须解决：

- 哪个 Issue 正在执行？
- 由哪个 Agent 角色执行？
- 是否持有 lease？
- 是否有执行前 checkpoint？
- 是否有 patch / command / validation 结果？
- 是否留下 evidence draft？
- 当前 run 是否 blocked / ready-for-review / completed？
- 哪些结果可以回传给 Project Loop？

Work Loop Runtime 不是 Project Loop，也不是 Audit / Delivery。

## Position In Flow

```text
Project Loop Scheduler
-> Issue Preflight
-> Work Loop Entry Proposal
-> Work Loop Runtime
-> Review / Audit / Delivery
```

Work Loop Runtime 只执行一个已确认的 Issue。

它不重新规划项目，不选择下一个 Issue，不判断项目是否完成。

## Runtime Principle

V1 必须遵守：

- Single Issue Run：一次 run 只对应一个 Issue。
- WIP=1：同一 Project 默认只允许一个 code-changing Issue 持有 active lease。
- Role-bound：run 必须绑定一个 Agent 角色。
- Contract-bound：run 只能使用 Work Loop Entry Proposal 中授权的 scope / files / commands。
- Evidence-first：任何完成声明必须有 evidence draft。
- Audit-separated：run 不生成最终审计报告。
- Delivery-separated：run 不生成最终交付报告。
- Project-state-separated：run 不直接修改 Project / Issue 的最终状态。
- Append-only preferred：runtime 记录优先追加，不覆盖历史。

## Inputs

Work Loop Runtime 读取：

```text
WorkLoopEntryProposal(status=confirmed)
.agentflow/spec/projects/<project-id>.json
.agentflow/spec/issues/<issue-id>.json
docs/projects/<project-id>/GOAL.md
docs/projects/<project-id>/PLAN.md
docs/projects/<project-id>/DECISIONS.md
```

可选读取：

```text
related source files
related evidence summaries
related delivery summaries
```

## Runtime Layers

Work Loop Runtime 拆成以下层：

```text
Run Record
Lease Record
Checkpoint Record
Patch Record
Command Result
Validation Result
Evidence Draft
Review Handoff
```

每一层只负责自己的事实。

不允许把多层事实混入同一个自由文本日志。

## WorkLoopRun

建议对象：

```text
WorkLoopRun
```

字段：

```text
runId
projectId
issueId
sourceProposalId
requiredAgentRole
riskLevel
status
leaseId
checkpointIds
patchIds
commandResultIds
validationResultIds
evidenceDraftId
reviewHandoffId
blockedReasons
createdAt
updatedAt
readonlySummary
```

## Run Status

```text
created
awaiting-lease
leased
planning
patch-proposed
patch-applied
validating
validation-failed
evidence-draft-ready
ready-for-review
blocked
cancelled
completed
```

### created

run 已创建，但尚未进入 lease gate。

### awaiting-lease

run 需要 lease，但还未持有。

### leased

run 已持有 active lease。

### planning

Agent 正在根据 Issue Contract 制定执行计划。

### patch-proposed

已生成拟议改动，但尚未确认应用或记录为 applied。

### patch-applied

改动已应用到工作区。

### validating

正在执行或记录验证。

### validation-failed

验证失败，必须留下失败原因和下一步建议。

### evidence-draft-ready

已生成 evidence draft，但尚未 review。

### ready-for-review

run 可以进入 Review / Audit 入口。

### blocked

run 因边界、验证、依赖、权限或环境问题阻塞。

### cancelled

run 被明确取消。

### completed

run 完成本次执行闭环。

`completed` 不等于 Issue Done，不等于 Project Done。

## State Transition Boundary

允许的核心状态流转：

```text
created
-> awaiting-lease
-> leased
-> planning
-> patch-proposed
-> patch-applied
-> validating
-> evidence-draft-ready
-> ready-for-review
-> completed
```

失败路径：

```text
validating -> validation-failed -> blocked
planning -> blocked
patch-proposed -> blocked
patch-applied -> blocked
```

取消路径：

```text
created -> cancelled
awaiting-lease -> cancelled
leased -> cancelled
planning -> cancelled
```

禁止：

- `created -> completed`
- `created -> patch-applied`
- `patch-applied -> completed` 且无 validation / evidence
- `completed -> patch-applied`
- `blocked -> completed` 且无解除记录

## Lease Record

建议对象：

```text
WorkLoopLease
```

字段：

```text
leaseId
projectId
issueId
runId
ownerRole
status
scope
createdAt
expiresAt
releasedAt
releaseReason
```

状态：

```text
active
released
expired
blocked
```

规则：

- code-changing run 必须持有 active lease 才能进入 `patch-applied`。
- 同一 Project 默认只允许一个 active code-changing lease。
- released lease 不阻断未来 run。
- corrupted / unreadable lease 必须 blocked。
- stale lease 只能提示，不自动危险恢复。

## Checkpoint Record

建议对象：

```text
WorkLoopCheckpoint
```

字段：

```text
checkpointId
runId
type
source
summary
changedFilesBefore
createdAt
rollbackHint
```

类型：

```text
before-change
after-change
validation
blocked
handoff
```

规则：

- code-changing run 必须先有 `before-change` checkpoint。
- checkpoint 不应复制完整源码树。
- checkpoint 应记录可审计摘要、diff anchor 或恢复线索。

## Patch Record

建议对象：

```text
WorkLoopPatch
```

字段：

```text
patchId
runId
status
proposedPatchPath
appliedPatchPath
worktreeDiffPath
changedFiles
blockedFiles
createdAt
appliedAt
```

状态：

```text
proposed
applied
rejected
blocked
```

规则：

- proposed patch 和 applied patch 必须区分。
- patch 只能触碰 proposal 授权范围。
- forbiddenFiles 命中必须 blocked。
- applied patch 必须有 worktree diff 记录。
- patch record 不负责 review / audit。

## Command Result

建议对象：

```text
WorkLoopCommandResult
```

字段：

```text
commandId
runId
command
source
status
exitCode
startedAt
endedAt
outputSummary
logPath
failureReason
```

状态：

```text
passed
failed
skipped
blocked
not-run
```

规则：

- 命令必须来自 validationPlan。
- 不允许隐藏失败。
- skipped 必须说明原因。
- 命令输出可以摘要化，但必须能追溯到 log。

## Validation Result

建议对象：

```text
WorkLoopValidationResult
```

字段：

```text
validationId
runId
requiredCommands
passedCommands
failedCommands
skippedCommands
status
summary
blockedReasons
```

状态：

```text
passed
failed
partial
blocked
```

规则：

- required command 失败时，validation 不能 passed。
- partial 不能直接进入 completed。
- failed 必须生成 blocked reason 或 repair recommendation。

## Evidence Draft

建议对象：

```text
WorkLoopEvidenceDraft
```

字段：

```text
evidenceDraftId
runId
projectId
issueId
scopeSummary
nonGoalsPreserved
changedFilesSummary
validationSummary
knownLimitations
rollbackPlan
boundaryEvidence
createdAt
```

规则：

- evidence draft 是 run 的产物，不是最终审计报告。
- evidence draft 不能直接标记 Issue Done。
- evidence draft 必须能被 Audit Agent 读取。
- 缺少 validationSummary 时不能 ready-for-review。

## Review Handoff

建议对象：

```text
WorkLoopReviewHandoff
```

字段：

```text
handoffId
runId
targetRole
summary
requiredReview
auditRecommended
deliveryRecommended
blockedReasons
```

`targetRole` 可以是：

```text
audit-agent
delivery-agent
goal-agent
human
```

Review Handoff 只推荐下一步，不执行下一步。

## Runtime Write Boundary

Work Loop Runtime 可以写 runtime 事实，但必须限制在未来明确的 runtime 区域。

建议逻辑区域：

```text
.agentflow/workloop/runs/**
.agentflow/workloop/leases/**
.agentflow/workloop/checkpoints/**
.agentflow/workloop/patches/**
.agentflow/workloop/commands/**
.agentflow/workloop/evidence-drafts/**
```

V1 文档只定义逻辑区域，不要求当前实现立即采用该目录结构。

禁止写：

```text
docs/product/**
docs/foundation/**
docs/requirements/**
docs/projects/**  # 除非后续对应角色明确授权
.agentflow/spec/** # Work Loop 不改 spec
.agentflow/projections/** # projection 由 projector 派生
```

## Source Code Boundary

Work Loop Runtime 本身不代表所有 run 都能修改源码。

只有满足以下条件时，后续执行器才允许修改源码：

- WorkLoopEntryProposal 已确认。
- Issue Contract 授权 write/create 路径。
- active lease 存在。
- before-change checkpoint 已存在。
- forbiddenFiles 未命中。
- risk gate 已处理。

## Relationship With Project Loop

Project Loop 选择方向。

Work Loop Runtime 执行单个 Issue。

Work Loop Runtime 完成后，只能把结果回传给 Project Loop：

```text
run summary
evidence draft
review handoff
blocked reason
```

Project Loop 决定：

- 是否重新调度。
- 是否进入 Audit。
- 是否进入 Delivery。
- 是否需要 Goal Recheck。
- 是否推荐下一个 Issue。

## Relationship With Audit / Delivery

Work Loop Runtime 不生成最终 Audit Report。

Work Loop Runtime 不生成最终 Delivery Report。

它只生成：

- Evidence Draft。
- Review Handoff。
- Validation Summary。

Audit Agent 读取这些事实后，才能生成 audit result。

Delivery Agent 读取 audit / evidence 后，才能生成 delivery result。

## Not In Scope

本文不定义：

- Project Loop Scheduler 实现。
- Issue Preflight 实现。
- Work Loop Entry Proposal 实现。
- Audit Report writer。
- Delivery Report writer。
- Goal Recheck writer。
- Remote PR / GitHub / Linear。
- Deploy / release。
- Model call orchestration。
- Desktop write UI。
- 当前版本目录迁移。

## Acceptance Criteria

- [ ] Work Loop Runtime 的位置和职责明确。
- [ ] runtime layers 明确。
- [ ] WorkLoopRun 字段明确。
- [ ] run status 和禁止流转明确。
- [ ] lease / checkpoint / patch / command / validation / evidence / handoff 分层明确。
- [ ] completed 不等于 Issue Done / Project Done。
- [ ] Work Loop 不选择下一个 Issue。
- [ ] Work Loop 不生成最终 Audit / Delivery。
- [ ] Work Loop 不改 SpecProject / SpecIssue。
- [ ] Runtime write boundary 明确。
- [ ] Source code write 条件明确。
- [ ] 不引入远程 PR / CI / SaaS 能力。
