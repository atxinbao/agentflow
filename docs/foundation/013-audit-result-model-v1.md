# 013 - Audit Result Model V1

创建日期：2026-06-17
执行者：Codex

## Source Foundation

本文从以下文档派生：

- [../product/005-goal-to-project-loop-flow-v1.md](../product/005-goal-to-project-loop-flow-v1.md)
- [011-work-loop-runtime-boundary-v1.md](011-work-loop-runtime-boundary-v1.md)
- [012-review-audit-entry-boundary-v1.md](012-review-audit-entry-boundary-v1.md)

## Purpose

本文定义 Audit Agent 的审计结果模型。

Audit Result 是对一个 Work Loop run、一个 Issue、一个阶段或一个交付候选的独立审查结论。它读取 Review / Audit Entry 产出的 `AuditEntryProposal`，结合 run facts、evidence draft、validation result、boundary evidence，生成可追溯的审计结果。

Audit Result 不是 Delivery Report。

Audit Result 也不直接标记 Issue Done 或 Project Done。

它回答：

- 本次 run 是否遵守 Issue Contract？
- Scope / Non-goals / Boundary 是否被保持？
- Validation 是否可信？
- Evidence 是否足够？
- 是否存在越界改动、缺失证据或阻塞风险？
- 是否可以建议进入 Delivery Entry？
- 是否必须返回 Build Agent / Spec Agent / Goal Agent 修复？

## Position In Flow

```text
Work Loop Runtime
-> Review / Audit Entry
-> AuditEntryProposal
-> Audit Agent
-> Audit Result
-> Project Loop / Delivery Entry
```

Audit Result 位于 Review / Audit Entry 之后，Delivery Entry 之前。

它只给出审计结论和下一步建议，不执行下一步。

## Inputs

Audit Agent 读取：

```text
AuditEntryProposal
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
patch records
command result logs
checkpoint summaries
prior audit results
delivery candidates
```

## Audit Result Object

建议对象：

```text
AuditResult
```

字段：

```text
auditResultId
projectId
issueId
runId
sourceAuditEntryProposalId
auditTarget
auditType
auditorRole
status
summary
contractVerdict
validationVerdict
evidenceVerdict
boundaryVerdict
findings
requiredFixes
recommendedNextAction
deliveryEligible
goalRecheckRecommended
humanConfirmationRequired
createdAt
readonly
```

## Audit Status

```text
passed
passed-with-warnings
failed
needs-fix
incomplete
blocked
needs-human-confirmation
```

### passed

表示审计未发现阻塞问题。

仍然不代表：

- Issue Done。
- Project Done。
- Delivery complete。
- 自动进入下一 Issue。

### passed-with-warnings

表示审计可接受，但存在非阻塞警告或后续注意事项。

### failed

表示存在明确阻塞问题，不能进入 Delivery Entry。

### needs-fix

表示必须回到 Build Agent / Spec Agent / Goal Agent 修复。

### incomplete

表示审计输入不足，无法形成可信结论。

### blocked

表示存在外部阻塞、权限阻塞或不可解析状态。

### needs-human-confirmation

表示审计需要用户确认风险、例外或范围变化。

## Verdicts

Audit Result 必须拆分多个 verdict，避免只给一个模糊结论。

建议对象：

```text
AuditVerdict
```

字段：

```text
status
summary
blocking
evidence
missingItems
```

`status` 取值：

```text
pass
warn
fail
incomplete
not-applicable
```

### Contract Verdict

检查：

- Issue goal 是否完成。
- Scope 是否保持。
- Non-goals 是否未越界。
- Dependencies 是否被尊重。
- Acceptance criteria 是否可追溯。

### Validation Verdict

检查：

- required commands 是否运行。
- failed / skipped 是否有解释。
- validation output 是否可信。
- validation 是否覆盖 acceptance criteria。

### Evidence Verdict

检查：

- evidence draft 是否完整。
- changed files summary 是否存在。
- rollback plan 是否存在。
- known limitations 是否明确。
- evidence 是否能支持审计结论。

### Boundary Verdict

检查：

- forbidden files 是否未被触碰。
- forbidden actions 是否未发生。
- unauthorized remote action 是否未发生。
- scope change 是否已确认。
- role boundary 是否保持。

## Finding Object

建议对象：

```text
AuditFinding
```

字段：

```text
findingId
category
severity
title
description
source
evidence
blocking
recommendedOwnerRole
recommendedFix
```

## Finding Category

```text
contract
scope
non-goal
validation
evidence
boundary
security
performance
architecture
test-gap
delivery-risk
goal-drift
process
```

## Finding Severity

```text
info
low
medium
high
critical
```

规则：

- `high` / `critical` 默认 blocking。
- `security` 不等于自动修复；只记录风险和修复建议。
- `goal-drift` 默认推荐 Goal Agent 复查。
- `delivery-risk` 默认不能直接进入 Delivery Entry。

## Required Fixes

建议对象：

```text
AuditRequiredFix
```

字段：

```text
fixId
findingId
ownerRole
requiredAction
target
blocking
```

`ownerRole` 可以是：

```text
build-agent
spec-agent
goal-agent
delivery-agent
human
```

Audit Agent 只能提出 required fix，不执行 fix。

## Recommended Next Action

允许输出：

```text
enter-delivery-entry
return-to-build-agent
return-to-spec-agent
return-to-goal-agent
request-human-confirmation
request-evidence-fix
request-validation-fix
blocked
```

不允许输出：

```text
mark-issue-done
mark-project-done
start-next-issue
create-pr
deploy
release
```

## Delivery Eligibility

`deliveryEligible` 是审计建议，不是交付状态。

允许：

```text
true
false
unknown
```

`deliveryEligible = true` 只表示可以进入 Delivery Entry。

它不表示：

- 交付已生成。
- 交付已验收。
- 用户已接受。
- 项目已完成。

## Goal Recheck Recommendation

Audit Result 可以建议 Goal Recheck，如果：

- 发现 goal drift。
- 实际实现偏离 PLAN.md。
- scope 被扩大或缩小。
- evidence 显示成功标准不再匹配。
- 用户新增反馈改变项目方向。

Audit Agent 不修改 GOAL.md / PLAN.md。

## Human Confirmation Rules

Audit Result 必须要求人工确认，如果：

- high / critical finding 需要接受风险。
- scope change 已发生。
- forbidden file exception 被使用。
- validation 未完整通过但想继续。
- deliveryEligible 依赖人工判断。
- goal drift 需要产品方向确认。

人工确认不等于审计通过；确认后可能仍需重新审计。

## Write Boundary

Audit Result 是 Audit Agent 的产物。

未来允许写入的逻辑区域建议为：

```text
.agentflow/audit/results/**
.agentflow/audit/findings/**
```

V1 文档只定义逻辑区域，不要求当前实现立即采用该目录结构。

禁止写：

```text
.agentflow/spec/**
.agentflow/workloop/**
.agentflow/projections/**
docs/product/**
docs/foundation/**
docs/requirements/**
```

Audit Agent 不能修改 Work Loop 事实。

Audit Agent 不能修改 SpecProject / SpecIssue。

## Relationship With Work Loop

Work Loop 提供 run facts。

Audit Result 审查这些 facts。

Audit Result 不能：

- 改 run status。
- 补 evidence draft。
- 重新运行 validation。
- 直接修复 source code。

如果需要修复，必须输出 `recommendedNextAction = return-to-build-agent` 或对应角色。

## Relationship With Project Loop

Project Loop 可以读取 Audit Result，用于判断下一步：

- 是否进入 Delivery Entry。
- 是否回到 Build Agent。
- 是否回到 Spec Agent。
- 是否回到 Goal Agent。
- 是否等待 human confirmation。

Project Loop 不能把 `AuditResult(status=passed)` 单独视为 Project complete。

## Relationship With Delivery

Delivery 必须在 Audit Result 之后。

Audit Result 只能输出：

```text
deliveryEligible = true
recommendedNextAction = enter-delivery-entry
```

Delivery Entry / Delivery Report 不在本文定义。

## Blocked Conditions

Audit Result 必须 blocked 或 incomplete，如果：

- AuditEntryProposal 缺失。
- WorkLoopRun 缺失。
- Evidence Draft 缺失。
- Validation Result 缺失且无解释。
- changedFilesSummary 缺失。
- boundary evidence 缺失。
- run 对应多个 Issue。
- audit target 不清楚。
- audit input 互相冲突。
- required command output 不可追溯。

## Output

V1 输出：

```text
AuditResult
AuditVerdict[]
AuditFinding[]
AuditRequiredFix[]
RecommendedNextAction
DeliveryEligibility
```

不输出：

```text
Delivery Report
Issue Done
Project Done
Source Code Patch
Command Result
PR
Release
Deploy
```

## Not In Scope

本文不定义：

- Delivery Entry。
- Delivery Report writer。
- Fix execution。
- Validation rerun。
- Source code repair。
- Project status mutation。
- Issue status mutation。
- Remote PR / GitHub / Linear。
- Deploy / release。
- Model call orchestration。
- Desktop write UI。

## Acceptance Criteria

- [ ] Audit Result 的位置和职责明确。
- [ ] AuditResult 字段明确。
- [ ] audit status 语义明确。
- [ ] Contract / Validation / Evidence / Boundary verdict 分离。
- [ ] Finding model 明确。
- [ ] Required Fix model 明确。
- [ ] Audit Agent 只提出修复建议，不执行修复。
- [ ] `passed` 不等于 Issue Done / Project Done。
- [ ] `deliveryEligible = true` 只表示可进入 Delivery Entry。
- [ ] Audit Result 不修改 Work Loop facts。
- [ ] Audit Result 不修改 SpecProject / SpecIssue。
- [ ] Audit Result 不生成 Delivery Report。
- [ ] Audit Result 不创建 PR / release / deploy。
