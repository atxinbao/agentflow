# 012 - Review Audit Entry Boundary V1

创建日期：2026-06-17
执行者：Codex

## Source Foundation

本文从以下文档派生：

- [../product/005-goal-to-project-loop-flow-v1.md](../product/005-goal-to-project-loop-flow-v1.md)
- [010-work-loop-entry-proposal-v1.md](010-work-loop-entry-proposal-v1.md)
- [011-work-loop-runtime-boundary-v1.md](011-work-loop-runtime-boundary-v1.md)

## Purpose

本文定义 Review / Audit Entry 的边界。

Review / Audit Entry 位于 Work Loop Runtime 之后，负责判断一个已完成或待审查的 run 是否具备进入审计流程的条件。

它不是 Audit Report writer。

它回答：

- Work Loop run 是否留下了足够的 evidence draft？
- validation 是否完整？
- boundary evidence 是否足够？
- changed files 是否在授权范围内？
- 是否存在必须先修复的阻塞？
- 是否应该进入 Audit Agent？
- 是否应该回到 Build Agent 修复？
- 是否应该要求人类确认？

## Position In Flow

```text
Work Loop Runtime
-> Evidence Draft
-> Review Handoff
-> Review / Audit Entry
-> Audit Entry Proposal
-> Audit Agent
```

Review / Audit Entry 是 Work Loop 到 Audit Agent 之间的入口 gate。

它只判断是否可以进入审计，不生成最终审计报告。

## Inputs

Review / Audit Entry 读取：

```text
WorkLoopRun
WorkLoopEvidenceDraft
WorkLoopValidationResult
WorkLoopReviewHandoff
WorkLoopEntryProposal
.agentflow/spec/projects/<project-id>.json
.agentflow/spec/issues/<issue-id>.json
docs/projects/<project-id>/GOAL.md
docs/projects/<project-id>/PLAN.md
```

可选读取：

```text
patch record
command result logs
checkpoint summaries
delivery summaries
prior audit summaries
```

## Entry Conditions

进入 Review / Audit Entry 前必须满足：

- WorkLoopRun 存在。
- WorkLoopRun 对应单个 Issue。
- WorkLoopRun 有 ReviewHandoff。
- Evidence Draft 存在。
- Validation Result 存在或明确说明未运行原因。
- Patch / changed files 有摘要。
- Boundary evidence 存在。
- Run 未处于 active execution 状态。

如果 run 仍在执行中，必须返回 blocked。

## Audit Entry Proposal

建议对象：

```text
AuditEntryProposal
```

字段：

```text
proposalId
projectId
issueId
runId
sourceEvidenceDraftId
sourceValidationResultId
sourceReviewHandoffId
auditTarget
auditType
riskLevel
readinessStatus
evidenceCompleteness
validationCompleteness
boundaryCompleteness
changedFilesSummary
knownLimitations
blockedReasons
recommendedNextAction
requiredAuditRole
humanConfirmationRequired
readonly
```

## Audit Target

`auditTarget` 可以是：

```text
issue-run
stage
delivery-candidate
goal-drift
repair-candidate
```

V1 默认从 `issue-run` 开始。

其他 target 只定义语义，不要求实现。

## Audit Type

`auditType` 可以是：

```text
code-audit
contract-audit
evidence-audit
boundary-audit
delivery-readiness-audit
goal-alignment-audit
```

V1 默认：

- code-changing run -> `code-audit`
- docs-only run -> `contract-audit`
- validation-only run -> `evidence-audit`
- high-risk run -> `boundary-audit`

## Readiness Status

```text
ready-for-audit
needs-build-fix
needs-evidence-fix
needs-validation-fix
needs-human-confirmation
blocked
not-ready
```

### ready-for-audit

表示 Audit Agent 可以读取 proposal 并生成审计结果。

不表示审计已通过。

### needs-build-fix

表示必须回到 Build Agent 修复实现或边界问题。

### needs-evidence-fix

表示实现可能完成，但 evidence draft 不足。

### needs-validation-fix

表示验证失败、缺失或不可解释。

### needs-human-confirmation

表示风险或边界需要人类确认后才能审计。

### blocked

表示存在阻塞，不能进入 Audit Agent。

### not-ready

表示输入资料不足，尚不能判断。

## Readiness Checks

建议对象：

```text
AuditEntryCheck
```

字段：

```text
name
status
message
blocking
source
```

`status` 取值：

```text
passed
warning
blocked
not-applicable
```

## Required Checks

V1 必须检查：

| Check | Passed 条件 | Blocked 条件 |
| --- | --- | --- |
| run-state | run 已 ready-for-review / completed | run 仍在 active execution |
| evidence-draft | Evidence Draft 存在 | 缺失 |
| validation | required validation passed 或失败有解释 | 缺失且无解释 |
| boundary-evidence | boundary evidence 存在 | 缺失 |
| scope | changed files 在 allowed scope 内 | 存在未授权改动 |
| forbidden-files | 未触碰 forbidden files | 命中 forbidden files |
| non-goals | non-goals preserved | 发现越界实现 |
| rollback | rollback plan 存在 | 缺失 |
| known-limitations | limitations 已说明或为空 | 存在未知风险 |
| handoff | ReviewHandoff 存在 | 缺失 |

## Evidence Completeness

建议对象：

```text
AuditEvidenceCompleteness
```

字段：

```text
scopeSummary
changedFilesSummary
validationSummary
boundaryEvidence
rollbackPlan
knownLimitations
commandLogs
checkpointSummary
completenessStatus
missingItems
```

`completenessStatus`：

```text
complete
partial
missing
blocked
```

## Recommended Next Action

允许输出：

```text
start-audit
return-to-build-agent
request-evidence-fix
request-validation-fix
wait-human-confirmation
blocked
```

不允许输出成直接执行命令。

## Required Audit Role

`requiredAuditRole` 默认是：

```text
audit-agent
```

特殊情况：

- 目标偏移 -> `goal-agent`
- 交付前检查 -> `delivery-agent`
- 规格不一致 -> `spec-agent`
- 实现修复 -> `build-agent`

Review / Audit Entry 可以推荐角色，但不启动角色。

## Human Confirmation Rules

Review / Audit Entry 必须要求人工确认，如果：

- forbidden file 被触碰但有例外申请。
- high-risk run validation 未完整通过。
- scope change 未记录。
- evidence 与实际 diff 冲突。
- rollback plan 缺失且改动不可安全撤销。
- Audit Target 从 issue-run 扩展到 stage / delivery / goal-drift。

人工确认只解除审计入口 gate，不代表 audit pass。

## Readonly Boundary

Review / Audit Entry V1 只读。

允许：

- 读取 WorkLoopRun。
- 读取 Evidence Draft。
- 读取 Validation Result。
- 读取 Patch / Command / Checkpoint 摘要。
- 读取 SpecProject / SpecIssue。
- 输出 AuditEntryProposal。
- 输出 blocked reasons。
- 输出 next recommended action。

不允许：

- 不写 `.agentflow/`。
- 不写 `docs/projects/**`。
- 不修改 source code。
- 不修改 Issue status。
- 不标记 run completed。
- 不生成 Audit Report。
- 不生成 Delivery Report。
- 不执行验证命令。
- 不修复 evidence。
- 不启动 Audit Agent。
- 不调用模型。
- 不创建远程 PR / GitHub Issue / Linear Issue。

## Relationship With Work Loop

Work Loop Runtime 产出 run facts。

Review / Audit Entry 检查这些 run facts 是否足够进入审计。

分工：

| Layer | Responsibility |
| --- | --- |
| Work Loop Runtime | 执行单个 Issue，产出 evidence draft / validation / handoff |
| Review / Audit Entry | 检查 run 是否可进入审计 |
| Audit Agent | 后续生成审计结果，不在本文定义 |

Review / Audit Entry 不能回写 Work Loop。

它只能输出：

```text
AuditEntryProposal
RecommendedNextAction
BlockedReasons
```

## Relationship With Project Loop

Project Loop 可以读取 AuditEntryProposal 的摘要，用于判断项目下一步。

但 Project Loop 不能把 `ready-for-audit` 视为完成。

只有后续 Audit Result / Delivery Result / Goal Recheck 才能影响项目级派生状态。

## Relationship With Delivery

Review / Audit Entry 不直接进入 Delivery。

Delivery 必须在 Audit 之后。

允许输出：

```text
delivery-readiness-audit recommended
```

不允许输出：

```text
delivery-ready
delivered
project-complete
```

## Blocked Conditions

Review / Audit Entry 必须 blocked，如果：

- WorkLoopRun 缺失。
- Evidence Draft 缺失。
- Validation Result 缺失且无跳过原因。
- ReviewHandoff 缺失。
- forbidden file 被触碰。
- changedFilesSummary 缺失。
- boundaryEvidence 缺失。
- rollbackPlan 缺失。
- run 仍处于 active execution。
- run 对应多个 Issue。
- audit target 与 Issue Contract 不一致。

## Output

V1 输出：

```text
AuditEntryProposal
AuditEntryCheck[]
AuditEvidenceCompleteness
BlockedReasons
RecommendedNextAction
```

不输出：

```text
Audit Report
Delivery Report
Issue Done
Project Done
Source Code Patch
Command Result
```

## Not In Scope

本文不定义：

- Audit Agent execution。
- Audit Report writer。
- Delivery Report writer。
- Evidence repair writer。
- Validation rerun。
- Source code repair。
- Project status mutation。
- Issue status mutation。
- Remote PR / GitHub / Linear。
- Deploy / release。
- Model call orchestration。
- Desktop write UI。

## Acceptance Criteria

- [ ] Review / Audit Entry 的位置和职责明确。
- [ ] AuditEntryProposal 字段明确。
- [ ] readiness status 语义明确。
- [ ] required checks 明确。
- [ ] ready-for-audit 不等于 audit passed。
- [ ] Review / Audit Entry 不生成 Audit Report。
- [ ] Review / Audit Entry 不生成 Delivery Report。
- [ ] Review / Audit Entry 不修改 Work Loop run。
- [ ] Review / Audit Entry 不修改 Project / Issue 状态。
- [ ] Evidence completeness 结构明确。
- [ ] 与 Work Loop / Project Loop / Delivery 的边界明确。
- [ ] V1 不写 `.agentflow/`。
- [ ] V1 不运行命令。
- [ ] V1 不调用模型。
