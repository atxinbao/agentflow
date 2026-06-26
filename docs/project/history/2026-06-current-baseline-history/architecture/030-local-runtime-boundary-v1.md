# Local Runtime Boundary V1

创建日期：2026-06-23
执行者：Codex

## Purpose

本文档定义 AgentFlow 本地 Runtime 的部署边界。

它回答：

```text
本地 AgentFlow Runtime 运行在哪里？
哪些目录是 authority？
哪些 API 可以写入？
worker 生命周期如何追踪？
Pack、Connector、Provider、Projection 分别能做什么？
```

`v0.9.0 / V090-001` 只定义本地 Runtime，不定义 Cloud Runtime。

## Local Runtime Shape

本地 Runtime 是完整开发体验的运行边界，但它不能绕过 Runtime authority。

```text
Project Workspace
  -> .agentflow/spec/**
  -> .agentflow/runtime/**
  -> .agentflow/events/**
  -> .agentflow/tasks/**
  -> .agentflow/projections/**
  -> .agentflow/packs/**
```

本地 Runtime 可以读取项目内 `.agentflow/**`，但只能通过 Runtime API 写入
authority。

## Authority Boundary

| Surface | Path | Authority Class | Writer |
| --- | --- | --- | --- |
| Requirement Record | `docs/requirements/**` | public_record | Spec Loop materializer |
| Spec Issue | `.agentflow/spec/issues/<issue-id>.json` | authority | Spec Loop materializer |
| Runtime Command | `.agentflow/runtime/commands/<command-id>.json` | authority | Runtime API |
| Action Proposal | `.agentflow/runtime/proposals/<proposal-id>.json` | authority | Runtime API |
| Decision | `.agentflow/runtime/decisions/<proposal-id>.json` | authority | Runtime arbitration |
| Event Store | `.agentflow/events/*.jsonl` | authority | Runtime event writer |
| Task Evidence | `.agentflow/tasks/<issue-id>/evidence/**` | authority | Work Loop evidence writer |
| Projection | `.agentflow/projections/**` | read_model | Projection rebuild |
| Pack Definitions | `.agentflow/packs/**` / `packs/**` | definition | Pack registry loader |

Desktop、Pack、Connector、Provider、Projection、SDK 都不能直接改写 authority
文件。它们必须进入 Runtime API / Command Surface。

## API Plane

本地 Runtime 的 API plane 分为四类：

| API | Capability | Boundary |
| --- | --- | --- |
| Command API | 提交 runtime command、action proposal、decision | write authority |
| Query API | 读取 projection、runtime status、diagnostics | read only |
| Event API | 追加 event、读取 event stream、触发 replay | controlled write / read |
| Pack API | 读取 pack registry、validation、simulation、command mapping | read / dry-run |

本地 Runtime 不允许通过 Query API 写入 authority。

## Worker Lifecycle

本地 worker 必须具备可追踪生命周期：

```text
registered
-> available
-> claimed
-> running
-> paused
-> completed
-> failed
-> cancelled
```

Worker lifecycle 记录必须包含：

- `workerId`
- `workerKind`
- `provider`
- `capabilities`
- `claimId`
- `commandId`
- `issueId`
- `runId`
- `startedAt`
- `updatedAt`
- `terminalReason`

Worker 可以执行命令，但不能成为任务 authority。

## Event Store Location

本地 Event Store 位于：

```text
.agentflow/events/*.jsonl
```

事件是 Runtime 的 durable fact。Projection 可以重建，Event Store 不能由 Projection
反写。

事件必须包含：

- `eventId`
- `eventType`
- `subjectType`
- `subjectId`
- `correlationId`
- `causationId`
- `createdAt`
- `payload`

## Projection Rebuild Path

Projection 位于：

```text
.agentflow/projections/**
```

Projection 是只读 view model。

重建路径是：

```text
spec facts
-> event store
-> task facts
-> pack definitions
-> projection rebuild
```

如果 Projection 缺失或 stale，本地 Runtime 必须重建；不能把 stale projection 当作
authority。

## Pack Registry Source

本地 Runtime 的 Pack source 必须可追踪：

```text
packs/**
.agentflow/packs/**
test fixtures
```

Pack registry 可以有 fallback 标记，但 fallback 不能伪装成 source of truth。

Release gate 必须能证明：

```text
pack-registry.source == fixture-files 或 file-backed
pack-registry.fallback == false
```

## Connector / Provider Capability

Connector 和 Provider 只提供能力，不拥有 authority。

Capability status 来自：

```text
.agentflow/runtime/capability-registry.json
provider smoke artifact
connector diagnostics
```

Disabled / requires-auth / degraded capability 只能影响 admission decision 和 command
availability，不能直接修改 issue、task、event 或 projection。

## Shutdown / Resume

本地 Runtime 必须支持安全停止和恢复。

Shutdown 前必须保存：

- active commands
- worker claims
- event writer cursor
- projection rebuild cursor
- provider session snapshot

Resume 后必须执行：

```text
load runtime state
-> recover incomplete commands
-> validate worker claims
-> replay missing events
-> rebuild stale projections
-> refresh capability status
```

Resume 不能重复提交已完成 command，也不能把 provider session snapshot 当作 authority。

## Local Runtime Invariants

- Runtime API 是唯一 authority 写入口。
- Event Store 是 durable fact，不由 Projection 反写。
- Projection 是只读 view model。
- Pack 定义行业现场，不写 Runtime authority。
- Connector / Provider 输出必须经过 Runtime API admission。
- Worker lifecycle 可追踪，但 worker 不拥有任务事实。
- 本地文件系统可以承载完整开发体验，但不能绕过 command / event / projection 边界。

## Non-goals

本文件不做：

- Cloud Runtime；
- remote fleet；
- 多租户权限系统；
- Message Bus go / no-go decision；
- 生产云部署；
- Runtime SDK freeze。
