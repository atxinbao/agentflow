# Worker / Tool Capability Registry V1

日期：2026-06-23
执行者：Codex

## Purpose

`Worker / Tool Capability Registry` 定义 AgentFlow 的只读能力目录。

它回答：

```text
当前有哪些 worker / provider / connector？
它们暴露哪些 command？
它们健康状态如何？
某个 command 当前能不能出现在 Command Surface 上？
不能用时，原因是什么？
```

## Runtime Position

```text
role-policy
  -> role / tool scope

mcp
  -> provider profile / provider health / capability status

capability-registry
  -> worker registry / command availability / disabled reason

command surface
  -> reads capability-registry decision
```

## Current Crate

实现位置：

```text
crates/capability-registry
```

## Registry Scope

V1 覆盖以下 worker：

| Worker | Kind | Purpose |
| --- | --- | --- |
| `codex` | agent-provider | Codex provider session |
| `claude` | agent-provider | Claude provider session |
| `local-shell-validator` | validator | local build / test / browser smoke |
| `git-provider` | connector | git status / branch / diff |
| `github` | agent-provider / connector | PR / issue closeout connector |
| `mcp-connector` | connector | MCP provider and session read surface |
| `audit-worker` | runtime-worker | independent audit read/report/finding proposal |

## Model

核心对象：

- `CapabilityRegistry`
- `WorkerRegistryEntry`
- `WorkerCapability`
- `CommandSurfaceDecision`

每个 `WorkerRegistryEntry` 必须包含：

- `workerId`
- `kind`
- `health`
- `requiresAuth`
- `disabledReason`
- `runtimeRoles`
- `skillPacks`
- `toolKinds`
- `capabilities`

每个 capability 必须包含：

- `capabilityId`
- `command`
- `required`
- `available`
- `requiresAuth`
- `policy`
- `disabledReason`

## Command Surface Rule

Command Surface 不直接调用 provider，也不直接读取散落 health 文件。

它只读：

```text
evaluate_command(registry, workerId, command)
```

返回：

- `enabled=true`：命令可以显示为可执行；
- `enabled=false`：命令必须显示 disabled；
- `disabledReason`：禁用原因必须给用户可读解释。

## Health Boundary

`capability-registry` 不主动探测 provider。

它只消费外部传入的 `McpProviderStatus`：

```text
mcp provider health
  -> capability-registry
```

如果 provider 没有 health status：

```text
health = unknown
command disabled
disabledReason = provider <id> health has not been checked
```

## Authority Boundary

`capability-registry` 不写：

- `.agentflow/spec/**`
- `.agentflow/events/**`
- `.agentflow/tasks/**`
- `.agentflow/projections/**`
- provider session files

它不是 authority。

它只产出只读能力投影，供 Command Surface 和 runtime foundation gate 使用。

## Non-goals

V1 不做：

- Worker Fleet；
- 远程 worker 调度；
- provider auth 管理；
- provider smoke gate；
- provider session launch；
- Command Surface UI 改造；
- Pack System；
- Cloud Runtime。

Provider smoke gate 由后续 `V072-006` 处理。
