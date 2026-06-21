# AgentFlow v0.6.1 Release Closeout & Acceptance Gate Refinement

日期：2026-06-21
执行者：Codex
状态：Remediation Planning Draft / v0.6.0 审计后续 / 不授权 Build Agent 执行

## 1. Purpose

本目录收口 `v0.6.0` 发布审计后的修复任务。

`v0.6.1` 不是新方向版本。

它的目标是：

```text
修复 v0.6.0 发布收口问题
-> 把验收作为 Work Loop 主闭环的一等环节
-> 保持 Audit 独立于默认 Done 闭环
```

大白话：

```text
v0.6.0 已经把 Work Loop 受控执行链打出来。
v0.6.1 要把发布事实、版本文档和验收闭环收干净。
```

## 2. Audit Judgment

`v0.6.0` 的功能主线可以保留。

但不建议把 `v0.6.0` 当成 clean stable closeout。

原因：

- GitHub Release 和 tag 已发布；
- `main` release-gate 最终通过；
- V060 实现 PR 已合并；
- 但 tag 内容里的版本元数据、CHANGELOG、v0.6.0 文档状态和 release 事实不一致；
- Work Loop 仍主要表达为 Evidence Gate，不是完整 Acceptance Gate；
- Completion Commit / Delivery Record / Optional Audit Trigger 的权威边界还需要正式化。

## 3. Reading Order

1. [AGENTFLOW_V0_6_0_RELEASE_AUDIT_FINDINGS_V1.md](AGENTFLOW_V0_6_0_RELEASE_AUDIT_FINDINGS_V1.md)
2. [AGENTFLOW_V0_6_1_REMEDIATION_TASKS_V1.md](AGENTFLOW_V0_6_1_REMEDIATION_TASKS_V1.md)

## 4. v0.6.1 Boundary

`v0.6.1` 包含：

- 修复 release metadata 和版本号漂移；
- 修复 CHANGELOG / docs 状态与 release 事实不一致；
- 修复 release-gate 脚本默认版本仍指向 `v0.5.1` 的问题；
- 补正式 v0.6.0 release closeout；
- 将 Evidence Gate 升级为 Acceptance Gate；
- 定义 Completion Commit 的权威写入顺序；
- 定义 Done 后的 optional audit trigger evaluation；
- 补验收闭环测试和 release audit 证据。

`v0.6.1` 不包含：

- 完整 OS Console；
- 行业 Pack；
- 云端 Runtime；
- Message Bus 中心化；
- Eve / Vercel adapter；
- 自动审计；
- 大规模多 Agent 调度。

## 5. Main Chain

`v0.6.1` 要把 `v0.6.0` 主链压成两层。

产品主闭环：

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

Runtime 详细链路：

```text
Confirmed Spec Issue
-> Work Handoff
-> Runtime Admission
-> Work Session
-> Verification Run
-> Evidence Pack
-> Acceptance Gate
-> Completion Commit
-> Done
-> Optional Audit Trigger Evaluation
```

关键规则：

- 验证是技术检查；
- 证据是可追溯材料；
- 验收是 Done 决策；
- 写回是完成落库；
- 审计是 Done 后的独立复查。

## 6. Completion Standard

`v0.6.1` 完成时，必须满足：

- workspace / Desktop / Tauri version 与 release 版本一致；
- CHANGELOG 包含 `0.6.0` closeout 和 `0.6.1` 修复目标；
- `docs/v0.6.0/**` 不再标记为未执行 planning draft；
- `docs/v0.5.1/**` 状态不再阻塞已发布的 `v0.6.0`；
- release-gate 默认版本和 E2E tag / URL 不再硬编码 `v0.5.1`；
- Acceptance Gate 覆盖 Verification / Evidence / Contract / State；
- Completion Commit 明确 Event Store 是 authority，Projection 只是只读派生；
- Done 后不自动创建 Audit，只输出 optional audit trigger evaluation；
- release audit 文档能追溯 tag、release、PR、gate、遗留问题和修复任务。

## 7. Boundary

本目录是 `v0.6.1` 的修复规划和审计后续。

它不代表：

- 已写入正式 `docs/requirements/**`；
- 已写入 `.agentflow/spec/**`；
- 已授权 Build Agent 执行；
- 已创建 GitHub issue；
- 已发布 `v0.6.1`。
