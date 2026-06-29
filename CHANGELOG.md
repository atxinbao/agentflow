# Changelog

更新日期：2026-06-29
执行者：Codex

## Current Baseline

当前发布基线：

- [docs/delivery/releases/v1.0.6/README.md](docs/delivery/releases/v1.0.6/README.md)
- [docs/delivery/releases/v1.0.6/AGENTFLOW_V1_0_6_CORE_EVIDENCE_KERNEL_TASKS_V1.md](docs/delivery/releases/v1.0.6/AGENTFLOW_V1_0_6_CORE_EVIDENCE_KERNEL_TASKS_V1.md)

下一版计划：

- v1.0.7 Decision Kernel。

## Unreleased

下一版保留给 Decision Kernel，不在本次 release 中认证。

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
