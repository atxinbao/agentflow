# AgentFlow v0.5.1 Release Hygiene & Authority Closure

日期：2026-06-21
执行者：Codex
状态：Historical Remediation Baseline / Folded Into v0.6.0 Release Path

## 1. Purpose

本目录记录 AgentFlow `v0.5.1` 的修复版本目标和历史约束。

`v0.5.1` 不是一个新产品方向版本，而是一个收口版本：

```text
修复 v0.5.0 发布卫生问题
-> 修复 Runtime authority 顺序问题
-> 修复版本文档与版本事实不一致问题
```

历史目标：

```text
先把 v0.5.0 收干净，再进入 v0.6.0。
```

`v0.6.0` 已经发布。本文档不再阻塞已发布事实；未完全收口的问题进入 `v0.6.1`。

## 2. Boundary

`v0.5.1` 的职责是：

- 修复 release metadata；
- 修复 tag / release gate；
- 修复 Spec Loop 专用 gate；
- 修复 arbitration-before-materialization 权威顺序；
- 修复 durable runtime command records；
- 修复 authority manifest；
- 修复版本文档和 release closeout 口径。

它不负责：

- 引入新的 Work Loop 产品能力；
- 启动 `v0.6.0` 的实现；
- 把 Spec Loop 改写成另一套模型。

## 3. Reading Order

1. [AGENTFLOW_V0_5_1_RELEASE_HYGIENE_AUTHORITY_CLOSURE_TASKS_V1.md](AGENTFLOW_V0_5_1_RELEASE_HYGIENE_AUTHORITY_CLOSURE_TASKS_V1.md)

## 4. Issue Chain

`v0.5.1` 修复链：

1. `V051-001 Release Metadata Repair`
2. `V051-002 Tag Release Gate`
3. `V051-003 Spec Loop Gate`
4. `V051-004 Arbitration-before-Materialization`
5. `V051-005 Durable Runtime Command Records`
6. `V051-006 Spec Authority Manifest`
7. `V051-007 Documentation Closeout`

## 5. Completion Standard

`v0.5.1` 作为历史修复链，要求满足：

- release metadata 与 tag / GitHub Release / workspace version 一致；
- release-gate 能覆盖 tag / release 场景；
- Spec Loop 进入 authority write 前，必须先经过 proposal / arbitration；
- runtime command / proposal / decision / accepted action 可持久化追溯；
- preview artifact、authority artifact、derived projection 的边界可见；
- `v0.4.0`、`v0.5.0`、`v0.5.1` 文档状态与实际版本事实一致；
- 已发布的 `v0.6.0` 不再被本文档阻塞；未完成或重新暴露的问题必须进入 `v0.6.1`。

## 6. Relationship To v0.6.0

历史设计上，`v0.6.0` 不应直接建立在 `v0.5.0` 之上。

必须先经过：

```text
v0.5.0 Functional Baseline
-> v0.5.1 Hygiene + Authority Closure
-> v0.6.0 Work Loop Handoff
```
