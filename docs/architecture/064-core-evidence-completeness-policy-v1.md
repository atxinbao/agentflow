# 064 - Core Evidence Completeness Policy V1

日期：2026-06-29
执行者：Codex

## 1. 目标

Evidence Completeness Policy 定义证据是否足够进入 Decision Kernel。

它不决定任务完成，只回答：

```text
当前 evidence 是否满足本 action / task / route 的证据要求？
```

## 2. 权威来源

```text
crates/ontology/src/evidence.rs
docs/architecture/064-core-evidence-completeness-policy-v1.md
release-gate runtime/core-evidence-completeness-policy.json
```

## 3. Policy Contract

Policy contract 固定为：

```text
agentflow-core-evidence-completeness-policy.v1
```

Policy 必须包含：

| 字段 | 说明 |
| --- | --- |
| `policyId` | 稳定 policy id |
| `routeRef` | 适用 route |
| `actionRef` | 适用 action |
| `requirementGroups` | evidence requirement groups |

## 4. Requirement Groups

Evidence requirement group 支持四类：

| groupKind | 说明 |
| --- | --- |
| `required` | 必须满足，否则 outcome 是 `incomplete` |
| `optional` | 可记录，可缺失，不阻断 completeness |
| `alternative` | 多种 evidence source type 至少满足一种 |
| `deferred` | 可延期，但 outcome 是 `deferred`，不能写 completed state |

每个 group 必须包含：

```text
groupId
groupKind
acceptedSourceTypes
minCollected
deferredReason
```

## 5. Evaluation Outcomes

评估结果只能是：

```text
complete
incomplete
deferred
invalid
```

含义：

- `complete`：required / alternative 都满足，没有 deferred 缺口。
- `incomplete`：required 或 alternative 缺失。
- `deferred`：核心要求满足，但仍有 deferred evidence 缺口。
- `invalid`：policy 或 evidence pack 本身无效。

## 6. Stable Reasons

必须输出稳定机器可读原因：

| 场景 | Stable reason |
| --- | --- |
| required 缺失 | `evidence-required-missing:<group-id>` |
| alternative 缺失 | `evidence-alternative-missing:<group-id>` |
| deferred 缺口 | `evidence-deferred:<group-id>` |
| evidence 无效 | `evidence-invalid:<evidence-id>` |

## 7. Done Boundary

Evidence Completeness Policy 不写 completed state。

```text
complete    -> 允许交给 Decision Kernel
incomplete  -> 不允许交给 Decision Kernel 判断 Done
deferred    -> 不允许写 completed state
invalid     -> 不允许继续 acceptance
```

Decision Kernel 才能做最终 acceptance / done 判断。

## 8. 非目标

- 不决定任务完成。
- 不把 Audit 放回主业务链。
- 不定义行业专属证据类型。
- 不让 Projection authority 参与 policy 判断。

## 9. Release Gate

`v1.0.6` release gate 必须生成：

```text
runtime/core-evidence-completeness-policy.json
runtime/core-evidence-completeness-policy-rust-test.log
```

证明内容：

- policy version 是 `agentflow-core-evidence-completeness-policy.v1`；
- required / optional / alternative / deferred group 都有样例；
- `complete` / `incomplete` / `deferred` / `invalid` 都可被评估；
- missing required evidence 产生稳定原因；
- deferred evidence 不写 completed state。
