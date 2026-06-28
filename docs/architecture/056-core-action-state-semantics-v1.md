# 056 - Core Action / State Semantics V1

日期：2026-06-28  
执行者：Codex

## 1. 目标

本文件定义 Core Ontology Kernel 之上的 Action / State 语义合同。

它回答两个问题：

```text
Core 对象可以发生哪些通用动作？
Core 对象在生命周期里有哪些通用状态？
```

## 2. 权威边界

Core Action / State semantics 的权威来源：

```text
crates/ontology/src/semantics.rs
docs/architecture/056-core-action-state-semantics-v1.md
release-gate runtime/core-action-state-semantics.json
```

本合同只描述通用对象生命周期，不定义 Software Dev 专有流程。Software Dev Pack 可以把这些语义映射成 issue、PR、release、repository patch、test log 等行业词，但这些映射不是 Core authority。

Machine-readable boundary phrase: reference mappings are not Core authority.

## 3. Core Actions

| Action | Category | Target Object | 说明 |
| --- | --- | --- | --- |
| captureObject | intake | RequestObject | 捕获来自人或系统的对象 |
| normalizeObject | intake | IntentObject | 把对象内容归一化成结构化形式 |
| routeObject | route | IntentObject | 把已理解对象路由到目标边界 |
| acceptObject | decision | DecisionObject | 记录对象边界已被接受 |
| startObject | execution | ExecutionObject | 开始对 ready 对象进行受控工作 |
| attachEvidence | evidence | EvidenceObject | 绑定支撑状态变化的证据 |
| attachArtifact | artifact | ArtifactObject | 绑定持久输出引用 |
| submitForReview | review | ReviewObject | 把 active 对象移入独立 review |
| completeObject | completion | DecisionObject | 在证据满足后记录完成 |
| blockObject | exception | DecisionObject | 记录外部条件阻止推进 |
| cancelObject | exception | DecisionObject | 记录对象不再继续 |
| supersedeObject | exception | DecisionObject | 记录对象被另一个对象替代 |

## 4. Core States

| State | 说明 | Terminal | Blocking |
| --- | --- | --- | --- |
| captured | 已捕获，尚未归一化 | no | no |
| understood | 已归一化，可被路由 | no | no |
| planned | 已有路径，但尚未 ready | no | no |
| ready | 已可进入受控工作 | no | no |
| active | 当前正在处理 | no | no |
| reviewing | 等待独立 review 或接受 | no | no |
| completed | 已接受完成 | yes | no |
| blocked | 外部阻断未清除 | no | yes |
| cancelled | 已停止，不再继续 | yes | no |
| superseded | 已被替代 | yes | no |

## 5. Core Transitions

| Transition | Action | From | To | Required Evidence |
| --- | --- | --- | --- | --- |
| capture | captureObject | empty | captured | none |
| normalize | normalizeObject | captured | understood | none |
| route | routeObject | understood | planned | none |
| accept | acceptObject | planned | ready | DecisionRef |
| start | startObject | ready | active | none |
| attach-evidence | attachEvidence | active | active | EvidenceRef |
| attach-artifact | attachArtifact | active | active | ArtifactRef |
| submit-review | submitForReview | active | reviewing | EvidenceRef |
| complete | completeObject | reviewing | completed | EvidenceRef / DecisionRef |
| block | blockObject | non-terminal | blocked | DecisionRef |
| cancel | cancelObject | non-terminal / blocked | cancelled | DecisionRef |
| supersede | supersedeObject | non-terminal / blocked | superseded | DecisionRef |

## 6. 禁止进入 Core 的行业词

Core Action / State semantics 不得要求：

```text
bug
feature
issue
pr
pull-request
release
repository
repository-patch
test-log
github-issue
```

这些词只能作为 Reference App mapping 或行业 Pack 的输出词。

## 7. Release Gate 证明

release-gate 必须生成：

```text
runtime/core-action-state-semantics.json
runtime/core-action-state-semantics-rust-test.log
```

证明内容必须包含：

- `agentflow-core-action-state-semantics.v1`；
- 12 个 Core actions；
- 10 个 Core states；
- 12 个 Core transitions；
- action required/resulting state 均指向已定义 Core State；
- transition action/from/to 均指向已定义 Core Action / Core State；
- forbidden terms 未污染 Core Action / State semantics。
