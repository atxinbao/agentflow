# AgentFlow v0.9.1 Runtime Governance Stabilization

日期：2026-06-25
执行者：Codex
状态：Planned remediation baseline

## Purpose

`v0.9.1` 是 `v0.9.0` 发布审计后的修复版本。

一句话：

```text
先把 v0.9.0 的 governance、deployment evidence、migration、Pack registry 和 release source 边界修实，再进入 v1.0 planning。
```

`v0.9.0` 已经完成 Deployment Shape and Runtime Governance 的功能主线，但审计发现部分能力仍停留在 report / fixture / receipt 级别。`v0.9.1` 负责把这些能力接回真实 Runtime 闭环。

## Reading Order

1. [AGENTFLOW_V0_9_1_RUNTIME_GOVERNANCE_STABILIZATION_TASKS_V1.md](AGENTFLOW_V0_9_1_RUNTIME_GOVERNANCE_STABILIZATION_TASKS_V1.md)
2. [../v0.9.0/README.md](../v0.9.0/README.md)
3. [../v0.9.0/AGENTFLOW_V0_9_0_DEPLOYMENT_RUNTIME_GOVERNANCE_TASKS_V1.md](../v0.9.0/AGENTFLOW_V0_9_0_DEPLOYMENT_RUNTIME_GOVERNANCE_TASKS_V1.md)
4. [../architecture/032-runtime-api-sdk-contract-v1.md](../architecture/032-runtime-api-sdk-contract-v1.md)
5. [../architecture/036-runtime-governance-policy-v1.md](../architecture/036-runtime-governance-policy-v1.md)
6. [../architecture/038-deployment-evidence-rollback-model-v1.md](../architecture/038-deployment-evidence-rollback-model-v1.md)
7. [../architecture/039-v090-release-certification-v1.md](../architecture/039-v090-release-certification-v1.md)

## Scope

`v0.9.1` 只处理 `v0.9.0` 审计发现的修复项：

- release source 中 Agent entry 与 tracked docs 的自洽；
- Runtime Governance 接入 command admission 主链；
- Deployment Evidence 从存在性证明升级为语义一致性证明；
- Pack migration apply / rollback 区分 receipt-only 和 authority-applied；
- release gate 使用项目级 `.agentflow/packs/**` fixture 证明 project Pack path；
- negative semantic fixtures 覆盖错误 tag、错误 commit、禁用 capability 仍执行、伪 migration receipt；
- `v0.9.1` release certification 证明 `v0.9.x` 可以作为 `v1.0` planning 地基。

## Non-goals

`v0.9.1` 不包含：

- `v1.0.0` compatibility freeze；
- Pack marketplace；
- 多租户商业权限系统；
- 默认启用中心化 Message Bus；
- 绑定某一个云厂商；
- 自动远程审计；
- 新行业壳；
- 大规模 UI 改版；
- 手写 `.agentflow/spec/**` 或 `.agentflow/tasks/**` 事实。

## Release Boundary

`v0.9.1` 通过后，才允许把 `v0.9.x` 视为进入 `v1.0` planning 的稳定地基。

最低证明链：

```text
Runtime command
-> Governance admission
-> Proposal / Arbitration
-> Event Store
-> Projection
-> Deployment evidence semantic certification
-> Release certification
```

如果 Governance 仍只是独立报告，或 deployment / migration 仍只证明文件存在，`v0.9.1` 不能标记为 clean remediation release。

