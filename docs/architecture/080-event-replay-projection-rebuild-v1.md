# Event Replay Projection Rebuild V1

日期：2026-06-30
执行者：Codex

## Purpose

本文件定义 Event Replay 到 Projection Rebuild 的证明合同。

Projection rebuild 的目标不是让当前 UI 状态成为事实源，而是证明：

```text
Event Store / Core fact refs
-> deterministic projection rebuild
-> read model outputs
-> replay receipt
```

## Authority Boundary

Projection rebuild 只能读取：

- `.agentflow/events/**`
- `.agentflow/spec/**`
- `.agentflow/tasks/**`
- `.agentflow/audit/**`
- `.agentflow/release/**`
- `.agentflow/packs/**`

Projection rebuild 只能写：

- `.agentflow/projections/**`
- `.agentflow/indexes/**`

Projection rebuild 不能写：

- `.agentflow/events/**`
- `.agentflow/spec/**`
- `.agentflow/tasks/**`
- `.agentflow/runtime/**`
- `.agentflow/audit/**`
- `.agentflow/release/**`
- `docs/**`

## Replay Report Contract

`projection replay-report` 必须输出：

```text
version
status
sourceRefs
eventCount
taskCount
projectCount
rebuiltPaths
inputDigest
outputDigest
receiptId
deterministic
failures
writesAuthority
projectionAuthority
generatedAt
```

字段规则：

| Field | Rule |
| --- | --- |
| `sourceRefs` | 指向被 replay 消费的 event / fact refs |
| `inputDigest` | 由输入事件序列稳定生成 |
| `outputDigest` | 由 rebuilt projection/index 文件稳定生成 |
| `receiptId` | 由 input/output digest 组合生成 |
| `deterministic` | 成功 rebuild 必须为 `true` |
| `writesAuthority` | 必须为 `false` |
| `projectionAuthority` | 必须为 `false` |

## Failure Rule

以下情况必须失败并输出 structured failure：

- event 输入缺失；
- event JSON 损坏；
- authority fact 缺失；
- rebuild 输出无法读取；
- projection output digest 无法生成。

失败报告必须保留：

```text
status = failed
deterministic = false
writesAuthority = false
projectionAuthority = false
failures[].stage
failures[].message
```

## Determinism Rule

同一组 event / fact refs 重复 rebuild 时：

- `sourceRefs` 必须一致；
- `inputDigest` 必须一致；
- `outputDigest` 必须一致；
- `receiptId` 必须一致；
- `rebuiltPaths` 必须一致。

`generatedAt` 可以变化，不能参与 deterministic receipt。

## Release Gate Evidence

Release gate 必须生成并校验：

```text
runtime/event-replay-projection-report.json
runtime/event-replay-projection-failure-report.json
runtime/projection-readmodel-contract.json
```

quick-audit package 必须包含 happy / failure 两份 replay report。

验收规则：

- happy report `status=passed`；
- happy report 有 `sourceRefs`、`inputDigest`、`outputDigest`、`receiptId`；
- happy report `deterministic=true`；
- failure report `status=failed`；
- failure report `deterministic=false`；
- 两类报告都必须证明 `writesAuthority=false` 和 `projectionAuthority=false`。

## Non-goals

- 不引入默认 Message Bus；
- 不让 Event Store projection output 成为 mutable authority；
- 不把 GitHub issue、provider session 或 Desktop transient state 当作 projection source authority；
- 不改变 Audit sidecar 的独立边界。
