# AgentFlow v0.8.0 Pack System and First Industry Shell

日期：2026-06-23
执行者：Codex
状态：Released Pack System baseline / release closeout record

## Purpose

`v0.8.0` 已把 AgentFlow 从“软件开发场景可用的 Project OS”推进到“可承载多个行业壳的 Pack System”。

一句话：

```text
Runtime Core 不再硬编码行业现场。
行业现场通过 Pack 接入。
```

## Reading Order

1. [AGENTFLOW_V0_8_0_PACK_SYSTEM_TASKS_V1.md](AGENTFLOW_V0_8_0_PACK_SYSTEM_TASKS_V1.md)
2. [../v0.7.2/AGENTFLOW_V0_7_2_FOUNDATION_READINESS_REPORT_V1.md](../v0.7.2/AGENTFLOW_V0_7_2_FOUNDATION_READINESS_REPORT_V1.md)
3. [../architecture/018-api-plane-manifest-v1.md](../architecture/018-api-plane-manifest-v1.md)
4. [../foundation/agentflow-filesystem-workflow-architecture-v1.md](../foundation/agentflow-filesystem-workflow-architecture-v1.md)
5. [../v0.4.0/AGENTFLOW_VERSION_ROADMAP_DRAFT_V1.md](../v0.4.0/AGENTFLOW_VERSION_ROADMAP_DRAFT_V1.md)

## Scope

`v0.8.0` 已完成：

- Pack filesystem contract；
- Pack manifest schema；
- Pack registry；
- Domain Pack contract；
- Surface Pack contract；
- Connector Pack contract；
- Pack validation、versioning、migration preview；
- Pack simulation / dry-run；
- Pack-aware Projection read models；
- Pack-aware Command Surface；
- Software Dev Pack baseline；
- UI Design Pack baseline；
- Pack release gate and readiness certification。

## Non-goals

`v0.8.0` 不包含：

- Cloud Runtime；
- remote Agent fleet；
- Pack marketplace；
- 大规模行业生态；
- 把 Audit 放回软件开发主链；
- 自动远程审计；
- Message Bus 中心化；
- 完整生产级 Figma adapter；
- 完整生产级 GitHub / GitLab / Linear connector 产品化；
- 长任务 provider production E2E。

## Core Boundary

`v0.8.0` 必须保持这条边界：

```text
Pack 描述行业现场。
Runtime Core 执行通用项目闭环。
Projection Surface 展示只读状态。
Command Surface 只发合法命令。
```

Pack 不能直接写 authority。

Pack UI 不能直接写 `.agentflow/**`。

Connector 输出不能直接变成项目事实。

Audit 仍然是独立 sidecar flow，不进入 Software Dev Pack 的主业务链路。

## Industry Shell Pilot

本版本用两个行业壳完成 Pack System 基线证明：

| Industry Shell | Purpose | Boundary |
| --- | --- | --- |
| Software Dev Pack | 把当前软件开发现场 Pack 化，作为第一正式行业壳 | 主链是 Requirement -> Spec -> Issue -> Run -> Acceptance -> Delivery -> Release；Audit 是 sidecar |
| UI Design Pack | 证明 AgentFlow 不是只为写代码设计 | 主链是 Product Brief -> Direction -> Wireframe -> HiFi -> Design System -> Handoff |

## Completion Standard

`v0.8.0` release closeout 已满足：

- Pack manifest 可被机器读取；
- Runtime 可以加载 Pack registry；
- Domain Pack / Surface Pack / Connector Pack 分层清晰；
- Software Dev Pack 能表达当前软件开发项目现场；
- UI Design Pack 能表达独立于代码执行的设计现场；
- Audit 在 Software Dev Pack 中保持 sidecar；
- Projection 通过 Pack-aware read models 展示行业对象；
- Command Surface 通过 Pack-aware command mapping 发起合法命令；
- Pack validation 能拒绝 schema、依赖、capability、surface mapping 不完整的 Pack；
- Pack simulation 能预览 Pack command 的影响，不写 authority；
- release gate 输出 Pack readiness artifact。
