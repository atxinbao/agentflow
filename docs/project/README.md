# Project

更新日期：2026-07-01
执行者：Codex

## Purpose

本目录定义 AgentFlow 项目本身。

```text
docs/project
= 项目目标
+ 项目路线图
+ 项目上下文
+ 项目术语
+ 历史上下文
```

## Documents

| 文档 | 作用 |
| --- | --- |
| [goal.md](goal.md) | 项目总目标：Spec-Driven AI OS Project 和可交付结果型商业目标 |
| [roadmap.md](roadmap.md) | 从项目目标拆出的版本路线图和 loop 推进顺序 |
| [context.md](context.md) | 当前项目背景、范围、目录平面和行业主线 |
| [glossary.md](glossary.md) | 项目术语表 |
| [history/README.md](history/README.md) | 真实历史迁移和旧文档索引，不属于新建项目默认模板 |

## Boundary

- `docs/project/**` 定义项目，不直接授权实现。
- `docs/project/roadmap.md` 定义版本方向，不直接拆 issue。
- 内置 Pack 不写入 `docs/project/**`；Pack 本体由 AgentFlow App 管理。
- 行业壳源码属于 `products/**`，不是 `docs/project/**`，也不是 `crates/**` Core。
- `docs/project/history/**` 只作为历史参考。
- Agent 角色、任务合同、事件、证据和运行状态属于 `.agentflow/**`。
