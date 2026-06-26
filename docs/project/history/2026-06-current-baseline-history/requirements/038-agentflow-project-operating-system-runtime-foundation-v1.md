# 038 - AgentFlow Project Operating System Runtime Foundation V1

创建日期：2026-06-18
执行者：Codex

## 用户目标

把 AgentFlow 从当前“功能逐步拼起来的本地 Agent 工具”，重构为一套稳定的 Project Operating System。

这套底座必须满足：

- Project 是最高聚合根；
- Contract 是执行 authority；
- Workflow 推状态；
- Event Store 记历史；
- Projection 提供 UI 只读模型；
- Agent / Provider / Surface 都可替换；
- 本地 runtime 与公开交付严格分层。

## 背景

当前项目已经完成多轮：

- 旧 input / execute / output 路径退役；
- runtime facts 收口到 `.agentflow/spec/**`、`.agentflow/events/**`、`.agentflow/tasks/**`、`.agentflow/audit/**`、`.agentflow/projections/**`；
- Desktop 任务中心、Build loop、外部 provider 启动链初步打通。

但底层仍然存在三个核心问题：

1. 架构 authority 还没有形成一份统一总蓝图；
2. Agent 角色、技能、handoff、workflow、event、projection 之间的边界尚未完全定死；
3. 当前 crates、runtime、task page 仍然带有历史阶段的收口痕迹，后续继续开发会越来越难维护。

因此本需求的目标不是继续堆功能，而是先把底层架构正式压成一套可执行的 runtime foundation。

## 依赖文档

本需求以下列文档为输入基线：

- [docs/architecture/001-project-operating-system-v1.md](/Users/mac/Documents/AgentFlow/docs/architecture/001-project-operating-system-v1.md)
- [docs/architecture/002-agent-capability-matrix-v1.md](/Users/mac/Documents/AgentFlow/docs/architecture/002-agent-capability-matrix-v1.md)
- [docs/architecture/003-workflow-schema-v1.md](/Users/mac/Documents/AgentFlow/docs/architecture/003-workflow-schema-v1.md)
- [docs/architecture/004-event-and-projection-model-v1.md](/Users/mac/Documents/AgentFlow/docs/architecture/004-event-and-projection-model-v1.md)
- [docs/architecture/current-module-boundaries.md](/Users/mac/Documents/AgentFlow/docs/architecture/current-module-boundaries.md)
- [docs/architecture/mcp-provider-adapter.md](/Users/mac/Documents/AgentFlow/docs/architecture/mcp-provider-adapter.md)

## 核心原则

1. 先定宪法，再改实现。
2. Project 是最高聚合根，Issue / Run / Session 都不是顶层 authority。
3. Workflow 决定状态推进，Agent 只执行当前阶段动作。
4. Event 是唯一历史真相，Projection 是唯一 UI 读模型。
5. Provider 只做外部适配，不参与业务 authority。
6. 本地 `.agentflow` 只保存内部事实，公开交付进入 PR/MR、`CHANGELOG.md`、release notes。
7. 开发顺序必须先文档、再 schema、再 runtime、再 UI。

## 范围

本需求覆盖以下内容：

- Project Operating System 总蓝图落地为正式开发基线；
- Agent Role / Skill / Handoff 模型落地；
- Project / Work / Audit / Delivery 四类 workflow schema 落地；
- Event / State / Projection 模型落地；
- crates 结构与命名收口到新蓝图；
- Workflow Runtime、Event Store、Projection、Dispatcher、MCP 按新蓝图对齐；
- Task Page / Project Page 改成状态流读模型；
- Audit / Delivery / Completion 进入闭环。

## 非目标

- 不在本需求里继续新增产品功能页。
- 不在本需求里加入新的自由聊天编排模式。
- 不引入外部现成框架替换 AgentFlow 底层。
- 不把 provider session 直接当作业务 authority。
- 不恢复旧 `.agentflow/input/**`、`.agentflow/execute/**`、`.agentflow/output/**`。

## 目标架构

后续实现必须对齐以下底层分层：

```text
Project Constitution Layer
-> Contract Layer
-> Workflow Layer
-> Fact Layer
-> Projection Layer

Agent Layer
Provider Layer
Surface Layer
```

长期稳定的核心主链：

```text
Requirement
-> Goal / Plan / Decisions
-> SpecProject / SpecIssue
-> Project Flow
-> Work Flow
-> Audit Flow
-> Delivery Flow
-> Goal Recheck / Completion
```

## 目标模块边界

目标 crates 方向如下：

- `project-brain` 或等价的 Project Brain 模块
- `spec`
- `panel`
- `workflow-core`
- `workflow-runtime`
- `event-store`
- `projection`
- `state`
- `agent-dispatcher`
- `mcp`
- `task-artifacts`
- `audit`
- `release`
- `cli`
- `desktop`

说明：

- 如果暂时不新增 `project-brain` crate，也必须在现有代码里明确该层的 authority 与未来位置。
- `task-loop` 旧语义不能继续混用，必须收口成明确的 project-level / work-level 语义。

## 阶段拆解

本需求拆成三个大阶段。

### 阶段 A - 宪法与 Schema 固化

目标：

- 固化总蓝图；
- 固化 Agent Capability Matrix；
- 固化 Workflow Schema；
- 固化 Event / Projection Model。

交付物：

- 4 份架构总文档；
- `docs/architecture/README.md`；
- `docs/README.md` 同步入口。

状态：

- 本阶段当前已完成。

### 阶段 B - Runtime Spine 重构

目标：

- 把 schema 和 authority 落到 crates；
- 让 workflow runtime 真正成为状态 authority；
- 让 event store 成为唯一历史事实；
- 让 projection 成为唯一 UI 读模型。

范围：

- `workflow-core`
- `workflow-runtime`
- `event-store`
- `projection`
- `state`
- `task-artifacts`
- `agent-dispatcher`
- `mcp`

预期结果：

- Project Flow / Work Flow / Audit Flow / Delivery Flow 可独立推进；
- 事件可以 replay；
- projection 可以 rebuild；
- provider session 只是执行事实，不是状态 authority。

### 阶段 C - Product Surface 收口

目标：

- 把 Task Page / Project Page 改成状态流产品表面；
- Work / Audit / Delivery / Completion 在 UI 上统一可见；
- 交付与审计不再是分散的旧栏目逻辑。

范围：

- Desktop Task Page
- Desktop Project Page
- audit / delivery summary read model
- CLI / Browser Preview 对齐新 projection

预期结果：

- 当前状态显示实时事件流；
- 已完成状态显示历史日志；
- 未来状态只显示等待，不显示假日志；
- 左侧 project / issue 树来自 projection，而不是直接读 spec 或临时事实。

## 建议开发切片

本需求建议拆成以下切片。

### 038.1 - Project Constitution and Project Brain Authority

目标：

- 定义或补足 Project Brain 层；
- 明确 Project / Goal / Plan / Decision / Completion 的 authority；
- 固化 Project 是最高聚合根。

### 038.2 - Agent Capability Runtime Binding

目标：

- 把 role / skill / handoff 模型绑定到 runtime；
- 让 workflow stage 能唯一映射到 role；
- 让 dispatcher 基于 role 和 skill pack 启动执行。

### 038.3 - Workflow Schema Runtime Alignment

目标：

- 把四类 workflow schema 落地；
- 把状态、guard、action、terminal state 统一到 runtime；
- 停止页面或 provider 直接驱动状态推进。

### 038.4 - Event Store and Projection Rebuild

目标：

- 统一 event envelope；
- 明确 event taxonomy；
- 支持 replay / rebuild；
- 让 task / project / audit / delivery view 全部来自 projection。

### 038.5 - Task Center State Timeline Refactor

目标：

- 把任务右侧详情页重构为状态流 + 事件流；
- Work / Audit / Delivery 统一回到任务页；
- current / history / future 三种展示语义固定。

### 038.6 - Provider Session Demotion

目标：

- provider session 从 authority 降级为执行事实；
- Codex / Claude / 其他 provider 保持 adapter 地位；
- provider 状态只通过 event -> projection 进入 UI。

### 038.7 - Delivery and Completion Closure

目标：

- Work / Audit / Delivery / Completion 闭环；
- 公开交付只进入 PR/MR / `CHANGELOG.md` / release notes；
- 不再往 `.agentflow/tasks/<issue-id>/delivery/**` 回写公开交付。

## 数据来源

后续实现必须只从以下几类 authority 读取：

- `docs/product/**`
- `docs/foundation/**`
- `docs/architecture/**`
- `docs/requirements/**`
- `.agentflow/spec/**`
- `.agentflow/events/**`
- `.agentflow/tasks/**`
- `.agentflow/audit/**`
- `.agentflow/projections/**`

禁止把以下内容当作主 authority：

- provider session snapshot
- Browser Preview mock
- UI local state
- prompt text
- 某个外部 Agent 的对话历史

## 交互边界

### Human

人类只在以下节点拥有确认权：

- Goal Draft confirm
- Plan Draft confirm
- Project Entry confirm
- Scope change confirm
- Final acceptance confirm

### Agent

Agent 只在被授权的 workflow stage 内执行。

### Provider

Provider 只负责：

- launch
- poll
- cancel
- logs

Provider 不拥有：

- 任务排序权
- Project 状态 authority
- Issue 状态 authority

## 验收标准

- [ ] AgentFlow 底层架构已形成统一总蓝图，不再依赖散落规则。
- [ ] Project 是最高聚合根，Issue / Run / Session 不再被误用为顶层 authority。
- [ ] Goal / Spec / Work / Audit / Delivery 的角色、职责、技能和 handoff 边界清晰。
- [ ] Project / Work / Audit / Delivery 四类 workflow schema 固定。
- [ ] Event Store 成为唯一历史真相源。
- [ ] Projection / State 成为 Task Page / Project Page 的唯一读模型。
- [ ] Provider session 不再直接驱动业务状态。
- [ ] 本地运行事实与公开交付记录严格分层。
- [ ] crates 职责与文档蓝图一致。
- [ ] 后续需求可以从本需求切片继续拆成 project + issues。

## 验证命令

文档阶段至少要求：

- `git diff --check`

进入代码阶段后至少要求：

- `cargo test`
- `npm --prefix apps/desktop run build`
- `git diff --check`

具体命令以对应切片需求为准。

## PR 说明要求

后续从本需求派生的 PR 必须说明：

- 当前切片属于 038 的哪一阶段；
- 本次是否改动 authority 边界；
- 本次是否改动 workflow schema；
- 本次是否改动 event / projection；
- 本次是否影响 Task Page / Project Page 的读模型；
- 本次是否涉及 provider session 语义变更。

## Codex 执行指令

执行本需求时，必须遵守：

1. 先从对应切片文档执行，不要直接跳整包。
2. 一次只处理一类 authority 问题。
3. 不在同一 PR 里同时混改：
   - workflow schema
   - provider adapter
   - UI 读模型
4. 不允许新增绕过 Projection 的页面读取路径。
5. 不允许新增绕过 Workflow Runtime 的状态写入路径。
6. 不允许恢复旧 `input / execute / output` 模型。

## 完成定义

本需求完成，不是指“写完文档”，而是指：

- 架构文档、runtime、projection、task page、provider adapter、audit / delivery / completion 已全部按总蓝图收口；
- 新需求和后续 issue 拆解可以稳定建立在这套底座之上；
- AgentFlow 后续再扩展 Goal Agent、Delivery Agent、更多 provider、更多 surface 时，不需要再回头重做底层 authority。
