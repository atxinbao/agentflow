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

当前商业产品只聚焦 Software Dev，但 Software Dev Pack 是 AgentFlow App 的内置能力，不是用户项目文档。

用户新建项目时，`docs/project/**` 不生成 `industry/` 目录，也不复制 Pack 本体。

项目运行态只记录当前启用的 Pack 引用，例如：

```text
.agentflow/project/active-pack.json
```

Pack registry、Pack source、Pack upgrade 和 role / skill binding 规则见：

- [../architecture/builtin-pack-registry.md](../architecture/builtin-pack-registry.md)
