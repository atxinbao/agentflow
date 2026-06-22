# AgentFlow v0.7.0 Projection Surface OS Console Tasks V1

日期：2026-06-21
执行者：Codex
状态：Version Planning Draft / 开发前置文档 / 不授权 Build Agent 执行

## 1. Purpose

本文档沉淀 AgentFlow `v0.7.0` 的开发任务规划。

`v0.7.0` 的目标不是继续扩底层 Work Loop，而是把底层事实投影成可用的 Project OS Console。

主链：

```text
Facts
-> Projection Read Models
-> View Models
-> Project OS Console
-> Command Surface
-> Runtime API
-> Action Proposal
```

一句话：

```text
让用户不读 JSON、不读日志，也能知道项目现在在哪里、为什么停在这里、下一步该怎么合法推进。
```

## 2. Version Boundary

`v0.7.0` 处理 Projection Surface 和 Console。

它只做软件开发场景的第一版 Project OS Console：

- 只读事实展示；
- 状态解释；
- 下一步建议；
- 证据追溯；
- 验收 / 交付 / 审计阅读；
- Command Surface 回流 Runtime。

它不做多行业 Pack，不做云端 Runtime，不做行业市场。

## 3. Precondition

进入 `v0.7.0` 前，必须先完成 `v0.6.1`：

1. release closeout 与版本事实对齐；
2. Acceptance Gate 成为 Done 决策入口；
3. Completion Commit 明确权威写入顺序；
4. Done 后 Audit 仍保持独立流程；
5. release audit certification 可追溯。

如果这些前置条件未完成，`v0.7.0` 不能进入正式开发。

## 4. Issue Preview

| Issue | Title | Dependency | Priority | First executable |
| --- | --- | --- | --- | --- |
| `V070-001` | Projection Surface Contract and Console IA | 无 | P0 | 是 |
| `V070-002` | Projection Query API and Unified Read Models | `V070-001` | P0 | 否 |
| `V070-003` | Project Home and Next Action Console | `V070-002` | P0 | 否 |
| `V070-004` | Spec Workbench Projection Surface | `V070-002` | P0 | 否 |
| `V070-005` | Task Workbench Execution and Acceptance Surface | `V070-002`, `V070-003` | P0 | 否 |
| `V070-006` | Event Timeline and Evidence Graph | `V070-002`, `V070-005` | P0 | 否 |
| `V070-007` | Acceptance and Delivery Surface | `V070-005`, `V070-006` | P0 | 否 |
| `V070-008` | Audit Read-only Surface and Audit Trigger Visibility | `V070-006`, `V070-007` | P1 | 否 |
| `V070-009` | Command Surface Runtime API Bridge | `V070-001`, `V070-002` | P0 | 否 |
| `V070-010` | Advanced Runtime Diagnostics Surface | `V070-002` | P1 | 否 |
| `V070-011` | Desktop View Models and Browser Preview Regression | `V070-003`, `V070-004`, `V070-005`, `V070-007`, `V070-008`, `V070-009` | P1 | 否 |
| `V070-012` | Project OS Console Acceptance and Release Readiness | `V070-011` | P1 | 否 |

## 5. Development Tasks

### V070-001 - Projection Surface Contract and Console IA

目标：

定义 Projection Surface 和 Project OS Console 的页面职责、事实边界和命令回流规则。

架构合同：

- [../architecture/011-projection-surface-console-ia-v1.md](../architecture/011-projection-surface-console-ia-v1.md)

范围：

- 定义 Projection Surface contract；
- 冻结 Console 一级信息架构；
- 定义 Project Home / Spec Workbench / Task Workbench / Audit / Files / Advanced 的职责；
- 定义 read model 与 view model 的边界；
- 定义 Command Surface 只能回流 Runtime API；
- 定义 UI forbidden writes；
- 明确 Projection 只读，不成为 authority。

验收标准：

- 每个 Console 页面都有唯一职责；
- UI 不能直接修改 `.agentflow/spec/**`、`.agentflow/tasks/**`、`.agentflow/events/**`；
- Command Surface 不直接改状态；
- Projection / View Model / Runtime API 边界明确；
- 后续任务能按该合同实现页面。

非目标：

- 不实现完整页面；
- 不新增行业 Pack；
- 不迁移底层事实源。

### V070-002 - Projection Query API and Unified Read Models

目标：

提供 Project OS Console 需要的统一只读查询模型。

实现状态：

- 已完成；
- 新增 `get_projection_surface_catalog`，作为 Console 进入各类 Projection Read Model 的统一目录；
- 目录覆盖 `requirement-intake`、`spec-preview`、`spec-loop`、`project-home`、`task-workbench`、`delivery-package`、`runtime-health`、`audit-surface`；
- 每个 read model entry 都声明 query 名称、对象类型、对象 ID、projection path、source refs、freshness、missing facts，并保持 `authority=false`。

范围：

- 定义 project read model；
- 定义 spec loop read model；
- 定义 issue / run / session read model；
- 定义 verification / evidence / acceptance read model；
- 定义 delivery read model；
- 定义 audit read model；
- 定义 query API；
- 定义 projection freshness / missing fact / stale fact 表达。

验收标准：

- Console 页面不直接读散落事实文件；
- query API 能返回 project / spec / task / acceptance / delivery / audit 的统一摘要；
- missing / stale / conflict 状态可被 UI 表达；
- Projection 只读；
- 测试覆盖 read model 组装。

非目标：

- 不实现写命令；
- 不做复杂订阅系统；
- 不引入数据库迁移。

### V070-003 - Project Home and Next Action Console

目标：

把工作台做成真正的 Project Home。

实现状态：

- 已完成；
- 工作台首屏展示项目阶段、下一步、活跃任务、完成判断和 Project Brain 健康；
- 新增明确的 `Command Surface` 区块，入口只调用 Runtime API / 页面跳转，不直接修改事实文件；
- Agent 入口补齐 `Goal Agent`、`Spec Agent`、`Work Agent`、`Delivery Agent`、`Audit Agent` 五类角色，帮助用户判断下一步由谁接管。

范围：

- 展示项目阶段；
- 展示下一步建议；
- 展示当前活跃任务；
- 展示 blockers；
- 展示 Goal / Plan / Decisions 摘要；
- 展示 Acceptance / Delivery / Audit 总览；
- 展示下一步为什么是这个动作；
- 提供 Command Surface 入口，但不直接改事实。

验收标准：

- 用户第一屏能看懂项目当前阶段；
- 下一步卡片能解释原因和前置条件；
- 活跃任务与阻塞项可见；
- Goal / Plan / Decisions 健康状态可见；
- 主操作通过 Command Surface 回流 Runtime API。

非目标：

- 不做完整任务时间线；
- 不做 Goal 文档编辑器；
- 不执行命令。

### V070-004 - Spec Workbench Projection Surface

目标：

让 Spec Loop 状态成为可读、可追踪、可确认的工作台视图。

实现状态：

- 已完成；
- 新增 Desktop `需求` 一级栏目，作为只读 Spec Workbench；
- Tauri 新增 `load_spec_workbench_projection`，从 Projection Read Model 读取 requirement intake、spec preview 和 spec loop；
- 页面展示 intake / classification / context / boundary / route / preview / confirmation / materialization 八阶段；
- 页面明确区分 preview artifact、project authority、issue authority、derived projection 和 runtime action proposal；
- Browser Preview 补齐 Spec Workbench mock projection，真实 Tauri 客户端继续读取本地 projection。

范围：

- 展示 intake / classification / context / boundary / route / preview / confirmation / materialization；
- 展示 preview artifact 与 authority artifact 的关系；
- 展示 source requirement；
- 展示 generated project / issues；
- 展示 Spec-to-Action Proposal 摘要；
- 显示哪些状态仍需人类确认。

验收标准：

- 用户能看懂需求如何变成 spec issue；
- preview 与 authority 不混淆；
- 未确认状态不会被显示成可执行；
- Spec Workbench 只读 projection；
- 相关命令通过 Command Surface。

非目标：

- 不重新实现 Spec Loop；
- 不直接写 `docs/requirements/**`；
- 不直接写 `.agentflow/spec/**`。

### V070-005 - Task Workbench Execution and Acceptance Surface

目标：

把任务页做成执行、验证、证据、验收和完成写入的主工作面。

实现状态：

- 已完成；
- 任务页右侧 `执行与交付` 面板新增 `验收门` 和 `完成写回` 槽位；
- Desktop TaskProjection 类型接入 `acceptance` 投影，展示 outcome、sub-gates、判定记录、evidence、closeout proof；
- Browser Preview mock 补齐 pending / needs-human-decision / passed 三类验收状态；
- Smoke 覆盖 Acceptance Gate、Completion Commit、交付槽位和审计独立入口；
- 页面只读展示 Projection/View Model，不直接执行 Build Agent，不写任务 authority。

范围：

- 展示 issue tree；
- 展示 issue 状态；
- 展示 active run / work session；
- 展示 preflight / lock / queue；
- 展示 verification run；
- 展示 evidence pack；
- 展示 Acceptance Gate；
- 展示 Completion Commit；
- 展示 delivery record；
- 展示 optional audit trigger status。

验收标准：

- 用户能看懂任务为什么能开始或不能开始；
- 用户能看懂验证是否通过；
- 用户能看懂 evidence 是否完整；
- 用户能看懂 acceptance 是否 passed / rejected / needs-human-decision；
- 用户能看懂 Done 是如何写入的；
- Audit 不被显示成任务 Done 的默认步骤。

非目标：

- 不执行 Build Agent；
- 不写审计报告；
- 不直接改 issue 状态。

### V070-006 - Event Timeline and Evidence Graph

目标：

让项目事实变化和证据链可追溯。

实现状态：

- 已完成；
- 任务页主列新增 `事件时间线 / 证据图` 只读区域；
- Event Timeline 从 TaskProjection timeline events 聚合，按状态、执行者、时间和 artifact refs 展示；
- Evidence Graph 展示 requirement / spec issue / context / run / verification / evidence / acceptance / delivery 主链；
- Audit 作为独立旁支展示，不混入 Done 主链；
- 缺失事实显示为 `未生成` / `等待` / `独立审计未触发`，timeline 不作为 authority。

范围：

- 建立 Event Timeline；
- 建立 Evidence Graph；
- 支持 requirement -> spec -> issue -> run -> verification -> evidence -> acceptance -> delivery；
- 支持 audit finding 作为独立分支；
- 支持事件缺失 / 证据缺失 / projection stale 的显示；
- 支持按 project / issue / run 过滤。

验收标准：

- 用户能解释状态为什么变化；
- evidence graph 能追踪每个 delivery 的来源；
- audit finding 不被混入 Done 主链；
- 缺失事实被明确显示；
- timeline 不作为 authority。

非目标：

- 不做完整图数据库；
- 不做复杂可视化编辑；
- 不让用户手工改 evidence graph。

### V070-007 - Acceptance and Delivery Surface

目标：

把验收和交付做成一等只读表面。

实现状态：

- 已完成；
- 任务页主列新增 `验收与交付表面`；
- Surface 明确区分 Verification、Evidence、Acceptance、Completion、Delivery、Release Readiness；
- Acceptance 展示 decision、sub-gates、failed reason、repair suggestion；
- Completion 展示 Completion Commit、merge commit、writeback state；
- Delivery 展示 delivery summary、public delivery links、缺失公开记录；
- 明确 `Projection refresh 不是 authority`，无 audit 时也能完成 delivery 阅读。

范围：

- 展示 Acceptance Gate 子 gate；
- 展示 acceptance decision；
- 展示 failed reason / repair suggestion；
- 展示 Completion Commit 摘要；
- 展示 delivery record；
- 展示 public delivery links；
- 展示 release readiness。

验收标准：

- 用户能区分 verification、evidence、acceptance、completion、delivery；
- Acceptance failed 时能看到具体原因；
- Delivery record 能追溯 evidence；
- Projection refresh 不被显示成 authority；
- 无 audit 时也能完成 delivery 阅读。

非目标：

- 不发布 release；
- 不生成 audit report；
- 不改 delivery 事实。

### V070-008 - Audit Read-only Surface and Audit Trigger Visibility

目标：

把 Audit 保持成独立只读表面，同时显示 Done 后是否需要审计。

范围：

- 展示 audit issue；
- 展示 audit status；
- 展示 audit findings；
- 展示 evidence map；
- 展示 traceability；
- 展示 optional audit trigger evaluation；
- 展示 no audit / audit queued / audit running / audit completed。

验收标准：

- Audit Surface 不执行审计；
- Audit Surface 不修改 Work Loop facts；
- Done 后 no audit 是合法状态；
- audit queued 不等于 audit passed；
- findings 能追溯到 evidence。

非目标：

- 不自动创建 audit；
- 不写 audit report；
- 不把 Audit 放回 Work Loop。

实现状态：

- 已完成；
- Task detail 新增 `Audit Read-only Surface`，作为 Work Loop Done 主链之外的独立旁支；
- 展示 Audit Trigger / Audit Status / Findings / Evidence Map / Traceability / Boundary；
- 支持 `no audit`、`audit queued`、`audit running`、`audit completed` 四类只读状态；
- 明确 `Audit Surface 不修改 Work Loop facts`、`audit queued 不等于 audit passed`、`Done 后 no audit 是合法状态`；
- Browser Preview mock 已覆盖 queued / running / completed 审计状态；
- Browser Preview smoke 已覆盖审计只读表面和样式入口。

### V070-009 - Command Surface Runtime API Bridge

目标：

让用户操作通过统一 Command Surface 回流 Runtime API。

范围：

- 定义 command surface action；
- 支持 approveSpec / requestFix / startWork / requestAudit / acceptDelivery / reopenIssue / createFollowUp 等命令入口；
- 将 UI action 转成 Runtime Command；
- 将 Runtime Command 转成 Action Proposal；
- 显示 command pending / accepted / rejected / queued / needs-human-decision。

验收标准：

- UI action 不直接写事实源；
- 每个 command 都能追溯到 Runtime API；
- 被 rejected 的 command 不改变事实；
- command 状态能在 Console 中显示；
- Command Surface 不绕过 Arbitration。

非目标：

- 不实现所有行业命令；
- 不做云端 command queue；
- 不引入 Message Bus。

### V070-010 - Advanced Runtime Diagnostics Surface

目标：

把底层 runtime 事实放到 Advanced，只服务调试和审计追溯，不干扰主工作流。

范围：

- 展示 projection freshness；
- 展示 runtime status；
- 展示 event replay summary；
- 展示 provider / session 摘要；
- 展示 role policy / boundary 摘要；
- 展示 missing facts / stale projection / conflict diagnostics；
- 提供只读 JSON detail。

验收标准：

- Advanced 不成为普通用户主入口；
- Advanced 只读；
- 可定位 projection stale / missing event / conflict；
- 不暴露直接写事实按钮；
- 调试信息不污染 Project Home。

非目标：

- 不做完整开发者工具平台；
- 不执行修复命令；
- 不替代 release audit。

### V070-011 - Desktop View Models and Browser Preview Regression

目标：

把 Projection Surface 映射为稳定 Desktop View Models，并用 browser preview 回归证明页面可用。

范围：

- 定义 Project Home view model；
- 定义 Spec Workbench view model；
- 定义 Task Workbench view model；
- 定义 Event Timeline / Evidence Graph view model；
- 定义 Acceptance / Delivery / Audit view model；
- 更新 browser preview data；
- 增加 regression tests。

验收标准：

- View Model 不直接写事实；
- browser preview 覆盖核心页面；
- 页面能展示 loading / empty / missing / stale / conflict / ready / done；
- 文案能解释状态和下一步；
- `npm --prefix apps/desktop run build` 应作为正式 SPEC 验证方向。

非目标：

- 不重做设计系统；
- 不引入复杂前端状态库；
- 不实现行业 Surface。

### V070-012 - Project OS Console Acceptance and Release Readiness

目标：

证明软件开发场景可以通过 Project OS Console 完成可读闭环。

范围：

- 增加 Console acceptance；
- 覆盖 Project Home；
- 覆盖 Spec Workbench；
- 覆盖 Task Workbench；
- 覆盖 Acceptance / Delivery；
- 覆盖 Audit read-only；
- 覆盖 Command Surface reject / accepted 状态；
- 生成 release readiness evidence。

验收标准：

- 用户能从 Console 理解 project -> spec -> task -> work -> acceptance -> delivery -> audit read-only；
- Console 不直接写事实；
- Command Surface 只经 Runtime API；
- Projection stale / missing / conflict 能被显示；
- 软件开发场景可用；
- v0.8.0 可以基于此进入 Pack System。

非目标：

- 不发布 `v0.7.0` tag；
- 不做行业 Pack；
- 不做云端部署。

## 6. Suggested Milestones

### Milestone 1 - Projection Contract

包含：

- `V070-001`
- `V070-002`

完成后，Console 有统一的 read model / view model 边界。

### Milestone 2 - Core Console

包含：

- `V070-003`
- `V070-004`
- `V070-005`

完成后，Project Home、Spec Workbench、Task Workbench 成为核心操作面。

### Milestone 3 - Traceability And Delivery

包含：

- `V070-006`
- `V070-007`
- `V070-008`

完成后，证据、交付、审计阅读形成闭环。

### Milestone 4 - Command And Diagnostics

包含：

- `V070-009`
- `V070-010`

完成后，用户操作能通过 Runtime API 合法回流，Advanced 只做诊断。

### Milestone 5 - Acceptance

包含：

- `V070-011`
- `V070-012`

完成后，`v0.7.0` 可以判断为软件开发场景第一版 Project OS Console。

## 7. Completion Criteria

`v0.7.0` 完成时，必须满足：

- Projection Surface contract 稳定；
- Project Home 可解释当前阶段和下一步；
- Spec Workbench 可追踪需求到 issue；
- Task Workbench 可解释执行、验证、证据、验收、交付；
- Event Timeline / Evidence Graph 可追溯事实；
- Acceptance / Delivery Surface 可读；
- Audit Surface 独立只读；
- Command Surface 不绕过 Runtime API；
- Advanced 只读诊断；
- Desktop browser preview 和回归测试覆盖核心视图；
- Console 不直接写事实源。

## 8. Verification Direction

正式 SPEC 生成时，应至少覆盖：

- `npm --prefix apps/desktop run build`；
- Desktop browser preview / smoke；
- projection read model tests；
- command surface rejected / accepted tests；
- workflow regression tests；
- `git diff --check`。

## 9. Next Step

下一步不是直接执行这些任务。

正确顺序是：

```text
SPEC Draft Preview
-> Project Preview
-> Issues Preview
-> Human Confirmation
-> docs/requirements/**
-> .agentflow/spec/projects/**
-> .agentflow/spec/issues/**
```

确认前，本文件只是 `v0.7.0` 的开发前置规划。
