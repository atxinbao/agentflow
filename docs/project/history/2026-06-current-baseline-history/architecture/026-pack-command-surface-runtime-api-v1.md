# Pack Command Surface Runtime API V1

创建日期：2026-06-23
执行者：Codex

## 目标

本文件定义 Pack command 如何进入 AgentFlow Runtime。

核心规则：

```text
Pack command
-> Pack Surface route
-> Action Contract mapping
-> Runtime Command
-> Arbitration
-> Event / Projection
```

Pack 只能提供行业定义、页面入口和命令映射，不能直接写 Runtime authority。

## Command Surface 边界

Pack command surface 提供六类入口：

| Entry | Boundary | 说明 |
| --- | --- | --- |
| `list_pack_commands` | readonly | 列出 built-in / local Pack 暴露的命令 |
| `validate_pack_command` | readonly | 校验 Pack command 是否能映射到 Runtime command |
| `dry_run_pack_command` | readonly | 预览命令影响，不写 authority、不写 event store、不执行 provider |
| `submit_pack_action_proposal` | command | 把 Pack command 转成 Runtime command，并进入 arbitration |
| `query_pack_capability_status` | readonly | 查询 Pack command 需要的 provider capability |
| `query_pack_surface_route` | readonly | 查询命令对应页面、route 和 action contract |

## 禁止事项

Pack command 不允许：

- 绕过 Action Contract；
- 绕过 Arbitration；
- 直接写 `.agentflow/spec/**`；
- 直接写 `.agentflow/events/**`；
- 直接写 `.agentflow/tasks/**`；
- 直接调用 provider / MCP；
- 把 Command Surface 变成新的事实源。

## Runtime Mapping

Pack command 必须先映射为 AgentFlow 现有 Runtime command。

当前内置映射：

| Pack command | Action contract | Runtime command | Target |
| --- | --- | --- | --- |
| `spec.intake.start` | `action-contract:spec.intake` | `submitRequirement` | `Requirement` |
| `work.issue.start` | `action-contract:issue.start` | `startRun` | `Issue` |
| `acceptance.evaluate` | `action-contract:acceptance.evaluate` | `runValidation` | `Run` |
| `delivery.open` | `action-contract:delivery.open` | `prepareDelivery` | `Run` |
| `audit.request.sidecar` | `action-contract:audit.request` | `requestAudit` | `Issue` |

如果 Pack command 没有受支持的 Runtime command 映射，必须返回可读 rejection reason。

## Dry-run 规则

`dry_run_pack_command` 只做影响预览：

```text
writesAuthority = false
writesEventStore = false
executesProvider = false
wouldSubmitToArbitration = true/false
```

它可以返回：

- Runtime command preview；
- expected events；
- affected projections；
- rejected reasons。

但它不能写入任何 Runtime authority。

## Submit 规则

`submit_pack_action_proposal` 是唯一写侧 Pack command 入口。

它必须：

1. 解析 Pack Surface route；
2. 校验 Action Contract mapping；
3. 构造 `RuntimeCommandRequest`；
4. 调用 `execute_command_via_arbitration`；
5. 返回标准 `RuntimeCommandResponse`。

因此，Pack command 的所有写动作仍由 Runtime API 负责，不由 Pack 负责。

## API Plane

API Plane 必须明确标记 Pack command entry：

| API | Boundary |
| --- | --- |
| `pack.command.list` | readonly |
| `pack.command.validate` | readonly |
| `pack.command.dry-run` | readonly |
| `pack.command.submit-proposal` | command |
| `pack.capability.status` | readonly |
| `pack.surface.route` | readonly |
| `pack.command.resolve-runtime` | internal |

`pack.command_surface` category 不能出现 `authority` boundary。

## 实现位置

- `crates/runtime-api/src/pack.rs`
- `crates/runtime-api/src/api_plane.rs`

## 验收

- Pack command 能列出 Software Dev / UI Design 的内置命令；
- `work.issue.start` 能映射到 `startRun`；
- invalid Pack command 返回可读 rejection reason；
- dry-run 不写 `.agentflow/**`；
- submit 通过 Runtime Arbitration 入口；
- API Plane manifest 标记 readonly / command / internal 边界。
