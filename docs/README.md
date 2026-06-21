# Docs Index

更新日期：2026-06-21
执行者：Codex

## 当前文档入口

| 路径 | 作用 |
| --- | --- |
| `product/` | AgentFlow Project Operating Model 产品设计基线 |
| `foundation/` | 下一代 Project Operating Model 的基础能力需求 |
| `architecture/` | AgentFlow 底层技术蓝图、模块边界与运行时模型 |
| `verification/` | Gate、E2E 证据链和正式验证入口说明 |
| `v0.4.0/` | v0.4.0 Definition-driven Runtime Foundation 已发布版本基线与 closeout 文档 |
| `v0.5.0/` | v0.5.0 Spec Loop Productization 功能基线与风险延期记录 |
| `v0.5.1/` | v0.5.1 Release Hygiene 与 Authority Closure 修复链 |
| `v0.6.0/` | v0.6.0 Work Loop Handoff & Controlled Execution 开发前置规划，受 v0.5.1 修复链约束 |
| `v0.6.1/` | v0.6.0 审计后续、release closeout 与 Acceptance Gate 修复规划 |
| `v0.7.0/` | v0.7.0 Projection Surface 与 Project OS Console 开发前置规划 |
| `requirements/` | 新需求文档入口，后续开发只从这里开始 |
| `archive/2026-05-agentflow-legacy/` | 旧需求、旧规划、旧规格和旧验证摘要归档 |

## 默认阅读顺序

1. [../README.md](../README.md)
2. [v0.4.0/README.md](v0.4.0/README.md)
3. [v0.5.0/README.md](v0.5.0/README.md)
4. [v0.5.1/README.md](v0.5.1/README.md)
5. [v0.6.0/README.md](v0.6.0/README.md)
6. [v0.6.1/README.md](v0.6.1/README.md)
7. [v0.7.0/README.md](v0.7.0/README.md)
8. [requirements/README.md](requirements/README.md)
9. [requirements/next-requirements.md](requirements/next-requirements.md)
10. [product/README.md](product/README.md)
11. [product/design-system.md](product/design-system.md)
12. [foundation/README.md](foundation/README.md)
13. [architecture/README.md](architecture/README.md)
14. [verification/058h-release-gate-e2e-v1.md](verification/058h-release-gate-e2e-v1.md)
15. [verification/064-v0-3-1-release-gate-certification-v1.md](verification/064-v0-3-1-release-gate-certification-v1.md)
16. [verification/history.md](verification/history.md)

## 规则

- `archive/` 下文档不作为后续开发依据。
- `product/` 下文档作为产品模型和项目设计基线，不直接等同于实现任务。
- `foundation/` 下文档作为下一代基础能力需求，不混入当前版本迭代队列。
- `architecture/` 下文档作为底层技术蓝图、authority 边界和运行时模型，不直接等同于实现任务。
- `verification/` 下文档定义正式 gate、E2E 证据链和可复跑验证入口。
- `v0.4.0/` 下文档记录已发布版本的技术基线与 closeout 事实，不再视为未执行草案。
- `v0.5.0/` 下文档记录已发布版本的 Spec Loop 功能基线，但当前只能视为 functional baseline，不应直接视为 clean stable release。
- `v0.5.1/` 下文档记录修复版本入口；在 `v0.5.1` 完成前，不进入 `v0.6.0` 实现。
- `v0.6.0/` 下文档记录下一版本 Work Loop handoff 与受控执行规划；进入开发前仍必须先完成 `v0.5.1` release hygiene 与 authority closure 修复链，再转成正式 requirement 和 spec issue。
- `v0.6.1/` 下文档记录 `v0.6.0` 发布审计后的修复规划，重点是 release closeout、Acceptance Gate、Completion Commit 和 Audit separation。
- `v0.7.0/` 下文档记录 Projection Surface 与 Project OS Console 规划；进入实现前必须先完成 `v0.6.1` 验收闭环和发布收口。
- 后续新开发需求仍然要进入 `requirements/`；新版本运行时事实源仍然以 `.agentflow/spec/**` 为准。
- 根目录旧 `GOAL.md`、`ROADMAP.md` 和 `verification.md` 已退出入口；历史验证记录迁入 `verification/history.md`。
- 根目录 `design.md` 只保留兼容入口；完整设计基线迁入 `product/design-system.md`。
- 新功能、新页面、新数据模型和新验收标准必须写入 `requirements/`。
- 从 `product/` 派生实现时，必须先转成 `requirements/` 下的开发需求切片。
- 从 `foundation/` 进入当前版本开发时，必须再明确拆成当前版本可执行需求。
- 从 `architecture/` 进入当前版本开发时，必须先确认对应的 `requirements/` 切片、阶段目标和验收方式。
- 未进入 `requirements/` 的旧文档内容不能自动转化为 issue 或实现任务。
