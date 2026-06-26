# Local Project Seed v0 Boundary

创建日期：2026-05-23
执行者：Codex

## 定位

Local Project Seed v0 Boundary 定义从只读 `LocalProjectModelSnapshot` 进入本地组织模型事实源的写入合同。

边界阶段先定义写入合同；`Local Project Seed v0 实现` 已在该合同下新增本地 CLI preview / writer。默认 preview 不创建文件；只有显式 `agentflow project-seed --write --yes` 才写入 seed 文件。

## 前置状态

已完成前置能力：

- `Local Workspace / Team / Project Model v0` 锁定最小关系。
- `Local Project Model v0` 从现有 `.agentflow/` 事实源只读派生 `LocalProjectModelSnapshot`。
- `Desktop Project View v0` 和 `Desktop Workspace Overview v0` 只读展示该 snapshot。

## 目标关系

```text
LocalWorkspace
  -> LocalTeams
      -> IssueContracts
      -> LocalProjects
  -> LocalProjects
      -> Milestones
      -> IssueContracts
  -> GoalLoopSelection
```

`LocalProjectSeed` 的职责是把当前只读派生关系固化成默认本地事实源草案。它不是执行授权。

## 当前阶段允许

| 能力 | 边界 |
| --- | --- |
| 定义写入路径 | 只定义 `.agentflow/workspace.json`、`.agentflow/teams/*.json`、`.agentflow/projects/*.json` |
| 定义 seed source | 从 `LocalProjectModelSnapshot`、goal、settings、scope-state、roadmap、issues 派生 |
| 定义 schema | 明确 workspace、team、project seed 文件的最小字段 |
| 定义确认门 | 定义 future writer 何时必须让用户确认 |
| 定义验证 | no-file proof、schema examples、search/readiness anchors |
| 定义 evidence | 后续 writer 必须留下哪些证据 |
| 实现 preview | `agentflow project-seed` 只读展示将要创建的文件和确认门 |
| 实现 writer | 仅 `agentflow project-seed --write --yes` 创建默认 seed，且拒绝覆盖 |

## 当前阶段不允许

| 禁止项 | 说明 |
| --- | --- |
| 创建 `.agentflow/workspace.json` | 本阶段必须保持不存在 |
| 创建 `.agentflow/teams/` | 本阶段必须保持不存在 |
| 创建 `.agentflow/projects/` | 本阶段必须保持不存在 |
| 默认执行 `agentflow project-seed` 写文件 | preview 必须只读 |
| 改写 issue contract | issue project link 后置，边界见 `docs/specs/issue-project-link-boundary.md` |
| 改写 `agentflow goal next` | Project-aware GoalLoop 后置 |
| 新增 Desktop 写入 UI | Desktop Interaction Gate 后置 |
| 调用模型或远程服务 | seed 必须本地确定性派生 |

## Seed Source

future writer 只能从这些本地输入派生默认 seed：

| Source | 用途 |
| --- | --- |
| `LocalProjectModelSnapshot` | 默认 workspace、team、project、milestone、issue refs |
| `.agentflow/goal.json` | project goal |
| `.agentflow/settings.json` | project name、validation commands |
| `.agentflow/scope-state.json` | WIP limit |
| `.agentflow/roadmap.md` | next issue intent |
| `.agentflow/issues/*.json` | issue ids 和 status |

禁止从模型、远程 API、Linear workspace、GitHub PR 或 Desktop UI 临时状态派生 seed。

## 写入路径合同

future writer 的唯一默认写入路径：

```text
.agentflow/workspace.json
.agentflow/teams/{team-id}.json
.agentflow/projects/{project-id}.json
```

默认 seed ids：

| 对象 | 默认 id | 路径 |
| --- | --- | --- |
| LocalWorkspace | `default` | `.agentflow/workspace.json` |
| LocalTeam | `core` | `.agentflow/teams/core.json` |
| LocalProject | `agentflow-local-execution` | `.agentflow/projects/agentflow-local-execution.json` |

路径规则：

- id 只能使用小写字母、数字和 `-`。
- writer 必须拒绝 `..`、绝对路径、隐藏文件名和空 id。
- writer 不得写入 `.agentflow/issues/`、`.agentflow/runs/`、`.agentflow/evidence/`、`.agentflow/reviews/`。
- writer 不得写入 `.agentflow/search/` 或 `.agentflow/queries/`。

## Schema 候选

### `.agentflow/workspace.json`

```json
{
  "version": "0.0.1",
  "id": "default",
  "name": "AgentFlow",
  "defaultTeamId": "core",
  "activeProjectId": "agentflow-local-execution",
  "teamIds": ["core"],
  "projectIds": ["agentflow-local-execution"],
  "source": {
    "kind": "local-project-model-snapshot",
    "generatedFrom": "read_local_project_model_snapshot"
  }
}
```

### `.agentflow/teams/core.json`

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
  "wipLimit": 1,
  "issueIds": []
}
```

### `.agentflow/projects/agentflow-local-execution.json`

```json
{
  "version": "0.0.1",
  "id": "agentflow-local-execution",
  "name": "AgentFlow",
  "status": "active",
  "goal": "Build the local-first AgentFlow execution loop.",
  "teamIds": ["core"],
  "activeMilestoneId": "current-roadmap",
  "milestones": [
    {
      "id": "current-roadmap",
      "name": "Current Roadmap",
      "status": "planned",
      "issueIds": [],
      "completedIssueIds": [],
      "nextIssueIntent": "Issue Project Link v0 边界定义"
    }
  ],
  "issueIds": [],
  "nextIssueIntent": "Issue Project Link v0 边界定义"
}
```

## 用户确认门

future writer 必须在以下动作前要求用户确认：

| Gate | 触发条件 |
| --- | --- |
| `create-workspace-file` | 创建 `.agentflow/workspace.json` |
| `create-team-directory` | 创建 `.agentflow/teams/` |
| `create-project-directory` | 创建 `.agentflow/projects/` |
| `create-team-file` | 创建 `.agentflow/teams/{team-id}.json` |
| `create-project-file` | 创建 `.agentflow/projects/{project-id}.json` |
| `overwrite-existing-seed` | 覆盖任何已有 workspace/team/project seed |
| `change-default-team` | 修改 `defaultTeamId` |
| `change-active-project` | 修改 `activeProjectId` |
| `link-existing-issues` | 把既有 issue ids 写入 team/project/milestone |

默认实现必须先支持 dry-run / preview，再进入写入。

## 覆盖和回滚

- 默认不得覆盖已有 seed 文件。
- overwrite 必须显示旧路径、新内容摘要和 source trace。
- writer 出错时不得留下半写入目录树。
- 后续实现若需要回滚，只允许删除本次 writer 创建的文件，不得删除用户已有文件。

## 实现边界

`Local Project Seed v0 实现` 已完成：

- 新增 `LocalProjectSeedPreview`、`LocalProjectSeedFile`、`LocalProjectSeedWriteSummary`。
- 新增 `agentflow project-seed` CLI，默认只输出 preview。
- 从 `LocalProjectModelSnapshot` 生成 deterministic seed draft。
- 在 `--write --yes` 后创建 workspace/team/project seed 文件。
- 用 tempdir 单元测试覆盖显式写入行为，live repo 验证保持 no seed files。

仍不允许：

- 迁移既有 issue schema。
- 实现 Project-aware GoalLoop。
- 新增 Desktop 写入 UI。
- 调用模型或远程服务。

Issue Project Link 后续顺序：

1. 已定义 `IssueContract.projectLink` 边界。
2. 已实现 Issue Project Link Writer。
3. 下一步定义 Project-aware GoalLoop 边界。

## 验证矩阵

| 验证 | 要求 |
| --- | --- |
| Boundary docs | README、ROADMAP、MVP Spec、Local Pro、Local Workspace Boundary 引用本文件 |
| Preview no write | `agentflow project-seed` 不创建 `.agentflow/workspace.json`、`.agentflow/teams/`、`.agentflow/projects/` |
| Explicit write tests | tempdir 单元测试覆盖 `--write --yes` 对三类 seed 文件的创建 |
| Search trace | `agentflow search "Local Project Seed"` 返回本地可追溯结果 |
| Existing readers | `agentflow projects` 仍从只读 snapshot 派生 |
| GoalLoop next | Issue Project Link Writer 完成后推荐 `Project-aware GoalLoop v0 边界定义` |
| Readiness | `bash checks/agentflow-readiness.sh` 通过 |

## Evidence 要求

本阶段 evidence 至少包含：

- 本文件路径。
- no live seed file proof。
- `agentflow project-seed` preview 输出摘要。
- tempdir explicit write test proof。
- `agentflow goal next` 输出摘要。
- `agentflow projects` 输出摘要。
- `agentflow search "Local Project Seed"` 输出摘要。
- readiness 通过证明。

## 验收

1. Local Project Seed 的写入路径、schema、source、确认门和 evidence 要求明确。
2. `agentflow project-seed` 默认只读 preview，不创建 workspace/team/project seed 文件。
3. `agentflow project-seed --write --yes` 的显式写入行为由 tempdir 单元测试覆盖。
4. 当前阶段没有迁移 issue 或实现 Project-aware GoalLoop。
5. `agentflow goal next` 在本 issue 完成后推荐 `Issue Project Link v0 边界定义`。
