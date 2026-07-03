# AgentFlow v1.1.4 Project Creation and Product Workspace

更新日期：2026-07-02
执行者：Codex

## Status

`v1.1.4` 是 Project Creation and Product Workspace release baseline。

本版本确认：

```text
Product source selection
-> dry-run receipt binding
-> confirmed submit
-> Product workspace creation
-> standard docs and .agentflow fact roots
-> active workspace projection
-> failure / duplicate / recovery handling
-> release-gate proof artifacts
```

## Scope

`v1.1.4` 收口以下内容：

1. Product Submit Receipt Binding。
2. Desktop Confirm-submit Interaction Proof。
3. Product Bridge Semantic Pollution Scanner。
4. Project Workspace Creation Contract。
5. Product-selected Workspace Bootstrap。
6. Standard Docs and AgentFlow Fact Source Initialization。
7. Active Product Workspace State and Projection。
8. Workspace Init Failure / Duplicate / Recovery。
9. Software Dev Default Workspace Golden Path。
10. v1.1.4 Release Certification。

## Public Records

- [AGENTFLOW_V1_1_4_PROJECT_CREATION_PRODUCT_WORKSPACE_TASKS_V1.md](AGENTFLOW_V1_1_4_PROJECT_CREATION_PRODUCT_WORKSPACE_TASKS_V1.md)
- [../v1.1.3/README.md](../v1.1.3/README.md)
- [../../../../products/software-dev/product.toml](../../../../products/software-dev/product.toml)
- [../../../../products/synthetic-review/product.toml](../../../../products/synthetic-review/product.toml)

## Release Gate Artifacts

`v1.1.4` release gate must produce:

```text
runtime/v114-product-submit-receipt-binding.json
runtime/v114-desktop-confirm-submit-interaction.json
runtime/v114-product-bridge-semantic-pollution-scan.json
runtime/v114-project-workspace-creation-contract.json
runtime/v114-product-selected-workspace-bootstrap.json
runtime/v114-standard-docs-agentflow-fact-source-init.json
runtime/v114-active-product-workspace-projection.json
runtime/v114-workspace-init-failure-recovery.json
runtime/v114-software-dev-workspace-golden-path.json
runtime/v114-release-certification.json
quick-audit-manifest.json
```

## Authority Rules

- Product source remains `products/**`.
- Project workspace creation must bind to selected Product source, not hardcoded Software Dev constants.
- Desktop submit must reuse the exact dry-run receipt issued for the same Product command target and input.
- Workspace initialization must create public `docs/project/**` records and local `.agentflow/spec/**`, `.agentflow/events/**`, `.agentflow/tasks/**` fact roots.
- Projection reads active Product workspace state; it does not write authority facts.
- Duplicate, partial and invalid workspace states must be machine-readable and recoverable.

## Known Boundaries

- This release creates local Product-backed workspaces; it does not introduce marketplace install or remote Product distribution.
- Desktop Product command submit remains local Runtime API submit.
- Software Dev is the certified default Product workspace golden path; Product-specific behavior remains outside Core bridge code.

## Next Version

`v1.1.5` continues toward Spec Intake to Goal / Roadmap / Task Productization after this creation baseline is certified.
