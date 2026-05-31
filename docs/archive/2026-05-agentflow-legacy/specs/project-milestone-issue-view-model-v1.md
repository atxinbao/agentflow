# Project / Milestone / Issue / View Model v1

创建日期：2026-05-28
执行者：Codex
状态：implemented / schema-adapter-readonly / writer-preview-aligned

## Goal

本规格把 AgentFlow MVP 的产品主干收敛为四层：

```text
Workspace / Team
-> Project
   -> Milestone
      -> Issue
-> View
```

`View` 不是业务层级，只是 saved filter。它可以筛选“当前 Todo”、“高风险任务”、“缺证据任务”或“My agent queue”，但不能改变 Project / Milestone / Issue 的归属关系，也不能承载业务状态。

## Core Relationship

```text
Project = 做什么，为什么做，做到什么程度算完成
Milestone = 一个阶段，控制阶段边界和验收门
Issue = 最小可执行任务，Agent 只吃 issue
View = saved filter，不改变 Project / Milestone / Issue 关系
```

| 对象 | 负责什么 | 包含什么 | 不应该包含什么 |
| --- | --- | --- | --- |
| Project | 方向、边界、整体成功标准 | milestones、issue order、non-goals、closure gate | 不写完整实现细节 |
| Milestone | 阶段目标、阶段出口 | issues、entry / exit criteria、stage validation | 不直接执行代码 |
| Issue | 单个可执行合同 | goal、scope、non-goals、validation、evidence | 不跨 milestone 扩范围 |
| View | 展示和过滤 | query / filter / sort / layout | 不承载业务状态 |

## Product Invariants

```text
Project 不执行。
Milestone 不执行。
Issue 执行。
View 只展示。
Queue Preflight 决定谁能执行。
Evidence 决定是否 Done。
```

这些不变量高于 UI 呈现方式。Desktop 可以展示 Project、Milestone、Issue 和 View，但不能因为展示方便而把 View 做成业务父级，也不能让 Project / Milestone 绕过 Issue 进入执行。

## Project Template

Project 是 charter，不是 issue body 的合集。

```md
# Project: <name>

## Goal
<这个项目最终要交付什么>

## Target Maturity
<L1 / L2 / L3...>

## Target Layers
- <Engine / Layer>

## Scope
- <允许做什么>

## Non-goals
- <明确不做什么>

## Success Criteria
- [ ] <项目完成标准>

## Milestones
1. <Milestone 1>
2. <Milestone 2>
3. <Milestone 3>

## Issue Order
1. <Issue title>
2. <Issue title>

## Dependencies
- <Project / Milestone / Issue dependencies>

## Validation Gate
- `git diff --check`
- `<project checks command>`

## Evidence Requirements
- PR links
- merge commits
- checks result
- validation output
- stage audit input
- root docs refresh

## Queue Rule
- WIP=1
- Only one eligible issue can enter Todo
- Issue execution requires queue preflight

## Closure Gate
- All issues Done
- Stage Code Audit
- Root Docs Refresh
- Final closure summary

## Boundary
<禁止能力 / 禁止范围>
```

Project 页面只展示项目级信息：

```text
Project header:
- name / status / target maturity / owner / progress

Progress:
- milestone progress
- issue progress
- evidence completeness

Milestones:
- ordered milestone list

Queue:
- current eligible issue
- Todo / In Progress / In Review count
- blocker status

Closure:
- audit status
- docs refresh status
```

## Milestone Template

Milestone 是阶段门。它控制“这一段做到什么程度可以进入下一段”。

```md
# Milestone: <name>

## Goal
<这个阶段要达成什么>

## Entry Criteria
- [ ] <进入本阶段前必须满足什么>

## Scope
- <本阶段允许做什么>

## Non-goals
- <本阶段不做什么>

## Issues
| Order | Issue | Dependency | Status |
|---|---|---|---|

## Exit Criteria
- [ ] 所有 issues Done
- [ ] validation passed
- [ ] evidence complete
- [ ] no blocker
- [ ] milestone review complete

## Validation
- `<commands>`

## Evidence Required
- completed issue list
- PR / checks / merge evidence
- milestone review
- deferred work

## Next Milestone Gate
<是否允许进入下一 milestone>
```

一个很小的 Project 可以只有一个 Milestone；中大型 Project 应强制拆 Milestone，避免 Agent 一次跨阶段执行。

## Issue Template

Issue 是真正给 Agent 执行的合同，必须最严格。

```md
# Issue: <title>

## Goal
<唯一目标>

## Scope
- <允许做什么>

## Non-goals
- <禁止做什么>

## Dependencies
- <blocked by ...>

## Codex Instructions
1. Read source docs.
2. Inspect existing implementation.
3. Modify only authorized files / layers.
4. Add focused tests.
5. Run validation.
6. Create PR with evidence.

## Acceptance Criteria
- [ ] 功能 / 文档 / 合同完成
- [ ] dependencies satisfied
- [ ] no scope creep
- [ ] validation passed
- [ ] evidence complete

## Validation Commands
- `git diff --check`
- `<focused test>`
- `<full check>`

## Evidence Required
- Linked issue
- changed files summary
- validation output
- PR URL
- checks result
- merge commit
- boundary evidence
- rollback plan

## Allowed Files / Areas
- <allowed paths>

## Forbidden Files / Areas
- <forbidden paths>

## Boundary
<明确禁止能力>

## Initial State
Backlog / non-executable
```

## View Template

View 是 saved filter，只能保存展示偏好：

```json
{
  "id": "view-current-todo",
  "name": "当前 Todo",
  "entity": "issue",
  "filter": {
    "projectId": "<project-id>",
    "status": ["Todo"],
    "riskLevel": ["medium", "high"]
  },
  "sort": [
    {"field": "milestoneOrder", "direction": "asc"},
    {"field": "issueOrder", "direction": "asc"}
  ],
  "layout": "list"
}
```

View 不允许写 Project / Milestone / Issue 状态，不允许执行 run / verify / review，不允许保存搜索结果作为事实源。

## State Machines

### Project

```text
Draft -> Planned -> Active -> Closing -> Completed
                         -> Blocked
```

含义：

- `Draft`：草稿，尚未确认边界。
- `Planned`：项目范围和 Milestones 已确认，但还未激活执行。
- `Active`：当前正在推进。
- `Closing`：所有执行任务已完成，正在做 closure / audit / docs refresh。
- `Completed`：完成并收口。
- `Blocked`：存在阻塞，不能推进。

### Milestone

```text
Draft -> Ready -> Active -> Review -> Done
                         -> Blocked
```

Milestone 是阶段门。它可以维护阶段 gate 状态，但不能执行代码；后续如果 MVP 状态模型继续保持“Milestone 不维护产品状态”，则这些状态只能作为 gate 派生状态或 review 状态展示。

### Issue

```text
Backlog -> Todo -> In Progress -> In Review -> Done
                          -> Blocked
                          -> Repair
```

关键区别：

```text
Backlog = 已创建，但不可执行
Todo = queue preflight 后唯一可执行
In Progress = 已被 agent 领取
In Review = PR / checks / review 阶段
Done = merged + evidence complete
```

不要把 `Backlog` 当成可执行。真正可执行的是 `Todo`，且同一 Project 默认只能有一个 code-changing `Todo`。

## Compatibility With Current MVP Status Model

当前已实现的 `Project / Issue Status Model v0` 使用小写 canonical status：

```text
Project: draft / active / paused / completed / canceled
Issue: backlog / todo / in_progress / in_review / done / canceled
Milestone: 不维护独立产品状态
```

本 v1 是产品目标模型，不要求立刻破坏既有 `.agentflow/` 或 reader。后续实现迁移时按以下方向收敛：

| v1 Product State | Current Project Status | Current Issue Status |
| --- | --- | --- |
| Draft | draft | backlog |
| Planned | draft | backlog / todo, 由 queue preflight 决定 |
| Active | active | in_progress |
| Closing | active / paused, 由 closure state 派生 | in_review |
| Completed / Done | completed | done |
| Blocked | paused | backlog |
| Repair | - | backlog 或 todo，必须显式标记 repair 类型 |
| Canceled | canceled | canceled |

后续 writer 可以先继续写当前 canonical status；queue preflight 再负责把可执行 issue 从 `backlog` 推进到 `todo`。

## Schema Adapter v1

当前实现已经新增只读 adapter：

```text
read_project_milestone_issue_view_model_snapshot
  -> LocalProjectModelSnapshot
  -> .agentflow/issues/*.json
  -> .agentflow/views/*.json
  -> ProjectMilestoneIssueViewModelSnapshot
```

该 adapter 只读派生 v1 schema，不写 `.agentflow/`，不修改现有 Project / Issue status，不改变 writer 行为。

输出对象：

```text
ProjectMilestoneIssueViewModelSnapshot
V1WorkspaceRef
V1TeamRef
V1Project
V1Milestone
V1Issue
V1View
V1ViewFilter
V1ViewSort
```

状态处理：

- `rawStatus` 保留现有事实源状态。
- `status` 输出 v1 派生状态。
- Project `draft / active / paused / completed / canceled` 通过 adapter 映射到 v1 目标状态。
- Issue `backlog / todo / in_progress / in_review / done / canceled` 通过 adapter 映射到 v1 目标状态。
- Milestone 状态从 active milestone、issue progress 和 issue 状态派生，不写回事实源。

View 处理：

- `SavedView` 派生为 `V1View`。
- `V1View.entity = issue`。
- `V1View.layout = list`。
- `V1View` 只保存 filter / sort / layout，不保存结果，不写业务状态。

## Writer Preview Alignment v1

当前实现已经把 Team / Project / Milestone / Issue 的创建预览对齐到 v1 产品模型，但不改变落盘 schema。

```text
agentflow team create
  -> CreationPreview.v1Contract.team

agentflow project create
  -> CreationPreview.v1Contract.projectCharter

agentflow milestone create
  -> CreationPreview.v1Contract.milestoneGate

agentflow issue create
  -> CreationPreview.v1Contract.issueContract
```

预览输出包含：

```text
CreationV1ContractPreview
TeamCreationV1Preview
ProjectCharterV1Preview
MilestoneGateV1Preview
IssueContractV1Preview
ViewFilterV1Preview
```

边界：

- writer preview 可以展示 Project charter、Milestone gate、Issue execution contract。
- writer preview 仍默认只读，只有 `--write --yes` 才按 v0 writer 规则落盘。
- writer preview 不执行 run / verify / review。
- writer preview 不调用模型，不创建远程 GitHub / Linear 对象。
- View writer 后置；当前只保留 `ViewFilterV1Preview` 类型作为后续 saved filter 合同。

## Queue Preflight

Queue Preflight 是系统稳定自动推进的核心。

```md
# Queue Preflight

## Target
- project:
- milestone:
- candidateIssue:

## Checks
- [ ] Project status is Planned or Active
- [ ] Candidate issue exists
- [ ] Candidate issue is Backlog
- [ ] Dependencies are Done
- [ ] Todo count = 0
- [ ] In Progress count = 0
- [ ] In Review count = 0
- [ ] WIP=1 satisfied
- [ ] Issue body has complete execution contract
- [ ] No active conflicting PR
- [ ] Source record exists on main
- [ ] No stale handoff / stale workspace conflict

## Result
- eligible: true / false
- promotedIssue:
- blockedReason:
- nextAction:
```

只有 Queue Preflight 通过，才允许：

```text
Backlog -> Todo
```

## Recommended Data Model

```ts
type Project = {
  id: string
  name: string
  status: "Draft" | "Planned" | "Active" | "Closing" | "Completed" | "Blocked"
  goal: string
  targetMaturity?: string
  targetLayers: string[]
  scope: string[]
  nonGoals: string[]
  milestones: string[]
  issueOrder: string[]
  validationGate: string[]
  evidenceRequired: string[]
  closureGate: string[]
}

type Milestone = {
  id: string
  projectId: string
  name: string
  status: "Draft" | "Ready" | "Active" | "Review" | "Done" | "Blocked"
  goal: string
  entryCriteria: string[]
  exitCriteria: string[]
  issueIds: string[]
  validation: string[]
  evidenceRequired: string[]
}

type Issue = {
  id: string
  projectId: string
  milestoneId?: string
  title: string
  status: "Backlog" | "Todo" | "In Progress" | "In Review" | "Done" | "Blocked" | "Repair"
  goal: string
  scope: string[]
  nonGoals: string[]
  dependencies: string[]
  acceptanceCriteria: string[]
  validationCommands: string[]
  evidenceRequired: string[]
  allowedFiles: string[]
  forbiddenFiles: string[]
  boundary: string[]
  riskLevel: "low" | "medium" | "high"
  lease?: {
    owner: string
    expiresAt: string
  }
}
```

## Desktop Panel Structure

左侧栏目保留：

```text
项目
任务
视图
```

内部组织：

```text
项目
- Project 列表
- 每个 Project 下显示 milestone progress

任务
- 所有 issue
- 当前 active issue
- blocked issue
- missing evidence issue

视图
- 当前 Todo
- 当前 In Progress
- 缺证据
- 高风险
- Ready for closure
- My agent queue
```

Project 详情页：

```text
Project Charter
Milestones
Issue Order
Queue Status
Evidence Progress
Closure Gate
```

Milestone 详情页：

```text
Milestone Goal
Entry Criteria
Issues
Exit Criteria
Milestone Review
```

Issue 详情页：

```text
Header
- issue id / title / status / type / risk / owner

Contract
- goal / scope / non-goals / dependencies

Execution
- instructions / allowed files / forbidden files

Validation
- commands / acceptance criteria / expected output

Evidence
- required artifacts / PR evidence / merge evidence

Queue / Lease
- eligible / blocker / lease owner / lease expires

Boundary
- allowed / forbidden / non-goals

Notes
- human notes / external confirmation
```

## Non-Goals

- 不做完整 Linear clone。
- 不把 View 做成业务状态或业务父级。
- 不让 Project / Milestone 直接执行 run / verify / review。
- 不让 Desktop 写 Project / Milestone / Issue / View，除非另有 writer contract 和用户确认门。
- 不接入 SaaS、账号、支付、云同步。
- 不创建远程 GitHub / Linear 对象。
- 不调用模型自动拆解或自动执行。

## Implementation Direction

后续实现应按以下顺序推进：

1. 已完成：`Project / Milestone / Issue / View Model v1 writer preview 对齐`。
2. 已完成：`Desktop Project / Milestone / Issue 页面职责收敛 v0`。Desktop 通过 `ProjectMilestoneIssueViewModelSnapshot` 只读展示 Project charter、Milestone gate、Issue contract 和 View filter；Project 不展示 issue 执行细节，Milestone 不展示完整 evidence / review，Issue 不展示 project closure / audit。
3. Queue Preflight v1：把 `Backlog -> Todo` 从人工状态切换改为本地计算 + 显式确认。
4. Saved View v1：把“当前 Todo / 高风险 / 缺证据 / Ready for closure”做成 filter，而不是新业务层级。
