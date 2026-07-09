# AgentFlow v1.3.0 Commercial Backend Stable Closure Tasks

更新日期：2026-07-10
执行者：Codex

This document records the planned public delivery traceability for `v1.3.0`.

## Task Traceability

| Task | GitHub Issue | Title | Status | Primary proof |
| --- | --- | --- | --- | --- |
| V130-001 | #993 | v1.2.9 Release Audit and Certification Semantics Repair | in progress | `proofs/v130-001-v129-release-audit-facts.json` |
| V130-002 | #994 | Commercial Backend Stable Contract | planned | TBD |
| V130-003 | #995 | Paid Report Flow State Machine | planned | TBD |
| V130-004 | #996 | Commercial Authority Boundary Freeze | planned | TBD |
| V130-005 | #997 | Product SKU Extension Contract | planned | TBD |
| V130-006 | #998 | Provider / Generator Adapter Boundary | planned | TBD |
| V130-007 | #999 | Payment Provider Adapter Boundary | planned | TBD |
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

