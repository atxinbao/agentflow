# Connector Pack Contract V1

日期：2026-06-23
执行者：Codex

## Purpose

`Connector Pack Contract` 定义 Pack 如何声明外部工具、provider、MCP 和 capability mapping。

它回答：

```text
一个行业 Pack 需要哪些外部 connector？
connector 需要哪些 capability？
connector health 从哪里来？
provider smoke 是否影响命令可用性？
外部写动作是否能直接改变 AgentFlow authority？
connector output 能否成为项目事实源？
```

本合同只定义 connector 能力声明和可用性映射。

它不实现完整生产级外部 connector 套件，不绕过 Runtime command boundary，也不把 connector output 当项目事实源。

## Connector Path

Connector Pack 固定放在：

```text
.agentflow/packs/<pack-id>/connectors/
```

Connector 入口文件后续可以由 manifest 指向，但 Connector schema 的逻辑模型必须至少表达：

```text
connector id
provider type
supported actions
required capability
health source
smoke policy
evidence output
disabled reason
command boundary
```

## Runtime Boundary

Connector Pack 是能力定义，不是事实源。

它不能直接写：

```text
docs/requirements/**
.agentflow/spec/**
.agentflow/events/**
.agentflow/projections/**
.agentflow/tasks/**
.agentflow/audit/**
task done state
completion authority
release authority
audit authority
```

Connector 输出只能作为：

```text
context
evidence
external-fact
```

外部写动作必须转成：

```text
Runtime Command / Action Proposal
```

再经过：

```text
Command Surface
-> Runtime API
-> Arbitration
-> Event Store
-> Projection
```

## Capability Registry Mapping

`agentflow-capability-registry` 负责把 Connector Pack 中的 capability requirement 映射到当前 worker / provider 的可用性。

映射规则：

- Pack connector 声明 `requiredCapability`；
- Connector provider type 映射到 worker id；
- Capability Registry 读取 worker health、capability availability 和 provider smoke status；
- provider smoke 失败时，相关 Pack command 不可用；
- disabled reason 统一来自 `capability-registry.disabled-reason` 语义。

这保证 Pack 只描述需要什么能力，不自己判断外部工具是否可用。

## Software Dev Connectors

Software Dev Pack 的初始 Connector：

```text
GitHub
Git
Codex
Claude
Browser Preview
```

初始能力方向：

| Connector | Capability |
| --- | --- |
| GitHub | `repo.read`、`pull_request.create` |
| Git | `git.status`、`git.diff` |
| Codex | `launch`、`build_agent.complete` |
| Claude | `launch` |
| Browser Preview | `browser_preview.smoke` |

## UI Design Connectors

UI Design Pack 的初始 Connector：

```text
Figma
image assets
frontend repo
design export
browser preview
```

UI Design Connector 只表达设计现场需要的外部能力，不把设计输出直接写成 Runtime authority。

## Schema Anchor

`agentflow-pack` 提供 Connector Pack 的最小 schema：

```text
PackConnectorDefinition
PackConnectorValidationReport
software_dev_connector_definition()
ui_design_connector_definition()
validate_connector_definition()
```

`agentflow-capability-registry` 提供 Pack connector 可用性映射：

```text
evaluate_pack_connector_commands()
PackConnectorCommandDecision
```

校验规则至少保证：

- `writesAuthority` 必须为 `false`；
- connector 必须声明 provider type；
- connector 必须声明 supported actions；
- connector 必须声明 required capabilities；
- supported action 必须引用 connector required capability；
- connector output 不能成为 authority；
- command boundary 必须要求 Runtime Command Surface；
- external write action 不能绕过 Runtime Command Surface；
- provider smoke failure 会禁用相关 Pack command。

## Non-goals

本合同不做：

- 完整生产级 GitHub / GitLab / Figma connector；
- 远程行业客户端；
- provider production E2E；
- 直接执行外部写动作；
- 直接写 Runtime authority。
