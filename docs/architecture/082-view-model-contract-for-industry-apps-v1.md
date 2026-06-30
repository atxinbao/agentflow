# View Model Contract for Industry Apps V1

日期：2026-06-30
执行者：Codex

## Purpose

本文件定义 Industry AgentFlow apps 如何消费 Core read models。

核心规则：

```text
Industry app
-> View Model
-> Projection read model
-> Core authority facts
```

Industry app 不能直接读取或修改 Core authority facts。

## View Model Boundary

View Model 是 app-safe 展示层合同。

它只能读取 Projection read model：

- Core Spec read model；
- Core Evidence read model；
- Core Decision read model；
- Core Delivery read model。

它不能读取：

- `.agentflow/spec/**`
- `.agentflow/tasks/<issue-id>/evidence/**`
- `.agentflow/runtime/decisions/**`
- `.agentflow/release/**`

## Required View Fields

每个 app-safe View Model 必须包含：

```text
viewVersion
viewId
sourceReadModelRefs
primaryObjectRef
sections
actions
disabledReasons
staleInvalidDeferredState
readOnlyBoundary
```

## Field Mapping

View Model 必须显式映射 read model 字段：

| Read model field | View model field | Surface |
| --- | --- | --- |
| `objectId` | `primaryObjectRef` | identity |
| `status` | `sections.status` | display |
| `freshness` | `staleInvalidDeferredState` | state |
| `reasonLinks` | `disabledReasons` | explanation |
| `evidenceLinks` | `sections.evidence` | evidence |
| `authorityBoundary` | `readOnlyBoundary` | boundary |

## State Rule

View Model 必须原样暴露：

```text
fresh
stale
invalid
deferred
```

`stale` / `invalid` / `deferred` 不能被隐藏成 normal ready UI。

## Command Surface Rule

View Model 里的 `actions` 只能描述 Runtime action proposal：

```text
commandSurface = runtime-action-proposal-only
allowedActionKind = proposal
```

View Model 不允许直接写 authority，不允许绕过 Runtime API。

## Stable Surfaces

v1.0.8 先冻结两个 app-safe surface：

| Surface id | Source read models | Purpose |
| --- | --- | --- |
| `industry.project-home` | Spec / Decision | 项目首页只读状态和下一步建议 |
| `industry.task-workbench` | Spec / Evidence / Decision / Delivery | 任务状态流、证据、判定和交付槽位 |

## Negative Fixtures

必须拒绝：

| Fixture | Forbidden read |
| --- | --- |
| `industry-view-direct-spec-authority-read` | `.agentflow/spec/**` |
| `industry-view-direct-evidence-authority-read` | `.agentflow/tasks/<issue-id>/evidence/**` |
| `industry-view-direct-decision-authority-read` | `.agentflow/runtime/decisions/**` |
| `industry-view-direct-delivery-authority-read` | `.agentflow/release/**` |

所有 fixture 的结果必须是：

```text
expectedResult = rejected
```

## Release Gate Evidence

Release gate 必须生成：

```text
runtime/core-view-model-contract.json
```

该文件必须证明：

- View Model contract version 是 `projection-view-model-contract.v1`；
- View Model 不写 authority；
- View Model 不直接读 authority；
- field mappings 覆盖 identity / display / state / explanation / evidence / boundary；
- surface 覆盖 project home 和 task workbench；
- stale / invalid / deferred 状态可见；
- negative fixtures 覆盖 forbidden authority reads。

## Non-goals

- 不实现商业 Software Dev UI；
- 不修改设计系统；
- 不新增具体行业 app；
- 不新增 Message Bus；
- 不让 Industry app 直接读取 raw `.agentflow` authority facts。
