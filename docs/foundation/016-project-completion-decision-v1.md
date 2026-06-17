# 016 - Project Completion Decision V1

创建日期：2026-06-17
执行者：Codex

## Source Foundation

本文从以下文档派生：

- [../product/005-goal-to-project-loop-flow-v1.md](../product/005-goal-to-project-loop-flow-v1.md)
- [014-delivery-entry-and-report-model-v1.md](014-delivery-entry-and-report-model-v1.md)
- [015-project-completion-evaluation-v1.md](015-project-completion-evaluation-v1.md)

## Purpose

本文定义 Project Completion Decision 的边界。

Project Completion Decision 是 Completion Candidate 之后的确认决策层。它记录系统完成候选是否被接受、由谁接受、是否带 warning 接受、是否需要 Goal Recheck、是否应该进入后续 Project status mutation proposal。

它不是 Project Done writer。

它回答：

- Completion Candidate 是否被接受？
- 接受方式是 human acceptance 还是 project policy acceptance？
- warnings / deferred work / risk 是否被明确接受？
- 是否需要 Goal Recheck？
- 是否应该继续 Project Loop？
- 是否可以建议进入 Project Status Mutation Boundary？

## Position In Flow

```text
Project Completion Evaluation
-> Completion Candidate
-> Project Completion Decision
-> Project Status Mutation Boundary
```

Project Completion Decision 位于 Completion Candidate 之后。

它只产出完成决策，不直接修改 Project status。

## Inputs

Project Completion Decision 读取：

```text
ProjectCompletionEvaluation
ProjectCompletionCandidate
DeliveryReport[]
AuditResult[]
docs/projects/<project-id>/GOAL.md
docs/projects/<project-id>/PLAN.md
docs/projects/<project-id>/DECISIONS.md
.agentflow/spec/projects/<project-id>.json
.agentflow/spec/issues/<issue-id>.json
```

可选读取：

```text
user acceptance notes
goal recheck summaries
project policy
deferred work register
risk acceptance notes
```

## Completion Decision Object

建议对象：

```text
ProjectCompletionDecision
```

字段：

```text
decisionId
projectId
sourceEvaluationId
sourceCandidateId
decisionType
decisionStatus
acceptanceMode
acceptedBy
acceptedAt
acceptedWarnings
acceptedDeferredWork
acceptedRisks
rejectedReasons
requiredFollowups
goalRecheckRequired
statusMutationRecommended
nextRecommendedAction
createdAt
readonly
```

## Decision Type

`decisionType` 取值：

```text
accept-completion
accept-with-warnings
reject-completion
request-goal-recheck
continue-project-loop
pause-project
blocked
```

### accept-completion

表示 Completion Candidate 被接受，并且没有需要单独接受的 warning / deferred work / risk。

这仍然不表示 Project status 已经变成 done。

### accept-with-warnings

表示 Completion Candidate 被接受，但存在已明确接受的 warning / deferred work / risk。

必须记录接受项。

### reject-completion

表示 Completion Candidate 不被接受。

必须记录 rejectedReasons 和 requiredFollowups。

### request-goal-recheck

表示当前完成候选需要回到 Goal Agent / Project Brain 重新核对目标。

### continue-project-loop

表示项目还应继续进入 Project Loop，而不是进入完成状态。

### pause-project

表示项目应暂停，等待用户或项目策略补齐事实。

### blocked

表示不能形成可靠完成决策。

## Decision Status

`decisionStatus` 取值：

```text
draft
pending-human-acceptance
accepted
accepted-with-warnings
rejected
needs-goal-recheck
needs-more-work
blocked
cancelled
```

### draft

表示决策草稿已生成，但尚未确认。

### pending-human-acceptance

表示需要用户确认才能继续。

### accepted

表示完成候选被接受。

### accepted-with-warnings

表示完成候选带 warning 被接受。

### rejected

表示完成候选被拒绝。

### needs-goal-recheck

表示必须回到 Goal Recheck。

### needs-more-work

表示仍需继续 Project Loop。

### blocked

表示输入事实不足或互相冲突。

### cancelled

表示本次完成决策被用户取消。

## Acceptance Mode

`acceptanceMode` 取值：

```text
human
project-policy
not-accepted
```

### human

用户显式确认。

必须使用 human，如果：

- Completion Candidate 有 warnings。
- 存在 accepted deferred work。
- 存在 high-risk issue。
- 存在 skipped validation。
- 存在 known limitations。
- Goal Recheck 改变过目标或成功标准。
- Delivery Report 要求 human acceptance。
- 项目策略要求人工确认。

### project-policy

项目策略自动接受。

只允许在 clean candidate 下使用。

clean candidate 必须满足：

- Goal Verdict pass。
- Plan Verdict pass。
- Issue Verdict pass。
- Audit Verdict pass。
- Delivery Verdict pass。
- Human Acceptance Verdict not-applicable 或 pass。
- Deferred Work Verdict pass。
- Risk Verdict pass。
- 没有 warnings。
- 没有 blocking deferred work。
- 没有 unresolved high-risk。
- 没有 skipped required validation。
- 没有 active run。
- 没有 active lease。

### not-accepted

表示未接受完成候选。

常见于 rejected、needs-goal-recheck、needs-more-work、blocked。

## Accepted Warning Model

建议对象：

```text
AcceptedCompletionWarning
```

字段：

```text
warningId
source
title
summary
blocking
acceptedBy
acceptedAt
acceptanceReason
```

`blocking = true` 的 warning 不能被普通接受。

blocking warning 必须先被修复、降级为 non-blocking，或进入明确的 scope / goal change。

## Accepted Deferred Work Model

建议对象：

```text
AcceptedDeferredWork
```

字段：

```text
deferredWorkId
sourceIssueId
title
reason
blocking
targetFutureProject
acceptedBy
acceptedAt
```

规则：

- blocking deferred work 不能进入 accept-completion。
- non-blocking deferred work 可以进入 accept-with-warnings。
- deferred work 必须可追踪，不能只写一句“以后再做”。

## Accepted Risk Model

建议对象：

```text
AcceptedCompletionRisk
```

字段：

```text
riskId
riskLevel
source
summary
mitigation
acceptedBy
acceptedAt
```

high risk 必须 human acceptance。

project-policy 不能自动接受 high risk。

## Rejected Reason

`rejectedReasons` 可以包含：

```text
goal-not-met
plan-not-covered
required-issue-missing
audit-blocking
delivery-missing
evidence-incomplete
human-rejected
goal-recheck-required
active-run
active-lease
blocking-deferred-work
inconclusive-facts
```

每个 rejected reason 必须能追溯到输入事实。

## Required Followup

建议对象：

```text
ProjectCompletionFollowup
```

字段：

```text
followupId
type
summary
ownerRole
source
blocking
recommendedNextAction
```

`type` 取值：

```text
goal-recheck
project-loop
audit-fix
delivery-fix
evidence-fix
risk-review
human-acceptance
blocked
```

## Next Recommended Action

允许输出：

```text
propose-project-status-mutation
run-goal-recheck
continue-project-loop
request-audit-fix
request-delivery-fix
request-evidence-fix
wait-human-acceptance
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

## Project Status Mutation Boundary

Project Completion Decision 可以设置：

```text
statusMutationRecommended = true
```

但它不能写：

```text
Project.status = done
Project.status = completed
Project.status = archived
```

后续必须由独立 Project Status Mutation Boundary 定义：

- 谁能写 Project status。
- 写入前需要读取哪些 decision。
- 写入后如何生成 evidence。
- 如何防止绕过 Completion Decision。

## Human Acceptance Rules

Human Acceptance 是完成决策的一部分，不是直接完成项目。

用户确认后只能生成或更新：

```text
ProjectCompletionDecision
```

不能直接触发：

```text
Project Done
Project Archive
Release
Deploy
Remote PR
```

Human Acceptance 必须记录：

- 接受的 candidate。
- 接受的 warnings。
- 接受的 deferred work。
- 接受的 risks。
- 是否仍需 Goal Recheck。
- 下一步建议。

## Project Policy Acceptance Rules

Project Policy Acceptance 是系统策略确认。

它只能处理 clean candidate。

它不能覆盖：

- 用户拒绝。
- blocking audit finding。
- blocking deferred work。
- missing delivery。
- missing evidence。
- high risk unresolved。
- goal drift。
- active run / lease。

如果任何一项存在，必须退回 human 或 blocked。

## Goal Recheck Path

当 `goalRecheckRequired = true` 时：

```text
ProjectCompletionDecision
-> Goal Recheck Proposal
-> Goal Agent / Project Brain
```

Goal Recheck 可能输出：

- Goal still satisfied。
- Goal changed。
- Plan needs update。
- Project needs more work。
- Completion candidate remains valid。

本文不定义 Goal Recheck writer。

## Write Boundary

Project Completion Decision V1 是决策模型定义。

允许未来写入的逻辑区域建议为：

```text
.agentflow/completion/decisions/**
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

Completion Decision 不修改 Project status。

Completion Decision 不修改 Issue status。

## Relationship With Completion Evaluation

Completion Evaluation 负责形成：

```text
ProjectCompletionEvaluation
ProjectCompletionCandidate
```

Completion Decision 负责接受、拒绝或退回该 candidate。

Completion Decision 不重新计算整个项目完成度。

如果 candidate 过期，必须重新运行 Completion Evaluation。

## Relationship With Project Loop

Project Loop 读取 Completion Decision 后，可以：

- 继续调度 Project Loop。
- 请求 Goal Recheck。
- 请求 Audit Fix。
- 请求 Delivery Fix。
- 等待用户确认。
- 进入 Project Status Mutation Boundary。

Project Loop 不能把 Completion Decision 直接当作 Project Done。

## Relationship With Delivery

Delivery Report 是 Completion Decision 的依据之一。

如果 Delivery Report 标记 required human acceptance，则 decision 必须进入 `pending-human-acceptance`，不能使用 project-policy。

## Relationship With Audit

Audit Result 是 Completion Decision 的依据之一。

blocking audit finding 必须阻止 accepted / accepted-with-warnings。

audit warning 可以被 human 接受，但必须记录 accepted warning。

## Relationship With Goal Agent

Goal Agent 可以读取 Completion Decision，用于：

- 判断用户是否接受当前完成候选。
- 判断是否需要 Goal Recheck。
- 判断哪些 deferred work 被接受。
- 判断哪些风险被接受。

Goal Agent 不应直接写 Project Done。

## Blocked Conditions

Completion Decision 必须 blocked，如果：

- 没有 Completion Candidate。
- sourceEvaluation 不存在。
- sourceCandidate 不存在。
- candidate 与 evaluation 不匹配。
- candidate 已过期。
- required human acceptance 缺失。
- warning 未被接受。
- blocking warning 存在。
- blocking deferred work 存在。
- active run 存在。
- active lease 存在。
- audit / delivery 互相矛盾。
- Goal Recheck required 但未完成。
- 用户明确拒绝完成。

## Output

V1 输出：

```text
ProjectCompletionDecision
AcceptedCompletionWarning[]
AcceptedDeferredWork[]
AcceptedCompletionRisk[]
RejectedReasons[]
ProjectCompletionFollowup[]
NextRecommendedAction
```

不输出：

```text
Project Done
Project Status Mutation
Issue Done
Archive
Release
Deploy
Remote PR
```

## Not In Scope

本文不定义：

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

- [ ] Project Completion Decision 的位置和职责明确。
- [ ] ProjectCompletionDecision 字段明确。
- [ ] decision type / decision status / acceptance mode 语义明确。
- [ ] human acceptance 与 project policy acceptance 边界明确。
- [ ] warnings / deferred work / risks 的接受记录明确。
- [ ] rejected reasons 和 required followups 明确。
- [ ] Goal Recheck 路径明确。
- [ ] statusMutationRecommended 只是建议，不是状态写入。
- [ ] Project Completion Decision 不修改 Project / Issue 状态。
- [ ] Project Completion Decision 不创建 release / deploy / PR。
- [ ] Project Completion Decision 不写 `.agentflow/` 当前运行态数据。
