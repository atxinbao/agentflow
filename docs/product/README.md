# AgentFlow Project Operating Model V1

创建日期：2026-06-17
执行者：Codex

## Purpose

本目录定义 AgentFlow 的项目设计基线。

AgentFlow 是项目目标驱动的 Agent 交付系统。用户入口不是单个任务，也不是工作流画布，而是一个 Project。Project 可以很大，例如一个完整电商 APP；也可以很小，例如一个前端 BUG 修复。无论大小，AgentFlow 都从项目目标、计划、执行、审计和交付的角度组织工作。

当前产品目标已收敛为：

```text
AgentFlow = Spec-Driven Software Dev Workflow
```

底层 Project OS 能力继续作为技术底座，但当前商业产品主线只聚焦 Software Dev。

## Core Principle

```text
Project 是顶层目标容器。
Goal 定义项目方向。
Plan 定义项目路径。
Issue 定义最小执行合同。
Evidence 证明执行结果。
Delivery 面向人类交付结果。
```

AgentFlow 的主线：

```text
Add Project
-> Goal Draft
-> Plan Draft
-> Confirmed Project
-> Work Loop
-> Audit
-> Delivery
-> Goal Recheck
```

当前 Software Dev 产品主线：

```text
Intent
-> Spec Bundle
-> Route
-> Domain-specific Plan / Tasks / Artifacts
-> Agent Execution
-> Evidence
-> Acceptance
-> Delivery
-> Feedback
-> Spec Evolution
```

## Agent Roles

Project 由五个核心 Agent 角色协作完成：

| Role | 中文名 | 核心职责 |
| --- | --- | --- |
| Goal Agent | 项目大脑 | 从项目全局维护目标、方向和计划迭代 |
| Spec Agent | 规划拆解 | 把 Goal / Plan 转成可确认的阶段和 Issue Contract |
| Work Agent | 执行代理 | 执行确认后的 Issue Contract |
| Audit Agent | 审计验收 | 检查结果、证据、边界和风险 |
| Delivery Agent | 交付整理 | 把结果整理成人能接收的交付包 |

Specialist Agent 不是固定顶层角色，而是按需加载的专家能力，例如设计、前端、后端、测试、部署、文档等。

## Document Map

| 文档 | 作用 |
| --- | --- |
| [001-project-agent-role-model-v1.md](001-project-agent-role-model-v1.md) | 定义 Project 级 Agent 角色和边界 |
| [002-project-lifecycle-v1.md](002-project-lifecycle-v1.md) | 定义项目从添加到交付再回看的生命周期 |
| [003-goal-plan-document-model-v1.md](003-goal-plan-document-model-v1.md) | 定义 GOAL.md / PLAN.md / DECISIONS.md |
| [004-requirement-to-goal-flow-v1.md](004-requirement-to-goal-flow-v1.md) | 定义用户输入如何转成项目目标 |
| [005-goal-to-project-loop-flow-v1.md](005-goal-to-project-loop-flow-v1.md) | 定义目标和计划确认后如何进入项目循环 |
| [006-spec-driven-software-dev-product-goal-v1.md](006-spec-driven-software-dev-product-goal-v1.md) | 固定当前商业产品目标：Spec-Driven Software Dev Workflow |
| [design-system.md](design-system.md) | 定义当前桌面客户端设计基线和前端 Foundation 规则 |

## Relationship To Requirements

`docs/product/` 是产品操作模型和设计基线。

`docs/requirements/` 是可执行开发需求入口。

新的实现需求必须进入 `docs/requirements/`；新的项目设计原则应先进入 `docs/product/`，确认后再拆成 requirements。

## Boundary

AgentFlow 不做：

- 不从任务开始定义产品。
- 不把 Project 降级成任务列表。
- 不把 Workflow 画布作为主入口。
- 不把 Agent Runner 作为产品核心。
- 不把 Spec 文档生成器作为产品核心。
- 不把多行业平台作为当前商业产品目标。
- 不把 Goal / Plan 当成一次性文档。
- 不让 Work Agent 修改项目目标。
- 不让 Audit Agent 执行任务。
- 不让 Delivery Agent 判断项目方向。
- 不跳过用户确认直接从原始需求生成可执行任务。

AgentFlow 要做：

- 用 Spec 驱动 Software Dev 工作流。
- 从 Project 管理项目目标和交付过程。
- 用 Goal Agent 持续校准方向。
- 用 Spec Agent 把目标转为计划和任务合同。
- 用 Work Agent 执行确认后的任务。
- 用 Audit Agent 检查结果是否可信。
- 用 Delivery Agent 整理交付物。
