# Changelog

更新日期：2026-07-10
执行者：Codex

## Current Baseline

当前发布基线：

- [docs/delivery/releases/v1.2.9/README.md](docs/delivery/releases/v1.2.9/README.md)
- [docs/delivery/releases/v1.2.9/AGENTFLOW_V1_2_9_PAID_REPORT_COMMERCIAL_ORDER_ACCESS_CLOSURE_TASKS_V1.md](docs/delivery/releases/v1.2.9/AGENTFLOW_V1_2_9_PAID_REPORT_COMMERCIAL_ORDER_ACCESS_CLOSURE_TASKS_V1.md)

上一发布基线：

- [docs/delivery/releases/v1.2.8/README.md](docs/delivery/releases/v1.2.8/README.md)
- [docs/delivery/releases/v1.2.8/AGENTFLOW_V1_2_8_PAID_REPORT_RUN_DELIVERY_ARTIFACT_CLOSURE_TASKS_V1.md](docs/delivery/releases/v1.2.8/AGENTFLOW_V1_2_8_PAID_REPORT_RUN_DELIVERY_ARTIFACT_CLOSURE_TASKS_V1.md)

历史发布基线：

- [docs/delivery/releases/v1.2.7/README.md](docs/delivery/releases/v1.2.7/README.md)
- [docs/delivery/releases/v1.2.7/AGENTFLOW_V1_2_7_PAID_REPORT_RUNTIME_HANDOFF_CLOSURE_TASKS_V1.md](docs/delivery/releases/v1.2.7/AGENTFLOW_V1_2_7_PAID_REPORT_RUNTIME_HANDOFF_CLOSURE_TASKS_V1.md)
- [docs/delivery/releases/v1.2.6/README.md](docs/delivery/releases/v1.2.6/README.md)
- [docs/delivery/releases/v1.2.6/AGENTFLOW_V1_2_6_PROJECT_SCOPED_COMMERCIAL_PRODUCT_INSTANCE_TASKS_V1.md](docs/delivery/releases/v1.2.6/AGENTFLOW_V1_2_6_PROJECT_SCOPED_COMMERCIAL_PRODUCT_INSTANCE_TASKS_V1.md)

下一版计划：

- [docs/delivery/releases/v1.3.0/README.md](docs/delivery/releases/v1.3.0/README.md)
- [docs/delivery/releases/v1.3.0/AGENTFLOW_V1_3_0_COMMERCIAL_BACKEND_STABLE_CLOSURE_TASKS_V1.md](docs/delivery/releases/v1.3.0/AGENTFLOW_V1_3_0_COMMERCIAL_BACKEND_STABLE_CLOSURE_TASKS_V1.md)

## Unreleased

Commercial Backend Stable Closure:

- started v1.3.0 delivery planning with `#993` through `#1002`;
- recorded v1.2.9 live release audit facts as the first v1.3.0 proof artifact;
- split v1.2.9 release certification coverage between live GitHub release
  authority and synthetic project release sidecar rejection;
- updated v1.2.9 final certification payload to expose concrete tag kind,
  annotated tag object id and peeled commit sha when live release provenance is
  available;
- added the v1.3.0 commercial backend stable contract as a machine-readable
  Product / Order / Entitlement / Run / Artifact / Evidence / Decision /
  Delivery / Feedback schema inventory;
- added the v1.3.0 Paid Report Flow state machine covering draft order through
  closed delivery, with positive lifecycle fixtures and negative invalid
  transition fixtures that cannot write accepted or delivery-ready authority;
- added the v1.3.0 commercial authority boundary map, separating writable Order
  / Entitlement / Run / Artifact / Evidence / Decision / Access / Policy facts
  from read-only Projection / Customer View / Download View / sidecar surfaces;
- added the v1.3.0 Product SKU extension contract, defining how Product / Pack
  / SKU files bind concrete paid-report SKU fields while missing SKU definitions
  stay invalid and never fall back to generic hardcoded report content;
- added the v1.3.0 Provider / Generator adapter boundary, freezing dry-run
  generation request / receipt / artifact / evidence refs and negative fixtures
  without calling a real provider;
- added the v1.3.0 Payment Provider adapter boundary, freezing dry-run payment
  intent / checkout session / entitlement authorization / refund status fixtures
  without executing real checkout, charge or refund flows;
- added the v1.3.0 Customer Delivery Backend contract, freezing customer
  access, expiry, revocation, refund, repair, rerun and feedback fixtures with
  stable `nextAction` semantics;
- added the v1.3.0 Commercial E2E golden scenario, connecting Product SKU,
  Order, Entitlement, Admission, Generation Adapter, Artifact, Evidence,
  Decision, Delivery, Access and Feedback into one generic machine-readable
  proof;
- added the v1.3.0 release certification artifact, binding all V130 primary
  proofs, release metadata, commercial backend stable status and explicit
  non-goal boundary flags;
- required explicit migration or version bump policy for backward-incompatible
  commercial backend contract changes after v1.3.0.

## v1.2.9 - 2026-07-09

Paid Report Commercial Order and Access Closure:

- repaired published release provenance / release facts commit alignment so synthetic project-release-gate-e2e facts cannot satisfy release certification;
- repaired annotated and lightweight tag kind certification by separating tag object id and peeled commit sha semantics;
- added generic Paid Report Order Record contract binding request, Product Instance, order intent, input snapshot and offer metadata;
- added payment / entitlement authorization boundary with paid, waived, deferred, refunded and missing states, without provider checkout;
- added Order-to-Run Admission Gate requiring valid order record, authorization, input snapshot and runtime receipt;
- added Customer Delivery Access Projection for accessible, blocked, expired and repair-needed report delivery states;
- added report access receipt contract for allowed, expired and revoked customer delivery access;
- added refund / repair / controlled rerun policy contract without mutating delivered artifacts in place;
- added commercial negative fixtures for stale release facts, unknown tag kind, fake paid state, refunded order, mismatched product instance, missing input snapshot, missing accepted decision and expired access token;
- added v1.2.9 release delivery baseline and GitHub issue traceability for `#979` through `#988`;
- advanced workspace and desktop version metadata to `1.2.9`.

## v1.2.8 - 2026-07-09

Paid Report Run and Delivery Artifact Closure:

- repaired release provenance and tag policy certification so stale fixture commit facts cannot pass published release certification;
- made Paid Report product instance identity project-unique by binding it to the active project/workspace digest;
- added generic Paid Report input snapshot and order intent contracts without introducing concrete industry SKU authority;
- added Paid Report run execution receipt with deterministic started / completed / blocked cases;
- added report artifact schema and storage boundary for project-scoped report artifacts;
- added report generation evidence pack linking input snapshot, run receipt, report artifact and generation receipt;
- added delivery decision gate with accepted / needs-fix / rejected / deferred / blocked outcomes;
- added delivery package projection and download/display contract that is ready only after accepted decision;
- added feedback and repair request loop projection without mutating delivered artifacts;
- added v1.2.8 release delivery baseline and GitHub issue traceability for `#967` through `#976`;
- advanced workspace and desktop version metadata to `1.2.8`.

## v1.2.7 - 2026-07-08

Paid Report Runtime Handoff Closure:

- aligned v1.2.7 planning around project-scoped Paid Report Runtime handoff closure;
- documented Software Dev as Managed Project Flow Reference App and Paid Report as generic backend handoff;
- added project-scoped Paid Report product instance resolver API;
- added project-scoped Paid Report preflight / Runtime proposal handoff API;
- updated Desktop Paid Report preflight bridge to use active project root;
- certified golden path source semantics so Core Runtime stays generic and concrete paid report SKU names are not Core authority;
- added Runtime proposal admission receipt for Paid Report handoff;
- added Paid Report run contract boundary that requires admission receipt and cannot start directly from preflight;
- added Evidence / Decision / Delivery projection contract for Paid Report flow;
- added v1.2.7 release delivery baseline and GitHub issue traceability for `#956` through `#965`;
- advanced workspace and desktop version metadata to `1.2.7`.

## v1.2.6 - 2026-07-07

Project-scoped Commercial Product Instance Hardening:

- added top-level `certificationKind` to final release certification proof semantics;
- separated production commercial registry from fixture-only negative commercial registry;
- added project-scoped commercial registry resolver for Runtime and Desktop commands;
- hardened Commercial Product read model status semantics with `ready` / `partial` / `deferred` / `invalid` / `unavailable`;
- moved commercial golden path proof to registry-only inputs;
- updated Desktop commercial Runtime command to use active project root;
- added Paid Report product instance contract with input, report definition, evidence, decision and delivery requirements;
- added Paid Report preflight to Runtime proposal handoff that cannot start a run directly;
- added negative fixture isolation proof so fixture-only product ids cannot leak into production product surface;
- added v1.2.6 release delivery baseline and GitHub issue traceability for `#945` through `#954`;
- advanced workspace and desktop version metadata to `1.2.6`.

## v1.2.5 - 2026-07-07

Published Release Certification and Registry-backed Commercial Runtime:

- added release publication state proof that separates candidate, tagged, released and published states;
- split candidate certification from published certification so PR / local release gates cannot claim published release facts;
- tightened waiver contract consistency across absent, complete and invalid waiver cases;
- added `products/commercial-runtime/**` as the commercial product registry and entitlement source fixture;
- moved Commercial Product read model production source to registry/config-backed Runtime input while keeping default inputs as fallback/test data only;
- added entitlement source coverage for active, trial, deferred and invalid commercial states;
- added Paid Report product definition fixture with required inputs, evidence requirements and decision requirements;
- certified Desktop commercial production surface as Runtime/Tauri read-only with marked Browser Preview fallback;
- added registry-backed Commercial Surface golden path proof;
- added release-event artifact certification for tag, GitHub Release URL, workflow run, source commit, artifact manifest and milestone facts;
- added v1.2.5 release delivery baseline and GitHub issue traceability for `#934` through `#943`;
- advanced workspace and desktop version metadata to `1.2.5`.

## v1.2.4 - 2026-07-07

Commercial Runtime Read Model and Closeout Distinction:

- split live closeout facts into `hasNoOpenIssues` and `isMilestoneClosed` so zero open issues cannot be treated as a closed milestone;
- added final release certification rejection for deferred live closeout without explicit waiver metadata;
- added Commercial Product Runtime read model API for paid report and managed project commercial state;
- added Commercial Product projection query surface with read-only authority semantics;
- updated Desktop Commercial Surface to consume the Runtime read model and mark Browser Preview fallback explicitly;
- added Paid Report preflight Runtime API before Runtime command admission;
- added Managed Project commercial boundary Runtime fixture so commercial entitlement does not change Core Runtime authority;
- added commercial negative Runtime fixtures for disabled, expired, deferred, missing and wrong-flow states;
- added Commercial Surface golden path proof from read model to projection, Desktop and preflight;
- added v1.2.4 release delivery baseline and GitHub issue traceability for `#923` through `#932`;
- advanced workspace and desktop version metadata to `1.2.4`.

## v1.2.3 - 2026-07-07

Release Closeout Proof Hardening and Commercial Surface Traceability:

- added live GitHub milestone closeout certification for release proof that depends on remote issue / milestone state;
- added a negative fixture so release closeout proof cannot self-assert remote provider state;
- recorded the repaired `v1.2.2` milestone closeout as a v1.2.3 release-gate proof without rewriting the published v1.2.2 tag;
- added a v1.2.2 commercial proof version negative fixture to reject wrong-version commercial primary proofs;
- added commercial product read model contract proof for projection-only commercial boundary facts;
- added paid report flow preflight proof so paid-only flows fail before Runtime execution when entitlement or paid feature state is invalid;
- added managed project flow commercial boundary proof so managed project workflows cannot gain paid report authority;
- added Desktop commercial boundary surface proof for read-only disabled / deferred / managed-project commercial states;
- added commercial boundary negative fixtures covering unavailable, invalid and wrong-authority states;
- added v1.2.3 release delivery baseline and GitHub issue traceability for `#903` through `#912`;
- advanced workspace and desktop version metadata to `1.2.3`.

## v1.2.2 - 2026-07-06

Release Proof Hardening and Commercial Boundary Preflight:

- added a dedicated V121 release certification gate so the v1.2.1 proof chain is externally readable;
- aligned root `certification.json` top-level metadata with runtime release certification metadata;
- added primary proof artifact manifest indexing path, sha256, byte size, proof role and GitHub issue refs;
- added V121 issue traceability and milestone closeout proof for release-gate certification;
- bound Desktop team workflow surfaces to Runtime read model commands;
- added Commercial Boundary Contract for Product layer concepts, Product surface boundary and commercial non-goals;
- added License / Entitlement Boundary covering active, disabled, expired, missing and deferred entitlement states;
- added Paid Feature Boundary so paid-only Product flows are blocked before Runtime command admission;
- added Paid Report Flow vs Managed Project Flow Contract while keeping both mapped to Spec / Evidence / Decision / Delivery;
- added v1.2.2 release delivery baseline and GitHub issue traceability for `#883` through `#892`;
- advanced workspace and desktop version metadata to `1.2.2`.

## v1.2.1 - 2026-07-05

First-run Execution Closure and Team Workflow Boundary:

- closed first-run Desktop execution gaps by routing onboarding through Runtime API commands;
- bound Desktop onboarding readiness to Runtime read models instead of static UI assumptions;
- converted the guided sample into an actual deterministic Runtime run with success and failure receipts;
- exposed guided sample evidence / decision / delivery proof paths and kept failed runs repairable;
- added first-run failure / retry UI state so failed sample runs do not silently become Done;
- added the local/lightweight Team Workflow Boundary Contract and kept cloud multi-tenant, public commercial launch, payment and new industry work out of scope;
- added Project Sharing Read Model for product, goal, roadmap, task, latest decision, delivery and feedback summaries;
- added Role / Permission / Handoff View for Spec Agent, Build Agent, Audit Agent, Review Agent, Human Owner and Viewer;
- added Team Delivery / Decision History View with decision, delivery, feedback route and optional audit sidecar entries;
- added v1.2.1 release delivery baseline and GitHub issue traceability for `#863` through `#872`;
- advanced workspace and desktop version metadata to `1.2.1`.

## v1.2.0 - 2026-07-05

Product Onboarding and First-run Experience:

- added Runtime API first-run onboarding contract for choosing Software Dev Reference App, bootstrapping a workspace, checking readiness and running a guided sample;
- added Product onboarding readiness report covering Product definition, workspace projection, provider smoke, connector status and skill status;
- added guided sample run plan from intake through tasks, executor, evidence, delivery and repairable feedback;
- added Desktop Runtime API bridge commands for onboarding contract, readiness and guided sample plan;
- updated Desktop first-run onboarding surface to present Software Dev Reference App, Runtime facts and provider / connector / skill readiness without exposing `.agentflow` internals as primary UX;
- added release-gate proof artifacts for top-level metadata, primary proof manifest, first-run contract, product bootstrap, readiness, provider setup, guided sample, Desktop surface, hidden `.agentflow` boundary and release certification;
- added v1.2.0 release task traceability for GitHub issues `#852` through `#861`;
- advanced workspace and desktop version metadata to `1.2.0`.

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
