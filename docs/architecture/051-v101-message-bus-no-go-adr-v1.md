# V101 Message Bus No-go ADR V1

日期：2026-06-26
执行者：Codex

## Status

Accepted for `v1.0.1`.

## Decision

AgentFlow `v1.0.1` 不默认引入中心化 cross-process Message Bus。
Message Bus 不能默认成为 Runtime 中心。

当前决策：

```text
decision = no-go
```

Message Bus 可以继续作为本地通知和刷新机制存在，但不能成为 authority，也不能替代 Event Store、Runtime API、Projection rebuild 或 Release certification。

## Context

`v1.0.0` 已经冻结 Project OS stable core。发布后审计确认：当前缺口不在“缺少中心化队列”，而在 release provenance、clean-room reproducibility、provider smoke proof、governance telemetry 和 public delivery audit sidecar policy。

在没有真实跨进程 worker fleet、云端 fanout、durable queue ordering 或多租户调度证据之前，引入中心化 Message Bus 会提前制造新边界：

- ordering；
- retry；
- idempotency；
- authority ownership；
- replay source；
- failure recovery；
- operator observability。

这些问题不应该在 `v1.0.1` 里用新基础设施放大。

## Why Local Runtime Is Enough

当前 `v1.0.1` 范围内，本地 Runtime 已能承担：

- command admission；
- workflow state transition；
- Event Store authority；
- projection rebuild；
- local fanout / refresh signal；
- release gate reproducibility。

因此默认路径保持：

```text
Runtime API
-> Event Store
-> Projection rebuild
-> UI / CLI read models
```

Message Bus 只允许做：

```text
notification / refresh / fanout signal
```

不能做：

```text
authority write / durable replay source / task source / release evidence source
```

## Go Criteria

未来只有出现以下证据时，才允许重新评估：

- 真实跨进程 worker 需要同一项目上并发认领任务；
- 本地 Runtime API 无法承担可靠调度；
- cloud runtime fanout 成为产品主线；
- durable queue ordering 有明确业务需求；
- Event Store replay 与 queue retry 的边界已经定义；
- operator 能看到 message claim、ack、retry、dead-letter 和 replay proof。

即使满足 go criteria，也必须先新增 contract，不允许同一 PR 同时引入实现。

## Release Gate Binding

release gate 必须继续输出：

```text
runtime/scheduling-decision.json
```

并证明：

- `decision = no-go`；
- `writesAuthority = false`；
- `expandsImplementationScope = false`；
- `alternativeMechanism` 非空；
- durable replay source 仍是 Event Store。

## Consequences

好处：

- v1 stable core 不被新基础设施扩大；
- release hardening 聚焦证据和可复现性；
- authority 边界保持清楚。

代价：

- 当前不提供 cross-process worker queue；
- 需要未来版本再评估云端 fanout；
- provider execution 仍通过 Runtime / Dispatcher 受控进入。
