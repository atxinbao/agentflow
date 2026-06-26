# Docs Index

更新日期：2026-06-26
执行者：Codex

## 当前基线

本文档目录已按当前版本基线重整。

当前基线是：

```text
AgentFlow = Spec-Driven Software Dev Workflow

AI OS Project
= Core Runtime
+ Industry AgentFlow Product
```

其中：

```text
Core Runtime
= Spec + Ontology + Runtime + Evidence + Decision + Projection
```

```text
Industry AgentFlow Product
= Domain Pack + Surface Pack + Connector Pack
```

## 当前文档入口

| 路径 | 作用 |
| --- | --- |
| `product/` | 当前产品目标和产品边界；Software Dev 是当前商业产品主线 |
| `foundation/` | AI OS Project Core 的底层通用能力和 filesystem-first 边界 |
| `architecture/` | v1.x 当前稳定架构合同和运行时边界 |
| `requirements/` | 后续 confirmed Spec Bundle 的公共记录入口 |
| `v1.0.1/` | 当前发布基线：Release Hardening and Operational Certification |
| `verification/` | 当前验证记录入口 |
| `archive/2026-06-current-baseline-history/` | 当前基线之前的历史文档归档 |
| `archive/2026-05-agentflow-legacy/` | 早期 legacy 项目文档归档 |

## 默认阅读顺序

1. [../README.md](../README.md)
2. [product/README.md](product/README.md)
3. [product/006-spec-driven-software-dev-product-goal-v1.md](product/006-spec-driven-software-dev-product-goal-v1.md)
4. [foundation/README.md](foundation/README.md)
5. [foundation/021-ai-os-project-core-capabilities-v1.md](foundation/021-ai-os-project-core-capabilities-v1.md)
6. [foundation/agentflow-filesystem-workflow-architecture-v1.md](foundation/agentflow-filesystem-workflow-architecture-v1.md)
7. [architecture/README.md](architecture/README.md)
8. [architecture/041-v100-stable-contract-baseline-v1.md](architecture/041-v100-stable-contract-baseline-v1.md)
9. [architecture/049-v100-software-dev-pack-stable-baseline-v1.md](architecture/049-v100-software-dev-pack-stable-baseline-v1.md)
10. [architecture/052-v101-software-dev-pack-usage-baseline-v1.md](architecture/052-v101-software-dev-pack-usage-baseline-v1.md)
11. [v1.0.1/README.md](v1.0.1/README.md)
12. [requirements/README.md](requirements/README.md)
13. [verification/README.md](verification/README.md)
14. [archive/2026-06-current-baseline-history/README.md](archive/2026-06-current-baseline-history/README.md)

## 规则

- `docs/` 面向人类团队、第三方集成方和 Spec Builder。
- `.agentflow/` 面向 Agent、Runtime、Projection、Decision Gate 和 Audit Agent。
- `docs/product/**` 是产品目标和产品边界，不直接授权实现。
- `docs/foundation/**` 是底层能力设计，不直接授权实现。
- `docs/architecture/**` 是 v1.x 当前稳定架构合同，不直接等同于当前迭代需求。
- `docs/requirements/**` 只保存后续 confirmed Spec Bundle；旧 flat requirement records 已移入历史归档。
- `docs/v1.0.1/**` 是当前发布基线；更早版本文档已移入历史归档。
- `docs/archive/**` 只作为历史参考，不自动生成 issue、SPEC、实现任务或 `.agentflow/**` 事实。
- 后续新开发必须先进入 confirmed Spec Bundle，再派生 `.agentflow/spec/**` 执行合同。

## 历史归档

当前基线之前的文档已归档到：

```text
docs/archive/2026-06-current-baseline-history/
```

归档范围包括：

- v0.4.0 到 v1.0.0 的版本计划和发布文档；
- 旧 flat requirements；
- 旧 Project Operating Model 文档；
- 旧 foundation 切片；
- v0.x / pre-v1 architecture 文档；
- 旧 verification gate 文档。

这些文档保留上下文价值，但不再作为当前开发入口。
