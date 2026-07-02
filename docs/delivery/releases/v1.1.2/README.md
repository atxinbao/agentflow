# AgentFlow v1.1.2 Product Execution Proof and Command Surface Hardening

更新日期：2026-07-02
执行者：Codex

## Status

`v1.1.2` 是 `v1.1.1` Product Contract Data-driven hardening 之后的 Product Execution Proof and Command Surface hardening release baseline。

本版本确认：

```text
products/**
-> Product Registry
-> Runtime API validate / dry-run
-> Projection read model
-> Desktop Product Command Surface
-> release-gate proof artifacts
```

## Scope

`v1.1.2` 收口以下内容：

1. 真实 Product Runtime proof harness。
2. 真实 Product Projection read model proof harness。
3. 递归 Product bridge pollution scanner。
4. `products/synthetic-review/**` 作为直接 registry-discovered second Product。
5. Desktop Product command route read model。
6. Desktop Product command button dry-run installation。
7. Multi-product console valid / invalid / deferred state proof。
8. v1.1.2 release certification。

## Public Records

- [AGENTFLOW_V1_1_2_PRODUCT_EXECUTION_PROOF_COMMAND_SURFACE_TASKS_V1.md](AGENTFLOW_V1_1_2_PRODUCT_EXECUTION_PROOF_COMMAND_SURFACE_TASKS_V1.md)
- [../v1.1.1/README.md](../v1.1.1/README.md)
- [../../../../products/software-dev/product.toml](../../../../products/software-dev/product.toml)
- [../../../../products/synthetic-review/product.toml](../../../../products/synthetic-review/product.toml)

## Release Gate Artifacts

`v1.1.2` release gate must produce:

```text
runtime/v112-real-product-runtime-proof.json
runtime/v112-real-product-projection-proof.json
runtime/v112-product-bridge-pollution-scan.json
runtime/v112-registry-discovered-second-product.json
runtime/v112-desktop-product-command-route-read-model.json
runtime/v112-desktop-command-button-installation.json
runtime/v112-multi-product-console-states.json
runtime/v112-release-certification.json
quick-audit-manifest.json
```

## Authority Rules

- Product source remains `products/**`.
- Runtime proof must call Runtime API validation and dry-run, not hand-assemble JSON.
- Projection proof must call Projection API read models.
- Desktop Product Command Surface is read-only and can only dry-run commands before runtime submission.
- Product bridge pollution scanner must recursively scan bridge crates and fail on hardcoded product-route helpers.
- Multi-product console must expose valid, invalid and deferred states without falling back to Software Dev.

## Known Boundaries

- This release installs read-only / dry-run Desktop command route behavior, not authority-writing command submission.
- Product marketplace, installer and external distribution remain out of scope.
- Synthetic Review is a certified reference Product for bridge hardening, not a user-facing product shell.

## Next Version

`v1.1.x` can continue toward Product command submission, richer Product console UX and marketplace-ready installation after this proof baseline is certified.
