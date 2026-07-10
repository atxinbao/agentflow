# AgentFlow v1.3.0 Commercial Backend Stable Closure Tasks

更新日期：2026-07-10
执行者：Codex

This document records the planned public delivery traceability for `v1.3.0`.

## Task Traceability

| Task | GitHub Issue | Title | Status | Primary proof |
| --- | --- | --- | --- | --- |
| V130-001 | #993 | v1.2.9 Release Audit and Certification Semantics Repair | done | `proofs/v130-001-v129-release-audit-facts.json` |
| V130-002 | #994 | Commercial Backend Stable Contract | done | `runtime/v130-commercial-backend-stable-contract.json` |
| V130-003 | #995 | Paid Report Flow State Machine | done | `runtime/v130-paid-report-flow-state-machine.json` |
| V130-004 | #996 | Commercial Authority Boundary Freeze | done | `runtime/v130-commercial-authority-boundary.json` |
| V130-005 | #997 | Product SKU Extension Contract | done | `runtime/v130-product-sku-extension-contract.json` |
| V130-006 | #998 | Provider / Generator Adapter Boundary | done | `runtime/v130-provider-generator-adapter-boundary.json` |
| V130-007 | #999 | Payment Provider Adapter Boundary | done | `runtime/v130-payment-provider-adapter-boundary.json` |
| V130-008 | #1000 | Customer Delivery Backend Contract | planned | TBD |
| V130-009 | #1001 | Commercial End-to-End Golden Scenario | planned | TBD |
| V130-010 | #1002 | v1.3.0 Release Certification | planned | TBD |

## Dependency Order

```text
#993
-> #994
-> #995
-> #996
-> #997
-> #998
-> #999
-> #1000
-> #1001
-> #1002
```

## V130-001 Certification Semantics Repair

`V130-001` repairs confusing wording and payload fields from `v1.2.9`:

- live GitHub release provenance is the published release authority;
- synthetic project release sidecar facts cannot satisfy published release certification;
- final certification reports concrete `tagKind` / `tagObjectKind` when
  `release-provenance.json` is concrete;
- annotated tags expose `annotatedTagObjectId`;
- all tag types expose peeled commit sha for source commit matching.

The coverage names are intentionally split:

```text
live-github-release-provenance-matches-source-commit
synthetic-project-release-sidecar-rejected
```

This avoids the previous `published-release-facts-match-source-commit` wording,
which mixed live GitHub release authority with synthetic sidecar facts.

## V130-002 Commercial Backend Stable Contract

`V130-002` freezes the generic Paid Report commercial backend contract as a
machine-readable baseline:

```text
agentflow-commercial-backend-stable-contract.v1
```

The release-gate proof at `runtime/v130-commercial-backend-stable-contract.json`
must list every stable backend object, required field, optional/defaulted field,
status value, object version, stable error/decision state, and migration policy.

The stable object groups are:

```text
Product
Order
Entitlement
Run
Artifact
Evidence
Decision
Delivery
Feedback
```

The stable error / decision states are:

```text
invalid
deferred
blocked
accepted
revoked
expired
refunded
repair-needed
delivery-ready
```

After `v1.3.0`, backward-incompatible commercial backend contract changes require
an explicit migration or version bump and a release-gate proof update.

## V130-003 Paid Report Flow State Machine

`V130-003` defines the generic Paid Report backend lifecycle as a stable runtime
state machine:

```text
agentflow-paid-report-flow-state-machine.v1
```

The release-gate proof at `runtime/v130-paid-report-flow-state-machine.json`
must include:

- all lifecycle states from `draft-order` through `closed`;
- positive transition fixtures for the order, entitlement, run, artifact,
  evidence, decision, delivery and feedback chain;
- negative transition fixtures for illegal shortcuts such as
  `draft-order -> accepted` and `artifact-ready -> delivery-ready`;
- machine-readable failure reasons for every invalid transition;
- explicit authority flags proving invalid transitions cannot write accepted or
  delivery-ready authority.

The stable state list is:

```text
draft-order
order-ready
authorized
admitted
running
artifact-ready
evidence-complete
accepted
delivery-ready
feedback-needed
repair-requested
rerun-needs-authorization
refunded
expired
closed
```

## V130-004 Commercial Authority Boundary Freeze

`V130-004` freezes the writable authority map for generic Paid Report backend
objects:

```text
agentflow-commercial-authority-boundary.v1
```

The release-gate proof at `runtime/v130-commercial-authority-boundary.json`
must include:

- writable authority rules for Order, Entitlement, Run Admission, Run,
  Artifact, Evidence, Decision, Access Receipt and Commercial Policy;
- read-only projection rules for Delivery Package Projection, Customer Delivery
  Access Projection and Feedback Loop Projection;
- read-only surface list for Projection, Customer View, Download View,
  Synthetic Release Fixture and Release Sidecar;
- negative fixtures proving those read-only surfaces cannot write authority;
- negative fixtures proving synthetic release sidecars cannot satisfy live
  release authority.

The read-only surfaces are:

```text
Projection
Customer View
Download View
Synthetic Release Fixture
Release Sidecar
```
