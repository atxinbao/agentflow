# AgentFlow v1.2.4 Commercial Runtime Read Model and Closeout Distinction Tasks

更新日期：2026-07-07
执行者：Codex

This document records the public delivery traceability for `v1.2.4`.

## Task Traceability

| Task | GitHub Issue | Title | Status | Primary proof |
| --- | --- | --- | --- | --- |
| V124-001 | #923 | Live Closeout Distinction Semantics Fix | done | `runtime/v124-live-closeout-distinction.json` |
| V124-002 | #924 | Final Release Certification Rejects Deferred Live Closeout | done | `runtime/v124-final-closeout-certification.json` |
| V124-003 | #925 | Commercial Product Read Model Runtime API | done | `runtime/v124-commercial-product-read-model-runtime-api.json` |
| V124-004 | #926 | Commercial Product Projection Query Surface | done | `runtime/v124-commercial-product-projection-query.json` |
| V124-005 | #927 | Desktop Commercial Surface Uses Runtime Read Model | done | `runtime/v124-commercial-product-read-model-runtime-api.json` |
| V124-006 | #928 | Paid Report Preflight Runtime API | done | `runtime/v124-paid-report-preflight-runtime-api.json` |
| V124-007 | #929 | Managed Project Commercial Boundary Runtime Fixture | done | `runtime/v124-managed-project-commercial-runtime-fixture.json` |
| V124-008 | #930 | Commercial Negative Fixtures as Runtime Tests | done | `runtime/v124-commercial-negative-runtime-fixtures.json` |
| V124-009 | #931 | Commercial Surface Golden Path | done | `runtime/v124-commercial-golden-path.json` |
| V124-010 | #932 | v1.2.4 Release Certification | done | `runtime/v124-release-certification.json` |

## Dependency Order

```text
#923
-> #924
-> #925
-> #926
-> #927
-> #928
-> #929
-> #930
-> #931
-> #932
```

## Certified Boundary

`v1.2.4` keeps the v1.2.3 commercial boundary baseline, then moves the commercial surface into Runtime read model and projection facts:

- closeout state separates `hasNoOpenIssues` from `isMilestoneClosed`；
- final certification rejects deferred live closeout unless waiver fields are complete；
- Commercial Product read model is Runtime API backed；
- projection query is readonly and cannot write authority；
- Desktop consumes `load_commercial_product_read_model` and only uses Browser Preview fixture when runtime is unavailable；
- Paid Report preflight is a Runtime API decision point and still requires Runtime command admission；
- Managed Project fixture proves commercial entitlement does not change Core Runtime authority；
- negative fixtures run as Runtime tests / artifacts；
- golden path proves read model -> projection -> Desktop -> preflight continuity。

## Release Gate Artifacts

The release certification uses the standard release-gate artifact bundle for `v1.2.4`.

The small certification artifact must include:

```text
releaseVersion
releaseTag
sourceCommit
workflowRunId
artifactNames
primaryProofs
runtimeReleaseCertificationPath = runtime/v124-release-certification.json
```

## V124 Primary Proof Alignment

V124 issue traceability is certified by these primary proof paths:

```text
#923 -> runtime/v124-live-closeout-distinction.json
#924 -> runtime/v124-final-closeout-certification.json
#925 -> runtime/v124-commercial-product-read-model-runtime-api.json
#926 -> runtime/v124-commercial-product-projection-query.json
#927 -> runtime/v124-commercial-product-read-model-runtime-api.json
#928 -> runtime/v124-paid-report-preflight-runtime-api.json
#929 -> runtime/v124-managed-project-commercial-runtime-fixture.json
#930 -> runtime/v124-commercial-negative-runtime-fixtures.json
#931 -> runtime/v124-commercial-golden-path.json
#932 -> runtime/v124-release-certification.json
```

