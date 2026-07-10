# Commercial End-to-End Golden Scenario V1

更新日期：2026-07-10
执行者：Codex

## Purpose

本文定义 `v1.3.0` 的 generic commercial backend E2E golden scenario。

它把前面冻结的合同串成一条完整链路：

```text
Product SKU
-> Order
-> Entitlement
-> Admission
-> Generation Adapter
-> Artifact
-> Evidence
-> Decision
-> Delivery
-> Access
-> Feedback
```

## Boundary

This golden scenario is generic.

It must not implement:

- concrete report domain SKU；
- provider-specific final report generation；
- production payment checkout；
- production charge / refund execution；
- customer account system；
- public commercial launch。

## Machine-readable Proof

Release gate proof:

```text
runtime/v130-commercial-e2e-golden-scenario.json
```

Proof version:

```text
agentflow-commercial-e2e-golden-scenario.v1
```

## Ordered Facts

The proof must include facts in this exact order:

```text
ProductSkuExtensionDefinition
PaidReportOrderRecord
PaidReportEntitlementAuthorization
PaidReportOrderToRunAdmission
ProviderGeneratorAdapterReceipt
PaidReportArtifact
PaidReportEvidencePack
PaidReportDecisionRecord
PaidReportDeliveryPackageProjection
PaidReportCustomerDeliveryAccessProjection
PaidReportFeedbackLoopProjection
```

Each fact must include:

- `factId`；
- `factType`；
- `contractVersion`；
- `status`；
- `authorityOwner`；
- `sourceRef`。

## Success Path

The success path must prove:

- decision outcome is `accepted`；
- delivery status is `delivery-ready`；
- download access is visible；
- access handle is generated；
- delivered artifact is not mutated；
- `nextAction = show-download`；
- all ordered facts are referenced。

## Failure / Repair Path

The repair path must prove:

- status is `failed-as-expected`；
- decision outcome is `needs-fix`；
- delivery status is `repair-needed`；
- download access is not visible；
- access handle is not generated；
- delivered artifact is not mutated in place；
- `nextAction = create-repair-proposal`。

## Certification Artifacts

The E2E proof must reference these prior artifacts:

```text
runtime/v130-commercial-backend-stable-contract.json
runtime/v130-paid-report-flow-state-machine.json
runtime/v130-commercial-authority-boundary.json
runtime/v130-product-sku-extension-contract.json
runtime/v130-provider-generator-adapter-boundary.json
runtime/v130-payment-provider-adapter-boundary.json
runtime/v130-customer-delivery-backend-contract.json
```

## Release Gate

`scripts/verify_release_gate.sh` must run:

```text
cargo run -p agentflow-runtime-api --example v130_commercial_e2e_golden_scenario_proofs -- \
  <runtime-dir>/v130-commercial-e2e-golden-scenario.json
```

The gate fails if:

- ordered facts are missing or out of order；
- any fact lacks contract version / authority owner / source ref；
- success path cannot expose access；
- repair path mutates delivered artifact；
- repair path generates access；
- proof claims a concrete domain SKU implementation；
- prior certification artifact refs are missing。
