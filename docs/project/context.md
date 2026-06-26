# Project Context

更新日期：2026-06-26
执行者：Codex

## Current Positioning

AgentFlow 是 Spec-Driven Software Dev Workflow。

它的产品公式是：

```text
AgentFlow AI OS Project
= Spec-Driven Core Runtime
+ Industry AgentFlow Product
```

当前商业产品只聚焦 Software Dev。

## Directory Planes

AgentFlow 至少有两个平面：

```text
docs/       = Human / Project / Third-party Knowledge Plane
.agentflow/ = Agent Runtime Control Plane
```

`docs/` 给人和第三方看，解释项目目标、需求、架构和交付。

`.agentflow/` 给 Agent 和 Runtime 用，保存角色、技能、执行合同、事件、任务状态、证据和投影。

## Current Docs Standard

```text
docs/project       定义项目
docs/requirements  定义确认后的 Spec
docs/architecture  沉淀长期架构
docs/delivery      记录交付结果
```

## Current Industry

当前行业合同：

- [industry/software-dev/README.md](industry/software-dev/README.md)

未来新增行业时，必须先在 `docs/project/industry/<industry>/` 定义人类可读合同，再考虑 `.agentflow/packs/<industry>/` 的机器可执行合同。
