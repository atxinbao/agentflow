# AgentFlow v1.1.1 Product Contract Data-driven Hardening

更新日期：2026-07-01
执行者：Codex

## Status

`v1.1.1` 是 `v1.1.0` Product Surface hardening 之后的 Product Contract Data-driven hardening release baseline。

本版本确认：

```text
products/software-dev/surface/definition.json
= command mapping source

Product command mapping
-> Product-to-Pack bridge
-> Runtime resolver
-> Projection read model
-> release-gate proof artifacts
```

## Scope

`v1.1.1` 收口以下内容：

1. Product command mapping schema。
2. Product-to-Pack bridge data-driven conversion。
3. Runtime resolver data-driven route / skill / connector / capability resolution。
4. Projection Product read model data-driven conversion。
5. Product bridge crate pollution gate。
6. Runtime and Projection proof artifacts。
7. Synthetic second Product fixture。
8. v1.1.1 release certification。

## Public Records

- [AGENTFLOW_V1_1_1_PRODUCT_CONTRACT_DATA_DRIVEN_TASKS_V1.md](AGENTFLOW_V1_1_1_PRODUCT_CONTRACT_DATA_DRIVEN_TASKS_V1.md)
- [../v1.1.0/README.md](../v1.1.0/README.md)
- [../../../../products/software-dev/surface/definition.json](../../../../products/software-dev/surface/definition.json)
- [../../../../products/_fixtures/synthetic-review/product.toml](../../../../products/_fixtures/synthetic-review/product.toml)

## Release Gate Artifacts

`v1.1.1` release gate must produce:

```text
runtime/v111-product-schema-command-mapping.json
runtime/v111-product-to-pack-data-driven-bridge.json
runtime/v111-runtime-data-driven-product-resolver.json
runtime/v111-projection-data-driven-product-readmodel.json
runtime/v111-product-bridge-pollution-gate.json
runtime/v111-runtime-projection-proof-artifacts.json
runtime/v111-synthetic-second-product-fixture.json
runtime/v111-release-certification.json
quick-audit-manifest.json
```

## Authority Rules

- Product source declares command mapping fields.
- Product source remains read-only input and cannot write Runtime authority.
- Runtime resolver consumes Product source mapping and returns source refs.
- Projection converts Product source read models without Software Dev-only hardcoding.
- Synthetic fixture proves the path is generic and not tied to Software Dev command names.

## Known Boundaries

- UI command buttons remain follow-up work.
- Synthetic second Product is a test fixture only.
- Product installer and marketplace behavior remain out of scope.

## Next Version

`v1.1.x` can continue with Desktop command route installation and richer multi-product console behavior after this data-driven contract baseline is certified.
