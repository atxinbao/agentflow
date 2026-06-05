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
- [012.1 - Desktop Human Audit Entry Polish](docs/requirements/012-1-desktop-human-audit-entry-polish.md)
- [013 - Workflow State / Gate Orchestration V1](docs/requirements/013-workflow-state-gate-orchestration-v1.md)
- [013.1 - Browser Preview Verification Polish](docs/requirements/013-1-browser-preview-verification-polish.md)
- [013.2 - Browser Preview Smoke Script](docs/requirements/013-2-browser-preview-smoke-script.md)
- [014 - AgentFlow End-to-End Workflow Acceptance V1](docs/requirements/014-agentflow-end-to-end-workflow-acceptance-v1.md)
- [014.1 + 014.2 - Agent Locale and Voice Style Policy V1](docs/requirements/014-1-014-2-agent-locale-and-voice-style-policy-v1.md)

当前目标是在 012 Human-triggered Audit Report V1 之后，完成 012.1 Desktop Human Audit Entry Polish、013 Workflow State / Gate Orchestration V1、013.1 Browser Preview Verification Polish、013.2 Browser Preview Smoke Script、014 AgentFlow End-to-End Workflow Acceptance V1 和 014.1 + 014.2 Agent Locale and Voice Style Policy V1：Desktop 提供人类可见的人工审计入口；`.agentflow/state/` 作为派生状态总控层集中呈现 workflow stage、允许动作、阻断原因、locks、sessions 和 indexes；Browser Preview 能只读展示 release delivery 和 audit report，并通过可复跑 smoke 命令证明只读边界；014 用临时 fixture 项目证明 define / panel / input / execute / output / state / human audit 闭环可达且用户源码不被修改；014.1 + 014.2 固定 managed manuals 为英文，记录 agentLocale，并把默认表达风格收敛为 plain-work-style。

## 当前约束

- 不根据旧文档继续派生新 issue。
- 不把旧 roadmap、旧 specs 或旧 validation summary 当作实现授权。
- 不默认继续 Workflow Control / Product Feature / Closure 等旧阶段。
- 新功能必须来自 `docs/requirements/` 下的新需求文档。

## 新需求入口

```text
docs/requirements/next-requirements.md
```
