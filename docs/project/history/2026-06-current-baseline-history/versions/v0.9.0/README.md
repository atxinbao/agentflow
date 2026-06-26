# AgentFlow v0.9.0 Deployment Shape and Runtime Governance

日期：2026-06-23
执行者：Codex
状态：Released closeout baseline

## Purpose

`v0.9.0` 已完成 Deployment Shape and Runtime Governance 发布收口。

一句话：

```text
让 AgentFlow Runtime 可以被清楚地部署、重放、迁移、模拟和治理。
```

到 `v0.9.0`，AgentFlow 不再只是本地 Pack baseline。它已经明确本地 Runtime、云端 Runtime、Runtime API / SDK、Event Replay、Migration、Simulation、Governance 和部署证据模型。

## Reading Order

1. [AGENTFLOW_V0_9_0_DEPLOYMENT_RUNTIME_GOVERNANCE_TASKS_V1.md](AGENTFLOW_V0_9_0_DEPLOYMENT_RUNTIME_GOVERNANCE_TASKS_V1.md)
2. [../v0.8.1/README.md](../v0.8.1/README.md)
3. [../v0.8.0/README.md](../v0.8.0/README.md)
4. [../architecture/018-api-plane-manifest-v1.md](../architecture/018-api-plane-manifest-v1.md)
5. [../architecture/032-runtime-api-sdk-contract-v1.md](../architecture/032-runtime-api-sdk-contract-v1.md)
6. [../architecture/012-schema-version-migration-registry-v1.md](../architecture/012-schema-version-migration-registry-v1.md)
7. [../architecture/013-simulation-dry-run-runtime-v1.md](../architecture/013-simulation-dry-run-runtime-v1.md)
8. [../architecture/014-local-message-bus-contract-v1.md](../architecture/014-local-message-bus-contract-v1.md)
9. [../v0.4.0/AGENTFLOW_VERSION_ROADMAP_DRAFT_V1.md](../v0.4.0/AGENTFLOW_VERSION_ROADMAP_DRAFT_V1.md)

## Scope

`v0.9.0` 包含：

- Local Runtime Boundary；
- Cloud Runtime Boundary；
- Runtime API / SDK Contract Hardening；
- Event Replay and Projection Rebuild；
- Ontology / Pack Migration Execution Model；
- Simulation / Evaluation Layer；
- Runtime Governance Policy；
- Cross-process Scheduling Decision Gate；
- Deployment Evidence and Rollback Model；
- Release Certification。

## Non-goals

`v0.9.0` 不包含：

- v1.0 compatibility freeze；
- Pack marketplace；
- 多租户商业权限系统；
- 绑定某一个生产云厂商；
- 自动远程审计；
- 把 Message Bus 作为默认中心化架构；
- 把行业 UI 塞进 Runtime Core；
- 让 connector 直接写 authority；
- 让 Projection 成为事实源。

## Runtime Boundary

`v0.9.0` 必须保持这条边界：

```text
Runtime Core owns command / event / state transitions.
Industry clients own surface and interaction.
Pack owns definitions.
Connector owns external integration.
Projection owns read-only views.
Governance owns policy and admission decisions.
```

云端 Runtime 只能承载 Runtime Core 和 API plane，不能绑定某个行业 UI。

本地 Runtime 可以承载完整 developer experience，但它仍然不能绕过 Runtime authority。
