# Pack Filesystem Contract V1

日期：2026-06-23
执行者：Codex

## Purpose

`Pack Filesystem Contract` 定义 Pack 在 AgentFlow 项目文件系统里的位置、职责和边界。

它回答：

```text
Pack 放在哪里？
Pack 里面有哪些固定分区？
Pack 能描述什么？
Pack 不能替代什么？
Pack 如何进入 Runtime？
```

本合同只定义文件系统边界。

它不实现 Pack loader，不定义完整 manifest schema，不迁移现有 authority，也不写行业 Pack 内容。

## Runtime Position

Pack 是定义层。

它描述行业现场、领域对象、页面入口、外部工具能力和命令映射。

Pack 不能直接推进 Runtime。
Pack 不能直接写 authority。
Pack 不能直接写事件。
Pack 不能直接写任务产物。

正确关系是：

```text
.agentflow/packs/**
  -> Pack Registry
  -> Runtime API / Command Surface
  -> Action Contract / Arbitration
  -> Event Store
  -> Projection
  -> Desktop / CLI / Industry Client
```

Pack 进入 Runtime 的唯一入口是 Runtime API / Command Surface。

## Root Path

项目级 Pack 根目录固定为：

```text
.agentflow/packs/
```

每个 Pack 使用一个稳定 `pack-id` 作为目录名：

```text
.agentflow/packs/<pack-id>/
```

示例：

```text
.agentflow/
  packs/
    software-dev/
      pack.json
      domain/
      surface/
      connectors/
    ui-design/
      pack.json
      domain/
      surface/
      connectors/
```

## Required Paths

每个 Pack 至少预留以下路径：

| Path | Meaning |
| --- | --- |
| `.agentflow/packs/<pack-id>/pack.json` | Pack manifest。描述 pack id、version、kind、dependencies、capability requirements 和入口文件。 |
| `.agentflow/packs/<pack-id>/domain/` | Domain Pack。描述行业对象、关系、动作、状态和验收语义。 |
| `.agentflow/packs/<pack-id>/surface/` | Surface Pack。描述页面、工作台、视图模型、命令入口和 projection route。 |
| `.agentflow/packs/<pack-id>/connectors/` | Connector Pack。描述外部工具、provider、MCP、文件系统和 capability mapping。 |

本 issue 不定义各文件的完整 schema。

完整 schema 由后续 Pack Manifest、Domain Pack、Surface Pack、Connector Pack 任务分别定义。

## Manifest Path

Pack manifest 固定路径：

```text
.agentflow/packs/<pack-id>/pack.json
```

`pack.json` 是 Pack 的入口清单，但不是 Runtime authority。

它只能被 Pack Registry / Validation / Simulation / Projection / Command Surface 读取。

## Domain Path

Domain Pack 固定路径：

```text
.agentflow/packs/<pack-id>/domain/
```

Domain 目录描述：

- object types；
- relationships；
- action semantics；
- status semantics；
- acceptance semantics；
- domain-specific vocabulary。

Domain 目录不写：

- `.agentflow/spec/**`；
- `.agentflow/events/**`；
- `.agentflow/tasks/**`；
- `.agentflow/audit/**`。

## Surface Path

Surface Pack 固定路径：

```text
.agentflow/packs/<pack-id>/surface/
```

Surface 目录描述：

- page routes；
- view model binding；
- command entry；
- projection route；
- read-only / command / internal surface classification；
- industry client hints。

Surface 目录只定义 UI 和命令入口的映射。

它不能让 Desktop 或行业客户端直接写 `.agentflow/**` authority。

## Connector Path

Connector Pack 固定路径：

```text
.agentflow/packs/<pack-id>/connectors/
```

Connector 目录描述：

- provider requirement；
- MCP capability requirement；
- local filesystem capability；
- external tool capability；
- smoke requirement；
- disabled reason mapping。

Connector 目录不能直接调用 provider。

实际调用必须经过 Runtime API / Command Surface，再进入 provider / MCP adapter。

## Authority Boundary

Pack 文件不能替代以下 authority：

| Authority | Meaning |
| --- | --- |
| `.agentflow/spec/**` | Requirement / Project / Issue / Contract authority。 |
| `.agentflow/events/**` | Append-only runtime event authority。 |
| `.agentflow/tasks/**` | Task-local runtime artifact and evidence authority。 |
| `.agentflow/audit/**` | Audit sidecar authority。 |

Pack 是定义，不是事实。

如果 Pack 定义需要影响 Runtime，必须通过：

```text
Pack definition
-> Runtime API / Command Surface
-> Action Proposal
-> Arbitration
-> Event Store
-> Projection
```

## Runtime Entry Boundary

Pack 允许暴露可执行意图，但不允许直接执行。

合法入口：

```text
Runtime API
Command Surface
Pack-aware simulation
Pack-aware validation
Pack-aware projection query
```

非法入口：

```text
Pack file writes .agentflow/spec/**
Pack file writes .agentflow/events/**
Pack file writes .agentflow/tasks/**
Pack UI writes authority directly
Pack connector calls provider directly
Pack loader silently mutates project state
```

## Write Boundary

本合同建立后，后续实现必须遵守：

- Pack authoring 可以写 `.agentflow/packs/<pack-id>/**`；
- Runtime execution 不把 Pack 文件当作执行结果；
- Projection 可以读取 Pack metadata，但不能把 Pack metadata 当作事件事实；
- Command Surface 可以根据 Pack 暴露命令，但必须生成 action proposal；
- Release gate 可以读取 Pack readiness artifact，但不能用 Pack 文件替代 release evidence。

## Non-goals

本合同不做：

- Pack registry；
- Pack manifest schema；
- Domain Pack schema；
- Surface Pack schema；
- Connector Pack schema；
- Pack validation；
- Pack migration；
- Pack simulation；
- Pack-aware projection；
- Pack-aware command runtime；
- Software Dev Pack 内容；
- UI Design Pack 内容；
- Pack marketplace。

