# AgentFlow v1.3.1 Commercial Certification and First SKU Readiness Hardening

更新日期：2026-07-11
执行者：Codex

## Release Intent

`v1.3.1` 是 `v1.3.0` 后的 release hygiene 和 commercial readiness hardening release。

这一版不发布具体付费 SKU，也不接真实 provider / payment checkout。它只修复
`v1.3.0` 证明链里的两个核心问题：

```text
release-event certification closeout
-> certification artifact primary proof inclusion
-> quick audit manifest truth source repair
-> project roadmap commercial baseline alignment
-> first paid report SKU readiness contract
-> SKU pack boundary negative fixtures
-> adapter dry-run receipt hardening
-> v1.3.1 release certification
```

## Scope

`v1.3.1` closes:

1. v1.3.0 release-event certification closeout.
2. Certification artifact primary proof inclusion.
3. Quick audit manifest truth source repair.
4. Project roadmap commercial baseline alignment.
5. First paid report SKU readiness contract.
6. SKU pack boundary negative fixtures.
7. Adapter dry-run receipt hardening.
8. v1.3.1 release certification.

## Non-goals

`v1.3.1` does not include:

- concrete paid report SKU launch;
- provider-backed report generation;
- production payment checkout;
- public commercial launch;
- cloud multi-tenant launch;
- full customer account system;
- industry-specific SKU copy in Core Runtime.

## Primary Proof Index

| Proof | Path / URL | Purpose |
| --- | --- | --- |
| v1.3.0 release-event closeout | `proofs/v131-001-v130-release-event-certification-closeout.json` | records failed release-event run as superseded and binds final successful release-event run |
| v1.3.1 release-event closeout | `runtime/v131-release-event-certification-closeout.json` | machine-readable V131 release-event certification closeout |
| v1.3.1 certification artifact inclusion | `runtime/v131-certification-artifact-primary-proof-inclusion.json` | proves docs and runtime primary proofs are copied into full and quick certification artifacts |
| v1.3.1 quick audit manifest truth source | `runtime/v131-quick-audit-manifest-truth-source.json` | proves quick-audit manifest is derived from `certification.json.primaryProofs` and filesystem truth |
| v1.3.1 roadmap alignment | `runtime/v131-project-roadmap-commercial-baseline-alignment.json` | proves project roadmap includes v1.2.9, v1.3.0 and v1.3.1 commercial baselines |
| v1.3.1 first SKU readiness contract | `runtime/v131-first-paid-report-sku-readiness-contract.json` | freezes readiness fields, outcomes and reason codes without launching a concrete SKU |
| v1.3.1 SKU pack boundary negative fixtures | `runtime/v131-sku-pack-boundary-negative-fixtures.json` | proves missing SKU pack facts and concrete-domain leakage are rejected |
| v1.3.1 adapter dry-run receipt hardening | `runtime/v131-adapter-dry-run-receipt-hardening.json` | freezes provider/generator and payment dry-run receipt semantics without secrets or real charges |
| v1.3.1 release certification | `runtime/v131-release-certification.json` | binds all V131 primary proofs, release metadata and commercial non-launch boundary flags |

## GitHub Traceability

This release line starts with GitHub issue `#1014` and ends with `#1021`.

