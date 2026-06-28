# AgentFlow

更新日期：2026-06-28
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

这些 Pack 是 AgentFlow App 的内置能力，不是用户项目里的可见文档目录。用户项目只记录当前启用的 Pack 引用和运行事实。

## Canonical Entries

| 入口 | 作用 |
| --- | --- |
| [docs/project/goal.md](docs/project/goal.md) | 当前项目总目标 |
| [docs/project/roadmap.md](docs/project/roadmap.md) | 从目标到版本阶段的路线图 |
| [docs/architecture/021-ai-os-project-core-capabilities-v1.md](docs/architecture/021-ai-os-project-core-capabilities-v1.md) | AI OS Project 底层通用能力 |
| [docs/architecture/builtin-pack-registry.md](docs/architecture/builtin-pack-registry.md) | App 内置 Pack Registry 边界 |
| [docs/README.md](docs/README.md) | 当前文档地图 |
| [docs/delivery/releases/v1.0.4/README.md](docs/delivery/releases/v1.0.4/README.md) | 当前发布基线 |
| [CHANGELOG.md](CHANGELOG.md) | 当前 changelog 指针 |

## Current Boundary

- `docs/` 面向人类团队、第三方集成方和 Spec Builder。
- `.agentflow/` 面向 Agent、Runtime、Projection、Decision Gate 和 Audit Agent。
- `docs/project/**` 定义产品目标和产品边界。
- `docs/project/roadmap.md` 定义版本路线图，不直接授权实现。
- `docs/architecture/**` 定义底层能力，不直接授权实现。
- 内置 Pack 由 AgentFlow App 管理，不写入 `docs/project/**`。
- `docs/requirements/**` 只保存后续 confirmed Spec Bundle。
- `.agentflow/spec/**` 才是执行合同事实源。
- Audit 是独立 sidecar，不回到主业务链。

历史文档已归档到 [docs/project/history/2026-06-current-baseline-history/](docs/project/history/2026-06-current-baseline-history/)。
