# AgentFlow v1.0 Runtime API / SDK Freeze V1

日期：2026-06-25
执行者：Codex

```yaml
runtimeApiSdkContractVersion: agentflow-runtime-api-sdk-freeze.v1
runtimeApiSdkContractStatus: active
stableContractBaseline: agentflow-stable-contract-baseline.v1
authority: docs/core/architecture/042-v100-runtime-api-sdk-freeze-v1.md
```

## Purpose

本文档冻结 `v1.0.0` 的 Runtime API / SDK 稳定面。

它不是冻结每个 Rust 内部函数，而是冻结外部客户端、Pack、connector、provider worker 和 Desktop 可以依赖的最小合同：

```text
Command API
Query API
Event API
Decision response
Error response
Runtime API 与 CLI command 的关系
```

本文档必须引用 [041-v100-stable-contract-baseline-v1.md](041-v100-stable-contract-baseline-v1.md)，并服从其中的 Stable Public Contracts、Version Field Rule、Breaking Change Rule 和 Deprecation Rule。

## Stable API Planes

| Plane | Stable boundary | SDK visibility | Authority rule |
| --- | --- | --- | --- |
| Command API | command request、admission、decision response | submit command only | 必须先经过 Governance Admission，不能直接写 proposal 或 accepted event |
| Query API | projection read model query | readonly | 只能读 Projection，不能触发写入、provider、connector mutation |
| Event API | event replay、receipt inspect | readonly | SDK 只能 replay / inspect，append / claim 是 internal |
| CLI command | stable Runtime API 的本地调用壳 | local command surface | CLI 不能绕过 Runtime API / Governance Admission |

## Command Input Contract

Runtime command request 的稳定字段是：

```json
{
  "version": "agentflow-runtime-command-api.v1",
  "commandId": "cmd-001",
  "commandType": "project.materialize",
  "sourceSurface": "desktop",
  "actorRole": "spec-agent",
  "targetObjectRef": {
    "objectType": "project",
    "objectId": "project-001"
  },
  "input": {},
  "evidenceRefs": [],
  "artifactRefs": [],
  "idempotencyKey": "cmd-001/project.materialize",
  "createdAt": "2026-06-25T00:00:00Z"
}
```

稳定规则：

- `version` 必须是可机器读取的 schema version；
- `commandId` 必须唯一，且作为 response `correlationId` 的默认来源；
- `commandType` 必须能映射到 Runtime-controlled action proposal；
- `sourceSurface` 只能描述入口，不拥有 authority；
- `actorRole` 必须参与 role / governance 判断；
- `targetObjectRef` 指向被操作对象，不能替代 authority fact；
- `input` 是命令输入，不允许混入直接写 `.agentflow/**` 的路径操作；
- `evidenceRefs` / `artifactRefs` 只能引用已存在证据或候选产物；
- `idempotencyKey` 用于重复提交保护；
- `createdAt` 必须保留请求时间。

## Query Input Contract

Runtime query request 的稳定字段是：

```json
{
  "version": "agentflow-runtime-query-api.v1",
  "queryId": "query-001",
  "queryType": "projection.task-workbench",
  "sourceSurface": "desktop",
  "actorRole": "human-owner",
  "targetObjectRef": {
    "objectType": "issue",
    "objectId": "AF-001"
  },
  "cursor": null,
  "createdAt": "2026-06-25T00:00:00Z"
}
```

稳定规则：

- Query 只能读取 Projection / Read Model；
- Query 不能写 Event Store；
- Query 不能写 Spec / Task / Release authority；
- Query 不能触发 provider session；
- Query 不能触发 connector mutation；
- Query response 可以包含 `nextQueryHint`，但不能包含写命令执行结果。

## Event Output Contract

Event replay response 的稳定字段是：

```json
{
  "version": "agentflow-runtime-event-api.v1",
  "queryId": "event-query-001",
  "status": "accepted",
  "cursor": "cursor-001",
  "events": [
    {
      "eventId": "evt-001",
      "eventType": "runtime.action.accepted",
      "objectRef": {
        "objectType": "issue",
        "objectId": "AF-001"
      },
      "causationId": "cmd-001",
      "correlationId": "cmd-001",
      "recordedAt": "2026-06-25T00:00:01Z",
      "schemaVersion": "agentflow-runtime-event-api.v1"
    }
  ],
  "nextCursor": "cursor-002"
}
```

稳定规则：

- SDK-facing Event API 只允许 replay / inspect；
- `event.runtime.append-accepted-action` 是 internal；
- `event.task.claim` 是 internal；
- 任何 SDK candidate entry 都不能 append authority event。

## Decision Output Contract

Runtime command response 的稳定状态只有：

| Status | Meaning | Writes proposal? | Writes accepted event? |
| --- | --- | --- | --- |
| `accepted` | Governance admission 与 arbitration 均通过 | yes | yes |
| `rejected` | Governance 或 arbitration 拒绝 | no | no |
| `deferred` | Governance 要求延后或缺前置条件 | no | no |
| `failed` | Runtime 命令处理失败 | no | no |

兼容说明：

- 当前实现仍可能返回 `humanDecisionRequired`、`queued`、`superseded`、`cancelled`、`invalidCommand`；
- 对 v1 SDK 用户，它们必须映射为 `deferred`、`failed` 或只读 query 状态；
- 后续 patch release 不允许新增破坏性状态值。

## Error Model

错误响应必须稳定包含：

```json
{
  "code": "governanceRejected",
  "stage": "governance-admission",
  "reason": "runtime governance admission rejected this command",
  "evidencePath": "runtime/governance-admission.json"
}
```

稳定规则：

- `code` 是机器可判断错误类型；
- `stage` 指向失败阶段；
- `reason` 是人类可读说明；
- `evidencePath` 指向 release gate 或 runtime artifact；
- rejected / deferred / failed 必须能解释为什么没有进入 proposal 或 accepted event。

## Governance Admission Rule

Command path 必须按下面顺序执行：

```text
validate command
-> governance admission
-> action proposal
-> arbitration
-> accepted action event
-> projection refresh
```

禁止：

- rejected / deferred 写入 proposal；
- rejected / deferred append accepted event；
- CLI command 直接写 Runtime proposal；
- SDK 直接写 authority fact；
- connector / provider session 自己决定 Done。

## Runtime API And CLI Relationship

CLI 只是 Runtime API 的本地调用壳。

稳定规则：

- CLI command 必须调用 Runtime API 或 release gate 认可的 Runtime wrapper；
- CLI 输出文本不是 stable SDK contract；
- CLI 可以输出 JSON artifact，但 artifact 必须带 stable `version` 字段；
- CLI 不能新增绕过 Governance Admission 的写入口。

## Minimal SDK Surface

v1 SDK 使用者只能依赖：

| SDK surface | Stable entry |
| --- | --- |
| Submit command | `runtime.command.validate` / `runtime.command.execute` |
| Query workbench | `projection.task-workbench` |
| Query pack workbench | `projection.pack-industry-workbench` |
| Replay runtime events | `event.runtime.replay` |
| Replay task events | `event.task.replay` |
| List / validate / dry-run Pack command | `pack.command.list` / `pack.command.validate` / `pack.command.dry-run` |
| Inspect Pack capability / route | `pack.capability.status` / `pack.surface.route` |

SDK 使用者不能依赖：

- Rust crate private function signature；
- debug CLI text；
- provider process shape；
- test-only fixture path；
- Desktop component internal state；
- experimental cross-process Message Bus。

## Compatibility Fixture

Release gate 必须生成并验证：

```text
runtime/runtime-api-sdk-compatibility.json
```

该 fixture 必须证明：

- contract doc 存在且 metadata 正确；
- command / query / event / decision / error / governance / CLI / SDK section 完整；
- API Plane manifest 存在 command、query、event 三条稳定路径；
- `sdk-candidate` 全部是 readonly；
- rejected / deferred 不写 proposal 或 accepted event；
- error model 包含 code、stage、reason、evidencePath；
- SDK examples 覆盖 command、query、event。

## V100 Binding

`V100-002` 完成后，下游任务必须引用本文档：

- `V100-005` Projection contract 必须以 Query API 只读边界为前提；
- `V100-006` Evidence / Acceptance contract 必须以 Decision output 和 Error model 为前提；
- `V100-007` Executor Adapter contract 必须以 Command API 和 Governance Admission 为前提；
- `V100-010` Release Certification 必须检查 compatibility fixture。

## Non-goals

- 不发布多语言 SDK package；
- 不定义 REST / GraphQL / gRPC 传输层；
- 不冻结内部 Rust 函数签名；
- 不绑定特定云 API gateway；
- 不让 SDK 拥有 `.agentflow/**` authority 写权限。
