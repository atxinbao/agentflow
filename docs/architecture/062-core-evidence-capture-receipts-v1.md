# 062 - Core Evidence Capture Receipts V1

日期：2026-06-29
执行者：Codex

## 1. 目标

Core Evidence Capture Receipt 定义证据被捕获时留下的稳定收据。

它回答：

- 捕获的对象是本地 artifact 还是外部证明引用？
- 这个对象位于哪个 path 或 URI？
- 捕获时有多少 bytes？
- sha256 digest 是什么？
- 谁在什么时候捕获了它？
- 它属于哪个 source type？
- 这份收据应该保留到什么时候？

## 2. 权威来源

```text
crates/ontology/src/evidence.rs
docs/architecture/062-core-evidence-capture-receipts-v1.md
release-gate runtime/core-evidence-capture-receipts.json
```

## 3. Receipt 字段

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `version` | string | 固定为 `agentflow-core-evidence-capture-receipt.v1` |
| `receiptId` | string | 稳定收据标识 |
| `status` | string | 当前只允许 `collected` |
| `location` | object | 本地 path 或外部 URI |
| `byteCount` | number | 捕获时的字节数 |
| `sha256` | string | 捕获内容的 sha256 hex |
| `capturedAt` | string | 捕获时间 |
| `producer` | object | 捕获者 |
| `sourceType` | string | 来自 Core Evidence Source Type Registry |
| `retentionHint` | object | 保留策略提示 |

## 4. 本地 Artifact

本地 artifact 必须使用：

```json
{
  "locationKind": "local-path",
  "path": ".agentflow/tasks/task-core/evidence/evidence.log",
  "uri": null,
  "authority": "local-artifact"
}
```

本地 artifact 校验必须检查：

- `path` 非空；
- `uri` 不存在；
- `authority` 是 `local-artifact`；
- `byteCount` 大于 0；
- `sha256` 与实际 bytes 匹配。

## 5. 外部证明引用

外部证明只能作为 reference，不允许伪装成本地 authority。

```json
{
  "locationKind": "external-uri",
  "path": null,
  "uri": "https://example.invalid/proof/123",
  "authority": "external-reference"
}
```

外部 reference 校验必须检查：

- `uri` 非空；
- `path` 不存在；
- `authority` 是 `external-reference`；
- 不能把外部 bytes 作为本地权威重新校验。

## 6. Stable Negative Reasons

Release gate 必须验证以下 malformed receipt 会失败并返回稳定原因：

| Fixture | Stable reason |
| --- | --- |
| missing digest | `receipt-sha256-missing` |
| empty artifact | `receipt-artifact-empty` |
| wrong digest | `receipt-sha256-mismatch` |
| stale receipt | `receipt-stale` |

## 7. 非目标

- 不建设 artifact storage service。
- 不要求云上传。
- 不让外部 URI 成为本地 authority。
- 不定义 Evidence Completeness Policy。

## 8. Release Gate

`v1.0.6` release gate 必须生成：

```text
runtime/core-evidence-capture-receipts.json
runtime/core-evidence-capture-receipts-rust-test.log
```

证明内容：

- receipt version 是 `agentflow-core-evidence-capture-receipt.v1`；
- local artifact receipt 可以从文件生成；
- `byteCount` 和 `sha256` 可校验；
- digest mismatch 会失败；
- external proof reference 不会被当成本地 authority；
- negative fixtures 返回稳定原因。
