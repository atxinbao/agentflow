# 039 - v0.9.0 Release Certification V1

创建日期：2026-06-25
执行者：Codex

## 1. 目标

v0.9.0 release certification 负责把 deployment runtime governance 的九条前置任务收成一份可审计结论。

它回答：

```text
V090-001 到 V090-009 是否都有 release-gate 证据？
当前是否可以进入 v1.0 planning？
哪些东西仍然不能被提升为 authority？
```

## 2. Certification Artifact

release gate 输出：

```text
certification.json
certification.md
```

其中必须包含：

```text
v090Coverage
v1PlanningReadiness
v1PlanningBlockers
messageBusDecisionRecordPath
messageBusDecision
authorityBoundaryCertification
```

## 3. V090 Coverage

coverage 必须覆盖：

```text
V090-001 Local Runtime Boundary
V090-002 Cloud Runtime Boundary
V090-003 Runtime API / SDK Contract
V090-004 Event Replay / Projection Rebuild
V090-005 Pack Migration Execution
V090-006 Simulation Evaluation Layer
V090-007 Runtime Governance Policy
V090-008 Cross-process Scheduling Decision
V090-009 Deployment Evidence and Rollback Model
```

每一项都必须指向具体 release-gate artifact。

## 4. v1.0 Planning Decision

`v1PlanningReadiness` 只能有两种值：

```text
ready
blocked
```

只有当 V090 coverage 全部通过时，才允许标记为 `ready`。

如果有任何覆盖项失败，必须把对应 `V090-*` 写入 `v1PlanningBlockers`。

## 5. Message Bus Decision

Message Bus 不能因为 v0.9.0 发布而默认进入后续版本。

Certification 必须引用：

```text
runtime/scheduling-decision.json
```

当前 v0.9.0 的合法结论是：

```text
decision = no-go
```

含义是：

- 不引入中心化 Message Bus；
- Runtime API 继续作为命令入口；
- Event Store 继续作为 durable authority；
- local in-memory fanout / refresh 继续覆盖当前需要。

## 6. Authority Boundary

Certification 必须明确：

```text
projectionIsAuthority = false
connectorIsAuthority = false
industryUiIsAuthority = false
runtimeApiRemainsAuthorityBoundary = true
eventStoreRemainsDurableAuthority = true
```

也就是说：

- Projection 只能是 read model；
- Connector 只能提供 capability / evidence；
- 行业 UI 只能读 projection 和发 command；
- authority 写入仍必须经过 Runtime API / Event Store / release runtime。

## 7. 非目标

不做：

- 不替代独立 Audit Agent 流程；
- 不把 release certification 当作人工审计报告；
- 不跳过 V090 证据链直接宣称 v1.0 ready；
- 不把 Message Bus、Projection、Connector 或行业 UI 升级为 authority。
