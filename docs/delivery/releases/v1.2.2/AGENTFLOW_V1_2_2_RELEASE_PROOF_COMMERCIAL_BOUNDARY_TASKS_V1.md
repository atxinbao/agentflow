# AgentFlow v1.2.2 Release Proof Hardening and Commercial Boundary Preflight Tasks

更新日期：2026-07-06
执行者：Codex

This document records the public delivery traceability for `v1.2.2`.

## Task Traceability

| Task | GitHub Issue | Title | Status | Primary proof |
| --- | --- | --- | --- | --- |
| V122-001 | #883 | v1.2.1 Dedicated Release Certification Gate | done | `runtime/v121-release-certification.json` |
| V122-002 | #884 | Root Certification Top-level Metadata Alignment | done | root `certification.json` metadata alignment |
| V122-003 | #885 | V121 Primary Proof Artifact Generation | done | `runtime/v121-certification-artifact-manifest-primary-proof-index.json` |
| V122-004 | #886 | V121 Issue Traceability and Milestone Closeout Gate | done | `runtime/v121-issue-milestone-closeout.json` |
| V122-005 | #887 | Desktop Team Workflow Surface Binding | done | `runtime/v121-desktop-team-workflow-surface-binding.json` |
| V122-006 | #888 | Commercial Boundary Contract | done | `runtime/v122-commercial-boundary-contract.json` |
| V122-007 | #889 | License / Entitlement Boundary | done | `runtime/v122-license-entitlement-boundary.json` |
| V122-008 | #890 | Paid Feature Boundary | done | `runtime/v122-paid-feature-boundary.json` |
| V122-009 | #891 | Paid Report Flow vs Managed Project Flow Contract | done | `runtime/v122-commercial-workflow-shapes.json` |
| V122-010 | #892 | v1.2.2 Release Certification | done | `runtime/v122-release-certification.json` |

## Dependency Order

```text
#883
-> #884
-> #885
-> #886
-> #887
-> #888
-> #889
-> #890
-> #891
-> #892
```

## Certified Boundary

`v1.2.2` keeps the `v1.2.1` first-run and team workflow baseline, then adds commercial boundary preflight:

- release proof hardening is separated from product/commercial scope;
- root certification metadata is aligned with runtime release certification;
- primary proof artifacts are hash-indexed and issue-indexed;
- issue / milestone closeout is a release gate proof, not a release note only;
- Desktop team workflow surfaces are Runtime-backed and read-only;
- commercial product layer is a Product surface boundary, not a Runtime authority;
- license / entitlement states only decide Product access boundary;
- paid feature gates block paid-only flows before Runtime command admission;
- paid report and managed project flows both map to Spec / Evidence / Decision / Delivery;
- payment processing, cloud multi-tenant and new industry Product work remain excluded.

## Release Gate Artifacts

The release certification uses the standard release-gate artifact bundle for `v1.2.2`.

The small certification artifact must include:

```text
releaseVersion
releaseTag
sourceCommit
workflowRunId
artifactNames
primaryProofs
runtimeReleaseCertificationPath = runtime/v122-release-certification.json
```

## Commercial Primary Proof Alignment

V122 commercial issue traceability is certified by V122-scoped primary proof
paths:

```text
#888 -> runtime/v122-commercial-boundary-contract.json
#889 -> runtime/v122-license-entitlement-boundary.json
#890 -> runtime/v122-paid-feature-boundary.json
#891 -> runtime/v122-commercial-workflow-shapes.json
```

The corresponding `runtime/v121-*` commercial files are legacy aliases only.
They must be marked non-primary and must not satisfy the V122 primary proof
manifest. The release gate negative fixture
`runtime/v122-commercial-proof-version-negative-fixture.json` rejects those
wrong-version aliases as V122 commercial primary proof.

## Post-release Closeout Repair

V123-003 records the post-release closeout repair for this release. The v1.2.2
tag, GitHub Release, and source archive remain unchanged.

The repair closes the live `v1.2.2` GitHub milestone after all V122 issues were
already closed, then records the live provider proof in the v1.2.3 release gate:

```text
repairProof: runtime/v122-milestone-closeout-repair.json
milestoneNumber: 17
milestoneState: closed
openIssues: 0
closedAt: 2026-07-06T17:40:17Z
```

This repair hardens the closeout proof chain without rewriting the published
v1.2.2 release.

## Excluded Work

The following remain outside `v1.2.2`:

- payment provider integration；
- billing / checkout implementation；
- cloud multi-tenant collaboration；
- public commercial launch；
- customer account system；
- organization administration；
- new industry Product；
- managed service operations。
