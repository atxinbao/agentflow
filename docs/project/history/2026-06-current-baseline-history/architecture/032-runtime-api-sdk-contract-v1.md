# Runtime API / SDK Contract V1

日期：2026-06-24
执行者：Codex

## Purpose

`Runtime API / SDK Contract` 是 AgentFlow 给行业客户端、connector、provider worker 和 Pack surface 使用的统一入口合同。

它回答：

```text
客户端能调用什么？
客户端能读取什么？
客户端能订阅或重放什么？
哪些 API 绝不能写 authority？
Runtime 如何保证 SDK 不绕过 command / event / projection 边界？
```

## Contract Boundary

AgentFlow 的 SDK 只能暴露三类安全入口：

| Plane | SDK 可见性 | 权限 | 说明 |
| --- | --- | --- | --- |
| Command API | 不直接暴露写 authority | command | SDK 只能提交 Runtime Command 或 Pack Action Proposal，不能直接写事实源 |
| Query API | 可暴露 | readonly | 读取 Projection read model，不写 `.agentflow/**` authority |
| Event API | 可暴露 replay / receipt | readonly | 读取 event stream、cursor、receipt；append / claim 只允许 Runtime 内部使用 |

内部 Runtime 可以使用：

- Event append；
- Event claim；
- Runtime authority write；
- release authority command；
- migration apply receipt；
- acceptance / completion commit。

这些入口必须在 API Plane Manifest 中标记为 `internal` 或 `authority`，不能标记为 `sdk-candidate`。

## API Envelope

所有 SDK-facing request / response 必须具备以下字段或等价结构：

```text
version
requestId / commandId / queryId
correlationId
idempotencyKey
actorRole
sourceSurface
targetObjectRef
input
status
decision / result
errors / rejectedReasons
nextQueryHint
createdAt / recordedAt
```

### Command API

命令入口必须经过：

```text
validate_runtime_command
-> map_command_to_action_proposal
-> arbitrate_action
-> write_runtime_command_fact
-> write_runtime_proposal_fact
-> write_runtime_decision_fact
-> append_accepted_action_event
```

命令响应必须明确：

- `accepted`
- `rejected`
- `humanDecisionRequired`
- `queued`
- `superseded`
- `cancelled`
- `invalidCommand`

### Query API

查询入口只能读取 Projection：

```text
get_requirement_intake_view
get_spec_preview_view
get_spec_loop_view
get_project_home_view
get_task_workbench_view
get_work_loop_run_view
get_work_loop_session_view
get_audit_surface_view
get_delivery_package_view
get_pack_industry_workbench_view
get_runtime_health_view
```

Projection API 不允许：

- 写 Event Store；
- 写 Spec authority；
- 写 Task authority；
- 写 Release authority；
- 触发 provider；
- 触发 connector mutation。

### Event API

Event API 分成两类：

| Entry | Boundary | SDK |
| --- | --- | --- |
| `event.runtime.replay` | readonly | allowed |
| `event.task.replay` | readonly | allowed |
| `event.runtime.append-accepted-action` | internal | forbidden |
| `event.task.claim` | internal | forbidden |

SDK 可以重放或检查 event receipt，但不能 append event。

原因很简单：

```text
Event Store 是事实流。
append 权限等同于修改 Runtime 历史。
```

## Pack Compatibility

Software Dev Pack 和 UI Design Pack 都必须通过同一套 API contract：

```text
Pack Surface
-> pack.command.list / validate / dry-run
-> pack.surface.route
-> pack.capability.status
-> pack.command.submit-proposal
-> Runtime Command
-> Arbitration
-> Event / Projection
```

Pack 不能直接调用：

- Event append；
- Projection write；
- Spec materialization；
- Task completion writeback；
- Release authority write。

Pack command 只负责把行业交互转换成 Runtime-controlled proposal。

## API Plane Manifest Rules

Release gate 必须验证：

- Manifest 覆盖 `runtime_commands`；
- Manifest 覆盖 `projection_queries`；
- Manifest 覆盖 `event_api`；
- Manifest 覆盖 `command_surface_actions`；
- Manifest 覆盖 `connector_actions`；
- Manifest 覆盖 `provider_actions`；
- Manifest 覆盖 `audit_actions`；
- Manifest 覆盖 `release_actions`；
- Manifest 覆盖 `pack_actions`；
- Manifest 覆盖 `pack_command_surface`；
- `sdk-candidate` 只能是 `readonly`；
- `projection_queries` 必须是 `readonly`；
- Event API 必须同时存在 readonly replay path 和 internal append / claim path；
- Pack command 必须包含 list / validate / dry-run / submit-proposal / route / capability status。

## Fixtures / Tests

V1 覆盖：

- Command path：`runtime.command.validate`、`runtime.command.execute`；
- Query path：`projection.task-workbench`、`projection.pack-industry-workbench`；
- Event path：`event.runtime.replay`、`event.task.replay`、internal append / claim；
- Pack path：Software Dev 和 UI Design 共用 Pack command surface；
- SDK guard：所有 `sdk-candidate` 都必须是 readonly。

## Non-goals

本版本不做：

- 远程 API server；
- API auth；
- API gateway；
- 多租户权限；
- SDK package 发布；
- GraphQL / REST 传输层；
- connector 直接 authority write；
- projection 写入口。

## Acceptance

- API contract 明确输入、输出、错误和状态；
- SDK 不能绕过 Runtime authority；
- release gate 验证 command / query / event / Pack command manifest；
- Software Dev Pack 和 UI Design Pack 能通过同一套 Runtime API / SDK contract 接入；
- Projection API 仍然是 read-only surface。
