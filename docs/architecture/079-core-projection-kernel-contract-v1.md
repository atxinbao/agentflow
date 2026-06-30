# Core Projection Kernel Contract V1

日期：2026-06-30
执行者：Codex

## Purpose

Projection Kernel 把 Core facts 派生成 read models 和 view models。

Projection Kernel 只回答：

```text
当前事实应该如何展示给人、Desktop、Query API 或 Industry Product Surface？
```

它不回答：

```text
事实应该如何被创建、修改、确认或完成？
```

## Authority Boundary

Projection 不是 authority。

Projection 不允许写入：

- Spec facts；
- Runtime facts；
- Evidence facts；
- Decision facts；
- Completion facts；
- Delivery facts；
- Audit facts。

Projection 可以刷新 `.agentflow/projections/**` 和 `.agentflow/indexes/**`。
这些路径是派生读模型，不是任务事实源。

## Accepted Source Refs

Projection Kernel 只能读取明确的 Core source refs：

| Source ref | Path pattern | Authority |
| --- | --- | --- |
| Spec authority | `.agentflow/spec/**` | Spec Kernel |
| Event authority | `.agentflow/events/**` | Event Store |
| Task evidence authority | `.agentflow/tasks/<issue-id>/evidence/**` | Evidence Kernel |
| Decision authority | `.agentflow/runtime/decisions/**` | Decision Kernel |

Projection 不能把 Provider session、CLI output、GitHub issue、GitLab issue、
Chat thread 或 Desktop transient state 当成 authority。

## Read Model Outputs

Projection Kernel 输出必须带稳定字段：

- `version`
- `status`
- `sourceRefs`
- `readModelVersion`
- `viewModelVersion`
- `freshness`
- `rebuiltAt`

Read model 可以面向不同消费者：

- Desktop task center；
- Project home；
- Runtime health；
- Evidence surface；
- Delivery surface；
- Audit sidecar；
- Industry Product Surface。

## Lifecycle Semantics

Projection lifecycle 固定为：

| State | Meaning |
| --- | --- |
| `fresh` | read model 已按当前 source refs 重建 |
| `stale` | source facts 已变化，read model 需要刷新 |
| `invalid` | 必需 source facts 缺失或互相矛盾 |
| `deferred` | pack-specific projection 暂不可用，但不能 silent fallback |

`invalid` 和 `deferred` 必须展示原因，不能伪装成 `fresh`。

## Forbidden Authority Writes

Projection Kernel 不得提供任何 command 或 write API 来修改：

- `.agentflow/spec/**`
- `.agentflow/events/**`
- `.agentflow/tasks/<issue-id>/evidence/**`
- `.agentflow/runtime/decisions/**`
- completion decision
- public delivery record
- audit report

如果需要改变 authority，必须走对应 Kernel 或 Runtime API。

## Negative Fixtures

Projection Kernel 必须保留负向 fixture：

| Fixture | Rejected ref kind | Rejected target |
| --- | --- | --- |
| `projection-ref-as-authority` | `ProjectionRef` | `Decision` |
| `provider-session-as-authority` | `ProviderSessionRef` | `Completion` |
| `github-issue-as-authority` | `GitHubIssueRef` | `Spec` |

这些 fixture 的结果必须是 `rejected`。

## Release Gate Evidence

Release gate 必须生成：

```text
runtime/core-projection-kernel-contract.json
```

该文件必须证明：

- Projection contract version 是 `projection-kernel-contract.v1`；
- Projection contract status 是 `active`；
- `writesAuthority` 是 `false`；
- accepted source refs 覆盖 Spec / Event / Evidence / Decision；
- forbidden authority writes 覆盖 Spec / Runtime / Evidence / Decision /
  Completion / Delivery / Audit；
- lifecycle semantics 覆盖 `fresh` / `stale` / `invalid` / `deferred`；
- negative fixtures 覆盖 ProjectionRef / ProviderSessionRef / GitHubIssueRef。

## Non-goals

- 不实现 Industry UI；
- 不把 Projection 变成事实源；
- 不让 GitHub issues 成为 AgentFlow authority；
- 不让 provider session 或 CLI session 成为 project truth；
- 不改变 Audit sidecar 的独立边界。
