# Desktop Workbench MVP Boundary

创建日期：2026-05-22
执行者：Codex

## 定位

Desktop Workbench MVP v0 是 AgentFlow CLI 闭环之上的只读桌面工作台。它的职责是把 `.agentflow/` 中已经存在的 goal、goal loop、issue、run、validation、evidence、review、saved view 和 project summary 变成可浏览界面。

它不是新的执行引擎，不绕过 `agentflow-core`，不替代 CLI 的事实写入规则。

## 首屏目标

10 秒内回答四个问题：

1. 当前项目是否已经初始化。
2. 当前有多少 issue、run、update、saved view。
3. Goal Loop 当前建议的 next action 和 recommended command 是什么。
4. 哪些 issue 已完成，哪些需要处理，最近 evidence / review 在哪里。

## 输入事实源

| Source | 用途 | 权限 |
| --- | --- | --- |
| `.agentflow/goal.json` | 项目目标和边界 | read-only |
| `.agentflow/index.json` | issue 顺序和状态 | read-only |
| `.agentflow/context.json` | repo map 和 validation commands | read-only |
| `.agentflow/updates/PROJECT-SUMMARY.md` | 项目摘要 | read-only |
| `.agentflow/goal-loop.json` | 下一步本地决策 | read-only |
| `.agentflow/updates/GOAL-LOOP-SUMMARY.md` | 下一步建议摘要 | read-only |
| `.agentflow/views/*.json` | SavedView filter | read-only |
| `.agentflow/workspace.json` | Workspace active project / team references | read-only |
| `.agentflow/teams/*.json` | Team project / issue references | read-only |
| `.agentflow/projects/*.json` | Project charter / milestones / issue order | read-only |
| `.agentflow/issues/*.json` | issue contract 列表和详情 | read-only |
| `.agentflow/runs/*/run.json` | run 状态和 validation | read-only |
| `.agentflow/evidence/*.md` | evidence 内容 | read-only |
| `.agentflow/reviews/*.md` | review / assistant 内容 | read-only |
| `.agentflow/updates/PROJECT-UPDATE-*.md` | project update 内容和 issue 生命周期链接 | read-only |
| `.agentflow/index.sqlite` | 可重建查询索引 | read-only cache |

## MVP 内功能

| Area | Scope |
| --- | --- |
| 总览 | 显示 Workspace 摘要、Project Summary、Goal Loop Summary、counts、next action、recommended command |
| 团队 | 显示团队父栏目、项目子栏目、任务子栏目；支持多个团队，每个团队下展示关联项目和任务 |
| 项目 | 显示 Project charter、milestones、issue progress、queue status 和 closure gate |
| 任务 | 显示 Issue execution contract、goal、scope、non-goals、validation、evidence、boundary 和 status |
| 视图 | 显示 saved filter / sort / layout，不承载业务状态 |
| Refresh | 重新读取本地文件，不执行任务 |

Desktop 产品层级必须遵循 `Project / Milestone / Issue / View Model v1`。在用户界面中，`Default Workspace` 是系统隐含上下文，不进入用户模板；`Project` 是顶层业务容器，`Milestone` 是阶段容器，`Issue` 是唯一执行单元：

```text
Project
  -> Milestone
  -> Issue
View
```

`View` 是 saved filter，不是业务层级。它可以展示“当前 Todo / 高风险 / 缺证据 / Ready for closure”，但不能写 Project / Milestone / Issue 状态，不能保存结果为事实源，不能执行命令。

产品业务流程按以下主线收敛：

```text
定义项目 -> 拆阶段 -> 拆任务 -> Agent 执行 -> 系统派生进度
```

用户动线：

```text
1. 用户新建项目，只输入项目名称。
2. 用户填写 Project Goal：项目目标、背景 / 参考、范围、非目标、成功标准、验证门槛、证据要求。
3. Agent 辅助生成 Milestone 草案，用户确认阶段顺序和阶段目标。
4. 用户或 Agent 补充 Architecture / Environment / Agent 参与边界。
5. Agent 辅助生成 Issue Contracts，用户预览并确认写入。
6. 系统做 Queue Preflight，计算唯一可执行任务。
7. Agent 执行唯一 eligible issue。
8. 系统记录 evidence / progress，并派生下一个 issue / milestone 状态。
```

左侧栏目必须表达最小项目树：

```text
项目
  项目 1
    里程碑
    任务
  项目 2
    里程碑
    任务
```

Project 页面展示五个模板页签：`Goal | Milestone | Architecture | Environment | Agent`。Project 模板不展示执行细节，也不展示 project closure / milestone closeout。顶部派生状态统一命名为 `System Snapshot`，只在项目已有任务后展示；新建项目阶段不展示 runtime/status。Milestone 页面只展示里程碑列表和选中里程碑详情。Issue 页面展示任务队列和轻量任务执行卡片，只保留 Goal、Scope、Likely files、Non-goals、Dependencies、Acceptance criteria、Required commands、Evidence required、Boundary；Status、Eligible、Lease、PR、Checks、Merge commit、Evidence status、Blocked reason 只进入 `System Snapshot`，不进入前台合同正文。View 页面只展示 saved filter / sort / layout，不承载业务状态。

## 明确不做

| Out of Scope | Reason |
| --- | --- |
| 创建或编辑 issue contract | 避免 UI 绕过 CLI contract 规则 |
| 执行 `run / verify / review` | 第一版桌面只读，执行仍由 CLI 控制 |
| 调用模型或 Codex API | Runtime Adapter 仍是后续 gated work |
| 写入 `.agentflow/` 事实源 | v0 不改变执行事实 |
| 创建 PR / 远程 issue | 不接 GitHub / Linear / team workspace |
| 登录、账号、云同步 | 保持本地免费工具边界 |
| 数据库迁移管理 | `index.sqlite` 仍是可重建 cache |
| 完整 PM 看板 | 不是 Linear / Jira 替代品 |

## UI 边界

第一屏采用三栏密集工作台，不做 landing page：

| Region | 内容 |
| --- | --- |
| Left rail | 工作区、团队、项目、任务、视图 |
| Main pane | 当前选中对象详情 |
| Right pane | validation、evidence links、review assistant checklist |

视觉规则：

- 开发者工具风格，信息密集但清晰。
- 不做营销 hero、插画、装饰卡片堆叠。
- 卡片只用于 repeated items，不把页面区块包装成大卡片。
- 首屏必须直接显示项目状态，不出现空泛欢迎页。

## 技术边界

| Layer | Decision |
| --- | --- |
| Desktop shell | Tauri 2 |
| UI | React + TypeScript |
| Data access | 通过 Tauri command 调用 Rust reader |
| Core reuse | 复用 `agentflow-core` 类型和读取逻辑 |
| Write policy | v0 no fact writes |
| Network | disabled by product scope |

## 实现状态

| Area | Artifact |
| --- | --- |
| Rust snapshot reader | `agentflow-core::read_desktop_workbench_snapshot` |
| Tauri command | `apps/desktop/src-tauri/src/main.rs` 的 `load_workbench_snapshot` |
| v1 Tauri command | `apps/desktop/src-tauri/src/main.rs` 的 `load_project_milestone_issue_view_model_snapshot` |
| React UI | `apps/desktop/src/App.tsx` |
| UI types | `apps/desktop/src/types.ts` |
| Browser preview fallback | `apps/desktop/src/mockSnapshot.ts` |
| Styles | `apps/desktop/src/styles.css` |
| Desktop GoalLoop Trace v0 | 已实现“决策”只读视图，复用 `DesktopWorkbenchSnapshot.goalLoop` 和 Goal Loop summary markdown |
| Desktop Issue Lifecycle Trace v0 | 已实现“生命周期”只读视图，复用 `DesktopWorkbenchSnapshot` 的 issue / run / evidence / review / project update |
| Desktop Project Update Timeline v0 | 已实现“更新时间线”只读视图，复用 projectUpdates / issue / run / evidence / review |
| Desktop MVP Navigation Scope Reduction v0 | 已把 MVP 主导航收敛为总览、团队、项目、任务；trace/debug 视图不作为主入口 |
| Desktop Team Hierarchy v0 | 已把团队入口收敛为 workspace 多团队层级；每个团队下只读展示项目和任务，不提供新建/编辑入口 |
| Desktop Team Parent Child Columns v0 | 已把团队页改为三栏父子关系：团队是父栏目，项目和任务是选中团队下的子栏目 |
| Desktop Workspace Sidebar Tree v0 | 已把左侧栏目改为 Workspace / Team 工作区树，每个父节点下展示 project / issues 子项 |
| Desktop Teams Add Button v0 | 已在 TEAMS 标题右侧增加 `+` 入口；点击只打开新增团队面板，不保存、不写 `.agentflow/teams/` |
| Desktop Project / Milestone / Issue 页面职责收敛 v0 | 已通过 v1 snapshot 收敛 Project、Milestone、Issue、View 详情页职责，继续只读 |

## 验收标准

1. 不启动执行、不修改 issue、不写入 run。
2. 能读取 Project Summary 和 Goal Loop Summary 并展示 counts。
3. 能显示当前 next action 和 recommended command，且明确不自动执行。
4. 能打开任意 issue 的 contract 详情。
5. 能展示最近 run 的 validation command 状态。
6. 能展示 evidence / review / review assistant 文本。
7. 能展示 SavedView filter。
8. 刷新只重新读取本地事实源。
9. 没有 `.agentflow/` 时显示初始化缺失状态，不自动初始化。
10. MVP 主导航只展示总览、团队、项目、任务。
11. GoalLoop Trace、Issue Lifecycle Trace、Project Update Timeline、Search、Metrics 等只读能力不作为 MVP 主入口继续扩张。
12. Project / Milestone / Issue / View 详情页只展示各自职责范围内的信息，且通过 v1 snapshot 派生，不新增写入 UI。

## 后续授权门

Desktop Workbench MVP v0 只读壳通过后，后续任何写操作都必须另建 issue contract。尤其是执行按钮、issue 创建、模型调用、远程 PR / Linear issue 和 `.agentflow/` 写入都不能从本壳自然延伸，必须重新定义边界。
