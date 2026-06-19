# Current Module Boundaries

创建日期：2026-06-02
最后更新：2026-06-20
执行者：Codex

## 结论

AgentFlow 当前底层已经开始收口到一套 Project Operating System 架构。

公开需求先写到 `docs/requirements/**`。
项目大脑文档写到 `docs/projects/**`。
内部执行合同写到 `.agentflow/spec/**`。
运行事实写到 `.agentflow/events/**`、`.agentflow/projections/**`、`.agentflow/tasks/**` 和 `.agentflow/audit/**`。

当前事实流：

```text
docs/requirements/**
  -> docs/projects/**
  -> .agentflow/spec/**
  -> .agentflow/events/task-events.jsonl
  -> .agentflow/projections/**
  -> .agentflow/tasks/<issue-id>/**
  -> PR/MR / CHANGELOG / release notes
```

下面这些目录和模块已经退出活跃架构：

- `.agentflow/input/**`
- `.agentflow/execute/**`
- `.agentflow/output/**`
- `.agentflow/goal-tree/**`
- 旧 `core` 兼容层
- 旧 `workflow-events` crate

## Workspace and Desktop Entry

### Project Workspace Manager

负责：

- 准备本地项目工作区
- 创建或复用 `.agentflow/`
- 处理项目去重、移除、本地模式元数据
- 维护 `.git/info/exclude` 中的本地保护

不负责：

- 执行任务
- 调用模型
- 创建远端对象
- 删除用户源码

实现位置：

- `apps/desktop/src-tauri/src/project_workspace/commands.rs`
- `apps/desktop/src-tauri/src/project_workspace/prepare.rs`
- `apps/desktop/src-tauri/src/project_workspace/dedupe.rs`
- `apps/desktop/src-tauri/src/project_workspace/remove.rs`
- `apps/desktop/src-tauri/src/project_workspace/git.rs`
- `apps/desktop/src-tauri/src/project_workspace/ignore.rs`

### CLI

负责：

- 暴露当前 AgentFlow 官方命令入口
- 调用 `spec`、`task-loop`、`workflow-runtime`、`task-artifacts`、`release`、`audit` 等模块
- 提供 Work Agent（`build-agent` 兼容别名）/ 审计 / 运行时写回入口

不负责：

- 保存独立业务状态
- 保留旧 CLI 兼容壳

实现位置：

- `crates/cli/src/**`

## Requirement and Task Contract Layer

### Ontology

负责：

- 提供 Runtime Core 读取的 built-in Ontology Registry
- 定义核心 Object Type 和 Link Type
- 定义 Ontology Bundle、Definition Record 和基础验证规则
- 为后续 Action Contract / Role Policy / State Machine / Arbitration 提供统一世界模型

不负责：

- 写事件
- 推进任务状态
- 启动外部 Agent
- 重建 Projection
- 写 `.agentflow/**` 事实

实现位置：

- `crates/ontology/src/**`

说明：

- 当前只提供 built-in core ontology，不落 `.agentflow/ontology/**`。
- `BuildAgent` 兼容别名、Action Contract 和状态机迁移不在本模块完成。

### Action Contract

负责：

- 提供 Runtime 可读取的 built-in Action Type / Action Contract / Action Proposal schema
- 定义动作输入、目标对象、creates object、required evidence、expected event、expected link
- 校验 proposal 结构合法性，不直接判定接受或拒绝
- 为后续 Role Policy / State Machine / Arbitration 提供统一动作语言

不负责：

- 判定角色权限
- 抢锁或处理冲突
- 写 Event Store
- 更新 Projection
- 启动 Agent 会话

实现位置：

- `crates/action-contract/src/**`

说明：

- 当前只提供 built-in core action contracts，不落 `.agentflow/action-contracts/**`。
- `markIssueDone` 不自动触发 `requestAudit`。

### Role Policy

负责：

- 提供 Runtime 可读取的 built-in Product Role / Runtime Role / Role Capability / Handoff Rule 定义
- 收口产品层角色：`goal-agent / spec-agent / work-agent / audit-agent / delivery-agent`
- 收口 runtime 层角色：`goal-agent / spec-agent / work-agent / audit-agent / delivery-agent / review-agent / coordinator-agent / human-owner`
- 保留 `build-agent -> work-agent` 的兼容别名映射
- 为后续 Arbitration 提供角色动作矩阵、对象矩阵和边界规则

不负责：

- 直接判定 accepted / rejected
- 写 Event Store
- 推进 Projection
- 启动 Provider Session
- 覆盖 prompt 或 handoff 之外的运行时事实

实现位置：

- `crates/role-policy/src/**`

说明：

- `BuildAgent` 只保留为 `WorkAgent` 的兼容别名，不再作为长期主命名。
- `ReviewAgent / CoordinatorAgent / HumanOwner` 先在角色策略层收口，不在本阶段混入 UI 改造。

### Object State

负责：

- 提供 Runtime 可读取的核心对象状态机定义
- 收口 `Requirement / Spec / Issue / Run / Audit / Finding` 的生命周期
- 定义 transition、requiredEvidence、emittedEvents、linkEffects、projectionHints
- 提供旧状态词到新状态机状态的兼容映射
- 为后续 Arbitration 提供对象状态合法性判断入口

不负责：

- 直接追加 Event Store
- 直接改 Projection
- 直接推进 Task Loop
- 直接启动 Provider Session
- 替代 `workflow-core` 的流程定义

实现位置：

- `crates/object-state/src/**`

说明：

- `workflow-core` 继续负责流程状态和 handoff 定义；`object-state` 只负责对象生命周期。
- `Run.completed != Issue.done`、`Issue.done != Audit.requested`、`Finding.fixRequired` 通过修复 Issue 回流，这三条边界由这里固定。

### Action Arbitration

负责：

- 提供 Runtime 写事实前的唯一仲裁入口
- 组合 `action-contract / role-policy / object-state / dependency / evidence / object lock`
- 把 `Action Proposal` 判定为 `accepted / rejected / humanDecisionRequired`
- 生成 `AcceptedAction`，作为后续 Event Store append 输入
- 输出稳定的 rejected reason taxonomy 和 human decision 请求

不负责：

- 直接追加 Event Store
- 直接重建 Projection
- 直接启动 Provider Session
- 直接执行任务
- 直接处理分布式锁或跨项目事务

实现位置：

- `crates/action-arbitration/src/**`

说明：

- 所有 Runtime 写入都必须先过 arbitration，不能绕过。
- `accepted action` 不是事件，只是事件写入前的事实闸口结果。
- MVP 只做本地对象锁和稳定拒绝原因，不做 silent lock stealing。

### Project Brain / Constitution

负责：

- 管理 `docs/projects/<project-id>/**`
- 固化 `GOAL.md / PLAN.md / DECISIONS.md` 的只读 authority
- 作为 Project 的上游方向层
- 给后续 `SpecProject / SpecIssue` 提供目标、路径和确认记录

不负责：

- 推进 issue 状态
- 写运行时事件
- 持有 run / session / lease / checkpoint 事实
- 直接启动外部 Agent

实现位置：

- `docs/product/**`
- `docs/foundation/**`
- `crates/spec/src/model.rs`
- `crates/spec/src/storage.rs`

说明：

- Project 是最高聚合根。
- Issue / Run / Session 不是顶层 authority。
- Project Brain 只负责项目方向、计划和确认，不负责执行状态。
- Completion 只能来自 Project 级判断，不能由 issue done、run 完成或 session 结束直接替代。

### Spec

负责：

- 读取 `docs/requirements/<requirement-id>.md`
- 读取 `docs/projects/<project-id>/**`
- 管理 `.agentflow/spec/projects/**`
- 管理 `.agentflow/spec/issues/**`
- 校验 issue/project 合同、依赖、优先级、workflowRef、allowedPaths、expectedOutputs
- 把 Project Brain 下游 materialize 为执行合同

不负责：

- 持有项目方向 authority
- 推进运行状态
- 写事件流
- 执行任务
- 启动外部 Agent

实现位置：

- `crates/spec/src/model.rs`
- `crates/spec/src/storage.rs`
- `crates/spec/src/lib.rs`

### Agent Manual

负责：

- 生成 `AGENTS.md`
- 生成 `.agentflow/define/agent/**`
- 固化角色边界、语言策略、plain-work-style、技能锁
- 固化运行时角色事实：`goal-agent / spec-agent / work-agent / audit-agent / delivery-agent / specialist / system`
- 保留 `build-agent -> work-agent` 的 provider-facing 兼容别名说明

不负责：

- 代替 Spec 写任务合同
- 代替 Task Loop 调度任务
- 保留旧目录写法

实现位置：

- `crates/agent-manual/src/**`

## Project Context Layer

### Panel

负责：

- 项目文件、符号、关系索引
- Context Pack
- Panel preflight
- 原生 watcher 事件刷新
- 影响范围和测试建议

不负责：

- 执行任务
- 运行测试
- 调用模型
- 推进任务状态机

实现位置：

- `crates/panel/src/**`

说明：

- Panel 只写 `.agentflow/panel/**`
- watcher 只保留 OS native 路径，不再自动降级到 fallback watcher

### Project File Reader

负责：

- 只读文件浏览
- 目录分页
- 搜索与 quick open
- 文本范围读取
- 代码、Markdown、配置、媒体、PDF、DOCX 等只读渲染

不负责：

- 文件写入
- 命令执行
- 源码修改

实现位置：

- `apps/desktop/src-tauri/src/project_files/**`
- `apps/desktop/src/features/project-files/**`

## Runtime and Event Layer

### Workflow Core

负责：

- 解析 YAML workflow
- 校验状态、迁移、guard、action、terminal state
- 校验 `state -> role -> skillPack` 绑定
- 校验 role change 必须显式 handoff
- 区分 `ownership-transfer` 和 `bounded-capability-call`
- 提供任务工作流定义
- 提供 canonical `project / work / audit / delivery` workflow 定义和状态语义辅助函数

不负责：

- 读取具体 issue 实例
- 写事件
- 执行动作

实现位置：

- `crates/workflow-core/src/**`

### Event Store

负责：

- 追加 `.agentflow/events/task-events.jsonl`
- 统一 event envelope：`flowType / runId / authorityRole / correlationId / causationId / idempotencyKey`
- 提供 event taxonomy、event id、replay filter、replay cursor

不负责：

- 投影 UI
- 决定状态
- 执行任务

实现位置：

- `crates/event-store/src/**`

### Workflow Runtime

负责：

- 读取 workflow 定义和当前 projection
- 执行 guard / action
- 追加状态迁移事件
- 在有 `issueId + runId` 的前提下写入 checkpoint 事实
- 阻止非法状态跳转
- 解析当前 state authority role 和 next state authority role
- 把 handoff 边界作为 runtime 一等事实返回

不负责：

- 任意 shell 执行
- 公共交付写入
- provider 启动

实现位置：

- `crates/workflow-runtime/src/**`

### Agent Dispatcher

负责：

- 消费 `agent.launch.requested`
- 把运行时 authority role 映射成 provider-facing session role
- 保留 `build-agent -> work-agent` 的执行兼容
- 只负责 session claim / launch / lifecycle 事实

不负责：

- 决定 workflow authority
- 决定下一条 task
- 让 provider 覆盖 workflow role 绑定

实现位置：

- `crates/agent-dispatcher/src/**`

### Task Artifacts

负责：

- 管理 `.agentflow/tasks/<issue-id>/runs/<run-id>/**`
- 管理 `.agentflow/tasks/<issue-id>/evidence/**`
- 保存本地命令输出、校验结果、checkpoint、plan、验证证据
- 提供 run checkpoint 和 replay cursor 恢复锚点

不负责：

- 写 `.agentflow/tasks/<issue-id>/delivery/**`
- 写 PR/MR body
- 写 CHANGELOG 或 release notes
- 任务调度

实现位置：

- `crates/task-artifacts/src/**`

说明：

- 本地 `.agentflow` 运行产物在 evidence 结束
- 对外公开交付记录属于 PR/MR、CHANGELOG、release notes

## Scheduling and Session Dispatch Layer

### Task Loop

负责：

- 读取 spec project / issue
- 按依赖、优先级、编号排序
- 选择下一条可执行 issue
- 追加调度事件
- 发出 launch request

不负责：

- 直接调用外部 Agent CLI
- 管理 provider session
- 渲染 Desktop 页面

实现位置：

- `crates/task-loop/src/**`

### Agent Dispatcher

负责：

- 消费 `agent.launch.requested`
- 领取待启动 run
- 调用 `mcp` 创建 provider session
- 回写 `agent.session.*` 事件

不负责：

- 任务排序
- 直接决定 issue 状态
- 写源码

实现位置：

- `crates/agent-dispatcher/src/**`

### MCP

负责：

- provider health / capability
- launch plan 生成
- provider session snapshot
- poll / cancel / logs
- GitHub、GitLab、Codex、Browser Preview 等 provider 适配

不负责：

- 决定哪条任务先跑
- 替代 workflow runtime
- 持有任务 authority

实现位置：

- `crates/mcp/src/**`

## Projection and Read Model Layer

### Projection

负责：

- 从 task events 重建 task projection
- 从 task events + spec contracts 重建 project projection
- 生成任务页和项目页索引
- 提供 Desktop 只读模型

不负责：

- 写 spec 合同
- 调用 provider
- 执行本地命令

实现位置：

- `crates/projection/src/**`

说明：

- Desktop 读 projection，不直接读旧 input / execute / output 文件

### State

负责：

- 聚合健康状态
- 生成 workflow gates / blockers / indexes
- 提供跨模块只读状态摘要

不负责：

- 拥有独立业务真相源
- 放宽执行链路门禁

实现位置：

- `crates/state/src/**`

说明：

- `state` 是聚合层，不是事实源
- 事实源仍然是 `spec`、`event-store`、`task-artifacts`、`audit`

## Delivery and Audit Layer

### Release

负责：

- completion 之后的项目级 release gate
- 推进 canonical delivery workflow 的 release 状态
- 写 `.agentflow/release/projects/<project-id>.json`
- 写 `.agentflow/release/reviews/<project-id>.json`
- 写 `.agentflow/indexes/releases.json`
- 写 `.agentflow/indexes/external-reviews.json`
- 从完成态 task projection 和 PR/MR 元数据汇总公开交付记录
- 统一任务级 PR/MR body 模板
- 生成 CHANGELOG / release notes
- 生成 `docs/reviews/<project-id>.md` 外部 review handoff package
- 在显式触发时写公共交付文档

不负责：

- 单条任务状态推进
- Build Agent 执行
- 本地 runtime 证据存储
- 暴露 `.agentflow/tasks/**` 内部路径给外部 reviewer
- 代替 completion 决定项目是否结束

实现位置：

- `crates/release/src/**`

### Audit

负责：

- 管理 `.agentflow/audit/<audit-id>/**`
- 保存 audit report、findings、checklist、evidence map、traceability
- 输出项目级 audit review summary surface
- 支撑独立 audit issue 和 human-via-agent 审计请求

不负责：

- 修改源码
- 修改 spec/task 运行产物
- 创建任务

实现位置：

- `crates/audit/src/**`

## Acceptance

### Acceptance

负责：

- 端到端 acceptance harness
- 迁移后工作流回归验证
- Browser Preview smoke contract 注册

不负责：

- 业务运行时事实写入

实现位置：

- `crates/acceptance/src/**`

## Current Rules

1. 公开需求记录只写 `docs/requirements/**`
2. 内部任务合同只写 `.agentflow/spec/**`
3. 运行时事实只写 `.agentflow/events/**`、`.agentflow/projections/**`、`.agentflow/tasks/**`、`.agentflow/audit/**`
4. 本地证据留在 `.agentflow/tasks/<issue-id>/evidence/**`
5. 对外交付留在 PR/MR、CHANGELOG、release notes
6. 新产品逻辑不得依赖已退役目录或旧兼容模块
