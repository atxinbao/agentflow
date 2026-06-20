# AgentFlow v0.4.0 Definition-driven Runtime Foundation

日期：2026-06-20
执行者：Codex
状态：Released Baseline / Release Closeout / 已发布版本文档

## 1. Purpose

本目录收口 AgentFlow `v0.4.0` 的正式发布基线与版本 closeout 文档。

`v0.4.0` 的版本目标是：

```text
Definition-driven Runtime Foundation
```

它要把 AgentFlow 从 `v0.3.0` 的项目级闭环，推进到定义驱动的 Runtime Core：

```text
Runtime Command
→ Action Proposal
→ Action Contract
→ Role Policy
→ Object State Machine
→ Action Arbitration
→ Event Store
→ Projection
```

## 2. Boundary

本目录当前保存 Runtime Foundation 已发布版本的技术基线、设计来源和 release closeout 事实。

它不再表示“未进入实现的 planning draft”，而是：

- `v0.4.0` 已发布；
- Runtime Foundation 已收口；
- 这些文档现在用于版本回顾、架构追溯和后续版本衔接。

## 3. Reading Order

先读版本收敛和总方案：

1. [AGENTFLOW_VERSION_ROADMAP_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_VERSION_ROADMAP_DRAFT_V1.md)
2. [AGENTFLOW_NEXT_VERSION_SPEC_CONVERGENCE_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_NEXT_VERSION_SPEC_CONVERGENCE_DRAFT_V1.md)
3. [AGENTFLOW_RUNTIME_FOUNDATION_TECHNICAL_SUPPORT_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_RUNTIME_FOUNDATION_TECHNICAL_SUPPORT_DRAFT_V1.md)
4. [AGENTFLOW_AGENT_PROJECT_OS_ARCHITECTURE_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_AGENT_PROJECT_OS_ARCHITECTURE_V1.md)
5. [AGENTFLOW_ARCHITECTURE_DECISION_RECORD_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_ARCHITECTURE_DECISION_RECORD_DRAFT_V1.md)

再读基础概念草案：

1. [AGENTFLOW_ONTOLOGY_SCHEMA_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_ONTOLOGY_SCHEMA_DRAFT_V1.md)
2. [AGENTFLOW_ACTION_CONTRACT_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_ACTION_CONTRACT_DRAFT_V1.md)
3. [AGENTFLOW_AGENT_ROLE_POLICY_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_AGENT_ROLE_POLICY_DRAFT_V1.md)
4. [AGENTFLOW_OBJECT_STATE_MACHINE_DRAFT_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_OBJECT_STATE_MACHINE_DRAFT_V1.md)

最后读 10 个 issue 技术设计：

1. [AF-OS-001 Ontology Registry](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_AF_OS_001_ONTOLOGY_REGISTRY_TECHNICAL_DESIGN_DRAFT_V1.md)
2. [AF-OS-002 Action Contract](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_AF_OS_002_ACTION_CONTRACT_TECHNICAL_DESIGN_DRAFT_V1.md)
3. [AF-OS-003 Agent Role Policy](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_AF_OS_003_AGENT_ROLE_POLICY_TECHNICAL_DESIGN_DRAFT_V1.md)
4. [AF-OS-004 Object State Machine](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_AF_OS_004_OBJECT_STATE_MACHINE_TECHNICAL_DESIGN_DRAFT_V1.md)
5. [AF-OS-005 Action Arbitration](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_AF_OS_005_ACTION_ARBITRATION_TECHNICAL_DESIGN_DRAFT_V1.md)
6. [AF-OS-006 Event Store Integration](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_AF_OS_006_EVENT_STORE_INTEGRATION_TECHNICAL_DESIGN_DRAFT_V1.md)
7. [AF-OS-007 Projection Read Models](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_AF_OS_007_PROJECTION_READ_MODELS_TECHNICAL_DESIGN_DRAFT_V1.md)
8. [AF-OS-008 Runtime Command API](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_AF_OS_008_RUNTIME_COMMAND_API_TECHNICAL_DESIGN_DRAFT_V1.md)
9. [AF-OS-009 Migration Alignment](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_AF_OS_009_MIGRATION_ALIGNMENT_TECHNICAL_DESIGN_DRAFT_V1.md)
10. [AF-OS-010 Runtime Foundation Closeout](/Users/mac/Documents/AgentFlow/docs/v0.4.0/AGENTFLOW_AF_OS_010_RUNTIME_FOUNDATION_CLOSEOUT_TECHNICAL_DESIGN_DRAFT_V1.md)

## 4. Version Scope

`v0.4.0` 已完成：

- Project Ontology Registry；
- Action Contract；
- Agent Role Policy；
- Object State Machine；
- Action Arbitration；
- Event Store integration；
- Projection read models；
- Runtime Command API；
- migration alignment；
- integration closeout。

`v0.4.0` 未覆盖：

- 行业客户端壳；
- Figma/UI 产品化；
- Message Bus；
- 云端部署；
- Domain Pack 市场；
- 完整多 Agent 并发调度平台。

## 5. Release Closeout

`v0.4.0` 的正式 release 事实：

- tag：`v0.4.0`
- release：`AgentFlow v0.4.0`
- 发布入口：[GitHub Release](https://github.com/atxinbao/agentflow/releases/tag/v0.4.0)

当前版本 closeout 以以下事实为准：

- `CHANGELOG.md`
- `docs/v0.4.0/**`
- `docs/architecture/009-runtime-foundation-closeout-baseline-v1.md`
- GitHub Release notes
