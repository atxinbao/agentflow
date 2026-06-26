# 010 - Work Loop Entry Proposal V1

创建日期：2026-06-17
执行者：Codex

## Source Foundation

本文从以下文档派生：

- [../product/005-goal-to-project-loop-flow-v1.md](../product/005-goal-to-project-loop-flow-v1.md)
- [008-project-loop-scheduler-boundary-v1.md](008-project-loop-scheduler-boundary-v1.md)
- [009-issue-preflight-boundary-v1.md](009-issue-preflight-boundary-v1.md)

## Purpose

本文定义 Work Loop Entry Proposal 的边界。

Work Loop Entry Proposal 位于 Issue Preflight 之后，负责把通过 preflight 的候选 Issue 打包成后续 Work Loop 可以理解的执行入口提案。

它不是 Work Loop。

它回答：

- 哪个 Project / Issue 被建议进入执行入口？
- 这个 Issue 对应哪个 Agent 角色？
- 本次执行允许读取和修改哪些范围？
- 需要哪些验证命令？
- 需要留下哪些 evidence？
- 需要哪类 lease / checkpoint / result writer？
- 是否还需要人工确认？
- 如果不允许进入 Work Loop，阻塞原因是什么？

## Position In Flow

```text
Project Loop Scheduler
-> Issue Preflight
-> IssuePreflightResult(status=passed)
-> Work Loop Entry Proposal
-> Work Loop Entry Confirmation
```

Work Loop Entry Proposal 是执行前的结构化 handoff。

它只准备执行上下文，不执行。

## Inputs

Work Loop Entry Proposal 读取：

```text
IssuePreflightResult
.agentflow/spec/projects/<project-id>.json
.agentflow/spec/issues/<issue-id>.json
docs/projects/<project-id>/GOAL.md
docs/projects/<project-id>/PLAN.md
docs/projects/<project-id>/DECISIONS.md
```

可选读取：

```text
.agentflow/events/**
.agentflow/projections/**
docs/projects/<project-id>/PROJECT_HEALTH.md
docs/projects/<project-id>/EVIDENCE.md
docs/projects/<project-id>/DELIVERY.md
```

## Entry Conditions

生成 Work Loop Entry Proposal 前必须满足：

- IssuePreflightResult 存在。
- `IssuePreflightResult.status = passed`。
- `IssuePreflightResult.requiredNextGate = work-loop-entry`。
- Project / Issue 仍存在。
- Issue 仍是 Scheduler 推荐链路中的候选。
- Issue Contract 没有在 preflight 后被修改。
- 没有新增 human confirmation blocker。

如果上述条件不满足，必须返回 blocked proposal。

## Proposal Object

建议对象：

```text
WorkLoopEntryProposal
```

字段：

```text
proposalId
projectId
issueId
sourcePreflightId
issueTitle
issueGoal
requiredAgentRole
riskLevel
executionIntent
allowedInputs
allowedFiles
forbiddenFiles
forbiddenActions
validationPlan
evidencePlan
checkpointPlan
leasePlan
expectedOutputs
humanConfirmationRequired
blockedReasons
nextRecommendedAction
status
readonly
```

## Proposal Status

```text
draft
ready-for-confirmation
needs-human-confirmation
blocked
rejected
confirmed
```

### draft

表示 proposal 已生成，但还未完成结构化检查。

### ready-for-confirmation

表示 proposal 可以展示给用户或 Project Brain 确认。

它仍然不代表自动执行。

### needs-human-confirmation

表示 proposal 需要用户确认风险、范围或执行边界。

### blocked

表示 proposal 不能进入确认。

### rejected

表示用户或 Project Brain 拒绝该执行入口。

### confirmed

表示该 proposal 被确认，可以交给未来的 Work Loop Entry 实现。

V1 只定义状态语义，不实现确认写入。

## Execution Intent

`executionIntent` 描述后续 Work Loop 的目标，但不能包含具体代码改动。

字段：

```text
summary
scope
nonGoals
successCriteria
validationRequired
evidenceRequired
```

规则：

- 只能来自 SpecIssue / IssuePreflightResult。
- 不能由 Work Loop Entry Proposal 自行扩展 scope。
- 不能新增未确认的 non-goals 例外。
- 不能把多个 Issue 合并成一次执行。

## Required Agent Role

Work Loop Entry Proposal 必须明确后续执行应交给哪个 Agent 角色。

允许：

```text
spec-agent
build-agent
audit-agent
delivery-agent
goal-agent
```

V1 默认：

- 实现类 Issue -> `build-agent`
- 规格修订类 Issue -> `spec-agent`
- 审计类 Issue -> `audit-agent`
- 交付类 Issue -> `delivery-agent`
- 目标重检类 Issue -> `goal-agent`

如果角色无法判断，proposal 必须 blocked 或 needs-human-confirmation。

## Allowed Inputs

`allowedInputs` 定义后续 Agent 可以读取的输入。

建议字段：

```text
projectSpecPath
issueSpecPath
goalPath
planPath
decisionsPath
relatedEvidencePaths
relatedDeliveryPaths
relatedSourceAreas
```

Allowed Inputs 只代表读取授权，不代表写入授权。

## Allowed Files

`allowedFiles` 来自 Issue Contract。

格式建议：

```text
path
permission
reason
```

`permission` 取值：

```text
read
write
create
```

规则：

- 没有明确授权的文件默认不可写。
- `.agentflow/` 默认不可由 Build Agent 写入。
- `docs/projects/**` 默认只允许 Goal / Spec / Audit / Delivery 角色按边界写入。
- forbiddenFiles 优先级高于 allowedFiles。

## Forbidden Files

`forbiddenFiles` 来自 Issue Contract 和系统默认边界。

默认禁止：

```text
.agentflow/runtime/**
.agentflow/events/**
.agentflow/projections/**
.agentflow/execute/**
.codex/**
graphify-out/**
secrets
credentials
production config
unrelated packages
generated artifacts
```

任何 forbidden file 例外都必须回到 Issue Preflight human confirmation。

## Forbidden Actions

默认禁止：

```text
run command
modify source
write .agentflow runtime
create lease
create checkpoint
create patch
write evidence
create PR
merge PR
deploy
release
call model
create remote issue
change project status
advance next issue
```

Work Loop Entry Proposal 只描述这些动作未来是否需要，不执行它们。

## Validation Plan

`validationPlan` 定义后续 Work Loop 应该运行的验证，不在 V1 运行。

字段：

```text
requiredCommands
optionalCommands
manualChecks
expectedResults
failurePolicy
```

规则：

- requiredCommands 必须来自 Issue Contract。
- 如果命令缺失，proposal 必须 blocked。
- 如果命令需要外部权限，proposal 必须 needs-human-confirmation。

## Evidence Plan

`evidencePlan` 定义后续 Work Loop 完成后必须留下的证据，不在 V1 写入。

字段：

```text
requiredEvidence
evidenceFormat
expectedEvidencePaths
reviewNeeds
deliveryNeeds
```

V1 只描述 evidence 要求，不生成 evidence。

## Lease Plan

`leasePlan` 定义后续执行是否必须持有 lease。

字段：

```text
required
scope
ownerRole
conflictPolicy
stalePolicy
```

规则：

- code-changing Issue 必须需要 lease。
- docs-only Issue 可以需要 lease，取决于目标文件是否共享。
- V1 不创建 lease。
- V1 不判断 active lease 的最终占用，只引用 preflight 检查结果。

## Checkpoint Plan

`checkpointPlan` 定义后续 Work Loop 是否需要 checkpoint。

字段：

```text
required
beforeChangeSnapshot
afterChangeSnapshot
rollbackRequired
```

V1 不创建 checkpoint。

## Expected Outputs

`expectedOutputs` 说明未来 Work Loop 可能产出的结果。

允许列出：

```text
changedFilesSummary
validationOutput
evidenceRecord
reviewRecord
deliveryCandidate
rollbackPlan
```

不允许在 proposal 阶段写出这些文件。

## Confirmation Gate

Work Loop Entry Proposal 进入后续执行前必须确认。

确认可以来自：

- 用户显式确认。
- Project Brain 对低风险、边界完整 Issue 的自动确认策略。

V1 只定义确认语义，不实现自动确认策略。

确认前不允许：

- 不创建 run。
- 不创建 lease。
- 不执行命令。
- 不写 patch。
- 不写 evidence。
- 不修改 Issue 状态。

## Readonly Boundary

Work Loop Entry Proposal V1 只读。

允许：

- 读取 IssuePreflightResult。
- 读取 SpecProject / SpecIssue。
- 读取 Project Brain 文档。
- 输出 WorkLoopEntryProposal。
- 输出 blocked reasons。
- 输出 next recommended action。

不允许：

- 不写 `.agentflow/`。
- 不写 `docs/projects/**`。
- 不改 source code。
- 不创建 run。
- 不 acquire lease。
- 不创建 checkpoint。
- 不生成 patch。
- 不运行验证命令。
- 不写 evidence。
- 不启动 Agent。
- 不调用模型。
- 不创建远程 PR / GitHub Issue / Linear Issue。

## Relationship With Issue Preflight

Issue Preflight 负责判断候选 Issue 是否具备进入执行入口的条件。

Work Loop Entry Proposal 负责把通过检查的 Issue 变成后续执行可以消费的 handoff。

二者分工：

| Layer | Responsibility |
| --- | --- |
| Issue Preflight | 检查 Issue 是否可进入执行入口 |
| Work Loop Entry Proposal | 打包执行入口上下文和边界 |
| Work Loop | 后续真正执行，不在本文定义 |

## Relationship With Work Loop

Work Loop Entry Proposal 不执行 Work Loop。

它只允许输出：

```text
WorkLoopEntryProposal(status=ready-for-confirmation)
```

或在确认后进入未来的：

```text
WorkLoopEntry
```

本文不定义 WorkLoopEntry 的 runtime 写入。

## Blocked Conditions

Work Loop Entry Proposal 必须 blocked，如果：

- IssuePreflightResult 缺失。
- IssuePreflightResult 未 passed。
- IssuePreflightResult.requiredNextGate 不是 `work-loop-entry`。
- Project / Issue 已变化且未重新 preflight。
- requiredAgentRole 无法判断。
- allowedFiles 缺失且 Issue 需要改动文件。
- forbiddenFiles 与 allowedFiles 冲突。
- validationPlan 缺失 requiredCommands。
- evidencePlan 缺失 requiredEvidence。
- high-risk issue 未确认。
- proposal 需要新增 scope。

## Output

V1 输出：

```text
WorkLoopEntryProposal
BlockedReasons
NextRecommendedAction
```

不输出：

```text
Run
Lease
Checkpoint
Patch
Command Result
Evidence
Audit Report
Delivery Report
```

## Not In Scope

本文不定义：

- WorkLoopEntry runtime writer。
- Queue promotion。
- Lease acquire / release。
- Checkpoint writer。
- Patch writer。
- Command execution。
- Validation execution。
- Evidence writer。
- Audit execution。
- Delivery generation。
- Desktop write UI。
- Model call。
- Remote PR / GitHub / Linear。

## Acceptance Criteria

- [ ] Work Loop Entry Proposal 的位置和职责明确。
- [ ] Proposal object 字段明确。
- [ ] Proposal status 语义明确。
- [ ] requiredAgentRole 规则明确。
- [ ] allowedInputs / allowedFiles / forbiddenFiles 规则明确。
- [ ] validationPlan / evidencePlan / leasePlan / checkpointPlan 只描述不执行。
- [ ] confirmation gate 明确。
- [ ] `ready-for-confirmation` 不等于执行。
- [ ] V1 不写 `.agentflow/`。
- [ ] V1 不修改 docs。
- [ ] V1 不创建 run。
- [ ] V1 不 acquire lease。
- [ ] V1 不运行命令。
- [ ] V1 不调用模型。
