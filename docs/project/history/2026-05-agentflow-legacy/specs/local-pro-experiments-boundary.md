# Local Pro Experiments Boundary

创建日期：2026-05-22
执行者：Codex

## 定位

Local Pro Experiments v0 是 Desktop Workbench MVP v0 只读壳之后的本地高级能力边界。它只定义后续实验的产品范围、架构边界、授权门和验证方式，不实现任何具体功能。

Local Pro 的核心判断标准：

- 仍然本地优先，不引入账号、云同步或团队 workspace。
- 优先从 `.agentflow/` 事实源和可重建索引派生能力。
- 每个实验必须先有 IssueContract，再允许修改代码或事实源。
- Desktop Workbench 默认继续只读；任何交互写入都必须另建 issue 并明确授权。

## 当前阶段不允许

| 禁止项 | 原因 |
| --- | --- |
| 云同步 | 会改变本地免费工具边界 |
| 账号 / 支付 | 不属于本地产品验证阶段 |
| 团队协作 | 会引入权限、冲突和远程状态 |
| 远程 PR / Linear issue | 当前只允许本地 evidence / review |
| Desktop UI 执行 `run / verify / review` | 会绕过 CLI issue contract 和 WIP=1 约束 |
| 自动调用模型 | 需要单独的数据边界、成本边界和用户确认 |
| 绕过 IssueContract 写 `.agentflow/` | 会破坏事实源可信度 |
| 将 DuckDB 或 SQLite 作为事实源 | 它们只能是派生缓存或分析索引 |

## 候选实验范围

| 实验域 | v0 定义 | 首选方向 |
| --- | --- | --- |
| Local analytics / metrics | 从 issue、run、validation、evidence、review 派生本地指标 | 只读内存快照优先 |
| DuckDB 后置分析 | 面向较大历史的分析缓存 | 后置于 metrics snapshot，不先引入 |
| Local project intelligence | 基于本地事实给出项目健康、风险和下一步提示 | 规则引擎优先，模型后置 |
| Local search / saved query | 在 `.agentflow/` 文本和 JSON 上做本地查询 | 只读 search，saved query 需单独授权 |
| Local workspace / project model | 组织一个本地 workspace 下的 teams、projects、milestones 和 issues | 先只读模型，再 project-aware goal loop |
| Multi-project workspace | 读取多个 AgentFlow 项目的状态 | 后置于 LocalProject，不复制代码，不合并事实源 |
| Desktop Workbench interactions | 在桌面中增加更强操作入口 | v0 后仍默认禁用执行按钮 |

## 授权门矩阵

| 候选实验 | 只读 | 写 `.agentflow/` | IssueContract | 用户确认 | 验证 | Evidence |
| --- | --- | --- | --- | --- | --- | --- |
| Local Metrics Snapshot v0 | 是 | 否 | 必须 | 不需要，除非读取多个项目 | `cargo test`, `npm build`, snapshot fixture | metrics 字段、UI/CLI 输出、read-only 证明 |
| DuckDB Analytics Cache v0 | 否，写派生 cache | 只能写 `.agentflow/analytics/*.duckdb` 派生缓存 | 必须 | 需要确认写入路径 | import/rebuild parity、cache 可删除重建 | cache schema、rebuild log、source trace |
| Project Intelligence Rules v0 | 是 | 否 | 必须 | 不需要 | deterministic rules tests | rule output、false-positive limitation |
| Local Search Boundary v0 | 是 | 否 | 必须 | 不需要 | boundary anchors、path allow/deny trace | searchable paths、excluded paths、query deferral |
| Local Search Reader v0 | 是 | 否 | 必须 | 不需要 | query fixture、path boundary test | query result sample、excluded paths |
| Saved Query Boundary v0 | 是 | 否 | 必须 | 不需要 | boundary anchors、no `.agentflow/queries` directory | schema candidate、confirmation gates |
| Saved Query Writer v0 | 否 | 只允许写 `.agentflow/queries/*.json` | 必须 | 需要确认新文件格式和具体路径 | JSON schema、round-trip、path boundary | saved query artifact、no-result-persistence proof |
| Desktop Search Boundary v0 | 是 | 否 | 必须 | 不需要 | boundary anchors、no UI implementation | UI contract、read-only gate |
| Desktop Search Read-only View v0 | 是 | 否 | 必须 | 不需要 | UI build、search fixture、no write dirs | result render、source trace、read-only proof |
| Local Workspace / Project Model v0 | 是 | 否 | 必须 | 不需要 | no workspace/team/project files、model anchors | hierarchy contract、goal-loop selection boundary |
| Local Project Model v0 | 是 | 否 | 必须 | 不需要 | snapshot fixture、no issue migration | local project snapshot、active project read model |
| Local Project Seed v0 | 否，默认 preview 只读 | 只允许写 workspace/team/project seed | 必须 | `--write --yes` 明确确认 | preview no-write、tempdir explicit write tests | seed preview、no live seed proof、writer path trace |
| Issue Project Link v0 Boundary | 是 | 否 | 必须 | 不需要 | schema boundary、no migration proof | issue ownership boundary、migration deferral |
| Issue Project Link Writer v0 | 否，默认 preview 只读 | 只允许写 `.agentflow/issues/{issue-id}.json` 的 `projectLink` | 必须 | 需要确认目标 issue 和归属 | no-history-migration、JSON schema、round-trip | projectLink artifact、source trace |
| Multi-project Read-only Workspace v0 | 是 | 否 | 必须 | 需要确认项目根列表 | multi-root fixture、no-copy scan | root list、per-project summary |
| Desktop Interaction Gate v0 | 视交互而定 | 默认否 | 必须 | 涉及写入或执行必须确认 | UI smoke、command boundary tests | interaction contract、disabled states |

## 已完成和下一小切片

已完成小切片：

```text
Local Metrics Snapshot v0 只读实现
```

边界：

- 只从 `.agentflow/` 和现有 in-memory snapshot 派生指标。
- 不新增 DuckDB。
- 不写 `.agentflow/analytics`。
- 不新增 Desktop 执行按钮。
- 不调用模型。
- 输出可以是 Rust 内存结构、CLI stdout 或 Desktop read-only view；是否写文件必须另建 issue。

建议指标：

- issue 总数、completed / planned / active 数量。
- run 总数、passed / failed / missing validation 数量。
- evidence / review / project update 数量。
- goal readiness、active issue、next action。
- 最近 run、最近 evidence、最近 review。

实现状态：`implemented / read-only`

已完成边界小切片：

```text
Local Search v0 边界定义
```

Local Search v0 边界定义只允许定义搜索范围、排除路径、query 表达、saved query 后置规则、索引可重建规则、结果字段和 Desktop 只读展示边界，不实现搜索索引。

实现状态：`implemented / boundary-only`

已完成只读小切片：

```text
Local Search Reader v0 只读实现
```

边界：

- 只读扫描 `docs/specs/local-search-boundary.md` 授权的 `.agentflow/` JSON / JSONL / Markdown 路径。
- 当前只读扫描 `.agentflow/` 文件事实；DesktopWorkbenchSnapshot / LocalMetricsSnapshot 派生文本搜索后置。
- 不创建索引。
- 不写 `.agentflow/search` 或 `.agentflow/queries`。
- 不新增 Desktop 搜索 UI。
- 不调用模型。

实现状态：`implemented / read-only`

已完成边界小切片：

```text
Saved Query v0 边界定义
```

边界：

- 只定义 saved query / saved search 文件格式、确认点和验证方式。
- 不写 `.agentflow/queries`。
- 不保存搜索结果。
- 不新增 Desktop 搜索 UI。
- 不调用模型。
- Saved Query 与现有 SavedView 分离，后续写入首选 `.agentflow/queries/*.json`。

实现状态：`implemented / boundary-only`

下一候选小切片：

```text
Desktop Search Read-only View v0 边界定义
```

边界：

- 只定义 Desktop 搜索入口和只读展示约束。
- 不新增 Desktop 搜索 UI 实现。
- 不执行 run / verify / review。
- 不写 `.agentflow/search` 或 `.agentflow/queries`。

实现状态：`implemented / boundary-only`

下一候选小切片：

```text
Desktop Search Read-only View v0 实现
```

边界：

- 只调用 Local Search Reader 的只读能力。
- 只展示搜索结果和 source trace。
- 不保存 query，不保存结果，不写 `.agentflow/search` 或 `.agentflow/queries`。
- 不执行 run / verify / review，不创建 issue，不调用模型。

实现状态：`implemented / read-only`

下一候选小切片：

```text
Saved Query Writer v0 边界定义
```

边界：

- 只定义 `.agentflow/queries/*.json` 写入合同、schema、用户确认点和验证方式。
- 不创建 query 目录。
- 不写 query 文件。
- 不保存搜索结果。

实现状态：`implemented / boundary-only`

下一候选小切片：

```text
Saved Query Writer v0 实现
```

边界：

- 只能写 `.agentflow/queries/*.json` query definition。
- 必须有 IssueContract、用户确认点和路径确认。
- 不保存搜索结果，不创建 `.agentflow/search`，不创建索引或 cache。
- Desktop Search 不能自动触发 writer。

实现状态：`deferred`

已完成边界小切片：

```text
Local Workspace / Team / Project Model v0 边界定义
```

边界：

- 锁定 `LocalWorkspace -> LocalTeams / LocalProjects -> Milestones -> IssueContracts -> GoalLoopSelection`。
- 不创建 `.agentflow/workspace.json`、`.agentflow/teams/` 或 `.agentflow/projects/`。
- 不迁移现有 issue。
- 不改变 IssueContract 唯一执行授权。
- Project-aware GoalLoop 后续只能推荐，不执行。

实现状态：`implemented / boundary-only`

下一候选小切片：

```text
Local Project Model v0 只读实现
```

边界：

- 只读派生 LocalWorkspace / LocalTeam / LocalProject snapshot。
- 不创建 workspace/team/project 文件。
- 不迁移 issue，不改变 `agentflow run`。
- 为后续 Project-aware GoalLoop 提供只读输入。

实现状态：`implemented / read-only`

已完成小切片：

```text
Local Project Seed v0 实现
```

边界：

- 在 `docs/specs/local-project-seed-boundary.md` 的确认门下实现 seed preview / writer。
- 默认 preview 不写文件。
- 写入必须显式 `agentflow project-seed --write --yes`。
- 不迁移 issue。
- 不实现 Project-aware GoalLoop。

实现状态：`implemented / explicit-confirmation-writer`

已完成边界小切片：

```text
Issue Project Link v0 边界定义
```

边界：

- 只定义 IssueContract 与 LocalTeam / LocalProject / Milestone 的归属字段和迁移顺序。
- 不迁移现有 issue。
- 不改写 GoalLoop。
- 不放宽 IssueContract 唯一执行授权。

实现状态：`implemented / boundary-only`

已完成小切片：

```text
Issue Project Link Writer v0 实现
```

边界：

- 默认 preview，不迁移历史 issue。
- `agentflow issue-link ISSUE-XXXX` 默认只读 preview。
- `agentflow issue-link ISSUE-XXXX --write --yes` 只能在明确确认后写指定 issue 的 `projectLink`。
- writer 只写指定 issue 的 JSON / Markdown，拒绝覆盖已有 `projectLink`。
- 不实现 Project-aware GoalLoop。
- Desktop 仍不提供写入口。

实现状态：`implemented / explicit-confirmation-writer`

已完成小切片：

```text
Project-aware GoalLoop v0 边界定义
```

边界：

- 定义 active project / active milestone candidate 的推荐优先级。
- 保持 WIP=1、active issue 和 incomplete issue 优先级不变。
- 缺失 workspace seed、project seed、projectLink 或 nextIssueIntent 时回退 roadmap candidate。
- 只授权后续实现，不执行 recommended command。

实现状态：`implemented / boundary-only`

已完成小切片：

```text
Project-aware GoalLoop v0 实现
```

边界：

- `goal_loop_decision` 已加入 active project / active milestone candidate fallback。
- 只有无 active issue、无 incomplete issue 时才推荐 project candidate。
- 缺失 project candidate 时回退 roadmap candidate。
- 不执行 recommended command，不迁移 issue，不写 projectLink。

实现状态：`implemented / local-decision-only`

已完成小切片：

```text
Desktop GoalLoop Trace v0 只读展示
```

边界：

- Desktop Workbench 新增 GoalLoop Trace 只读视图。
- 只读取 `DesktopWorkbenchSnapshot.goalLoop`、`.agentflow/goal-loop.json` 和 `.agentflow/updates/GOAL-LOOP-SUMMARY.md`。
- 展示 GoalLoop 决策优先级、project candidate / roadmap fallback 和 recommended command。
- 不执行 recommended command，不写 `.agentflow/`，不创建 issue。

实现状态：`implemented / read-only`

已完成小切片：

```text
Desktop Issue Lifecycle Trace v0 只读展示
```

边界：

- Desktop Workbench 新增 Issue Lifecycle Trace 只读视图。
- 只读取 `DesktopWorkbenchSnapshot` 的 issues、runs、evidence、reviews 和 projectUpdates。
- 展示 contract、run、validation、evidence、review、project update、completed 链路。
- 不执行 run / verify / review，不写 `.agentflow/`，不创建或编辑 issue。

实现状态：`implemented / read-only`

下一候选小切片：

```text
Desktop Project Update Timeline v0 只读展示
```

边界：

- Desktop Workbench 新增 Project Update Timeline 只读视图。
- 只读取 `DesktopWorkbenchSnapshot` 的 projectUpdates、issues、runs、evidence 和 reviews。
- 展示 PROJECT-UPDATE、issue、run、validation、evidence、review 的项目推进链路。
- 不执行 run / verify / review，不创建或编辑 issue，不保存 timeline filter，不写 `.agentflow/`。

实现状态：`implemented / read-only`

下一候选小切片：

```text
Desktop MVP Navigation Scope Reduction v0
```

Desktop MVP Navigation Scope Reduction v0

边界：

- Desktop Workbench 主导航只展示总览、团队、项目、任务。
- GoalLoop Trace、Issue Lifecycle Trace、Project Update Timeline、Search、Metrics 等保留为内部只读能力，不作为 MVP 主入口。
- 不新增 trace 视图，不删除底层 reader，不执行 run / verify / review，不写 `.agentflow/`。

实现状态：`implemented / mvp-scope-reduction`

下一候选小切片：

```text
Desktop MVP Task Detail v0 收敛
```

## 事实源规则

| 数据 | 事实源 | 派生物 |
| --- | --- | --- |
| Goal / ProjectDefinition / ScopeState | `.agentflow/*.json` | readiness / loop 状态 |
| Issue | `.agentflow/issues/*.json` | metrics、search result、desktop list |
| Run / validation | `.agentflow/runs/*/run.json` | validation metrics |
| Evidence / review | `.agentflow/evidence/*.md`, `.agentflow/reviews/*.md` | text index、project intelligence hints |
| Project update | `.agentflow/updates/*.md` | timeline、summary |
| SavedView | `.agentflow/views/*.json` | filtered query result |
| LocalWorkspace / LocalTeam / LocalProject | 当前由 `LocalProjectModelSnapshot` 只读派生，seed writer 合同见 `docs/specs/local-project-seed-boundary.md` | Desktop Project View / Workspace Overview 已只读展示；project-aware goal loop 后置 |
| SQLite / DuckDB | 可重建缓存 | 不能成为事实源 |

## Desktop 边界

Desktop Workbench 在 Local Pro 阶段仍保持默认只读：

- 可以展示 Local Pro 只读派生结果。
- 可以刷新本地 snapshot。
- 可以显示 recommended command。
- 不可以直接执行 recommended command。
- 不可以创建、编辑或完成 issue。
- 不可以触发 run / verify / review。

任何 Desktop 写入或执行能力都必须满足：

1. 单独 IssueContract。
2. 明确写入路径或执行命令。
3. 明确用户确认点。
4. 明确失败回滚和 evidence。
5. 明确不会绕过 WIP=1。

## 验收

Local Pro Experiments v0 边界通过的标准：

1. 本文档存在并被 README / ROADMAP / MVP Spec 引用。
2. 每个候选实验都有授权门。
3. 当前阶段没有实现具体 Local Pro 功能。
4. Desktop Workbench 只读边界不被放宽。
5. `agentflow goal next` 在 Desktop GoalLoop Trace v0 完成后推荐 `Desktop Issue Lifecycle Trace v0 只读展示`。
