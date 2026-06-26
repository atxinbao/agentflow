# AgentFlow v0.7.2 Foundation Readiness Report V1

日期：2026-06-23
执行者：Codex

## 1. 结论

`v0.7.2` 的 Runtime Foundation hardening 已经把底层关键面收成可继续演进的基线。

当前状态不是 Pack / Cloud / 行业壳 ready，而是：

```text
Runtime Foundation baseline ready
Projection / Command / Connector / Provider / Audit / Release boundaries documented and testable
v0.8.0 can start Pack System planning on top of this foundation
```

## 2. 完成度定义

本报告使用四类判断：

| 判断 | 含义 |
| --- | --- |
| `completed` | 本阶段已经形成代码、文档和 release gate 可验证产物，可以作为后续版本的硬依赖 |
| `baseline` | 本阶段已经定义边界和最小实现，但只允许作为后续设计输入，不能被表述为完整产品化能力 |
| `deferred` | 明确不进入 `v0.7.2`，后续版本必须单独授权、单独验收，不能被 Pack System 默认继承为已完成 |
| `v0.8.0 carryover` | 不是缺陷，但必须作为 `v0.8.0` Pack System 输入 |

## 3. Foundation Readiness Matrix

| Foundation Area | Status | Evidence | v0.8.0 Carryover |
| --- | --- | --- | --- |
| Industry Input Boundary | `baseline` | `docs/architecture/current-module-boundaries.md` 固定 requirements / projects / spec / events / projections / tasks 的事实流；`docs/v0.7.2/README.md` 明确不进入行业壳 | 该 baseline 只证明行业输入不能直接写 Runtime facts；Pack System 仍需单独定义 Pack metadata 和 Pack validation |
| Standardization Boundary | `baseline` | `crates/schema-registry`、`crates/simulation`、`crates/message-bus`、`crates/capability-registry`、`crates/runtime-api` 已形成标准化模块；架构文档 `012` 到 `018` 固定边界 | 该 baseline 只证明模块边界存在；v0.8.0 仍需要为 Pack schema / Pack manifest / Pack projection 增加标准合同 |
| Runtime Core | `completed` | Audit sidecar、schema registry、simulation dry-run、message bus、capability registry、provider smoke、connector boundary、API Plane 都已进入代码或 release gate 路径 | Runtime Core 可作为 Pack runtime 的基础，但不能直接扩展成 Cloud Runtime |
| Projection Surface Output | `completed` | Desktop Advanced 已能只读展示 API Plane；release gate 能生成 runtime artifacts；Projection Query API 已作为 Console 读面存在 | Pack Console 必须继续只读 projection，不得直接读写 authority |
| Industry Products | `deferred` | `docs/v0.7.2/README.md` 和任务文档明确不做 Pack System、行业产品壳、remote fleet | v0.8.0 才能开始第一批 Pack / Industry Shell；不能引用 v0.7.2 作为行业产品完成证据 |
| Deployment / API Plane | `completed` | `agentflow api-plane manifest`、`runtime/api-plane-manifest.json`、Desktop Advanced API Plane、release gate manifest 检查 | SDK / Cloud API / remote API service 仍然 deferred |

## 4. Completed

### 4.1 Audit Sidecar

已完成：

- Work Done 不再依赖 Audit；
- Delivery Package 不再依赖 Audit；
- Completion Commit 不等待 Audit；
- Audit Surface 作为独立 sidecar；
- Finding 只能生成 follow-up proposal；
- `no-audit` 是合法 Done 状态。

验证来源：

- `docs/v0.7.2/AGENTFLOW_V0_7_2_RUNTIME_FOUNDATION_HARDENING_TASKS_V1.md`
- `crates/audit`
- `crates/task-loop`
- `scripts/verify_release_gate.sh`

### 4.2 Schema Version / Migration Registry

已完成：

- 当前 schema version 清单；
- legacy / missing-version / unknown-schema 检测；
- migration preview；
- preview receipt；
- explicit apply confirmation；
- applied receipt；
- preview 默认不写 authority。

验证来源：

- `docs/architecture/012-schema-version-migration-registry-v1.md`
- `crates/schema-registry`

### 4.3 Simulation Dry-run Runtime

已完成：

- command / issue / completion dry-run；
- expected events；
- rejected reasons；
- affected projections；
- gate impact；
- completion commit preview；
- risk / conflict 输出；
- dry-run 不写 authority / event store / provider。

验证来源：

- `docs/architecture/013-simulation-dry-run-runtime-v1.md`
- `crates/simulation`

### 4.4 Local Message Bus

已完成：

- runtime / projection / command / worker / audit channel；
- 本地 fanout / refresh signal；
- Event Store replay 到 bus envelope；
- envelope 带 `messageId / correlationId / causationId / idempotencyKey / createdAt`；
- bus 不保存 authority。

验证来源：

- `docs/architecture/014-local-message-bus-contract-v1.md`
- `crates/message-bus`

### 4.5 Worker / Tool Capability Registry

已完成：

- worker registry；
- tool capability；
- health / auth / disabled reason；
- provider smoke artifact 消费路径；
- Command Surface 可读取 capability decision。

验证来源：

- `docs/architecture/015-worker-tool-capability-registry-v1.md`
- `crates/capability-registry`

### 4.6 Provider Smoke Gate

已完成：

- 默认 clear skip；
- 显式 `PROVIDER_SMOKE=1` 才执行真实 provider smoke；
- provider health / minimal launch / session snapshot / terminal state；
- `runtime/provider-smoke-status.json` 进入 release gate artifact。

验证来源：

- `docs/architecture/016-provider-smoke-gate-v1.md`
- `crates/mcp`
- `scripts/verify_release_gate.sh`

### 4.7 Connector / MCP Boundary

已完成：

- connector 输出不算 authority；
- connector 只能输出 context / evidence / external fact；
- 外部写动作必须先变成 Runtime API / Command Surface 命令；
- provider / connector failure 进入 capability disabled reason。

验证来源：

- `docs/architecture/017-connector-mcp-boundary-v1.md`
- `crates/mcp`
- `crates/capability-registry`

### 4.8 Runtime / Projection / Command API Plane

已完成：

- `runtime_commands`
- `projection_queries`
- `command_surface_actions`
- `connector_actions`
- `provider_actions`
- `audit_actions`
- `release_actions`

每个 API entry 已标记：

- `authority`
- `readonly`
- `command`
- `internal`

验证来源：

- `docs/architecture/018-api-plane-manifest-v1.md`
- `crates/runtime-api/src/api_plane.rs`
- `agentflow api-plane manifest`
- `runtime/api-plane-manifest.json`
- Desktop Advanced `API Plane`
- `scripts/verify_release_gate.sh`

## 5. Baseline

这些能力已经有底层基线，但还不是完整产品化能力：

| Baseline | 当前边界 | 为什么不是 completed product |
| --- | --- | --- |
| Industry Input Boundary | requirements / projects / spec / events / projections / tasks 事实流已固定 | 还没有 Pack metadata、行业 schema、行业表单或行业模板 |
| Standardization Boundary | schema registry、simulation、message bus、capability registry、API Plane 已存在 | 还没有 Pack-level schema registry 和 Pack migration |
| Provider Smoke | 能证明 provider 最小可用或 clear skip | 不证明长任务执行、真实外部 Codex / Claude 生产 E2E |
| Connector Boundary | 权限边界已固定 | 不包含完整 GitHub / GitLab / Linear / Figma 产品化 |
| API Plane | 本地 API 清单已生成 | 不提供 SDK、Cloud API、远程服务或 API gateway |

## 6. Deferred

这些内容明确不进入 `v0.7.2`：

- Pack System；
- Cloud Runtime；
- remote Agent fleet；
- industry product shell；
- long-running provider production E2E；
- automatic remote audit；
- remote API service；
- SDK；
- API auth / gateway。

## 7. v0.8.0 Carryover

`v0.8.0` 应基于本报告继续推进：

1. Pack manifest schema；
2. Pack registry；
3. Pack simulation / dry-run；
4. Pack projection；
5. first industry shell；
6. Pack-specific capability mapping；
7. Pack-specific API Plane extension；
8. Pack release gate coverage。

`v0.8.0` 不应该绕过 `v0.7.2` 的底层边界：

- Pack 不能直接写 authority；
- Pack UI 不能直接写 `.agentflow/**`；
- Pack connector 输出不能成为 authority；
- Pack provider smoke 不能替代 runtime fixture gate；
- Pack API 必须进入 API Plane manifest。

## 8. Release Evidence Source Boundary

`v0.7.2` 的 release evidence 分成公开交付源和 AgentFlow 事实源。

| Source | 用途 | 是否为 AgentFlow authority |
| --- | --- | --- |
| GitHub source archive | 外部审阅当前提交的源码快照 | 否 |
| PR / Release notes | 公开交付说明和变更摘要 | 否 |
| `.agentflow/spec/**` | Spec / Project / Issue authority | 是 |
| `.agentflow/events/**` | Runtime event authority | 是 |
| `.agentflow/tasks/**` | Task evidence / run artifact authority | 是 |

外部审计可以读取 GitHub archive 和 release notes，但不能把它们反向当成 `.agentflow/**` runtime fact。

## 9. Bottom-line Readiness

当前底层完成度判断：

```text
Runtime foundation: ready
Projection read surface: ready
Command boundary: ready
Connector/provider minimum boundary: ready
Release gate foundation coverage: ready
Pack / Cloud / Industry productization: not started
```

因此：

```text
v0.7.2 Runtime Foundation = complete
v0.8.0 can start Pack System on top of this foundation
v0.8.0 must still implement Pack-specific schema, validation, projection, simulation, and readiness evidence
```
