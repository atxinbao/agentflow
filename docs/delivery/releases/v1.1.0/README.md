# AgentFlow v1.1.0 Product Surface Hardening

更新日期：2026-07-01
执行者：Codex

## Status

`v1.1.0` 是 `v1.0.9` Software Dev Reference App boundary certification 之后的 Product Surface hardening release baseline。

本版本确认：

```text
products/software-dev/**        = first-party Software Dev Product source
crates/pack/src/product.rs      = Product Registry loader and product-to-pack bridge
crates/runtime-api/**           = Product source first command route resolver
crates/projection/**            = Product source read model projection
crates/pack/fixtures/**         = fixture mirror only
crates/**                       = Core OS Runtime, not Software Dev authority
```

## Scope

`v1.1.0` 收口以下内容：

1. Roadmap and release goal alignment。
2. `products/**` Product Registry loader。
3. Software Dev Product manifest to Pack contract bridge。
4. Runtime API Product / Pack Registry command resolution。
5. Projection Product source read model generation without built-in fallback。
6. Core pollution detection release gate。
7. Product command route installation。
8. Software Dev end-to-end Product Surface scenario。
9. Quick audit Product source primary proofs。
10. v1.1.0 Release Certification。

## Public Records

- [AGENTFLOW_V1_1_0_PRODUCT_SURFACE_HARDENING_TASKS_V1.md](AGENTFLOW_V1_1_0_PRODUCT_SURFACE_HARDENING_TASKS_V1.md)
- [../../../architecture/086-industry-product-source-boundary-v1.md](../../../architecture/086-industry-product-source-boundary-v1.md)
- [../v1.0.9/README.md](../v1.0.9/README.md)
- [../../../../products/software-dev/product.toml](../../../../products/software-dev/product.toml)

## Release Gate Artifacts

`v1.1.0` release gate must produce:

```text
runtime/v110-roadmap-release-goal-alignment.json
runtime/v110-product-registry-loader.json
runtime/v110-product-to-pack-contract.json
runtime/v110-runtime-product-command-routes.json
runtime/v110-projection-product-source.json
runtime/v110-core-pollution-detection.json
runtime/v110-product-command-route-installation.json
runtime/v110-software-dev-e2e-product-surface.json
runtime/v110-quick-audit-product-source-proofs.json
runtime/v110-release-certification.json
quick-audit-manifest.json
```

## Known Boundaries

- `products/**` is Product source, not Runtime authority.
- `.agentflow/**` remains runtime fact output for user projects.
- Product Registry reads source definitions and does not write project facts.
- Runtime API may route commands from Product source, but action contracts remain Core / Pack owned.
- Projection is read-only and must not inject built-in Software Dev fallback when product / pack sources are missing.
- GitHub issues remain planning mirrors, not AgentFlow authority.

## Non-goals

- 不发布 Software Dev commercial beta；
- 不实现 Product installer；
- 不新增行业壳营销页面；
- 不把 Software Dev terms 写进 Core authority；
- 不把 fixture mirror 提升为正式 Product source；
- 不绕过 confirmed Spec Bundle 和 `.agentflow/spec/**` 执行合同。

## Known Risks

- Product Surface follow-up hardening remains open for richer installation and UI command route handling.
- Software Dev remains the first Product source proof, not a general marketplace system.
- Quick-audit package is intentionally small; full release gate artifacts remain separate.

## Next Version

`v1.1.x` should continue Product Surface hardening around installation, console command routes, UI integration and multi-product projection behavior.
