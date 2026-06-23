# Architecture Docs

创建日期：2026-06-18
执行者：Codex

## Purpose

`docs/architecture/` 负责定义 AgentFlow 的技术底座。

这里不直接写当前迭代需求，也不直接生成实现任务。
它回答的是：

- Project 的底层运行时如何组织？
- Agent、Workflow、Event、Projection 之间是什么关系？
- 哪些模块是长期地基，哪些模块是可替换层？
- Desktop / CLI / 外部 Provider 应该读取或写入什么？

## 文档范围

| 文档 | 作用 |
| --- | --- |
| [001-project-operating-system-v1.md](001-project-operating-system-v1.md) | 定义 AgentFlow 的总蓝图、authority、永久层与可替换层 |
| [002-agent-capability-matrix-v1.md](002-agent-capability-matrix-v1.md) | 定义 Goal / Spec / Work / Audit / Delivery 的角色、职责、技能与 handoff |
| [003-workflow-schema-v1.md](003-workflow-schema-v1.md) | 定义 Project / Work / Audit / Delivery 四类流程的统一 schema |
| [004-event-and-projection-model-v1.md](004-event-and-projection-model-v1.md) | 定义事件、状态、投影和 UI 读模型 |
| [005-public-delivery-standard-v1.md](005-public-delivery-standard-v1.md) | 定义任务级与版本级公开交付模板、边界和 projection 输出 |
| [006-release-runtime-v1.md](006-release-runtime-v1.md) | 定义 completion 之后的项目级 release gate、runtime、facts 和公开发布输出 |
| [007-external-review-surface-v1.md](007-external-review-surface-v1.md) | 定义外部 reviewer 的统一阅读面、evidence index、audit summary 和 handoff package |
| [008-runtime-terminology-convergence-v1.md](008-runtime-terminology-convergence-v1.md) | 定义 Work Agent / Build Agent、workflow ref、event、projection 的统一术语规则 |
| [009-runtime-foundation-closeout-baseline-v1.md](009-runtime-foundation-closeout-baseline-v1.md) | 定义 v0.4.0 Runtime Foundation 的正式 closeout baseline 与主链验证锚点 |
| [010-work-loop-filesystem-contract-v1.md](010-work-loop-filesystem-contract-v1.md) | 定义 v0.6.0 Work Loop / CodeFlow 的文件合同、authority 和阶段路径 |
| [011-projection-surface-console-ia-v1.md](011-projection-surface-console-ia-v1.md) | 定义 v0.7.0 Projection Surface、Project OS Console 信息架构、读写边界和命令回流 |
| [012-schema-version-migration-registry-v1.md](012-schema-version-migration-registry-v1.md) | 定义 schema version registry、legacy detection、migration preview 和 explicit apply 边界 |
| [013-simulation-dry-run-runtime-v1.md](013-simulation-dry-run-runtime-v1.md) | 定义 simulation / dry-run runtime 的只读边界、报告结构和 gate impact |
| [014-local-message-bus-contract-v1.md](014-local-message-bus-contract-v1.md) | 定义本地 Message Bus 的 channel、refresh signal、replay 和非 authority 边界 |
| [015-worker-tool-capability-registry-v1.md](015-worker-tool-capability-registry-v1.md) | 定义 worker / tool capability registry、health、command availability 和 disabled reason |
| [016-provider-smoke-gate-v1.md](016-provider-smoke-gate-v1.md) | 定义 provider smoke gate 的最小 health / launch / session / terminal projection 证明 |
| [017-connector-mcp-boundary-v1.md](017-connector-mcp-boundary-v1.md) | 定义 connector / MCP read-write capability、authority 边界和失败展示面 |
| [018-api-plane-manifest-v1.md](018-api-plane-manifest-v1.md) | 定义 Runtime / Projection / Command API Plane manifest 和 release gate 检查 |
| [019-pack-filesystem-contract-v1.md](019-pack-filesystem-contract-v1.md) | 定义 Pack System 文件系统位置、职责、authority 边界和 Runtime 入口 |
| [020-domain-pack-contract-v1.md](020-domain-pack-contract-v1.md) | 定义 Domain Pack 的对象、关系、状态、动作、验收、证据和审计提示边界 |
| [021-surface-pack-contract-v1.md](021-surface-pack-contract-v1.md) | 定义 Surface Pack 的页面、工作台、视图模型、命令入口、状态和 sidecar 边界 |
| [022-connector-pack-contract-v1.md](022-connector-pack-contract-v1.md) | 定义 Connector Pack 的外部 provider、capability、smoke、evidence 和 command boundary |
| [023-pack-validation-migration-preview-v1.md](023-pack-validation-migration-preview-v1.md) | 定义 Pack validation artifact、version compatibility、API Plane mapping 和 migration preview 边界 |
| [024-pack-simulation-dry-run-v1.md](024-pack-simulation-dry-run-v1.md) | 定义 Pack command dry-run、缺口识别、影响预览和只读边界 |
| [025-pack-aware-projection-read-models-v1.md](025-pack-aware-projection-read-models-v1.md) | 定义 Pack-aware projection read model、industry workbench view 和 readiness 读模型边界 |
| [026-pack-command-surface-runtime-api-v1.md](026-pack-command-surface-runtime-api-v1.md) | 定义 Pack command 到 Runtime API、Action Contract、Arbitration 的映射与只读 / 命令边界 |
| [027-software-dev-pack-baseline-v1.md](027-software-dev-pack-baseline-v1.md) | 定义 Software Dev Pack 作为第一个正式行业壳的主链、sidecar、connector 和 readiness 边界 |
| [028-ui-design-pack-baseline-v1.md](028-ui-design-pack-baseline-v1.md) | 定义 UI Design Pack 作为第二个正式行业壳的设计主链、handoff evidence policy 和 connector 边界 |
| [029-pack-release-gate-readiness-v1.md](029-pack-release-gate-readiness-v1.md) | 定义 Pack System release gate readiness artifact、状态词、失败规则和 Audit sidecar 非阻断边界 |
| [030-local-runtime-boundary-v1.md](030-local-runtime-boundary-v1.md) | 定义本地 Runtime 的 authority、API plane、worker 生命周期、Pack / Connector / Provider 和 resume 边界 |
| [031-cloud-runtime-boundary-v1.md](031-cloud-runtime-boundary-v1.md) | 定义云端 Runtime 的 Runtime Core、API / SDK、行业客户端、Pack / Connector 和治理边界 |
| [032-runtime-api-sdk-contract-v1.md](032-runtime-api-sdk-contract-v1.md) | 定义 Runtime API / SDK 的 command、query、event、Pack command 和 SDK 只读候选边界 |
| [current-module-boundaries.md](current-module-boundaries.md) | 当前 crates 和目录边界的事实快照 |
| [mcp-provider-adapter.md](mcp-provider-adapter.md) | 外部 provider / MCP 适配层边界 |

## 默认阅读顺序

1. [001-project-operating-system-v1.md](001-project-operating-system-v1.md)
2. [002-agent-capability-matrix-v1.md](002-agent-capability-matrix-v1.md)
3. [003-workflow-schema-v1.md](003-workflow-schema-v1.md)
4. [004-event-and-projection-model-v1.md](004-event-and-projection-model-v1.md)
5. [005-public-delivery-standard-v1.md](005-public-delivery-standard-v1.md)
6. [006-release-runtime-v1.md](006-release-runtime-v1.md)
7. [007-external-review-surface-v1.md](007-external-review-surface-v1.md)
8. [008-runtime-terminology-convergence-v1.md](008-runtime-terminology-convergence-v1.md)
9. [009-runtime-foundation-closeout-baseline-v1.md](009-runtime-foundation-closeout-baseline-v1.md)
10. [010-work-loop-filesystem-contract-v1.md](010-work-loop-filesystem-contract-v1.md)
11. [011-projection-surface-console-ia-v1.md](011-projection-surface-console-ia-v1.md)
12. [012-schema-version-migration-registry-v1.md](012-schema-version-migration-registry-v1.md)
13. [013-simulation-dry-run-runtime-v1.md](013-simulation-dry-run-runtime-v1.md)
14. [014-local-message-bus-contract-v1.md](014-local-message-bus-contract-v1.md)
15. [015-worker-tool-capability-registry-v1.md](015-worker-tool-capability-registry-v1.md)
16. [016-provider-smoke-gate-v1.md](016-provider-smoke-gate-v1.md)
17. [017-connector-mcp-boundary-v1.md](017-connector-mcp-boundary-v1.md)
18. [018-api-plane-manifest-v1.md](018-api-plane-manifest-v1.md)
19. [019-pack-filesystem-contract-v1.md](019-pack-filesystem-contract-v1.md)
20. [020-domain-pack-contract-v1.md](020-domain-pack-contract-v1.md)
21. [021-surface-pack-contract-v1.md](021-surface-pack-contract-v1.md)
22. [022-connector-pack-contract-v1.md](022-connector-pack-contract-v1.md)
23. [023-pack-validation-migration-preview-v1.md](023-pack-validation-migration-preview-v1.md)
24. [024-pack-simulation-dry-run-v1.md](024-pack-simulation-dry-run-v1.md)
25. [025-pack-aware-projection-read-models-v1.md](025-pack-aware-projection-read-models-v1.md)
26. [026-pack-command-surface-runtime-api-v1.md](026-pack-command-surface-runtime-api-v1.md)
27. [027-software-dev-pack-baseline-v1.md](027-software-dev-pack-baseline-v1.md)
28. [028-ui-design-pack-baseline-v1.md](028-ui-design-pack-baseline-v1.md)
29. [029-pack-release-gate-readiness-v1.md](029-pack-release-gate-readiness-v1.md)
30. [030-local-runtime-boundary-v1.md](030-local-runtime-boundary-v1.md)
31. [031-cloud-runtime-boundary-v1.md](031-cloud-runtime-boundary-v1.md)
32. [032-runtime-api-sdk-contract-v1.md](032-runtime-api-sdk-contract-v1.md)
33. [current-module-boundaries.md](current-module-boundaries.md)
34. [mcp-provider-adapter.md](mcp-provider-adapter.md)

## 规则

- `docs/architecture/` 只定义技术底座，不直接等同于当前迭代需求。
- `docs/architecture/` 不能绕过 `docs/requirements/` 直接授权开发。
- `docs/product/` 负责说明产品方向。
- `docs/foundation/` 负责说明领域模型和基础能力切片。
- `docs/requirements/` 负责当前版本的可执行需求。
- 当 `architecture`、`foundation` 与 `requirements` 有冲突时，必须先回到产品与架构边界重新确认，再继续开发。
