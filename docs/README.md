# Docs Index

更新日期：2026-06-20
执行者：Codex

## 当前文档入口

| 路径 | 作用 |
| --- | --- |
| `product/` | AgentFlow Project Operating Model 产品设计基线 |
| `foundation/` | 下一代 Project Operating Model 的基础能力需求 |
| `architecture/` | AgentFlow 底层技术蓝图、模块边界与运行时模型 |
| `verification/` | Gate、E2E 证据链和正式验证入口说明 |
| `v0.4.0/` | v0.4.0 Definition-driven Runtime Foundation 版本规划与技术设计草案 |
| `v0.5.0/` | v0.5.0 Spec Loop Productization 版本规划与任务草案 |
| `requirements/` | 新需求文档入口，后续开发只从这里开始 |
| `archive/2026-05-agentflow-legacy/` | 旧需求、旧规划、旧规格和旧验证摘要归档 |

## 默认阅读顺序

1. [../README.md](../README.md)
2. [v0.4.0/README.md](v0.4.0/README.md)
3. [v0.5.0/README.md](v0.5.0/README.md)
4. [requirements/README.md](requirements/README.md)
5. [requirements/next-requirements.md](requirements/next-requirements.md)
6. [product/README.md](product/README.md)
7. [product/design-system.md](product/design-system.md)
8. [foundation/README.md](foundation/README.md)
9. [architecture/README.md](architecture/README.md)
10. [verification/058h-release-gate-e2e-v1.md](verification/058h-release-gate-e2e-v1.md)
11. [verification/064-v0-3-1-release-gate-certification-v1.md](verification/064-v0-3-1-release-gate-certification-v1.md)
12. [verification/history.md](verification/history.md)

## 规则

- `archive/` 下文档不作为后续开发依据。
- `product/` 下文档作为产品模型和项目设计基线，不直接等同于实现任务。
- `foundation/` 下文档作为下一代基础能力需求，不混入当前版本迭代队列。
- `architecture/` 下文档作为底层技术蓝图、authority 边界和运行时模型，不直接等同于实现任务。
- `verification/` 下文档定义正式 gate、E2E 证据链和可复跑验证入口。
- `v0.4.0/` 下文档是版本规划和技术设计草案；进入实现前仍必须转成正式 `requirements/` 和 `.agentflow/spec/**`。
- `v0.5.0/` 下文档是版本规划和任务草案；进入实现前仍必须转成正式 `requirements/` 和 `.agentflow/spec/**`。
- 根目录旧 `GOAL.md`、`ROADMAP.md` 和 `verification.md` 已退出入口；历史验证记录迁入 `verification/history.md`。
- 根目录 `design.md` 只保留兼容入口；完整设计基线迁入 `product/design-system.md`。
- 新功能、新页面、新数据模型和新验收标准必须写入 `requirements/`。
- 从 `product/` 派生实现时，必须先转成 `requirements/` 下的开发需求切片。
- 从 `foundation/` 进入当前版本开发时，必须再明确拆成当前版本可执行需求。
- 从 `architecture/` 进入当前版本开发时，必须先确认对应的 `requirements/` 切片、阶段目标和验收方式。
- 未进入 `requirements/` 的旧文档内容不能自动转化为 issue 或实现任务。
