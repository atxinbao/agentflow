# Goal

更新日期：2026-06-01
执行者：Codex

## 当前状态

AgentFlow 的旧目标文档已停止作为后续开发依据。

后续开发将由新的需求文档重新定义，旧的 Workflow Control、Product Feature Flow、Goal + Criteria MVP、Project Closure 等阶段性目标均已归档，仅保留历史参考价值。

## 当前总目标

当前新功能需求已经定义为：

- [001 - 添加本地项目](docs/requirements/001-add-local-project.md)
- [002 - Graph V1](docs/requirements/002-graph-v1.md)
- [002.1 - Graph V1 Completion](docs/requirements/002-1-graph-v1-completion.md)

当前目标是在 Project Workspace Manager 和 Graph V1 主链路之后，实现 Graph V1 Completion：补齐 watcher、preflight、parser registry、impact/test recommendation、mobile semantics、protection 和状态通道，使 Graph 成为稳定可依赖的 Agent 工作现场服务。

## 当前约束

- 不根据旧文档继续派生新 issue。
- 不把旧 roadmap、旧 specs 或旧 validation summary 当作实现授权。
- 不默认继续 Workflow Control / Product Feature / Closure 等旧阶段。
- 新功能必须来自 `docs/requirements/` 下的新需求文档。

## 新需求入口

```text
docs/requirements/next-requirements.md
```
