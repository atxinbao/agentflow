# AgentFlow v1.2.3 Release Closeout Proof Hardening and Commercial Surface Traceability Tasks

更新日期：2026-07-07
执行者：Codex

This document records the public delivery traceability for `v1.2.3`.

## Task Traceability

| Task | GitHub Issue | Title | Status | Primary proof |
| --- | --- | --- | --- | --- |
| V123-001 | #903 | Live GitHub Milestone Closeout Certification | done | `runtime/live-github-milestone-closeout.json` |
| V123-002 | #904 | Release Closeout Proof Cannot Self-Assert Remote State | done | `runtime/release-closeout-proof-negative-fixture.json` |
| V123-003 | #905 | v1.2.2 Milestone Closeout Repair | done | `runtime/v122-milestone-closeout-repair.json` |
| V123-004 | #906 | V122 Commercial Proof Artifact Traceability Alignment | done | `runtime/v122-commercial-proof-version-negative-fixture.json` |
| V123-005 | #907 | Commercial Product Read Model Contract | done | `runtime/v123-commercial-product-read-model-contract.json` |
| V123-006 | #908 | Paid Report Flow Preflight Contract | done | `runtime/v123-paid-report-flow-preflight-contract.json` |
| V123-007 | #909 | Managed Project Flow Commercial Boundary | done | `runtime/v123-managed-project-flow-commercial-boundary.json` |
| V123-008 | #910 | Desktop Commercial Boundary Surface | done | `runtime/v123-desktop-commercial-boundary-surface.json` |
| V123-009 | #911 | Commercial Boundary Negative Fixtures | done | `runtime/v123-commercial-boundary-negative-fixtures.json` |
| V123-010 | #912 | v1.2.3 Release Certification | done | `runtime/v123-release-certification.json` |

## Dependency Order

```text
#903
-> #904
-> #905
-> #906
-> #907
-> #908
-> #909
-> #910
-> #911
-> #912
```

## Certified Boundary

`v1.2.3` keeps the `v1.2.2` commercial boundary baseline, then adds release closeout and commercial traceability hardening:

- release closeout uses live GitHub provider evidence when GitHub issue / milestone state is part of certification；
- self-asserted closeout proof is rejected；
- the repaired `v1.2.2` milestone closeout remains traceable without rewriting the published tag；
- wrong-version commercial proof aliases cannot satisfy current release primary proof requirements；
- commercial Product read model is projection-only and cannot submit Runtime authority writes；
- paid report flow rejects unavailable, disabled, expired, missing, deferred or invalid entitlement / feature states before Runtime execution；
- managed project flow remains a Core Runtime workflow and cannot inherit paid report authority；
- Desktop renders commercial read model state as disabled / deferred / invalid / managed-project facts without submitting commands；
- negative fixtures cover the expected preflight and Desktop commercial boundary failures。

## Release Gate Artifacts

The release certification uses the standard release-gate artifact bundle for `v1.2.3`.

The small certification artifact must include:

```text
releaseVersion
releaseTag
sourceCommit
workflowRunId
artifactNames
primaryProofs
runtimeReleaseCertificationPath = runtime/v123-release-certification.json
```

## V123 Primary Proof Alignment

V123 issue traceability is certified by these primary proof paths:

```text
#903 -> runtime/live-github-milestone-closeout.json
#904 -> runtime/release-closeout-proof-negative-fixture.json
#905 -> runtime/v122-milestone-closeout-repair.json
#906 -> runtime/v122-commercial-proof-version-negative-fixture.json
#907 -> runtime/v123-commercial-product-read-model-contract.json
#908 -> runtime/v123-paid-report-flow-preflight-contract.json
#909 -> runtime/v123-managed-project-flow-commercial-boundary.json
#910 -> runtime/v123-desktop-commercial-boundary-surface.json
#911 -> runtime/v123-commercial-boundary-negative-fixtures.json
#912 -> runtime/v123-release-certification.json
```

## Excluded Work

The following remain outside `v1.2.3`:

- payment provider integration；
- billing / checkout implementation；
- cloud multi-tenant launch；
- public commercial launch；
- customer account system；
- organization administration；
- new industry Product；
- managed service operations；
- rewriting `v1.2.2` release history。
