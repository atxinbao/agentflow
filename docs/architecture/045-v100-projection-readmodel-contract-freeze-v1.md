# V100 Projection / Read Model Contract Freeze V1

创建日期：2026-06-25  
执行者：Codex

projectionContractVersion: agentflow-projection-readmodel-contract.v1  
projectionContractStatus: active  
stableContractBaseline: agentflow-stable-contract-baseline.v1  
runtimeApiSdkVersion: agentflow-runtime-api-sdk-freeze.v1  
filesystemContractVersion: agentflow-filesystem-contract-freeze.v1  
packContractVersion: agentflow-pack-contract-freeze.v1  
authority: docs/architecture/045-v100-projection-readmodel-contract-freeze-v1.md

## Purpose

本文件冻结 v1.0.0 的 Projection / Read Model / View Model 合同。

核心判断：

```text
Projection 不是 authority。
Projection 是从 Event Store、Spec authority、Task artifacts、Pack definition 和 Audit sidecar 重建出来的只读读模型。
Desktop、CLI、行业 Surface 和 SDK 只能消费 Projection / Read Model，不直接读取写路径。
```

## References

- [041-v100-stable-contract-baseline-v1.md](041-v100-stable-contract-baseline-v1.md)
- [042-v100-runtime-api-sdk-freeze-v1.md](042-v100-runtime-api-sdk-freeze-v1.md)
- [043-v100-agentflow-filesystem-contract-freeze-v1.md](043-v100-agentflow-filesystem-contract-freeze-v1.md)
- [044-v100-pack-contract-freeze-v1.md](044-v100-pack-contract-freeze-v1.md)
- [025-pack-aware-projection-read-models-v1.md](../project/history/2026-06-current-baseline-history/architecture/025-pack-aware-projection-read-models-v1.md)

## Projection Authority Boundary

Projection 层只允许读：

```text
.agentflow/events/**
.agentflow/spec/**
.agentflow/tasks/**
.agentflow/packs/**
.agentflow/audit/**
.agentflow/release/**
```

Projection 层只允许写：

```text
.agentflow/projections/**
.agentflow/indexes/**
```

Projection 层不得写：

```text
.agentflow/spec/**
.agentflow/events/**
.agentflow/tasks/**
.agentflow/runtime/**
.agentflow/packs/**
.agentflow/audit/**
.agentflow/release/**
docs/**
```

任何 Projection rebuild / query / view model 产物必须显式声明：

```text
writesAuthority: false
projectionAuthority: false
```

## Stable Projection Surfaces

v1.0.0 冻结以下 Projection surface：

| Surface | Stable path / API | Purpose |
| --- | --- | --- |
| Requirement intake view | `projection.requirement-intake` | 读取需求摄入阶段 |
| Spec preview view | `projection.spec-preview` | 读取 preview artifact，不作为 authority |
| Spec loop view | `projection.spec-loop` | 读取 Spec Loop 状态、阶段、下一步 |
| Project home view | `projection.project-home` | 读取 Project 状态、当前 issue、完成度 |
| Task workbench view | `projection.task-workbench` | 读取 issue 状态流、事件流、交付槽位 |
| Work loop run view | `projection.work-loop-run` | 读取 run / validation / review / done 信息 |
| Work loop session view | `projection.work-loop-session` | 读取外部 Agent session 状态 |
| Audit surface view | `projection.audit-surface` | 读取 audit sidecar 和 findings |
| Delivery package view | `projection.delivery-package` | 读取公开交付记录和 release notes |
| Runtime health view | `projection.runtime-health` | 读取 runtime health，不推进状态 |
| Pack industry workbench view | `projection.pack-industry-workbench` | 读取 Pack-specific projection loading |

所有 surface 都是 Query API / readonly。

## Read Model Schema

Read Model 必须包含稳定字段：

```text
version
objectId
objectType
currentState / status
displayStatus
sourceRefs
eventRefs
artifactRefs
updatedAt
staleness
invalidReasons
deferredReasons
```

不同对象可以扩展自己的字段，但不得删除以上稳定字段。

## View Model Schema

View Model 是 Desktop / CLI / industry surface 可以直接消费的只读结构。

View Model 必须包含：

```text
viewVersion
viewId
sourceReadModelRefs
primaryObjectRef
sections
actions
disabledReasons
stale
invalid
deferred
```

`actions` 只能描述可以发起的 Runtime API command / proposal，不得直接写 authority。

## Rebuild Rule

Projection rebuild 必须满足：

```text
Event Store + authority facts + task artifacts + sidecars
-> deterministic projection rebuild
-> .agentflow/projections/**
-> .agentflow/indexes/**
```

规则：

- rebuild 可以重复执行；
- rebuild 不能改写 authority；
- rebuild 失败必须输出 structured failure；
- corrupt event / missing authority / invalid pack 都不能被静默忽略；
- replay report 必须列出 rebuilt projection paths；
- replay report 必须证明 `writesAuthority=false` 和 `projectionAuthority=false`。

## Freshness State

Projection freshness 只允许以下状态：

```text
current
stale
invalid
deferred
missing
```

含义：

| State | Meaning |
| --- | --- |
| `current` | projection 与输入事实同步 |
| `stale` | 输入事实比 projection 新，需要 rebuild |
| `invalid` | 输入事实缺失或不合法，不能安全展示为 ready |
| `deferred` | 依赖对象或 Pack 未就绪，等待后续事实 |
| `missing` | projection 尚未生成 |

`invalid` / `deferred` 必须展示原因，不允许 fallback 成 ready。

## Pack Projection Rule

Pack-specific projection loading 必须遵守：

```text
Pack definition
-> Pack registry
-> Pack validation / readiness
-> Pack-aware Projection
-> Industry Surface
```

规则：

- Projection 可以读取 Pack domain / surface / connector readiness；
- Projection 不得让 Pack 拥有 projection authority；
- missing Pack definition 必须显示 `invalid` 或 `deferred`；
- missing Pack definition 不得静默 fallback 到 Software Dev；
- disabled capability 只能让 command unavailable，不得污染 read model authority；
- Pack read model 必须保留 `packId`、`manifestPath`、`validationStatus`、`readinessStatus`。

## Evidence / Audit / Delivery Read Models

v1.0.0 冻结三个 sidecar read model：

### Evidence Graph Read Model

```text
task evidence
validation command result
changed files
closeout proof
acceptance decision
```

Evidence Graph 只读 `.agentflow/tasks/<issue-id>/evidence/**` 和相关 run artifact。

### Audit Sidecar Read Model

```text
audit request
audit report
findings
evidence gaps
repair recommendations
```

Audit sidecar 不改变 issue done authority。

### Delivery Read Model

```text
PR / MR body
merge proof
release notes
changelog refs
public delivery status
```

Delivery read model 展示公开交付记录是否齐全，但不直接发布 release。

## Industry Surface Rule

行业客户端、Desktop 页面、CLI inspection 和 SDK consumer 必须只读：

```text
Projection API
Read Model
View Model
```

禁止：

- UI 直接读 Event Store 写路径；
- UI 直接读 `.agentflow/tasks/**` 来推断状态；
- UI 直接写 Runtime authority；
- industry surface 在缺 Pack 时静默 fallback；
- view model 把 stale / invalid / deferred 渲染成 ready。

## Release Gate Fixture

release gate 必须生成：

```text
runtime/projection-readmodel-contract.json
```

必须验证：

- 本文档 metadata；
- Projection / Read Model / View Model 必需章节；
- Event replay happy path 可以重建 projections；
- Event replay failure path 输出 structured failure；
- replay report `writesAuthority=false`；
- replay report `projectionAuthority=false`；
- `.agentflow/projections/**` 和 `.agentflow/indexes/**` 是唯一 projection 写入面；
- Query API 的 `projection_queries` 全部 readonly；
- Pack projection missing definition 显示 invalid / deferred；
- Industry Surface 只消费 projection read model；
- Evidence / Audit / Delivery read model surface 已在 API plane 暴露。

## V100 Binding

| Downstream | Binding |
| --- | --- |
| V100-006 | Evidence / Acceptance 必须进入 Evidence Graph Read Model |
| V100-007 | Executor Adapter session 只能通过 Work loop session view 暴露给 UI |
| V100-008 | Replay / Migration 必须验证 projection rebuild 兼容性 |
| V100-009 | Software Dev Pack 必须提供可验证 Pack-specific projection |
| V100-010 | v1.0.0 release certification 必须检查 projection-readmodel-contract |

## Non-goals

- 不重做 Desktop UI；
- 不新增行业壳；
- 不把 Projection 变成 authority；
- 不让 Pack 直接写 projection；
- 不把 audit sidecar 变成 completion authority。
