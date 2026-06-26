# AgentFlow Foundation

创建日期：2026-06-17
执行者：Codex

## Purpose

本目录用于承载 AgentFlow 下一代项目模型的基础能力需求。

它不同于 `docs/requirements/`：

- `docs/requirements/` 服务当前版本迭代。
- `docs/foundation/` 服务下一代 Project Operating Model 的底层地基。
- `docs/product/` 定义产品设计基线，回答 AgentFlow 是什么。

Foundation 文档用于把 `docs/product/` 中的方向模型拆成可开发、可验证、可分阶段落地的基础能力。

## Relationship

```text
docs/product/
  定义 Project Operating Model

docs/foundation/
  从产品模型派生下一代基础能力需求

docs/requirements/
  当前版本迭代需求入口
```

## Current Foundation Slices

| 文档 | 作用 |
| --- | --- |
| [001-goal-agent-project-brain-v1.md](001-goal-agent-project-brain-v1.md) | 定义 Goal Agent / Project Brain V1 的第一刀边界 |
| [002-project-brain-document-store-v1.md](002-project-brain-document-store-v1.md) | 定义项目大脑文档存储和读写边界 |
| [003-requirement-to-goal-draft-v1.md](003-requirement-to-goal-draft-v1.md) | 定义用户需求如何转成 Goal Draft |
| [004-plan-draft-preview-v1.md](004-plan-draft-preview-v1.md) | 定义 Confirmed Goal 如何生成 Plan Draft Preview |
| [005-project-brain-desktop-view-v1.md](005-project-brain-desktop-view-v1.md) | 定义 Desktop 的 Project Brain 只读展示 |
| [006-project-brain-confirmation-gate-v1.md](006-project-brain-confirmation-gate-v1.md) | 定义草稿进入项目事实源的确认门 |
| [007-project-loop-entry-from-confirmed-plan-v1.md](007-project-loop-entry-from-confirmed-plan-v1.md) | 定义确认后的 Goal / Plan 如何进入 Project Loop |
| [008-project-loop-scheduler-boundary-v1.md](008-project-loop-scheduler-boundary-v1.md) | 定义 Project Loop 的只读调度和下一步建议边界 |
| [009-issue-preflight-boundary-v1.md](009-issue-preflight-boundary-v1.md) | 定义候选 Issue 进入执行入口前的只读检查边界 |
| [010-work-loop-entry-proposal-v1.md](010-work-loop-entry-proposal-v1.md) | 定义通过 preflight 后进入 Work Loop 前的执行入口提案 |
| [011-work-loop-runtime-boundary-v1.md](011-work-loop-runtime-boundary-v1.md) | 定义单个 Issue 执行 runtime 的分层边界 |
| [012-review-audit-entry-boundary-v1.md](012-review-audit-entry-boundary-v1.md) | 定义 Work Loop 结果进入审计前的只读入口检查 |
| [013-audit-result-model-v1.md](013-audit-result-model-v1.md) | 定义 Audit Agent 的独立审计结果模型 |
| [014-delivery-entry-and-report-model-v1.md](014-delivery-entry-and-report-model-v1.md) | 定义审计通过后的交付入口和交付报告模型 |
| [015-project-completion-evaluation-v1.md](015-project-completion-evaluation-v1.md) | 定义项目完成候选的系统派生判定模型 |
| [016-project-completion-decision-v1.md](016-project-completion-decision-v1.md) | 定义项目完成候选之后的确认决策模型 |
| [017-project-status-mutation-boundary-v1.md](017-project-status-mutation-boundary-v1.md) | 定义项目状态变更的提案和写入边界 |
| [018-project-status-mutation-writer-v1.md](018-project-status-mutation-writer-v1.md) | 定义项目状态写入器的幂等、证据和回滚边界 |
| [019-project-terminal-boundaries-v1.md](019-project-terminal-boundaries-v1.md) | 一次性定义项目终态后的归档、保留、取消、重开和后续项目边界 |
| [020-project-structural-information-principle-v1.md](020-project-structural-information-principle-v1.md) | 定义项目结构信息晋升原则、六个轻量质量维度和 Spec / Evidence / Pack / Simulation / Acceptance / Governance 应用边界 |
| [021-ai-os-project-core-capabilities-v1.md](021-ai-os-project-core-capabilities-v1.md) | 定义 `AI OS Project = Core Runtime + Industry AgentFlow Product`，并明确 6 个 Kernel、12 个通用能力、docs/ 与 .agentflow/ 平面边界和行业 Pack 接入方式 |
| [agentflow-filesystem-workflow-architecture-v1.md](agentflow-filesystem-workflow-architecture-v1.md) | 定义 AgentFlow filesystem-first workflow 架构基准、CodeFlow / DesignFlow 分层和未来 Eve adapter 边界 |

## Rules

- Foundation 文档必须引用对应的 `docs/product/` 基线。
- Foundation 文档不能直接混入 `docs/requirements/`。
- Foundation 文档确认后，才能进一步拆成当前版本可执行需求或新版本实现任务。
- Foundation 文档不自动授权代码实现。
- Foundation 文档不自动写 `.agentflow/` 运行态数据。
- Foundation 文档不自动生成 Issue。
- Foundation 文档不自动进入 Work Loop。

## Non-goals

- 不替代 `docs/product/`。
- 不替代 `docs/requirements/`。
- 不承载当前版本的 UI polish 或 bug fix。
- 不从旧归档文档继承需求。
- 不直接定义远程 PR / CI / SaaS 能力。
