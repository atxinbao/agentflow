# AgentFlow v0.6.0 Work Loop Handoff & Controlled Execution

日期：2026-06-21
执行者：Codex
状态：Released Functional Baseline / Release Closeout / v0.6.1 Remediation Required

## 1. Purpose

本目录沉淀 AgentFlow `v0.6.0` 的版本目标、功能发布基线和 release closeout 口径。

`v0.6.0` 的核心目标是：

```text
Spec Loop -> Work Loop Handoff & Controlled Execution
```

大白话：

```text
v0.5.0 负责把人类需求变成确认后的 SPEC 任务。
v0.6.0 负责让这些任务被 Work / Build Agent 安全执行、可验证、可写回。
```

`v0.6.0` 仍然处理 Build Loop，但不是直接扩成完整多 Agent 平台。

它要先解决：

- 任务能不能开始；
- 谁可以执行；
- 依赖是否已完成；
- 是否已有 Agent 正在改同一个对象；
- 执行过程是否有 durable session；
- 验证证据是否足够；
- 状态能不能合法迁移；
- Done 写回是否完整；
- Done 写回是否仍然不自动触发 Audit。

## 2. Baseline

`v0.6.0` 建立在以下版本基线上：

- `v0.4.0`：Runtime Foundation，提供 Ontology、Action Contract、Role Policy、Object State、Arbitration、Event Store、Projection、Runtime API。
- `v0.5.0`：Spec Loop Productization，提供 intake、classification、context、boundary、route、preview、confirmation、materialization、Spec-to-Action Proposal。

`v0.6.0` 发布前的历史前置条件是：

- `v0.5.1` 修复 release metadata / tag gate / release gate；
- Spec materialization 必须先经过 Arbitration，再写 authority；
- Runtime command / proposal / decision / action records 必须可持久化；
- Spec authority manifest 必须明确每层 authority 和派生视图边界。

`v0.6.0` 已发布后，剩余 release hygiene 和 authority closeout 缺口不再回滚版本事实，而是进入 `v0.6.1` 修复链。

## 3. Main Chain

`v0.6.0` 的主链分两层表达。

产品主闭环给人看，必须足够简单：

```text
确认任务
-> 准入
-> 执行
-> 验证
-> 证据归档
-> 验收判定
-> 完成写入
-> Done
```

Runtime 详细链路给实现看，必须说明每一步的权威边界：

```text
Confirmed Spec Issue
-> Work Handoff
   - Work Command
   - Work Action Proposal
   - Expected Outputs
   - Acceptance Criteria
   - Evidence Policy
-> Runtime Admission
   - Arbitration
   - Issue Preflight
   - Lock / Queue
-> Work Session
-> Verification Run
-> Evidence Pack
-> Acceptance Gate
   - Verification Gate
   - Evidence Gate
   - Contract Gate
   - State Gate
-> Completion Commit
   - Accepted Action
   - Event Store
   - Issue / Run Status Writeback
   - Projection Refresh
   - Delivery Record
-> Done
```

每一步都要能回答：

```text
谁发起？
谁裁决？
证据在哪里？
```

## 4. Scope

`v0.6.0` 包含：

- Work Loop filesystem contract；
- Spec Issue 到 Work Command 的 handoff；
- Work Agent Action Proposal contract；
- Issue preflight runtime gate；
- Issue / Object lock and lease；
- Dependency queue and next issue selection；
- Acceptance Gate：verification、evidence、contract、state 汇总判定；
- Work state transition enforcement；
- Durable work session and recovery；
- Work Loop event model and projection；
- Controlled multi-agent proposal arbitration；
- Completion Commit / delivery record / optional audit trigger boundary。

## 5. Non-goals

`v0.6.0` 不包含：

- 完整云端调度平台；
- Message Bus 中心化主链；
- 行业 Pack；
- 完整 OS Console；
- Eve / Vercel adapter；
- 自动审计；
- 行业客户端壳；
- Pack 市场；
- 云端多租户；
- 大规模多 Agent 抢占式并发。

## 6. Reading Order

1. [AGENTFLOW_V0_6_0_WORK_LOOP_HANDOFF_TASKS_V1.md](AGENTFLOW_V0_6_0_WORK_LOOP_HANDOFF_TASKS_V1.md)

## 7. Historical Development Entry

本版当时从以下任务开始执行：

```text
V060-001 Work Loop Filesystem Contract / CodeFlow Contract
```

原因：

- 没有 Work Loop 文件合同，handoff、session、evidence、lock、queue 会各写各的；
- 没有 CodeFlow contract，Build Agent 执行过程仍然只是外部聊天线程；
- 没有清晰文件边界，后续 Projection、Done writeback 和审计隔离都会失真。

## 8. Completion Standard

`v0.6.0` 发布时已经建立以下功能基线：

- 确认后的 spec issue 可以变成 Work Command；
- Work Command 必须生成 Work Action Proposal；
- Work Action Proposal 必须经过 Arbitration；
- Issue preflight 能阻止非法执行；
- Issue / Object lock 能阻止冲突写入；
- Dependency queue 能找到下一条合法可执行 issue；
- Verification Gate 能阻止验证失败的任务进入 Done；
- Evidence Gate 能阻止无证据 Done；
- Contract Gate 能阻止未满足任务合同的任务进入 Done；
- State Gate 能阻止非法状态跳转；
- Acceptance Gate 汇总 gate 结果并给出最终 Done 判定；
- 状态迁移必须符合 Object State / Work State 合同；
- Work session 可以中断、恢复、重试；
- Work Loop 事件可以被 Projection 重建；
- 多 Agent 只能提交 proposal，不能绕过 Runtime；
- Completion Commit 完成权威写入和 delivery record；
- Done 后只做可选 audit trigger evaluation，不自动触发 Audit。

## 9. Boundary

本目录不再是未执行规划稿。

它现在代表：

- `v0.6.0` 已发布为 Work Loop Handoff & Controlled Execution 功能基线；
- GitHub tag / Release 已经存在；
- V060 任务链已经合并到 main；
- 当前文档用于说明 release closeout、功能边界和后续修复入口。

它仍不代表 clean stable closeout。

`v0.6.0` 发布后发现的问题进入 `v0.6.1`：

- release metadata 曾与 tag / release fact 漂移；
- `CHANGELOG.md` 和版本文档状态需要补齐；
- release-gate 默认版本和 release fixture 需要参数化；
- Acceptance Gate 需要进一步细化；
- Completion Commit authority 顺序需要正式化。
