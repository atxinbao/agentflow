# 070 - Core Decision Model Contract V1

日期：2026-06-29
执行者：Codex

## 1. 目标

Core Decision Model 定义行业无关的 Decision record。

它回答：

```text
Decision 是什么？
Decision 能读取哪些 authority facts？
Decision 必须引用哪些输入？
Decision 可以写什么？
Decision 不能写什么？
Decision 的 outcome 和 reason 如何稳定表达？
```

本合同不实现产品 UI，不执行 completion commit，不把 Audit 放进主链。

## 2. 权威来源

```text
crates/ontology/src/decision.rs
docs/architecture/070-core-decision-model-contract-v1.md
release-gate runtime/core-decision-model-contract.json
```

## 3. Contract Version

Core Decision Model 固定版本：

```text
agentflow-core-decision-model.v1
```

## 4. Decision Record 必填字段

Decision record 必须包含：

| 字段 | 说明 |
| --- | --- |
| `version` | 合同版本 |
| `decisionId` | 稳定决策 id |
| `decidedAt` | 决策时间 |
| `decidedBy` | 决策角色或 actor |
| `subject` | 被判断对象 |
| `inputs` | authority input refs |
| `outcome` | 决策结果 |
| `reasons` | 稳定原因集合 |
| `writes` | 本次 Decision 允许写入的记录 |

## 5. Read Authority

Decision Kernel 只允许读取：

```text
spec
runtimeState
evidence
priorDecision
```

这些输入必须通过 ref 进入：

```text
SpecRef
RuntimeStateRef
EvidenceRef
DecisionRef
```

## 6. Write Authority

Decision Kernel 只允许写：

```text
decision-record
decision-event
```

明确禁止写：

```text
spec-authority
runtime-state-authority
evidence-authority
projection-read-model
provider-session-record
audit-sidecar-record
```

## 7. Outcome Set

Core outcome 固定为：

```text
accepted
rejected
deferred
blocked
cancelled
```

这些 outcome 是 Core 判断结果，不是 Software Dev 行业状态。

## 8. Reason Contract

每条 reason 必须包含：

```text
reasonCode
message
evidenceRefs
blocking
```

`reasonCode` 必须可机器读取，不能只写自然语言说明。

## 9. 禁止进入 Core 的行业词

Core Decision Model 不得要求：

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

这些词只能作为 Reference App mapping 或行业 Pack 输出词。

## 10. Release Gate

`v1.0.7` release gate 必须生成：

```text
runtime/core-decision-model-contract.json
runtime/core-decision-model-contract-rust-test.log
```

证明内容：

- `agentflow-core-decision-model.v1` 已定义；
- required record fields 完整；
- readable authority facts 覆盖 `spec` / `runtimeState` / `evidence`；
- write authority 只允许 `decision-record` / `decision-event`；
- forbidden writes 被显式列出；
- outcome set 完整；
- Software Dev 词汇没有污染 Core Decision Model；
- canonical Decision record fixture 能通过验证；
- unknown outcome / forbidden term fixture 会失败。

