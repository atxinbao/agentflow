# Project Context

更新日期：2026-06-30
执行者：Codex

## Current Positioning

AgentFlow 是 Spec-Driven AI OS Project。

它的产品公式是：

```text
AgentFlow AI OS Project
= Core OS Runtime
+ Industry Product Surface
```

当前优先目标是稳住 Core OS Runtime。Software Dev 是第一个官方 Reference App，用来认证 Core 的真实闭环。

Industry Product Surface 不一定是完整 App。它可以是：

- Paid Report Flow：输入、支付、一次受控 Run、交付报告；
- Managed Project Flow：目标、任务、证据、验收、交付；
- Assistant / Ops Flow：持续托管、监控、提醒、执行和反馈。

AgentFlow 的商业产品目标是卖可交付结果，不是先卖 Agent 本身。

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

`docs/project/roadmap.md` 位于 `goal.md` 和 `docs/requirements/**` 之间。
它定义版本阶段和 loop 推进顺序，但不直接授权执行。

## Current Reference Product Surface

当前第一个 Reference Product Surface 是 Software Dev Managed Project Flow，但 Software Dev Pack 是 AgentFlow 内置能力，不是用户项目文档。

用户新建项目时，`docs/project/**` 不生成 `industry/` 目录，也不复制 Pack 本体。

项目运行态只记录当前启用的 Pack 引用，例如：

```text
.agentflow/project/active-pack.json
```

Pack registry、Pack source、Pack upgrade 和 role / skill binding 规则见：

- [../architecture/builtin-pack-registry.md](../architecture/builtin-pack-registry.md)
