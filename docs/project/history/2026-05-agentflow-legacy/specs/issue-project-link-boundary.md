# Issue Project Link v0 Boundary

创建日期：2026-05-25
执行者：Codex

## 定位

Issue Project Link v0 Boundary 定义 `IssueContract` 如何后置关联 `LocalTeam`、`LocalProject` 和 `Milestone`。

边界阶段已完成；`Issue Project Link Writer v0 实现` 已在该边界下新增本地 CLI preview / writer。默认 preview 不迁移现有 issue，不写 project link 字段；只有显式 `agentflow issue-link ISSUE-XXXX --write --yes` 才写指定 issue 的 JSON / Markdown。

该边界的作用是把现在的关系链补齐到：

```text
LocalWorkspace
  -> LocalTeams
  -> LocalProjects
      -> Milestones
      -> IssueContracts
  -> GoalLoopSelection
```

`IssueContract` 仍是唯一执行授权。Project link 只是归属和选择上下文，不能直接授权 run / verify / review。

## 前置状态

已完成前置能力：

- `LocalProjectModelSnapshot` 可以从现有 `.agentflow/` 事实源只读派生 workspace / team / project / milestone。
- Desktop Project View 和 Workspace Overview 可以只读展示该层级。
- `agentflow project-seed` 可以预览 seed；显式 `--write --yes` 才能创建 workspace/team/project seed。
- `agentflow issue-link ISSUE-XXXX` 可以预览默认 issue project link；显式 `--write --yes` 才能写指定 issue。
- 当前 live repo 仍没有 `.agentflow/workspace.json`、`.agentflow/teams/`、`.agentflow/projects/`。

## 候选字段

后续 writer 的候选存储形态是 `IssueContract` 内的可选 `projectLink` 对象。

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

字段含义：

| 字段 | 含义 | v0 规则 |
| --- | --- | --- |
| `teamId` | issue 所属 LocalTeam | 默认候选为 `core` |
| `projectId` | issue 所属 LocalProject | 默认候选为 `agentflow-local-execution` |
| `milestoneId` | issue 所属 Milestone | 默认候选为 `current-roadmap` |
| `linkSource` | link 生成来源 | 必须记录 writer / manual confirmation / imported source |

字段规则：

- 所有 id 只能使用小写字母、数字和 `-`。
- `teamId` 必须存在于 LocalProjectModelSnapshot 或 seed facts。
- `projectId` 必须存在于 LocalProjectModelSnapshot 或 seed facts。
- `milestoneId` 必须存在于目标 project 的 milestones。
- `linkSource` 必须是确定性字符串，不能写入模型输出摘要或远程平台临时状态。

## Link Source 候选

| Source | 用途 |
| --- | --- |
| `issue-project-link-writer-v0` | 后续本地 writer 写入 |
| `human-confirmed` | 用户明确选择 team/project/milestone 后写入 |
| `local-project-seed` | 来自已落盘 workspace/team/project seed |
| `derived-default-preview` | 只读 preview 使用，不允许直接作为写入来源 |
| `imported-local` | 后续从本地文件导入，必须有单独边界 |

## 当前阶段允许

| 能力 | 边界 |
| --- | --- |
| 定义 schema | 已定义 `projectLink` 候选对象和四个字段 |
| 定义写入路径 | writer 只能写 `.agentflow/issues/{issue-id}.json` 和 `.agentflow/issues/{issue-id}.md` |
| 定义确认门 | 已定义何时需要用户确认 |
| 定义迁移顺序 | 先 writer，再 Project-aware GoalLoop |
| 定义验证 | no-migration proof、search trace、readiness anchors |
| 实现 preview | `agentflow issue-link ISSUE-XXXX` 默认只读 preview |
| 实现 writer | `agentflow issue-link ISSUE-XXXX --write --yes` 显式确认后只写指定 issue |
| 更新下一候选 | writer 完成后推荐 `Project-aware GoalLoop v0 边界定义` |

## 当前阶段不允许

| 禁止项 | 说明 |
| --- | --- |
| 迁移现有 issue | 不批量给 `.agentflow/issues/*.json` 增加真实 link 字段 |
| 默认写 `projectLink` | preview 只展示候选，默认不写入事实源 |
| 覆盖 `projectLink` | 已有 link 时 writer 拒绝覆盖 |
| 改写 GoalLoop | Project-aware GoalLoop 后置 |
| 从 Desktop 写 link | Desktop 仍只读 |
| 绕过 IssueContract | Project link 不是执行授权 |
| 远程同步 | 不创建 Linear/Jira/PingCode/GitHub issue |

## 用户确认门

writer 在以下动作前必须要求确认：

| Gate | 触发条件 |
| --- | --- |
| `write-project-link` | 给 issue 写入 `projectLink` |
| `overwrite-project-link` | 覆盖已有 `projectLink` |
| `link-existing-issue` | 给已存在 issue 写归属关系 |
| `use-derived-default` | 在 seed facts 不存在时使用派生默认 project/team |
| `change-team` | 从一个 team 改到另一个 team |
| `change-project` | 从一个 project 改到另一个 project |
| `change-milestone` | 从一个 milestone 改到另一个 milestone |
| `desktop-write-link` | 后续 Desktop 若提供写入口 |

默认 writer 必须先 preview，再写入。

## 后续实现顺序

1. `Issue Project Link Writer v0 实现`
   - 状态：已完成。
   - 增加 `IssueProjectLink` / `IssueProjectLinkPreview`。
   - `agentflow issue-link ISSUE-XXXX` 默认 preview。
   - `agentflow issue-link ISSUE-XXXX --write --yes` 只写指定 issue JSON / Markdown。
   - 默认不迁移历史 issue，不改写 GoalLoop。
2. `Project-aware GoalLoop v0 边界定义`
   - 状态：已完成。
   - 定义 GoalLoop 如何读取 active project backlog。
   - 明确 WIP=1 和 incomplete issue 优先级不变。
3. `Project-aware GoalLoop v0 实现`
   - 在无 active / incomplete issue 时，从 active project 推荐下一条 issue intent。
   - 仍只推荐 `agentflow plan "<intent>"`，不执行。

## 验证矩阵

| 验证 | 要求 |
| --- | --- |
| Boundary docs | README、ROADMAP、MVP Spec、Local Pro、Local Workspace Boundary、Local Project Seed Boundary 引用本文件 |
| Preview no write | `agentflow issue-link ISSUE-0025` 输出 preview 且不写 live issue |
| Explicit write tests | tempdir 单元测试覆盖 `--write --yes` 只写指定 issue |
| No issue migration | live `.agentflow/issues/*.json` 不出现真实 `projectLink`、`teamId`、`projectId`、`milestoneId`、`linkSource` 属性 |
| Existing GoalLoop | active issue / incomplete issue / roadmap candidate 语义不改 |
| Existing Seed boundary | `agentflow project-seed` 仍默认 preview，不创建 seed 文件 |
| Search trace | `agentflow search "Issue Project Link"` 返回可追溯结果 |
| Readiness | `bash checks/agentflow-readiness.sh` 通过 |

## Evidence 要求

writer implementation evidence 至少包含：

- 本文件路径。
- `agentflow issue-link ISSUE-0025` preview 输出摘要。
- explicit writer tempdir test 摘要。
- no issue migration proof。
- `agentflow search "Issue Project Link"` 输出摘要。
- `agentflow goal next` 输出摘要。
- no seed file proof。
- readiness 通过证明。

## 验收

1. Issue Project Link 的候选字段、写入路径、确认门和 evidence 要求明确。
2. `agentflow issue-link ISSUE-XXXX` 默认只 preview。
3. `agentflow issue-link ISSUE-XXXX --write --yes` 只写指定 issue。
4. 本阶段不批量迁移任何现有 issue。
5. live repo 验证不写 `projectLink`、`teamId`、`projectId`、`milestoneId` 或 `linkSource` 真实属性。
6. 本阶段不改写 GoalLoop 决策。
7. `agentflow goal next` 在 Project-aware GoalLoop 边界完成后推荐 `Project-aware GoalLoop v0 实现`。
