# 068 - Evidence Projection Read Model V1

日期：2026-06-29
执行者：Codex

## 1. 目标

Evidence Projection Read Model 让人和行业 App 可以读取 Evidence Kernel 的状态，但不直接读取或写入 authority internals。

它只回答：

```text
当前 evidence pack 覆盖了哪些 source type？
这些 evidence trace 到哪些 spec / task / run / action / decision？
当前 completeness policy 是 passed、invalid 还是 deferred？
缺失或无效 evidence 的原因是什么？
```

## 2. 权威来源

```text
crates/ontology/src/evidence.rs
crates/projection/src/query.rs
docs/architecture/068-evidence-projection-read-model-v1.md
release-gate runtime/evidence-projection-read-model.json
```

## 3. Read Model Boundary

Projection read model 固定版本：

```text
evidence-kernel-read-model.v1
```

它必须是：

```text
authority = false
readonly = true
```

Projection 可以读取：

- Core evidence pack；
- Core evidence completeness evaluation；
- Core missing evidence reports；
- trace refs；
- source summaries。

Projection 不允许：

- 写入 evidence pack；
- 写入 completeness decision；
- 把 invalid / missing evidence 标记成 passed；
- 把 Software Dev Reference App 字段提升为 Core-only schema。

## 4. View Model

Evidence Kernel read model 必须包含：

| 字段 | 说明 |
| --- | --- |
| `status` | 对外状态：`passed` / `invalid` / `deferred` |
| `policyId` | 读取到的 completeness policy |
| `sourceSummaries` | evidence id、source type、status、subject、producer role |
| `traceRefs` | spec / goal / task / run / action / decision 引用 |
| `missingReasons` | 缺失或无效 evidence 的稳定原因 |
| `completeness` | Core completeness evaluation 摘要 |
| `authority` | 固定 `false` |
| `readonly` | 固定 `true` |

## 5. 状态映射

Core completeness outcome 到 Projection status 的映射：

| Core outcome | Projection status |
| --- | --- |
| `complete` | `passed` |
| `invalid` | `invalid` |
| `incomplete` | `deferred` |
| `deferred` | `deferred` |

这样可以保证缺失 evidence 不会被 UI 或行业 App 看成 passed。

## 6. Invalid / Missing Fixture

Projection 必须保留两个负向 fixture：

```text
missing evidence -> deferred
invalid evidence -> invalid
```

负向 fixture 必须通过 Core policy 和 Core missing evidence report 推导，不允许 ad hoc 判断。

## 7. Catalog 暴露

Projection surface catalog 必须包含：

```text
kind = evidence-kernel
query = get_evidence_kernel_view
authority = false
```

这表示 Evidence Kernel 可以被 Console / Desktop / industry App 读取，但不是新的事实源。

## 8. Release Gate

`v1.0.6` release gate 必须生成：

```text
runtime/evidence-projection-read-model.json
runtime/evidence-projection-read-model-rust-test.log
```

证明内容：

- Evidence Kernel read model 已定义；
- Projection 可以读取 Evidence Pack 和 completeness state；
- invalid / missing evidence 以 `invalid` / `deferred` 呈现；
- Projection read model 不写 authority；
- Projection surface catalog 暴露 `evidence-kernel` 只读入口。
