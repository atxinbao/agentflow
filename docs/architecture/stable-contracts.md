# Architecture Docs

更新日期：2026-06-28
执行者：Codex

## Purpose

`docs/architecture/` 保存当前 v1.x 稳定架构合同和运行时边界。

更早的 v0.x / pre-v1 architecture 文档已经移入历史归档：

```text
docs/project/history/2026-06-current-baseline-history/architecture/
```

当前架构基线围绕：

```text
AgentFlow AI OS Project
= Core OS Runtime
+ Industry AgentFlow App
```

## Current Architecture Baseline

Pack 位置和用户项目暴露边界以 [builtin-pack-registry.md](builtin-pack-registry.md) 为准。旧合同中的项目级 Pack 路径保留为历史实现语义，不作为新建项目可见目录。

| 文档 | 作用 |
| --- | --- |
| [040-release-source-agent-entry-v1.md](040-release-source-agent-entry-v1.md) | 定义 release source 中的稳定 Agent entry、tracked docs 映射和 runtime-only 边界 |
| [builtin-pack-registry.md](builtin-pack-registry.md) | 定义 App 内置 Pack Registry、用户项目 active-pack 引用和 Pack 不进入 `docs/project/**` 的边界 |
| [041-v100-stable-contract-baseline-v1.md](041-v100-stable-contract-baseline-v1.md) | 定义 v1.0 Stable Contract Baseline、stable / internal / experimental 边界和 release gate 元数据 |
| [042-v100-runtime-api-sdk-freeze-v1.md](042-v100-runtime-api-sdk-freeze-v1.md) | 定义 v1.0 Runtime API / SDK 的 command、query、event、decision、error 与治理准入冻结合同 |
| [043-v100-agentflow-filesystem-contract-freeze-v1.md](043-v100-agentflow-filesystem-contract-freeze-v1.md) | 定义 v1.0 `.agentflow/` 文件系统 authority、projection、local cache、public record 和 retired path 冻结合同 |
| [044-v100-pack-contract-freeze-v1.md](044-v100-pack-contract-freeze-v1.md) | 定义 v1.0 Pack manifest、Domain、Surface、Connector、capability、migration 与 release gate compatibility 冻结合同 |
| [045-v100-projection-readmodel-contract-freeze-v1.md](045-v100-projection-readmodel-contract-freeze-v1.md) | 定义 v1.0 Projection API、Read Model、View Model、rebuild、freshness、Pack-specific projection 和 sidecar read model 冻结合同 |
| [046-v100-evidence-acceptance-contract-freeze-v1.md](046-v100-evidence-acceptance-contract-freeze-v1.md) | 定义 v1.0 Evidence Pack、Acceptance Gate、Completion Commit、failure reason、delivery record 和 Audit sidecar 冻结合同 |
| [047-v100-executor-adapter-contract-freeze-v1.md](047-v100-executor-adapter-contract-freeze-v1.md) | 定义 v1.0 Executor Adapter、work handoff、diff boundary、session isolation、provider mapping 和 executor result 归一化冻结合同 |
| [048-v100-replay-migration-upgrade-certification-v1.md](048-v100-replay-migration-upgrade-certification-v1.md) | 定义 v1.0 event replay、projection rebuild、Pack migration、retired path 和 upgrade certification 冻结合同 |
| [049-v100-software-dev-pack-stable-baseline-v1.md](049-v100-software-dev-pack-stable-baseline-v1.md) | 定义 v1.0 Software Dev Pack stable baseline、read model、connector、delivery 和 Audit sidecar 认证合同 |
| [050-v100-release-certification-v1.md](050-v100-release-certification-v1.md) | 定义 v1.0 final release certification、v1StableCore、support boundary 和 V100 coverage 认证合同 |
| [051-v101-message-bus-no-go-adr-v1.md](051-v101-message-bus-no-go-adr-v1.md) | 定义 v1.0.1 cross-process Message Bus no-go ADR、go criteria 和非 authority 边界 |
| [052-v101-software-dev-pack-usage-baseline-v1.md](052-v101-software-dev-pack-usage-baseline-v1.md) | 定义 v1.0.1 Software Dev Pack usage flow、authority 边界和 release gate 绑定 |
| [current-module-boundaries.md](current-module-boundaries.md) | 当前 crates 和目录边界的事实快照 |
| [mcp-provider-adapter.md](mcp-provider-adapter.md) | 外部 provider / MCP 适配层边界 |

## Default Reading Order

1. [041-v100-stable-contract-baseline-v1.md](041-v100-stable-contract-baseline-v1.md)
2. [builtin-pack-registry.md](builtin-pack-registry.md)
3. [043-v100-agentflow-filesystem-contract-freeze-v1.md](043-v100-agentflow-filesystem-contract-freeze-v1.md)
4. [044-v100-pack-contract-freeze-v1.md](044-v100-pack-contract-freeze-v1.md)
5. [046-v100-evidence-acceptance-contract-freeze-v1.md](046-v100-evidence-acceptance-contract-freeze-v1.md)
6. [047-v100-executor-adapter-contract-freeze-v1.md](047-v100-executor-adapter-contract-freeze-v1.md)
7. [049-v100-software-dev-pack-stable-baseline-v1.md](049-v100-software-dev-pack-stable-baseline-v1.md)
8. [052-v101-software-dev-pack-usage-baseline-v1.md](052-v101-software-dev-pack-usage-baseline-v1.md)

## Rules

- `docs/architecture/` 只定义当前稳定架构合同，不直接授权实现。
- 从 architecture 进入开发前，必须先转成 `docs/requirements/**` 下的 confirmed Spec Bundle。
- GitHub issues、PR、provider session 和外部工具状态不能成为 AgentFlow authority。
- Audit 继续作为 sidecar，不进入 Software Dev 主业务链路。
- 历史 architecture 文档只能作为参考，不自动继承。
