# 037 - Cross-process Scheduling Decision Gate V1

创建日期：2026-06-25
执行者：Codex

## Purpose

本文定义 AgentFlow 对跨进程调度和 Message Bus 的 go / no-go 决策门。

核心判断：

```text
Message Bus 不能默认成为 Runtime 中心。
必须先有真实跨进程调度证据，再定义更强的调度合同。
```

## Current Decision

`v0.9.0` 当前决策是：

```text
decision = no-go
```

原因：

- Runtime API 仍能承担 command admission；
- Event Store 已经承担 durable authority 和 replay；
- 本地 Message Bus 已经足够覆盖 fanout / refresh signal；
- 当前没有必须引入跨进程 worker queue 的证据；
- 当前没有必须引入 cloud fanout queue 的证据；
- 中心化 Message Bus 会提前引入 ordering、retry、idempotency 和 authority 边界风险。

## Decision Report

实现入口：

```text
crates/message-bus
agentflow message-bus decision
```

报告版本：

```text
agentflow-scheduling-decision-report.v1
```

报告必须包含：

- `decision`: `go` / `no-go` / `defer`
- `evidence`
- `requiredContract`
- `alternativeMechanism`
- `messageBusPolicy`
- `writesAuthority = false`
- `expandsImplementationScope = false`

## Decision Rules

### Go

只有出现以下硬证据时才允许 `go`：

- 需要跨进程 worker 调度；
- 需要 cloud runtime fanout；
- 需要 durable queue；
- 本地同步 Runtime 已经无法承担当前执行链路。

`go` 也只能定义合同，不得在同一轮扩大实现范围。

必须先定义：

- envelope；
- per aggregate ordering；
- retry；
- idempotency；
- durable replay source；
- authority boundary。

### No-go

当本地 Runtime 足够时，必须输出 `no-go`，并明确替代机制。

当前替代机制：

- Runtime API 继续作为 command admission boundary；
- Event Store 继续作为 durable authority；
- Projection rebuild 继续作为 read model 刷新路径；
- Local Message Bus 只做 fanout / notification / refresh signal。

### Defer

当证据不足但存在潜在调度需求时，输出 `defer`。

`defer` 表示：

- 继续使用同步 Runtime API；
- 继续使用本地 fanout；
- 收集真实 worker / cloud fanout 运行证据；
- 不提前引入中心化 Message Bus。

## Release Gate

release gate 必须生成：

```text
runtime/scheduling-decision.json
```

并检查：

- report version 正确；
- decision 属于 `go` / `no-go` / `defer`；
- evidence 非空；
- `writesAuthority = false`；
- `expandsImplementationScope = false`；
- `go` 有 required contract；
- `no-go` 有 alternative mechanism。

## Non-goals

本轮不做：

- 分布式消息队列；
- durable queue 实现；
- cloud pub/sub；
- 多进程 worker fleet；
- Message Bus authority；
- 用 bus message 代替 Event Store。

## Invariant

无论未来是否引入更强 Message Bus：

```text
Event Store 仍是 authority。
Message Bus 只能是调度和通知设施。
```
