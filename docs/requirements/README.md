# Requirements

创建日期：2026-06-01
更新日期：2026-06-07
执行者：Codex

## 目的

这里是 AgentFlow 后续开发的唯一需求入口。

旧文档已经归档到 `docs/archive/2026-05-agentflow-legacy/`，不再自动影响新需求。

## 使用方式

1. 把新的产品需求写入本目录。
2. 每个需求文档必须说明：
   - 用户目标
   - 页面或功能范围
   - 数据来源
   - 不做什么
   - 验收标准
   - 验证命令
3. 开发切片只能从本目录的需求文档派生。

## 当前入口

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
- [027-agentflow-agent-role-descriptor-and-issue-guard-v1.md](027-agentflow-agent-role-descriptor-and-issue-guard-v1.md)
- [028-agentflow-codex-role-usage-guide-v1.md](028-agentflow-codex-role-usage-guide-v1.md)
- [029-agentflow-dogfood-cutover-cleanup-and-start-v1.md](029-agentflow-dogfood-cutover-cleanup-and-start-v1.md) = dogfood cutover baseline
- [next-requirements.md](next-requirements.md)

## Dogfood Cutover 基线

- [024-agentflow-project-tree-multi-project-navigation-v1.md](024-agentflow-project-tree-multi-project-navigation-v1.md) = current base workspace baseline
- [025-agentflow-base-release-initialization-v1.md](025-agentflow-base-release-initialization-v1.md) = current base release initialization baseline
- [026-agentflow-release-audit-trigger-rules-v1.md](026-agentflow-release-audit-trigger-rules-v1.md) = current release audit trigger baseline
- [027-agentflow-agent-role-descriptor-and-issue-guard-v1.md](027-agentflow-agent-role-descriptor-and-issue-guard-v1.md) = current role guard baseline
- [028-agentflow-codex-role-usage-guide-v1.md](028-agentflow-codex-role-usage-guide-v1.md) = current Codex role usage baseline
- [029-agentflow-dogfood-cutover-cleanup-and-start-v1.md](029-agentflow-dogfood-cutover-cleanup-and-start-v1.md) = dogfood cutover baseline

## 历史设计参考

- [015-human-agent-guided-experience-v1.md](015-human-agent-guided-experience-v1.md) = historical design reference / pre-base requirement / superseded by dogfood workflow
- [017-agentflow-unified-ux-spec-v16-pr33-merged.md](017-agentflow-unified-ux-spec-v16-pr33-merged.md) = historical design reference / pre-base requirement / superseded by dogfood workflow
- [018-agentflow-code-cleanup-rust-workspace.md](018-agentflow-code-cleanup-rust-workspace.md) = pre-base cleanup reference / superseded by dogfood workflow
- [019-agentflow-issue-status-enhancement.md](019-agentflow-issue-status-enhancement.md) = pre-base status reference / superseded by dogfood workflow
- [021-agentflow-frontend-interaction-ux-v1.md](021-agentflow-frontend-interaction-ux-v1.md) = historical design reference / pre-base requirement / superseded by dogfood workflow
- [023-agentflow-github-code-latest-svg-page-repair-v1.md](023-agentflow-github-code-latest-svg-page-repair-v1.md) = historical design reference / pre-base requirement / superseded by dogfood workflow

## 不继承内容

以下内容不从旧文档继承，除非新需求重新明确：

- 旧 Workflow Control 流程
- 旧 Product Feature 流程
- 旧 Project Closure / Audit / Docs Refresh
- 旧 GoalLoop / Eligibility / Lease / Evidence 自动推进
- 旧 Desktop 页面职责收敛方案
