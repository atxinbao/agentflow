# Commercial Release Certification

更新日期：2026-07-10
执行者：Codex

## Purpose

This document defines the `v1.3.0` Commercial Backend Stable Closure release
certification artifact.

The certification artifact is:

```text
runtime/v130-release-certification.json
```

It is the small machine-readable closeout proof for GitHub issue `#1002`.

## Contract

The certification must record:

- `releaseVersion = v1.3.0`
- `releaseTag = v1.3.0`
- `sourceCommit`
- `workflowRunId`
- `artifactNames`
- `primaryProofs`
- `primaryProofIndex`
- `commercialBackendStable = true`

## Primary Proofs

The certification must include all primary proofs from `V130-001` through
`V130-009`:

```text
docs/delivery/releases/v1.3.0/proofs/v130-001-v129-release-audit-facts.json
runtime/v130-commercial-backend-stable-contract.json
runtime/v130-paid-report-flow-state-machine.json
runtime/v130-commercial-authority-boundary.json
runtime/v130-product-sku-extension-contract.json
runtime/v130-provider-generator-adapter-boundary.json
runtime/v130-payment-provider-adapter-boundary.json
runtime/v130-customer-delivery-backend-contract.json
runtime/v130-commercial-e2e-golden-scenario.json
runtime/v130-release-certification.json
```

## Boundary Flags

The certification must keep these non-goals explicit:

```text
publicCommercialLaunch = false
concretePaidReportSku = false
paymentProviderCheckout = false
realProviderGeneration = false
cloudMultiTenantLaunch = false
fullCustomerAccountSystem = false
concreteDomainCopyInCoreRuntime = false
```

## Milestone Closeout Rule

The GitHub milestone can close only after:

1. all V130 issues are complete;
2. the release gate passes with `runtime/v130-release-certification.json`
   included.

The certification artifact does not implement a concrete SKU, provider
generation, checkout, charge, refund, public launch, cloud tenant, or customer
account system.
