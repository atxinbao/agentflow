# Runtime / Projection / Command API Plane Manifest V1

日期：2026-06-23
执行者：Codex

## Purpose

`API Plane Manifest` 是 AgentFlow 本地 API 面的只读清单。

它回答：

```text
Runtime API 有哪些写入口？
Projection API 有哪些读入口？
Command Surface 暴露哪些动作？
Connector / Provider / Audit / Release 各自属于什么边界？
哪些 API 是 authority / readonly / command / internal？
```

## Manifest Path

Release gate 生成：

```text
runtime/api-plane-manifest.json
```

Desktop Advanced 展示：

```text
高级 -> API Plane
```

CLI 生成：

```bash
agentflow api-plane manifest --output runtime/api-plane-manifest.json
```

## Required Categories

Manifest 必须覆盖：

```text
runtime_commands
projection_queries
command_surface_actions
connector_actions
provider_actions
audit_actions
release_actions
```

## Boundary Markers

每个 API entry 必须标记：

| Boundary | Meaning |
| --- | --- |
| `authority` | 正式 authority 写入口，必须由 Runtime API 控制 |
| `readonly` | 只读 projection / diagnostics 查询 |
| `command` | Command Surface / Runtime command / provider action |
| `internal` | 内部实现，不直接给 Console 或 SDK 暴露 |

每个 API entry 还必须标记：

```text
local-only
sdk-candidate
internal
```

## Runtime Position

```text
Runtime API
  -> API Plane Manifest
  -> Desktop Advanced
  -> Release Gate Certification
```

Manifest 是描述面，不是执行面。

它不调用 provider，不写 authority，不替代 Event Store 或 Projection。

## Non-goals

本版本不做：

- 远程 API 服务；
- SDK；
- Cloud Runtime API；
- API auth；
- API gateway；
- remote fleet。

