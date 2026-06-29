# Core Completion Commit Authority V1

更新日期：2026-06-29
执行者：Codex

## Purpose

Decision Kernel 只能判断 subject 是否可以继续。它不能直接写 terminal state。

Completion Commit Authority 定义 accepted Decision 之后的唯一完成写入边界：

```text
Accepted Decision
-> Completion Commit Authority
-> Event Store completion event
-> Projection refresh
```

Projection 可以在事件追加后刷新读模型，但不能提交 completion authority。

## Contract Version

```text
agentflow-core-completion-commit-authority.v1
```

## Required Prior Decision

```text
accepted
```

只有 `accepted` Decision 可以进入 Completion Commit。

以下 Decision outcome 都不能写完成态：

- `rejected`
- `deferred`
- `blocked`
- `needs-fix`

## Completion Event

Core 使用领域无关事件类型：

```text
subject.completion.committed
```

该事件由 Event Store 作为 append-only authority 保存。后续 projection 只能读取该事件并刷新展示。

## Required Authority Refs

Completion Commit attempt 必须绑定：

```text
DecisionRef
EvidenceRef
RuntimeStateRef
```

`DecisionRef` 证明已经有 accepted Decision。`EvidenceRef` 证明 completion 不是裸写状态。`RuntimeStateRef` 证明 completion 写入基于当前 runtime 事实。

## Allowed Writers

```text
event-store
runtime-kernel
```

## Forbidden Writers

```text
projection
provider-session
delivery-context
audit-sidecar
```

这些只能提供上下文或展示，不是完成态写入权威。

## Forbidden Write Kinds

```text
projection-read-model
provider-session-record
delivery-record
audit-sidecar-record
```

Completion Commit 不能通过这些写入类型伪造完成。

## Validation Rules

1. Contract version 必须是 `agentflow-core-completion-commit-authority.v1`。
2. Prior Decision outcome 必须是 `accepted`。
3. Completion event type 必须是 `subject.completion.committed`。
4. Requested terminal state 必须是 `completed`。
5. Writer 必须是 `event-store` 或 `runtime-kernel`。
6. Projection / provider session / delivery context / audit sidecar 不能提交 completion。
7. Attempt 必须包含 `DecisionRef` 和 evidence refs。
8. Denied attempt 必须返回结构化 Failure Reason。
9. Projection refresh 只能在 completion event 追加之后发生。

## Runtime Artifact

Release gate 必须生成：

```text
runtime/core-completion-commit-authority.json
```

该 artifact 证明：

- Rust contract / validator 存在；
- accepted Decision 可以生成 completion event；
- rejected / deferred / blocked Decision 不能写完成态；
- projection writer 不能写 completion；
- projection write kind 不能伪造成 authority；
- missing DecisionRef 会被拒绝；
- Event Store 是 completion authority；
- Projection 是 read-only refresh surface。

## Non-goals

- 不实现产品 UI；
- 不运行 provider；
- 不生成 audit report；
- 不让 delivery 或 projection 成为 authority；
- 不改变 Evidence-to-Decision Gate 的 outcome 规则。
