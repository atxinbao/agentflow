# Next Requirements

创建日期：2026-06-01
更新日期：2026-06-07
执行者：Codex

## 状态

已确认的新功能需求：

- [001-add-local-project.md](001-add-local-project.md)
- [003-project-file-reader-v1-completion.md](003-project-file-reader-v1-completion.md)
- [004-legacy-cleanup-and-new-module-split.md](004-legacy-cleanup-and-new-module-split.md)
- [005-legacy-and-degraded-code-removal.md](005-legacy-and-degraded-code-removal.md)
- [006-legacy-cli-retirement-and-archive-pruning.md](006-legacy-cli-retirement-and-archive-pruning.md)
- [007-goal-tree-v1.md](007-goal-tree-v1.md)
- [007-1-goal-tree-agent-only-boundary-fix.md](007-1-goal-tree-agent-only-boundary-fix.md)
- [008-agent-working-manual-bootstrap-v1.md](008-agent-working-manual-bootstrap-v1.md)
- [008-1-agent-working-manual-health-polish.md](008-1-agent-working-manual-health-polish.md)
- [008-2-requirement-intake-filter-skill-v1.md](008-2-requirement-intake-filter-skill-v1.md)
- [008-3-agentflow-workflow-directory-blueprint-v1-final.md](008-3-agentflow-workflow-directory-blueprint-v1-final.md)
- [008-4-project-panel-v1.md](008-4-project-panel-v1.md)
- [008-4-1-project-panel-finalization-and-graph-removal-v1.md](008-4-1-project-panel-finalization-and-graph-removal-v1.md)
- [008-4-2-agentflow-workspace-ownership-guard-v1.md](008-4-2-agentflow-workspace-ownership-guard-v1.md)
- [009-input-model-v1.md](009-input-model-v1.md)
- [010-execute-patch-checkpoint-v1.md](010-execute-patch-checkpoint-v1.md)
- [010-2-agent-role-consolidation-v2.md](010-2-agent-role-consolidation-v2.md)
- [011-output-evidence-delivery-audit-v1.md](011-output-evidence-delivery-audit-v1.md)
- [012-human-triggered-audit-report-v1.md](012-human-triggered-audit-report-v1.md)
- [012-1-desktop-human-audit-entry-polish.md](012-1-desktop-human-audit-entry-polish.md)
- [013-workflow-state-gate-orchestration-v1.md](013-workflow-state-gate-orchestration-v1.md)
- [013-1-browser-preview-verification-polish.md](013-1-browser-preview-verification-polish.md)
- [013-2-browser-preview-smoke-script.md](013-2-browser-preview-smoke-script.md)
- [014-agentflow-end-to-end-workflow-acceptance-v1.md](014-agentflow-end-to-end-workflow-acceptance-v1.md)
- [014-1-014-2-agent-locale-and-voice-style-policy-v1.md](014-1-014-2-agent-locale-and-voice-style-policy-v1.md)
- [015-human-agent-guided-experience-v1.md](015-human-agent-guided-experience-v1.md)
- [016-desktop-design-system-v1.md](016-desktop-design-system-v1.md)
- [017-agentflow-unified-ux-spec-v16-pr33-merged.md](017-agentflow-unified-ux-spec-v16-pr33-merged.md)
- [018-agentflow-code-cleanup-rust-workspace.md](018-agentflow-code-cleanup-rust-workspace.md)
- [019-agentflow-issue-status-enhancement.md](019-agentflow-issue-status-enhancement.md)
- [021-agentflow-frontend-interaction-ux-v1.md](021-agentflow-frontend-interaction-ux-v1.md)
- [022-agentflow-local-agents-management-v1.md](022-agentflow-local-agents-management-v1.md)
- [023-agentflow-github-code-latest-svg-page-repair-v1.md](023-agentflow-github-code-latest-svg-page-repair-v1.md)
- [024-agentflow-project-tree-multi-project-navigation-v1.md](024-agentflow-project-tree-multi-project-navigation-v1.md)
- [025-agentflow-base-release-initialization-v1.md](025-agentflow-base-release-initialization-v1.md)
- [026-agentflow-release-audit-trigger-rules-v1.md](026-agentflow-release-audit-trigger-rules-v1.md)

后续新需求继续写入本文件或新增 `00N-*.md`。

## 当前前端设计基线

- [015-human-agent-guided-experience-v1.md](015-human-agent-guided-experience-v1.md) = historical design reference
- [017-agentflow-unified-ux-spec-v16-pr33-merged.md](017-agentflow-unified-ux-spec-v16-pr33-merged.md) = current frontend implementation baseline
- [021-agentflow-frontend-interaction-ux-v1.md](021-agentflow-frontend-interaction-ux-v1.md) = current frontend interaction baseline

## 背景

项目已完成旧文档清理。后续开发与 2026-05 的旧需求、旧规划、旧规格无继承关系。

## 新需求模板

```md
# <需求名称>

## 用户目标
<用户最终要完成什么>

## 范围
- 

## 非目标
- 

## 页面 / 功能
- 

## 数据来源
- 

## 交互边界
- 

## 验收标准
- [ ] 

## 验证命令
- `npm --prefix apps/desktop run build`
- `cargo test`
- `git diff --check`
```

## 当前不授权

- 不授权继续旧 roadmap。
- 不授权继续旧 workflow / product feature / closure 方向。
- 不授权从归档 specs 自动生成任务。
