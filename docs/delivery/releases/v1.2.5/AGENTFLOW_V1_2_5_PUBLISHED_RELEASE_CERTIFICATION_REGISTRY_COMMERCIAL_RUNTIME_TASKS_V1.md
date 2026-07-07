# AgentFlow v1.2.5 Published Release Certification and Registry-backed Commercial Runtime Tasks

更新日期：2026-07-07
执行者：Codex

This document records the public delivery traceability for `v1.2.5`.

## Task Traceability

| Task | GitHub Issue | Title | Status | Primary proof |
| --- | --- | --- | --- | --- |
| V125-001 | #934 | Release Publication State Contract | done | `runtime/v125-release-publication-state.json` |
| V125-002 | #935 | Candidate vs Published Certification Split | done | `runtime/v125-candidate-published-certification-split.json` |
| V125-003 | #936 | Waiver Contract Consistency Fix | done | `runtime/v125-waiver-contract-consistency.json` |
| V125-004 | #937 | Product Registry-backed Commercial Read Model | done | `runtime/v125-product-registry-commercial-read-model.json` |
| V125-005 | #938 | Entitlement Source / Local License Fixture | done | `runtime/v125-entitlement-source-fixture.json` |
| V125-006 | #939 | Paid Report Product Definition Fixture | done | `runtime/v125-paid-report-product-definition.json` |
| V125-007 | #940 | Desktop Runtime-only Commercial Surface Guard | done | `runtime/v125-desktop-runtime-only-commercial-surface.json` |
| V125-008 | #941 | Commercial Golden Path From Product Registry | done | `runtime/v125-commercial-golden-path-registry.json` |
| V125-009 | #942 | Release Event Artifact Certification | done | `runtime/v125-release-event-artifact-certification.json` |
| V125-010 | #943 | v1.2.5 Release Certification | done | `runtime/v125-release-certification.json` |

## Dependency Order

```text
#934
-> #935
-> #936
-> #937
-> #938
-> #939
-> #940
-> #941
-> #942
-> #943
```

## Certified Boundary

`v1.2.5` keeps the v1.2.4 closeout distinction and commercial Runtime API baseline, then moves the commercial input source and release proof semantics forward:

- release publication state separates candidate, tagged, released and published；
- candidate certification cannot satisfy published release certification；
- waiver consistency is checked through absent, complete and invalid fixtures；
- Commercial Product read model is registry/config backed；
- entitlement source is a local Runtime input fixture；
- Paid Report product definition carries input, evidence and decision requirements；
- Desktop commercial production surface is Runtime-only and Browser Preview fallback is marked；
- commercial golden path reads from product registry；
- release-event artifact certification is self-contained and verifies published release facts。

## Release Gate Artifacts

The release certification uses the standard release-gate artifact bundle for `v1.2.5`.

The small certification artifact must include:

```text
releaseVersion
releaseTag
sourceCommit
workflowRunId
artifactNames
primaryProofs
runtimeReleaseCertificationPath = runtime/v125-release-certification.json
```

## V125 Primary Proof Alignment

V125 issue traceability is certified by these primary proof paths:

```text
#934 -> runtime/v125-release-publication-state.json
#935 -> runtime/v125-candidate-published-certification-split.json
#936 -> runtime/v125-waiver-contract-consistency.json
#937 -> runtime/v125-product-registry-commercial-read-model.json
#938 -> runtime/v125-entitlement-source-fixture.json
#939 -> runtime/v125-paid-report-product-definition.json
#940 -> runtime/v125-desktop-runtime-only-commercial-surface.json
#941 -> runtime/v125-commercial-golden-path-registry.json
#942 -> runtime/v125-release-event-artifact-certification.json
#943 -> runtime/v125-release-certification.json
```
