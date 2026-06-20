# AgentFlow v0.5.0 Spec Loop Productization

日期：2026-06-20
执行者：Codex
状态：Version Planning Draft / 开发前置文档 / 不授权 Build Agent 执行

## 1. Purpose

本目录收口 AgentFlow `v0.5.0` 的版本开发任务规划。

`v0.5.0` 的版本目标是：

```text
Spec Loop Productization
```

它要把 AgentFlow 从 `v0.4.0` 的 Runtime Foundation，推进到可产品化的需求理解与 SPEC 生成链路：

```text
Human Input
-> Requirement Intake
-> Requirement Classification
-> Context Resolution
-> Boundary Check
-> Route Decision
-> Preview Generation
-> Confirmation Gate
-> Spec Materialization
-> Runtime Action Proposal
```

## 2. Boundary

本目录当前只保存 `v0.5.0` 的版本任务规划。

不代表：

- 已写入正式 `docs/requirements/**`；
- 已写入 `.agentflow/spec/**`；
- 已授权 Build Agent 执行；
- 已进入 Build Loop 多 Agent 并发开发；
- 已进入行业 Pack 或 UI Console 产品化。

后续进入正式开发前，仍必须先生成：

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

## 3. Reading Order

1. [AGENTFLOW_V0_5_0_SPEC_LOOP_PRODUCTIZATION_TASKS_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.5.0/AGENTFLOW_V0_5_0_SPEC_LOOP_PRODUCTIZATION_TASKS_V1.md)

## 4. Version Scope

`v0.5.0` 只做：

- Requirement Intake Normalizer；
- Requirement Classifier；
- Context Resolver；
- Boundary Checker；
- Route Decider；
- Preview Generator；
- Confirmation Gate；
- Spec Materializer；
- Spec-to-Action Proposal Bridge；
- Spec Loop Projection and Acceptance。

`v0.5.0` 不做：

- Build Loop 多 Agent 并发执行；
- 行业客户端壳；
- Domain Pack / Surface Pack / Connector Pack 标准；
- Message Bus；
- 云端部署；
- Desktop OS Console 全量产品化。

## 5. First Executable Candidate

第一条可执行任务应从：

```text
AF-SPEC-001 Requirement Intake Normalizer
```

开始。

原因很简单：Normalizer 是 Spec Loop 的输入合同。没有稳定输入合同，Classifier、Boundary Checker、Route Decider、Preview Generator 都会变成散乱判断。
