# Docs Index

更新日期：2026-06-26
执行者：Codex

## 当前文档入口

| 路径 | 作用 |
| --- | --- |
| `product/` | AgentFlow Project Operating Model 产品设计基线；当前产品目标是 Spec-Driven Software Dev Workflow |
| `foundation/` | 下一代 Project Operating Model 的基础能力需求；包含 AI OS Project Core 的通用 Kernel 能力定义 |
| `architecture/` | AgentFlow 底层技术蓝图、模块边界与运行时模型 |
| `verification/` | Gate、E2E 证据链和正式验证入口说明 |
| `v0.4.0/` | v0.4.0 Definition-driven Runtime Foundation 已发布版本基线与 closeout 文档 |
| `v0.5.0/` | v0.5.0 Spec Loop Productization 功能基线与风险延期记录 |
| `v0.5.1/` | v0.5.1 Release Hygiene 与 Authority Closure 历史修复基线，已折入 v0.6.0 发布路径 |
| `v0.6.0/` | v0.6.0 Work Loop Handoff & Controlled Execution 已发布功能基线与 release closeout |
| `v0.6.1/` | v0.6.0 审计后续、release closeout、Acceptance Gate 修复链与发布认证 |
| `v0.7.0/` | v0.7.0 Projection Surface 与 Project OS Console 开发前置规划 |
| `v0.7.1/` | v0.7.x Console Release Certification 证据收口与真实 readiness gate 修复链 |
| `v0.7.2/` | v0.7.2 Runtime Foundation hardening 任务基线 |
| `v0.8.0/` | v0.8.0 Pack System、Software Dev Pack 和 UI Design Pack 开发任务基线 |
| `v0.8.1/` | v0.8.1 Pack System clean remediation release certification |
| `v0.9.0/` | v0.9.0 Deployment Shape and Runtime Governance 已发布 closeout 基线 |
| `v0.9.1/` | v0.9.1 Runtime Governance Stabilization 修复任务基线 |
| `v1.0.0/` | v1.0.0 Project OS Stable Core 开发前置规划 |
| `v1.0.1/` | v1.0.1 Release Hardening and Operational Certification 补丁 closeout 基线 |
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
8. [v0.7.1/README.md](v0.7.1/README.md)
9. [v0.7.2/README.md](v0.7.2/README.md)
10. [v0.8.0/README.md](v0.8.0/README.md)
11. [v0.8.1/README.md](v0.8.1/README.md)
12. [v0.9.0/README.md](v0.9.0/README.md)
13. [v0.9.1/README.md](v0.9.1/README.md)
14. [v1.0.0/README.md](v1.0.0/README.md)
15. [v1.0.1/README.md](v1.0.1/README.md)
16. [requirements/README.md](requirements/README.md)
17. [requirements/next-requirements.md](requirements/next-requirements.md)
18. [product/README.md](product/README.md)
19. [product/design-system.md](product/design-system.md)
20. [foundation/README.md](foundation/README.md)
21. [architecture/README.md](architecture/README.md)
22. [verification/058h-release-gate-e2e-v1.md](verification/058h-release-gate-e2e-v1.md)
23. [verification/064-v0-3-1-release-gate-certification-v1.md](verification/064-v0-3-1-release-gate-certification-v1.md)
24. [verification/history.md](verification/history.md)

## 规则

- `archive/` 下文档不作为后续开发依据。
- `product/` 下文档作为产品模型和项目设计基线，不直接等同于实现任务。
- `foundation/` 下文档作为下一代基础能力需求，不混入当前版本迭代队列。
- `foundation/021-ai-os-project-core-capabilities-v1.md` 定义 AgentFlow Core 的 6 个 Kernel、12 个通用能力、`docs/` 与 `.agentflow/` 的职责边界，以及行业 Pack 接入方式；它不自动授权实现。
- `architecture/` 下文档作为底层技术蓝图、authority 边界和运行时模型，不直接等同于实现任务。
- `verification/` 下文档定义正式 gate、E2E 证据链和可复跑验证入口。
- `v0.4.0/` 下文档记录已发布版本的技术基线与 closeout 事实，不再视为未执行草案。
- `v0.5.0/` 下文档记录已发布版本的 Spec Loop 功能基线，但当前只能视为 functional baseline，不应直接视为 clean stable release。
- `v0.5.1/` 下文档记录历史 release hygiene 与 authority closure 修复基线；它不再阻塞已经发布的 `v0.6.0`，剩余问题进入 `v0.6.1`。
- `v0.6.0/` 下文档记录已发布的 Work Loop handoff 与受控执行功能基线；它不是 clean stable closeout，遗留 release hygiene 与 Acceptance Gate 问题进入 `v0.6.1`。
- `v0.6.1/` 下文档记录 `v0.6.0` 发布审计后的修复链和发布认证，重点是 release closeout、Acceptance Gate、Completion Commit 和 Audit separation。
- `v0.7.0/` 下文档记录 Projection Surface 与 Project OS Console 规划；进入实现前必须先完成 `v0.6.1` 验收闭环和发布收口。
- `v0.7.1/` 下文档记录 `v0.7.0` 发布后的 Console release certification 证据收口和 readiness gate 修复链。
- `v0.7.2/` 下文档记录 Runtime Foundation hardening 任务基线；优先收口 Audit sidecar、migration preview、simulation、message bus、provider smoke 和 release-gate foundation coverage。
- `v0.8.0/` 下文档记录 Pack System 和第一批行业壳任务基线；Software Dev Pack 的 Audit 必须保持 sidecar，不进入主业务链路。
- `v0.8.1/` 下文档记录 `v0.8.0` 发布后的 Pack System clean remediation release certification；重点是 file-backed Pack source、registry-driven resolver、Pack-specific Projection、capability-aware availability、negative fixtures 和 release certification。
- `v0.9.0/` 下文档记录已发布的 Deployment Shape and Runtime Governance closeout 基线；重点是 local/cloud Runtime boundary、Runtime API / SDK、event replay、migration apply、simulation/evaluation、governance、deployment evidence 和 release certification coverage。
- `v0.9.1/` 下文档记录 `v0.9.0` 发布审计后的稳定化修复任务；重点是 Governance admission 接入、Deployment Evidence 语义证明、Pack migration 语义拆分、project Pack registry fixture、negative semantic fixtures 和 release source Agent entry 自洽。
- `v1.0.0/` 下文档记录 Project OS Stable Core 稳定核心认证基线；最终 release gate 必须输出 `v1StableCore = ready` 和明确的 v1 support boundary。
- `v1.0.1/` 下文档记录 `v1.0.0` 发布后的 release hardening 和 operational certification 补丁 closeout；重点是 source Agent entry v1 对齐、tag/release event certification、provenance manifest、clean-room test reproducibility、Audit sidecar policy、provider smoke optional proof、Message Bus no-go ADR、Software Dev Pack usage baseline、trusted governance telemetry source 和 v101 release certification。
- 后续新开发需求仍然要进入 `requirements/`；新版本运行时事实源仍然以 `.agentflow/spec/**` 为准。
- 根目录旧 `GOAL.md`、`ROADMAP.md` 和 `verification.md` 已退出入口；历史验证记录迁入 `verification/history.md`。
- 根目录 `design.md` 只保留兼容入口；完整设计基线迁入 `product/design-system.md`。
- 新功能、新页面、新数据模型和新验收标准必须写入 `requirements/`。
- 从 `product/` 派生实现时，必须先转成 `requirements/` 下的开发需求切片。
- 从 `foundation/` 进入当前版本开发时，必须再明确拆成当前版本可执行需求。
- 从 `architecture/` 进入当前版本开发时，必须先确认对应的 `requirements/` 切片、阶段目标和验收方式。
- 未进入 `requirements/` 的旧文档内容不能自动转化为 issue 或实现任务。
