# 038.1 - AgentFlow Project Operating System Runtime Foundation Project And Issues V1

创建日期：2026-06-18
执行者：Codex

## Purpose

本文把 [038-agentflow-project-operating-system-runtime-foundation-v1.md](/Users/mac/Documents/AgentFlow/docs/requirements/038-agentflow-project-operating-system-runtime-foundation-v1.md) 继续压成一份可执行拆解稿。

目标是先得到：

- 一个正式的 Project 草案；
- 一组可落地的 Issue 草案；
- 明确执行顺序和依赖关系；
- 先完成规划，不写 `.agentflow`，不生成本地 spec facts。

## 当前规则

本文件当前只作为需求拆解草案使用。

### 允许

- 定义 Project Preview
- 定义 Issues Preview
- 定义依赖关系
- 定义执行顺序

### 不允许

- 不写 `.agentflow/spec/**`
- 不生成 `.agentflow/spec/projects/*.json`
- 不生成 `.agentflow/spec/issues/*.json`
- 不写 `.agentflow/events/**`
- 不直接进入 Build Agent 执行

## 来源需求

- [038-agentflow-project-operating-system-runtime-foundation-v1.md](/Users/mac/Documents/AgentFlow/docs/requirements/038-agentflow-project-operating-system-runtime-foundation-v1.md)

## 一句话目标

用一个 Project，把 AgentFlow 的底层从“任务工具集合”推进到“Project Operating System runtime foundation”。

## Project Preview

### Project ID

`project-agentflow-project-operating-system-runtime-foundation-v1`

### Project Title

AgentFlow Project Operating System Runtime Foundation

### Project Goal

把 AgentFlow 的底层 authority、workflow、event、projection、agent capability、provider adapter 和任务页状态流统一到一套稳定的项目运行时底座。

### Project Scope

- Project Brain / Constitution authority
- Agent role / skill / handoff runtime binding
- Workflow schema runtime alignment
- Event store / replay / projection rebuild
- Provider session authority demotion
- Task center timeline refactor
- Audit / Delivery / Completion 闭环

### Project Non-goals

- 不新增新的聊天驱动多 Agent 编排模式
- 不恢复旧 `input / execute / output` 路径
- 不在本阶段扩展更多产品页面
- 不引入外部框架替换 AgentFlow 底层

### Project Deliverables

- runtime authority 收口
- workflow schema 收口
- event / projection 收口
- provider session 降级为执行事实
- task page 状态流产品化
- audit / delivery / completion 闭环

## 建议执行阶段

### Phase A - Constitution / Capability / Workflow

目标：

- 先把 authority 和 runtime binding 定死。

包含：

- `AF-POS-001`
- `AF-POS-002`
- `AF-POS-003`

### Phase B - Event / Projection / Provider

目标：

- 让 runtime 真的跑起来，读模型稳定可重建。

包含：

- `AF-POS-004`
- `AF-POS-005`
- `AF-POS-006`

### Phase C - Surface / Closure

目标：

- 让用户在任务页和项目页看到完整状态流，并完成最后闭环。

包含：

- `AF-POS-007`
- `AF-POS-008`

## Issues Preview

### AF-POS-001 - Project Constitution And Project Brain Authority

#### 目标

- 明确 Project 是最高聚合根；
- 定义或补足 Project Brain authority；
- 固化 Goal / Plan / Decision / Completion 的上游关系；
- 清理现有代码和文档里仍然把 Issue / Run / Session 当作顶层 authority 的地方。

#### 范围

- `docs/architecture/**`
- `docs/foundation/**`
- `crates/spec/**`
- `crates/state/**`
- `crates/projection/**`

#### 非目标

- 不改 UI
- 不接 provider session

### AF-POS-002 - Agent Capability Runtime Binding

#### 目标

- 把 Goal / Spec / Work / Audit / Delivery 的 role / skill / handoff 模型正式绑定到 runtime；
- 明确 stage -> role -> skill pack 映射；
- 明确 ownership transfer 和 bounded capability call 的边界。

#### 范围

- `docs/architecture/**`
- `crates/agent-manual/**`
- `crates/agent-dispatcher/**`
- `crates/workflow-core/**`
- `crates/workflow-runtime/**`

#### 非目标

- 不启动真实 provider 执行
- 不改 task page 布局

### AF-POS-003 - Workflow Schema Runtime Alignment

#### 目标

- 固化 `project / work / audit / delivery` 四类 workflow schema；
- 把状态、guard、action、terminal state 收口到 runtime；
- 停止页面刷新或 provider session 直接驱动业务状态。

#### 范围

- `crates/workflow-core/**`
- `crates/workflow-runtime/**`
- `crates/state/**`
- `docs/architecture/**`

#### 非目标

- 不做 projection 页面展示
- 不改公开交付逻辑

### AF-POS-004 - Event Store Envelope And Replay

#### 目标

- 统一 event envelope；
- 明确 event taxonomy；
- 支持 correlation / causation / idempotency；
- 支持 replay / checkpoint / resume 的事实层能力。

#### 范围

- `crates/event-store/**`
- `crates/workflow-runtime/**`
- `crates/task-artifacts/**`
- `docs/architecture/**`

#### 非目标

- 不改具体 UI 展示
- 不改 delivery 页面

### AF-POS-005 - Projection Rebuild And Read Model Unification

#### 目标

- 让 task / project / audit / delivery 视图统一来自 projection；
- 停止 UI 直接读取散落事实文件；
- 建立 current / history / future 三类读模型语义。

#### 范围

- `crates/projection/**`
- `crates/state/**`
- `apps/desktop/src/features/**`
- `apps/desktop/src/types/**`

#### 非目标

- 不先做最终视觉优化
- 不做 provider session 深度展示

### AF-POS-006 - Provider Session Demotion And Dispatcher Alignment

#### 目标

- provider session 从业务 authority 降级为执行事实；
- dispatcher 只负责 launch / claim / session lifecycle；
- provider 状态只能通过 event -> projection 进入 UI。

#### 范围

- `crates/agent-dispatcher/**`
- `crates/mcp/**`
- `crates/workflow-runtime/**`
- `crates/projection/**`

#### 非目标

- 不新增新的 provider 类型
- 不扩大 session 自治能力

### AF-POS-007 - Task Center State Timeline Refactor

#### 目标

- 任务页右侧改成状态流 + 事件流主视图；
- 当前状态显示实时信息；
- 已完成状态显示历史日志；
- 未来状态只显示等待；
- Work / Audit / Delivery 信息统一回到任务页。

#### 范围

- `apps/desktop/src/App.tsx`
- `apps/desktop/src/AppShell.css`
- `apps/desktop/src/features/**`
- `apps/desktop/src/browserPreviewData.ts`

#### 非目标

- 不重做全站视觉语言
- 不新增自由聊天入口

### AF-POS-008 - Audit Delivery Completion Closure

#### 目标

- Work / Audit / Delivery / Completion 形成闭环；
- 公开交付进入 PR/MR body、`CHANGELOG.md`、release notes；
- 不再保留本地 `.agentflow/tasks/<issue-id>/delivery/**` 作为公开交付主路径；
- Project completion 判断回到 Goal Recheck 和 Completion Decision。

#### 范围

- `crates/audit/**`
- `crates/release/**`
- `crates/projection/**`
- `crates/state/**`
- `apps/desktop/src/features/output/**`
- `apps/desktop/src/features/state/**`

#### 非目标

- 不引入新的发布平台
- 不新增独立 release UI 流程

## 依赖关系

### 强依赖

- `AF-POS-002` 依赖 `AF-POS-001`
- `AF-POS-003` 依赖 `AF-POS-001`、`AF-POS-002`
- `AF-POS-004` 依赖 `AF-POS-003`
- `AF-POS-005` 依赖 `AF-POS-004`
- `AF-POS-006` 依赖 `AF-POS-002`、`AF-POS-003`、`AF-POS-004`
- `AF-POS-007` 依赖 `AF-POS-005`、`AF-POS-006`
- `AF-POS-008` 依赖 `AF-POS-005`、`AF-POS-006`、`AF-POS-007`

## 建议执行顺序

建议按以下顺序推进：

1. `AF-POS-001`
2. `AF-POS-002`
3. `AF-POS-003`
4. `AF-POS-004`
5. `AF-POS-005`
6. `AF-POS-006`
7. `AF-POS-007`
8. `AF-POS-008`

说明：

- `AF-POS-002` 和 `AF-POS-003` 理论上可局部并行，但不建议一开始并行推进；
- `AF-POS-006` 可以在 `AF-POS-005` 之前部分准备，但正式收口最好仍放在 event / projection 统一之后；
- `AF-POS-007` 必须晚于 projection 和 dispatcher 收口，否则页面会继续读到不稳定事实。

## Project 验收标准

- [ ] Project / Contract / Workflow / Event / Projection 五层 authority 收口清晰。
- [ ] Goal / Spec / Work / Audit / Delivery 的 role / skill / handoff 可稳定映射到 runtime。
- [ ] Project Flow / Work Flow / Audit Flow / Delivery Flow 关系清晰且可执行。
- [ ] Event Store 支持 replay / rebuild。
- [ ] Projection 成为 Task Page / Project Page 的唯一读模型。
- [ ] provider session 不再被误用为业务 authority。
- [ ] Task Page 右侧成为状态流主视图。
- [ ] Audit / Delivery / Completion 能闭环。

## 后续动作

当前文档只完成到“Project + Issues Preview”。

下一步如果确认继续，可以进入：

1. 把这组 Project / Issues 草案正式写入 `.agentflow/spec/**`；
2. 先生成 `SpecProject Preview`；
3. 再生成 `SpecIssue Preview`；
4. 最后等待用户确认后写入事实源。
