# 029 Pack Release Gate Readiness V1

创建日期：2026-06-23
执行者：Codex

## 目标

把 Pack System 纳入 release gate。

Pack 不能只在 crate 测试里证明可用。发布前必须生成一组可审计的 release gate artifact，说明当前版本内置 Pack 是否能加载、校验、投影、模拟，并且是否已经进入 API Plane。

## Release Gate 产物

`scripts/verify_release_gate.sh` 必须输出这些文件：

```text
pack-registry.json
pack-validation-report.json
pack-simulation-report.json
pack-projection-readiness.json
pack-api-plane-manifest.json
software-dev-pack-readiness.json
ui-design-pack-readiness.json
```

这些文件是 release gate artifact，不是 Runtime authority。

它们只回答一件事：

```text
当前发布版本是否可以把 Pack System 称为 ready / baseline。
```

## 状态词

Pack readiness report 只能使用以下状态：

| 状态 | 含义 |
| --- | --- |
| `completed` | 当前 Pack 已完整纳入 release gate，可作为正式能力发布 |
| `baseline` | 当前 Pack 已具备可用基线，但仍保留后续扩展空间 |
| `deferred` | 当前 Pack 缺少必要证据，不能发布 ready 结论 |
| `carryover` | 当前 Pack 的一部分能力明确转入后续版本 |

`software-dev` 当前为 `completed`。

`ui-design` 当前为 `baseline`。

## Gate 失败规则

只要以下任一条件不满足，release gate 必须失败：

- `pack-validation-report.json.status == passed`
- `pack-simulation-report.json.status == passed`
- `pack-projection-readiness.json.status == passed`
- `pack-api-plane-manifest.json.status == passed`
- `software-dev-pack-readiness.json.status == completed`
- `ui-design-pack-readiness.json.status == baseline`

失败时不能发布 Pack System ready 结论。

## Software Dev Pack Audit Sidecar

Software Dev Pack 的主链是：

```text
Requirement -> Spec -> Issue -> Run -> Acceptance -> Delivery -> Release
```

Audit sidecar 是：

```text
Delivery -> OptionalAuditRequest -> AuditReport -> Finding -> FollowUpProposal
```

`Finding` 不属于 Software Dev Pack 主链阻断条件。

发现问题时，sidecar 只生成 `FollowUpProposal`，不能把已完成的主链任务回滚成未完成。

## CLI 入口

Release gate 使用正式 CLI 入口生成 Pack artifact：

```bash
agentflow pack release-gate-readiness \
  --output-dir <artifact-dir> \
  --runtime-version <version>
```

该命令只读当前内置 Pack 定义、Projection query、Runtime API Plane 和 Simulation 结果，不写 `.agentflow/**` authority。

## 非目标

- 不用 GitHub issue 状态替代 AgentFlow fact source。
- 不把 release gate 等同于完整生产 E2E。
- 不自动触发远程审计。
- 不把 Audit sidecar 变成 Software Dev Pack 主链 blocker。
