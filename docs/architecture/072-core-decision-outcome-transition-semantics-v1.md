# 072 - Core Decision Outcome Transition Semantics V1

日期：2026-06-29
执行者：Codex

## 1. 目标

Core Decision Outcome Transition Semantics 定义 Decision outcome 的含义和允许的状态路线。

它回答：

```text
accepted / rejected / deferred / blocked / needs-fix 分别代表什么？
每个 outcome 允许从哪些 Core state 进入？
每个 outcome 允许转向哪些 Core state？
每个 outcome 必须留下怎样的 reason？
Decision 是否可以直接写 completed？
```

本合同只定义 outcome 到状态路线的授权语义，不写 completion state，不执行 Completion Commit。

## 2. 权威来源

```text
crates/ontology/src/decision.rs
crates/ontology/src/semantics.rs
docs/architecture/072-core-decision-outcome-transition-semantics-v1.md
release-gate runtime/core-decision-outcome-transitions.json
```

## 3. Contract Version

Core Decision Outcome Transition 固定版本：

```text
agentflow-core-decision-outcome-transition.v1
```

## 4. Canonical Outcomes

Core Decision outcome 固定为：

```text
accepted
rejected
deferred
blocked
needs-fix
```

这些 outcome 是 Core 判断结果，不是 Software Dev 状态。

## 5. Outcome Routes

| outcome | allowedFromStates | allowedNextStates | terminal |
| --- | --- | --- | --- |
| `accepted` | `planned`, `reviewing` | `ready` | false |
| `rejected` | `captured`, `understood`, `planned`, `ready`, `reviewing` | `cancelled` | true |
| `deferred` | `captured`, `understood`, `planned`, `ready` | `planned`, `blocked` | false |
| `blocked` | `captured`, `understood`, `planned`, `ready`, `active`, `reviewing` | `blocked` | false |
| `needs-fix` | `active`, `reviewing` | `active` | false |

## 6. Completion Boundary

Decision outcome 不能直接写：

```text
completed
```

Completion state 必须由后续 Completion Commit 边界处理。

## 7. Required Reason Shape

每个 Decision transition attempt 必须包含至少一条 reason。

每条 reason 必须包含：

```text
reasonCode
message
evidenceRefs
blocking
```

`reasonCode` 必须是机器可读字段，不能只写自然语言。

## 8. Illegal Transition Policy

非法迁移必须在写入任何 state authority 前失败。

Release gate 必须证明以下情况会失败：

```text
unknown outcome
completed next state
missing reason
unknown source state or next state
```

## 9. Release Gate

`v1.0.7` release gate 必须生成：

```text
runtime/core-decision-outcome-transitions.json
runtime/core-decision-outcome-transitions-rust-test.log
```

证明内容：

- `agentflow-core-decision-outcome-transition.v1` 已定义；
- canonical outcomes 完整；
- 所有 allowed states 均来自 Core Action / State Semantics；
- `completed` 不在任何 outcome 的 allowed next states；
- reason shape 完整；
- illegal transition negative fixtures 会失败。
