# Product SKU Extension Contract V1

更新日期：2026-07-10
执行者：Codex

## Purpose

本文定义 `v1.3.0` 的 Product SKU extension contract。

它回答一个问题：

```text
具体付费报告 SKU 如何接入 generic Paid Report backend，
同时不把行业语义、价格、生成器或报告模板写进 Core Runtime。
```

## Authority Boundary

Core Runtime owns:

- generic Paid Report backend contract；
- Paid Report lifecycle state machine；
- commercial authority writer map；
- SKU field validation and failure reasons。

Product / Pack / SKU owns:

- concrete `skuId`；
- concrete `productId` binding；
- required inputs；
- report sections；
- evidence / decision / delivery policies；
- pricing reference；
- generator reference；
- source references。

Core Runtime must not own:

- concrete industry terms；
- concrete report copy；
- model prompts；
- pricing content；
- provider-specific generator implementation；
- production payment provider checkout / charge / refund execution。

## Machine-readable Contract

Release gate proof:

```text
runtime/v130-product-sku-extension-contract.json
```

Contract version:

```text
agentflow-product-sku-extension-contract.v1
```

Required SKU fields:

```text
skuId
productId
requiredInputs
reportSections
evidencePolicy
decisionPolicy
deliveryPolicy
pricingRef
generatorRef
sourceRefs
```

## Missing SKU Rule

Missing SKU definition must not silently fall back to generic hardcoded report
content.

Correct result:

```text
status = invalid
unavailableReason = missing-sku-definition
canMaterializeProductInstance = false
fallsBackToGenericHardcodedContent = false
```

## Synthetic Fixture Boundary

`v1.3.0` includes only a synthetic SKU fixture to prove the contract shape.
It does not introduce a real industry SKU.

The synthetic fixture can prove:

- required fields；
- valid ready state；
- Product / Pack / SKU authority boundary；
- missing SKU negative fixture；
- synthetic sidecar cannot become live SKU authority。

It cannot prove:

- real report generation；
- real pricing；
- real checkout；
- real industry product readiness；
- public commercial launch。

## Negative Fixtures

Required negative fixtures:

```text
missing-sku-definition
core-runtime-domain-term-as-authority
synthetic-sku-sidecar-promoted-as-live-product
```

Each fixture must:

- expose machine-readable `failureReason`；
- keep `fallsBackToGenericHardcodedContent = false`；
- prevent Core Runtime from accepting concrete domain semantics as authority。

## Release Gate

`scripts/verify_release_gate.sh` must run:

```text
cargo run -p agentflow-runtime-api --example v130_product_sku_extension_contract_proofs -- \
  <runtime-dir>/v130-product-sku-extension-contract.json
```

The gate fails if:

- any required field is missing；
- the synthetic fixture is not ready；
- missing SKU can materialize a product instance；
- missing SKU falls back to hardcoded content；
- Core Runtime authority text contains concrete domain terms；
- synthetic SKU sidecar is promoted as live SKU authority。

## Non-goals

This contract does not implement:

- concrete paid-report industry SKU；
- model/provider final report generation；
- public commercial launch；
- cloud multi-tenant launch；
- customer account system；
- production payment provider checkout / charge / refund execution。
