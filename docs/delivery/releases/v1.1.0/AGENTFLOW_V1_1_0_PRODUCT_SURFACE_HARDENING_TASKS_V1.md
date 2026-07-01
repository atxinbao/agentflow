# AgentFlow v1.1.0 Product Surface Hardening Tasks

更新日期：2026-07-01
执行者：Codex

## Release

`v1.1.0` hardens the first-party Software Dev Product Surface over the certified `v1.0.9` Reference App boundary.

## Task Traceability

| Task | GitHub issue | Title | Status | Primary proof |
| --- | --- | --- | --- | --- |
| `V110-001` | `#746` | Roadmap and Release Goal Alignment | 状态：done | `runtime/v110-roadmap-release-goal-alignment.json` |
| `V110-002` | `#747` | Product Registry Loader for `products/**` | 状态：done | `runtime/v110-product-registry-loader.json` |
| `V110-003` | `#748` | Software Dev Product Manifest to Pack Contract Bridge | 状态：done | `runtime/v110-product-to-pack-contract.json` |
| `V110-004` | `#749` | Runtime API Uses Product / Pack Registry | 状态：done | `runtime/v110-runtime-product-command-routes.json` |
| `V110-005` | `#750` | Projection Uses Product Source, No Built-in Fallback | 状态：done | `runtime/v110-projection-product-source.json` |
| `V110-006` | `#751` | Core Pollution Detection Release Gate | 状态：done | `runtime/v110-core-pollution-detection.json` |
| `V110-007` | `#752` | Product Command Route Installation | 状态：done | `runtime/v110-product-command-route-installation.json` |
| `V110-008` | `#753` | Software Dev End-to-End Product Surface Scenario | 状态：done | `runtime/v110-software-dev-e2e-product-surface.json` |
| `V110-009` | `#754` | Quick Audit Product Source Primary Proofs | 状态：done | `runtime/v110-quick-audit-product-source-proofs.json` |
| `V110-010` | `#755` | v1.1.0 Release Certification | 状态：done | `runtime/v110-release-certification.json` |

## Dependency Order

```text
V110-001
-> V110-002
-> V110-003
-> V110-004
-> V110-005
-> V110-006
-> V110-007
-> V110-008
-> V110-009
-> V110-010
```

## Certified Source Boundary

```text
products/software-dev/**
```

is the first-party Software Dev Product Surface source boundary.

```text
crates/pack/fixtures/packs/software-dev/**
```

remains a fixture mirror only.

## Authority Rules

- `products/**` defines Product source metadata.
- Product Registry and Projection are read-only over Product source.
- Runtime API can resolve Product commands, but cannot make Product source a task authority.
- Core crates cannot define Software Dev product-specific authority.
- GitHub issues are planning mirrors, not AgentFlow authority.
- Quick-audit artifacts must include Product source primary proofs.

## Release Gate Binding

The v1.1.0 release gate must verify:

- exact task / issue title alignment for `#746` through `#755`;
- Product Registry can read `products/software-dev/product.toml` and all declared entrypoints;
- Product-to-Pack command bridge maps Software Dev commands to Runtime action contracts;
- Runtime API resolves Product source routes before explicit pack registry fallback;
- Projection read models use Product source and show invalid / deferred state when Product / Pack sources are missing;
- Core pollution detection rejects Software Dev authority inside Core crates;
- quick-audit includes Product source primary proof artifacts;
- v1.1.0 version metadata and release documentation are aligned.
