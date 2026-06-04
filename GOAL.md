# Goal

更新日期：2026-06-04
执行者：Codex

## 当前状态

AgentFlow 的旧目标文档已停止作为后续开发依据。

后续开发将由新的需求文档重新定义，旧的 Workflow Control、Product Feature Flow、Goal + Criteria MVP、Project Closure 等阶段性目标均已归档，仅保留历史参考价值。

## 当前总目标

当前新功能需求已经定义为：

- [001 - 添加本地项目](docs/requirements/001-add-local-project.md)
- [002 - Graph V1](docs/requirements/002-graph-v1.md)
- [002.1 - Graph V1 Completion](docs/requirements/002-1-graph-v1-completion.md)
- [002.2 - Graph V1 OS Native Watcher Closeout](docs/requirements/002-2-graph-v1-os-native-watcher-closeout.md)
- [003 - Project File Reader V1 Completion](docs/requirements/003-project-file-reader-v1-completion.md)
- [004 - Legacy Cleanup and New Module Split](docs/requirements/004-legacy-cleanup-and-new-module-split.md)
- [005 - Legacy and Degraded Code Removal](docs/requirements/005-legacy-and-degraded-code-removal.md)
- [006 - Legacy CLI Retirement and Archive Pruning](docs/requirements/006-legacy-cli-retirement-and-archive-pruning.md)
- [007 - Goal Tree V1](docs/requirements/007-goal-tree-v1.md)
- [007.1 - Goal Tree Agent-only Boundary Fix](docs/requirements/007-1-goal-tree-agent-only-boundary-fix.md)
- [008 - Agent Working Manual Bootstrap V1](docs/requirements/008-agent-working-manual-bootstrap-v1.md)
- [008.1 - Agent Working Manual Health Polish](docs/requirements/008-1-agent-working-manual-health-polish.md)
- [008.2 - Requirement Intake Filter Skill V1](docs/requirements/008-2-requirement-intake-filter-skill-v1.md)
- [008.3 - AgentFlow Workflow Directory Blueprint V1](docs/requirements/008-3-agentflow-workflow-directory-blueprint-v1-final.md)
- [008.4 - Project Panel V1](docs/requirements/008-4-project-panel-v1.md)

当前目标是在 Project Workspace、Graph V1、Project File Reader、Goal Tree 只读边界和 Agent Working Manual / Requirement Intake 之后，实现 008.4 Project Panel V1：将 Graph 产品概念升级为 Panel，新的 canonical path 为 `.agentflow/panel/`，旧 `.agentflow/output/graph/` 与 `.agentflow/graph/` 仅作为 legacy compatibility。

## 当前约束

- 不根据旧文档继续派生新 issue。
- 不把旧 roadmap、旧 specs 或旧 validation summary 当作实现授权。
- 不默认继续 Workflow Control / Product Feature / Closure 等旧阶段。
- 新功能必须来自 `docs/requirements/` 下的新需求文档。

## 新需求入口

```text
docs/requirements/next-requirements.md
```
