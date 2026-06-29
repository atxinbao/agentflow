# Core Decision Projection Read Model V1

更新日期：2026-06-29
执行者：Codex

## Purpose

Decision Kernel 需要一个只读投影，把 Decision、Evidence、Delivery Readiness 和 Optional Audit Sidecar 的结果汇总给 UI / Query API 读取。

这个投影不是新的事实源。

```text
Decision Result
+ Evidence Refs
+ Delivery Readiness Result
+ Optional Audit Sidecar Trigger Result
-> Decision Projection Read Model
```

Projection 只能解释和展示，不能写回 Decision、Completion、Delivery 或 Audit authority。

## Contract Version

```text
agentflow-core-decision-projection-read-model.v1
```

## Readable Sources

Projection 可以读取：

```text
DecisionRef
EvidenceRef
DeliveryReadinessRef
CompletionEventRef
AuditSidecarTriggerRef
```

含义：

- `DecisionRef` 提供 decision status；
- `EvidenceRef` 提供 proof / reason 可追踪入口；
- `DeliveryReadinessRef` 提供 delivery ready / not-ready 状态；
- `CompletionEventRef` 提供 completion authority event；
- `AuditSidecarTriggerRef` 只提供 sidecar trigger 展示，不改变主链 decision status。

## Required Read Model Fields

```text
version
projectionId
subjectRef
decisionRef
decisionStatus
reasonCodes
evidenceRefs
deliveryReady
readinessOutcome
auditSidecar
runtimeState
sourceRefs
authorityBoundary
writeAuthorityAllowed
```

固定规则：

```text
authorityBoundary = read-only-projection
writeAuthorityAllowed = false
```

## Negative Fixtures

Release gate 必须证明这些负向场景不能通过：

| Fixture | Expected result |
| --- | --- |
| missing evidence 被展示成 accepted / ready | fail |
| fake / invalid evidence 被展示成 accepted | fail |
| wrong runtime state 被展示成 completed | fail |
| ProjectionRef 被当成 authority source | fail |
| projection 尝试写 authority | fail |
| audit sidecar 改写 decision status 或 reason | fail |

## Audit Sidecar Boundary

Audit sidecar 可以被展示：

```text
auditSidecar.queued
auditSidecar.eventType
auditSidecar.target
auditSidecar.doneBlockedByAudit
```

但它不能：

- 改写 `decisionStatus`；
- 追加主链 `reasonCodes`；
- 直接写 completion / delivery / audit authority；
- 把普通 Done 变成默认审计阻断。

## Runtime Artifact

Release gate 必须生成：

```text
runtime/core-decision-projection-read-model.json
```

该 artifact 证明：

- Rust contract / validator 存在；
- read model 能展示 decision status、reason codes、evidence refs、delivery readiness 和 audit sidecar view；
- projection 保持 read-only；
- missing evidence、fake evidence、wrong state、projection-as-authority、audit-chain pollution 都有负向夹具；
- Projection 不替代 v1.0.8 的完整 Projection Kernel rebuild。

## Non-goals

- 不实现完整 Projection Kernel rebuild；
- 不定义 Desktop UI；
- 不写 Software Dev 行业专属状态；
- 不让 provider session、GitHub issue、PR 或 audit report 成为 Core authority；
- 不改变 Completion Commit Authority 和 Delivery Readiness Authority。
