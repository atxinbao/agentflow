# 061 - Core Evidence Source Type Registry V1

日期：2026-06-29
执行者：Codex

## 1. 目标

Core Evidence Source Type Registry 定义 Evidence Pack 的来源类型。

Evidence Kernel 需要先知道“这是什么类型的证明”，再判断证据是否完整、缺失、失效或延期。

## 2. 权威来源

```text
crates/ontology/src/evidence.rs
docs/architecture/061-core-evidence-source-type-registry-v1.md
release-gate runtime/core-evidence-source-type-registry.json
```

## 3. Source Types

Registry 必须包含以下 Core source types：

| Source type | Required fields |
| --- | --- |
| `artifact` | `producer` / `subject` / `digest` / `artifactRefs` / `provenance` / `traceRefs` |
| `log` | `producer` / `subject` / `digest` / `artifactRefs` / `provenance` / `traceRefs` |
| `screenshot` | `producer` / `subject` / `digest` / `artifactRefs` / `provenance` / `traceRefs` |
| `external-proof` | `producer` / `subject` / `digest` / `artifactRefs` / `provenance` / `traceRefs` |
| `command-output` | `producer` / `subject` / `digest` / `artifactRefs` / `provenance` / `traceRefs` |
| `diff` | `producer` / `subject` / `digest` / `artifactRefs` / `provenance` / `traceRefs` |
| `provenance` | `producer` / `subject` / `digest` / `artifactRefs` / `provenance` / `traceRefs` |
| `human-confirmation` | `producer` / `subject` / `digest` / `artifactRefs` / `provenance` / `traceRefs` |

## 4. Source Status

所有 source type 必须支持：

```text
collected
missing
invalid
deferred
superseded
```

这些是 source 状态，不是 Decision Kernel outcome。

## 5. Unknown Source Type

未知 source type 必须返回稳定原因：

```text
source-type-unknown
```

Runtime 可以把未知类型 deferred，但不能静默当作 collected。

## 6. Reference App Examples

Software Dev Pack 只能作为 reference app 映射。

| Reference app source | Core source type |
| --- | --- |
| `changed-content-proof` | `diff` |
| `local-command-proof` | `command-output` |
| `ui-proof` | `screenshot` |
| `merge-proof` | `external-proof` |

这些 examples 的状态必须是：

```text
reference-only
```

Reference App examples are not Core authority.

## 7. 非目标

- 不定义 Decision Kernel policy outcomes。
- 不把 Software Dev source types 变成完整 Core model。
- 不把 GitHub / GitLab / PR / issue 作为 Core authority。

## 8. Release Gate

`v1.0.6` release gate 必须生成：

```text
runtime/core-evidence-source-type-registry.json
runtime/core-evidence-source-type-registry-rust-test.log
```

证明内容：

- registry version 是 `agentflow-core-evidence-source-type-registry.v1`；
- 8 个 Core source type 全部存在；
- 5 个 source status 全部存在；
- unknown source type 返回 `source-type-unknown`；
- Software Dev examples 只以 reference app 方式映射到 Core source type。
