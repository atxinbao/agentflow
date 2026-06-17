# 006 - Project Brain Confirmation Gate V1

创建日期：2026-06-17
执行者：Codex

## Source Foundation

本文从以下文档派生：

- [002-project-brain-document-store-v1.md](002-project-brain-document-store-v1.md)
- [003-requirement-to-goal-draft-v1.md](003-requirement-to-goal-draft-v1.md)
- [004-plan-draft-preview-v1.md](004-plan-draft-preview-v1.md)

## Purpose

本文定义 Project Brain 的确认门。

确认门是从草稿进入事实源的唯一通道。没有确认，Goal Draft / Plan Draft 只能是 Preview，不能写入项目文档，也不能进入 Project Loop。

## Confirmation Targets

可被确认的对象：

```text
Goal Draft
Plan Draft
Decision Entry
Scope Change
Delivery Acceptance
Next Stage Proposal
```

V1 优先覆盖：

```text
Goal Draft
Plan Draft
Decision Entry
```

## Confirmation States

```text
draft
pending-human-confirmation
confirmed
rejected
needs-revision
expired
```

## Gate Flow

```text
Draft Preview
-> Human Review
-> Confirm / Reject / Revise
-> Write project brain documents
-> Record DECISIONS.md entry
```

## Confirm Goal Draft

确认后允许：

- 写入 GOAL.md。
- 写入 DECISIONS.md 的 Goal confirmation 记录。
- 进入 Plan Draft Preview。

确认后仍不允许：

- 不生成 Issue。
- 不进入 Work Loop。
- 不执行任务。

## Confirm Plan Draft

确认后允许：

- 写入 PLAN.md。
- 写入 DECISIONS.md 的 Plan confirmation 记录。
- 生成 SpecProject / SpecIssue materialization proposal。

确认后仍不允许：

- 不自动执行 Issue。
- 不绕过 Project Loop preflight。
- 不跳过 Audit / Delivery 规则。

## Confirm Decision Entry

确认后允许：

- 追加 DECISIONS.md。
- 更新 Project Brain Snapshot 的 pending confirmation 状态。

## Required Confirmation Record

每次确认必须记录：

```text
timestamp
actor
targetType
targetId
summary
decision
impact
nextAction
```

## Rejection

Reject 后：

- 不写项目文档。
- 不生成 Issue。
- 保留草稿为 rejected 状态。
- 输出下一步建议。

## Revision

Needs revision 后：

- 不写项目文档。
- 回到 Draft Preview。
- 标记需要修改的问题。

## Expiration

如果项目上下文变化，旧 Draft 可以过期。

过期条件：

- Goal 已变更。
- Plan 已变更。
- Project path 已变更。
- 关键约束已变更。
- 用户明确取消。

## Acceptance Criteria

- [ ] Confirmation target 明确。
- [ ] Confirmation state 明确。
- [ ] Goal Draft confirmation 边界明确。
- [ ] Plan Draft confirmation 边界明确。
- [ ] Decision Entry confirmation 边界明确。
- [ ] Confirm / Reject / Revise 行为明确。
- [ ] 每次确认必须记录 DECISIONS.md。
- [ ] 不确认不写入。
- [ ] 不确认不生成 Issue。
- [ ] 不确认不进入 Work Loop。
