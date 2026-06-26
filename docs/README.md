# Docs Index

更新日期：2026-06-26
执行者：Codex

## Current Baseline

```text
AgentFlow = Spec-Driven Software Dev Workflow

AgentFlow AI OS Project
= Spec-Driven Core Runtime
+ Industry AgentFlow Product
```

当前 `docs/` 只保留 6 个顶层目录：

```text
docs/
  product/       # 产品目标
  core/          # 底层 Core 与稳定架构合同
  industries/    # 行业 AgentFlow Product 标准
  requirements/  # confirmed Spec Bundle 公共记录
  releases/      # 当前发布基线
  archive/       # 历史文档
```

## Current Entries

| 路径 | 作用 |
| --- | --- |
| `../README.md` | 根目录总目标入口 |
| `../AGENTS.md` | release source archive 中的 Agent 入口 |
| `../CHANGELOG.md` | 当前 changelog 指针 |
| `product/` | 产品目标和产品边界 |
| `core/` | AI OS Project Core、filesystem-first 边界和 v1.x 稳定架构合同 |
| `industries/` | 行业产品标准目录；当前只定义 Software Dev |
| `requirements/` | 后续 confirmed Spec Bundle 公共记录入口 |
| `releases/` | 当前 release baseline |
| `archive/` | 当前基线之前的历史文档 |

## Default Reading Order

1. [../README.md](../README.md)
2. [product/006-spec-driven-software-dev-product-goal-v1.md](product/006-spec-driven-software-dev-product-goal-v1.md)
3. [core/021-ai-os-project-core-capabilities-v1.md](core/021-ai-os-project-core-capabilities-v1.md)
4. [industries/README.md](industries/README.md)
5. [industries/software-dev/README.md](industries/software-dev/README.md)
6. [core/architecture/041-v100-stable-contract-baseline-v1.md](core/architecture/041-v100-stable-contract-baseline-v1.md)
7. [releases/README.md](releases/README.md)
8. [releases/v1.0.1/README.md](releases/v1.0.1/README.md)
9. [requirements/README.md](requirements/README.md)
10. [../CHANGELOG.md](../CHANGELOG.md)
11. [archive/2026-06-current-baseline-history/README.md](archive/2026-06-current-baseline-history/README.md)

## Rules

- `docs/product/**` 定义产品目标，不直接授权实现。
- `docs/core/**` 定义底层能力和稳定架构合同，不直接授权实现。
- `docs/industries/**` 定义行业产品合同，不直接写 `.agentflow/**`。
- `docs/requirements/**` 只保存 confirmed Spec Bundle。
- `docs/releases/**` 只保存当前 release baseline 和发布任务。
- `docs/archive/**` 只作为历史参考，不自动生成 issue、SPEC、实现任务或 `.agentflow/**` 事实。
- 后续新开发必须先进入 confirmed Spec Bundle，再派生 `.agentflow/spec/**` 执行合同。

## Historical Archive

当前基线之前的文档已归档到：

```text
docs/archive/2026-06-current-baseline-history/
```

归档范围包括旧版本计划、旧 flat requirements、旧 Project Operating Model、旧 foundation 切片、旧 pre-v1 architecture、旧 verification gate、完整历史 changelog 和历史根目录 `design.md`。
