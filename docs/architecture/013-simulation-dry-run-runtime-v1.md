# 013 - Simulation Dry-run Runtime V1

创建日期：2026-06-23
执行者：Codex

## Purpose

本文定义 AgentFlow 的最小 dry-run / simulation 边界。

Simulation 用来回答：

```text
如果执行这个 command / issue / completion，会发生什么？
```

但它不执行真实写入。

## Module Boundary

实现模块：

```text
crates/simulation
```

负责：

- simulate command；
- simulate issue；
- simulate completion；
- 输出 expected events；
- 输出 rejected reasons；
- 输出 affected projections；
- 输出 gate impact；
- 输出 risk / conflict。

不负责：

- 执行 provider；
- 写 `.agentflow/spec/**`；
- 写 `.agentflow/tasks/**`；
- 修改 event store；
- 修改 projection；
- 生成真实 runtime decision。

## Read-only Invariant

Simulation report 固定包含：

```text
writesAuthority = false
writesEventStore = false
executesProvider = false
```

这三个字段是契约，不是运行时建议。

## Command Simulation

Command simulation 复用当前 Runtime API 的 command validation、action proposal mapping 和 action arbitration。

输出：

- validation report；
- proposal；
- arbitration decision；
- expected events；
- rejected reasons；
- affected projection；
- conflict scope；
- gate impact。

它不会调用：

- `prepare_runtime_workspace`
- `write_runtime_command_fact`
- `write_runtime_proposal_fact`
- `write_runtime_decision_fact`
- `append_task_event`
- provider launcher

## Issue Simulation

Issue simulation 面向 Work Loop 启动前检查。

它只检查 dry-run 视角的关键门：

- dependency readiness；
- context pack readiness；
- workspace clean；
- task / project projection impact。

失败只出现在 simulation report 里，不改 issue authority。

## Completion Simulation

Completion simulation 面向 done writeback 前检查。

它只检查：

- validation evidence refs；
- delivery artifact refs；
- PR/MR merge proof ref；
- task projection impact；
- release delivery summary impact。

它不写 Done，也不写 release。

## Acceptance

本边界成立时，应满足：

- simulate 不写 authority；
- simulate 不写 event store；
- simulate 不执行 provider；
- simulate 输出 expected events；
- simulate 输出 rejected reasons；
- simulate 输出 affected projections；
- simulate 输出 risk / conflict；
- simulate 输出 gate impact。
