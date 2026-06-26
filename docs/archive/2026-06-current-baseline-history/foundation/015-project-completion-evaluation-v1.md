# 015 - Project Completion Evaluation V1

创建日期：2026-06-17
执行者：Codex

## Source Foundation

本文从以下文档派生：

- [../product/005-goal-to-project-loop-flow-v1.md](../product/005-goal-to-project-loop-flow-v1.md)
- [013-audit-result-model-v1.md](013-audit-result-model-v1.md)
- [014-delivery-entry-and-report-model-v1.md](014-delivery-entry-and-report-model-v1.md)

## Purpose

本文定义 Project Completion Evaluation 的边界。

Project Completion Evaluation 是项目级完成判定器。它读取 Goal、Plan、SpecProject、SpecIssue、Audit Result、Delivery Report、Human Acceptance 等事实，判断项目是否满足“可以被认为完成”的条件。

它不是关闭动作。

它回答：

- Project Goal 是否已经达成？
- Plan 中 required stages 是否完成？
- required issues 是否全部通过 Work Loop / Audit / Delivery？
- 是否存在 blocking deferred work？
- 是否存在未解决 human confirmation？
- 是否需要 Goal Recheck？
- 是否可以生成 Project Completion Candidate？
- 是否仍然 blocked？

## Position In Flow

```text
Delivery Report
-> Project Loop
-> Project Completion Evaluation
-> Completion Candidate
-> Human Acceptance / Goal Recheck / Continue Project Loop
```

Project Completion Evaluation 位于 Delivery Report 之后，也可以由 Project Loop 在任意阶段主动运行。

它只派生完成判断，不直接写 Project Done。

## Inputs

Project Completion Evaluation 读取：

```text
.agentflow/spec/projects/<project-id>.json
.agentflow/spec/issues/<issue-id>.json
docs/projects/<project-id>/GOAL.md
docs/projects/<project-id>/PLAN.md
docs/projects/<project-id>/DECISIONS.md
DeliveryReport[]
AuditResult[]
WorkLoopRun[]
IssuePreflightResult[]
ProjectLoopSchedulerSnapshot
```

可选读取：

```text
docs/projects/<project-id>/PROJECT_HEALTH.md
docs/projects/<project-id>/EVIDENCE.md
docs/projects/<project-id>/DELIVERY.md
user acceptance notes
goal recheck summaries
```

## Completion Evaluation Object

建议对象：

```text
ProjectCompletionEvaluation
```

字段：

```text
evaluationId
projectId
projectTitle
sourceGoalPath
sourcePlanPath
goalVerdict
planVerdict
issueVerdict
auditVerdict
deliveryVerdict
humanAcceptanceVerdict
deferredWorkVerdict
riskVerdict
completionStatus
completionCandidate
blockedReasons
requiredNextAction
goalRecheckRecommended
humanConfirmationRequired
createdAt
readonly
```

## Completion Status

```text
complete-candidate
not-complete
needs-goal-recheck
needs-human-acceptance
needs-more-work
blocked
inconclusive
```

### complete-candidate

表示系统判断 Project 已满足完成候选条件。

仍然不表示：

- Project status 已写成 done。
- 用户已最终接受。
- 项目已归档。
- 远程 release 已完成。

### not-complete

表示项目还未满足完成条件。

### needs-goal-recheck

表示项目成果可能已经交付，但 Goal / Plan 与实际结果需要重新校准。

### needs-human-acceptance

表示系统证据完整，但需要用户确认是否接受完成候选。

### needs-more-work

表示仍有 required issue、required delivery、required audit 或 required evidence 未完成。

### blocked

表示存在阻塞，无法继续判断或无法形成完成候选。

### inconclusive

表示输入事实不足或互相冲突，不能得出可信结论。

## Verdict Model

建议对象：

```text
ProjectCompletionVerdict
```

字段：

```text
status
summary
blocking
evidence
missingItems
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

## Goal Verdict

Goal Verdict 检查：

- GOAL.md 存在。
- Goal 已确认。
- Success Criteria 可追溯到 Delivery Report / Audit Result。
- 当前交付结果覆盖 Goal outcome。
- 没有 unresolved goal drift。

必须 fail / incomplete，如果：

- Goal 缺失。
- Goal 未确认。
- Delivery 与 Goal 不一致。
- Audit Result 建议 Goal Recheck 但未完成。

## Plan Verdict

Plan Verdict 检查：

- PLAN.md 存在。
- Plan 已确认。
- required stages 已覆盖。
- required issue strategy 已落地。
- deferred work 已记录。
- Plan 与实际执行无阻塞偏移。

必须 fail / incomplete，如果：

- required stage 未完成。
- Plan 仍有 unresolved blocker。
- Plan 与 Delivery Report 矛盾。

## Issue Verdict

Issue Verdict 检查：

- required issues 均已完成执行闭环。
- required issues 均有 Audit Result。
- required issues 均有 Delivery Report 或明确无需 delivery。
- cancelled / deferred issues 有原因。
- 没有 active run。
- 没有 active lease。

必须 fail / incomplete，如果：

- required issue 未执行。
- required issue audit failed。
- required issue delivery missing。
- active run / lease 仍存在。
- deferred issue 是 blocking。

## Audit Verdict

Audit Verdict 检查：

- required Audit Result 均存在。
- blocking findings 为空。
- required fixes 已处理或明确 deferred non-blocking。
- audit warnings 已进入 Delivery Report。

必须 fail / incomplete，如果：

- required audit 缺失。
- high / critical finding 未解决。
- Audit Result incomplete / blocked。

## Delivery Verdict

Delivery Verdict 检查：

- required Delivery Report 均存在。
- deliveredChanges 可追溯。
- validation / evidence / audit summary 完整。
- delivered-with-warnings 的 warning 已说明。
- user impact 已说明。

必须 fail / incomplete，如果：

- required delivery 缺失。
- delivery blocked / rejected。
- delivery 包含未实现内容。
- delivery 与 audit 矛盾。

## Human Acceptance Verdict

Human Acceptance Verdict 检查：

- high-risk warning 是否被接受。
- delivered-with-warnings 是否被接受。
- deferred work 是否被接受。
- completion candidate 是否需要用户确认。

必须 needs-human-acceptance，如果：

- Project 有 high-risk changes。
- Delivery Report 要求 humanAcceptanceRequired。
- Goal Recheck 修改过成功标准。
- 用户尚未确认最终结果。

## Deferred Work Verdict

Deferred Work Verdict 检查：

- deferred work 是否均有原因。
- deferred work 是否 blocking。
- deferred work 是否有后续 owner / recommendation。

规则：

- blocking deferred work 不能 complete-candidate。
- non-blocking deferred work 可以 complete-candidate，但必须进入 warning。

## Risk Verdict

Risk Verdict 检查：

- high risk Issue 是否均有 Audit Result。
- security / production / external system 风险是否已确认。
- rollback plan 是否完整。
- known limitations 是否可接受。

必须 fail / needs-human-acceptance，如果：

- high risk unresolved。
- rollback plan missing。
- known limitation 影响 success criteria。

## Completion Candidate

建议对象：

```text
ProjectCompletionCandidate
```

字段：

```text
projectId
goalSummary
completedStages
completedIssues
deliveryReports
auditResults
acceptedDeferredWork
remainingWarnings
completionEvidence
recommendedDecision
```

`recommendedDecision` 可以是：

```text
accept-completion
request-goal-recheck
continue-project-loop
pause-project
blocked
```

Completion Candidate 是系统建议，不是状态写入。

## Required Next Action

允许输出：

```text
accept-completion-candidate
run-goal-recheck
continue-project-loop
request-human-acceptance
request-audit-fix
request-delivery-fix
blocked
```

不允许输出：

```text
mark-project-done
archive-project
release
deploy
create-pr
```

## Project Done Boundary

Project Completion Evaluation 不直接写 Project Done。

Project Done 至少需要后续独立动作：

```text
Completion Candidate
-> Human Acceptance or Project Policy Acceptance
-> Project Completion Decision
-> Project Status Mutation
```

本文只定义 Completion Candidate，不定义 Project Status Mutation。

Project Done 不能只依赖：

- 单个 WorkLoopRun completed。
- 单个 AuditResult passed。
- 单个 DeliveryReport delivered。
- Issue list 全部 done 但没有 evidence。
- 用户一句“完成了”但没有事实链。

## Human Acceptance Rules

必须要求用户确认，如果：

- Completion Candidate 有 warnings。
- 有 accepted deferred work。
- 有 high-risk issue。
- 有 skipped validation。
- 有 known limitations。
- Goal Recheck 改变过目标或成功标准。
- Delivery Report 要求 acceptance。

用户确认只允许进入 Project Completion Decision，不直接写 Project Done。

## Goal Recheck Rules

必须推荐 Goal Recheck，如果：

- Audit Result 标记 goalRecheckRecommended。
- Delivery Report 标记 goalRecheckRecommended。
- Goal success criteria 与实际 delivery 不一致。
- Plan 执行过程出现重大偏移。
- Deferred work 影响目标完整性。
- 用户新增需求改变目标。

Goal Recheck 不在本文定义。

## Write Boundary

Project Completion Evaluation V1 是派生判断。

允许未来写入的逻辑区域建议为：

```text
.agentflow/completion/evaluations/**
.agentflow/completion/candidates/**
```

V1 文档只定义逻辑区域，不要求当前实现立即采用该目录结构。

禁止写：

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

Completion Evaluation 不修改 Project status。

Completion Evaluation 不修改 Issue status。

## Relationship With Project Loop

Project Loop 调用 Completion Evaluation 来判断项目是否可以结束。

Completion Evaluation 只返回：

```text
ProjectCompletionEvaluation
ProjectCompletionCandidate
RequiredNextAction
BlockedReasons
```

Project Loop 再决定：

- 是否继续调度下一个 Issue。
- 是否进入 Goal Recheck。
- 是否请求用户验收。
- 是否进入 Project Completion Decision。

## Relationship With Delivery

Delivery Report 是 Completion Evaluation 的输入。

Delivery Report 不能单独决定项目完成。

Completion Evaluation 需要跨全部 required deliveries 汇总。

## Relationship With Audit

Audit Result 是 Completion Evaluation 的输入。

任何 blocking AuditFinding 都必须阻止 complete-candidate。

Audit warning 可以进入 complete-candidate，但必须要求 human acceptance 或明确接受策略。

## Relationship With Goal Agent

Goal Agent 可以读取 Completion Evaluation，用于：

- 判断是否目标已达成。
- 判断是否需要调整 GOAL.md / PLAN.md。
- 生成 Goal Recheck Proposal。

Completion Evaluation 不修改 GOAL.md / PLAN.md。

## Blocked Conditions

Completion Evaluation 必须 blocked 或 inconclusive，如果：

- SpecProject 缺失。
- GOAL.md 缺失。
- PLAN.md 缺失。
- required issues 无法解析。
- required Audit Result 缺失。
- required Delivery Report 缺失。
- 存在 active run。
- 存在 active lease。
- 存在 blocking deferred work。
- 存在 unresolved high-risk finding。
- Goal / Plan / Delivery 互相冲突。

## Output

V1 输出：

```text
ProjectCompletionEvaluation
ProjectCompletionVerdict[]
ProjectCompletionCandidate
BlockedReasons
RequiredNextAction
```

不输出：

```text
Project Done
Issue Done
Status Mutation
Release
Deploy
Remote PR
```

## Not In Scope

本文不定义：

- Project Completion Decision。
- Project status mutation。
- Project archive。
- Goal Recheck writer。
- Release Entry。
- Release Report。
- Deploy / publish。
- Source code repair。
- Audit rerun。
- Delivery rerun。
- Remote PR / GitHub / Linear。
- Model call orchestration。
- Desktop write UI。

## Acceptance Criteria

- [ ] Project Completion Evaluation 的位置和职责明确。
- [ ] ProjectCompletionEvaluation 字段明确。
- [ ] completion status 语义明确。
- [ ] Goal / Plan / Issue / Audit / Delivery / Human Acceptance / Deferred Work / Risk verdict 明确。
- [ ] Completion Candidate 是建议，不是状态写入。
- [ ] Delivery Report 不直接决定 Project Done。
- [ ] Audit Result 不直接决定 Project Done。
- [ ] WorkLoopRun completed 不直接决定 Project Done。
- [ ] Human Acceptance 规则明确。
- [ ] Goal Recheck 规则明确。
- [ ] Project Done Boundary 明确。
- [ ] V1 不修改 Project / Issue 状态。
- [ ] V1 不创建 release / deploy / PR。
