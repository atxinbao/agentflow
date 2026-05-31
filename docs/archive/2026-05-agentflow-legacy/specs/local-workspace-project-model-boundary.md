# Local Workspace / Team / Project Model v0 Boundary

创建日期：2026-05-22
执行者：Codex

## 定位

Local Workspace / Team / Project Model v0 是 AgentFlow 从“一条 issue 一条 issue 推进”升级到“本地项目组织模型”的边界定义阶段。

当前阶段只锁定最小关系模型，不创建 workspace、team、project 或 milestone 文件，不改变现有 issue 执行链。

`Local Project Model v0 只读实现` 已在该边界内完成：新增 `LocalProjectModelSnapshot` 和 `agentflow projects`，从现有 `.agentflow/` 事实源派生默认 workspace、team、project、milestone 和 issue refs，仍不创建 workspace/team/project 文件。

`Desktop Project View v0` 已把同一只读 snapshot 接到 Desktop Workbench：Project 视图只展示本地层级、GoalLoopSelection、source trace 和 recommended command 文本，不写任何 seed 文件。

`Desktop Workspace Overview v0` 已把 Workspace 入口放到总览：Workspace 下直接列出 Projects 和 Teams，Team 下只读展示 Issues 计数与关联 Projects；它仍不创建组织模型事实源文件。

`Local Project Seed v0` 已实现 `agentflow project-seed` preview / writer：默认只读展示 seed 草案，显式 `--write --yes` 才创建 workspace/team/project seed；当前仍不迁移 issue，不改变 GoalLoop。

最小关系锁定为：

```text
LocalWorkspace
  -> LocalTeams
      -> IssueContracts
  -> LocalProjects
      -> Milestones
      -> IssueContracts
  -> GoalLoop
      -> 从 active project 里选择下一条 issue
```

## Linear 关系取舍

AgentFlow 参考 Linear 的 workspace / team / project / issue 组织方式，但只保留本地最小可执行关系：

| Linear 概念 | AgentFlow 本地取舍 |
| --- | --- |
| Workspace | 保留为 `LocalWorkspace`，代表一个本地 AgentFlow 项目集合或当前本地项目根 |
| Team | 保留为 `LocalTeam`，用于 workflow、validation preset、issue ownership 分组 |
| Project | 保留为 `LocalProject`，用于承载一个有明确目标的交付包 |
| Issue | 保留为 `IssueContract`，仍是唯一执行授权 |
| Milestone | 保留为 `Milestone`，用于 project 内阶段切分 |
| Initiative / Cycle | v0 不做，避免变成重型项目管理平台 |

## 当前阶段允许

| 能力 | v0 边界 |
| --- | --- |
| 定义关系 | 锁定 workspace / team / project / milestone / issue / goal loop 的职责 |
| 定义文件路径候选 | 只定义候选路径，不创建文件 |
| 定义 GoalLoop 选择规则 | 后续从 active project backlog 推荐下一条 issue |
| 定义迁移顺序 | 先只读模型，再 issue 归属，再 project-aware goal next |
| 定义验证方式 | no-file proof、readiness anchors、goal next 仍只推荐 |

## 当前阶段不允许

| 禁止项 | 说明 |
| --- | --- |
| 默认创建 `.agentflow/workspace.json` | seed preview 默认只读，显式 `agentflow project-seed --write --yes` 才允许 |
| 默认创建 `.agentflow/teams/` | team seed 文件写入必须经过 seed writer 确认 |
| 默认创建 `.agentflow/projects/` | project seed 文件写入必须经过 seed writer 确认 |
| 迁移现有 issue | issue 归属关系需要单独实现 |
| 改写 GoalLoop 决策 | project-aware selection 需要后续实现 |
| 改变 `agentflow run` | IssueContract 仍是唯一执行输入 |
| 引入账号 / 权限 / 云同步 | 本地 v0 不做 SaaS workspace |
| 调用 Linear API | 当前不创建远程 workspace/team/project/issue |

## 对象职责

### LocalWorkspace

LocalWorkspace 是本地顶层容器。

职责：

- 记录当前本地 AgentFlow 项目的组织入口。
- 维护默认 team。
- 维护 active project 指针。
- 聚合 project、team、issue、run、evidence、review 的只读摘要。

v0 路径候选：

```text
.agentflow/workspace.json
```

默认 preview 不创建该文件；显式 seed writer 可创建 `.agentflow/workspace.json`。

### LocalTeam

LocalTeam 是本地 workflow 和 ownership 分组。

职责：

- 给 IssueContract 提供默认 workflow。
- 提供默认 validation command preset。
- 提供 team-level WIP 策略，但 v0 仍只有全局 WIP=1。
- 为后续 `core / desktop / docs / integrations` 之类分组预留空间。
- 一个 LocalWorkspace 可以包含多个 LocalTeam；每个 LocalTeam 可以关联多个 LocalProject 和多个 IssueContract。
- Desktop MVP 只读展示这层关系：左侧栏目用 `Workspace -> project / issues`、`Team -> project / issues` 表达工作区树；团队页用“团队父栏目 / 项目子栏目 / 任务子栏目”表达父子关系，不从桌面端创建或编辑 team / project / issue。

v0 路径候选：

```text
.agentflow/teams/{team-id}.json
```

默认 preview 不创建该目录或文件；显式 seed writer 可创建 `.agentflow/teams/core.json`。

### LocalProject

LocalProject 是一组 issue 的交付容器。

职责：

- 承载一个明确的目标、范围、非目标和完成标准。
- 拆出 milestones。
- 维护 issue backlog。
- 给 GoalLoop 提供“下一条 issue 从哪里选”的上下文。
- 聚合 project update，而不是替代 issue evidence。

v0 路径候选：

```text
.agentflow/projects/{project-id}.json
```

默认 preview 不创建该目录或文件；显式 seed writer 可创建 `.agentflow/projects/agentflow-local-execution.json`。

### Milestone

Milestone 是 LocalProject 内的阶段。

职责：

- 按阶段组织 issue。
- 定义阶段验收目标。
- 为 GoalLoop 提供局部优先级。

v0 不单独落盘 milestone 文件；先内嵌在 LocalProject schema 候选中。

### IssueContract

IssueContract 仍是唯一执行授权。

职责：

- 描述单条可执行任务的 scope、non-goals、validation、evidence 和 AEP 字段。
- 保留 WIP=1 的执行约束。
- 关联一个 LocalTeam。
- 可选关联一个 LocalProject 和 Milestone。

现有路径保持不变：

```text
.agentflow/issues/{issue-id}.json
.agentflow/issues/{issue-id}.md
```

### GoalLoopSelection

GoalLoopSelection 是后续 GoalLoop 的选择上下文，不是执行器。

职责：

- 若存在 active issue，仍优先返回 verify / review / update。
- 若无 active issue 且有 active project，从 active project 的 milestone/backlog 选择下一条 issue intent。
- 若无 active project，才回退到 ROADMAP 的候选施工包。
- 只输出 recommended intent / command，不执行命令。

## Schema 候选

### LocalWorkspace

```json
{
  "version": "0.0.1",
  "id": "default",
  "name": "AgentFlow",
  "defaultTeamId": "core",
  "activeProjectId": "agentflow-local-execution",
  "teamIds": ["core"],
  "projectIds": ["agentflow-local-execution"]
}
```

### LocalTeam

```json
{
  "version": "0.0.1",
  "id": "core",
  "name": "Core",
  "workflow": ["planned", "active", "completed"],
  "defaultValidationCommands": [
    "cargo fmt --check",
    "cargo test",
    "git diff --check"
  ],
  "wipLimit": 1
}
```

### LocalProject

```json
{
  "version": "0.0.1",
  "id": "agentflow-local-execution",
  "name": "AgentFlow Local Execution",
  "status": "active",
  "goal": "Build the local-first AgentFlow execution loop.",
  "teamIds": ["core"],
  "activeMilestoneId": "project-model",
  "milestones": [
    {
      "id": "project-model",
      "name": "Project Model",
      "status": "active",
      "issueIds": ["ISSUE-0019"]
    }
  ],
  "issueIds": ["ISSUE-0019"],
  "nextIssueIntent": "Local Project Model v0 只读实现"
}
```

### IssueContract 归属字段候选

```json
{
  "projectLink": {
    "teamId": "core",
    "projectId": "agentflow-local-execution",
    "milestoneId": "current-roadmap",
    "linkSource": "issue-project-link-writer-v0"
  }
}
```

具体边界见 `docs/specs/issue-project-link-boundary.md`。这些字段后续只能由明确的 writer 加入，当前不改写已有 issue schema。

## GoalLoop 后续选择规则

后续 `agentflow goal next` 的优先级应改为：

1. `goal check` 未通过：`wait-human`。
2. 存在 active issue：继续该 issue 的 verify / review / update。
3. 存在未完成 issue：继续未完成 issue，保持 WIP=1。
4. 存在 active project 且有 next issue intent：推荐该 project 下的下一条 issue。
5. 无 active project / active milestone candidate：回退到 root `ROADMAP.md` 或 `docs/planning/construction-plan.md` 的 `候选施工包`。
6. 无任何候选：返回 `wait-human`。

关键边界：

- GoalLoop 只推荐，不执行。
- Project backlog 不等于执行授权。
- 只有 `agentflow plan "<intent>"` 生成 IssueContract 后，才可执行 run。
- Project-aware GoalLoop v0 已按上述优先级实现；active issue 和 incomplete issue 仍优先于 project candidate。

## 后续迁移顺序

| 顺序 | 小切片 | 边界 |
| --- | --- | --- |
| 1 | Local Project Model v0 只读实现 | 已完成 reader 和内存 snapshot，不迁移 issue |
| 2 | Desktop Project View v0 | 已完成只读展示 workspace/team/project/milestone |
| 3 | Desktop Workspace Overview v0 | 已完成总览 workspace projects / teams 和 team issues / projects 摘要 |
| 4 | Local Project Seed v0 边界定义 | 已定义默认 seed 写入合同，不写文件 |
| 5 | Local Project Seed v0 | 已完成 preview / writer，默认 preview 不写，显式 `--write --yes` 才写 seed |
| 6 | Issue Project Link v0 边界定义 | 已定义 issue 归属字段和迁移门，不迁移现有 issue |
| 7 | Issue Project Link Writer v0 | 已完成 preview / writer，默认不迁移历史 issue，显式确认后只写指定 issue |
| 8 | Project-aware GoalLoop v0 边界定义 | 已定义 active project 选择规则、fallback 和禁止项 |
| 9 | Project-aware GoalLoop v0 | 已实现从 active project 推荐下一条 issue intent |
| 10 | Desktop GoalLoop Trace v0 | 已完成，只读展示 GoalLoop 决策来源 |
| 11 | Desktop Issue Lifecycle Trace v0 | 下一候选，只读展示 issue contract 到 evidence / review / update 链路 |

## 验证方式

当前边界阶段验证：

| 验证 | 要求 |
| --- | --- |
| Docs anchors | README、ROADMAP、MVP Spec、Local Pro、Architecture 都引用本模型 |
| Preview no write | `agentflow project-seed` 不创建 `.agentflow/workspace.json`、`.agentflow/teams/`、`.agentflow/projects/` |
| Explicit write tests | tempdir 单元测试覆盖 seed writer 创建三类 seed 文件 |
| Search trace | `agentflow search "Local Workspace"` 和 `agentflow search "Local Project"` 可追溯 |
| Seed boundary trace | `agentflow search "Local Project Seed"` 可追溯 |
| Issue link trace | `agentflow search "Issue Project Link"` 可追溯 |
| No issue migration | `.agentflow/issues/*.json` 没有真实 `projectLink` / `teamId` / `projectId` / `milestoneId` / `linkSource` 属性 |
| GoalLoop safety | `goal next` 和 Desktop Project recommended command 仍只推荐，不执行 |
| Existing validation | cargo / desktop build / readiness 仍通过 |

## Evidence 要求

本阶段 evidence 至少包含：

- 本文档路径。
- no workspace/team/project fact files proof。
- `agentflow project-seed` preview 输出摘要。
- `goal next` 输出摘要。
- `agentflow search "Local Workspace"` / `"Local Project"` 输出摘要。
- `agentflow search "Local Project Seed"` 输出摘要。
- readiness 通过证明。

## 验收

1. 最小关系 `LocalWorkspace -> LocalTeams/LocalProjects -> Milestones -> IssueContracts -> GoalLoopSelection` 被锁定。
2. `agentflow project-seed` 默认不创建 workspace/team/project 文件；显式写入行为有测试覆盖。
3. IssueContract 仍是唯一执行授权。
4. GoalLoop 未来从 active project 选择下一条 issue 的规则明确。
5. `agentflow goal next` 在 Project read model 完成后仍只推荐后续小切片，不执行。
