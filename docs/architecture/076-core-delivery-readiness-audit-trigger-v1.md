# Core Delivery Readiness / Optional Audit Trigger V1

更新日期：2026-06-29
执行者：Codex

## Purpose

Completion Commit 之后，系统还需要回答两个不同问题：

1. 交付是否已经具备公开记录和可读入口；
2. 是否需要把审计作为 sidecar evaluation 排队。

这两个问题不能混成一个默认主链。

```text
Completion Commit
-> Delivery Readiness Evaluation
-> Optional Audit Sidecar Trigger Evaluation
```

正常 Done 不默认等待审计。只有显式 policy 要求时，audit sidecar 才能进入排队结果。

## Contract Version

```text
agentflow-core-delivery-readiness-audit-trigger.v1
```

## Required Completion Event

```text
subject.completion.committed
```

Delivery readiness 必须发生在 Completion Commit 之后。

## Readiness Outcomes

| Outcome | Delivery ready | Meaning |
| --- | --- | --- |
| `ready` | true | completion event、evidence 和 public record 都存在 |
| `waiting-for-public-record` | false | completion / evidence 存在，但 public record 还没写入 |
| `evidence-missing` | false | completion 存在，但 evidence refs 缺失 |
| `completion-missing` | false | completion event 缺失或类型不可信 |

## Optional Audit Trigger Policy

默认策略：

```text
defaultAuditRequired = false
explicitPolicyRequired = true
sidecarEventType = subject.audit-sidecar.evaluation-queued
sidecarQueueTarget = audit-sidecar
policyBoundAuditMayBlockDone = true
```

含义：

- 默认不要求审计；
- 普通 Done 不被 audit sidecar 阻断；
- 只有显式 policy 绑定时，才会排队 audit sidecar；
- policy 绑定的 audit 可以阻断 done closeout，但这是 policy 例外，不是默认业务链。

## Forbidden Authority Writers

这些对象不能写 Delivery Readiness 或 Done authority：

```text
audit-sidecar
projection
provider-session
```

Audit sidecar 可以输出审计事实，但不能默认替代 Completion Commit 或 Delivery Readiness。

## Validation Rules

1. Contract version 必须是 `agentflow-core-delivery-readiness-audit-trigger.v1`。
2. Delivery Readiness 必须消费 `subject.completion.committed`。
3. `ready` 必须要求 `CompletionEventRef`、`EvidenceRef`、`PublicRecordRef`。
4. 缺 public record 进入 `waiting-for-public-record`。
5. 缺 evidence 进入 `evidence-missing`。
6. 缺 completion event 进入 `completion-missing`。
7. 默认 `defaultAuditRequired` 必须是 false。
8. Audit sidecar trigger 必须有显式 policy。
9. Sidecar queued result 必须只写 sidecar queue / event，不直接写 Done。
10. `doneBlockedByAudit` 只能在 sidecar 已排队时为 true。

## Runtime Artifact

Release gate 必须生成：

```text
runtime/core-delivery-readiness-audit-trigger.json
```

该 artifact 证明：

- Rust contract / validator 存在；
- delivery ready 有明确输入和输出；
- default path 不触发 audit；
- explicit policy path 会排队 audit sidecar；
- missing public record / evidence / completion 都有结构化 failure reason；
- audit sidecar 默认不回到主业务链。

## Non-goals

- 不实现完整 Audit Agent；
- 不把每个 Done 都绑定到 Audit；
- 不写 Software Dev 专属交付规则；
- 不让 projection / provider session 成为 authority；
- 不改变 Completion Commit Authority。
