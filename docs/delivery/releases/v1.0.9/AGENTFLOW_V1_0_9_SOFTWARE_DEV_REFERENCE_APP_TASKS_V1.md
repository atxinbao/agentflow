# AgentFlow v1.0.9 Software Dev Reference App Tasks

更新日期：2026-07-01
执行者：Codex

## Release

`v1.0.9` certifies Software Dev as the first-party Reference App over the stable Core Projection Kernel.

## Task Traceability

| Task | GitHub issue | Title | Status | Primary proof |
| --- | --- | --- | --- | --- |
| `V109-001` | `#734` | Release Task / GitHub Issue Traceability Gate | 状态：done | `runtime/v109-task-issue-traceability.json` |
| `V109-002` | `#735` | Quick Audit Pack Projection Primary Proof Inclusion | 状态：done | `quick-audit-manifest.json`, `pack-projection-readiness.json` |
| `V109-003` | `#736` | products/software-dev Reference App Contract | 状态：done | `runtime/v109-software-dev-product-contract.json` |
| `V109-004` | `#737` | Software Dev Spec Bundle to Task Flow | 状态：done | `runtime/v109-spec-task-flow.json` |
| `V109-005` | `#738` | Software Dev Connector Handoff and Runtime Command Baseline | 状态：done | `runtime/v109-connector-handoff.json` |
| `V109-006` | `#739` | Software Dev Evidence / Decision / Delivery Closed Loop | 状态：done | `runtime/v109-evidence-decision-delivery.json` |
| `V109-007` | `#740` | Software Dev Projection Workbench Read Models | 状态：done | `runtime/v109-workbench-read-models.json` |
| `V109-008` | `#741` | Core/Product Pack-backed Mapping Boundary Cleanup | 状态：done | `runtime/v109-mapping-boundary.json` |
| `V109-009` | `#742` | End-to-End Reference App Scenario and Core Boundary Negative Fixtures | 状态：done | `runtime/v109-golden-scenario.json` |
| `V109-010` | `#743` | v1.0.9 Reference App Boundary Release Certification | 状态：done | `runtime/v109-release-certification.json` |

## Dependency Order

```text
V109-001
-> V109-002
-> V109-003
-> V109-004
-> V109-005
-> V109-006
-> V109-007
-> V109-008
-> V109-009
-> V109-010
```

## Certified Source Boundary

```text
products/software-dev/**
```

is the first-party Reference App source boundary.

```text
crates/pack/fixtures/packs/software-dev/**
```

remains a fixture mirror only.

## Authority Rules

- GitHub issues are planning mirrors, not AgentFlow authority.
- PRs, release notes and provider transcripts are evidence inputs only.
- Core crates own generic concepts only.
- Software Dev terms belong under `products/software-dev/**`.
- Projection and UI cannot write authority.
- Audit is an optional sidecar, not the default Done blocker.

## Release Gate Binding

The v1.0.9 release gate must verify:

- exact task / issue title alignment for `#734` through `#743`;
- quick-audit inclusion of `pack-projection-readiness.json`;
- product source existence under `products/software-dev/**`;
- Core pollution negative fixtures;
- Software Dev golden scenario;
- v1.0.9 version metadata and release documentation.
