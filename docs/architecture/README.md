# Architecture

更新日期：2026-06-26
执行者：Codex

## Purpose

本目录沉淀 AgentFlow 的长期架构、Core 能力和稳定合同。

```text
docs/architecture
= AI OS Project Core
+ filesystem-first workflow
+ v1.x stable contracts
+ module boundaries
+ design system
+ provider / connector boundaries
```

## Core Architecture

Pack 位置和用户项目暴露边界以 [builtin-pack-registry.md](builtin-pack-registry.md) 为准。旧合同中的项目级 Pack 路径保留为历史实现语义，不作为新建项目可见目录。

| 文档 | 作用 |
| --- | --- |
| [021-ai-os-project-core-capabilities-v1.md](021-ai-os-project-core-capabilities-v1.md) | AI OS Project Core 通用能力 |
| [builtin-pack-registry.md](builtin-pack-registry.md) | App 内置 Pack Registry 与用户项目引用边界 |
| [agentflow-filesystem-workflow-architecture-v1.md](agentflow-filesystem-workflow-architecture-v1.md) | filesystem-first workflow 架构 |
| [stable-contracts.md](stable-contracts.md) | v1.x 稳定架构合同索引 |
| [current-module-boundaries.md](current-module-boundaries.md) | 当前模块边界 |
| [mcp-provider-adapter.md](mcp-provider-adapter.md) | MCP / provider 适配边界 |
| [design-system.md](design-system.md) | 当前设计系统和桌面客户端设计基线 |

## Stable Contracts

| 文档 | 作用 |
| --- | --- |
| [041-v100-stable-contract-baseline-v1.md](041-v100-stable-contract-baseline-v1.md) | v1.0 stable contract baseline |
| [042-v100-runtime-api-sdk-freeze-v1.md](042-v100-runtime-api-sdk-freeze-v1.md) | Runtime API / SDK freeze |
| [043-v100-agentflow-filesystem-contract-freeze-v1.md](043-v100-agentflow-filesystem-contract-freeze-v1.md) | AgentFlow filesystem contract freeze |
| [044-v100-pack-contract-freeze-v1.md](044-v100-pack-contract-freeze-v1.md) | Pack contract freeze |
| [045-v100-projection-readmodel-contract-freeze-v1.md](045-v100-projection-readmodel-contract-freeze-v1.md) | Projection / read model contract |
| [046-v100-evidence-acceptance-contract-freeze-v1.md](046-v100-evidence-acceptance-contract-freeze-v1.md) | Evidence / decision contract |
| [047-v100-executor-adapter-contract-freeze-v1.md](047-v100-executor-adapter-contract-freeze-v1.md) | Executor adapter contract |
| [048-v100-replay-migration-upgrade-certification-v1.md](048-v100-replay-migration-upgrade-certification-v1.md) | Replay / migration / upgrade certification |
| [049-v100-software-dev-pack-stable-baseline-v1.md](049-v100-software-dev-pack-stable-baseline-v1.md) | Software Dev Pack stable baseline |
| [050-v100-release-certification-v1.md](050-v100-release-certification-v1.md) | v1.0 release certification |
| [051-v101-message-bus-no-go-adr-v1.md](051-v101-message-bus-no-go-adr-v1.md) | Message Bus no-go ADR |
| [052-v101-software-dev-pack-usage-baseline-v1.md](052-v101-software-dev-pack-usage-baseline-v1.md) | Software Dev Pack usage baseline |
| [053-core-4d-spec-intake-kernel-v1.md](053-core-4d-spec-intake-kernel-v1.md) | Core 4-D Spec Intake Kernel |
| [054-core-ontology-kernel-contract-v1.md](054-core-ontology-kernel-contract-v1.md) | Core Ontology Kernel Contract |
| [055-core-object-link-schema-v1.md](055-core-object-link-schema-v1.md) | Core Object / Link Schema |
| [056-core-action-state-semantics-v1.md](056-core-action-state-semantics-v1.md) | Core Action / State Semantics |
| [057-core-skill-registry-action-authorization-v1.md](057-core-skill-registry-action-authorization-v1.md) | Core Skill Registry / Action Authorization |
| [058-core-evidence-decision-reference-model-v1.md](058-core-evidence-decision-reference-model-v1.md) | Core Evidence / Decision Reference Model |
| [059-core-file-backed-ontology-registry-projection-v1.md](059-core-file-backed-ontology-registry-projection-v1.md) | Core File-backed Ontology Registry / Projection |
| [060-core-evidence-pack-schema-v1.md](060-core-evidence-pack-schema-v1.md) | Core Evidence Pack Schema |
| [061-core-evidence-source-type-registry-v1.md](061-core-evidence-source-type-registry-v1.md) | Core Evidence Source Type Registry |

## Rules

- `docs/architecture/**` 不直接授权实现。
- 临时技术方案先属于 confirmed Spec Bundle。
- 只有长期有效的架构决策、边界、合同和 ADR 才沉淀到这里。
