# AEP Goal Initialization Protocol

创建日期：2026-05-22
执行者：Codex

## 目标

本协议把 Codex 的 `/goal` 固化为 AgentFlow 的新项目初始化入口。它对应 AEP 第一阶段：先形成 Project Definition 初始化合同，再生成可检查的本地事实源，最后才允许规划可执行 issue。

## 核心链路

```text
/goal
-> ProjectGoal
-> ProjectDefinition
-> Bootstrap Artifacts
-> ScopeState
-> GoalReadiness
-> IssueContract
-> GoalLoop
```

## 初始化产物

`agentflow init --from-goal GOAL.md` 和 `agentflow goal bootstrap` 必须保证以下产物存在：

| 产物 | 路径 | 作用 |
| --- | --- | --- |
| ProjectGoal | `.agentflow/goal.json` | 目标、成功标准、非目标 |
| ProjectDefinition | `.agentflow/project-definition.json` | AEP 第一阶段初始化合同 |
| ScopeState | `.agentflow/scope-state.json` | WIP=1、active issue、执行授权边界 |
| Bootstrap Sequence | `.agentflow/bootstrap/project-bootstrap-sequence.md` | 新项目启动顺序 |
| Blueprint Seed | `.agentflow/bootstrap/complete-blueprint-design.md` | 产品蓝图种子 |
| Product Surface Map | `.agentflow/bootstrap/product-surface-map.md` | 产品界面面域 |
| Frontend Wireframe | `.agentflow/bootstrap/frontend-surface-wireframe.md` | 前端首轮结构 |
| ViewModel Contract | `.agentflow/bootstrap/frontend-viewmodel-contract.md` | UI 读取契约 |
| Backend Use Case Contract | `.agentflow/bootstrap/backend-usecase-contract.md` | 后端用例边界 |
| Persistence Boundary | `.agentflow/bootstrap/persistence-boundary.md` | 事实源和索引边界 |
| Read Model Projection | `.agentflow/bootstrap/read-model-projection.md` | 摘要和只读视图来源 |
| API Contract | `.agentflow/bootstrap/api-contract.md` | Core / CLI / future Tauri 边界 |
| Linear Project Draft | `.agentflow/bootstrap/linear-project-draft.md` | 远程项目草案，不创建远程对象 |
| Linear Issue Draft | `.agentflow/bootstrap/linear-issue-draft.md` | 远程 issue 草案，不授权执行 |
| Agent Sandbox Profile | `.agentflow/bootstrap/agent-sandbox-profile.md` | 本地执行器边界 |

## Goal Readiness

`agentflow goal check` 是第一阶段机械门禁。它检查 goal、project-definition、scope-state、environment、architecture、roadmap、初始化 evidence 和 bootstrap 产物。缺失任一产物时返回失败。

## Issue Contract 增强

所有新 issue contract 必须包含 AEP 协议字段：

- stop condition
- fastest deterministic feedback loop
- vertical slice
- tracer bullet plan
- diagnose plan
- Graphify context status
- docs claim trace
- boundary confirmation
- PR handoff requirements

这些字段不替代 scope / non-goals / validation，而是把 AEP 执行前检查变成默认契约。

## Goal Loop Orchestrator

`agentflow goal next` 是 `/goal` 初始化后的本地推进决策器。它读取：

- `.agentflow/goal.json`
- `.agentflow/project-definition.json`
- `.agentflow/scope-state.json`
- `.agentflow/index.json`
- `.agentflow/roadmap.md`
- `.agentflow/issues/*`
- `.agentflow/runs/*`
- `.agentflow/evidence/*`
- `.agentflow/reviews/*`

输出：

- `.agentflow/goal-loop.json`
- `.agentflow/updates/GOAL-LOOP-SUMMARY.md`

决策结果只能是 `plan`、`run`、`verify`、`review`、`update` 或 `wait-human`。它只给出下一步建议，不自动执行、不绕过 IssueContract。

## 边界

- Project Definition 初始化合同不授权执行。
- Roadmap、candidate issue、SavedView、ProjectUpdate 都不授权执行。
- 只有 IssueContract 可以启动 run。
- Goal Loop 不能直接启动 run，只能输出 recommended command。
- 本地 v0 不创建远程 Linear issue、GitHub PR、merge 或团队 workspace 变更。
- Graphify 只记录 context status，正式集成后置。
