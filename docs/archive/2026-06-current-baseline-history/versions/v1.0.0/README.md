# AgentFlow v1.0.0 Project OS Stable Core

日期：2026-06-25
执行者：Codex
状态：Stable-core release certification baseline

## Purpose

`v1.0.0` 不继续扩功能。

一句话：

```text
把 AgentFlow 从能跑的 Project Runtime，冻结成可依赖的 Project OS Stable Core。
```

`v1.0.0` 的目标是稳定已经形成的核心合约：

- Runtime API / SDK；
- `.agentflow/` filesystem contract；
- Pack contract；
- Projection / Read Model contract；
- Evidence / Acceptance contract；
- Executor Adapter contract；
- Replay / Migration / Upgrade certification；
- Software Dev Pack baseline；
- Release certification。

## Required Precondition

`v1.0.0` 不能绕过 `v0.9.1`。

只有 `v0.9.1` 证明下面问题已经修复后，才允许启动 `v1.0.0` execution：

- Runtime Governance 已经接入 command admission 主链；
- Deployment Evidence 已经从存在性检查升级为语义一致性证明；
- Pack migration 已经区分 receipt-only 和 authority-applied；
- project `.agentflow/packs/**` path 已经被 release gate 证明；
- release source archive 的 Agent entry 自洽；
- negative semantic fixtures 已经阻断错误 happy path。

如果 `v0.9.1` release certification 输出 `v1PlanningReadiness = blocked`，`v1.0.0` 只能保留为规划文档。

## Reading Order

1. [AGENTFLOW_V1_0_0_PROJECT_OS_STABLE_CORE_TASKS_V1.md](AGENTFLOW_V1_0_0_PROJECT_OS_STABLE_CORE_TASKS_V1.md)
2. [../v0.9.1/README.md](../v0.9.1/README.md)
3. [../v0.9.1/AGENTFLOW_V0_9_1_RUNTIME_GOVERNANCE_STABILIZATION_TASKS_V1.md](../v0.9.1/AGENTFLOW_V0_9_1_RUNTIME_GOVERNANCE_STABILIZATION_TASKS_V1.md)
4. [../v0.9.0/README.md](../v0.9.0/README.md)
5. [../foundation/agentflow-filesystem-workflow-architecture-v1.md](../foundation/agentflow-filesystem-workflow-architecture-v1.md)
6. [../architecture/032-runtime-api-sdk-contract-v1.md](../architecture/032-runtime-api-sdk-contract-v1.md)
7. [../architecture/036-runtime-governance-policy-v1.md](../architecture/036-runtime-governance-policy-v1.md)
8. [../architecture/039-v090-release-certification-v1.md](../architecture/039-v090-release-certification-v1.md)
9. [../architecture/041-v100-stable-contract-baseline-v1.md](../architecture/041-v100-stable-contract-baseline-v1.md)
10. [../architecture/042-v100-runtime-api-sdk-freeze-v1.md](../architecture/042-v100-runtime-api-sdk-freeze-v1.md)
11. [../architecture/043-v100-agentflow-filesystem-contract-freeze-v1.md](../architecture/043-v100-agentflow-filesystem-contract-freeze-v1.md)
12. [../architecture/044-v100-pack-contract-freeze-v1.md](../architecture/044-v100-pack-contract-freeze-v1.md)
13. [../architecture/045-v100-projection-readmodel-contract-freeze-v1.md](../architecture/045-v100-projection-readmodel-contract-freeze-v1.md)
14. [../architecture/046-v100-evidence-acceptance-contract-freeze-v1.md](../architecture/046-v100-evidence-acceptance-contract-freeze-v1.md)
15. [../architecture/047-v100-executor-adapter-contract-freeze-v1.md](../architecture/047-v100-executor-adapter-contract-freeze-v1.md)
16. [../architecture/048-v100-replay-migration-upgrade-certification-v1.md](../architecture/048-v100-replay-migration-upgrade-certification-v1.md)
17. [../architecture/049-v100-software-dev-pack-stable-baseline-v1.md](../architecture/049-v100-software-dev-pack-stable-baseline-v1.md)
18. [../architecture/050-v100-release-certification-v1.md](../architecture/050-v100-release-certification-v1.md)

## Scope

`v1.0.0` 只处理稳定核心：

- 冻结 v1 contract baseline；
- 冻结 command / query / event API；
- 冻结 `.agentflow/` filesystem contract；
- 冻结 Domain Pack / Surface Pack / Connector Pack contract；
- 冻结 Projection / Read Model / View Model contract；
- 冻结 Evidence / Acceptance / Completion Commit contract；
- 冻结 Executor Adapter contract；
- 证明 replay、migration、upgrade 可复跑；
- 把 Software Dev Pack 作为 v1 默认稳定行业壳；
- 输出 v1.0.0 release certification。

## Non-goals

`v1.0.0` 不包含：

- 新的大行业市场；
- Pack marketplace；
- 默认中心化 Message Bus；
- 多租户云平台；
- 全新 UI 大改版；
- 把 Audit 放回主业务链；
- 把 GitHub issues 变成 AgentFlow authority；
- 把 Codex / Claude Code session 当成项目事实源；
- 任意绕过 `v0.9.1` readiness gate 的稳定承诺。

## Stable-Core Boundary

`v1.0.0` 之后，AgentFlow 应该能稳定支撑：

```text
Human / Client Input
-> Runtime API / SDK
-> Governance Admission
-> Spec Loop / Build Loop / Acceptance Loop
-> Event Store
-> Projection
-> Evidence / Delivery
-> Optional independent Audit sidecar
```

权威边界必须保持清楚：

- Authority lives in AgentFlow project facts and event records；
- Projection is read-only surface；
- Evidence proves work；
- Acceptance decides Done；
- Audit remains independent sidecar；
- Executor runtime performs work but does not own project truth。

## Release Boundary

`v1.0.0` 发布必须回答：

```text
后续行业壳、客户端、executor adapter 和 deployment shape，能不能在不破坏核心合约的前提下继续演进？
```

如果答案不是明确的 yes，不能发布 `v1.0.0`。
