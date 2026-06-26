# V100 Release Certification V1

日期：2026-06-25
执行者：Codex

releaseCertificationVersion: agentflow-v100-release-certification.v1
releaseCertificationStatus: active
stableContractBaseline: agentflow-stable-contract-baseline.v1
runtimeApiSdkContractVersion: agentflow-runtime-api-sdk-freeze.v1
filesystemContractVersion: agentflow-filesystem-contract-freeze.v1
packContractVersion: agentflow-pack-contract-freeze.v1
projectionContractVersion: agentflow-projection-readmodel-contract.v1
evidenceAcceptanceContractVersion: agentflow-evidence-acceptance-contract.v1
executorAdapterContractVersion: agentflow-executor-adapter-contract.v1
replayMigrationUpgradeCertificationVersion: agentflow-replay-migration-upgrade-certification.v1
softwareDevPackStableBaselineVersion: agentflow-software-dev-pack-stable-baseline.v1
runtimeArtifact: runtime/v100-release-certification.json

## Purpose

本文件定义 `v1.0.0` 的最终发布认证。

它不是新增功能，也不是把后续行业壳提前稳定化。
它只回答一个问题：

```text
AgentFlow Project OS Stable Core 是否已经可以被标记为 v1StableCore = ready。
```

## Certification Inputs

`v1.0.0` 认证必须同时读取并证明以下输入：

- Stable Contract Baseline；
- Runtime API / SDK compatibility；
- AgentFlow filesystem contract；
- Pack contract compatibility；
- Projection / Read Model contract；
- Evidence / Acceptance contract；
- Executor Adapter contract；
- Replay / Migration / Upgrade certification；
- Software Dev Pack stable baseline；
- negative semantic fixture coverage；
- v0.9.0 / v0.9.1 release coverage；
- remaining risk / deferred list。

## V100 Coverage Rule

`V100-001` 到 `V100-009` 都必须有 release gate coverage。

每一项都必须对应一个可复跑 runtime artifact：

```text
V100-001 -> runtime/stable-contract-baseline.json
V100-002 -> runtime/runtime-api-sdk-compatibility.json
V100-003 -> runtime/filesystem-contract.json
V100-004 -> runtime/pack-contract-compatibility.json
V100-005 -> runtime/projection-readmodel-contract.json
V100-006 -> runtime/evidence-acceptance-contract.json
V100-007 -> runtime/executor-adapter-contract.json
V100-008 -> runtime/replay-migration-upgrade-certification.json
V100-009 -> runtime/software-dev-pack-stable-baseline.json
V100-010 -> runtime/v100-release-certification.json
```

缺任一 artifact，`v1StableCore` 必须是 `blocked`。

## v1StableCore Decision

`v1StableCore` 只有两个值：

```text
ready
blocked
```

必须满足以下条件才允许输出 `ready`：

- `v1PlanningReadiness = ready`；
- `V100-001` 到 `V100-009` 全部通过；
- Governance admission 已在 Runtime command admission 主链；
- Projection 仍然是 read-only surface，不能绕过 authority；
- Acceptance Gate 能决定 Done；
- Audit 仍然是 independent sidecar，不进入主业务链；
- Executor runtime 只能执行工作，不能成为 project truth；
- v1 compatibility boundary 清楚。

任一条件不满足，必须输出：

```text
v1StableCore = blocked
```

并在 `v1StableCoreBlockers` 中列出阻断项。

## v1 Support Boundary

`v1.0.0` 的支持边界如下：

- stable core：Project OS runtime stable core；
- stable industry Pack：Software Dev Pack；
- experimental industry Pack：UI Design Pack；
- future Pack compatibility：不承诺；
- Audit：independent sidecar；
- executor runtime：非项目事实源；
- GitHub issue：临时协作镜像，不是 AgentFlow authority；
- Projection / Connector / Industry UI：只读或外部交互面，不拥有 authority；
- Completion authority：Acceptance Gate + Completion Commit。

## Negative Fixture Coverage

release gate 必须保留 negative semantic fixtures。

这些 fixture 要证明错误 happy path 会被阻断：

- 错误 tag / commit / URL；
- fake migration authority；
- invalid Pack / missing definition；
- governance deferred / rejected；
- projection authority uplift；
- executor diff boundary violation；
- Acceptance evidence missing；
- Audit sidecar 被误放回主链。

## Remaining Risk / Deferred List

`v1.0.0` 允许存在非阻断 deferred items，但必须明确写入 release certification：

- live provider smoke 可以是 optional；
- cross-process Message Bus 可以继续是 no-go；
- future Pack marketplace 不属于 v1 stable core；
- cloud Runtime 产品化不属于 v1 stable core；
- long-term commercial SLA 不属于 v1 stable core。

如果 deferred item 影响 stable core authority、Done 决策、Projection 边界或 executor truth boundary，则不能 deferred，必须 blocked。

## Release Gate Artifact

release gate 必须输出：

```text
runtime/v100-release-certification.json
```

该文件必须包含：

- `releaseCertificationVersion`；
- `v1StableCore`；
- `v1StableCoreBlockers`；
- `v100Coverage`；
- `v100CoveragePassed`；
- `v1SupportBoundary`；
- proof path list；
- negative fixture coverage；
- remaining risk / deferred list。

## Non-goals

本认证不做：

- 不替代独立 Audit Agent 流程；
- 不承诺长期商业 SLA；
- 不承诺所有 future Pack 兼容；
- 不把 v1.0.0 当成行业市场发布；
- 不把 Codex / Claude Code / GitHub / GitLab 变成项目事实源。

## Acceptance

- `runtime/v100-release-certification.json` 存在；
- `v1StableCore = ready`；
- `v100CoveragePassed = true`；
- `v1SupportBoundary.v1CompatibilityBoundaryClear = true`；
- `v1SupportBoundary.executorRuntimeOwnsProjectTruth = false`；
- `v1SupportBoundary.auditSidecarIndependent = true`；
- `v1SupportBoundary.projectionAuthority = false`；
- release gate checklist 包含 `v100-release-certification`。
