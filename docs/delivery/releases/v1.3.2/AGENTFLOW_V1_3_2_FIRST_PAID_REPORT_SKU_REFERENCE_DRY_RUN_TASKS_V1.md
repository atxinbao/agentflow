# AgentFlow v1.3.2 First Paid Report SKU Reference Dry-run Tasks

更新日期：2026-07-12
执行者：Codex

## Goal

Move the first Paid Report SKU from readiness planning into a Product / Pack sourced
dry-run reference that can be certified by release gate.

## Boundary

`v1.3.2` keeps commercial launch disabled:

- no real provider generation;
- no production payment checkout;
- no public commercial launch;
- no cloud multi-tenant launch;
- no full customer account system;
- no SKU-specific Core Runtime authority.

## Issues

| Issue | Task | Depends on | Acceptance proof |
| --- | --- | --- | --- |
| `#1023` | V132-001 v1.3.1 Release Audit Closeout | none | `runtime/v132-v131-release-audit-closeout.json` |
| `#1024` | V132-002 First Paid Report SKU Product Source | `#1023` | `runtime/v132-first-paid-report-sku-product-source.json` |
| `#1025` | V132-003 SKU Input Schema and Validation Contract | `#1024` | `runtime/v132-sku-input-schema-validation-contract.json` |
| `#1026` | V132-004 SKU Report Template / Output Schema Contract | `#1024`, `#1025` | `runtime/v132-sku-report-template-output-schema-contract.json` |
| `#1027` | V132-005 SKU Dry-run Generator Adapter Receipt | `#1024`, `#1025`, `#1026` | `runtime/v132-sku-dry-run-generator-adapter-receipt.json` |
| `#1028` | V132-006 SKU Order-to-Delivery Dry-run Golden Path | `#1024`, `#1025`, `#1026`, `#1027` | `runtime/v132-sku-order-to-delivery-dry-run-golden-path.json` |
| `#1029` | V132-007 SKU Customer Delivery Preview Projection | `#1028` | `runtime/v132-sku-customer-delivery-preview-projection.json` |
| `#1030` | V132-008 SKU Negative Fixtures and Core Pollution Gate | `#1024`-`#1029` | `runtime/v132-sku-negative-fixtures-core-pollution-gate.json` |
| `#1031` | V132-009 First SKU Product Documentation and Roadmap Alignment | `#1024`, `#1028`, `#1029`, `#1030` | `runtime/v132-first-sku-product-documentation-roadmap-alignment.json` |
| `#1032` | V132-010 v1.3.2 Release Certification | all previous | `runtime/v132-release-certification.json` |

## Completion Standard

`v1.3.2` is complete when:

1. all Product / Pack SKU source files are present under `products/commercial-runtime/skus/first-paid-report-reference/**`;
2. release gate rejects invalid input, invalid output, missing adapter receipt, delivery without accepted decision and Core pollution;
3. `crates/**` does not become SKU authority;
4. docs and roadmap explicitly call this a dry-run reference, not a public launch;
5. release certification includes all V132 proofs as primary proofs.
