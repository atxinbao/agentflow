# AgentFlow v0.7.2 Runtime Foundation Hardening

日期：2026-06-23
执行者：Codex
状态：Runtime Foundation hardening / GitHub issue execution baseline

## Purpose

`v0.7.2` 用来收紧 `v0.7.x` Console 之后暴露的底层运行时边界。

这不是新的 UI 功能版本。

它的目标是：

```text
把 Audit、Migration、Simulation、Message Bus、Provider Smoke、Projection Manifest 和 Release Gate Foundation 覆盖压成可验证底座。
```

## Reading Order

1. [AGENTFLOW_V0_7_2_RUNTIME_FOUNDATION_HARDENING_TASKS_V1.md](AGENTFLOW_V0_7_2_RUNTIME_FOUNDATION_HARDENING_TASKS_V1.md)
2. [../v0.7.1/README.md](../v0.7.1/README.md)
3. [../architecture/003-workflow-schema-v1.md](../architecture/003-workflow-schema-v1.md)
4. [../architecture/current-module-boundaries.md](../architecture/current-module-boundaries.md)

## Scope

`v0.7.2` 包含：

- Audit 从 Project / Work 主链中抽离为独立 Sidecar Loop；
- schema version / migration preview 基线；
- dry-run / simulation 基线；
- local message bus 合同；
- worker / tool capability registry；
- provider-smoke-gate 最小边界；
- Connector / MCP 边界；
- Runtime / Projection / Command API plane manifest；
- foundation readiness report；
- release-gate foundation coverage。

## Non-goals

`v0.7.2` 不包含：

- Pack System；
- Cloud Runtime；
- remote Agent fleet；
- 行业产品壳；
- 完整真实 Codex / Claude 长任务生产执行；
- 自动远程审计。

## Completion Standard

`v0.7.2` 完成时，必须满足：

- Audit 不再是 Work Done / Delivery Package / Completion Commit 的阻塞条件；
- migration 默认只生成 preview，不自动改 authority；
- simulation 不写 authority / event store；
- message bus 只定义本地进程内 event envelope 和 consumer boundary；
- provider smoke gate 有最小 launch / exit / projection 标准；
- projection / command / runtime plane 有 manifest；
- release-gate 能证明 Runtime Foundation hardening 覆盖。

