# 014 - Local Message Bus Contract V1

创建日期：2026-06-23
执行者：Codex

## Purpose

本文定义 AgentFlow 本地 Message Bus 的边界。

核心规则：

```text
Event Store 是 authority。
Message Bus 只负责 fanout / notification / refresh signal。
```

Message Bus 不是数据库，不是队列服务，也不是新的事实源。

## Module Boundary

实现模块：

```text
crates/message-bus
```

负责：

- 定义 runtime / projection / command / worker / audit channel；
- 发布本地内存 fanout message；
- 发布 projection refresh signal；
- 发布 console refresh signal；
- 将 Event Store replay 映射成 bus envelope；
- 明确 bus replay 的 durable source 是 Event Store。

不负责：

- 保存 authority；
- 写 `.agentflow/**`；
- 替代 Event Store；
- 分布式消息队列；
- 云端 pub/sub；
- provider 执行；
- projection rebuild。

## Channels

第一版固定五个 channel：

| Channel | 用途 |
| --- | --- |
| `runtime` | runtime 状态、gate、session 通知 |
| `projection` | projection refresh / stale signal |
| `command` | console command feedback / UI refresh |
| `worker` | worker / dispatcher launch signal |
| `audit` | audit sidecar notification |

## Authority Policy

Message Bus policy 固定：

```text
storesAuthority = false
durableReplaySource = event-store
```

如果需要恢复状态，必须从 Event Store replay，而不是从 bus 内存消息恢复。

## Replay Rule

Bus replay 只能做一件事：

```text
load Event Store events
-> map to MessageBusEnvelope
-> fanout to consumer
```

Replay 不得写新的 event。

## Refresh Rule

Projection 和 Console 可以通过 bus 收到 refresh signal。

但 refresh signal 只表示：

```text
请重新读取 projection / Event Store。
```

它不能携带或替代 authority payload。

## Envelope Rule

每一条 bus message 必须是可追踪 envelope，而不是裸 payload。

必填字段：

```text
messageId
correlationId
idempotencyKey
createdAt
channel
topic
subjectType
subjectId
replaySource
```

如果 message 来自 Event Store replay，还必须保留：

```text
eventRef
causationId
```

Live fanout message 也必须有真实 `createdAt`，不得使用 `0` 作为占位时间。

## Trace Rule

Message Bus 不制造新的事实链。

它只做两种 trace：

- Event Store replay message 继承原 event 的 `correlationId / causationId / idempotencyKey`；
- live refresh message 生成只用于本地 fanout 的 `correlationId / idempotencyKey`。

这保证 UI、worker、projection consumer 能定位消息来源，但不能把 bus message 当作 authority。

## Acceptance

本边界成立时，应满足：

- Message Bus 不保存 authority；
- Event Store 仍是事实源；
- Projection refresh 可以通过 bus 触发；
- Console 可以通过 bus 刷新；
- bus replay 仍以 Event Store 为准；
- 每条 bus envelope 都有唯一 message id、真实 timestamp 和 trace key。
