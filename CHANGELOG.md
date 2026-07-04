# Changelog

更新日期：2026-07-04
执行者：Codex

## Current Baseline

当前发布基线：

- [docs/delivery/releases/v1.1.9/README.md](docs/delivery/releases/v1.1.9/README.md)
- [docs/delivery/releases/v1.1.9/AGENTFLOW_V1_1_9_SOFTWARE_DEV_REFERENCE_APP_BETA_CERTIFICATION_TASKS_V1.md](docs/delivery/releases/v1.1.9/AGENTFLOW_V1_1_9_SOFTWARE_DEV_REFERENCE_APP_BETA_CERTIFICATION_TASKS_V1.md)

下一版计划：

- v1.2.0 Product Onboarding and First-run Experience。

## Unreleased

下一版保留给 Product Onboarding and First-run Experience，不在本次 release 中认证。

## v1.1.9 - 2026-07-04

Software Dev Reference App Beta Certification:

- aligned `v1.1.9` release authority with `docs/project/roadmap.md` as the Software Dev reference app beta baseline;
- added top-level release certification metadata contract checks for `releaseVersion`, `releaseTag`, `sourceCommit`, `workflowRunId`, `artifactNames` and `primaryProofs`;
- hardened recovery receipt idempotency so same path with different key or missing key is rejected instead of overwriting existing receipts;
- added positive projection rebuild recovery proof from event replay and negative proof for missing event inputs;
- tightened workspace health so configured provider names do not count as readiness without provider smoke and skill smoke evidence;
- documented Software Dev reference app beta scope across Domain Pack, Surface Pack, Connector Pack and Desktop without treating it as Core GA or commercial launch;
- added golden Project -> Intake -> Tasks, Executor -> Evidence -> Decision -> Delivery, and Failure -> Retry -> Feedback proof scenarios;
- added Desktop beta readiness smoke proof for Runtime API-backed executor flow display;
- added v1.1.9 release-gate proof artifacts and task traceability for GitHub issues `#841` through `#850`;
- advanced workspace and desktop version metadata to `1.1.9`.

## v1.1.8 - 2026-07-04

Recovery / Resume / Failure Handling:

- aligned `v1.1.8` release authority with `docs/project/roadmap.md` as the recovery and failure handling baseline;
- hardened release closeout metadata so certification records `releaseVersion`, `releaseTag`, `sourceCommit`, `workflowRunId`, `artifactNames` and `primaryProofs`;
- tightened executor evidence graph completion so `complete` is only emitted after run, handoff, boundary, validation, evidence and closeout facts are all ready;
- added Runtime API resume receipts for interrupted or failed runs without marking terminal tasks as done;
- added failed command recovery receipts for retry, replace, rerun and block paths while preserving the original failed command evidence;
- added interrupt lifecycle closeout semantics so interrupted executor sessions remain resumable and do not fake completion;
- added duplicate command / idempotency handling for recovery and resume operations;
- added projection rebuild recovery receipts and workspace health reports for missing authority, stale projections and provider readiness markers;
- connected Desktop task details to the `load_executor_flow_read_model` Runtime API query for read-only executor flow display;
- added v1.1.8 release-gate proof artifacts and task traceability for GitHub issues `#830` through `#839`;
- advanced workspace and desktop version metadata to `1.1.8`.

## v1.1.7 - 2026-07-04

Evidence / Decision / Delivery User Readability:

- aligned `v1.1.7` release authority with `docs/project/roadmap.md` as user-readable executor closure;
- hardened executor surface path validation so invalid paths, absolute paths, parent traversal, malformed workspace refs and unsupported glob patterns are rejected instead of normalized into `docs`;
- added Desktop Runtime API command `load_executor_flow_read_model` so Desktop can render executor state through Runtime API instead of reading authority files directly;
- added executor flow read model with action visibility, evidence graph, decision reasons, delivery package, repair actions and portable diagnostic refs;
- projected evidence graph nodes and links for run, handoff, boundary, validation, evidence and closeout facts;
- projected Decision accepted / not-ready reasons and remediation paths from boundary, evidence and closeout facts;
- projected Delivery Package summaries for ready and not-ready states without reintroducing Audit as a default blocker;
- documented that Audit remains optional sidecar for executor delivery readability;
- separated portable project refs from local-only diagnostic paths in executor projections;
- added v1.1.7 release-gate proof artifacts and task traceability for GitHub issues `#819` through `#828`;
- advanced workspace and desktop version metadata to `1.1.7`.

## v1.1.6 - 2026-07-03

Executor Adapter Real Execution Closure:

- aligned `v1.1.6` release authority with `docs/project/roadmap.md` so the release focuses on Executor Adapter real execution, not provider launch closure;
- hardened Core route next-action semantics so `clarify` and `research` never expose confirmation or materialization authority actions;
- exposed Product Spec Intake through Desktop Runtime API commands for preview, confirmation and materialization flows;
- added Executor Adapter handoff package generation under `.agentflow/tasks/<issue-id>/runs/<run-id>/launch/**`;
- added allowed surface and diff boundary checking before executor result writeback;
- added executor evidence capture that binds command evidence, validation output, handoff refs and diff boundary reports;
- added executor result writeback that normalizes issue and run status after evidence and boundary checks pass;
- added explicit failure, timeout, cancel and retry lifecycle receipts, with retry creating a new run;
- certified the Software Dev real executor golden path from Spec Issue to handoff, evidence, boundary, writeback and projection;
- added v1.1.6 release-gate proof artifacts and task traceability for GitHub issues `#808` through `#817`;
- advanced workspace and desktop version metadata to `1.1.6`.

## v1.1.5 - 2026-07-03

Spec Intake to Goal / Roadmap / Task Productization:

- aligned the release plan with `docs/project/roadmap.md` so `v1.1.5` is Spec Intake productization, not provider launch closure;
- added Product-level Intent Intake that preserves raw human input, selected Product, workspace id, source surface, locale and source refs;
- added Product route decisions for `clarify`, `research`, `define`, `plan`, `task`, `decide`, `deliver` and `evolve` without granting preview authority writes;
- added preview-first Product Spec artifacts under `.agentflow/previews/spec-intake/**`;
- added confirmation records bound to preview id and preview hash before authority writes are allowed;
- added confirmed materialization from Product Spec preview to public `docs/requirements/**` and local `.agentflow/spec/projects/**` / `.agentflow/spec/issues/**`;
- added Desktop Runtime API bridge commands for Product Workspace creation/projection and Product Spec Intake preview/confirmation/materialization;
- added portable `workspace://` receipt/projection refs while keeping absolute paths confined to local diagnostics;
- added v1.1.5 release-gate proof artifacts and task traceability for GitHub issues `#797` through `#806`;
- advanced workspace and desktop version metadata to `1.1.5`.

## v1.1.4 - 2026-07-02

Project Creation and Product Workspace:

- added Product command dry-run receipt binding so submit must reuse the exact dry-run receipt for the same target and input;
- added Desktop confirm-submit behavior proof over the Product command dry-run and submit path;
- added Product workspace creation contract with machine-readable receipt, paths, active Product binding, blockers and statuses;
- initialized standard `docs/project/**` records and `.agentflow/spec/**`, `.agentflow/events/**`, `.agentflow/tasks/**` local fact roots for Product-selected workspaces;
- added active Product workspace projection for read-only Desktop and runtime surfaces;
- added duplicate, partial, invalid-root and missing-product recovery states for workspace initialization;
- certified Software Dev as the default Product workspace golden path while keeping Product-specific constants out of Core bridge code;
- expanded release-gate proof artifacts with v1.1.4 receipt, Desktop, pollution, workspace, bootstrap, projection, failure, golden path and release certification proofs;
- added v1.1.4 release task traceability for GitHub issues `#785` through `#794`;
- advanced workspace and desktop version metadata to `1.1.4`.

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
