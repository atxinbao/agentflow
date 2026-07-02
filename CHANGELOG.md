# Changelog

更新日期：2026-06-30
执行者：Codex

## Current Baseline

当前发布基线：

- [docs/delivery/releases/v1.1.3/README.md](docs/delivery/releases/v1.1.3/README.md)
- [docs/delivery/releases/v1.1.3/AGENTFLOW_V1_1_3_PRODUCT_COMMAND_SUBMISSION_TASKS_V1.md](docs/delivery/releases/v1.1.3/AGENTFLOW_V1_1_3_PRODUCT_COMMAND_SUBMISSION_TASKS_V1.md)

下一版计划：

- v1.1.4 Project Creation and Product Workspace。

## Unreleased

下一版保留给 Project Creation and Product Workspace，不在本次 release 中认证。

Planned v1.1.4 foundation:

- initialize Product-backed project workspaces;
- keep `products/**` as source definitions and `.agentflow/**` as runtime facts;
- preserve Runtime submit authority and projection proof boundaries.

## v1.1.3 - 2026-07-02

Product Command Submission and State Semantics:

- added explicit Product command states for `valid`, `invalid`, `deferred`, `unavailable`, `rejected` and `submitted`;
- added controlled Product command submit through Runtime API governance and arbitration;
- added confirm-then-submit Desktop Product command flow with pending confirmation state;
- added Product command submit receipts and evidence handoff records;
- added event-store taxonomy support for issue running lifecycle events emitted by accepted runtime actions;
- expanded release-gate proof artifacts with Product command state, submit, runtime, Desktop, evidence handoff, multi-product and semantic bridge pollution proofs;
- added v1.1.3 release task traceability for GitHub issues `#775` through `#782`;
- advanced workspace and desktop version metadata to `1.1.3`.

## v1.1.2 - 2026-07-02

Product Execution Proof and Command Surface hardening:

- added direct `products/synthetic-review/**` Product source so the registry discovers a second Product outside `_fixtures`;
- added Runtime API Product Command Surface read model that validates and dry-runs Product commands through existing Runtime APIs;
- added Desktop Tauri commands and Project Home command surface rendering for Product command routes and dry-run actions;
- added a real v1.1.2 runtime proof harness that calls `validate_pack_command` and `dry_run_pack_command` for positive and negative Product commands;
- added a real v1.1.2 projection proof harness that calls Product read models through Projection API;
- added recursive Product bridge pollution scanning for `crates/pack`, `crates/runtime-api` and `crates/projection`;
- expanded release-gate quick-audit primary proofs with v1.1.2 Product execution, projection, desktop and multi-product state artifacts;
- added v1.1.2 release task traceability for GitHub issues `#766` through `#773`;
- advanced workspace and desktop version metadata to `1.1.2`.

## v1.1.1 - 2026-07-01

Product Contract Data-driven hardening:

- added Product command mapping fields for command, runtime, action contract, target object, page, skill, connector, capability, evidence policy and acceptance policy refs;
- changed Product-to-Pack command routing to read mapping fields from Product source instead of hardcoded Software Dev command names;
- changed Runtime API Product resolver to use Product source page, skill, connector and capability refs;
- changed Projection Product conversion to read domain actions, acceptance semantics, evidence policy, command pages and connector supported actions from Product source;
- added a synthetic second Product fixture under `products/_fixtures/synthetic-review/**` to prove generic behavior outside Software Dev command names;
- expanded release-gate quick-audit primary proofs with v1.1.1 Runtime / Projection proof artifacts and Product bridge pollution checks;
- added v1.1.1 release task traceability for GitHub issues `#757` through `#764`;
- advanced workspace and desktop version metadata to `1.1.1`.

## v1.1.0 - 2026-07-01

Product Surface Hardening:

- added a read-only Product Registry loader for `products/<product-id>/product.toml` and all declared product entrypoints;
- mapped `products/software-dev/**` into the existing pack/runtime command surface without making fixture mirrors authoritative;
- made Runtime API command resolution product-source-first while preserving explicit `.agentflow/packs/**` custom pack support;
- made Projection read models consume product source definitions and expose invalid/deferred state when product/pack sources are missing instead of silently injecting built-in Software Dev fallback data;
- added Software Dev product route, product-to-pack contract and missing-source negative tests;
- added v1.1.0 release task traceability and certification artifacts for GitHub issues `#746` through `#755`;
- published the v1.1.0 release baseline at [docs/delivery/releases/v1.1.0/README.md](docs/delivery/releases/v1.1.0/README.md);
- advanced workspace and desktop version metadata to `1.1.0`.

## v1.0.9 - 2026-07-01

Software Dev Reference App Boundary Certification:

- added `products/software-dev/**` as the first-party Software Dev Reference App source boundary;
- kept `crates/pack/fixtures/packs/software-dev/**` as fixture mirror only;
- certified task-to-GitHub issue traceability for V109 planning mirrors without granting GitHub issue authority;
- included Pack projection primary proof in the quick-audit certification package;
- added product contract, Spec-to-task flow, connector handoff, evidence / decision / delivery, workbench projection and mapping-boundary release-gate artifacts;
- certified Software Dev golden scenario and negative authority fixtures for GitHub issue-only, provider transcript-only, PR-only, release-note-only, direct projection write, missing product mapping and audit default blocker cases;
- advanced workspace and desktop version metadata to `1.0.9`.

## v1.0.8 - 2026-06-30

Core Projection Kernel baseline:

- defined the Core Projection Kernel read-only contract over Spec, Event, Evidence and Decision authority facts;
- added deterministic event replay / projection rebuild reports with failure fixtures;
- stabilized Core read model schemas for spec, evidence, decision and delivery surfaces;
- stabilized view model contracts for industry app surfaces without direct authority reads;
- documented Pack-specific projection mapping boundaries and invalid / missing app definition fail-closed behavior;
- added feedback surface projection and projection freshness receipts so stale / incomplete surfaces route to Spec evolution preview without writing authority;
- added release-gate artifacts through `runtime/core-projection-kernel-contract.json`, `runtime/event-replay-projection-report.json`, `runtime/event-replay-projection-failure-report.json`, `runtime/core-read-model-schema.json`, `runtime/core-view-model-contract.json`, `runtime/projection-feedback-freshness-receipts.json`, `pack-projection-readiness.json`, and `runtime/v108-release-certification.json`;
- advanced workspace and desktop version metadata to `1.0.8`.

## v1.0.7 - 2026-06-29

Core Decision Kernel baseline:

- defined release provenance tag policy and v1.0.6 Evidence Kernel handoff;
- added Core Decision Model, Decision Input Binding, Outcome Transition Semantics and Failure Reason / Remediation contracts;
- connected Evidence-to-Decision Gate so missing / invalid / wrong evidence cannot become accepted-ready;
- protected Completion Commit authority so projection, provider session, delivery context and audit sidecar cannot write completion authority;
- defined Delivery Readiness and optional Audit Sidecar Trigger as a separate evaluation, not the default Done chain;
- added Decision Projection Read Model with negative fixtures for missing evidence, fake evidence, wrong state, projection-as-authority and audit-chain pollution;
- added release-gate artifacts through `runtime/core-decision-projection-read-model.json` and `runtime/v107-release-certification.json`;
- advanced workspace and desktop version metadata to `1.0.7`.

## v1.0.6 - 2026-06-29

Core Evidence Kernel baseline:

- defined Core Evidence Pack Schema, Source Type Registry, Capture Receipts, Authority Trace Links, Completeness Policy, Missing Evidence Handling, External Proof Provenance, Software Dev Reference Evidence Mapping, and Evidence Projection Read Model;
- certified fake / missing / wrong evidence negative fixture coverage through release-gate artifacts;
- kept Software Dev evidence as Reference App mapping only, not Core Evidence authority;
- added release-gate artifacts `core-evidence-pack-schema`, `core-evidence-source-type-registry`, `core-evidence-capture-receipts`, `core-evidence-authority-trace-links`, `core-evidence-completeness-policy`, `core-missing-evidence-handling`, `core-external-proof-provenance`, `software-dev-reference-evidence-mapping`, `evidence-projection-read-model`, and `v106-release-certification`;
- advanced workspace and desktop version metadata to `1.0.6`.

## v1.0.5 - 2026-06-28

Core Runtime Kernel baseline:

- connected Core Runtime command, Runtime Admission, Action Proposal, Arbitration, executor closeout and task / run state writeback to the Core Ontology Kernel;
- certified Software Dev as Reference App mapping only, not Core Runtime authority;
- added release-gate artifacts `core-runtime-kernel`, `core-runtime-admission`, `core-runtime-arbitration`, `core-runtime-negative-fixtures`, and `v105-release-certification`;
- covered positive and negative fixtures for command, admission, proposal, arbitration, file-backed registry loader, executor closeout, state writeback and Software Dev reference mapping;
- advanced workspace and desktop version metadata to `1.0.5`.

## v1.0.4 - 2026-06-28

Core Ontology Kernel baseline:

- defined Core Ontology Kernel, Object / Link Schema, Action / State Semantics, Skill Registry, and Evidence / Decision Reference Model;
- added file-backed Core ontology registry and read-only projection contract;
- extended release gate coverage with `core-ontology-kernel`, `core-object-link-schema`, `core-action-state-semantics`, `core-skill-registry`, `core-evidence-decision-reference-model`, `core-file-backed-ontology-registry`, and `v104-release-certification` artifacts;
- kept Software Dev terminology as Reference App mapping only, not Core authority;
- advanced workspace and desktop version metadata to `1.0.4`.

## v1.0.3 - 2026-06-28

Core 4-D Spec Intake baseline:

- confirmed Core 4-D intake contract for Deconstruct / Diagnose / Develop / Deliver;
- added Core Spec Bundle slices across Intent, Domain, Goal, Plan, Task, Decision, Output and Feedback;
- certified cross-industry reference mappings for Software Dev, UI Design and Video Production;
- extended release gate coverage with `v103-release-fix-certification` and `core-4d-spec-intake` artifacts;
- preserved `v1.0.2` release audit certification while advancing the Core Spec Kernel roadmap.

## v1.0.2 - 2026-06-26

Release audit fix baseline:

- runtime governance telemetry now ignores request-input provider-ready claims;
- release provenance distinguishes lightweight and annotated tag semantics;
- release certification records V102 negative fixture coverage;
- product goal baseline is Spec-Driven Software Dev Workflow.

## Historical Changelog

完整历史 changelog 已归档到：

- [docs/project/history/2026-06-current-baseline-history/CHANGELOG.md](docs/project/history/2026-06-current-baseline-history/CHANGELOG.md)

历史版本记录只作为追溯材料，不作为当前开发入口。
