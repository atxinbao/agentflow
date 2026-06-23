# AgentFlow v0.9.0 Deployment Shape and Runtime Governance Tasks V1

日期：2026-06-23
执行者：Codex

## Goal

`v0.9.0` 聚焦 Deployment Shape and Runtime Governance。

主线是：

```text
Local Runtime Boundary
-> Cloud Runtime Boundary
-> Runtime API / SDK Contract
-> Event Replay / Projection Rebuild
-> Migration Execution
-> Simulation / Evaluation
-> Runtime Governance
-> Scheduling Decision
-> Deployment Evidence / Rollback
-> Release Certification
```

本版本要回答的是：

```text
AgentFlow Runtime 如何从本地项目工具升级成可部署、可治理、可重放的 Project Runtime？
```

## Architecture Principle

`v0.9.0` 不能把部署问题误解成“把所有模块塞进云端”。

正确结构是：

```text
Local Runtime = 本地完整开发体验和文件事实源操作边界。
Cloud Runtime = Runtime Core + API plane + governed execution boundary。
Runtime API / SDK = 行业客户端、connector、worker 的统一入口。
Event Replay = 事实可重建。
Migration = 定义升级可控执行。
Simulation = 执行前预测影响。
Governance = 权限、capability、connector、provider、audit sidecar 的统一政策。
Message Bus = 到 decision gate 再判断是否需要。
```

## Issues

| Issue | Title | Priority | Dependency | Status |
| --- | --- | --- | --- | --- |
| `V090-001` | Local Runtime Boundary | P0 | v0.8.1 closeout | done |
| `V090-002` | Cloud Runtime Boundary | P0 | V090-001 | done |
| `V090-003` | Runtime API / SDK Contract Hardening | P0 | V090-001, V090-002 | done |
| `V090-004` | Event Replay and Projection Rebuild | P0 | V090-003 | planned |
| `V090-005` | Ontology / Pack Migration Execution Model | P0 | V090-004 | planned |
| `V090-006` | Simulation / Evaluation Layer | P0 | V090-003, V090-005 | planned |
| `V090-007` | Runtime Governance Policy | P0 | V090-003, V090-006 | planned |
| `V090-008` | Cross-process Scheduling Decision Gate | P1 | V090-007 | planned |
| `V090-009` | Deployment Evidence and Rollback Model | P0 | V090-001, V090-002, V090-003, V090-004, V090-005, V090-006, V090-007 | planned |
| `V090-010` | v0.9.0 Release Certification | P0 | V090-001, V090-002, V090-003, V090-004, V090-005, V090-006, V090-007, V090-008, V090-009 | planned |

## V090-001 Local Runtime Boundary

### Scope

明确本地 Runtime 的数据目录、进程边界、API plane 和 worker 生命周期。

必须处理：

- local runtime data directory；
- local API plane；
- local worker lifecycle；
- local event store location；
- local projection rebuild path；
- local pack registry source；
- local connector / provider capability status；
- local runtime shutdown / resume behavior。

### Acceptance

- 文档或 schema 明确本地 Runtime 的 authority 边界；
- 本地 Runtime 不能绕过 command / event / projection 边界；
- local worker 生命周期可追踪；
- 本地数据目录和 Pack 目录职责清晰。

## V090-002 Cloud Runtime Boundary

### Scope

定义云端 Runtime 只承载 Runtime Core，不绑定行业 UI。

必须处理：

- Cloud Runtime 的职责边界；
- Cloud Runtime 与行业客户端的 API 边界；
- Cloud Runtime 与 Pack / Connector 的加载边界；
- Cloud Runtime 与 local filesystem source 的关系；
- Cloud Runtime 不直接拥有行业页面。

### Acceptance

- Cloud Runtime 只暴露 Runtime API / SDK；
- 行业 UI 通过 API / Projection 读取，不进入 Runtime Core；
- Connector 输出不能直接写 authority；
- 文档说明 local / cloud 的不同部署形态。

## V090-003 Runtime API / SDK Contract Hardening

### Scope

固化 command / query / event API，给行业客户端和 connector 使用。

必须处理：

- Command API；
- Query API；
- Event API；
- Projection API；
- capability lookup；
- Pack registry lookup；
- rejected validation report；
- action proposal lifecycle；
- idempotency / trace id / correlation id。

### Acceptance

- API contract 明确输入、输出、错误和状态；
- SDK 不允许绕过 Runtime authority；
- tests 或 fixtures 覆盖 command / query / event 基础路径；
- API contract 能服务 Software Dev Pack 和 UI Design Pack。

## V090-004 Event Replay and Projection Rebuild

### Scope

让 Event Store 可以重放，Projection 可以稳定重建。

必须处理：

- replay input；
- replay ordering；
- projection rebuild；
- replay failure report；
- schema / pack version compatibility；
- deterministic rebuild evidence；
- corrupted / missing event handling。

### Acceptance

- Event Store replay 能重建 read model；
- Projection rebuild 不依赖旧缓存；
- replay 失败产生结构化 report；
- release gate 覆盖 replay happy path 和 failure path。

## V090-005 Ontology / Pack Migration Execution Model

### Scope

从 preview 进入受控 apply，但必须有 receipt、confirmation 和 rollback / cancel 语义。

必须处理：

- migration preview；
- human or policy confirmation；
- migration apply；
- applied receipt；
- cancel / rollback rule；
- event replay compatibility；
- Pack version compatibility；
- failed migration report。

### Acceptance

- migration apply 不能绕过 confirmation；
- applied receipt 与 preview receipt 明确区分；
- rollback / cancel 语义可验证；
- migration 后 replay / projection rebuild 可运行或给出结构化失败。

## V090-006 Simulation / Evaluation Layer

### Scope

执行动作前能预测影响、证据需求、冲突和后续状态。

必须处理：

- action dry-run；
- object impact preview；
- required evidence preview；
- state transition preview；
- conflict preview；
- downstream trigger preview；
- acceptance gate impact preview。

### Acceptance

- simulation 不写 authority；
- simulation report 能说明将影响哪些对象；
- simulation 能暴露缺失证据和冲突；
- release gate 覆盖至少一个 Pack command simulation。

## V090-007 Runtime Governance Policy

### Scope

统一权限、角色、capability、connector、provider、audit sidecar 的治理规则。

必须处理：

- role policy；
- action policy；
- capability policy；
- connector policy；
- provider smoke policy；
- audit sidecar policy；
- rejected / deferred / allowed decision；
- policy trace evidence。

### Acceptance

- Governance Policy 能做 allow / reject / defer；
- policy decision 可追溯；
- disabled capability 和 failed provider smoke 能影响 Runtime admission；
- audit sidecar 仍然独立，不回到主链。

## V090-008 Cross-process Scheduling Decision Gate

### Scope

到这里再判断是否需要真正 Message Bus。

不要提前中心化。

必须处理：

- 当前同步 Runtime 是否足够；
- 是否存在跨进程 worker 调度需求；
- 是否存在 cloud runtime fan-out 需求；
- 是否存在事件广播和订阅需求；
- 如果需要 Message Bus，定义 envelope、ordering、retry、idempotency；
- 如果不需要，明确 defer 到后续版本。

### Acceptance

- 输出 Message Bus go / no-go decision；
- decision 有证据，不是架构偏好；
- no-go 时明确替代机制；
- go 时只定义 contract，不直接扩大实现范围。

## V090-009 Deployment Evidence and Rollback Model

### Scope

本地 / 云端部署都要能证明版本、配置、Pack、事件、Projection 的一致性。

必须处理：

- deployment manifest；
- runtime version；
- config fingerprint；
- Pack version fingerprint；
- event store fingerprint；
- projection rebuild proof；
- migration receipt；
- rollback target；
- failed deployment report。

### Acceptance

- release artifact 能证明 local / cloud deployment shape；
- rollback model 不依赖人工口头说明；
- deployment evidence 能关联 Runtime API / Pack / Event Store / Projection；
- release gate 覆盖 deployment evidence。

## V090-010 v0.9.0 Release Certification

### Scope

发布 gate 必须覆盖 local / cloud boundary、replay、migration、simulation、governance。

必须输出：

- release certification artifact；
- local runtime boundary proof；
- cloud runtime boundary proof；
- API / SDK contract proof；
- replay / projection rebuild proof；
- migration apply receipt proof；
- simulation / evaluation report；
- governance policy decision report；
- scheduling decision report；
- deployment evidence and rollback proof；
- remaining risk / deferred list。

### Acceptance

- `v0.9.0` release gate 覆盖 V090-001 到 V090-009；
- release certification 明确是否可进入 v1.0 planning；
- Message Bus 是否进入后续版本必须有 decision record；
- 不把 Projection、Connector 或行业 UI 提升为 authority。

## Execution Order

建议执行顺序：

```text
V090-001
-> V090-002
-> V090-003
-> V090-004
-> V090-005
-> V090-006
-> V090-007
-> V090-008
-> V090-009
-> V090-010
```

`V090-010` 必须最后执行。
