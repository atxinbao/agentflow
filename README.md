# AgentFlow

更新日期：2026-06-26
执行者：Codex

## Project Goal

AgentFlow 的总目标是：

```text
AgentFlow = Spec-Driven Software Dev Workflow
```

AgentFlow 不是 Agent Runner，也不是 Spec 文档生成器。AgentFlow 要做的是让软件开发团队把意图变成可确认的 Spec Bundle，再由 Spec 派生计划、任务、执行、证据、判定、交付和反馈。

## System Formula

```text
AgentFlow AI OS Project
= Spec-Driven Core Runtime
+ Industry AgentFlow Product
```

当前商业产品主线只聚焦 Software Dev：

```text
AgentFlow Software Dev Product
= Core Runtime
+ Software Dev Domain Pack
+ Software Dev Surface Pack
+ Software Dev Connector Pack
```

## Canonical Entries

| 入口 | 作用 |
| --- | --- |
| [docs/product/006-spec-driven-software-dev-product-goal-v1.md](docs/product/006-spec-driven-software-dev-product-goal-v1.md) | 当前项目总目标 |
| [docs/core/021-ai-os-project-core-capabilities-v1.md](docs/core/021-ai-os-project-core-capabilities-v1.md) | AI OS Project 底层通用能力 |
| [docs/industries/software-dev/README.md](docs/industries/software-dev/README.md) | 当前行业产品合同 |
| [docs/README.md](docs/README.md) | 当前文档地图 |
| [docs/releases/v1.0.1/README.md](docs/releases/v1.0.1/README.md) | 当前发布基线 |
| [CHANGELOG.md](CHANGELOG.md) | 当前 changelog 指针 |

## Current Boundary

- `docs/` 面向人类团队、第三方集成方和 Spec Builder。
- `.agentflow/` 面向 Agent、Runtime、Projection、Decision Gate 和 Audit Agent。
- `docs/product/**` 定义产品目标和产品边界。
- `docs/core/**` 定义底层能力，不直接授权实现。
- `docs/industries/**` 定义行业产品合同，不直接授权实现。
- `docs/requirements/**` 只保存后续 confirmed Spec Bundle。
- `.agentflow/spec/**` 才是执行合同事实源。
- Audit 是独立 sidecar，不回到主业务链。

历史文档已归档到 [docs/archive/2026-06-current-baseline-history/](docs/archive/2026-06-current-baseline-history/)。
