# 058 - Core Evidence / Decision Reference Model V1

日期：2026-06-28  
执行者：Codex

## 1. 目标

本文件定义 Core Evidence / Decision Reference Model。

它回答两个问题：

```text
Action 需要引用哪些 Evidence？
Decision 如何用 Evidence 决定完成、拒绝或下一条 Route？
```

## 2. 权威边界

Core Evidence / Decision Reference Model 的权威来源：

```text
crates/ontology/src/decision.rs
docs/architecture/058-core-evidence-decision-reference-model-v1.md
release-gate runtime/core-evidence-decision-reference-model.json
```

本合同只描述通用 proof / judgment flow。Software Dev Pack 可以把 Evidence / Decision 映射成 issue、PR、release、repository patch、test log 等行业词，但这些映射不是 Core authority。

Machine-readable boundary phrase: reference mappings are not Core authority.

## 3. Evidence References

| Evidence | Accepted Ref | Actions | 说明 |
| --- | --- | --- | --- |
| intentEvidence | EvidenceRef | captureObject / normalizeObject / routeObject | 证明输入来源和归一化摘要 |
| decisionEvidence | DecisionRef | acceptObject / blockObject / cancelObject / supersedeObject / completeObject | 证明 actor、reason 和 outcome |
| progressEvidence | EvidenceRef | attachEvidence / submitForReview / completeObject | 证明动作、对象和结果 |
| artifactEvidence | ArtifactRef | attachArtifact | 证明持久输出引用和 producer |
| reviewEvidence | EvidenceRef | submitForReview / completeObject / blockObject | 证明 review 对象和 conclusion |

## 4. Decision References

| Decision | Applies To | Outcomes |
| --- | --- | --- |
| boundaryDecision | acceptObject | accepted / rejected / needsMoreInput |
| routeDecision | routeObject / supersedeObject | routeSelected / routeDeferred / replacementSelected |
| completionDecision | completeObject / blockObject / cancelObject | completed / followUpRequired / blocked / cancelled |

## 5. Outcome Routes

| Outcome | Resulting State | Route Label | Required Evidence |
| --- | --- | --- | --- |
| accepted | ready | continue | decisionEvidence |
| rejected | cancelled | stop | decisionEvidence |
| needsMoreInput | understood | clarify | intentEvidence / decisionEvidence |
| routeSelected | planned | continue | intentEvidence |
| routeDeferred | blocked | wait | intentEvidence / decisionEvidence |
| replacementSelected | superseded | replace | decisionEvidence |
| completed | completed | finish | progressEvidence / decisionEvidence |
| followUpRequired | active | continue | reviewEvidence / decisionEvidence |
| blocked | blocked | wait | decisionEvidence |
| cancelled | cancelled | stop | decisionEvidence |

## 6. 禁止进入 Core 的行业词

Core Evidence / Decision Reference Model 不得要求：

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
runtime/core-evidence-decision-reference-model.json
runtime/core-evidence-decision-reference-model-rust-test.log
```

证明内容必须包含：

- `agentflow-core-evidence-decision-reference-model.v1`；
- 5 个 Core evidence references；
- 3 个 Core decision references；
- Decision outcome 引用的 evidence type 均存在；
- Decision outcome 引用的 resulting state 均来自 Core Action / State Semantics；
- Action reference 均来自 Core Action / State Semantics；
- forbidden terms 未污染 Core Evidence / Decision Reference Model。
