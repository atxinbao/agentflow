# Project-aware GoalLoop v0 Boundary

创建日期：2026-05-25
执行者：Codex

## 定位

Project-aware GoalLoop v0 Boundary 定义 `agentflow goal next` 如何在本地 workspace / project / milestone 语义下推荐下一条 issue intent。

边界阶段已完成；`Project-aware GoalLoop v0 实现` 已在该边界下更新 `goal_loop_decision`。实现只在 goal ready、无 active issue、无 incomplete issue 时读取 active project / active milestone candidate；不自动执行 plan / run / verify / review，不迁移历史 issue，不批量写 `projectLink`。

Project-aware GoalLoop 只解决一个问题：当项目已经没有 active issue 和未完成 issue 时，下一条候选 intent 应优先来自 active project / active milestone，而不是直接回退到 root roadmap。

## 最小关系

v0 锁定以下最小关系：

```text
LocalWorkspace.activeProjectId
  -> LocalProject.activeMilestoneId
      -> Milestone.issueIds
      -> Milestone.nextIssueIntent
          -> IssueContract.projectLink
          -> GoalLoopSelection
```

字段职责：

| 字段 | 职责 | v0 边界 |
| --- | --- | --- |
| `LocalWorkspace.activeProjectId` | 指向当前工作空间默认推进的项目 | 只能影响推荐来源，不能授权执行 |
| `LocalProject.activeMilestoneId` | 指向当前项目默认推进的 milestone | 只能影响候选 intent 优先级 |
| `Milestone.issueIds` | 记录 milestone 下的 issue 归属 | 不要求本阶段补齐历史 issue |
| `Milestone.nextIssueIntent` | 当 milestone 没有可继续 issue 时的下一条候选 intent | 只用于生成 recommended command |
| `IssueContract.projectLink` | issue 归属 team / project / milestone 的显式 link | 缺失时不能阻塞现有 GoalLoop |
| `GoalLoopSelection` | GoalLoop 对 active project 和 next action 的只读选择结果 | 只输出推荐，不执行命令 |

## 决策优先级

后续 Project-aware GoalLoop 的决策顺序必须保持：

1. `goal readiness` 未通过：返回 `wait-human`，推荐补齐 Flow 0 / readiness。
2. 存在 `active issue`：保持 WIP=1，继续该 issue 的 `verify` / `review` / `update`。
3. 存在未完成 issue：继续未完成 issue，不创建新 issue。
4. 存在 active project / active milestone candidate：推荐该 project / milestone 的下一条 issue intent。
5. 无 project candidate：回退到 root `ROADMAP.md` / `docs/planning/construction-plan.md` 的候选施工包。
6. 无任何候选：返回 `wait-human`。

关键规则：

- WIP=1 优先级不能被 project backlog 降低。
- active issue 和 incomplete issue 永远优先于 active project candidate。
- `GoalLoopSelection.recommendedCommand` 仍只能是文本建议。
- recommended command 可以是 `agentflow plan "<intent>"`，但 GoalLoop 不执行它。

## Fallback 规则

缺失数据时的行为必须可预测：

| 缺失项 | Fallback |
| --- | --- |
| 没有 `.agentflow/workspace.json` seed | 使用 `LocalProjectModelSnapshot` 的只读默认 workspace；若不可用，回退 roadmap candidate |
| 没有 `.agentflow/projects/{project-id}.json` seed | 使用派生 active project；若不可用，回退 roadmap candidate |
| issue 没有 `projectLink` | 不迁移、不补写；active / incomplete issue 仍按现有状态继续 |
| active project 没有 `nextIssueIntent` | 回退 roadmap candidate |
| active milestone 不存在 | 回退 active project 的 project-level candidate；仍不可用时回退 roadmap candidate |
| project candidate 为空字符串 | 视为无 candidate，回退 roadmap candidate |

Fallback 不得创建文件、不得调用模型、不得把缺失 link 当作错误阻塞现有 run / verify / review。

## 当前阶段允许

| 能力 | 边界 |
| --- | --- |
| 定义关系 | 明确 workspace / project / milestone / issue link / selection 的最小关系 |
| 定义优先级 | 固定 readiness、active issue、incomplete issue、project candidate、roadmap fallback、wait-human 顺序 |
| 定义 fallback | 明确 seed / projectLink / nextIssueIntent 缺失时如何回退 |
| 定义验证 | search trace、readiness、no code decision change、no migration proof |
| 更新下一候选 | 实现完成后推荐 `Desktop GoalLoop Trace v0 只读展示` |

## 当前阶段不允许

| 禁止项 | 说明 |
| --- | --- |
| 绕过优先级改写 `goal_loop_decision` | active issue / incomplete issue 优先级不能被 project candidate 降低 |
| 自动执行命令 | 不执行 plan / run / verify / review / update |
| 批量写 `projectLink` | Issue link writer 已存在，但本阶段不迁移历史 issue |
| 迁移历史 issue | 不改写 `.agentflow/issues/*.json` 的归属字段 |
| Desktop 执行入口 | Desktop 仍只能展示 recommended command 文本 |
| 调用模型 | 本地决策和文档边界，不做语义规划 |
| 创建远程对象 | 不创建 GitHub PR、Linear issue、Jira/PingCode item |
| 绕过 IssueContract | Project candidate 必须先通过 `agentflow plan` 生成 IssueContract |

## 后续实现顺序

1. `Project-aware GoalLoop v0 实现`
   - 状态：已完成。
   - 在 `goal_loop_decision` 中加入 active project / active milestone candidate 读取。
   - 保持 readiness、active issue、incomplete issue 优先级不变。
   - 无 project candidate 时继续回退 roadmap candidate。
   - 只写 `goal-loop.json` 和 `GOAL-LOOP-SUMMARY.md`。
2. `Desktop GoalLoop Trace v0 只读展示`
   - 状态：已完成。
   - Desktop 展示 GoalLoop 决策来源、fallback、recommended command。
   - 不执行 recommended command。
   - 不写 issue、project seed 或 query 文件。
3. `Desktop Issue Lifecycle Trace v0 只读展示`
   - 下一候选。
   - Desktop 只读展示 issue contract -> run -> validation -> evidence -> review -> project update 链路。
   - 不执行 run / verify / review，不写 `.agentflow/`。

## 验证矩阵

| 验证 | 要求 |
| --- | --- |
| Boundary docs | README、ROADMAP、MVP Spec、Local Workspace Boundary、Issue Project Link Boundary 引用本文件 |
| Search trace | `agentflow search "Project-aware GoalLoop"` 返回可追溯结果 |
| Existing GoalLoop safety | active issue / incomplete issue 优先级保持文档约束 |
| Decision behavior | `goal_loop_decision` 只在无 active / incomplete issue 时读取 project candidate |
| No issue migration | live `.agentflow/issues/*.json` 不批量新增 `projectLink` |
| Existing issue-link preview | `agentflow issue-link ISSUE-0025` 仍默认 preview |
| Readiness | `bash checks/agentflow-readiness.sh` 通过 |

## Evidence 要求

本阶段 evidence 至少包含：

- 本文件路径。
- `agentflow search "Project-aware GoalLoop"` 输出摘要。
- `agentflow goal next` 输出摘要。
- project candidate decision proof。
- roadmap fallback proof。
- no issue migration proof。
- readiness 通过证明。

## 验收

1. Project-aware GoalLoop v0 按边界实现。
2. 只在无 active issue、无 incomplete issue 时读取 project candidate。
3. WIP=1 和 active issue 优先级不变。
4. 不迁移历史 issue。
5. 不自动执行任何命令。
6. `goal next` 在 Desktop GoalLoop Trace 完成后推荐 `Desktop Issue Lifecycle Trace v0 只读展示`。
