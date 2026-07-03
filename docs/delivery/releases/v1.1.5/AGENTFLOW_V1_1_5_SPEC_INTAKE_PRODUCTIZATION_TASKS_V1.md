# AgentFlow v1.1.5 Spec Intake Productization Tasks

更新日期：2026-07-03
执行者：Codex

## Purpose

This document records the public delivery traceability for `v1.1.5`.

`v1.1.5` turns Product-level human input into a preview-first Spec Intake flow. It keeps raw intent intact, routes the request through Core route semantics, derives Goal / Roadmap / Task previews, requires confirmation bound to a preview hash, and materializes only confirmed outputs into `docs/requirements/**` and `.agentflow/spec/**`.

## Task Traceability

| Task ID | GitHub Issue | Scope | Status | Primary Proof |
| --- | --- | --- | --- | --- |
| V115-001 | #797 | Next Release Planning Alignment | done | `runtime/v115-next-release-planning-alignment.json` |
| V115-002 | #798 | Product Workspace Desktop Entry Bridge | done | `runtime/v115-product-workspace-desktop-entry-bridge.json` |
| V115-003 | #799 | Portable Workspace Receipt / Projection Paths | done | `runtime/v115-portable-workspace-receipt-projection-paths.json` |
| V115-004 | #800 | Intent Intake Contract | done | `runtime/v115-intent-intake-contract.json` |
| V115-005 | #801 | Core Route Policy | done | `runtime/v115-core-route-policy.json` |
| V115-006 | #802 | Spec Bundle to Goal / Roadmap / Task Derivation | done | `runtime/v115-spec-bundle-goal-roadmap-task-derivation.json` |
| V115-007 | #803 | Confirmation Gate and Authority Write Boundary | done | `runtime/v115-confirmation-gate-authority-boundary.json` |
| V115-008 | #804 | Spec Materializer to docs / `.agentflow` | done | `runtime/v115-spec-materializer-docs-agentflow-authority.json` |
| V115-009 | #805 | Software Dev Spec-to-Tasks Golden Path | done | `runtime/v115-software-dev-spec-to-tasks-golden-path.json` |
| V115-010 | #806 | v1.1.5 Release Certification | done | `runtime/v115-release-certification.json` |

## Acceptance

`v1.1.5` is accepted only when:

1. workspace and desktop versions are `1.1.5`;
2. `CHANGELOG.md` contains the `v1.1.5` entry;
3. `docs/delivery/README.md` points to `v1.1.5` as current baseline;
4. Product Workspace receipt/projection contains portable `workspace://` refs;
5. Product Intent Intake preserves raw input and source envelope fields;
6. Core route policy covers `clarify`, `research`, `define`, `plan`, `task`, `decide`, `deliver` and `evolve`;
7. preview artifacts are not authority and do not write `docs/requirements/**` or `.agentflow/spec/**`;
8. confirmation records bind to preview id and preview hash;
9. stale, rejected, revised or expired previews cannot materialize authority;
10. confirmed materialization writes public requirement docs and local `.agentflow/spec/**` contracts;
11. the release gate writes all `runtime/v115-*` proof artifacts;
12. all GitHub issues `#797` through `#806` are closed by the release PR.

## Non-goals

- Do not start Build Agent execution from the generated issues.
- Do not create provider launch sessions.
- Do not treat GitHub issues as Spec authority.
- Do not hardcode Software Dev behavior into Core route policy.
