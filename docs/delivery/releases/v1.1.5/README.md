# AgentFlow v1.1.5 Spec Intake to Goal / Roadmap / Task Productization

更新日期：2026-07-03
执行者：Codex

## Status

`v1.1.5` 是 Spec Intake to Goal / Roadmap / Task Productization release baseline。

本版本确认：

```text
Raw Human Request
-> Product Intent Intake
-> Core Route Decision
-> Preview Artifact
-> Confirmation Gate
-> Requirement Authority
-> .agentflow/spec project / issue authority
-> Runtime Action Proposal readiness
```

## Scope

`v1.1.5` 收口以下内容：

1. Next Release Planning Alignment。
2. Product Workspace Desktop Entry Bridge。
3. Portable Workspace Receipt / Projection Paths。
4. Intent Intake Contract。
5. Core Route Policy。
6. Spec Bundle to Goal / Roadmap / Task Derivation。
7. Confirmation Gate and Authority Write Boundary。
8. Spec Materializer to docs / `.agentflow`。
9. Software Dev Spec-to-Tasks Golden Path。
10. v1.1.5 Release Certification。

## Public Records

- [AGENTFLOW_V1_1_5_SPEC_INTAKE_PRODUCTIZATION_TASKS_V1.md](AGENTFLOW_V1_1_5_SPEC_INTAKE_PRODUCTIZATION_TASKS_V1.md)
- [../v1.1.4/README.md](../v1.1.4/README.md)
- [../../../../docs/project/roadmap.md](../../../../docs/project/roadmap.md)
- [../../../../products/software-dev/product.toml](../../../../products/software-dev/product.toml)

## Release Gate Artifacts

`v1.1.5` release gate must produce:

```text
runtime/v115-next-release-planning-alignment.json
runtime/v115-product-workspace-desktop-entry-bridge.json
runtime/v115-portable-workspace-receipt-projection-paths.json
runtime/v115-intent-intake-contract.json
runtime/v115-core-route-policy.json
runtime/v115-spec-bundle-goal-roadmap-task-derivation.json
runtime/v115-confirmation-gate-authority-boundary.json
runtime/v115-spec-materializer-docs-agentflow-authority.json
runtime/v115-software-dev-spec-to-tasks-golden-path.json
runtime/v115-release-certification.json
quick-audit-manifest.json
```

## Authority Rules

- `docs/project/roadmap.md` is the release planning authority for `v1.1.5`.
- Raw user input is preserved in the Product Intent Intake envelope.
- `clarify` and `research` routes cannot write authority.
- Preview artifacts are traceable and machine-readable, but not authority.
- Confirmation must bind to the preview id and preview hash.
- Materialization can only run from an accepted, unmodified preview.
- Confirmed authority writes land in public `docs/requirements/**` and local `.agentflow/spec/**`.
- Product-specific Software Dev wording remains Product mapping, not Core Runtime policy.

## Known Boundaries

- This release does not start Build Agent execution from Spec Intake output.
- This release does not create provider launch sessions.
- This release does not move Product workspace lifecycle beyond creation and projection.

## Next Version

`v1.1.6` can continue toward Executor Adapter Real Execution Closure after this Spec Intake baseline is certified.
