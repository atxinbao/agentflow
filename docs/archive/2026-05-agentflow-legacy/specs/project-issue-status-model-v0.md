# Project / Issue Status Model v0

创建日期：2026-05-26
执行者：Codex
状态：implemented / canonical-status-model

## Goal

本阶段把 Goal + Criteria Driven MVP 中的状态模型落到代码、事实源读取、CLI 展示和 Desktop 展示。

当前 MVP 的状态边界：

```text
Workspace / Team = 本地容器，不维护状态
Project = 产品目标状态
Milestone = Project 下的阶段分组，不维护独立产品状态
Issue = 最小任务状态
```

## Canonical Status

Project canonical status：

```text
draft
active
paused
completed
canceled
```

Issue canonical status：

```text
backlog
todo
in_progress
in_review
done
canceled
```

## v1 Product Model Alignment

`Project / Milestone / Issue / View Model v1` 已把后续产品目标状态扩展为：

```text
Project:
Draft -> Planned -> Active -> Closing -> Completed
                         -> Blocked

Milestone:
Draft -> Ready -> Active -> Review -> Done
                         -> Blocked

Issue:
Backlog -> Todo -> In Progress -> In Review -> Done
                          -> Blocked
                          -> Repair
```

当前 v0 代码不立即迁移或破坏现有事实源。兼容原则：

- v0 canonical status 继续作为当前写入和读取口径。
- v1 中 `Planned / Closing / Blocked / Repair` 先作为产品建模目标和后续迁移方向。
- `Backlog -> Todo` 必须由 Queue Preflight 决定，不能靠人工随便标记。
- Milestone 当前仍不维护独立产品状态；如后续引入 v1 gate state，只能表达阶段门状态，不能让 Milestone 直接执行。

## Legacy Compatibility

旧 `.agentflow/` 事实源允许继续读取。兼容映射只用于读取、派生、展示和过滤，不作为新写入的长期格式。

| Legacy | Project canonical | Issue canonical |
| --- | --- | --- |
| `planned` | `draft` | `todo` |
| `ready` | `draft` | `todo` |
| `active` | `active` | `in_progress` |
| `completed` | `completed` | `done` |
| `done` | `completed` | `done` |
| `cancelled` | `canceled` | `canceled` |
| `canceled` | `canceled` | `canceled` |
| `blocked` | `paused` | `backlog` |

实现侧保留若干历史 workflow 状态读取，例如 `audit`、`docs-refresh`、`final-review`、`pr`、`checks-passing`、`merged`、`evidence-captured`。这些状态会映射到当前 MVP 的 canonical status，而不是成为新的产品状态。

## Milestone Progress

Milestone 不作为产品状态机。

Milestone 保留字段：

```text
id
name / title
description
sortOrder
target
issueIds
nextIssueIntent
```

历史 `milestone.status` 继续兼容读取，但 CLI 和 Desktop 不把它作为 MVP 状态展示。

Milestone 完成度从 issue 状态派生：

```text
doneIssueCount / nonCanceledIssueCount
percent
canceledIssueCount
totalIssueCount
```

`canceled` issue 不计入完成度分母。

## Write Rules

新写入规则：

```text
Product Feature Project 默认 status = active
Product Feature Issue 默认 status = todo
agentflow plan 新建 Issue 默认 status = todo
review 完成 Issue 后写 status = done
```

Product Feature Project 仍默认 `active`，原因是 `agentflow feature create --write --yes` 会立即把该 Project 写入 `.agentflow/workspace.json.activeProjectId`，使用户创建后可以马上在 CLI / Desktop 中看到当前 active project。

## Read Model Output

`LocalProjectModelSnapshot` 输出：

```text
LocalProject.status              = 原始事实源状态
LocalProject.canonicalStatus     = canonical Project status
LocalProjectIssueRef.status      = 原始事实源状态
LocalProjectIssueRef.canonicalStatus = canonical Issue status
LocalMilestone.progress          = 派生完成度
```

`ProductFeatureExecutionSnapshot` 输出：

```text
projectStatus
projectCanonicalStatus
currentIssue.status
currentIssue.canonicalStatus
milestones[].progress
```

## CLI / Desktop

CLI：

```text
agentflow projects
agentflow feature status
```

展示 canonical Project / Issue status，并展示 milestone 派生进度。

Desktop：

```text
Project 视图展示 Project canonical status
Issue / Task 视图展示 Issue canonical status
Milestone 展示 derived progress，不展示 milestone status
```

## Non-Goals

- 不自动执行 run / verify / review。
- 不调用模型。
- 不创建远程 PR / GitHub issue / Linear issue。
- 不接入 SaaS、账号、支付、云同步。
- 不破坏历史 `.agentflow/` 读取。
- 不把 Milestone 重新做成产品状态机。

## Verification

```bash
cargo fmt --check
cargo test
npm --prefix apps/desktop run build
cargo run -p agentflow-cli -- goal check
cargo run -p agentflow-cli -- projects
cargo run -p agentflow-cli -- feature status
cargo run -p agentflow-cli -- feature create "状态模型验证功能"
bash checks/agentflow-readiness.sh
git diff --check
```
