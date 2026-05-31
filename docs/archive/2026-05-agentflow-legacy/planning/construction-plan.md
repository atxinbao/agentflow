# Construction Plan / Local Draft

创建日期：2026-05-21
最近压缩：2026-05-22
执行者：Codex

## 定位

对应 AEP `2. Construction Plan / Linear Draft`。AgentFlow 当前只生成 local draft，不创建远程 issue，不授权执行。

## AEP 落点

| AEP Flow | AgentFlow |
| --- | --- |
| 0 New Project Initialization | startup docs + `.agentflow/` Flow 0 |
| 1 Human Planning | product / design / strategy |
| 2 Construction Plan | 本文档 |
| 3 Execution Contract | local issue contract |
| 5 Single Issue Execution | `agentflow run ISSUE-*` |
| 7 Stage Code Audit | evidence + review checklist |
| 8 Root Docs Refresh | latest summary + `verification.md` |

## 施工顺序

| Order | Work Package | Validation |
| --- | --- | --- |
| 1 | Goal Compiler | goal parse test |
| 2 | Local Project Store | fixture parse |
| 3 | Context Collector | repo map test |
| 4 | Planner | candidate issue test |
| 5 | Issue Contract Builder | JSON parse |
| 6 | Codex Runtime Adapter | dry-run fixture |
| 7 | Validation Runner | command fixture |
| 8 | Evidence Chain | evidence fixture |
| 9 | SQLite Index + Saved Views | rebuild / filter test |
| 10 | Project Update Summary | source trace test |
| 11 | Review / PR Assistant | snapshot test |
| 12 | Desktop Workbench Boundary | boundary doc / no shell code |
| 13 | Desktop Workbench Read-only Shell | screenshot / read-only smoke |
| 14 | Local Pro Experiments Boundary | boundary doc / no feature implementation |
| 15 | Local Metrics Snapshot | read-only metrics fixture |
| 16 | Local Search Boundary | boundary doc / no search implementation |
| 17 | Local Search Reader | read-only search fixture / no index |
| 18 | Saved Query Boundary | boundary doc / no query file write |
| 19 | Desktop Search Boundary | boundary doc / no Desktop search UI implementation |
| 20 | Desktop Search Read-only View | read-only UI / no execution / no query write |
| 21 | Saved Query Writer Boundary | boundary doc / no query file write |
| 22 | Local Workspace / Team / Project Model Boundary | boundary doc / no workspace files |
| 23 | Local Project Model | read-only model snapshot / no issue migration |
| 24 | Desktop Project View | read-only project model UI / no seed write |
| 25 | Local Project Seed Boundary | seed write contract / no file write |
| 26 | Local Project Seed | confirmed seed writer / user confirmation |
| 27 | Issue Project Link Writer | explicit issue project link preview / writer |
| 28 | Project-aware Goal Loop | active project next issue selection |
| 29 | Desktop GoalLoop Trace | read-only decision source UI |
| 30 | Desktop Issue Lifecycle Trace | read-only issue lifecycle UI |
| 31 | Desktop Project Update Timeline | read-only project update timeline UI |
| 32 | Desktop MVP Navigation Scope Reduction | reduce desktop main nav to overview/team/project/task |
| 33 | Desktop Team Hierarchy | read-only workspace/team/project/task hierarchy |
| 34 | Desktop Team Parent Child Columns | explicit parent/team and child/project-task columns |
| 35 | Desktop Workspace Sidebar Tree | workspace/team tree nav with project/issues children |
| 36 | Desktop MVP Task Detail | next task detail convergence slice |
| 37 | AgentFlow AI Delivery Workflow Contract v1 | controlled delivery contract |
| 38 | Workflow State Machine | local state guard |
| 39 | Workflow Control Core | eligibility / lease / run gate |
| 40 | Project Audit / Docs Refresh Boundary | closure gate boundary |
| 41 | Project Closure State | project closure state guard |
| 42 | Project Code Audit Snapshot | read-only audit input package |

## 第一候选 Issue Draft

Title: `Goal Compiler + Core/CLI Bootstrap v0`

Scope:

- 建立 Rust workspace。
- 创建 `crates/agentflow-core` 和 `crates/agentflow-cli`。
- 实现 `agentflow init --from-goal GOAL.md`。
- 生成 `.agentflow/goal.md` 和 `.agentflow/goal.json`。
- 定义 Flow 0 输出文件数据结构。
- 实现离线模板版 `agentflow plan`。

Non-Goals:

- 不启动桌面端。
- 不接真实模型 API。
- 不实现 `agentflow run`。
- 不引入 DuckDB。
- 不创建 PR 或远程 issue。

Validation:

- `cargo fmt --check`
- `cargo test`
- `agentflow init --from-goal GOAL.md`
- `agentflow plan "..."`
- `git diff --check`

状态：`implemented / local validation pass`

## Root Docs Refresh

每阶段只刷新必要入口：`README.md`、`GOAL.md`、`ROADMAP.md`、`construction-plan.md`、latest summary、`verification.md`。

## Desktop Workbench Boundary

Desktop Workbench MVP v0 已先定义边界，再创建工程骨架。边界阶段只允许文档和本地 evidence 变更，不启动 Tauri / React 代码。

核心规则：

- 只读 `.agentflow/` 本地事实源。
- 不创建或编辑 issue contract。
- 不执行 run / verify / review。
- 不调用模型或远程 API。
- 不写入 `.agentflow/` 执行事实。
- 不做完整 PM 看板。

状态：`implemented / boundary established`

## Local Pro Experiments Boundary

Local Pro Experiments v0 只定义本地高级能力授权门，不实现功能。

候选实验：

- Local analytics / metrics。
- DuckDB 后置分析。
- Local project intelligence。
- Local search / saved query。
- Multi-project workspace。
- Desktop Workbench 后续交互能力。

已完成小切片：

- `Local Metrics Snapshot v0 只读实现`，状态 `implemented / read-only`。
- `Local Search v0 边界定义`，状态 `implemented / boundary-only`。
- `Local Search Reader v0 只读实现`，状态 `implemented / read-only`。
- `Saved Query v0 边界定义`，状态 `implemented / boundary-only`。
- `Desktop Search Read-only View v0 边界定义`，状态 `implemented / boundary-only`。
- `Desktop Search Read-only View v0 实现`，状态 `implemented / read-only`。
- `Saved Query Writer v0 边界定义`，状态 `implemented / boundary-only`。
- `Local Workspace / Team / Project Model v0 边界定义`，状态 `implemented / boundary-only`。
- `Local Project Model v0 只读实现`，状态 `implemented / read-only`。
- `Desktop Project View v0 只读实现`，状态 `implemented / read-only`。
- `Local Project Seed v0 边界定义`，状态 `implemented / boundary-only`。
- `Local Project Seed v0 实现`，状态 `implemented / explicit-confirmation-writer`。
- `Issue Project Link v0 边界定义`，状态 `implemented / boundary-only`。
- `Issue Project Link Writer v0 实现`，状态 `implemented / explicit-confirmation-writer`。
- `Project-aware GoalLoop v0 边界定义`，状态 `implemented / boundary-only`。
- `Project-aware GoalLoop v0 实现`，状态 `implemented / local-decision-only`。
- `Desktop GoalLoop Trace v0 只读展示`，状态 `implemented / read-only`。
- `Desktop Issue Lifecycle Trace v0 只读展示`，状态 `implemented / read-only`。
- `Desktop Project Update Timeline v0 只读展示`，状态 `implemented / read-only`。
- `Desktop MVP Navigation Scope Reduction v0`，状态 `implemented / mvp-scope-reduction`。
- `Desktop Team Hierarchy v0 收敛`，状态 `implemented / read-only hierarchy`。
- `Desktop Team Parent Child Columns v0`，状态 `implemented / parent-child columns`。
- `Desktop Workspace Sidebar Tree v0`，状态 `implemented / workspace tree nav`。
- `MVP Productization Project v0`，状态 `implemented / seed-backed project milestones`。
- `Milestone-aware Issue Planning v0`，状态 `implemented / active milestone issue writer`。
- `MVP Execution Loop v0`，状态 `implemented / milestone execution trace`。
- `AgentFlow AI Delivery Workflow Contract v1`，状态 `implemented / contract`。
- `Workflow State Machine v0`，状态 `implemented / local-state-check`。
- `Workflow Control Core v0`，状态 `implemented / local-control-core`。
- `Project Audit / Docs Refresh v0 边界定义`，状态 `implemented / boundary-only`。
- `Project Closure State v0 实现`，状态 `implemented / local-closure-state`。
- `Project Code Audit Snapshot v0 只读实现`，状态 `implemented / read-only-audit-snapshot`。

`Root Docs Refresh Snapshot v0` 已完成只读 docs refresh input package：`agentflow project docs-refresh` 生成 `.agentflow/state/project-docs-refresh.json` 和 `.agentflow/updates/PROJECT-DOCS-REFRESH-SUMMARY.md`。Snapshot 只汇总 checked docs、missing docs、updated-needed docs、required updates 和 blockers，不创建 `.agentflow/audits/`，不修改文档，不调用模型，不标记 Project done。

下一候选：`Product Feature Creation Flow v0`。

状态：`implemented / mvp projectization active`

## MVP Productization Project

MVP Productization Project v0 已参考 Linear 的 workspace / team / project / milestone / issue / view 最小关系，把 AgentFlow 当前工作从松散功能切片升级为本地项目计划。

已落地事实源：

- `.agentflow/workspace.json`
- `.agentflow/teams/core.json`
- `.agentflow/projects/agentflow-local-execution.json`

当前 project milestones：

| Milestone | Status | Issue |
| --- | --- | --- |
| `mvp-project-foundation` | completed | `ISSUE-0037` |
| `mvp-issue-planning` | completed | `ISSUE-0038` |
| `mvp-execution-loop` | completed | `ISSUE-0039` |
| `mvp-desktop-polish` | completed | `ISSUE-0040` |
| `mvp-release-readiness` | completed | `ISSUE-0041` |
| `workflow-core-state-machine` | completed | `ISSUE-0042` |
| `workflow-core-eligibility-engine` | completed | Workflow Control Core v0 goal |
| `workflow-core-closure-gates` | active | `agentflow project closure` / `agentflow project code-audit` |
| `goal-criteria-driven-mvp` | active | current MVP completion criteria |

`read_local_project_model_snapshot` 已支持 seed 优先读取；`agentflow projects` 现在展示 active milestone、milestone issue 归属和执行链路。`Milestone-aware Issue Planning v0` 已完成，`agentflow plan` 会自动把新 issue 关联到 active team / project / milestone，并同步更新 seed `issueIds`。`MVP Execution Loop v0`、`Desktop MVP Productization v0` 和 `AgentFlow AI Delivery Workflow Contract v1` 已完成，`ISSUE-0041` 生成 `mvp-release-readiness` milestone evidence summary。`Workflow State Machine v0` 已提供 `agentflow state check`。`Workflow Control Core v0` 已提供 eligibility / lease / run gate / evidence-based done 本地闭环。`Project Audit / Docs Refresh v0` 已定义 closure gate。`Project Closure State v0` 已提供 `agentflow project closure`。`Project Code Audit Snapshot v0` 已提供 `agentflow project code-audit`，当前 code audit gate 为 `snapshot-ready`，但 Project 仍不能 done。

Goal + Criteria Driven MVP 已把当前完成标准收敛为 Team / Project / Milestone / Issue 的本地创建和展示。Agent 自动执行链路保留为后续执行层能力，不再是当前 MVP 主目标。下一切片是 `Project / Issue Status Model v0`：Project canonical status 为 `draft / active / paused / completed / canceled`，Issue canonical status 为 `backlog / todo / in_progress / in_review / done / canceled`，Milestone 不维护独立状态。

`AgentFlow AI Delivery Workflow Contract v1` 把 MVP 后续方向从“Linear issue runner”收敛为“AI coding agent 受控交付系统”：

```text
Human controls scope
Milestone controls phase
Issue controls execution
Eligibility controls when
Lease controls who
PR / checks / evidence controls Done
Audit / docs controls closure
```

## Workflow State Machine

Workflow State Machine v0 已实现本地状态检查入口：

```bash
agentflow state check
```

输出：

- `.agentflow/state/workflow-state.json`
- `.agentflow/updates/WORKFLOW-STATE-SUMMARY.md`

它检查 Project / Milestone / Issue 状态、active milestone 唯一性、issue contract 完整性、projectLink 归属、completed issue evidence / review 和 transition guard。该切片不执行命令、不申请 lease、不调用模型、不创建远程 issue / PR、不新增 Desktop UI。

## Workflow Control Core

Workflow Control Core v0 已实现本地受控闭环：

```text
state check
-> eligibility
-> lease
-> run
-> verify
-> review
-> evidence
-> milestone summary
-> next milestone
```

新增 CLI：

- `agentflow eligibility`
- `agentflow eligibility ISSUE-XXXX`
- `agentflow lease`

新增事实源：

- `.agentflow/state/eligibility.json`
- `.agentflow/state/leases.json`
- `.agentflow/updates/ELIGIBILITY-SUMMARY.md`
- `.agentflow/updates/LEASE-SUMMARY.md`
- `.agentflow/leases/LEASE-*.json`

`run` 必须先通过 eligibility 并 acquire lease；`review` 完成 evidence / review / project update 后 release lease。当前不新增 Desktop UI、不接入远程 PR / GitHub / Linear、不调用模型。

## Project Audit / Docs Refresh Boundary

Project Audit / Docs Refresh v0 已定义 project closure gate：

```text
active
-> audit
-> docs-refresh
-> final-review
-> done
```

边界锁定：

- Code Audit 检查 duplicate code、temporary code、unused code、TODO / FIXME、security / auth / permission risk、performance risk、architecture drift、test gaps 和 unexpected public API changes。
- Root Docs Refresh 检查 README、ROADMAP、MVP Spec、architecture docs、contracts、validation docs、runbook / known limitations。
- Final Evidence Summary 必须汇总 project goal、completed milestones、completed issues、runs / validations、evidence / reviews、known gaps、deferred work 和 final recommendation。
- Human Final Approval 缺失时，Project 不能进入 done。

Project Closure State v0 已实现本地状态守卫：

```text
agentflow project closure
-> .agentflow/state/project-closure.json
-> .agentflow/updates/PROJECT-CLOSURE-SUMMARY.md
```

该命令不实现自动审计器、不修改 Desktop UI、不创建 `.agentflow/audits/`、不接入远程 PR / GitHub / Linear。

Project Code Audit Snapshot v0 已实现只读 audit input package：

```text
agentflow project code-audit
-> .agentflow/state/project-code-audit.json
-> .agentflow/updates/PROJECT-CODE-AUDIT-SUMMARY.md
```

该命令只汇总 audit candidates 和 blockers，不创建 `.agentflow/audits/`，不自动修复，不修改代码或文档，不标记 Project done。

Root Docs Refresh Snapshot v0 已实现只读 docs refresh input package：

```text
agentflow project docs-refresh
-> .agentflow/state/project-docs-refresh.json
-> .agentflow/updates/PROJECT-DOCS-REFRESH-SUMMARY.md
```

该命令只汇总 checked docs、missing docs、updated-needed docs、required updates 和 blockers，不创建 `.agentflow/audits/`，不修改文档，不调用模型，不标记 Project done。

下一候选：`Product Feature Creation Flow v0`。

## Local Search Boundary

Local Search v0 只定义本地搜索和 saved query 边界，不实现搜索索引、不新增 Desktop 搜索 UI、不写查询文件。

Reader 只能只读扫描 `docs/specs/local-search-boundary.md` 授权的 `.agentflow/` JSON / JSONL / Markdown 路径，并返回可追溯结果。saved query、FTS、Desktop search view 和模型语义搜索都必须另建 IssueContract。

状态：`implemented / read-only`

## Saved Query Boundary

Saved Query v0 只定义 saved query / saved search 的产品、文件格式候选、授权门、用户确认点、验证方式和 evidence 要求。

核心规则：

- 不创建 `.agentflow/queries`。
- 不写 saved query JSON 文件。
- 不保存搜索结果。
- 不新增 CLI 或 Desktop 搜索 UI。
- Saved Query 与 `.agentflow/views/*.json` 的 SavedView 分离，避免混用 issue/run filter 和全文搜索 query。
- 后续写入 `.agentflow/queries/*.json` 必须另建 IssueContract，并需要用户确认点。

状态：`implemented / boundary-only`

## Local Project Model

Local Project Model v0 只读实现已提供 `LocalProjectModelSnapshot` 和 `agentflow projects`。

核心规则：

- 只从现有 `.agentflow/` 事实源派生默认 workspace、core team、active local project、current milestone 和 issue refs。
- 不创建 `.agentflow/workspace.json`。
- 不创建 `.agentflow/teams/`。
- 不创建 `.agentflow/projects/`。
- 不迁移或改写既有 issue schema。
- 不实现 Project-aware GoalLoop。
- 为后续 Local Project Seed 和 Project-aware GoalLoop 提供只读输入。

状态：`implemented / read-only`

## Local Project Seed Boundary

Local Project Seed v0 Boundary 已定义从只读 `LocalProjectModelSnapshot` 进入本地 workspace/team/project seed 文件的写入合同。

核心规则：

- 默认写入路径仅限 `.agentflow/workspace.json`、`.agentflow/teams/{team-id}.json`、`.agentflow/projects/{project-id}.json`。
- Seed source 只能来自 `LocalProjectModelSnapshot` 和现有 `.agentflow/` 事实源。
- 创建目录、创建文件、覆盖文件、修改 default team / active project、链接既有 issue 前都必须用户确认。
- 当前 seed 实现阶段已新增 `agentflow project-seed` preview / writer。
- 默认 `agentflow project-seed` 不创建 seed 文件。
- 只有显式 `agentflow project-seed --write --yes` 才创建 workspace/team/project seed。
- v0 不覆盖已有 seed，不迁移 issue，不实现 Project-aware GoalLoop。

状态：`implemented / explicit-confirmation-writer`

## Issue Project Link Boundary

Issue Project Link v0 Boundary 已定义 IssueContract 后置关联 LocalTeam / LocalProject / Milestone 的字段和迁移门。

边界目标：

- 候选存储形态是 `IssueContract.projectLink`，包含 `teamId`、`projectId`、`milestoneId`、`linkSource`。
- 当前只定义边界，不迁移现有 issue，不改变 GoalLoop。
- 后续 writer 必须先 preview，再由用户确认写入。
- Project link 不是执行授权，不能绕过 IssueContract。

状态：`implemented / boundary-only`

## Issue Project Link Writer

Issue Project Link Writer v0 已实现 `agentflow issue-link ISSUE-XXXX` preview / writer。

边界目标：

- 默认 preview，不迁移历史 issue。
- 只允许在明确确认后给指定 issue 写 `projectLink`。
- writer 只写 `.agentflow/issues/{issue-id}.json` 和 `.agentflow/issues/{issue-id}.md`。
- 已有 `projectLink` 时拒绝覆盖。
- 不实现 Project-aware GoalLoop。
- 不新增 Desktop 写入口。

状态：`implemented / explicit-confirmation-writer`

## Project-aware GoalLoop Boundary

Project-aware GoalLoop v0 Boundary 已定义 active project / active milestone candidate 如何进入 GoalLoop 推荐链。

边界目标：

- 定义 GoalLoop 何时读取 active project / milestone 的候选 backlog。
- 保持 goal readiness、active issue、未完成 issue 的优先级不变。
- 只推荐下一条 issue intent，不执行 plan / run / verify / review。
- 不绕过 IssueContract，不批量改写历史 issue。

状态：`implemented / boundary-only`

## Project-aware GoalLoop Implementation

Project-aware GoalLoop v0 Implementation 已在 `goal_loop_decision` 中加入 active project / active milestone candidate fallback。

边界目标：

- 在 `goal_loop_decision` 中加入 active project / active milestone candidate fallback。
- 只有 goal ready、无 active issue、无 incomplete issue 时才读取 project candidate。
- 缺失 workspace/project seed、issue link 或 milestone candidate 时回退 roadmap candidate。
- 仍只写 `goal-loop.json` 和 `GOAL-LOOP-SUMMARY.md`，不执行 recommended command。

状态：`implemented / local-decision-only`

## Desktop GoalLoop Trace

下一候选是 `Desktop GoalLoop Trace v0 只读展示`。

边界目标：

- Desktop 只读展示 GoalLoop 决策来源和 fallback trace。
- 展示 active issue / incomplete issue / project candidate / roadmap fallback 的当前命中原因。
- recommended command 只显示，不执行。
- 不写 `.agentflow/`，不创建 issue，不迁移 projectLink。

状态：`draft / not authorized`

## Desktop Project View

Desktop Project View v0 已在 Desktop Workbench 中展示 Local Project Model 只读 snapshot。

核心规则：

- 通过 Tauri `load_project_model_snapshot` 调用 Rust core `read_local_project_model_snapshot`。
- 展示 workspace、teams、active project、milestones、issue refs、GoalLoopSelection、source trace 和 recommended command 文本。
- 不创建 `.agentflow/workspace.json`。
- 不创建 `.agentflow/teams/` 或 `.agentflow/projects/`。
- 不执行 recommended command，不替代 IssueContract。

状态：`implemented / read-only`

## Desktop Search Read-only View

Desktop Search Read-only View v0 已在 Desktop Workbench 中实现只读搜索视图。

核心规则：

- 通过 Tauri `load_search_snapshot` 调用 Rust core `read_local_search_snapshot`。
- 展示 query 输入框、result list、source trace、empty / loading / error 状态、read-only badge 和 recommended command 文本。
- 不写 `.agentflow/search`。
- 不写 `.agentflow/queries`。
- 不保存搜索结果。
- 不创建 issue、不执行 run / verify / review、不调用模型、不上传远程。

状态：`implemented / read-only`

## Desktop Search Boundary

Desktop Search Read-only View v0 边界定义已完成，Desktop Search Read-only View v0 实现已在该边界内完成。

核心规则：

- 后续 UI 只能调用 Local Search Reader 的只读能力。
- UI 必须覆盖 query 输入框、result list、source trace、empty / loading / error 状态、read-only badge 和 recommended command 只展示。
- 不写 `.agentflow/search`。
- 不写 `.agentflow/queries`。
- 不保存搜索结果。
- 不执行 run / verify / review。
- 不创建 issue、不调用模型、不上传远程。

状态：`implemented / boundary-only`

## Saved Query Writer Boundary

Saved Query Writer v0 边界定义只定义 `.agentflow/queries/*.json` query definition 的写入合同，不实现 writer。

核心规则：

- 不创建 `.agentflow/queries`。
- 不写 saved query JSON 文件。
- 不保存搜索结果，不创建 `.agentflow/search`、索引或 cache。
- 后续 Writer 实现必须先有 IssueContract，并在创建目录、创建 query、覆盖或删除 query 前设置用户确认点。
- Writer 只能保存 query definition；`resultPersistence` 必须为 `none`。
- Desktop Search 输入框不能自动触发 Writer。

状态：`implemented / boundary-only`

## Local Workspace / Team / Project Model Boundary

Local Workspace / Team / Project Model v0 边界定义锁定本地最小组织关系：

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

核心规则：

- 当前阶段不创建 `.agentflow/workspace.json`。
- 当前阶段不创建 `.agentflow/teams/`。
- 当前阶段不创建 `.agentflow/projects/`。
- 不迁移现有 issue。
- IssueContract 仍是唯一执行授权。
- GoalLoop 后续可以从 active project backlog 推荐下一条 issue intent，但不能执行。
- LocalTeam 只表示 workflow / validation preset / ownership 分组，不表示账号、权限或远程协作。

状态：`implemented / boundary-only`
