# 014 - Delivery Entry And Report Model V1

创建日期：2026-06-17
执行者：Codex

## Source Foundation

本文从以下文档派生：

- [../product/005-goal-to-project-loop-flow-v1.md](../product/005-goal-to-project-loop-flow-v1.md)
- [012-review-audit-entry-boundary-v1.md](012-review-audit-entry-boundary-v1.md)
- [013-audit-result-model-v1.md](013-audit-result-model-v1.md)

## Purpose

本文定义 Delivery Entry 和 Delivery Report 的边界。

Delivery Entry 位于 Audit Result 之后，负责判断一个已审计的 Issue / Stage / Delivery Candidate 是否可以进入交付整理。

Delivery Report 是 Delivery Agent 产出的交付摘要，用于让用户理解本次交付了什么、如何验证、证据在哪里、还剩什么风险。

Delivery Report 不是 Project Done。

它回答：

- Audit Result 是否允许进入交付？
- 本次交付的对象是什么？
- 交付内容、验证结果、证据链是否完整？
- 用户需要看到哪些结果？
- 是否还有 deferred work / known limitations？
- 是否建议进入 Goal Recheck / Project Completion Evaluation？

## Position In Flow

```text
Audit Result
-> Delivery Entry
-> Delivery Report
-> Project Loop
-> Goal Recheck / Project Completion Evaluation
```

Delivery 位于 Audit 之后，Project Completion Evaluation 之前。

Delivery 不直接标记 Project Done。

## Delivery Entry Inputs

Delivery Entry 读取：

```text
AuditResult
AuditFinding[]
AuditRequiredFix[]
WorkLoopEvidenceDraft
WorkLoopValidationResult
WorkLoopRun
.agentflow/spec/projects/<project-id>.json
.agentflow/spec/issues/<issue-id>.json
docs/projects/<project-id>/GOAL.md
docs/projects/<project-id>/PLAN.md
```

可选读取：

```text
prior delivery reports
project health summary
stage summary
user acceptance notes
```

## Delivery Entry Conditions

进入 Delivery Entry 前必须满足：

- AuditResult 存在。
- AuditResult.status 是 `passed` 或 `passed-with-warnings`。
- AuditResult.deliveryEligible = true。
- 没有 blocking AuditFinding。
- required fixes 已处理或明确 deferred。
- validation verdict 不是 fail / incomplete。
- evidence verdict 不是 fail / incomplete。
- delivery target 明确。

如果上述条件不满足，Delivery Entry 必须 blocked。

## Delivery Entry Proposal

建议对象：

```text
DeliveryEntryProposal
```

字段：

```text
proposalId
projectId
issueId
runId
sourceAuditResultId
deliveryTarget
deliveryType
readinessStatus
deliveryScope
includedEvidence
includedValidation
includedChanges
knownLimitations
deferredWork
blockedReasons
recommendedNextAction
humanAcceptanceRequired
readonly
```

## Delivery Target

`deliveryTarget` 可以是：

```text
issue
stage
project-slice
delivery-candidate
```

V1 默认从 `issue` 开始。

阶段级和项目切片级交付只定义语义，不要求立即实现。

## Delivery Type

```text
implementation-summary
docs-summary
validation-summary
audit-passed-summary
stage-summary
user-facing-summary
```

V1 默认：

- code-changing run -> `implementation-summary`
- docs-only run -> `docs-summary`
- validation-only run -> `validation-summary`
- stage audit passed -> `stage-summary`

## Delivery Readiness Status

```text
ready-for-delivery
needs-audit-fix
needs-evidence-fix
needs-validation-fix
needs-human-acceptance
blocked
not-ready
```

### ready-for-delivery

表示 Delivery Agent 可以生成 Delivery Report。

不表示用户已验收，也不表示 Project Done。

### needs-audit-fix

表示必须回到 Audit Agent 或对应 owner 处理审计问题。

### needs-evidence-fix

表示缺少交付所需证据。

### needs-validation-fix

表示验证不足以支持交付。

### needs-human-acceptance

表示需要用户确认交付内容、延期项或风险接受。

### blocked

表示存在阻塞，不能生成交付报告。

### not-ready

表示交付输入不足。

## Delivery Report Object

建议对象：

```text
DeliveryReport
```

字段：

```text
deliveryReportId
projectId
issueId
runId
sourceDeliveryEntryProposalId
sourceAuditResultId
deliveryTarget
deliveryType
status
title
summary
deliveredChanges
validationSummary
evidenceSummary
auditSummary
knownLimitations
deferredWork
userImpact
rollbackSummary
nextRecommendedAction
goalRecheckRecommended
projectCompletionEvaluationRecommended
humanAcceptanceRequired
createdAt
readonly
```

## Delivery Report Status

```text
draft
ready-for-review
delivered
delivered-with-warnings
blocked
rejected
```

### draft

Delivery Report 已生成草稿，但还未完成结构化检查。

### ready-for-review

Delivery Report 可以展示给用户或 Project Loop。

### delivered

表示交付报告完整，且没有阻塞风险。

仍然不表示 Project Done。

### delivered-with-warnings

表示交付可接受，但存在 known limitations 或 deferred work。

### blocked

表示交付报告不能成立。

### rejected

表示用户或 Project Loop 拒绝该交付。

## Delivered Changes

建议对象：

```text
DeliveredChange
```

字段：

```text
summary
sourceFiles
behaviorImpact
userVisibleImpact
scopeTrace
```

规则：

- deliveredChanges 必须能追溯到 WorkLoopEvidenceDraft。
- 不允许把未实现的内容写成交付。
- 不允许把 deferred work 写成交付完成。

## Validation Summary

建议对象：

```text
DeliveryValidationSummary
```

字段：

```text
requiredCommands
passedCommands
failedCommands
skippedCommands
manualChecks
status
notes
```

`status` 取值：

```text
passed
passed-with-skips
failed
not-run
```

规则：

- failed 不能生成 `delivered`。
- skipped 必须说明原因。
- not-run 必须导致 blocked 或 needs-human-acceptance。

## Evidence Summary

建议对象：

```text
DeliveryEvidenceSummary
```

字段：

```text
evidenceDraftId
auditResultId
changedFilesSummary
boundaryEvidence
rollbackPlan
knownLimitations
evidenceCompleteness
```

`evidenceCompleteness`：

```text
complete
partial
missing
blocked
```

## Audit Summary

建议对象：

```text
DeliveryAuditSummary
```

字段：

```text
auditResultId
auditStatus
blockingFindings
warningFindings
requiredFixes
deliveryEligibility
```

规则：

- blockingFindings 非空时，Delivery Report 不能 delivered。
- warningFindings 必须出现在 delivered-with-warnings。

## Deferred Work

Delivery Report 必须明确 deferred work。

建议对象：

```text
DeliveryDeferredWork
```

字段：

```text
title
reason
ownerRole
blocking
recommendedFollowup
```

deferred work 不等于完成。

如果 deferred work 是 blocking，则不能 delivered。

## Human Acceptance

Delivery Report 必须要求人工确认，如果：

- delivered-with-warnings。
- 有 deferred work。
- 有 skipped validation。
- 有 high-risk warning。
- user-facing behavior changed。
- rollback plan 需要用户理解。
- Delivery Report 建议进入 Project Completion Evaluation。

人工确认只确认交付报告，不自动标记 Project Done。

## Next Recommended Action

允许输出：

```text
return-to-build-agent
return-to-audit-agent
request-human-acceptance
run-goal-recheck
run-project-completion-evaluation
schedule-next-issue
blocked
```

不允许输出：

```text
mark-project-done
mark-issue-done
deploy
release
create-pr
merge-pr
```

## Project Done Boundary

Delivery Report 不直接决定 Project Done。

Project Done 必须由后续 Project Completion Evaluation 或 Project Loop 派生判断。

Delivery Report 最多只能建议：

```text
projectCompletionEvaluationRecommended = true
```

Project Done 不能只依赖单个 Delivery Report。

至少还需要检查：

- Goal 是否达成。
- Plan 是否全部覆盖。
- Required Issues 是否完成。
- Required Audits 是否通过。
- Required Deliveries 是否存在。
- Deferred work 是否非阻塞。
- Human acceptance 是否完成。

## Write Boundary

Delivery Report 是 Delivery Agent 的产物。

未来允许写入的逻辑区域建议为：

```text
.agentflow/delivery/reports/**
.agentflow/delivery/summaries/**
```

V1 文档只定义逻辑区域，不要求当前实现立即采用该目录结构。

禁止写：

```text
.agentflow/spec/**
.agentflow/workloop/**
.agentflow/audit/**
.agentflow/projections/**
docs/product/**
docs/foundation/**
docs/requirements/**
```

Delivery Agent 不能修改 Audit Result。

Delivery Agent 不能修改 Work Loop facts。

Delivery Agent 不能修改 SpecProject / SpecIssue。

## Relationship With Audit

Audit Result 决定是否可进入 Delivery Entry。

Delivery Report 总结已审计成果。

Delivery Report 不能推翻 Audit Result。

如果发现交付阶段证据冲突，必须：

```text
recommendedNextAction = return-to-audit-agent
```

## Relationship With Project Loop

Project Loop 读取 Delivery Report，用于判断：

- 是否继续下一个 Issue。
- 是否进入 Goal Recheck。
- 是否进入 Project Completion Evaluation。
- 是否要求用户验收。

Project Loop 不能把单个 Delivery Report 直接视为 Project Done。

## Relationship With Goal Agent

Delivery Report 可以建议 Goal Recheck，如果：

- 交付内容与 Goal 有偏差。
- 用户影响与预期不同。
- deferred work 影响成功标准。
- audit warning 影响目标判断。

Delivery Agent 不修改 GOAL.md / PLAN.md。

## Relationship With Remote Release

Delivery Report 不等于 release。

V1 不定义：

- deploy。
- release。
- publish。
- GitHub Release。
- App Store / SaaS 发布。
- production rollout。

如果项目未来需要 release，应独立定义 Release Entry / Release Report。

## Blocked Conditions

Delivery Entry 必须 blocked，如果：

- AuditResult 缺失。
- AuditResult 未 passed / passed-with-warnings。
- AuditResult.deliveryEligible 不是 true。
- 存在 blocking AuditFinding。
- required fixes 未处理。
- evidence summary 缺失。
- validation summary 缺失。
- delivery target 不明确。
- deliveredChanges 无法追溯到 evidence。
- deferred work 是 blocking。

Delivery Report 必须 blocked，如果：

- Delivery Entry Proposal blocked。
- 交付内容与 Audit Result 冲突。
- deliveredChanges 包含未实现内容。
- validation failed。
- evidence missing。
- human acceptance required 但未完成。

## Output

V1 输出：

```text
DeliveryEntryProposal
DeliveryReport
DeliveredChange[]
DeliveryValidationSummary
DeliveryEvidenceSummary
DeliveryAuditSummary
DeliveryDeferredWork[]
RecommendedNextAction
```

不输出：

```text
Project Done
Issue Done
Audit Result
Source Code Patch
Command Result
PR
Release
Deploy
```

## Not In Scope

本文不定义：

- Project Completion Evaluation。
- Goal Recheck writer。
- Release Entry。
- Release Report。
- Deploy / publish。
- Source code repair。
- Validation rerun。
- Audit rerun。
- Project status mutation。
- Issue status mutation。
- Remote PR / GitHub / Linear。
- Model call orchestration。
- Desktop write UI。

## Acceptance Criteria

- [ ] Delivery Entry 和 Delivery Report 分层明确。
- [ ] DeliveryEntryProposal 字段明确。
- [ ] DeliveryReport 字段明确。
- [ ] Delivery readiness status 语义明确。
- [ ] Delivery report status 语义明确。
- [ ] Delivery Report 不等于 Project Done。
- [ ] Delivery Report 不等于 release / deploy。
- [ ] deliveredChanges 必须可追溯到 evidence。
- [ ] validation / evidence / audit summary 明确。
- [ ] deferred work 规则明确。
- [ ] human acceptance 规则明确。
- [ ] Project Done boundary 明确。
- [ ] Delivery Agent 不修改 Audit / Work Loop / Spec facts。
- [ ] V1 不创建 PR / release / deploy。
