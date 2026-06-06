# Changelog

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
