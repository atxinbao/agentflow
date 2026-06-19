# AgentFlow v0.4.0 Definition-driven Runtime Foundation

日期：2026-06-20
执行者：Codex
状态：Version Planning Draft / 非执行需求 / 不授权 Build Agent 执行

## 1. Purpose

本目录收口 AgentFlow `v0.4.0` 的版本前置文档。

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

本目录当前只保存版本规划和技术设计草案。

不代表：

- 已写入正式 `docs/requirements/**`；
- 已写入 `.agentflow/spec/**`；
- 已授权 Build Agent 执行；
- 已进入当前 `v0.3.x` 审计或修复流。

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

`v0.4.0` 只做：

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

`v0.4.0` 不做：

- 行业客户端壳；
- Figma/UI 产品化；
- Message Bus；
- 云端部署；
- Domain Pack 市场；
- 完整多 Agent 并发调度平台。

## 5. Next Gate

下一步如果进入正式开发，必须先生成：

```text
SPEC Draft Preview
Project Preview
Issues Preview
```

经人类确认后，才允许写：

```text
docs/requirements/**
.agentflow/spec/projects/**
.agentflow/spec/issues/**
```
