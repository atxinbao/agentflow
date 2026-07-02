# AgentFlow v1.1.2 Product Execution Proof and Command Surface Tasks

更新日期：2026-07-02
执行者：Codex

## Release

`v1.1.2` proves Product contracts drive real Runtime / Projection outputs and installs Product command routes into Desktop / Console surfaces.

## Task Traceability

| Task | GitHub issue | Title | Status | Primary proof |
| --- | --- | --- | --- | --- |
| `V112-001` | `#766` | Real Product Runtime Proof Harness | 状态：done | `runtime/v112-real-product-runtime-proof.json` |
| `V112-002` | `#767` | Real Projection Read Model Proof Harness | 状态：done | `runtime/v112-real-product-projection-proof.json` |
| `V112-003` | `#768` | Recursive Product Bridge Pollution Scanner | 状态：done | `runtime/v112-product-bridge-pollution-scan.json` |
| `V112-004` | `#769` | Registry-discovered Second Product Fixture | 状态：done | `runtime/v112-registry-discovered-second-product.json` |
| `V112-005` | `#770` | Desktop Product Command Route Read Model | 状态：done | `runtime/v112-desktop-product-command-route-read-model.json` |
| `V112-006` | `#771` | Desktop Command Button Installation | 状态：done | `runtime/v112-desktop-command-button-installation.json` |
| `V112-007` | `#772` | Multi-product Console Invalid / Deferred States | 状态：done | `runtime/v112-multi-product-console-states.json` |
| `V112-008` | `#773` | v1.1.2 Release Certification | 状态：done | `runtime/v112-release-certification.json` |

## Dependency Order

```text
V112-001
-> V112-002
-> V112-003
-> V112-004
-> V112-005
-> V112-006
-> V112-007
-> V112-008
```

## Certified Source Boundary

```text
products/software-dev/**
products/synthetic-review/**
crates/pack/**
crates/runtime-api/**
crates/projection/**
apps/desktop/**
scripts/verify_release_gate.sh
.github/workflows/release-gate.yml
```

## Release Gate Binding

The v1.1.2 release gate must verify:

- Runtime proof calls `validate_pack_command` and `dry_run_pack_command` for positive and negative Product commands;
- Projection proof calls `get_pack_industry_workbench_view` for Product read models;
- recursive scanner scans `crates/pack`, `crates/runtime-api` and `crates/projection`;
- second Product is discovered from `products/synthetic-review/product.toml`, not from `_fixtures`;
- Desktop loads Product command route read model through Tauri;
- Desktop command buttons invoke Product command dry-run before any runtime submission;
- multi-product console exposes valid, invalid and deferred command states;
- v1.1.2 version metadata, docs and release-gate primary proofs are aligned.
