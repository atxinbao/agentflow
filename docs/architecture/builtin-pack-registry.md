# Built-in Pack Registry

更新日期：2026-06-28
执行者：Codex

## Purpose

本文定义 AgentFlow 的内置 Pack Registry 边界。

结论：

```text
Pack 是 AgentFlow App 的隐形能力。
用户项目不生成 Pack 本体。
用户项目只记录当前启用的 Pack 引用和运行事实。
```

## Positioning

AgentFlow 的上层公式仍然是：

```text
AgentFlow AI OS Project
= Core OS Runtime
+ Industry AgentFlow App
```

其中：

```text
Industry AgentFlow App
= Domain Pack
+ Surface Pack
+ Connector Pack
```

但这些 Pack 不应该放在用户项目的 `docs/project/**` 中。

`docs/project/**` 只解释项目目标、上下文、术语和历史。Pack 本体由 AgentFlow App 或 AgentFlow 源码包管理。

## App Internal Source

AgentFlow 源码中可以用下面任一形态保存内置 Pack：

```text
packs/builtin/software-dev/
```

或：

```text
crates/pack-registry/builtin/software-dev/
```

打包后的 App 可以把内置 Pack 放入产品资源目录：

```text
AgentFlow.app/Contents/Resources/packs/builtin/software-dev/
```

具体路径可以随实现演进，但边界不变：

- Pack registry 属于 App internal source；
- Pack resolver 从 App internal source 读取 Pack；
- Pack migration、compatibility、capability 和 projection mapping 由 App 管；
- 用户项目不拥有 Pack 本体。

旧 v1 合同中出现的项目级 Pack 路径只能理解为运行态或历史实现语义。新建项目模板不暴露 Pack 本体目录。

## Project Reference

用户项目只需要记录当前启用的 Pack 引用。

建议运行态引用文件：

```text
.agentflow/project/active-pack.json
```

示例：

```json
{
  "packId": "software-dev",
  "packVersion": "1.0.0",
  "source": "builtin"
}
```

这份文件只说明项目正在使用哪个 Pack。它不是 Pack manifest，也不是 Pack schema。

## Docs Boundary

新建项目的 `docs/` 标准目录保持四个入口：

```text
docs/project       定义项目
docs/requirements  定义确认后的 Spec
docs/architecture  沉淀长期架构
docs/delivery      记录交付结果
```

不要在新建项目里生成项目内行业定义目录、项目内 Pack 文档目录或用户可见 Pack 根目录。

如果需要向人解释当前项目使用的行业能力，只在 `docs/project/context.md` 中写一句当前启用的 Pack，并链接到本文件。

## Runtime Boundary

`.agentflow/**` 负责记录运行事实：

```text
.agentflow/project/active-pack.json
.agentflow/spec/**
.agentflow/tasks/**
.agentflow/events/**
.agentflow/tasks/<issue-id>/evidence/**
```

这些文件说明项目发生了什么、当前任务是什么、证据在哪里、状态如何推进。

它们不应该反向变成 Pack 本体。

## Role And Skill Binding

Agent 角色和技能由当前启用的 Pack、Core OS Runtime 规则和 AgentFlow role policy 共同决定。

稳定边界：

- Pack 声明领域对象、页面入口、connector 能力和 evidence policy；
- Runtime 根据 Pack 和任务合同生成 role handoff；
- 本地 materialized role / skill lock 可以写入 `.agentflow/define/agent/**`；
- `.agentflow/define/agent/**` 是运行态手册快照，不是 Pack source。

## Rules

- Pack 本体不写入用户项目 `docs/`。
- Pack 本体不作为用户可见目录。
- Pack 本体不直接写 `.agentflow/spec/**`、`.agentflow/events/**` 或 `.agentflow/tasks/**`。
- Pack command 必须通过 Runtime API、Action Contract、Governance 和 Acceptance Gate。
- Pack upgrade 必须可验证、可回滚或可取消。
- 缺失 Pack 定义时，Projection 必须显示 `invalid` 或 `deferred`，不能静默 fallback。

## Software Dev Reference App Baseline

Software Dev 是第一个官方 Reference App。

Software Dev Pack 应作为 App 内置 Pack 提供：

- Domain Pack：Requirement / Spec / Issue / Run / PR / Release；
- Surface Pack：Project Home / Task Workbench / Delivery / Audit Sidecar；
- Connector Pack：Git / GitHub / Codex / Claude Code / Browser Preview。

Audit 仍然是独立 sidecar，不进入 Software Dev 主业务链。

## Non-goals

- 不在用户项目里暴露 Pack marketplace。
- 不把 `docs/project/**` 变成 Pack authoring surface。
- 不把 `.agentflow/project/active-pack.json` 变成 Pack manifest。
- 不把 Pack registry 变成 GitHub issues 或外部工具状态。
- 不让 Pack 绕过 Core OS Runtime。
