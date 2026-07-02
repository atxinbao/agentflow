# AgentFlow v1.1.3 Product Command Submission and State Semantics

更新日期：2026-07-02
执行者：Codex

## Status

`v1.1.3` 是 Product Command Submission and State Semantics release baseline。

本版本确认：

```text
Product Command Surface
-> explicit command state
-> confirm-then-submit
-> Runtime API submit
-> governance / arbitration
-> evidence handoff
-> release-gate proof artifacts
```

## Scope

`v1.1.3` 收口以下内容：

1. Product Command State Contract。
2. Product Command Submission Contract。
3. Runtime Product Command Submit API。
4. Desktop confirm-then-submit command flow。
5. Product Command Evidence Handoff。
6. Multi-product state UI proof。
7. Semantic Product bridge pollution scanner。
8. v1.1.3 release certification。

## Public Records

- [AGENTFLOW_V1_1_3_PRODUCT_COMMAND_SUBMISSION_TASKS_V1.md](AGENTFLOW_V1_1_3_PRODUCT_COMMAND_SUBMISSION_TASKS_V1.md)
- [../v1.1.2/README.md](../v1.1.2/README.md)
- [../../../../products/software-dev/product.toml](../../../../products/software-dev/product.toml)
- [../../../../products/synthetic-review/product.toml](../../../../products/synthetic-review/product.toml)

## Release Gate Artifacts

`v1.1.3` release gate must produce:

```text
runtime/v113-product-command-state-contract.json
runtime/v113-product-command-submit-contract.json
runtime/v113-runtime-product-command-submit-api.json
runtime/v113-desktop-confirm-submit-command-flow.json
runtime/v113-product-command-evidence-handoff.json
runtime/v113-multi-product-state-ui-proof.json
runtime/v113-semantic-product-bridge-pollution-scan.json
runtime/v113-release-certification.json
quick-audit-manifest.json
```

## Authority Rules

- Product source remains `products/**`.
- Product Command Surface read models must expose machine-readable states: `valid`, `invalid`, `deferred`, `unavailable`, `rejected`, `submitted`.
- Desktop can submit a Product Command only after dry-run / validation receipt confirmation.
- Runtime submit must pass governance, arbitration, event-store taxonomy and evidence handoff.
- Product bridge pollution scanner must fail hardcoded command bridge helpers that bypass Product source definitions.

## Known Boundaries

- This release enables controlled Product command submit, not marketplace installation.
- Desktop submit remains local Runtime API submit; provider launch and external Product distribution remain out of scope.
- Synthetic Review is a certified reference Product for bridge hardening, not a user-facing product shell.

## Next Version

`v1.1.4` can continue toward Project Creation and Product Workspace after this submit baseline is certified.
