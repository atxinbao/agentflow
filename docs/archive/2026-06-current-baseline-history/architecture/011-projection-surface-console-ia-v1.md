# 011 - Projection Surface Console IA V1

创建日期：2026-06-22
执行者：Codex
状态：Architecture Contract / v0.7.0 V070-001

## Purpose

本文定义 AgentFlow `v0.7.0` 的 Projection Surface 合同和 Project OS Console 信息架构。

它回答一个问题：

```text
底层 facts / events / projections 已经存在以后，Desktop 应该如何读取、解释和呈现，而不重新变成新的事实源？
```

## 1. Core Boundary

Project OS Console projection surfaces 只能读取 projection read models 和 runtime query API。

它不能直接解释底层 authority 文件，也不能直接写 `.agentflow/**` authority facts。

这条边界只约束 Console / projection surface。

它不否认 Desktop 的项目打开、项目注册、prepare workspace 等 onboarding 流程需要写初始化文件，例如：

```text
.agentflow/workspace.yaml
.agentflow/config.yaml
```

这些初始化写入必须由 project onboarding / prepare workspace 的 owning runtime path 负责，不能由 Console 页面、View Model 或 Projection Surface 偷偷完成。

```text
Authority facts
-> Event Store
-> Projection rebuild
-> Projection Query API
-> Desktop View Models
-> Project OS Console
-> Command Surface
-> Runtime API
-> Action Proposal / Arbitration
-> Event Store
```

核心规则：

- Event 是事实记录；
- Projection 是事实的只读解释；
- View Model 是页面渲染模型；
- Console 是用户操作面；
- Command Surface 只提交命令，不直接改状态；
- Runtime API 才能把命令转换成 action proposal；
- Arbitration 接受后才能写入新的 runtime event。

## 2. Surface Contract

Projection Surface 必须暴露统一读模型，而不是让页面直接拼散落文件。

### 2.1 Required Read Models

| Read model | Owner | Consumer | Purpose |
| --- | --- | --- | --- |
| Project summary | `projection` | Project Home / Sidebar | 项目阶段、下一步、阻塞、活跃任务 |
| Spec loop summary | `projection` | Spec Workbench | 需求到 preview / confirmation / materialization 的状态 |
| Task workbench summary | `projection` | Task Workbench | issue、run、session、verification、acceptance、delivery |
| Event timeline | `projection` | Task Workbench / Advanced | 状态变化原因和事件顺序 |
| Evidence graph | `projection` | Task Workbench / Acceptance / Delivery | requirement -> issue -> run -> evidence -> delivery 追溯 |
| Audit summary | `projection` / `audit` | Audit Surface | 独立审计事实，只读展示 |
| Runtime diagnostics | `projection` / `runtime-api` | Advanced | freshness、missing fact、stale projection、conflict |
| Allowed actions | `runtime-api` query | Command Surface | 当前页面允许提交的 command hint |

### 2.2 Required Status Metadata

每个 read model 必须能表达：

- `fresh`: projection 与事件游标一致；
- `stale`: projection 落后于事件；
- `missing-fact`: 必要 authority fact 不存在；
- `conflict`: 多个事实互相冲突；
- `read-only`: 当前页面只允许查看；
- `command-available`: 存在可提交命令；
- `blocked`: 当前命令被前置条件阻断。

页面不能用空状态隐藏这些状态。缺失事实必须被明确说出来。

## 3. Console IA

Project OS Console 的一级栏目保持收敛：

```text
工作台
任务
审计
文件
高级
```

`执行` 和 `交付` 不再作为一级栏目。

原因：

- 执行是任务状态流的一部分；
- 交付是任务完成和公开发布的一部分；
- 它们都必须挂在具体 task / project 上，否则用户会看到割裂的业务流。

## 4. Page Responsibilities

### 4.1 Project Home

目标：

让用户第一屏知道项目在哪里、下一步做什么、为什么。

展示：

- Project phase；
- next action；
- active issue；
- blockers；
- Goal / Plan / Decisions 摘要；
- Spec / Work / Acceptance / Delivery / Audit 总览；
- allowed command hints。

不展示：

- 原始 JSON；
- provider session 内部细节；
- 独立执行记录列表；
- 独立交付包列表。

### 4.2 Task Workbench

目标：

作为执行、验证、证据、验收、交付的主工作面。

展示：

- project / issue tree；
- issue status；
- stage timeline；
- current event stream；
- run / session summary；
- verification summary；
- evidence graph；
- acceptance decision；
- completion commit；
- public delivery record；
- optional audit trigger evaluation。

规则：

- 已完成阶段展示历史事件；
- 当前阶段展示实时或最近事件；
- 未来阶段只展示等待条件；
- Delivery 不独立成页面，而是展示为 task / project 的结果；
- Audit 不混入 Done 主链，只显示独立触发和独立结果。

### 4.3 Spec Workbench

目标：

让用户读懂需求如何变成正式 spec / project / issue。

展示：

- intake；
- classification；
- context；
- boundary；
- route；
- preview；
- confirmation；
- materialization；
- preview artifact 与 authority artifact 的关系；
- source requirement；
- generated project / issue；
- runtime action proposal summary。

规则：

- preview 不是 authority；
- confirmation 必须绑定具体 preview；
- 未确认的 spec 不显示为可执行任务；
- 所有写入动作必须通过 Command Surface。

### 4.4 Audit Surface

目标：

只读展示独立审计事实。

展示：

- audit request；
- audit report；
- findings；
- evidence map；
- traceability；
- accepted / needs-repair / rejected；
- source delivery / source issue / source release。

规则：

- Audit 不自动改变 issue Done；
- Audit 不创建 Build / Work issue；
- Audit repair 必须通过新的 command proposal。

### 4.5 Files

目标：

只读查看项目文件。

规则：

- Files 不是编辑器；
- Files 不写源码；
- Files 不写 `.agentflow/**`；
- 文件读取失败要显示原因，而不是变成空白。

### 4.6 Advanced

目标：

诊断 runtime 和 projection，不承担普通工作流。

展示：

- projection freshness；
- missing facts；
- stale projection；
- conflict diagnostics；
- event cursor；
- command response；
- runtime errors；
- provider session snapshot summary。

规则：

- Advanced 是诊断面；
- 不提供绕过 Runtime API 的写入口；
- 不把内部文件结构当普通用户主流程。

## 5. Read Model vs View Model

### Read Model

Read Model 是跨页面稳定合同。

它来自：

- `.agentflow/projections/**`；
- `.agentflow/events/**` 的派生结果；
- `.agentflow/spec/**` 的只读引用；
- `.agentflow/tasks/**` 的证据摘要；
- `.agentflow/audit/**` 的审计摘要；
- public delivery records。

Read Model 由 `crates/projection` 和 `crates/runtime-api` query 提供。

### View Model

View Model 是 Desktop 页面模型。

它负责：

- 组合 read model；
- 翻译成用户文案；
- 决定页面显示顺序；
- 生成 icon / badge / empty state；
- 显示 allowed action hint。

它不负责：

- 判断事实是否合法；
- 推进 workflow；
- 写 `.agentflow/**`；
- 直接调用 provider；
- 生成 authority event。

## 6. Command Surface

Command Surface 是 UI 唯一合法操作入口。

用户点击按钮后，只能生成 Runtime Command：

```text
UI click
-> command request
-> Runtime API
-> Action Proposal
-> Arbitration
-> accepted / rejected / queued / human-decision-required
-> Event Store
-> Projection rebuild
-> UI refresh
```

Command Surface 必须展示：

- command label；
- target object；
- required role；
- required evidence；
- blocked reason；
- expected state transition；
- accepted / rejected result。

禁止：

- 点击按钮直接改 issue status；
- 点击按钮直接写 task artifact；
- 点击按钮直接写 audit report；
- 点击按钮直接修改 projection；
- 页面自己猜测 command 是否成功。

## 7. Console Forbidden Writes

Project OS Console 页面、Projection Surface 和 View Model 不允许直接写：

```text
.agentflow/spec/**
.agentflow/events/**
.agentflow/projections/**
.agentflow/tasks/**
.agentflow/audit/**
docs/requirements/**
docs/projects/**
CHANGELOG.md
release notes
```

如果用户动作最终需要写这些路径，必须通过 Runtime API 和对应 authority role。

例外：

- Project onboarding / prepare workspace 拥有的初始化文件，例如 `.agentflow/workspace.yaml`、`.agentflow/config.yaml`；
- 本地项目 registry；
- 窗口状态；
- 用户偏好；
- search / filter / selected tab 等纯 UI 状态。

这些例外不能写入 spec、events、tasks、audit 或 projection authority facts。

## 8. Page To Source Mapping

| Page | Primary read model | Command source | Authority write path |
| --- | --- | --- | --- |
| Project Home | project summary | project command hints | Runtime API |
| Task Workbench | task workbench summary | work command hints | Runtime API |
| Spec Workbench | spec loop summary | spec command hints | Runtime API |
| Audit Surface | audit summary | audit command hints | Runtime API |
| Files | file read model | none in v0.7.0 | none |
| Advanced | diagnostics read model | diagnostic commands only | Runtime API |

## 9. v0.7.0 Implementation Order

本合同对应 V070-001。

后续任务必须按这个边界推进：

1. `V070-002` 实现统一 query API 和 read models；
2. `V070-003` 基于 Project Home read model 实现工作台；
3. `V070-004` 基于 Spec Workbench read model 实现 spec 表面；
4. `V070-005` 基于 Task Workbench read model 实现任务主工作面；
5. `V070-006` 补 event timeline 和 evidence graph；
6. `V070-007` 把 acceptance / delivery 收进任务表面；
7. `V070-008` 把 audit 收成独立只读验收面；
8. `V070-009` 接 Command Surface 到 Runtime API；
9. `V070-010` 补 Advanced 诊断表面；
10. `V070-011` 做 Desktop view model 和 browser preview 回归；
11. `V070-012` 做 release readiness 和验收。

## 10. Acceptance For V070-001

V070-001 完成条件：

- Projection Surface contract 已定义；
- Console 一级页面职责已定义；
- `执行` / `交付` 不再作为一级业务栏目；
- read model 与 view model 边界明确；
- Command Surface 只能回流 Runtime API；
- UI forbidden writes 明确；
- 后续 V070 issues 可以引用本文作为实现边界。
