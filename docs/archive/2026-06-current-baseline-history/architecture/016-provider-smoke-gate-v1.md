# Provider Smoke Gate V1

日期：2026-06-23
执行者：Codex

## 目标

`provider-smoke-gate` 只证明外部 Agent provider 的最小可用性。

它不替代 `runtime-fixture-gate`，也不升级成生产级 provider E2E。

最小证明链是：

```text
provider health
-> minimal launch request
-> session snapshot
-> clean terminal state
-> provider-smoke artifact
```

## 默认行为

release gate 默认不跑真实 provider smoke：

```text
PROVIDER_SMOKE=0
```

此时必须写出 clear skip：

```text
runtime/provider-smoke-status.json
status = skipped
reason = PROVIDER_SMOKE=0
```

这保证没有 provider auth 的本地和 CI 环境不会误报失败，也不会把 provider smoke 伪装成已覆盖。

## 显式执行

只有显式设置：

```text
PROVIDER_SMOKE=1
```

才执行真实 provider smoke。

执行内容：

1. 检查 provider health。
2. 如果 provider 不 ready，写 clear skip。
3. 如果 provider ready，生成最小 launch request。
4. 创建 provider session。
5. 读取 session snapshot。
6. cancel session，确保 terminal provider state 可投影。
7. 写 `provider-smoke artifact`。

## 产物

release gate 会输出：

```text
runtime/provider-smoke-status.json
runtime/provider-smoke-artifact.json
```

MCP 模块会写：

```text
.agentflow/state/mcp/provider-smoke/<provider>-<timestamp>.json
.agentflow/state/mcp/sessions/<session-id>.json
```

## 边界

Provider Smoke Gate 负责：

- provider health check；
- minimal launch request；
- session snapshot 可读；
- terminal state 可投影；
- clear skip / passed / failed 结果。

Provider Smoke Gate 不负责：

- 长时间 provider 编排；
- Work Loop 完整执行；
- Runtime fixture gate；
- 生产 provider E2E；
- 自动远程审计；
- 替代 `agent-dispatcher`。

## 与 Runtime Fixture Gate 的关系

`runtime-fixture-gate` 验证 AgentFlow 本地 runtime 链路。

`provider-smoke-gate` 验证外部 provider 的最小启动和退出。

两者必须分开：

```text
runtime-fixture-gate = AgentFlow 自身闭环
provider-smoke-gate = 外部 provider 最小可用性
```
