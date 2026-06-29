# 060 - Core Evidence Pack Schema V1

日期：2026-06-29
执行者：Codex

## 1. 目标

Core Evidence Pack 定义 AgentFlow AI OS Project 的通用证据包。

它回答：

- 谁产生了证据？
- 证据证明哪个对象？
- 证据来源类型是什么？
- 证据内容如何用 digest 固定？
- 证据关联哪些 durable artifacts？
- 证据从哪里采集？
- 证据如何追溯到 Spec / Goal / Task / Run / Action / Decision？

## 2. 权威来源

Core Evidence Pack Schema 的权威来源：

```text
crates/ontology/src/evidence.rs
docs/architecture/060-core-evidence-pack-schema-v1.md
release-gate runtime/core-evidence-pack-schema.json
```

## 3. 边界

Core Evidence Pack 是行业中立合同。

Software Dev Pack 可以把 Core Evidence Pack 映射成 diff、command output、browser proof、PR record 或 deployment proof，但这些映射不是 Core authority。

Reference App mappings are not Core authority.

Audit 只能作为 sidecar evidence consumer。Audit 可以读取 Evidence Pack 并给出判断，但不能替代主链路的 evidence authority。

## 4. Schema 字段

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `version` | string | 固定为 `agentflow-core-evidence-pack.v1` |
| `evidenceId` | string | 稳定证据标识 |
| `status` | string | `collected` / `missing` / `invalid` / `deferred` / `superseded` |
| `producer` | object | 证据生产者 |
| `subject` | object | 被证明对象 |
| `sourceType` | string | 来源类型。具体 registry 由后续 Evidence Source Type Registry 定义 |
| `digest` | object | 证据包自身摘要 |
| `artifactRefs` | array | durable artifact 引用 |
| `provenance` | object | 采集来源和采集方式 |
| `traceRefs` | object | 追溯引用 |

## 5. Producer

```json
{
  "actorRef": "actor:work-agent",
  "roleRef": "role:work",
  "toolRef": "tool:local-validator",
  "producedAt": "2026-06-29T00:00:00Z"
}
```

## 6. Subject

```json
{
  "subjectRefKind": "TaskRef",
  "subjectRef": "task:core-evidence-pack"
}
```

## 7. Digest

Digest 只接受 `sha256`。

```json
{
  "algorithm": "sha256",
  "value": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
}
```

## 8. Artifact Refs

Artifact Ref 必须是 durable output 引用。

```json
{
  "artifactRef": "artifact:core-evidence-pack-canonical",
  "artifactKind": "generic-artifact",
  "digest": {
    "algorithm": "sha256",
    "value": "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789"
  }
}
```

## 9. Provenance

```json
{
  "captureRef": "capture:local-run",
  "captureMethod": "local-runtime",
  "collectedAt": "2026-06-29T00:00:01Z"
}
```

## 10. Trace Refs

Trace refs 必须保留到以下对象类型的关系：

```json
{
  "specRefs": ["spec:core-evidence-pack"],
  "goalRefs": ["goal:evidence-kernel"],
  "taskRefs": ["task:core-evidence-pack"],
  "runRefs": ["run:core-evidence-pack"],
  "actionRefs": ["action:attach-evidence"],
  "decisionRefs": ["decision:accept-evidence"]
}
```

## 11. Negative Fixtures

Release gate 必须验证以下 malformed pack 会失败并返回稳定原因：

| Fixture | Stable reason |
| --- | --- |
| missing evidence id | `evidence-id-missing` |
| missing source type | `source-type-missing` |
| missing digest | `digest-value-invalid` |
| unsupported digest algorithm | `digest-algorithm-unsupported` |
| missing artifact refs | `artifact-refs-missing` |
| missing provenance | `provenance-capture-ref-missing` |
| missing trace refs | `trace-spec-refs-missing` |
| industry term pollution | `forbidden-core-term:github-issue` |

## 12. 非目标

- 不定义 Decision Kernel completion logic。
- 不把 Software Dev evidence fields 变成 Core-only authority。
- 不依赖 GitHub issues 作为 AgentFlow authority。
- 不让 Audit 成为 main-chain authority。

## 13. Release Gate

`v1.0.6` release gate 必须生成：

```text
runtime/core-evidence-pack-schema.json
runtime/core-evidence-pack-schema-rust-test.log
```

该证明必须说明：

- canonical Core Evidence Pack fixture passed；
- invalid fixtures failed with stable reasons；
- schema version 是 `agentflow-core-evidence-pack.v1`；
- schema fields documented and implemented；
- Core surface 未被 Software Dev 行业词污染。
