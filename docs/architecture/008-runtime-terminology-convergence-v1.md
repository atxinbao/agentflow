# 008 - Runtime Terminology Convergence V1

创建日期：2026-06-20
执行者：Codex

## Purpose

本文定义 AgentFlow 运行时术语收口规则。

目标是让文档、任务合同、Projection、Desktop 和 Provider 适配层都围绕同一套主术语工作，不再出现一边写 `work-agent`，另一边还把 `build-agent` 当主名字的双轨状态。

## 收口原则

1. Runtime 主术语优先。
2. Provider 兼容别名可以保留，但不能反向成为主命名。
3. Projection 是读模型，不是事实源。
4. Event Store 是事实源，Projection 和页面都只读它的派生结果。
5. 兼容不等于继续扩散旧命名。

## 核心映射表

| 旧术语 / 旧写法 | 新主术语 | 处理规则 |
| --- | --- | --- |
| `build-agent` | `work-agent` | `build-agent` 只保留为 provider-facing 兼容别名 |
| `BuildAgent` | `WorkAgent` | CamelCase 旧写法只用于兼容解析 |
| `build-agent.issue-loop@v1` | `work-agent.issue-loop@v1` | 新写入统一用 `work-agent.issue-loop@v1`，旧 ref 继续可读 |
| Build Agent Issue Loop | Work Agent Issue Loop | 文档、Projection、页面标题统一改成 Work Agent |
| `Workflow State` | `Object State + Projection State` | 写侧状态和读侧状态明确拆开 |
| `Projection` 作为事实源 | `Projection Read Model` | Projection 只做查询，不承担 authority |
| `AuditFinding` | `Finding` | 对象命名统一收口到 Finding |
| `Delivery` 作为写对象 | `DeliveryPackageView` / public delivery record | MVP 不引入独立 Delivery 写对象 |

## 角色规则

### Runtime 主角色

- `goal-agent`
- `spec-agent`
- `work-agent`
- `audit-agent`
- `delivery-agent`
- `review-agent`
- `coordinator-agent`
- `human-owner`

### 兼容别名

- `build-agent -> work-agent`

规则：

- Workflow authority、Projection authority、Object State authority 都使用 `work-agent`。
- Provider 会话、CLI 子命令、外部启动口允许继续使用 `build-agent` 作为兼容入口。
- 新生成的 Spec Issue 默认 `requiredAgentRole = work-agent`。
- 旧 `requiredAgentRole = build-agent` 的任务必须继续可读、可执行、可展示。

## Workflow Ref 规则

### Canonical Ref

```text
work-agent.issue-loop@v1
```

### Compatibility Ref

```text
build-agent.issue-loop@v1
```

规则：

- 新写入的 issue / projection / state / preview fixture 统一使用 canonical ref。
- workflow parser 必须能把 `build-agent.issue-loop@v1` 解析到 canonical workflow。
- 不要求批量改写历史文件。

## Event 与 Session 命名

运行时事件主命名统一保持 generic event 语言：

- `agent.launch.requested`
- `agent.session.created`
- `agent.session.running`
- `agent.session.in_review`
- `agent.session.completed`

规则：

- 事件类型不再继续扩散 `build-agent.*` 风格名字。
- 若旧文档里出现 `build-agent.launch.*`，只作为历史说明，不再作为当前推荐术语。
- 事件里的 authority role 应写 `work-agent`；若 actor 来自 provider 兼容层，可在 payload 或 alias 说明中记录 `build-agent`。

## 文档更新要求

本轮之后，以下文档必须遵守这套术语：

- `docs/architecture/current-module-boundaries.md`
- `docs/architecture/002-agent-capability-matrix-v1.md`
- `docs/architecture/004-event-and-projection-model-v1.md`
- `docs/architecture/mcp-provider-adapter.md`

## 非目标

- 不做一次性全量历史文档重写。
- 不删除旧事实文件。
- 不要求当前 CLI 子命令从 `build-agent` 改名。
- 不在这一轮引入新的 Delivery 写对象。

## Acceptance

完成时应满足：

- 新写出的 workflow ref 统一为 `work-agent.issue-loop@v1`。
- 新写出的 Spec Issue 默认角色是 `work-agent`。
- Desktop / Browser Preview 的默认执行角色统一为 `work-agent`，但继续兼容读取 `build-agent`。
- 文档中明确写出 `build-agent -> work-agent` 的兼容关系。
- Event / Projection / Runtime 文档不再把旧术语当主语言。
