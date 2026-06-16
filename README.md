# AgentFlow

更新日期：2026-06-14
执行者：Codex

## 当前文档状态

项目文档已重置。

2026-05 期间形成的旧需求、旧规划、旧规格和旧验证摘要已经统一归档到：

```text
docs/archive/2026-05-agentflow-legacy/
```

这些文档只作为历史参考，不再作为后续开发需求、实现授权或验收依据。

## 新需求入口

后续开发只从新的需求文档开始：

```text
docs/requirements/
```

当前入口：

- [GOAL.md](GOAL.md)
- [ROADMAP.md](ROADMAP.md)
- [docs/requirements/README.md](docs/requirements/README.md)
- [docs/requirements/next-requirements.md](docs/requirements/next-requirements.md)

## 当前产品边界

- AgentFlow 是本地优先的桌面工作台。
- Desktop 当前只读取和展示本地项目内容。
- Project Workspace Manager 会准备以 spec / tasks / events / projections 为核心的 `.agentflow/` 本地工作区。
- Agent Manual Bootstrap 会接管根目录 `AGENTS.md` 作为 canonical Agent entry，并写入 `.agentflow/define/agent/**` 工作手册、skills 和 lock。
- Workflow Directory Blueprint V1 会准备 `.agentflow/workspace-manifest.json`，并把 `define/` 收敛为 `agent/spec/tdd/release/audit` 工作手册区。
- Spec Contract V1 将公开需求记录放在 `docs/requirements/**`，将内部 project / issue 合同放在 `.agentflow/spec/projects/**` 和 `.agentflow/spec/issues/**`。旧 `.agentflow/input/` 已退休，不再作为任务事实源或兼容读取路径。
- Task Workflow Runtime V1 使用 YAML workflow、事件日志和投影驱动任务状态。运行期事实写入 `.agentflow/events/**`、`.agentflow/projections/**` 和 `.agentflow/tasks/<issue-id>/**`。
- Task Artifacts V1 将本地运行产物收敛到 `.agentflow/tasks/<issue-id>/runs/**`，验证证据收敛到 `.agentflow/tasks/<issue-id>/evidence/**`。公开交付记录进入 PR/MR body、CHANGELOG 或 release notes，不再写本地 `.agentflow/tasks/<issue-id>/delivery/**`。
- Agent Role Consolidation V2 将顶层 Agent 角色收敛为 Spec / Build / Audit；Release Agent 不再独立存在，公开交付记录由 Build Agent 在 PR/MR 和发布记录中完成。
- Human-triggered Audit Report V1 将审计定义为独立流程。任务 Done 不会自动触发审计；审计只从独立 audit issue 或明确的人类审计请求开始。
- Desktop Human Audit Entry Polish 在 Desktop 里提供人工审计入口：人类选择任务公开交付记录并填写 reason 后才会请求 audit；浏览器预览不会写 `.agentflow/audit`。
- Projection V1 从 `.agentflow/spec/**` 和 `.agentflow/events/**` 重建任务页、项目状态和 issue-status index。Desktop 展示读取 projection，不把旧 input/execute/output 当成任务权威。
- Browser Preview Verification Polish 为 Desktop 浏览器预览补齐只读任务交付摘要和 audit report mock，使人工审计入口可以完成可视核对；它不写 `.agentflow/audit`。
- Browser Preview Smoke Script 新增 `npm --prefix apps/desktop run preview:smoke`，用可复跑本地断言验证 Browser Preview mock、人工审计禁用边界和 `.agentflow/audit` 禁写。
- AgentFlow End-to-End Workflow Acceptance V1 新增系统级验收：用临时 fixture 项目证明 define / panel / input / execute / output / state / human audit 闭环可达，并验证用户源码 hash 不变。
- Agent Locale and Voice Style Policy V1 固定 AgentFlow managed manuals 为英文，记录 `agentLocale`，新增 `plain-work-style` 默认表达规则，并要求 Agent 新写代码注释跟随 `agentLocale`。
- Project Panel canonical path 为 `.agentflow/panel/`；不再保留旧代码地图兼容路径。
- Desktop human UI 不执行命令。
- Build Agent 只能从当前 `.agentflow/spec/issues/<issue-id>.json` 启动，并通过 workflow runtime、task loop、agent dispatcher 和 task artifacts 完成 preflight、运行、验证、PR/MR、合并和 Done 写回。
- 未经 execute 流水线授权不写用户业务源码。
- 不写旧 `.agentflow/issues`、`runs`、`evidence`、`reviews`、`updates`、`views` 路径。
- 不调用模型。
- 不创建远程 PR、GitHub issue 或 Linear issue。
- 后续能力必须由新的需求文档重新定义。

## 桌面开发

```bash
npm --prefix apps/desktop run build
npm --prefix apps/desktop run dev
```

Tauri 开发：

```bash
cd apps/desktop
npm run tauri -- dev
```

## 验证

```bash
npm --prefix apps/desktop run build
cargo test -p agentflow-agent-manual
cargo test -p agentflow-spec
cargo test -p agentflow-event-store
cargo test -p agentflow-workflow-core
cargo test -p agentflow-workflow-runtime
cargo test -p agentflow-task-loop
cargo test -p agentflow-task-artifacts
cargo test -p agentflow-projection
cargo test -p agentflow-agent-dispatcher
cargo test -p agentflow-mcp
cargo test -p agentflow-state
cargo test -p agentflow-workflow-acceptance
cargo test -p agentflow-panel
cargo test
npm --prefix apps/desktop run preview:smoke
git diff --check
```

历史验证记录仍保留在 [verification.md](verification.md)，但它只说明旧阶段发生过什么，不授权新的开发方向。
