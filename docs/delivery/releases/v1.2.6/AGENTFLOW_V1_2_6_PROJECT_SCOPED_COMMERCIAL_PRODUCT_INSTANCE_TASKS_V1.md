# AgentFlow v1.2.6 Project-scoped Commercial Product Instance Hardening Tasks

更新日期：2026-07-07
执行者：Codex

This document records the public delivery traceability for `v1.2.6`.

## Task Traceability

| Task | GitHub Issue | Title | Status | Primary proof |
| --- | --- | --- | --- | --- |
| V126-001 | #945 | Top-level Published Certification Kind | done | `runtime/v126-certification-kind-negative-fixture.json` |
| V126-002 | #946 | Production Registry / Negative Fixture Separation | done | `runtime/v126-production-fixture-separation.json` |
| V126-003 | #947 | Project-scoped Commercial Registry Resolver | done | `runtime/v126-project-commercial-registry-resolver.json` |
| V126-004 | #948 | Commercial Read Model Status Semantics | done | `runtime/v126-commercial-read-model-status-semantics.json` |
| V126-005 | #949 | Registry-only Commercial Golden Path | done | `runtime/v126-registry-only-commercial-golden-path.json` |
| V126-006 | #950 | Desktop Project-scoped Commercial Read Model | done | `runtime/v126-desktop-project-commercial-read-model.json` |
| V126-007 | #951 | Paid Report Product Instance Contract | done | `runtime/v126-paid-report-product-instance-contract.json` |
| V126-008 | #952 | Paid Report Preflight to Runtime Proposal Handoff | done | `runtime/v126-paid-report-preflight-runtime-proposal-handoff.json` |
| V126-009 | #953 | Commercial Negative Fixture Isolation Gate | done | `runtime/v126-commercial-negative-fixture-isolation-gate.json` |
| V126-010 | #954 | v1.2.6 Release Certification | done | `runtime/v126-release-certification.json` |

## Dependency Order

```text
#945
-> #946
-> #947
-> #948
-> #949
-> #950
-> #951
-> #952
-> #953
-> #954
```

## Certified Boundary

`v1.2.6` keeps the v1.2.5 published release and registry-backed commercial Runtime baseline, then hardens commercial product instance handling:

- final certification exposes top-level `certificationKind`；
- production registry and negative fixture registry are separated；
- projectRoot is used by Runtime and Desktop commercial read model commands；
- missing project registry returns non-ready state；
- aggregate status no longer hides per-entry availability；
- primary commercial proof is registry-only；
- Paid Report product instance defines inputs, report definition, evidence, decision and delivery requirements；
- allowed preflight creates Runtime proposal handoff only；
- negative fixture isolation is certified by release gate。

## Release Gate Artifacts

The release certification uses the standard release-gate artifact bundle for `v1.2.6`.

The final certification artifact must include:

```text
certificationKind
releaseVersion
releaseTag
sourceCommit
workflowRunId
artifactNames
primaryProofs
runtimeReleaseCertificationPath = runtime/v126-release-certification.json
```

## V126 Primary Proof Alignment

V126 issue traceability is certified by these primary proof paths:

```text
#945 -> runtime/v126-certification-kind-negative-fixture.json
#946 -> runtime/v126-production-fixture-separation.json
#947 -> runtime/v126-project-commercial-registry-resolver.json
#948 -> runtime/v126-commercial-read-model-status-semantics.json
#949 -> runtime/v126-registry-only-commercial-golden-path.json
#950 -> runtime/v126-desktop-project-commercial-read-model.json
#951 -> runtime/v126-paid-report-product-instance-contract.json
#952 -> runtime/v126-paid-report-preflight-runtime-proposal-handoff.json
#953 -> runtime/v126-commercial-negative-fixture-isolation-gate.json
#954 -> runtime/v126-release-certification.json
```
