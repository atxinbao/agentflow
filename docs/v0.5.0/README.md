# AgentFlow v0.5.0 Spec Loop Productization

日期：2026-06-20
执行者：Codex
状态：Released Baseline / Release Closeout / 已发布版本文档

## 1. Purpose

本目录收口 AgentFlow `v0.5.0` 的正式发布基线与版本任务结果。

`v0.5.0` 的版本目标是：

```text
Spec Loop Productization
```

它要把 AgentFlow 从 `v0.4.0` 的 Runtime Foundation，推进到可产品化的需求理解与 SPEC 生成链路：

```text
Human Input
-> Intake Artifact
-> Classification Artifact
-> Context Artifact
-> Boundary Artifact
-> Route Artifact
-> Preview Artifact
-> Confirmation Artifact
-> Requirement / Spec / Issue Authority
-> Runtime Action Proposal
```

## 2. Boundary

本目录当前保存 `v0.5.0` 的发布基线、任务范围和收口结论。

它说明的是：

- `v0.5.0` 已经发布；
- Spec Loop 主链已经正式落地；
- `.agentflow/spec/requirements/<requirement-id>/**` 已经成为当前版本的文件化阶段合同事实源；
- 当前文档用于 release closeout、版本回顾和后续版本衔接。

## 3. Reading Order

1. [AGENTFLOW_V0_5_0_SPEC_LOOP_PRODUCTIZATION_TASKS_V1.md](/Users/mac/Documents/AgentFlow/docs/v0.5.0/AGENTFLOW_V0_5_0_SPEC_LOOP_PRODUCTIZATION_TASKS_V1.md)

## 4. Version Scope

`v0.5.0` 已完成：

- Spec Loop Filesystem Contract；
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

`v0.5.0` 未覆盖：

- Build Loop 多 Agent 并发执行；
- 行业客户端壳；
- Domain Pack / Surface Pack / Connector Pack 标准；
- Message Bus；
- 云端部署；
- Desktop OS Console 全量产品化。

## 5. Filesystem-first Constraint

`v0.5.0` 最终落地的硬约束：

```text
Spec Loop 不能只是一组内存模块。
它必须有文件化阶段合同。
```

也就是说，`v0.5.0` 不只是把需求理解链路跑通，还必须回答：

```text
intake / classification / context / boundary / route / preview / confirmation
这些阶段的输入、输出、证据、状态分别落在哪里？
```

当前版本已经把这层文件合同落到 `.agentflow/spec/requirements/<requirement-id>/**`，后续版本继续在这条 filesystem-first 基线上推进。

## 6. Historical Execution Entry

本版执行链当时从：

```text
AF-SPEC-001 Requirement Intake Normalizer
```

开始。

原因是：

- `AF-SPEC-001` 不再只是 Normalizer；
- 它是整条 Spec Loop 的文件合同入口；
- 没有它，Classifier、Context Resolver、Boundary Checker、Route Decider、Preview Generator 都会变成散乱判断。

## 7. Release Closeout

`v0.5.0` 已发布，正式 release 事实：

- tag：`v0.5.0`
- release：`AgentFlow v0.5.0`
- 发布入口：[GitHub Release](https://github.com/atxinbao/agentflow/releases/tag/v0.5.0)

本版的 release closeout 以以下事实为准：

- `CHANGELOG.md`
- `docs/v0.5.0/**`
- GitHub Release notes
- 当前 `main` 上通过的 `release-gate`
