# MVP Spec

创建日期：2026-05-21
最近压缩：2026-05-22
执行者：Codex

## 闭环

```text
/goal -> init -> context -> plan -> issue -> run -> verify -> evidence -> review -> update
```

AEP Goal Initialization Protocol v0 在 `init` 与 `plan` 之间增加第一阶段机械门禁：

```text
/goal -> ProjectGoal -> ProjectDefinition -> ScopeState -> GoalReadiness -> IssueContract
```

Goal Loop Orchestrator v0 在 `update` 之后增加本地下一步决策：

```text
ProjectUpdate -> GoalLoopState -> plan / run / verify / review / update / wait-human
```

Local Workspace / Team / Project Model v0 锁定本地组织层：

```text
LocalWorkspace
  -> LocalTeams
      -> IssueContracts
  -> LocalProjects
      -> Milestones
      -> IssueContracts
  -> GoalLoopSelection
```

边界文档见 `docs/specs/local-workspace-project-model-boundary.md`。该阶段先锁定关系和 schema 候选，不创建 `.agentflow/workspace.json`、`.agentflow/teams/` 或 `.agentflow/projects/`，不改变 IssueContract 的唯一执行授权。

Local Project Model v0 在边界之后提供只读派生模型：

```text
.agentflow/ facts -> LocalProjectModelSnapshot -> agentflow projects
```

该 reader 从现有 goal、settings、scope-state、goal-loop、roadmap、issues、runs、evidence、reviews 和 updates 派生默认 workspace、core team、active local project、current milestone 和 issue refs，不创建组织模型事实源文件。

Desktop Project View v0 复用同一只读 read model：

```text
LocalProjectModelSnapshot -> Tauri load_project_model_snapshot -> React Project read-only view
```

该 surface 只展示 workspace、teams、active project、milestones、issue refs、GoalLoopSelection 和 source trace，不创建 project seed，不执行 recommended command。

Desktop Workspace Overview v0 把同一层级入口放到总览：

```text
LocalProjectModelSnapshot -> React Overview Workspace Entry
  -> Workspace Projects
  -> Workspace Teams
      -> Team Issues
      -> Team Projects
```

总览只显示 workspace / team / project 摘要，不增加写入入口，不把 Team 当成账号或远程协作边界。

Local Project Seed v0 Boundary 定义后续 seed writer 的写入合同：

```text
LocalProjectModelSnapshot
  -> LocalProjectSeed preview
  -> user confirmation gates
  -> .agentflow/workspace.json
  -> .agentflow/teams/{team-id}.json
  -> .agentflow/projects/{project-id}.json
```

边界文档见 `docs/specs/local-project-seed-boundary.md`。`Local Project Seed v0 实现` 已新增 `LocalProjectSeedPreview`、`agentflow project-seed` preview 和显式 `--write --yes` writer。默认 preview 不创建 seed 文件；writer 不覆盖已有 seed、不迁移 issue、不实现 Project-aware GoalLoop。

Issue Project Link v0 Boundary 定义 IssueContract 后置归属字段：

```text
IssueContract
  -> projectLink
      -> teamId
      -> projectId
      -> milestoneId
      -> linkSource
```

边界文档见 `docs/specs/issue-project-link-boundary.md`。`Issue Project Link Writer v0 实现` 已新增 `IssueProjectLink`、`IssueProjectLinkPreview`、`agentflow issue-link ISSUE-XXXX` preview 和显式 `--write --yes` writer。默认 preview 不写 `.agentflow/`；writer 只写指定 issue 的 `.json` / `.md`，拒绝覆盖已有 `projectLink`，不批量迁移历史 issue，不改写 GoalLoop。

Project-aware GoalLoop v0 Boundary 定义后续 project candidate 如何进入 GoalLoop：

```text
LocalWorkspace.activeProjectId
  -> LocalProject.activeMilestoneId
      -> Milestone.nextIssueIntent
          -> GoalLoopSelection
```

边界文档见 `docs/specs/project-aware-goalloop-boundary.md`。Project-aware GoalLoop v0 已按该边界实现，并在 MVP Minimal Workflow v0 中收紧为：readiness -> active issue -> active milestone queue preflight -> incomplete fallback -> project candidate -> roadmap fallback -> wait-human。实现只写 `goal-loop.json` / `GOAL-LOOP-SUMMARY.md`，不执行 recommended command，不迁移历史 issue。

MVP Productization Project v0 把本地组织模型从派生 read model 推进为 seed 事实源：

```text
.agentflow/workspace.json
  -> activeProjectId
.agentflow/teams/core.json
  -> issueIds
.agentflow/projects/agentflow-local-execution.json
  -> activeMilestoneId
  -> milestones[]
      -> issueIds
      -> nextIssueIntent
```

`read_local_project_model_snapshot` 现在优先读取 workspace / team / project seed；seed 缺失时才回退派生模型。`IssueProjectLink` 也优先绑定 active milestone，而不是固定到第一个 milestone。

当前 MVP milestones：

| Milestone | Status | Purpose |
| --- | --- | --- |
| `mvp-project-foundation` | completed | project seed / milestones 成为可读取事实源 |
| `mvp-issue-planning` | completed | 基于 active milestone 创建和归属 IssueContract |
| `mvp-execution-loop` | completed | WIP=1 执行、验证、证据、审查、更新 |
| `mvp-desktop-polish` | completed | 桌面 MVP 的总览、团队、项目、任务、视图打磨 |
| `mvp-release-readiness` | completed | 安装、启动、验证和演示路径 |

当前 issue 链路：

```text
mvp-project-foundation -> ISSUE-0037 -> completed
mvp-issue-planning -> ISSUE-0038 -> completed
mvp-execution-loop -> ISSUE-0039 -> completed
mvp-desktop-polish -> ISSUE-0040 -> completed
mvp-release-readiness -> ISSUE-0041 -> completed
root roadmap -> Workflow State Machine v0 边界定义 -> next -> plan
```

Milestone-aware Issue Planning v0 已完成：`agentflow plan "<intent>"` 在存在 workspace / team / project seed 时，会把新 IssueContract 自动写入 active team / active project / active milestone 的 `issueIds`，并在 issue JSON / Markdown 中写入 `projectLink`。当 plan 消费了 active milestone 的 `nextIssueIntent` 时，会清空该 candidate，避免完成后重复推荐同一个 issue intent。

MVP Execution Loop v0 把执行链路收敛为只读可追溯模型：

```text
Project -> Milestone -> Issue -> ExecutionRun -> VerificationEvidence -> ReviewEvidence -> ProjectUpdate
```

`LocalProjectIssueRef` 派生 latest run、run status、validation status、execution state、evidence path、review path 和 project update path。Desktop Project 视图和 `agentflow projects` 都只展示这些本地事实，不执行命令、不创建远程 PR、不调用模型。

MVP Minimal Workflow v0 进一步收紧执行调度：

```text
Project
  -> Milestone
      -> IssueContract
          -> ExecutionRun
          -> VerificationEvidence
          -> ReviewEvidence
          -> ProjectUpdate
```

`agentflow goal next` 必须先做 active milestone queue preflight：当 active milestone 已有 issue 队列时，只推进该 milestone 下唯一 eligible issue；如果同一 milestone 下存在多个未完成 issue，返回 `wait-human`，要求先收敛 WIP=1。全局未完成 issue 不再越过 active milestone 直接抢占当前阶段。

`agentflow review ISSUE-XXXX` 在 issue 验证通过后继续写 evidence / review / project update；如果该 issue 所属 milestone 的全部 issue 都已 completed，则自动写 `.agentflow/evidence/MILESTONE-<milestone-id>-evidence-summary.md`，并把 `.agentflow/projects/<project-id>.json` 中当前 milestone 标记为 completed，再激活下一个 planned milestone。PR / checks / merge 在当前 MVP 中仍是后置 artifact，不从 Desktop 或 CLI 自动创建远程对象。

AgentFlow AI Delivery Workflow Contract v1 将 MVP 主链路提升为受控交付合同：

```text
Workspace / Team
  -> Project
      -> Milestone
          -> Issue
              -> Lease
              -> Execution Run
              -> PR / Checks
              -> Evidence
      -> Milestone Review
  -> Project Audit
  -> Root Docs Refresh
```

正式合同文档见 `docs/contracts/agentflow-ai-delivery-workflow-contract-v1.md`，实现侧引用入口见 `docs/specs/agentflow-ai-delivery-workflow-contract-v1.md`。v1 定义五个能力面：Workflow State Machine、Eligibility Engine、Lease / Lock、Execution Evidence、Milestone / Project Closure。

Workflow State Machine v0 已完成第一条可执行纵切：

```text
.agentflow/projects/*.json
  -> .agentflow/issues/*.json
  -> agentflow state check
  -> .agentflow/state/workflow-state.json
  -> .agentflow/updates/WORKFLOW-STATE-SUMMARY.md
```

`agentflow state check` 校验 Project / Milestone / Issue 状态不变量、active milestone 唯一性、issue contract 完整性、projectLink 归属、completed issue evidence / review 归档和固定 transition guards。它只写 workflow state 检查产物，不执行 run / verify / review，不调用模型，不创建远程 issue / PR，不改写 `goal_loop_decision`，也不新增 Desktop UI。规格见 `docs/specs/workflow-state-machine-v0.md`。

Workflow Control Core v0 已继续把本地控制链路补齐：

```text
agentflow state check
  -> agentflow eligibility
  -> agentflow run ISSUE-XXXX --dry-run
  -> local lease acquired
  -> agentflow verify ISSUE-XXXX
  -> agentflow review ISSUE-XXXX
  -> evidence / review / project update
  -> lease released
  -> milestone summary
```

新增 `agentflow eligibility` 和 `agentflow lease`。`eligibility` 写 `.agentflow/state/eligibility.json` 与 `.agentflow/updates/ELIGIBILITY-SUMMARY.md`；`lease` 写 `.agentflow/state/leases.json` 与 `.agentflow/updates/LEASE-SUMMARY.md`。`run` 现在必须通过 eligibility，并在创建 `RUN-XXXX` 前 acquire `.agentflow/leases/LEASE-*.json`；`AgentRun` 记录 `projectId`、`milestoneId` 和 `leaseId`。`review` 完成 evidence / review / project update 后释放 lease。规格见 `docs/specs/workflow-control-core-v0.md`。

Project Audit / Docs Refresh v0 已定义 Project closure gate：

```text
active
  -> audit
  -> docs-refresh
  -> final-review
  -> done
```

边界文档见 `docs/specs/project-audit-docs-refresh-boundary.md`。该阶段只定义 Code Audit、Root Docs Refresh、Final Evidence Summary 和 Human Final Approval 的输入、输出和 gate；不实现自动审计器，不修改 Desktop UI，不创建 `.agentflow/audits/`，不接入远程 PR / GitHub / Linear。下一实现切片是 `Project Closure State v0 实现`，用于把 project 不能直接 done 的状态守卫落到本地 workflow core。

Project Closure State v0 已实现 closure gate 的本地只读检查：

```text
agentflow project closure
  -> .agentflow/state/project-closure.json
  -> .agentflow/updates/PROJECT-CLOSURE-SUMMARY.md
```

该命令读取 project、milestones、issues、runs、evidence、reviews 和 updates，判断当前 Project 是 `active`、`audit-ready`、`audit`、`docs-refresh`、`final-review`、`done-blocked` 或 `done`。当前项目输出 `audit-ready`，但 `can_mark_done=false`；缺失 code audit、docs refresh、final evidence summary 和 human final approval 时，Project 不能 done。`goal next` 在 closure 阶段推荐 `agentflow project closure`，不自动执行 audit / docs refresh / final approval。

Project Code Audit Snapshot v0 已实现只读 audit input package：

```text
agentflow project code-audit
  -> .agentflow/state/project-code-audit.json
  -> .agentflow/updates/PROJECT-CODE-AUDIT-SUMMARY.md
```

该命令读取 `.agentflow/state/project-closure.json`、project / milestone / issue / run / evidence / review / update 事实源和源码树，只汇总 duplicate code、temporary code、unused / dead code、TODO / FIXME、security / auth / permission risk、performance risk、architecture drift、test gap 和 unexpected public API change 候选项。它不创建 `.agentflow/audits/`，不修改代码，不修改文档，不调用模型，不标记 Project done。snapshot 存在后，`agentflow project closure` 会把 code audit gate 显示为 `snapshot-ready`；该状态仍不是 final Code Audit passed。

Root Docs Refresh Snapshot v0 已实现只读 docs refresh input package：

```text
agentflow project docs-refresh
  -> .agentflow/state/project-docs-refresh.json
  -> .agentflow/updates/PROJECT-DOCS-REFRESH-SUMMARY.md
```

该命令读取 `.agentflow/state/project-closure.json`、`.agentflow/state/project-code-audit.json`、README、ROADMAP、MVP Spec、architecture docs、contracts、validation docs 和 `verification.md`，只汇总 checked docs、missing docs、updated-needed docs、required updates 和 blockers。它不创建 `.agentflow/audits/`，不修改文档，不调用模型，不标记 Project done。snapshot 存在后，`agentflow project closure` 会把 docs refresh gate 显示为 `snapshot-ready`；该状态仍不是 final Root Docs Refresh passed。

Product Feature Creation Flow v0 已实现第一个产品功能入口：

```text
agentflow feature create "<feature goal>"
  -> preview only
agentflow feature create "<feature goal>" --write --yes
  -> Project
  -> Project Charter / Milestone Plan / Issue Contracts / Validation Evidence milestones
  -> IssueContracts
  -> workspace.activeProjectId
  -> goal next / eligibility
```

该入口不调用模型自动拆解，也不复制 Linear。它只把 Human 输入的产品功能目标确定性落成本地 Project、Milestones 和 IssueContracts。每个新 issue 都包含 `projectLink`、`scope`、`nonGoals`、`validation`、`evidenceRequirements`、`rollbackPlan` 和 `riskLevel`。写入后第一条 issue 位于 active milestone `project-charter`，`agentflow goal next` 推荐 `agentflow run ISSUE-XXXX --dry-run`，`agentflow eligibility` 解释 ready / eligible 状态；执行仍必须经过 eligibility + lease，不从 Desktop 创建或执行。

Product Feature Execution Flow v0 已实现创建后的只读执行入口：

```text
agentflow feature status
  -> active Product Feature Project
  -> active milestone
  -> current issue
  -> eligibility / latest run / validation / evidence / review

agentflow feature next
  -> run / verify / review / wait-human recommendation
```

`feature status` 展示当前 feature project 的完整状态；`feature next` 只输出下一步决策。当前 `feature-0043` 下推荐 `agentflow run ISSUE-0043 --dry-run`。如果 issue 已 run，则推荐 verify；如果 verify 已通过，则推荐 review；review 完成后由既有 milestone summary writer 激活下一个 milestone。该入口不自动执行命令、不调用模型、不创建远程对象、不写 `.agentflow/audits/`。

Product Feature Controlled Run v0 已把第一条 feature issue 的 dry-run 做成受控执行入口：

```text
agentflow run ISSUE-0043 --dry-run
  -> active project / active milestone gate
  -> eligibility gate
  -> lease gate
  -> RUN-XXXX/run.json
  -> ControlledRunPlan
  -> feature next -> verify
```

`AgentRun.runPlan` 记录 goal、non-goals、expected files、blocked files / areas、planned steps、validation commands、evidence requirements 和 rollback plan。CLI 输出同一份 run plan，`feature status` 展示 `dry-run recorded`、latest run plan、expected files、blocked files、validation commands 和 evidence requirements。该阶段仍不修改源码、不调用模型、不创建远程 PR / GitHub issue / Linear issue、不从 Desktop 执行 run。

Goal + Criteria Driven MVP 已重新定义当前项目完成标准：

```text
Goal:
把 AgentFlow 做成一个本地优先、免费、闭源的 AI coding agent 受控交付工具。

MVP:
1. 用户可以在本地创建和管理 Team / Project / Milestone / Issue。
2. 用户和 Agent 可以共同把产品功能目标拆成 Project -> Milestones -> Issues，并保存为本地事实源。
```

当前 MVP 不把 Agent 自动执行流程作为主产品目标。`eligibility / lease / run / verify / review / evidence / milestone summary` 只作为后续执行层能力保留。

状态模型锁定为：

```text
Project:
draft | active | paused | completed | canceled

Issue:
backlog | todo | in_progress | in_review | done | canceled

Milestone:
no status
```

Milestone 完成度从其包含的 Issues 状态派生。Project 完成度从其包含的 Milestones / Issues 派生，但 Project 是否 `completed` 必须由用户或明确确认动作设置。完整 Criteria 见 `docs/specs/goal-criteria-driven-mvp.md`。

Project / Milestone / Issue / View Model v1 将当前 MVP 产品主干进一步收敛为：

```text
Workspace / Team
-> Project
   -> Milestone
      -> Issue
-> View
```

职责边界：

| 对象 | MVP 职责 | 边界 |
| --- | --- | --- |
| Project | 方向、边界、整体成功标准 | 不执行代码，不写完整实现细节 |
| Milestone | 阶段目标、阶段出口、阶段验收门 | 不直接执行代码 |
| Issue | 单个可执行合同 | 不跨 milestone 扩范围 |
| View | saved filter / sort / layout | 不承载业务状态，不改变层级 |

硬规则：

```text
Project 不执行
Milestone 不执行
Issue 执行
View 只展示
Queue Preflight 决定谁能执行
Evidence 决定是否 Done
```

本 v1 规格不要求立即破坏现有 `.agentflow/` 或 canonical status；它是后续 writer、queue preflight 和 Desktop 页面收敛的产品合同。当前已新增只读 `ProjectMilestoneIssueViewModelSnapshot` adapter，从现有 `LocalProjectModelSnapshot`、IssueContract 和 SavedView 派生 v1 schema，不改变现有写入行为。完整规格见 `docs/specs/project-milestone-issue-view-model-v1.md`。

当前 writer preview 已对齐 v1 产品模型：`agentflow team/project/milestone/issue create` 的 `CreationPreview.v1Contract` 会展示 Team relation、Project charter、Milestone gate 和 Issue execution contract。该对齐只增强 preview 和开发文档验收，不改变 `.agentflow/` 写入 schema，不执行 run / verify / review。

Desktop 已将该 v1 产品模型接入只读工作台：`load_project_milestone_issue_view_model_snapshot` 读取 `ProjectMilestoneIssueViewModelSnapshot`。Project 页面只展示 Project charter、milestones、issue progress、queue status 和 closure gate；Milestone 区块只展示 milestone goal、entry criteria、issues、exit criteria 和 derived progress；Issue 页面只展示 issue contract 的 goal、scope、non-goals、validation、evidence、boundary 和 status；View 页面只展示 saved filter / sort / layout，不承载业务状态。

Desktop GoalLoop Trace v0 已把该决策链路接入桌面只读壳：

```text
.agentflow/goal-loop.json
  -> DesktopWorkbenchSnapshot.goalLoop
  -> GoalLoop Trace read-only view
```

Trace 视图展示 readiness、active issue、incomplete issue、project candidate、roadmap fallback、next action、recommended intent 和 recommended command；它只解释本地决策来源，不执行命令、不创建 issue、不写 `.agentflow/`。

Desktop Issue Lifecycle Trace v0 已把单个 issue 的本地执行链路接入桌面只读壳：

```text
.agentflow/issues/*.json
  -> runs / validation
  -> evidence / review / project update
  -> Issue Lifecycle Trace read-only view
```

Lifecycle 视图展示 contract、run、validation、evidence、review、project update 和 completed 状态；它只解释 issue 当前卡在哪个步骤，不执行 run / verify / review，不创建或编辑 issue，不写 `.agentflow/`。

Desktop Project Update Timeline v0 已把项目更新链路接入桌面只读壳：

```text
.agentflow/updates/PROJECT-UPDATE-*.md
  -> issue / run / validation
  -> evidence / review
  -> Project Update Timeline read-only view
```

Timeline 视图按最新优先展示 PROJECT-UPDATE，并解释每条更新来自哪个 issue、run、validation、evidence 和 review；它只展示链接和文本，不执行命令、不创建或编辑 issue、不保存 timeline filter、不写 `.agentflow/`。

Desktop MVP Navigation Scope Reduction v0 将桌面主导航收敛为 MVP 最小产品骨架：

```text
总览
团队
项目
任务
```

总览展示 workspace 摘要、next action、推荐命令和 counts；团队展示 workspace/team 下的项目和任务关系；项目展示 LocalProject、Milestone 和 GoalLoopSelection；任务展示 IssueContract、latest run、validation commands 和证据链接。GoalLoop Trace、Issue Lifecycle Trace、Project Update Timeline、Search、Metrics 等能力保留为内部 trace/debug 或底层 reader，不作为 MVP 主入口。

MVP 只验证 CLI 闭环。Desktop、团队同步和商业化不能先于它。

Desktop Workbench MVP v0 在 CLI 闭环之后提供只读桌面壳：

```text
.agentflow/ -> DesktopWorkbenchSnapshot -> Tauri command -> React read-only workbench
```

实现边界见 `docs/specs/desktop-workbench-mvp-boundary.md`。Desktop 不写入事实源，不执行 run / verify / review，不调用模型，不创建远程 PR / Linear issue。

Local Pro Experiments v0 在 Desktop Workbench 之后只定义本地高级能力边界：

```text
DesktopWorkbenchSnapshot -> Local Pro candidate gates -> first read-only metrics slice
```

边界文档见 `docs/specs/local-pro-experiments-boundary.md`。Local Pro Boundary 不实现功能，只定义 analytics / metrics、DuckDB、project intelligence、search / saved query、多项目 workspace 和 Desktop interaction 的授权门。

Local Search v0 在 Local Metrics Snapshot v0 之后只定义本地搜索边界：

```text
.agentflow/ + DesktopWorkbenchSnapshot + LocalMetricsSnapshot -> Local Search Boundary -> Local Search Reader candidate
```

边界文档见 `docs/specs/local-search-boundary.md`。Local Search v0 不实现搜索引擎、不新增索引、不写 `.agentflow/search` 或 `.agentflow/queries`、不新增 Desktop 搜索 UI、不调用模型。

Local Search Reader v0 在边界定义之后提供 CLI 只读搜索：

```text
.agentflow/ allowed text facts -> LocalSearchSnapshot -> agentflow search "<query>"
```

Reader 只做 literal text query，不支持 regex、boolean grammar、语义搜索或远程搜索；不创建索引、不写 `.agentflow/search` / `.agentflow/queries`，不新增 Desktop 搜索 UI。

Saved Query v0 在 Local Search Reader v0 之后只定义边界：

```text
LocalSearchSnapshot + saved query schema candidate -> Saved Query Boundary -> Desktop Search boundary candidate
```

边界文档见 `docs/specs/saved-query-boundary.md`。Saved Query Boundary 不创建 `.agentflow/queries`，不写 query 文件，不保存搜索结果，不新增 CLI 或 Desktop 搜索 UI。

Desktop Search Read-only View v0 在 Saved Query v0 之后先定义 Desktop 搜索入口边界：

```text
LocalSearchSnapshot -> Desktop Search Boundary -> Desktop Search Read-only View implementation candidate
```

边界文档见 `docs/specs/desktop-search-readonly-boundary.md`。Desktop Search Boundary 不新增 UI 实现，不执行 run / verify / review，不创建 issue，不调用模型，不写 `.agentflow/search` 或 `.agentflow/queries`。

Desktop Search Read-only View v0 已实现：

```text
React Search View -> Tauri load_search_snapshot -> read_local_search_snapshot -> LocalSearchSnapshot
```

Desktop Search 只读视图展示 query 输入、result list、source trace、empty / loading / error、read-only badge 和 recommended command 文本；不保存 query、不保存结果、不执行命令。

Saved Query Writer v0 在 Desktop Search 只读视图之后只定义写入边界：

```text
SavedQueryDefinition schema candidate -> Saved Query Writer Boundary -> Saved Query Writer implementation candidate
```

边界文档见 `docs/specs/saved-query-writer-boundary.md`。Saved Query Writer Boundary 只定义 `.agentflow/queries/*.json` 写入合同、schema、路径边界、用户确认点、验证矩阵和 evidence 要求；当前不创建 `.agentflow/queries`，不写 query 文件，不实现 writer。

## `.agentflow/` v0

```text
.agentflow/
  goal.{md,json}
  project-definition.json
  scope-state.json
  environment.md
  architecture.md
  roadmap.md
  bootstrap/*.md
  settings.json
  index.json
  index.sqlite
  goal-loop.json
  workspace.json
  teams/*.json
  projects/*.json
  issues/ISSUE-0001.{md,json}
  runs/RUN-0001/{run.json,transcript.md,commands.jsonl,diff-summary.md}
  evidence/ISSUE-0001-evidence.md
  reviews/ISSUE-0001-review.md
  reviews/ISSUE-0001-assistant.md
  views/*.json
  updates/*.md
  updates/PROJECT-SUMMARY.md
  updates/GOAL-LOOP-SUMMARY.md
```

## 核心对象

### `goal.json`

```json
{
  "version": "0.0.1",
  "objective": "...",
  "successCriteria": ["..."],
  "nonGoals": ["..."],
  "constraints": ["local-first", "no-code-upload"],
  "firstCandidate": "Goal Compiler + Core/CLI Bootstrap v0"
}
```

### `settings.json`

```json
{
  "version": "0.0.1",
  "projectName": "example",
  "defaultModelProvider": "deepseek",
  "modelProviders": [{"id": "deepseek", "type": "openai-compatible", "apiKeyEnv": "DEEPSEEK_API_KEY"}],
  "validationCommands": ["npm test"],
  "dataPolicy": {"localFirst": true, "uploadCodeByDefault": false}
}
```

### `project-definition.json`

```json
{
  "version": "0.0.1",
  "sourceGoal": ".agentflow/goal.md",
  "phase": "AEP Stage 1 / Goal Initialization",
  "status": "initialized",
  "outputs": [{"id": "goal", "name": "GOAL.md copy", "path": "goal.md", "status": "initialized"}],
  "rules": ["IssueContract 是唯一执行授权。"]
}
```

### `scope-state.json`

```json
{
  "version": "0.0.1",
  "wipLimit": 1,
  "activeIssueId": null,
  "currentPhase": "goal-initialization",
  "executionAuthorized": false,
  "authorizationSource": "issue-contract",
  "boundaries": ["每次执行必须先绑定唯一 IssueContract。"]
}
```

### `IssueContract`

```json
{
  "id": "ISSUE-0001",
  "status": "planned",
  "intent": "...",
  "riskLevel": "medium",
  "scope": ["..."],
  "nonGoals": ["..."],
  "context": {"repo": ".", "files": []},
  "executionPlan": ["..."],
  "validation": {"commands": ["..."]},
  "evidenceRequirements": ["transcript", "command-output", "diff-summary", "known-limitations"],
  "rollbackPlan": ["回退本 issue 修改。"],
  "humanGate": {"beforeFileEdits": false, "beforeExternalNetwork": true},
  "aep": {
    "phase": "AEP Issue Execution",
    "stopCondition": "...",
    "fastestFeedbackLoop": ["cargo test"],
    "verticalSlice": "...",
    "tracerBulletPlan": ["..."],
    "diagnosePlan": ["..."],
    "graphifyContextStatus": "not-integrated-v0-local-context-only",
    "docsClaimTrace": [".agentflow/goal.json", ".agentflow/project-definition.json"],
    "boundaryConfirmation": ["IssueContract is the only execution input."],
    "prHandoffRequirements": ["Local review assistant only."]
  }
}
```

### `LocalWorkspace` 候选 / 只读派生

当前由 `LocalProjectModelSnapshot` 只读派生；seed 写入前不创建 `.agentflow/workspace.json`：

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

### `LocalTeam` 候选 / 只读派生

当前由 `LocalProjectModelSnapshot` 只读派生；seed 写入前不创建 `.agentflow/teams/`：

```json
{
  "version": "0.0.1",
  "id": "core",
  "name": "Core",
  "workflow": ["planned", "active", "completed"],
  "defaultValidationCommands": ["cargo fmt --check", "cargo test", "git diff --check"],
  "wipLimit": 1
}
```

### `LocalProject` 候选 / 只读派生

当前由 `LocalProjectModelSnapshot` 只读派生；seed 写入前不创建 `.agentflow/projects/`：

```json
{
  "version": "0.0.1",
  "id": "agentflow-local-execution",
  "name": "AgentFlow Local Execution",
  "status": "active",
  "teamIds": ["core"],
  "activeMilestoneId": "project-model",
  "milestones": [
    {"id": "project-model", "name": "Project Model", "status": "active", "issueIds": []}
  ],
  "issueIds": [],
  "nextIssueIntent": "Local Project Seed v0 边界定义"
}
```

### `LocalProjectModelSnapshot`

```json
{
  "version": "0.0.1",
  "initialized": true,
  "projectRoot": "/Users/mac/Documents/AgentFlow",
  "workspace": {"id": "default", "activeProjectId": "agentflow-local-execution"},
  "teams": [{"id": "core", "wipLimit": 1, "issueIds": ["ISSUE-0020"]}],
  "projects": [
    {
      "id": "agentflow-local-execution",
      "status": "active",
      "activeMilestoneId": "current-roadmap",
      "nextIssueIntent": "Local Project Seed v0 边界定义"
    }
  ],
  "goalLoopSelection": {"nextAction": "plan", "recommendedCommand": "agentflow plan \"Local Project Seed v0 边界定义\""},
  "boundary": {"readOnly": true}
}
```

### `run.json`

```json
{
  "id": "RUN-0001",
  "issueId": "ISSUE-0001",
  "status": "completed",
  "mode": "dry-run",
  "validationCommands": [{"command": "...", "exitCode": 0}],
  "outputs": {"commands": "commands.jsonl", "evidence": "../../evidence/ISSUE-0001-evidence.md"}
}
```

### `SavedView`

```json
{
  "version": "0.0.1",
  "id": "completed",
  "name": "completed",
  "filter": {
    "issueStatus": "completed",
    "runStatus": "completed",
    "validationStatus": "passed",
    "issueId": null
  }
}
```

`index.sqlite` 是可重建查询索引，事实源仍是 `.agentflow/` 下的 JSON / Markdown 文件。

### `GoalLoopState`

```json
{
  "version": "0.0.1",
  "goalReady": true,
  "activeIssueId": null,
  "incompleteIssues": [],
  "nextAction": "plan",
  "recommendedIssueIntent": "Desktop Workbench MVP v0 只读壳实现",
  "recommendedCommand": "agentflow plan \"Desktop Workbench MVP v0 只读壳实现\"",
  "rationale": ["Goal readiness is ready."],
  "counts": {"issues": 8, "completedIssues": 8, "runs": 6},
  "sources": {"goal": ".agentflow/goal.json", "scopeState": ".agentflow/scope-state.json"}
}
```

Project-aware GoalLoop 的已锁定选择优先级：goal readiness -> active issue -> incomplete issue -> active project next issue intent -> ROADMAP 候选施工包 -> wait-human。GoalLoop 仍只推荐，不执行。

### `DesktopWorkbenchSnapshot`

```json
{
  "version": "0.0.1",
  "initialized": true,
  "projectRoot": "/Users/mac/Documents/AgentFlow",
  "projectSummaryMarkdown": "# Project Summary",
  "goalLoopSummaryMarkdown": "# Goal Loop Summary",
  "goalLoop": {"nextAction": "plan", "recommendedCommand": "agentflow plan ..."},
  "issues": [],
  "runs": [],
  "savedViews": [],
  "evidence": [],
  "reviews": [],
  "projectUpdates": [],
  "counts": {"issues": 9, "runs": 7, "evidenceReports": 9, "reviews": 9, "projectUpdates": 9},
  "boundary": {"readOnly": true, "disallowedActions": ["run", "verify", "review"]}
}
```

## CLI v0

| Command | 行为 |
| --- | --- |
| `agentflow init --from-goal GOAL.md` | 编译 `/goal`，创建 `.agentflow/` |
| `agentflow goal bootstrap` | 在既有 `.agentflow/` 中补齐 AEP 第一阶段初始化包 |
| `agentflow goal check` | 检查 goal / ProjectDefinition / ScopeState / bootstrap 是否完整 |
| `agentflow goal next` | 只读本地事实源，写出 GoalLoopState 和下一步建议 |
| `agentflow context` | 收集 repo map、测试命令和相关文件 |
| `agentflow plan` | goal + context -> candidate issue / contract |
| `agentflow run` | 读取 contract，默认 dry-run，生成 run |
| `agentflow verify` | 执行 validation commands |
| `agentflow review` | 生成 evidence、review、handoff、update |
| `agentflow index rebuild` | 从 `.agentflow/` 事实源重建 SQLite 查询索引 |
| `agentflow view save` | 保存 SavedView filter，不保存结果 |
| `agentflow view show` | 从 SQLite 索引读取 SavedView 结果 |
| `agentflow update summary` | 从 issue / run / update / view 事实生成当前项目摘要 |
| `agentflow metrics` | 从本地事实源和 DesktopWorkbenchSnapshot 派生只读 metrics |
| `agentflow projects` | 从本地事实源派生只读 LocalProjectModelSnapshot，不写 workspace/team/project 文件 |
| `agentflow search "<query>"` | 只读扫描授权 `.agentflow/` 文本事实，返回可追溯搜索结果 |
| `agentflow review-assistant` | 生成本地 issue 审查助手清单，不做远程 PR 操作 |

## Desktop v0

| Surface | 行为 |
| --- | --- |
| 总览 | 展示 workspace 摘要、Project Summary、Goal Loop Summary、counts、next action 和 recommended command |
| 团队 | 展示团队父栏目、项目子栏目、任务子栏目；一个 workspace 可以有多个团队，每个团队下展示关联项目和任务 |
| 项目 | 展示 LocalProject、Milestone、GoalLoopSelection 和 issue refs |
| 任务 | 展示 issue id、status、title、scope、non-goals、latest run、validation commands、evidence / review links |
| Refresh | 重新读取本地快照，不执行命令 |

Desktop 左侧栏目采用工作区树，而不是平铺功能菜单：

```text
Workspace
  project
  issues
Team1
  project
  issues
Team2
  project
  issues
```

## Local Pro v0

| Candidate | v0 Gate |
| --- | --- |
| Local metrics | 只读派生，优先内存 snapshot，不写 `.agentflow/` |
| DuckDB analytics | 后置实验，只能是可重建缓存 |
| Project intelligence | 规则引擎优先，模型调用必须另行授权 |
| Local search boundary | 已定义可搜索路径、排除路径、结果字段、literal query 和索引边界 |
| Local search reader | 已实现 CLI 只读扫描授权文本事实，不建索引、不写文件 |
| Saved query boundary | 已定义文件格式候选、确认门和 no-result-persistence 规则，不写 `.agentflow/queries` |
| Desktop search boundary | 已定义 Desktop 搜索入口、只读展示和 no-execution 规则 |
| Desktop search read-only view | 已实现只读 UI，不写 query、不执行命令 |
| Saved query writer boundary | 已定义 `.agentflow/queries/*.json` 写入合同、schema、路径边界和用户确认 |
| Local workspace/project model boundary | 已锁定本地最小层级关系，不创建 workspace/team/project 文件 |
| Local project model | 已实现只读派生 workspace/team/project snapshot，不迁移 issue |
| Desktop project view | 已实现只读 Project 视图，复用 LocalProjectModelSnapshot，不写 seed 文件 |
| Desktop workspace overview | 已实现总览 Workspace 入口，展示 Workspace Projects / Teams 与 Team Issues / Projects 摘要 |
| Local project seed boundary | 已定义 seed 写入合同、schema、确认门和 evidence 要求，不写文件 |
| Local project seed | 下一候选，在用户确认门下实现 seed writer / preview |
| Saved query writer | 后置，只能在确认门下写 query definition，不保存结果 |
| Multi-project workspace | 只读多根目录概览，不复制代码、不合并事实源 |
| Desktop interactions | 默认禁用执行和写入，任何交互必须单独 IssueContract |

### `LocalMetricsSnapshot`

```json
{
  "version": "0.0.1",
  "initialized": true,
  "projectRoot": "/Users/mac/Documents/AgentFlow",
  "issues": {"total": 20, "completed": 20, "planned": 0, "active": 0},
  "runs": {"total": 18, "passed": 18, "failed": 0, "missingValidation": 0},
  "artifacts": {"evidenceReports": 21, "reviews": 36, "projectUpdates": 18, "savedViews": 1},
  "goalReady": true,
  "activeIssueId": null,
  "nextAction": "plan",
  "recommendedCommand": "agentflow plan \"Local Project Seed v0 边界定义\"",
  "latestRun": {"id": "RUN-0018", "issueId": "ISSUE-0020", "status": "completed", "validationStatus": "passed"},
  "latestEvidence": {"path": ".agentflow/evidence/ISSUE-0020-evidence.md", "title": "ISSUE-0020-evidence"},
  "latestReview": {"path": ".agentflow/reviews/ISSUE-0020-review.md", "title": "ISSUE-0020-review"}
}
```

### `LocalSearchSnapshot`

```json
{
  "version": "0.0.1",
  "initialized": true,
  "projectRoot": "/Users/mac/Documents/AgentFlow",
  "query": {"query": "Local Search"},
  "results": [
    {
      "sourceType": "file",
      "entityKind": "issue",
      "entityId": "ISSUE-0014",
      "path": ".agentflow/issues/ISSUE-0014.md",
      "title": "Local Search Reader v0 只读实现",
      "field": "Scope",
      "line": 9,
      "snippet": "新增 LocalSearchQuery / LocalSearchResult / LocalSearchSnapshot 数据对象。",
      "score": 100
    }
  ],
  "searchedPaths": [".agentflow/issues/ISSUE-0014.md"],
  "excludedPaths": [".agentflow/index.sqlite*", ".agentflow/search/", ".agentflow/queries/"],
  "boundary": {"readOnly": true, "disallowedActions": ["run", "verify", "review"]}
}
```

### `LocalSearchResult`

Local Search Reader v0 的最小结果字段如下：

```json
{
  "sourceType": "file",
  "entityKind": "issue",
  "entityId": "ISSUE-0012",
  "path": ".agentflow/issues/ISSUE-0012.md",
  "title": "Local Search v0 边界定义",
  "field": "Scope",
  "line": 9,
  "snippet": "定义可搜索路径、必须排除路径、搜索结果字段...",
  "score": 100
}
```

Reader v0 只允许 literal text query，不支持 regex、boolean grammar、embedding、模型语义搜索或远程搜索。

### `SavedQueryDefinition` 候选

Saved Query v0 当前只定义 schema 候选，不创建 `.agentflow/queries`：

```json
{
  "version": "0.0.1",
  "id": "local-search-boundary",
  "name": "Local Search Boundary",
  "query": {
    "text": "Local Search",
    "mode": "literal",
    "caseSensitive": false
  },
  "scope": {
    "sourceTypes": ["file"],
    "entityKinds": ["issue", "run", "evidence", "review", "project-update"],
    "paths": [".agentflow/issues/*.md", ".agentflow/evidence/*.md"],
    "includeDerivedSnapshots": false
  },
  "display": {
    "fields": ["path", "line", "snippet", "entityKind", "entityId", "score"],
    "maxResults": 50,
    "sort": "score-desc-then-path"
  },
  "resultPersistence": "none"
}
```

Saved Query 不复用 `.agentflow/views/*.json`。SavedView 继续只保存 issue/run filter；Saved Query 后续若实现写入，首选 `.agentflow/queries/*.json`，且必须另建 IssueContract 和用户确认点。

### Saved Query Writer Boundary

Saved Query Writer v0 当前只定义写入合同，不创建 `.agentflow/queries`：

```json
{
  "writer": "SavedQueryDefinition",
  "path": ".agentflow/queries/{query-id}.json",
  "confirmationGates": [
    "create-directory",
    "create-query",
    "overwrite-query",
    "delete-query",
    "change-query-scope"
  ],
  "writes": ["query-definition-only"],
  "resultPersistence": "none",
  "disallowedActions": ["save-results", "create-search-index", "model-call", "remote-upload"]
}
```

Writer 后续实现必须先验证 schema、路径、确认门、round-trip 和 no-result-persistence。Desktop Search 输入框不能自动触发 Writer；Desktop 写入 UI 必须另建 Desktop Interaction Gate。

### Desktop Search Read-only View Boundary

Desktop Search Read-only View v0 已实现只读 UI，仍只允许展示 Local Search Reader 派生结果：

```json
{
  "source": "LocalSearchSnapshot",
  "ui": {
    "queryInput": true,
    "resultList": ["path", "line", "snippet", "entityKind", "entityId", "score"],
    "sourceTrace": true,
    "states": ["empty", "loading", "error"],
    "readOnlyBadge": true,
    "recommendedCommandMode": "display-only"
  },
  "writes": [],
  "disallowedActions": ["run", "verify", "review", "create-issue", "model-call", "remote-upload"]
}
```

当前 UI 只能通过 Tauri command 调用 `read_local_search_snapshot`，不能 shell 执行 CLI，不能保存 query 或结果。

当前 Desktop Search 只读实现已满足该契约：`load_search_snapshot` 直接调用 Rust core reader，React Search 视图只展示结果和推荐命令文本。

### Project / Issue Status Model v0

当前 MVP 状态模型已经从执行状态机收敛为产品创建模型：

```text
Project status = draft / active / paused / completed / canceled
Issue status = backlog / todo / in_progress / in_review / done / canceled
Milestone status = 不作为产品状态机，只展示 derived progress
```

`LocalProjectModelSnapshot` 会同时输出 raw status 和 canonical status：

```text
LocalProject.status
LocalProject.canonicalStatus
LocalProjectIssueRef.status
LocalProjectIssueRef.canonicalStatus
LocalMilestone.progress
```

旧 `.agentflow/` 中的 `planned / ready / active / completed / done / cancelled / canceled / blocked` 继续兼容读取，但新 Product Feature issue 和 `agentflow plan` issue 默认写入 `todo`，review 完成后写入 `done`。

### Team / Project / Milestone / Issue Writers v0

当前 MVP 的第一个核心创建闭环已经落到 CLI：

```text
agentflow team create "<team name>"
agentflow project create "<project title>"
agentflow milestone create "<milestone title>"
agentflow issue create "<issue title>"
```

所有 create 命令默认只 preview，不写 `.agentflow/`。只有 `--write --yes` 后才允许写入。

写入规则：

```text
Team -> .agentflow/teams/{team-id}.json + workspace.teamIds
Project -> .agentflow/projects/{project-id}.json + workspace.projectIds + team.projectIds
Milestone -> project.milestones[]
Issue -> .agentflow/issues/ISSUE-XXXX.{json,md} + project/team/index 引用
```

状态规则：

```text
Project default status = draft
Issue default status = todo
Milestone status = 不写入，不作为产品状态机
```

Project 可以通过明确参数写成 `active`，但 writer 不会隐式覆盖 `workspace.activeProjectId`。Desktop 继续只读展示这些本地事实源，不提供写入口。

## 规则

- SavedView 只保存 filter，不保存结果、不授权执行。
- ProjectUpdate 必须追溯 run、validation、evidence 或 review。
- 未定义 validation 的 run 不允许 completed。
- `goal check` 未通过前，不进入可执行 issue。
- active issue WIP limit = 1。
- `goal next` 只做本地决策，不执行命令、不调用模型、不创建远程 issue。
- `goal next` 必须优先完成 active issue；有 active milestone issue queue 时，只推进当前 milestone 的唯一 eligible issue。
- Project-aware GoalLoop 可从 active project / active milestone backlog 推荐下一条 issue intent，但仍不能执行。
- IssueContract 必须包含 AEP issue protocol 字段。
- LocalWorkspace / LocalTeam / LocalProject 是本地组织模型，不引入账号、权限、云同步或远程团队协作。
- Desktop Workbench v0 只读读取 `DesktopWorkbenchSnapshot`，不拥有事实源写权限。
- Local Pro Experiments v0 只定义边界，不授权具体功能实现。
- DuckDB、saved query writer、多项目 workspace 和 Desktop 写交互都必须另建 IssueContract。
- Local Metrics Snapshot v0 只读派生，不新增 DuckDB、不写 `.agentflow/analytics`。
- Local Search v0 只定义边界，不实现搜索引擎、不写 `.agentflow/search` / `.agentflow/queries`、不新增 Desktop 搜索 UI。
- Local Search Reader v0 只读扫描授权路径，saved query、FTS、Desktop search view 和语义搜索都必须另建 IssueContract。
- `agentflow search` 不创建索引、不写 query 文件、不调用模型、不修改 `.agentflow/` 事实源。
- Saved Query v0 只定义 schema 候选、路径边界、确认门和验证方式，不创建 `.agentflow/queries`，不保存搜索结果。
- Desktop Search Read-only View v0 边界已完成，Search UI 已实现但仍不执行命令、不写 `.agentflow/search` 或 `.agentflow/queries`。
- Desktop Search Read-only View v0 实现只调用 Local Search Reader，不保存 query、不保存结果、不写 `.agentflow/search` 或 `.agentflow/queries`。
- Saved Query Writer v0 边界只定义 `.agentflow/queries/*.json` 写入合同和确认门，不创建目录、不写 query 文件、不实现 writer。
- Local Workspace / Team / Project Model v0 只定义本地层级模型，不创建 `.agentflow/workspace.json`、`.agentflow/teams/` 或 `.agentflow/projects/`。
- Local Project Model v0 只读派生 `LocalProjectModelSnapshot`，不创建 workspace/team/project 文件，不迁移 issue，不实现 Project-aware GoalLoop。
- Desktop Project View v0 只调用 `load_project_model_snapshot` 展示同一只读 snapshot，不写 seed 文件，不执行 recommended command。
- Desktop Workspace Overview v0 只把 snapshot 层级摘要放到总览，不创建 workspace/team/project seed，不新增执行按钮。
- Local Project Seed v0 默认只读 preview，只有显式 `agentflow project-seed --write --yes` 才写 `.agentflow/workspace.json`、`.agentflow/teams/core.json`、`.agentflow/projects/agentflow-local-execution.json`。
- Issue Project Link v0 Boundary 只定义 `projectLink.teamId/projectId/milestoneId/linkSource` 候选字段，不迁移现有 issue，不改写 GoalLoop。
- Issue Project Link Writer v0 默认只读 preview，只有显式 `agentflow issue-link ISSUE-XXXX --write --yes` 才写指定 issue 的 JSON / Markdown，且拒绝覆盖已有 `projectLink`。
- Project-aware GoalLoop v0 已实现 active project candidate fallback；没有 candidate 时回退 roadmap，且不执行 recommended command。
- Desktop GoalLoop Trace v0、Desktop Issue Lifecycle Trace v0、Desktop Project Update Timeline v0 保留为内部只读 trace 能力，不作为 MVP 主导航入口。
- Desktop MVP Navigation Scope Reduction v0 将主导航收敛为总览、团队、项目、任务，不新增 trace 视图，不删除底层 reader，不写 `.agentflow/`。
- Project canonical status 只允许 `draft / active / paused / completed / canceled`。
- Issue canonical status 只允许 `backlog / todo / in_progress / in_review / done / canceled`。
- Milestone 不维护独立产品状态，CLI / Desktop 只展示从 issue 派生的完成度。
- Project / Milestone / Issue / View Model v1 是当前 MVP 的产品主干：Project 负责方向和成功标准，Milestone 负责阶段出口，Issue 负责执行合同，View 只负责过滤展示。
- View 不能写 Project / Milestone / Issue 状态，不能保存搜索结果作为事实源，不能执行 `run / verify / review`。
- Queue Preflight 是 `Backlog -> Todo` 的唯一授权门；不能把 Backlog 当成可执行任务。
- Desktop Project / Milestone / Issue 页面职责收敛 v0 只能展示 v1 snapshot；Project 不展示 issue 执行细节，Milestone 不展示完整 evidence / review，Issue 不展示 project closure / audit，View 不写任何业务状态。

## 验收

1. 从 `GOAL.md` 初始化 `.agentflow/goal.{md,json}`。
2. 收集上下文并生成候选 issue。
3. 生成 issue contract。
4. dry-run。
5. 执行 validation。
6. 生成 evidence / review / update。
7. 重建 SQLite index 并读取 SavedView。
8. 生成 Project Summary 和 Review Assistant。
9. 定义 Desktop Workbench MVP v0 只读边界。
10. 补齐 AEP Goal Initialization Protocol v0。
11. 实现 Goal Loop Orchestrator v0。
12. 启动 Desktop Workbench MVP v0 只读壳，展示 Project Summary、Goal Loop Summary、recommended command、issues、evidence、review 和 saved views。
13. 定义 Local Pro Experiments v0 边界，并让下一候选指向 `Local Metrics Snapshot v0 只读实现`。
14. 实现 Local Metrics Snapshot v0 只读派生，CLI 和 Desktop 都只展示 metrics，不执行命令。
15. 定义 Local Search v0 边界，并让下一候选指向 `Local Search Reader v0 只读实现`。
16. 实现 Local Search Reader v0 只读搜索，并让下一候选指向 `Saved Query v0 边界定义`。
17. 定义 Saved Query v0 边界，并让下一候选指向 `Desktop Search Read-only View v0 边界定义`。
18. 定义 Desktop Search Read-only View v0 边界，并让下一候选指向 `Desktop Search Read-only View v0 实现`。
19. 实现 Desktop Search Read-only View v0，并让下一候选指向 `Saved Query Writer v0 边界定义`。
20. 定义 Saved Query Writer v0 边界，并让下一候选指向 `Saved Query Writer v0 实现`。
21. 定义 Local Workspace / Team / Project Model v0 边界，并让下一候选指向 `Local Project Model v0 只读实现`。
22. 实现 Local Project Model v0 只读派生，并让下一候选指向 `Local Project Seed v0 边界定义`。
23. 实现 Desktop Project View v0 只读展示，继续保持 `Local Project Seed v0 边界定义` 为下一候选。
24. 实现 Desktop Workspace Overview v0 只读入口，让总览展示 workspace projects / teams 和 team issues / projects。
25. 定义 Local Project Seed v0 边界，并让下一候选指向 `Local Project Seed v0 实现`。
26. 实现 Local Project Seed v0 preview / writer，并让下一候选指向 `Issue Project Link v0 边界定义`。
27. 定义 Issue Project Link v0 边界，并让下一候选指向 `Issue Project Link Writer v0 实现`。
28. 实现 Issue Project Link Writer v0 preview / writer，并让下一候选指向 `Project-aware GoalLoop v0 边界定义`。
29. 定义 Project-aware GoalLoop v0 边界，并让下一候选指向 `Project-aware GoalLoop v0 实现`。
30. 实现 Project-aware GoalLoop v0，并让下一候选指向 `Desktop GoalLoop Trace v0 只读展示`。
31. 实现 Desktop GoalLoop Trace v0 只读展示，并让下一候选指向 `Desktop Issue Lifecycle Trace v0 只读展示`。
32. 实现 Desktop Issue Lifecycle Trace v0 只读展示，并让下一候选指向 `Desktop Project Update Timeline v0 只读展示`。
33. 实现 Desktop Project Update Timeline v0 只读展示，并让下一候选指向 `Desktop Run Validation Trace v0 只读展示`。
34. 实现 Desktop MVP Navigation Scope Reduction v0，并让下一候选指向 `Desktop MVP Task Detail v0 收敛`。
35. 实现 Desktop Team Hierarchy v0，把团队入口收敛为 workspace 多团队层级，只读展示每个团队下的项目和任务，不开放 Desktop 写入口。
36. 实现 Desktop Team Parent Child Columns v0，在栏目上明确表达团队是父级，项目和任务是团队下的子级。
37. 实现 Desktop Workspace Sidebar Tree v0，把左侧栏目收敛为 Workspace / Team 工作区树，每个父节点下展示 project / issues 子项。
38. 实现 Desktop Teams Add Button v0，在 TEAMS 标题右侧增加 `+` 新增入口，点击只打开本地初始化创建面板，不保存 team。
39. 实现 Goal + Criteria Driven MVP，把当前 MVP 主目标收敛为用户和 Agent 共同创建 Team / Project / Milestone / Issue。
40. 实现 Project / Issue Status Model v0，CLI 和 Desktop 展示 canonical Project / Issue status，Milestone 展示 derived progress。
41. 实现 Team / Project / Milestone / Issue Writers v0，提供 preview-first 的本地 CLI 创建闭环。
42. 定义并实现 Project / Milestone / Issue / View Model v1 只读 schema adapter，把 MVP 产品主干、对象职责、模板、Queue Preflight 和 Desktop 页面职责固定下来。
43. 实现 Project / Milestone / Issue / View Model v1 writer preview 对齐，让 create preview 输出 Team relation、Project charter、Milestone gate 和 Issue execution contract。
44. 实现 Desktop Project / Milestone / Issue 页面职责收敛 v0，让 Desktop 通过 v1 snapshot 只读展示 Project、Milestone、Issue 和 View 各自负责的内容。
