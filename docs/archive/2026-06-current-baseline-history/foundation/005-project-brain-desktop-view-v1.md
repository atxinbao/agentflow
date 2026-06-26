# 005 - Project Brain Desktop View V1

创建日期：2026-06-17
执行者：Codex

## Source Foundation

本文从以下文档派生：

- [001-goal-agent-project-brain-v1.md](001-goal-agent-project-brain-v1.md)
- [002-project-brain-document-store-v1.md](002-project-brain-document-store-v1.md)
- [003-requirement-to-goal-draft-v1.md](003-requirement-to-goal-draft-v1.md)
- [004-plan-draft-preview-v1.md](004-plan-draft-preview-v1.md)

## Purpose

本文定义 Desktop 如何展示 Project Brain。

Desktop View 是只读项目大脑视图。它帮助用户理解当前 Project 的 Goal / Plan / Decisions 状态，但不执行、不写入、不生成 Issue。

## Page Role

Project Brain Desktop View 用于回答：

- 当前项目目标是否存在？
- 当前项目计划是否存在？
- 关键决策是否有记录？
- 项目大脑是否 ready？
- 下一步建议是什么？
- 是否需要用户确认？

## View Sections

建议包含：

```text
Project Brain Summary
Goal Status
Plan Status
Decisions Status
Open Questions
Next Recommended Action
Document Links
Readonly Boundary
```

## Project Brain Summary

展示：

- Project title
- Project path
- Brain status
- Last updated
- Readonly indicator

## Goal Status

展示：

- GOAL.md exists / missing
- goalStatus
- outcome summary
- success criteria count
- open questions count

## Plan Status

展示：

- PLAN.md exists / missing
- planStatus
- current stage
- next recommended step
- risk / blocker count

## Decisions Status

展示：

- DECISIONS.md exists / missing
- latest decision
- pending confirmation count

## Next Recommended Action

允许展示的 action：

```text
补充项目目标
确认项目目标
生成计划草稿
确认计划草稿
进入项目循环
回看项目方向
暂停等待用户确认
```

不允许展示成直接执行按钮。

## Interaction Boundary

V1 Desktop 只能：

- 查看状态。
- 查看文档内容。
- 复制建议。
- 打开本地文档。

V1 Desktop 不能：

- 不写 GOAL.md。
- 不写 PLAN.md。
- 不写 DECISIONS.md。
- 不生成 Issue。
- 不执行 Work Loop。
- 不调用模型。
- 不创建 PR。
- 不写 `.agentflow/` runtime data。

## Empty State

如果 Project Brain 未初始化：

```text
项目大脑未初始化。
需要先生成 Goal Draft Preview，并等待用户确认。
```

如果 GOAL.md 存在但 PLAN.md 缺失：

```text
项目目标已存在。
下一步建议生成 Plan Draft Preview。
```

## Acceptance Criteria

- [ ] Desktop View 只读边界明确。
- [ ] Project Brain Summary 明确。
- [ ] Goal / Plan / Decisions 状态展示明确。
- [ ] Next Recommended Action 不等同于执行按钮。
- [ ] 不写文档。
- [ ] 不写 `.agentflow/` runtime data。
- [ ] 不生成 Issue。
- [ ] 不调用模型。
