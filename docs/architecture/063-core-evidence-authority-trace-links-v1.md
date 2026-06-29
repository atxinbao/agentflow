# 063 - Core Evidence Authority Trace Links V1

日期：2026-06-29
执行者：Codex

## 1. 目标

Evidence 只有能回答“证明了什么”才有意义。

Core Evidence Authority Trace Links 定义 Evidence Pack 如何追溯到运行时权威事实：

```text
Evidence Pack
-> Spec Bundle
-> Task
-> Run
-> Action Proposal
-> Accepted Action
-> evidence.collected event
```

## 2. 权威来源

```text
crates/ontology/src/evidence.rs
crates/event-store/src/model.rs
docs/architecture/063-core-evidence-authority-trace-links-v1.md
release-gate runtime/core-evidence-authority-trace-links.json
```

## 3. Trace Contract

Trace contract 固定为：

```text
agentflow-core-evidence-authority-trace.v1
```

必须包含：

| 字段 | 说明 |
| --- | --- |
| `evidenceId` | 被追溯的 Evidence Pack ID |
| `traceRefs` | Evidence Pack 的 generic trace refs |
| `authorityFacts` | 可验证的权威事实引用 |
| `collectionEvent` | event-store 中的 evidence collection event 链接 |

## 4. Authority Facts

必须至少覆盖以下 fact kind：

```text
SpecBundle
Task
Run
ActionProposal
AcceptedAction
```

每个 authority fact 必须包含：

```json
{
  "factKind": "Task",
  "factRef": "task:core-evidence-pack",
  "authorityPath": ".agentflow/spec/issues/task-core-evidence-pack.json"
}
```

## 5. Orphan Evidence

Evidence 不能脱离 runtime authority。

如果 `traceRefs` 中的 Spec / Task / Run / Action ref 找不到对应 `authorityFacts`，必须返回稳定原因：

```text
evidence-trace-orphaned
```

如果缺少 Action Proposal 或 Accepted Action，必须返回：

```text
evidence-trace-authority-kind-missing:ActionProposal
evidence-trace-authority-kind-missing:AcceptedAction
```

## 6. Evidence Collection Event

event-store 必须公开 evidence collection event：

```text
evidence.collected
```

该事件 payload 必须至少携带：

```text
evidenceId
receiptId
specRefs
taskRefs
runRefs
actionRefs
receiptRef
```

Event link 必须包含：

```text
eventType = evidence.collected
eventRef
eventStorePath = .agentflow/events/task-events.jsonl
correlationId
causationId
```

## 7. 非目标

- 不实现 UI graph visualization。
- 不让 Projection 成为 authority。
- 不把 GitHub / GitLab issue 当成 task authority。
- 不把外部证明 URI 当成本地 authority。

## 8. Release Gate

`v1.0.6` release gate 必须生成：

```text
runtime/core-evidence-authority-trace-links.json
runtime/core-evidence-authority-trace-links-rust-test.log
runtime/core-evidence-authority-trace-links-event-store-rust-test.log
```

证明内容：

- trace contract version 是 `agentflow-core-evidence-authority-trace.v1`；
- fixture 可以从 evidence 追溯到 Spec / Task / Run / Action Proposal / Accepted Action；
- orphan evidence 会失败；
- `evidence.collected` event 被 event-store 归类为 runtime event；
- release gate 不依赖 Projection authority。
