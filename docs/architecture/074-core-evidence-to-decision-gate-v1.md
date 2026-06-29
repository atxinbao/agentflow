# Core Evidence-to-Decision Gate V1

更新日期：2026-06-29
执行者：Codex

## Purpose

Evidence Kernel 只回答证据是否完整、缺失、延期或无效。

Decision Kernel 必须把这个结果转换成稳定 Decision outcome，不能把“有证据文件”直接当成可以接受。

```text
Evidence Completeness Evaluation
-> Evidence-to-Decision Gate
-> Decision Outcome
-> Optional structured Failure Reason
```

## Contract Version

```text
agentflow-core-evidence-to-decision-gate.v1
```

## Consumed Evidence Kernel Version

```text
agentflow-core-evidence-completeness-policy.v1
```

## Gate Rules

| Evidence outcome | Decision outcome | Failure reason | Remediation route |
| --- | --- | --- | --- |
| `complete` | `accepted` | none | none |
| `incomplete` | `deferred` | `evidence-missing` | `collect-evidence` |
| `deferred` | `deferred` | `evidence-deferred` | `wait-for-authority` |
| `invalid` | `rejected` | `evidence-invalid` | `collect-evidence` |
| authority mismatch | `rejected` | `authority-mismatch` | `revise-subject` |

`complete` 是唯一能得到 `accepted-ready` 的 evidence outcome。

## Required Failure Reason Fields

非 `complete` evidence outcome 必须生成结构化 failure reason，字段来自 [073](073-core-decision-failure-reason-remediation-v1.md)：

```text
reasonCode
message
authorityRefs
missingEvidenceRefs
remediationRoute
retryEligible
blocking
```

## Validation Rules

1. Gate 必须消费 `agentflow-core-evidence-completeness-policy.v1`。
2. `complete` 必须映射为 `accepted`，且不能带 failure reason。
3. 非 `complete` 不能映射为 `accepted-ready`。
4. `incomplete` 和 `deferred` 必须进入 `deferred` decision outcome。
5. `invalid` 和 authority mismatch 必须进入 `rejected` decision outcome。
6. 每个非 accepted 结果必须通过 Failure Reason Contract 校验。
7. Decision Gate 不能写 `completed`。
8. Core 合同内不能出现 Software Dev 行业词汇。

## Runtime Artifact

Release gate 必须生成：

```text
runtime/core-evidence-to-decision-gate.json
```

该 artifact 证明：

- Rust contract / validator 存在；
- valid evidence 会生成 accepted-ready；
- missing evidence 会生成 deferred + structured reason；
- fake / invalid evidence 会生成 rejected + structured reason；
- wrong-subject / authority mismatch 会生成 rejected + structured reason；
- 非 complete evidence 无法伪装成 accepted-ready。

## Non-goals

- 不重新运行 provider 工具；
- 不读取 provider / CLI session 作为 authority；
- 不把 evidence existence 当作 acceptance；
- 不实现 Completion Commit；
- 不实现 Audit sidecar。
