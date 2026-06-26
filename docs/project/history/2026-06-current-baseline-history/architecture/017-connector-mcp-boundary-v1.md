# Connector / MCP Boundary V1

日期：2026-06-23
执行者：Codex

## Purpose

`Connector / MCP Boundary` 定义 AgentFlow 对外部系统的最小权限边界。

它回答：

```text
Connector 能读什么？
Connector 能写什么？
Connector 的输出算不算 authority？
写动作是否必须通过 Runtime API / Command Surface？
Connector 失败时应该如何展示？
```

## Rule

Connector 输出不是 authority。

Connector 只能产出：

```text
Context
Evidence
External Fact
```

Connector 不能直接写：

```text
docs/requirements/**
.agentflow/spec/**
task done state
completion authority
release authority
audit authority
```

任何写动作都必须先成为 Runtime API / Command Surface 上的命令，再由 runtime gate 决定是否执行。

## Boundary Model

Connector boundary 必须显式声明：

| Field | Meaning |
| --- | --- |
| `readCapabilities` | Connector 可读取的外部事实能力 |
| `writeCapabilities` | Connector 可请求的外部写动作 |
| `authorityWrite` | 是否允许写 AgentFlow authority，必须为 `false` |
| `runtimeCommandRequired` | 写动作是否必须走 Runtime API / Command Surface，必须为 `true` |
| `outputChannels` | 输出落点，只能是 `context / evidence / external-fact` |
| `failureSurface` | 失败展示面，统一使用 `capability-registry.disabled-reason` |

## Runtime Position

```text
External System
  -> Connector / MCP Provider
  -> Connector Boundary
  -> Capability Registry
  -> Command Surface
  -> Runtime API
  -> Event Store / Projection
```

Connector 可以读取外部状态，也可以提出外部写请求。

但它不能绕过 Runtime API 直接改变 AgentFlow 的内部事实源。

## Current Workers

| Worker | Boundary |
| --- | --- |
| `git-provider` | read-only connector；输出 `context / external-fact` |
| `mcp-connector` | read-only connector；输出 `context / evidence / external-fact` |
| `github` | external write connector；PR / issue 写动作必须走 Command Surface |
| `gitlab` | external write connector；MR / issue 写动作必须走 Command Surface |
| `codex` / `claude` | agent provider；session 输出进入 `context / evidence` |
| `browser-preview` | validation connector；输出 `evidence / external-fact` |

## Failure Surface

Connector failure 不应该伪装成 authority 缺失。

失败必须展示为：

```text
command disabled
provider unavailable
provider unauthenticated
provider permission denied
capability missing
```

展示来源统一读取：

```text
capability-registry.disabled-reason
```

## Non-goals

本边界不做：

- Connector Pack；
- GitHub / GitLab / Linear / Figma 完整产品化；
- 新增远程写入能力；
- 远程审计；
- provider production E2E。

