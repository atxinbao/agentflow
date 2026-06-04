# AgentFlow

更新日期：2026-06-05
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
- Project Workspace Manager 会准备 `.agentflow/` 三段式本地工作区。
- Agent Manual Bootstrap 会接管根目录 `AGENTS.md` 作为 canonical Agent entry，保留 `AGENT.MD` 为 legacy compatibility，并写入 `.agentflow/define/agent/**` 工作手册、skills 和 lock。
- Workflow Directory Blueprint V1 会准备 `.agentflow/workspace-manifest.json`，并把 `define/` 收敛为 `agent/spec/tdd/release/audit` 工作手册区。
- Input Model V1 是新的需求事实源；canonical path 为 `.agentflow/input/`，旧 `.agentflow/spec/` 和 `.agentflow/goal-tree/` 仅作为 legacy marker，不再作为新写入路径。
- Execute Patch / Checkpoint V1 是受控执行层；canonical path 为 `.agentflow/execute/`，只能从 `.agentflow/input/issues/<issue-id>.json` 启动，结果证据写入 `.agentflow/output/evidence/`。
- Agent Role Consolidation V2 将顶层 Agent 角色收敛为 Spec / Build / Audit；Release Agent 不再独立存在，release delivery 能力归入 Build Agent，交付材料写入 `.agentflow/output/release/`。
- Output Evidence / Delivery / Audit V1 将 `.agentflow/output/` 收口为交付与证据层：`output/evidence` 是 Build Agent 执行证明，`output/release` 是 Build Agent 本地交付材料。
- Human-triggered Audit Report V1 将 `output/audit` 明确为人类主动触发的完整审计报告包；它不会随 execute / output 自动生成，只在 `request_human_audit` 后写入 `.agentflow/output/audit/<audit-id>/`。
- Desktop Human Audit Entry Polish 在 Desktop 里提供人工审计入口：人类选择 release delivery 并填写 reason 后才会请求 audit；浏览器预览不会写 `.agentflow/output/audit`。
- Workflow State / Gate Orchestration V1 新增 `.agentflow/state/` 派生状态总控层，聚合 define / panel / input / execute / output / audit 健康状态，输出 gates、next actions、blockers、sessions、locks、events 和 indexes；它只写 `.agentflow/state/**`。
- Browser Preview Verification Polish 为 Desktop 浏览器预览补齐只读 release delivery 和 audit report mock，使人工审计入口可以完成可视核对；它不写 `.agentflow/output/audit`。
- Project Panel canonical path 为 `.agentflow/panel/`；不再保留旧代码地图兼容路径。
- Desktop human UI 不执行命令。
- Execute API 允许 Agent-only 受控 patch / command，但必须通过 preflight、lease、plan、checkpoint 和 allowedWritePaths / allowedCommands。
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
cargo test -p agentflow-goal-tree
cargo test -p agentflow-input
cargo test -p agentflow-output
cargo test -p agentflow-execute
cargo test -p agentflow-state
cargo test -p agentflow-panel
cargo test
git diff --check
```

历史验证记录仍保留在 [verification.md](verification.md)，但它只说明旧阶段发生过什么，不授权新的开发方向。
