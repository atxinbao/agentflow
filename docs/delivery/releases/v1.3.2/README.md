# AgentFlow v1.3.2 First Paid Report SKU Reference Dry-run

更新日期：2026-07-12
执行者：Codex

## Release Intent

`v1.3.2` 是 `v1.3.1` 后的 first paid-report SKU reference dry-run release。

这一版不做真实商业发布。它只把第一条 Paid Report SKU 从 readiness contract
推进到 Product / Pack 源里的可验证 dry-run reference：

```text
v1.3.1 release audit closeout
-> first paid report SKU product source
-> SKU input schema and validation contract
-> SKU report template / output schema contract
-> dry-run generator adapter receipt
-> order-to-delivery dry-run golden path
-> customer delivery preview projection
-> negative fixtures and Core pollution gate
-> documentation and roadmap alignment
-> v1.3.2 release certification
```

## Scope

`v1.3.2` closes:

1. v1.3.1 release audit closeout.
2. First Paid Report SKU Product source.
3. SKU input schema and validation contract.
4. SKU report template / output schema contract.
5. SKU dry-run generator adapter receipt.
6. SKU order-to-delivery dry-run golden path.
7. SKU customer delivery preview projection.
8. SKU negative fixtures and Core pollution gate.
9. First SKU product documentation and roadmap alignment.
10. v1.3.2 release certification.

## Product Source Boundary

The first paid-report SKU reference source lives under:

```text
products/commercial-runtime/skus/first-paid-report-reference/**
```

This is Product / Pack source. It is not Core Runtime authority.

The SKU source binds:

- product id;
- SKU id;
- route;
- domain object refs;
- skill refs;
- connector refs;
- input schema;
- report output schema;
- report template;
- dry-run generator receipt contract;
- customer-safe preview projection;
- golden path and negative fixtures.

## Non-goals

`v1.3.2` does not include:

- real provider-backed report generation;
- production payment checkout;
- public commercial launch;
- cloud multi-tenant launch;
- full customer account system;
- Core Runtime hardcoded SKU behavior;
- customer-visible paid delivery outside dry-run preview.

## Primary Proof Index

| Proof | Path / URL | Purpose |
| --- | --- | --- |
| v1.3.1 release audit closeout | `runtime/v132-v131-release-audit-closeout.json` | records v1.3.1 as the source baseline for v1.3.2 |
| v1.3.2 SKU product source | `runtime/v132-first-paid-report-sku-product-source.json` | proves the first paid-report SKU is Product / Pack sourced |
| v1.3.2 SKU input schema | `runtime/v132-sku-input-schema-validation-contract.json` | proves required / optional inputs and rejection reasons |
| v1.3.2 SKU output schema | `runtime/v132-sku-report-template-output-schema-contract.json` | proves report sections, required fields and delivery artifact refs |
| v1.3.2 adapter receipt | `runtime/v132-sku-dry-run-generator-adapter-receipt.json` | proves dry-run adapter receipt shape without provider or billing calls |
| v1.3.2 golden path | `runtime/v132-sku-order-to-delivery-dry-run-golden-path.json` | proves order-to-delivery dry-run chain and failure path non-readiness |
| v1.3.2 preview projection | `runtime/v132-sku-customer-delivery-preview-projection.json` | proves customer-safe preview projection and non-ready states |
| v1.3.2 negative fixtures | `runtime/v132-sku-negative-fixtures-core-pollution-gate.json` | proves invalid paths and Core pollution are rejected |
| v1.3.2 docs alignment | `runtime/v132-first-sku-product-documentation-roadmap-alignment.json` | proves docs and roadmap state the dry-run boundary |
| v1.3.2 release certification | `runtime/v132-release-certification.json` | binds all V132 primary proofs and non-launch boundary flags |

## GitHub Traceability

This release line starts with GitHub issue `#1023` and ends with `#1032`.
