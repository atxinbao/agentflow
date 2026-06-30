# Core Read Model Schema V1

日期：2026-06-30
执行者：Codex

## Purpose

本文件定义 Core read model schema 的稳定边界。

目标：

```text
Spec / Evidence / Decision / Delivery authority
-> Projection Kernel
-> stable Core read models
-> Desktop / SDK / Industry Product Surface
```

Core read model 是只读展示合同，不是事实源。

## Stable Schema Family

v1.0.8 冻结四类 Core read model：

| Model kind | Version | Object type | Source refs |
| --- | --- | --- | --- |
| `spec` | `core-spec-read-model.v1` | `SpecObject` | `spec-authority`, `event-authority` |
| `evidence` | `core-evidence-read-model.v1` | `EvidenceObject` | `task-evidence-authority`, `event-authority` |
| `decision` | `core-decision-read-model.v1` | `DecisionObject` | `decision-authority`, `spec-authority`, `task-evidence-authority`, `event-authority` |
| `delivery` | `core-delivery-read-model.v1` | `DeliveryObject` | `delivery-authority`, `event-authority` |

## Required Fields

每个 Core read model 必须包含：

```text
objectId
objectType
readModelVersion
sourceRefs
freshness
status
reasonLinks
evidenceLinks
authorityBoundary
updatedAt
```

## Freshness States

所有 Core read model 共享：

```text
fresh
stale
invalid
deferred
```

`invalid` / `deferred` 必须带 reason links，不能被降级成 `fresh`。

## Authority Boundary Fields

每个 Core read model 必须声明：

```text
writesAuthority
projectionAuthority
sourceAuthority
readOnly
```

规则：

- `writesAuthority=false`
- `projectionAuthority=false`
- `readOnly=true`
- `sourceAuthority` 必须指向上游 authority kernel

## Negative Fixtures

必须拒绝以下组合：

| Fixture | Meaning |
| --- | --- |
| `spec-read-model-missing-spec-source-ref` | Spec read model 缺少 Spec authority |
| `evidence-read-model-missing-evidence-ref` | Evidence read model 缺少 task evidence authority |
| `decision-read-model-missing-evidence-ref` | Decision read model 缺少 decision / evidence authority |
| `delivery-read-model-missing-public-record-ref` | Delivery read model 缺少 delivery authority |

这些 fixture 的结果必须是：

```text
expectedResult = rejected
```

## Release Gate Evidence

Release gate 必须生成：

```text
runtime/core-read-model-schema.json
```

该文件必须证明：

- schema family 覆盖 `spec` / `evidence` / `decision` / `delivery`；
- 每个 schema 都有 identity / version / source refs / freshness / status；
- 每个 schema 都有 reason links / evidence links；
- 每个 schema 都有 authority boundary fields；
- negative fixtures 覆盖缺失 source refs 和非法组合；
- Projection 不写 authority。

## Non-goals

- 不暴露 raw `.agentflow` runtime internals 给 apps；
- 不把 Software Dev 专用对象名编码进 Core schema；
- 不让 GitHub issues、provider sessions、CLI sessions 成为 read model authority；
- 不实现具体 Industry Product Surface UI。
