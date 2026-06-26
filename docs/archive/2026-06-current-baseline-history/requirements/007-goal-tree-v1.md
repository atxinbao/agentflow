# 007 - Goal Tree V1

创建日期：2026-06-02  
执行者：Codex

## 007.1 边界修正

2026-06-03 已新增：

```text
docs/requirements/007-1-goal-tree-agent-only-boundary-fix.md
```

该修正不推翻 Goal Tree V1 模型和 `.agentflow/define/**` 本地事实源，但明确：

```text
Goal Tree 是 Agent 使用的目标工作地图。
Desktop 人类界面只能只读查看，不创建、不编辑、不归档、不排序、不准备 Graph Context。
写入能力保留为未来 Agent-only / system-only 通道。
```

## 用户目标

AgentFlow 已经完成了当前阶段的三个基础底座：

```text
001 Project Workspace Manager
= 把本地项目文件夹接入 AgentFlow，并准备 .agentflow/

002 Graph
= 自动整理本地代码现场，生成 Agent 可用的项目地图

003 Project File Reader
= 稳定只读浏览本地项目文件
```

下一步需要进入：

```text
Goal Tree V1
```

大白话：

> 用户打开一个本地项目后，不只是看代码，也不只是建代码索引。  
> AgentFlow 还需要让用户把“这个项目要做什么”拆成清楚的目标树。  
> 目标树由 Goal / Milestone / Issue 组成。  
> 未来 Agent 执行时，必须基于这些目标和验收标准推进，而不是随便扫代码、随便写代码。

---

## 一句话定义

> **Goal Tree V1 是 AgentFlow 的本地目标树模块。它负责在一个本地 Project Workspace 下管理 Goal / Milestone / Issue，明确目标、范围、非目标、验收标准、依赖和执行顺序，为未来 AgentRun 提供稳定输入。**

---

## 与 OpenSpec 的关系

Goal Tree V1 不与 OpenSpec 冲突。

本需求采用 OpenSpec 的思想：

```text
先定义需求
先锁定边界
先明确非目标
先写验收标准
确认后再实现
```

但本阶段不引入 OpenSpec 工具链。

本阶段不新增：

```text
openspec/
openspec CLI
proposal.md
design.md
tasks.md
specs/
```

当前只使用：

```text
docs/requirements/007-goal-tree-v1.md
```

作为需求锁定入口。

关系是：

```text
OpenSpec 思想
= 管开发前的需求定义和变更边界

Goal Tree V1
= AgentFlow 内部的产品目标树能力
```

不能把 Goal Tree 当成 OpenSpec 的替代品。

---

## 背景

当前仓库已经经过旧代码清理：

```text
004 Legacy Cleanup and New Module Split
005 Legacy and Degraded Code Removal
006 Legacy CLI Retirement and Archive Pruning
```

旧 Workflow / Product Feature / GoalLoop / Run / Verify / Review / Closure / Evidence 等旧流程不再作为新主干继承。

因此，Goal Tree V1 必须是新的模型，不能复用旧流程模型。

特别禁止默认复用：

```text
旧 ProjectDefinition
旧 ProjectGoal
旧 ProductFeatureDraft
旧 IssueContract
旧 AgentRun
旧 GoalLoopState
旧 Eligibility
旧 Lease
旧 Evidence / Review / Update
旧 Project Closure / Code Audit / Docs Refresh
```

---

## 核心概念

Goal Tree V1 只有三个核心业务对象：

```text
Goal
Milestone
Issue
```

它们的父级不是旧 Product Project，而是：

```text
本地 Project Workspace
```

也就是：

```text
Local Project Folder
  └── Goal Tree
      ├── Goal
      │   ├── Milestone
      │   │   ├── Issue
      │   │   └── Issue
      │   └── Milestone
      └── Goal
```

---

# 1. 范围

Goal Tree V1 做这些事情：

```text
1. 定义 Goal / Milestone / Issue 的新模型
2. 定义 Goal / Milestone / Issue 的状态
3. 定义 Goal / Milestone / Issue 的本地存储
4. 定义 Goal Tree 的读取、创建、更新、排序、归档
5. 定义 Issue 依赖关系
6. 定义 Issue 验收标准
7. 定义 Issue 与 Graph Context Pack 的关系
8. 定义 Desktop 最小 UI
9. 定义 Tauri commands
10. 定义验证和完整性检查
```

---

# 2. 非目标

Goal Tree V1 不做以下事情：

```text
不启动 Agent
不定义 AgentRun
不定义 Lease
不执行测试
不执行项目命令
不调用模型
不创建 PR
不创建 GitHub Issue
不创建 Linear Issue
不接 OpenSpec 工具链
不接 Superpowers
不接 gstack
不复活旧 CLI
不复用旧 IssueContract
不复用旧 GoalLoop
不复用旧 Product Feature
不复用旧 Project Closure / Audit / Docs Refresh
不做多人协作
不做云同步
不写用户业务源码
不把 .agentflow/ 提交到 Git
```

---

# 3. 设计原则

## 3.1 Project 是本地项目文件夹

在 Goal Tree V1 中：

```text
Project = 本地项目文件夹 / Project Workspace
```

不是旧流程里的 Product Project。

因此 Goal Tree 的根是：

```text
projectRoot
```

不是：

```text
team_id
project_id
workspace_id
legacy project model
```

---

## 3.2 Goal / Milestone / Issue 是新的模型

Goal Tree V1 必须新建模型。

不能直接使用旧：

```text
IssueContract
ProjectDefinition
GoalLoopState
AgentRun
Lease
```

---

## 3.3 Human editable contract 优先

每个对象都分三层：

```text
1. Human editable contract
2. Agent assistance draft
3. System readonly state
```

解释：

```text
Human editable contract
= 用户确认过、可以作为产品约束的内容

Agent assistance draft
= Agent 或系统给出的建议，不自动成为事实

System readonly state
= 系统生成，不让用户手改，用于追踪状态、时间、来源、路径
```

---

## 3.4 Graph 只提供上下文，不替代目标

Graph 能告诉 AgentFlow：

```text
这个 Issue 可能相关哪些文件
哪些符号
哪些测试
哪些配置
```

但 Graph 不能替用户定义目标。

所以：

```text
Goal Tree = 要做什么
Graph = 项目现场在哪里
Project File Reader = 人怎么看文件
```

---

## 3.5 V1 不自动执行

Goal Tree V1 只是定义和管理目标树。

它不负责：

```text
claim issue
start run
lease
execute command
write code
verify code
review code
merge PR
```

这些属于未来 AgentRun / Execution 层。

---

# 4. 本地目录结构

当前 `.agentflow/` 已经有：

```text
.agentflow/
├── workspace.yaml
├── config.yaml
├── define/
│   ├── goals/
│   ├── milestones/
│   └── issues/
├── execute/
└── output/
```

Goal Tree V1 使用：

```text
.agentflow/define/
├── goal-tree.json
├── goals/
│   └── <goal-id>.json
├── milestones/
│   └── <milestone-id>.json
├── issues/
│   └── <issue-id>.json
└── exports/
    └── goal-tree.md
```

说明：

```text
goal-tree.json
= 目标树索引和排序

goals/
= Goal 对象

milestones/
= Milestone 对象

issues/
= Issue 对象

exports/goal-tree.md
= 可选的人类阅读导出，不是事实源
```

事实源只有：

```text
.agentflow/define/**/*.json
```

---

## 4.1 不进 Git

继续遵守：

```text
.agentflow/ 不进 Git
.agentflow/ 不进 PR
```

Goal Tree 所有数据也不进 Git：

```text
.agentflow/define/goals/**
.agentflow/define/milestones/**
.agentflow/define/issues/**
.agentflow/define/goal-tree.json
```

---

# 5. 数据模型

## 5.1 Goal

Goal 是一个阶段内最高层的产品目标。

大白话：

> Goal 说明这个本地项目接下来要完成什么。

### Goal JSON

```json
{
  "version": "goal.v1",
  "id": "goal-001",
  "projectRoot": "/path/to/project",
  "status": "active",

  "human": {
    "title": "Build local AgentFlow workflow foundation",
    "objective": "让 AgentFlow 能在本地项目中管理目标、阶段和任务。",
    "scope": [],
    "nonGoals": [],
    "successCriteria": [],
    "milestoneOrder": [],
    "validationGate": [],
    "closureGate": []
  },

  "agentDraft": {
    "suggestedMilestones": [],
    "suggestedRisks": [],
    "suggestedQuestions": [],
    "suggestedIssueBreakdown": []
  },

  "system": {
    "createdAt": 1780290000,
    "updatedAt": 1780290000,
    "createdBy": "human",
    "updatedBy": "human",
    "path": ".agentflow/define/goals/goal-001.json",
    "revision": 1
  }
}
```

### Goal status

```text
draft
active
paused
completed
archived
```

### Goal Human editable contract

```text
title
objective
scope
nonGoals
successCriteria
milestoneOrder
validationGate
closureGate
```

---

## 5.2 Milestone

Milestone 是 Goal 下的阶段目标。

大白话：

> Milestone 说明这个 Goal 要分几段完成，每一段进入和退出的标准是什么。

### Milestone JSON

```json
{
  "version": "milestone.v1",
  "id": "ms-001",
  "goalId": "goal-001",
  "projectRoot": "/path/to/project",
  "status": "active",

  "human": {
    "title": "Goal Tree V1",
    "stageGoal": "定义并实现 Goal / Milestone / Issue 的本地目标树。",
    "entryCriteria": [],
    "scope": [],
    "nonGoals": [],
    "issueOrder": [],
    "exitCriteria": [],
    "nextGate": []
  },

  "agentDraft": {
    "suggestedIssues": [],
    "suggestedRisks": [],
    "suggestedQuestions": []
  },

  "system": {
    "createdAt": 1780290000,
    "updatedAt": 1780290000,
    "createdBy": "human",
    "updatedBy": "human",
    "path": ".agentflow/define/milestones/ms-001.json",
    "revision": 1
  }
}
```

### Milestone status

```text
draft
planned
active
blocked
completed
archived
```

### Milestone Human editable contract

```text
title
stageGoal
entryCriteria
scope
nonGoals
issueOrder
exitCriteria
nextGate
```

---

## 5.3 Issue

Issue 是 AgentFlow 未来 Agent 可以执行的最小产品任务单位。

大白话：

> Issue 说明这件事要做什么、范围是什么、不能做什么、怎么验收、需要看哪些上下文。

### Issue JSON

```json
{
  "version": "issue.v1",
  "id": "iss-001",
  "goalId": "goal-001",
  "milestoneId": "ms-001",
  "projectRoot": "/path/to/project",
  "status": "ready",

  "human": {
    "title": "Create Goal Tree storage",
    "goal": "实现 Goal / Milestone / Issue 的本地 JSON 存储。",
    "scope": [],
    "nonGoals": [],
    "dependencies": [],
    "acceptanceCriteria": [],
    "validationCommands": [],
    "evidenceRequirements": [],
    "boundary": []
  },

  "agentDraft": {
    "suggestedFiles": [],
    "suggestedSymbols": [],
    "suggestedTests": [],
    "suggestedImplementationPlan": [],
    "suggestedRisks": [],
    "questions": []
  },

  "system": {
    "createdAt": 1780290000,
    "updatedAt": 1780290000,
    "createdBy": "human",
    "updatedBy": "human",
    "path": ".agentflow/define/issues/iss-001.json",
    "revision": 1,
    "graphContextPackPath": ".agentflow/output/graph/context-packs/iss-001.json"
  }
}
```

### Issue status

```text
draft
ready
blocked
completed
canceled
archived
```

V1 不使用：

```text
claimed
leased
running
in_review
merged
```

这些属于未来 AgentRun / Execution 层。

### Issue Human editable contract

```text
title
goal
scope
nonGoals
dependencies
acceptanceCriteria
validationCommands
evidenceRequirements
boundary
```

注意：

```text
validationCommands 可以填写，但 V1 不执行。
```

---

# 6. Goal Tree Index

路径：

```text
.agentflow/define/goal-tree.json
```

示例：

```json
{
  "version": "goal-tree.v1",
  "projectRoot": "/path/to/project",
  "activeGoalId": "goal-001",
  "goalOrder": ["goal-001"],
  "milestoneOrderByGoal": {
    "goal-001": ["ms-001", "ms-002"]
  },
  "issueOrderByMilestone": {
    "ms-001": ["iss-001", "iss-002"]
  },
  "updatedAt": 1780290000
}
```

用途：

```text
保存排序
保存 active goal
快速读取树结构
避免每次 UI 重新推断顺序
```

---

# 7. 完整性规则

Goal Tree V1 必须提供 integrity check。

## 7.1 必须满足

```text
Goal id 唯一
Milestone id 唯一
Issue id 唯一
Milestone.goalId 必须存在
Issue.goalId 必须存在
Issue.milestoneId 必须存在
Goal.milestoneOrder 里的 id 必须存在
Milestone.issueOrder 里的 id 必须存在
Issue.dependencies 里的 id 必须存在
不能出现循环依赖
不能引用 archived 对象作为 active dependency
```

## 7.2 状态规则

```text
completed Goal 不能包含 active Milestone
completed Milestone 不能包含 ready / blocked Issue
archived Goal 不在 activeGoalId 中
archived Milestone 不参与默认排序展示
archived Issue 不参与默认 ready 列表
```

## 7.3 验收规则

Issue 进入 `ready` 至少需要：

```text
title 非空
goal 非空
acceptanceCriteria 至少一条
boundary 至少一条
```

V1 不强制：

```text
validationCommands 必填
evidenceRequirements 必填
Graph Context Pack 必须存在
```

但缺失时要给 warning。

---

# 8. Graph 集成

Goal Tree V1 需要使用 Graph，但不能依赖 Graph 成功才能工作。

## 8.1 Issue Context Pack

当创建或更新 Issue 时，可以请求 Graph 生成 Context Pack。

输入：

```text
Issue title
Issue goal
acceptanceCriteria
scope
nonGoals
```

输出：

```text
recommendedFiles
recommendedSymbols
recommendedTests
impactHints
testHints
```

保存到：

```text
.agentflow/output/graph/context-packs/<issue-id>.json
```

Issue system 字段记录：

```json
"graphContextPackPath": ".agentflow/output/graph/context-packs/iss-001.json"
```

## 8.2 Graph 状态

如果 Graph 状态是：

```text
ready
degraded
```

Goal Tree 可以显示推荐上下文。

如果 Graph 状态是：

```text
missing
indexing
failed
```

Goal Tree 仍然可以创建和编辑对象，但显示 warning：

```text
代码地图尚未准备好，Issue 上下文推荐可能不完整。
```

---

# 9. Project File Reader 集成

Project File Reader 继续只读。

Goal Tree 可以把 Issue 的推荐文件传给 Project File Reader：

```text
Issue -> Graph Context Pack -> recommendedFiles -> Project File Reader
```

点击推荐文件：

```text
打开只读文件阅读器
不编辑
不写入
不执行命令
```

---

# 10. Rust 模块设计

## 10.1 新 crate

建议新增：

```text
crates/goal-tree/
```

Cargo package：

```toml
[package]
name = "agentflow-goal-tree"
```

加入 workspace：

```toml
members = [
  "crates/agentflow-core",
  "crates/agentflow-cli",
  "crates/graph",
  "crates/goal-tree",
  "apps/desktop/src-tauri"
]
```

原因：

```text
Goal Tree 是新产品核心模型
不能放进 agentflow-core legacy 兼容层
也不应该和 graph 混在一起
独立 crate 可以避免误依赖旧流程
```

## 10.2 crate 结构

```text
crates/goal-tree/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── manager.rs
    ├── storage.rs
    ├── integrity.rs
    ├── ids.rs
    ├── export.rs
    └── model/
        ├── mod.rs
        ├── goal.rs
        ├── milestone.rs
        ├── issue.rs
        ├── tree.rs
        └── snapshot.rs
```

## 10.3 禁止依赖

`agentflow-goal-tree` 不允许依赖：

```text
agentflow_core::legacy
agentflow_core::active
旧 IssueContract
旧 GoalLoop
旧 AgentRun
旧 Lease
```

如果需要通用路径 / JSON 工具，可以自行实现，或者后续抽出真正独立的 shared crate。

---

# 11. Tauri Commands

新增 Tauri command wrapper：

```text
apps/desktop/src-tauri/src/commands/goal_tree.rs
```

并新增内部模块：

```text
apps/desktop/src-tauri/src/goal_tree/
```

如果只是包装 crate，可以只需要 command wrapper。

## 11.1 Commands

### load_goal_tree_snapshot

```ts
load_goal_tree_snapshot(projectRoot?: string): GoalTreeSnapshot
```

用途：

```text
读取整个 Goal Tree
```

### create_goal_tree_goal

```ts
create_goal_tree_goal(projectRoot: string, input: CreateGoalInput): GoalRecord
```

### update_goal_tree_goal

```ts
update_goal_tree_goal(projectRoot: string, goalId: string, patch: UpdateGoalInput): GoalRecord
```

### archive_goal_tree_goal

```ts
archive_goal_tree_goal(projectRoot: string, goalId: string): GoalRecord
```

### create_goal_tree_milestone

```ts
create_goal_tree_milestone(projectRoot: string, goalId: string, input: CreateMilestoneInput): MilestoneRecord
```

### update_goal_tree_milestone

```ts
update_goal_tree_milestone(projectRoot: string, milestoneId: string, patch: UpdateMilestoneInput): MilestoneRecord
```

### archive_goal_tree_milestone

```ts
archive_goal_tree_milestone(projectRoot: string, milestoneId: string): MilestoneRecord
```

### create_goal_tree_issue

```ts
create_goal_tree_issue(projectRoot: string, milestoneId: string, input: CreateIssueInput): IssueRecord
```

### update_goal_tree_issue

```ts
update_goal_tree_issue(projectRoot: string, issueId: string, patch: UpdateIssueInput): IssueRecord
```

### archive_goal_tree_issue

```ts
archive_goal_tree_issue(projectRoot: string, issueId: string): IssueRecord
```

### reorder_goal_tree

```ts
reorder_goal_tree(projectRoot: string, input: ReorderGoalTreeInput): GoalTreeSnapshot
```

### validate_goal_tree

```ts
validate_goal_tree(projectRoot: string): GoalTreeValidationSnapshot
```

### prepare_goal_tree_issue_context

```ts
prepare_goal_tree_issue_context(projectRoot: string, issueId: string): GoalTreeIssueContextSnapshot
```

该命令可以调用 Graph Context Pack，但不启动 Agent。

---

# 12. Desktop UI

新增或改造一个页面区域：

```text
Goal Tree
```

## 12.1 页面结构

```text
左侧：Goal Tree 列表
  Goal
    Milestone
      Issue

中间：详情编辑器
  当前选中 Goal / Milestone / Issue 的 Human contract

右侧：Context
  Graph 推荐文件
  Graph 推荐测试
  Project File Reader 入口
```

## 12.2 V1 UI 必须支持

```text
创建 Goal
编辑 Goal
归档 Goal
创建 Milestone
编辑 Milestone
归档 Milestone
创建 Issue
编辑 Issue
归档 Issue
调整 Milestone 顺序
调整 Issue 顺序
查看 integrity warning
查看 Graph context warning
查看推荐文件
点击推荐文件打开 Project File Reader
```

## 12.3 V1 UI 不做

```text
拖拽排序可以不做，先用上移 / 下移
不做多人协作
不做自动生成目标
不做模型生成
不做 Agent 执行按钮
不做 Run 状态
不做 PR 状态
```

---

# 13. Frontend 模块设计

建议新增：

```text
apps/desktop/src/features/goal-tree/
├── index.ts
├── GoalTreePage.tsx
├── GoalTree.css
│
├── browser/
│   ├── GoalTreeBrowser.tsx
│   └── goalTreeRows.ts
│
├── editor/
│   ├── GoalEditor.tsx
│   ├── MilestoneEditor.tsx
│   └── IssueEditor.tsx
│
├── context/
│   ├── GoalTreeContextPanel.tsx
│   └── issueGraphContext.ts
│
├── hooks/
│   ├── useGoalTree.ts
│   ├── useGoalTreeSelection.ts
│   └── useGoalTreeValidation.ts
│
└── model/
    ├── goalTreeTypes.ts
    └── goalTreeUtils.ts
```

---

# 14. Types

新增：

```text
apps/desktop/src/types/goalTree.ts
```

并从：

```text
apps/desktop/src/types/index.ts
```

导出。

包含：

```text
GoalTreeSnapshot
GoalRecord
MilestoneRecord
IssueRecord
GoalHumanContract
MilestoneHumanContract
IssueHumanContract
GoalAgentDraft
MilestoneAgentDraft
IssueAgentDraft
GoalSystemState
MilestoneSystemState
IssueSystemState
GoalTreeValidationSnapshot
GoalTreeIssueContextSnapshot
```

---

# 15. Browser Preview

浏览器预览模式下不能写真实 `.agentflow/`。

需要新增 mock：

```text
createBrowserPreviewGoalTreeSnapshot
createBrowserPreviewGoal
createBrowserPreviewMilestone
createBrowserPreviewIssue
createBrowserPreviewGoalTreeValidation
```

要求：

```text
浏览器预览 UI 不崩
不写真实文件
不调用 Tauri
```

---

# 16. 与旧代码的关系

Goal Tree V1 必须明确不依赖：

```text
agentflow_core::legacy
legacy/archive_2026_05.rs
legacy/team_project_milestone_issue
legacy/workflow_control
legacy/run_verify_review
legacy/eligibility_lease
legacy/project_closure
```

如果发现需要复用某个工具函数，必须：

```text
1. 确认它不是业务旧流程
2. 移到独立 shared crate 或 goal-tree 内部
3. 不通过 legacy import
```

---

# 17. 数据写入边界

Goal Tree V1 只允许写：

```text
.agentflow/define/goal-tree.json
.agentflow/define/goals/**
.agentflow/define/milestones/**
.agentflow/define/issues/**
.agentflow/define/exports/**
```

允许调用 Graph 时写：

```text
.agentflow/output/graph/context-packs/**
```

不允许写：

```text
用户源码
.gitignore
远程服务
旧 .agentflow/issues/
旧 .agentflow/runs/
旧 .agentflow/evidence/
旧 .agentflow/reviews/
旧 .agentflow/updates/
旧 .agentflow/views/
```

---

# 18. 开发切片

## Slice 1：Spec and crate scaffold

目标：

```text
新增 docs/requirements/007-goal-tree-v1.md
新增 crates/goal-tree
加入 workspace
定义空模型和测试骨架
```

验收：

```text
cargo test -p agentflow-goal-tree 通过
```

## Slice 2：Data model + storage

目标：

```text
Goal / Milestone / Issue / GoalTreeIndex 模型
JSON 读写
atomic write
load snapshot
```

验收：

```text
可以创建文件
可以读取 snapshot
不会写 legacy path
```

## Slice 3：CRUD and ordering

目标：

```text
create/update/archive goal
create/update/archive milestone
create/update/archive issue
reorder goal/milestone/issue
```

验收：

```text
排序稳定
归档对象默认隐藏但可读取
```

## Slice 4：Integrity validation

目标：

```text
validate_goal_tree
循环依赖检查
缺失字段 warning
孤儿对象检查
```

验收：

```text
错误和 warning 可读
```

## Slice 5：Graph context integration

目标：

```text
prepare_goal_tree_issue_context
调用 Graph Context Pack
把 path 写入 Issue system.graphContextPackPath
```

验收：

```text
Graph missing 时不阻塞
Graph ready 时生成推荐上下文
```

## Slice 6：Tauri commands

目标：

```text
新增 goal_tree command wrapper
注册 commands
```

验收：

```text
Tauri build 通过
command 名称稳定
```

## Slice 7：Desktop UI

目标：

```text
GoalTreePage
GoalTreeBrowser
Goal/Milestone/Issue Editor
Context Panel
```

验收：

```text
能创建和编辑 Goal Tree
能查看推荐文件
不启动 Agent
```

## Slice 8：Browser preview + docs

目标：

```text
browser preview mock
docs update
verification update
```

验收：

```text
browser preview 不崩
npm build 通过
```

---

# 19. 验收标准总表

```text
- [ ] 新增 docs/requirements/007-goal-tree-v1.md。
- [ ] 新增 crates/goal-tree。
- [ ] Goal / Milestone / Issue 使用新模型，不复用旧 IssueContract。
- [ ] Goal Tree 父级是本地 Project Workspace。
- [ ] .agentflow/define/goals 可以写入 Goal。
- [ ] .agentflow/define/milestones 可以写入 Milestone。
- [ ] .agentflow/define/issues 可以写入 Issue。
- [ ] goal-tree.json 可以记录排序。
- [ ] 支持创建 / 更新 / 归档 Goal。
- [ ] 支持创建 / 更新 / 归档 Milestone。
- [ ] 支持创建 / 更新 / 归档 Issue。
- [ ] 支持 Milestone / Issue 排序。
- [ ] 支持 Issue dependency。
- [ ] 支持 integrity validation。
- [ ] 支持 Graph Context Pack 关联。
- [ ] Graph 失败不阻塞 Goal Tree 编辑。
- [ ] Project File Reader 可以打开推荐文件。
- [ ] Desktop UI 可以展示 Goal Tree。
- [ ] Browser Preview 有 mock。
- [ ] 不启动 Agent。
- [ ] 不执行命令。
- [ ] 不调用模型。
- [ ] 不写用户源码。
- [ ] 不写旧 .agentflow/issues/runs/evidence 路径。
- [ ] 不依赖 agentflow_core::legacy。
- [ ] cargo fmt --check 通过。
- [ ] cargo test -p agentflow-goal-tree 通过。
- [ ] cargo test 通过。
- [ ] npm --prefix apps/desktop run build 通过。
- [ ] git diff --check 通过。
```

---

# 20. 验证命令

必须执行：

```bash
cargo fmt --check
cargo test -p agentflow-goal-tree
cargo test
npm --prefix apps/desktop run build
git diff --check
```

如果改动 Desktop Tauri：

```bash
cargo test -p agentflow-desktop
```

---

# 21. PR 说明要求

PR 描述必须说明：

```text
1. Goal Tree 是否复用旧模型：必须说明没有复用。
2. 写入了哪些 .agentflow 路径。
3. 没有写入哪些旧路径。
4. 是否调用 Graph。
5. Graph 失败时如何降级。
6. 是否启动 Agent：必须说明没有。
7. 是否执行命令：必须说明没有。
8. 是否调用模型：必须说明没有。
9. 验证命令和结果。
```

---

# 22. Codex 执行指令

```md
请执行 007 - Goal Tree V1。

目标：
实现 AgentFlow 的本地 Goal Tree V1，用新的 Goal / Milestone / Issue 模型管理本地 Project Workspace 下的目标树，为未来 AgentRun 提供输入。

必须遵守：
1. 不接 OpenSpec 工具链。
2. 但需求和实现必须遵守 OpenSpec 思想：先锁定范围和验收，再实现。
3. 不复用旧 IssueContract。
4. 不复用旧 GoalLoop。
5. 不复用旧 Product Feature。
6. 不复用旧 AgentRun。
7. 不依赖 agentflow_core::legacy。
8. 不启动 Agent。
9. 不执行项目命令。
10. 不调用模型。
11. 不写用户源码。
12. 不写旧 .agentflow/issues、runs、evidence、reviews、updates、views。
13. 只写 .agentflow/define/**，以及 Graph Context Pack 写入 .agentflow/output/graph/context-packs/**。
14. Graph 失败时不阻塞 Goal Tree。
15. Project File Reader 只读打开推荐文件。

实现范围：
- 新增 crates/goal-tree。
- 新增 Goal / Milestone / Issue / GoalTreeIndex 模型。
- 新增 JSON storage。
- 新增 load/create/update/archive/reorder/validate APIs。
- 新增 Tauri goal_tree commands。
- 新增 Desktop Goal Tree UI。
- 新增 browser preview mock。
- 新增 types/goalTree.ts。
- 更新 docs 和 verification。

验证命令：
- cargo fmt --check
- cargo test -p agentflow-goal-tree
- cargo test
- npm --prefix apps/desktop run build
- git diff --check
```

---

# 23. 完成定义

Goal Tree V1 完成后，AgentFlow 应达到：

```text
用户可以在本地 Project Workspace 下创建 Goal
用户可以把 Goal 拆成 Milestone
用户可以把 Milestone 拆成 Issue
Issue 有明确目标、范围、非目标、依赖、验收标准和边界
Goal Tree 数据本地存储在 .agentflow/define/**
Graph 可以给 Issue 提供上下文包
Project File Reader 可以打开推荐文件
系统不启动 Agent、不执行命令、不调用模型
新模型不依赖旧 legacy 流程
```

最终一句话：

> **Goal Tree V1 是 AgentFlow 从“项目接入 + 代码现场”进入“目标驱动开发”的第一步。它只定义目标树，不执行目标。**
