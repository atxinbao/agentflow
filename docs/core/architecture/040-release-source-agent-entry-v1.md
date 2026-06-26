# Release Source Agent Entry V1

日期：2026-06-25
执行者：Codex

## Goal

让 release source archive 自带一个可读的 Agent entry，同时不把本地
runtime facts 提交进源码包。

这个入口解决的问题是：

```text
源码包下载后，Agent / reviewer 应该知道先读哪里。
本地 .agentflow 运行态缺失时，源码包仍然能说明 AgentFlow 的产品、领域和技术边界。
```

## Source Entry

根目录 `AGENTS.md` 是 release source 中唯一稳定 Agent entry。

它只做三件事：

1. 指向 tracked docs。
2. 说明 `.agentflow/**` 中哪些内容不是 release source authority。
3. 说明 `.agentflow/define/agent/**` 是本地 materialized manual，不替代 tracked docs。

## Tracked Documentation Targets

`AGENTS.md` 必须至少指向这些 tracked docs：

| Path | Role |
| --- | --- |
| `docs/product/README.md` | 产品方向 |
| `docs/core/README.md` | 领域基础 |
| `docs/industries/software-dev/README.md` | 当前 Software Dev 行业产品合同 |
| `docs/core/architecture/README.md` | 技术底座入口 |
| `docs/core/architecture/current-module-boundaries.md` | 当前模块边界事实 |
| `docs/product/006-spec-driven-software-dev-product-goal-v1.md` | 当前产品目标 |
| `docs/core/021-ai-os-project-core-capabilities-v1.md` | 当前 AI OS Project Core 能力定义 |
| `docs/releases/v1.0.1/README.md` | 当前 release hardening baseline |
| `docs/core/architecture/050-v100-release-certification-v1.md` | v1 release certification 边界 |
| `docs/archive/2026-06-current-baseline-history/README.md` | 历史文档入口 |

这些文件是 release source 可读入口。它们可以被 GitHub source archive、
tag checkout 和本地 checkout 直接读取。

## Local Runtime Boundary

以下路径保持本地运行态，不进入 release source authority：

```text
.agentflow/runs/**
.agentflow/tmp/**
.agentflow/tasks/**
.agentflow/index.sqlite
.agentflow/index.sqlite-*
```

`.agentflow/define/agent/**` 的定位是：

```text
tracked docs
-> runtime materialization
-> .agentflow/define/agent/**
```

也就是说，`.agentflow/define/agent/**` 可以作为本地运行时手册快照，但不是源码包
必须携带的 authority。release source 的可读入口由 `AGENTS.md` 和 tracked docs
承担。

## Release Gate

release gate 必须验证：

1. `AGENTS.md` 存在于 release source checkout。
2. `AGENTS.md` 指向的 tracked docs 存在。
3. `AGENTS.md` 不再把 `docs/v0.9.1/README.md` 当成当前稳定入口。
4. runtime-only paths 没有被 Git 跟踪。
5. gate 输出 `runtime/source-agent-entry.json` 作为证明。

失败时，release gate 应停在 `source.agent-entry` 阶段。

## Non-goals

- 不把 `.agentflow/tasks/**`、run、evidence、local DB 或 tmp 文件提交进源码包。
- 不把 `.agentflow/define/agent/**` 升级成 release source authority。
- 不新增另一套 Agent role 文档源。
