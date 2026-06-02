# AgentFlow

更新日期：2026-06-01
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
- Goal Tree V1 允许用户在本地项目下写入 `.agentflow/define/**` 目标树事实源。
- 不执行命令。
- 不写用户业务源码。
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
cargo test -p agentflow-goal-tree
cargo test
git diff --check
```

历史验证记录仍保留在 [verification.md](verification.md)，但它只说明旧阶段发生过什么，不授权新的开发方向。
