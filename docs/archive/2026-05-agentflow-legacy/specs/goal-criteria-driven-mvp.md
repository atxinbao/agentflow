# Goal + Criteria Driven MVP

创建日期：2026-05-26
执行者：Codex
状态：defined / project-control-contract

## 目标

AgentFlow 后续项目完成方式改为 Goal + Criteria 驱动。

当前 MVP 不再以 Agent 自动执行闭环作为主目标，而是先完成本地产品建模和用户 / Agent 协作创建能力。

## Goal

```text
把 AgentFlow 做成一个本地优先、免费、闭源的 AI coding agent 受控交付工具。

当前 MVP 只聚焦两项核心能力：

1. 用户可以在本地创建和管理 Team / Project / Milestone / Issue。
2. 用户和 Agent 可以共同把一个产品功能目标拆成 Project -> Milestones -> Issues，并保存为本地事实源。
```

## Criteria

1. 本地存在清晰层级：Workspace -> Teams -> Projects -> Milestones -> Issues。
2. Workspace 只是本地工作区容器，不维护状态。
3. Team 只是本地组织容器，不维护状态。
4. Project 必须维护状态：draft / active / paused / completed / canceled。
5. Issue 必须维护状态：backlog / todo / in_progress / in_review / done / canceled。
6. Milestone 不维护独立状态，只作为 Project 下的阶段分组。
7. Milestone 完成度从其包含的 Issues 状态派生：done issues / total non-canceled issues。
8. Project 完成度从其包含的 Milestones / Issues 派生，但 Project 是否 completed 必须由用户或明确确认动作设置。
9. 用户可以创建 Team。
10. 用户可以创建 Project。
11. 用户可以在 Project 下创建 Milestone。
12. 用户可以在 Milestone 下创建 Issue。
13. Agent 可以基于用户输入的产品功能目标，生成 Project / Milestones / Issues 草案。
14. 所有创建动作默认 preview，不写 `.agentflow/`。
15. 只有用户显式确认后，才允许写入 `.agentflow/`。
16. Desktop 能展示 Workspace -> Teams -> Projects -> Milestones -> Issues。
17. Desktop 能展示 Project status 和 Issue status。
18. Desktop 当前不执行 run / verify / review。
19. 当前 MVP 不把 Agent 自动执行流程作为主产品目标。
20. eligibility / lease / run / verify / review / evidence / milestone summary 只作为后续执行层能力保留，不进入当前 MVP 主目标。
21. 不做 SaaS、账号、支付、云同步。
22. 不做完整 Linear clone。
23. 不接入远程 GitHub / Linear 写入。
24. 所有本地数据必须保存在 `.agentflow/`，保持可读、可迁移。

## 状态模型

Workspace 和 Team 是容器，不维护状态。

Project 维护状态：

```text
draft
active
paused
completed
canceled
```

Issue 维护状态：

```text
backlog
todo
in_progress
in_review
done
canceled
```

Milestone 不维护独立状态。Milestone 的完成度从其包含的 issue 状态派生。

## 旧状态兼容

当前 `.agentflow/` 历史事实源仍存在旧状态。MVP 状态模型迁移时必须提供兼容映射：

| Legacy | Project status | Issue status |
| --- | --- | --- |
| `planned` | draft | todo |
| `ready` | draft | todo |
| `active` | active | in_progress |
| `completed` | completed | done |
| `done` | completed | done |
| `cancelled` | canceled | canceled |
| `canceled` | canceled | canceled |
| `blocked` | paused | backlog |

兼容映射只能用于读取和展示，不能作为长期新写入格式。

## Product Boundary

当前主产品闭环：

```text
Human / Agent input
-> Team
-> Project
-> Milestones
-> Issues
-> Local fact source
-> Desktop read-only hierarchy
```

`Project / Milestone / Issue / View Model v1` 将该闭环固定为：

```text
Workspace / Team
-> Project
   -> Milestone
      -> Issue
-> View
```

其中：

- Project 负责方向、边界和整体成功标准。
- Milestone 负责阶段目标和阶段出口。
- Issue 是 Agent 唯一执行合同。
- View 是 saved filter，只展示和过滤，不承载业务状态。

当前 MVP 的产品原则：

```text
Project 不执行
Milestone 不执行
Issue 执行
View 只展示
Queue Preflight 决定谁能执行
Evidence 决定是否 Done
```

后续执行层能力保留为底层能力：

```text
eligibility
lease
run
verify
review
evidence
milestone summary
```

但它们不再是当前 MVP 的完成标准。

## 当前实现状态

```text
Team / Project / Milestone / Issue Writers v0
```

已完成：

- `agentflow team create "<team name>"` 默认 preview，`--write --yes` 写 `.agentflow/teams/{team-id}.json` 并更新 workspace teamIds。
- `agentflow project create "<project title>"` 默认 preview，`--write --yes` 写 `.agentflow/projects/{project-id}.json` 并更新 workspace / team 引用。
- `agentflow milestone create "<milestone title>"` 默认 preview，`--write --yes` 追加到目标 Project `milestones[]`。
- `agentflow issue create "<issue title>"` 默认 preview，`--write --yes` 写 `ISSUE-XXXX.{json,md}` 并同步 project / milestone / team / index 引用。
- Project 默认 `draft`，Issue 默认 `todo`，Milestone 不写产品状态。
- Desktop 继续只展示层级，不执行 run / verify / review，也不提供写入 UI。

前置已完成：

- `ProjectStatus` / `IssueStatus` canonical 定义已落到 core。
- `LocalProjectModelSnapshot` 输出 `canonicalStatus` 和 milestone derived progress。
- Product Feature Creation 和 `agentflow plan` 新建 issue 默认写 `todo`。
- review 完成 issue 后写 `done`。
- CLI / Desktop 已展示 canonical Project / Issue status。

## 验证矩阵

```bash
cargo fmt --check
cargo test
npm --prefix apps/desktop run build
cargo run -p agentflow-cli -- goal check
cargo run -p agentflow-cli -- goal next
cargo run -p agentflow-cli -- projects
cargo run -p agentflow-cli -- feature status
cargo run -p agentflow-cli -- feature next
bash checks/agentflow-readiness.sh
git diff --check
```
