# 071 - Core Decision Input Binding V1

日期：2026-06-29
执行者：Codex

## 1. 目标

Core Decision Input Binding 定义 Decision 在产生 outcome 前必须绑定哪些权威输入。

它回答：

```text
Decision 读取的 Spec / State / Evidence 是否真实存在？
Decision 能否读取 Ontology object？
Delivery 是否只是可选上下文？
Projection / Provider Session 能不能伪装成 authority？
过期输入如何阻断 Decision？
```

本合同只处理 input binding，不计算 outcome，不创建 completion，不触发 Audit。

## 2. 权威来源

```text
crates/ontology/src/decision.rs
docs/architecture/071-core-decision-input-binding-v1.md
release-gate runtime/core-decision-input-binding.json
```

## 3. Contract Version

Core Decision Input Binding 固定版本：

```text
agentflow-core-decision-input-binding.v1
```

## 4. Required Authority Inputs

Decision input binding 必须绑定：

| inputKind | acceptedRefKind | sourceKernel | required |
| --- | --- | --- | --- |
| `specBundle` | `SpecBundleRef` | `spec-kernel` | true |
| `ontologyObject` | `OntologyObjectRef` | `ontology-kernel` | true |
| `runtimeActionState` | `RuntimeActionStateRef` | `runtime-kernel` | true |
| `evidencePack` | `EvidencePackRef` | `evidence-kernel` | true |

## 5. Optional Context

Delivery context 可以进入 Decision input binding，但只能作为上下文：

| inputKind | acceptedRefKind | sourceKernel | required |
| --- | --- | --- | --- |
| `deliveryContext` | `DeliveryContextRef` | `delivery-context` | false |

Delivery context 不能替代 Spec / Runtime State / Evidence。

## 6. Rejected Inputs

Decision input binding 必须拒绝：

```text
ProjectionRef
ProviderSessionRef
CliSessionRef
ChatThreadRef
```

这些 ref 只能作为执行过程或只读展示事实，不能成为 Decision authority。

## 7. Freshness Rule

每个 required authority ref 必须是 current。

如果任一 required ref 标记为 stale，Decision input binding 必须阻断并输出稳定原因：

```text
decision input binding stale authority ref `<inputKind>`
```

## 8. Negative Fixtures

Release gate 必须证明以下输入会失败：

```text
missing specBundle
stale runtimeActionState
ProjectionRef as authority
ProviderSessionRef as authority
```

## 9. Release Gate

`v1.0.7` release gate 必须生成：

```text
runtime/core-decision-input-binding.json
runtime/core-decision-input-binding-rust-test.log
```

证明内容：

- `agentflow-core-decision-input-binding.v1` 已定义；
- required authority refs 完整；
- optional delivery context 被定义为非必填；
- rejected ref kinds 覆盖 Projection / Provider / CLI / Chat；
- positive fixture 绑定 Spec + Ontology + Runtime State + Evidence；
- missing / stale / projection-only / provider-session negative fixtures 会失败。
