# AgentFlow v1.2.0 Product Onboarding and First-run Experience Tasks

更新日期：2026-07-05
执行者：Codex

This document records the public delivery traceability for `v1.2.0`.

## Task Traceability

| Task | GitHub Issue | Title | Status | Primary proof |
| --- | --- | --- | --- | --- |
| V120-001 | #852 | Release Certification Top-level Metadata Fix | done | `runtime/v120-release-certification-top-level-metadata.json` |
| V120-002 | #853 | Certification Artifact Manifest Primary Proof Index | done | `runtime/v120-certification-artifact-manifest-primary-proof-index.json` |
| V120-003 | #854 | First-run Product Onboarding Contract | done | `runtime/v120-first-run-product-onboarding-contract.json` |
| V120-004 | #855 | Product Selection and Workspace Bootstrap | done | `runtime/v120-product-selection-workspace-bootstrap.json` |
| V120-005 | #856 | Workspace Readiness Preflight | done | `runtime/v120-workspace-readiness-preflight.json` |
| V120-006 | #857 | Provider / Connector / Skill Readiness Setup | done | `runtime/v120-provider-connector-skill-readiness.json` |
| V120-007 | #858 | Guided Sample Project Golden Run | done | `runtime/v120-guided-sample-project-golden-run.json` |
| V120-008 | #859 | Desktop First-run Onboarding Surface | done | `runtime/v120-desktop-first-run-onboarding-surface.json` |
| V120-009 | #860 | User-hidden .agentflow Boundary | done | `runtime/v120-user-hidden-agentflow-boundary.json` |
| V120-010 | #861 | v1.2.0 Release Certification | done | `runtime/v120-release-certification.json` |

## Dependency Order

```text
#852
-> #853
-> #854
-> #855
-> #856
-> #857
-> #858
-> #859
-> #860
-> #861
```

## Certified Boundary

`v1.2.0` keeps the Product and Runtime authority model from `v1.1.9` and adds first-run onboarding proof:

- Software Dev is the selected Product for the golden first-run path.
- Workspace bootstrap writes project docs and local Runtime facts through Runtime commands.
- Readiness preflight checks product definition, workspace projection, provider smoke, connector status and skill status.
- Desktop consumes Runtime-backed read models and commands.
- Normal users do not see `.agentflow/**` as the primary product surface.
- Diagnostic refs remain available for advanced inspection.
- Guided sample failure stays repairable/retry and does not silently pass.

## Release Gate Artifacts

The release certification requires the following files:

```text
runtime/v120-release-certification-top-level-metadata.json
runtime/v120-certification-artifact-manifest-primary-proof-index.json
runtime/v120-first-run-product-onboarding-contract.json
runtime/v120-product-selection-workspace-bootstrap.json
runtime/v120-workspace-readiness-preflight.json
runtime/v120-provider-connector-skill-readiness.json
runtime/v120-guided-sample-project-golden-run.json
runtime/v120-desktop-first-run-onboarding-surface.json
runtime/v120-user-hidden-agentflow-boundary.json
runtime/v120-release-certification.json
```
