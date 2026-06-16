# Changelog

## 0.2.0 - 2026-06-17

执行者：Codex

AgentFlow 0.2.0 将工作流主线从旧 `input / execute / output` 分层收口到任务驱动架构。

### Included

- 新增 `spec / task-loop / task-artifacts / event-store / projection / release / agent-dispatcher / mcp` 等底层模块边界。
- 清理旧 `input / execute / output / core legacy / workflow-events / degraded fallback` 活跃依赖。
- 任务页成为主工作台，右侧展示状态时间线、事件流、实时会话信息和最终交付摘要。
- Build Agent loop 支持 `start / claim-launch / prepare-review / write-merge-proof / complete` 官方命令链路。
- Build Agent session 支持 `interrupted / resumed` 生命周期，并通过事件流写回投影。
- Project Loop 支持手动触发、按依赖顺序推进 issue，并在当前 issue Done 后尝试拉起下一条 issue。
- 公开交付记录写入 PR/MR、CHANGELOG 或 release notes，不再依赖旧 `.agentflow/output/**`。
- Browser Preview mock 和 smoke 覆盖任务状态流、公开交付和投影读取。

### Architecture

- `docs/requirements/**` 是公开需求记录。
- `.agentflow/spec/**` 是本地项目和 issue 合同事实源。
- `.agentflow/events/**` 是任务状态事件流。
- `.agentflow/tasks/<issue-id>/**` 是本地 run 与验证证据事实源。
- `.agentflow/projections/**` 和 `.agentflow/indexes/**` 是 Desktop 只读展示投影。

### Validation

- `npm --prefix apps/desktop run build`
- `cargo test --workspace`
- `git diff --check`

## 0.1.0 - 2026-06-07

执行者：Codex

AgentFlow 0.1.0 是第一个可交付本地版本。

### Included

- 本地项目接入和空项目工作台。
- Agent working manual、locale、voice style 和 workspace ownership guard。
- Project Panel、项目文件只读阅读器、任务流转、交付、审计和高级详情页面。
- Input / Execute / Output / State 工作流基础层。
- Browser Preview mock 和 smoke 验证。
- Desktop Design System V1 和统一 V16 UX。
- Base Release 初始化。
- Release Audit Trigger Rules V1。

### Release Audit

- Release Delivery 生成后，AgentFlow 自动登记 `release-auto` audit request。
- 同一个 Release Delivery 不重复创建 `release-auto` audit request。
- Desktop 普通 UI 只展示审计状态、触发来源、报告、发现项、证据链和追溯关系。
- Desktop 普通 UI 不创建审计请求。

### Validation

- `cargo fmt --check`
- `cargo check --workspace`
- `cargo test`
- `npm --prefix apps/desktop run build`
- `npm --prefix apps/desktop run preview:smoke`
- `cargo build --release -p agentflow-cli`
- `npm --prefix apps/desktop run tauri -- build`
- `git diff --check`
