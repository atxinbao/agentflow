# AgentFlow v1.1.1 Product Contract Data-driven Tasks

更新日期：2026-07-01
执行者：Codex

## Release

`v1.1.1` hardens Product command contracts so Product source data, not Rust hardcoding, drives bridge / runtime / projection behavior.

## Task Traceability

| Task | GitHub issue | Title | Status | Primary proof |
| --- | --- | --- | --- | --- |
| `V111-001` | `#757` | Product Schema Command Mapping Contract | 状态：done | `runtime/v111-product-schema-command-mapping.json` |
| `V111-002` | `#758` | Product-to-Pack Bridge Data-driven Conversion | 状态：done | `runtime/v111-product-to-pack-data-driven-bridge.json` |
| `V111-003` | `#759` | Runtime Product Resolver Data-driven Capability | 状态：done | `runtime/v111-runtime-data-driven-product-resolver.json` |
| `V111-004` | `#760` | Projection Product Read Model Data-driven Conversion | 状态：done | `runtime/v111-projection-data-driven-product-readmodel.json` |
| `V111-005` | `#761` | Core Pollution Gate Covers Product Bridge Crates | 状态：done | `runtime/v111-product-bridge-pollution-gate.json` |
| `V111-006` | `#762` | Actual Runtime and Projection Proof Artifacts | 状态：done | `runtime/v111-runtime-projection-proof-artifacts.json` |
| `V111-007` | `#763` | Synthetic Second Product Fixture | 状态：done | `runtime/v111-synthetic-second-product-fixture.json` |
| `V111-008` | `#764` | v1.1.1 Release Certification | 状态：done | `runtime/v111-release-certification.json` |

## Dependency Order

```text
V111-001
-> V111-002
-> V111-003
-> V111-004
-> V111-005
-> V111-006
-> V111-007
-> V111-008
```

## Certified Source Boundary

```text
products/software-dev/**
products/_fixtures/synthetic-review/**
```

`products/software-dev/**` is the first-party Software Dev Product source. `products/_fixtures/synthetic-review/**` is a test-only Product fixture and must not become a first-party Product registry entry.

## Release Gate Binding

The v1.1.1 release gate must verify:

- Product command mappings include command, runtime, action contract, target object, page, skill, connector, capability, evidence policy and acceptance policy refs;
- Product-to-Pack route conversion reads those fields from Product source;
- Runtime resolver returns Product source refs and does not infer page or capability from Software Dev command names;
- Projection converts Product domain / surface / connector / read model data from Product source;
- Product bridge crates are scanned for Software Dev-only helper pollution;
- runtime and projection proof artifacts include positive and negative examples;
- a synthetic second Product fixture proves generic behavior;
- v1.1.1 version metadata and release documentation are aligned.
