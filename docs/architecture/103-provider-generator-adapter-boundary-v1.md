# Provider / Generator Adapter Boundary V1

更新日期：2026-07-10
执行者：Codex

## Purpose

本文定义 `v1.3.0` 的 Provider / Generator Adapter boundary。

它回答一个问题：

```text
未来真实报告生成 provider 如何把 input snapshot + SKU definition
转换成 report artifact，同时不让 provider 调用、提示词、凭证或实现细节
成为 Core Runtime authority。
```

## Authority Boundary

Core Runtime owns:

- generic adapter request / receipt contract；
- input snapshot reference；
- SKU definition reference；
- generation request reference；
- generation receipt reference；
- output artifact reference；
- evidence refs；
- failure reasons；
- delivery blocked / remediation route semantics。

Provider / Generator Adapter owns:

- provider-specific model call；
- prompt / template execution；
- provider credential usage；
- concrete generator implementation；
- raw provider response normalization。

Provider / Generator Adapter must not own:

- accepted decision authority；
- delivery-ready authority；
- payment authority；
- Core Runtime product semantics；
- concrete SKU authority。

## Machine-readable Contract

Release gate proof:

```text
runtime/v130-provider-generator-adapter-boundary.json
```

Contract version:

```text
agentflow-provider-generator-adapter-boundary.v1
```

Required objects:

```text
inputSnapshot
skuDefinition
generationRequest
generationReceipt
outputArtifact
evidenceRefs
failureReasons
```

## Positive Fixture

`v1.3.0` uses a dry-run fixture only.

The positive fixture proves:

- input snapshot ref exists；
- SKU definition ref exists；
- generation request ref exists；
- generator ref exists；
- provider ref exists；
- generation receipt succeeds；
- output artifact ref exists；
- evidence refs exist；
- provider-specific call is not Core authority；
- delivery is not ready merely because generation succeeded。

## Negative Fixtures

Required negative fixtures:

```text
missing-input-snapshot
provider-call-promoted-to-core-authority
failed-generation-keeps-delivery-blocked
```

Each negative fixture must:

- keep `deliveryBlocked = true`；
- expose stable `failureReasons`；
- set `expectedDeliveryState = blocked`；
- avoid producing a successful report artifact。

## Stable Failure Reasons

The adapter boundary must expose stable failure reason strings:

```text
missing-input-snapshot
missing-sku-definition
generation-failed
output-artifact-missing
provider-call-cannot-write-core-authority
evidence-refs-missing
```

## Release Gate

`scripts/verify_release_gate.sh` must run:

```text
cargo run -p agentflow-runtime-api --example v130_provider_generator_adapter_boundary_proofs -- \
  <runtime-dir>/v130-provider-generator-adapter-boundary.json
```

The gate fails if:

- any required object is missing；
- dry-run success lacks input snapshot, SKU definition, generator ref,
  provider ref, receipt, output artifact or evidence refs；
- provider-specific calls are treated as Core authority；
- failed generation produces an artifact；
- failed generation does not keep delivery blocked；
- failure reasons are missing。

## Non-goals

This boundary does not implement:

- real provider/model calls；
- concrete paid-report industry SKU；
- final report content generation；
- public commercial launch；
- cloud multi-tenant launch；
- production payment checkout / charge / refund execution。
