# 036 - AgentFlow Bottom Module Retirement V1

更新日期：2026-06-16  
执行者：Codex

## 背景

AgentFlow 底层已经从早期的 `input -> execute -> output` 三段式，迁移到以任务状态和事件流为中心的新架构：

- 公开需求记录：`docs/requirements/**`
- 内部任务合同：`.agentflow/spec/projects/**`、`.agentflow/spec/issues/**`
- 本地运行状态和证据：`.agentflow/tasks/<issue-id>/**`
- 状态事件：`.agentflow/events/**`
- UI 读模型：`.agentflow/projections/**`、`.agentflow/indexes/**`
- 审计：`.agentflow/audit/**`
- 公开交付：PR/MR body、CHANGELOG、release notes

旧的 `crates/input`、`crates/execute` 和 `crates/loop` 已经成为重复维护面。后续审计又确认 `crates/core` 仍保存旧 workflow archive，`crates/workflow-events` 与 `crates/event-store` 职责重复，CLI 仍保留旧 command 壳，Agent 手册还残留旧兼容入口。这些继续存在会导致状态来源不一致、任务页展示和底层事实源冲突，以及 Build Agent loop 被旧路径误导。

## 目标

一次性清理底层旧模块，让 AgentFlow 只保留新任务状态架构。

## 范围

### 需要退休

- `crates/input`
- `crates/execute`
- `crates/loop`
- `crates/core`
- `crates/workflow-events`
- `crates/cli` 中旧 command 壳
- Cargo workspace 中对应成员
- 所有生产代码对 `agentflow-input`、`agentflow-execute`、`agentflow-loop`、`agentflow-core`、`agentflow-workflow-events` 的依赖
- 旧路径的新写入逻辑：
  - `.agentflow/input/**`
  - `.agentflow/execute/**`
  - `.agentflow/output/**`
- Agent 手册内部旧兼容入口：
  - `legacyAgentEntry`
  - `AGENT.MD`
  - `managed-legacy`
  - `.agentflow/define/goals/**`
  - `.agentflow/define/milestones/**`
  - `.agentflow/define/issues/**`

### 保留的新模块

- `crates/spec`：从 `docs/requirements/**` 生成 `.agentflow/spec/**`
- `crates/task-artifacts`：写 `.agentflow/tasks/<issue-id>/**`
- `crates/task-loop`：调度 issue，生成 Build Agent launch request
- `crates/event-store`：记录任务状态事件、consumer offset 和 dead-letter
- `crates/projection`：生成任务页读模型
- `crates/release`：生成公开交付记录
- `crates/audit`：只负责人类审计
- `crates/state`：只聚合健康、门禁、索引和状态

## 架构规则

1. `docs/requirements/**` 是人类可审计的需求记录。
2. `.agentflow/spec/issues/<issue-id>.json` 是 Build Agent 的任务合同。
3. `.agentflow/tasks/<issue-id>/runs/<run-id>/**` 是本地执行过程留痕。
4. `.agentflow/tasks/<issue-id>/evidence/**` 是本地验证证据。
5. `.agentflow/events/**` 是状态变化事实流。
6. `.agentflow/projections/**` 和 `.agentflow/indexes/**` 是 UI 读模型。
7. 公开交付不写入 `.agentflow/output/**`，只写 PR/MR body、CHANGELOG 或 release notes。
8. 不保留旧 `input/execute/output` 降级读取。
9. 任务事件只通过 `event-store` 写入和消费，不再保留 `workflow-events` crate。
10. CLI 只暴露当前仍可执行的 Build Agent、Task Loop、Agent Bridge、Projection 和 Release 命令，不保留旧命令壳。

## 开发要求

1. 删除 Cargo workspace 中旧 crate 成员。
2. `state` 只从 `spec`、`projection`、`task-artifacts`、`event-store`、`audit` 读取。
3. `acceptance` 测试夹具改为新链路：
   - 写 `docs/requirements/**`
   - 写 `.agentflow/spec/issues/**`
   - 通过 `task-loop` 创建 run / launch request
   - 通过 `task-artifacts` 写验证和 evidence
   - 通过 `projection` 生成状态读模型
4. Desktop Tauri 命令不再依赖 `agentflow-input`。
5. Agent 手册和 README 不再把 `.agentflow/input/**` 描述为当前事实源。
6. 保留历史需求文档中的旧路径描述，但新增文档必须明确旧路径已退休。
7. 删除 `core` legacy archive 和 active wrapper，不再保留旧 workflow 兼容层。
8. 将 `workflow-events` 的 consumer / dead-letter / payload 能力合并进 `event-store`。
9. Agent 手册不再生成或迁移 `AGENT.MD`、`legacyAgentEntry` 或 `managed-legacy`。

## 验收标准

- `cargo metadata` 中不存在 `agentflow-input`、`agentflow-execute`、`agentflow-loop`、`agentflow-core`、`agentflow-workflow-events`。
- `rg "agentflow_input|agentflow_execute|agentflow_loop|agentflow_core|agentflow_workflow_events|agentflow-input|agentflow-execute|agentflow-loop|agentflow-core|agentflow-workflow-events" crates apps/desktop/src-tauri Cargo.toml` 无结果。
- 新初始化不会创建 `.agentflow/input`、`.agentflow/execute`、`.agentflow/output`。
- 任务页仍能从 projection/index 读取任务状态。
- 审计仍使用 `.agentflow/audit/**`。
- 交付仍通过公开 PR/MR / CHANGELOG / release notes 留痕。
- Agent 手册不再返回 `managed-legacy` 或 `legacyAgentEntry`。

## 验证命令

```bash
cargo fmt --check
cargo test --workspace
npm --prefix apps/desktop run build
git diff --check
```

## 不做事项

- 不恢复旧 `input/execute/output` fallback。
- 不新增第二套交付模块。
- 不把公开交付重新写回 `.agentflow/output/**`。
- 不做 `state` 瘦身。`state` 本轮只处理删除旧 enum / crate 后必要的编译边界。
- 不改用户项目源码。
- 不引入新的业务能力。
