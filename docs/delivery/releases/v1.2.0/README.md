# AgentFlow v1.2.0 Product Onboarding and First-run Experience

更新日期：2026-07-05
执行者：Codex

## Release Baseline

`v1.2.0` 是 Product Onboarding and First-run Experience release baseline。

这一版把 `v1.1.9` 的 Software Dev reference app beta baseline 推进到用户第一次打开产品时可理解、可诊断、可复跑的首轮引导：

```text
Choose Product
-> Bootstrap Workspace
-> Readiness Preflight
-> Provider / Connector / Skill Setup
-> Guided Sample Run
-> Delivery Summary
```

## Scope

`v1.2.0` 收口以下内容：

1. Release certification top-level metadata fix。
2. Certification artifact manifest primary proof index。
3. First-run product onboarding contract。
4. Product selection and workspace bootstrap。
5. Workspace readiness preflight。
6. Provider / Connector / Skill readiness setup。
7. Guided sample project golden run。
8. Desktop first-run onboarding surface。
9. User-hidden `.agentflow` boundary。
10. v1.2.0 release certification。

## Release Gate Artifacts

`v1.2.0` release gate must produce:

- `runtime/v120-release-certification-top-level-metadata.json`
- `runtime/v120-certification-artifact-manifest-primary-proof-index.json`
- `runtime/v120-first-run-product-onboarding-contract.json`
- `runtime/v120-product-selection-workspace-bootstrap.json`
- `runtime/v120-workspace-readiness-preflight.json`
- `runtime/v120-provider-connector-skill-readiness.json`
- `runtime/v120-guided-sample-project-golden-run.json`
- `runtime/v120-desktop-first-run-onboarding-surface.json`
- `runtime/v120-user-hidden-agentflow-boundary.json`
- `runtime/v120-release-certification.json`

## Certification Metadata

The v1.2.0 certification record must expose release-gate metadata at the top level:

```text
releaseVersion
releaseTag
sourceCommit
workflowRunId
artifactNames
primaryProofs
```

The artifact manifest also carries a primary proof index with path, sha256, byte size and proof role for each proof.

## Product Onboarding Boundary

`v1.2.0` certifies the Software Dev first-run onboarding path.

It is:

- a product onboarding baseline for selecting Software Dev Reference App;
- a Runtime-backed workspace bootstrap and readiness preflight;
- a proof that provider, connector and skill names alone do not count as readiness;
- a Desktop first-run surface that hides `.agentflow` internals from normal users while keeping diagnostics available in advanced surfaces.

It is not public commercial launch and does not add another industry product.

## GitHub Traceability

Task traceability is recorded in:

- [AGENTFLOW_V1_2_0_PRODUCT_ONBOARDING_FIRST_RUN_TASKS_V1.md](AGENTFLOW_V1_2_0_PRODUCT_ONBOARDING_FIRST_RUN_TASKS_V1.md)

## Authority Rules

- Product selection starts from Product source and Runtime commands.
- Desktop shows read models and command results, not raw authority files.
- `.agentflow/**` remains hidden from normal first-run UX.
- Advanced diagnostics may expose local diagnostic refs for debugging.
- Provider / Connector / Skill readiness requires smoke or status evidence.
- Guided sample failure must remain repairable/retry and must not silently become Done.

## Non-goals

- This release does not certify public commercial launch.
- This release does not make GitHub issues task authority.
- This release does not add cloud onboarding.
- This release does not introduce a new industry product.
- This release does not make Audit a default blocker.

## Next Version

The next release can build on this first-run baseline to improve product console continuity after onboarding.
