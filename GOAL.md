# Goal

更新日期：2026-06-05
执行者：Codex

## 当前状态

AgentFlow 的旧目标文档已停止作为后续开发依据。

后续开发将由新的需求文档重新定义，旧的 Workflow Control、Product Feature Flow、Goal + Criteria MVP、Project Closure 等阶段性目标均已归档，仅保留历史参考价值。

## 当前总目标

当前新功能需求已经定义为：

- [001 - 添加本地项目](docs/requirements/001-add-local-project.md)
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
- [008.4.1 - Project Panel Finalization and Graph Removal V1](docs/requirements/008-4-1-project-panel-finalization-and-graph-removal-v1.md)
- [008.4.2 - AgentFlow Workspace Ownership Guard V1](docs/requirements/008-4-2-agentflow-workspace-ownership-guard-v1.md)
- [009 - Input Model V1](docs/requirements/009-input-model-v1.md)
- [010 - Execute Patch / Checkpoint V1](docs/requirements/010-execute-patch-checkpoint-v1.md)
- [010.2 - Agent Role Consolidation V2](docs/requirements/010-2-agent-role-consolidation-v2.md)
- [011 - Output Evidence / Delivery / Audit V1](docs/requirements/011-output-evidence-delivery-audit-v1.md)
- [012 - Human-triggered Audit Report V1](docs/requirements/012-human-triggered-audit-report-v1.md)

当前目标是在 011 Output Evidence / Delivery / Audit V1 之后，实现 012 Human-triggered Audit Report V1：Audit 不再是每次 execute / output 的默认步骤，而是人类主动触发后生成 `.agentflow/output/audit/<audit-id>/` 完整审计报告包。

## 当前约束

- 不根据旧文档继续派生新 issue。
- 不把旧 roadmap、旧 specs 或旧 validation summary 当作实现授权。
- 不默认继续 Workflow Control / Product Feature / Closure 等旧阶段。
- 新功能必须来自 `docs/requirements/` 下的新需求文档。

## 新需求入口

```text
docs/requirements/next-requirements.md
```
