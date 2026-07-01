# Changelog

更新日期：2026-06-30
执行者：Codex

## Current Baseline

当前发布基线：

- [docs/delivery/releases/v1.0.9/README.md](docs/delivery/releases/v1.0.9/README.md)
- [docs/delivery/releases/v1.0.9/AGENTFLOW_V1_0_9_SOFTWARE_DEV_REFERENCE_APP_TASKS_V1.md](docs/delivery/releases/v1.0.9/AGENTFLOW_V1_0_9_SOFTWARE_DEV_REFERENCE_APP_TASKS_V1.md)

下一版计划：

- v1.1.0 Product Surface hardening。

## Unreleased

下一版保留给 Product Surface hardening，不在本次 release 中认证。

Planned v1.1.0 foundation:

- harden product surface installation and user-facing console routes;
- keep `products/**` as source definitions and `.agentflow/**` as runtime facts;
- expand reference app certification without adding Core industry-specific authority.

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
