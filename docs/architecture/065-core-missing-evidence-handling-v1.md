# 065 - Core Missing Evidence Handling V1

日期：2026-06-29
执行者：Codex

## 1. 目标

Missing Evidence Handling 把缺失证据作为一等 runtime 状态处理。

它只回答：

```text
当前缺什么证据？
当前证据为什么不能作为 proof？
需要如何补证据？
这个状态是否允许进入 completed authority？
```

它不伪造证据，不默认触发 Audit Agent，也不决定任务完成。

## 2. 权威来源

```text
crates/ontology/src/evidence.rs
docs/architecture/065-core-missing-evidence-handling-v1.md
release-gate runtime/core-missing-evidence-handling.json
```

## 3. Report Contract

Missing evidence report 固定版本：

```text
agentflow-core-missing-evidence-report.v1
```

每条 report 必须包含：

| 字段 | 说明 |
| --- | --- |
| `reportId` | 稳定 report id |
| `sourceType` | 缺失或无效证据的 source type |
| `expectedProof` | 期望拿到的 proof |
| `currentState` | 当前证据状态 |
| `remediationHint` | 补证据提示 |
| `evidenceRef` | 关联 evidence id，可为空 |
| `outcome` | `incomplete` / `deferred` / `invalid` |
| `reasons` | 稳定机器可读原因 |
| `decisionBoundary` | completed 写入边界 |

## 4. Runtime Outcomes

Missing evidence 只能产生这些状态：

```text
incomplete
deferred
invalid
```

含义：

- `incomplete`：缺少 required / alternative evidence，不允许进入 Done 判断。
- `deferred`：允许延期记录，但不能写 completed state。
- `invalid`：现有 evidence 是伪证、缺 digest 或结构无效，不允许作为 proof。

## 5. Stable Reasons

必须输出稳定机器可读原因：

| 场景 | Stable reason |
| --- | --- |
| required 缺失 | `evidence-required-missing:<group-id>` |
| alternative 缺失 | `evidence-alternative-missing:<group-id>` |
| deferred 缺口 | `evidence-deferred:<id>` |
| fake proof | `evidence-fake-proof:<evidence-id>` |
| missing local file | `evidence-file-missing:<evidence-id>` |
| missing external URL | `evidence-external-url-missing:<evidence-id>` |
| missing digest | `evidence-missing-digest:<evidence-id>` |
| invalid evidence | `evidence-invalid:<evidence-id>` |

## 6. Negative Fixtures

`v1.0.6` 必须覆盖四类负向样例：

```text
fake proof
missing file
missing external URL
missing digest
```

这些样例必须证明：

- fake proof 被判定为 `invalid`；
- missing file / missing external URL 被判定为 `incomplete`；
- missing digest 被判定为 `invalid`；
- 所有结果都不能写 completed state。

## 7. Completed Boundary

Missing evidence report 的固定边界是：

```text
missing-evidence-does-not-write-completed-state
```

任何 `incomplete`、`deferred`、`invalid` report 都不能直接写 task / run completed authority。

Decision Kernel 后续只能在 evidence 状态被补齐后继续判断。

## 8. 非目标

- 不自动生成 fake evidence。
- 不默认调用 Audit Agent。
- 不写 task / run completed state。
- 不替代 Evidence Completeness Policy。

## 9. Release Gate

`v1.0.6` release gate 必须生成：

```text
runtime/core-missing-evidence-handling.json
runtime/core-missing-evidence-handling-rust-test.log
```

证明内容：

- missing evidence report contract 已定义；
- source type、expected proof、current state、remediation hint 都存在；
- fake proof / missing file / missing external URL / missing digest 都有负向样例；
- missing evidence 只能返回 incomplete / deferred / invalid；
- missing evidence 不写 completed state。
