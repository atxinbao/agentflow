# Legacy Removal Audit

创建日期：2026-06-02  
执行者：Codex

## Purpose

This audit implements `005 - Legacy and Degraded Code Removal` and records the
follow-up 006 CLI retirement delta.

The goal is not to delete every archived implementation from
`crates/agentflow-core/src/legacy/archive_2026_05.rs`. The goal is to remove
legacy code that is safely unreachable, narrow the public export surface, and
record which compatibility paths still exist because Desktop, CLI, or tests use
them.

## Audit Commands

```bash
rg "pub struct|pub enum|pub fn|pub const|pub type" crates/agentflow-core/src/legacy
rg "pub use archive_2026_05::\*|pub use legacy::\*|use agentflow_core::legacy::\*|use agentflow_core::\*" crates apps
rg "legacy::evidence|AepIssueProtocol|IndexedRun|IndexedUpdate" crates apps docs --glob '!docs/archive/**' --glob '!apps/desktop/dist/**'
cargo test -p agentflow-core
cargo test -p agentflow-cli
```

## Archive Public Symbol Inventory

`legacy/archive_2026_05.rs` currently contains:

| Kind | Count | Notes |
| --- | ---: | --- |
| `pub const` | 2 | `AGENTFLOW_DIR`, `VERSION` are archived compatibility constants. |
| `pub enum` | 2 | `ProjectStatus`, `IssueStatus` support legacy status normalization. |
| `pub struct` | 129 | Most are DTOs used by old CLI commands, transitional Desktop snapshots, and archived tests. |
| `pub fn` | 43 | Functions are old command/read-model entrypoints. |

The archive module is now private:

```rust
mod archive_2026_05;
```

Therefore these public symbols are no longer blanket-visible from the crate
root or from `agentflow_core::legacy::*`. They are reachable only when a named
legacy submodule explicitly re-exports them.

## Classification Summary

| Category | Meaning | Action |
| --- | --- | --- |
| `active-read-model` | Current Desktop read-only snapshots still need this path through `active/`. | Keep through `agentflow_core::active`. |
| `cli-retired` | Old CLI command still parses but must not execute archived writes. | Route through `agentflow-cli/src/retirement.rs`. |
| `test-only` | Used by archived unit tests or internal helpers but not by active/CLI import paths. | Keep in private archive unless a later test-retirement requirement removes it. |
| `unused` | No active/CLI/Desktop/test compatibility import. | Delete or hide public compatibility export. |
| `uncertain` | Public DTO appears in nested return fields or serde shapes and should not be removed without deeper replacement. | Keep private or explicitly re-export only where needed. |

## Active Read Model Symbols

These remain available through `crates/agentflow-core/src/active/` and are not
legacy product authorization:

| Symbol | Kind | Source compatibility module | Used by | Category | Action |
| --- | --- | --- | --- | --- | --- |
| `read_desktop_workbench_snapshot` | fn | `legacy::workflow_control` | Tauri `load_workbench_snapshot` | active-read-model | keep |
| `DesktopWorkbenchSnapshot` | struct | `legacy::workflow_control` | Tauri return type | active-read-model | keep |
| `read_local_metrics_snapshot` | fn | `legacy::workflow_control` | Tauri + CLI metrics | active-read-model | keep |
| `LocalMetricsSnapshot` | struct | `legacy::workflow_control` | Tauri + CLI metrics | active-read-model | keep |
| `read_local_project_model_snapshot` | fn | `legacy::team_project_milestone_issue` | Tauri + CLI projects | active-read-model | keep |
| `LocalProjectModelSnapshot` | struct | `legacy::team_project_milestone_issue` | Tauri + CLI projects | active-read-model | keep |
| `read_project_milestone_issue_view_model_snapshot` | fn | `legacy::team_project_milestone_issue` | Tauri legacy read model | active-read-model | keep |
| `ProjectMilestoneIssueViewModelSnapshot` | struct | `legacy::team_project_milestone_issue` | Tauri legacy read model | active-read-model | keep |
| `read_local_search_snapshot` | fn | `legacy::workflow_control` | Tauri + CLI search | active-read-model | keep |
| `LocalSearchSnapshot` | struct | `legacy::workflow_control` | Tauri + CLI search | active-read-model | keep |
| `WorkbenchBoundary` | struct | `legacy::workflow_control` | active boundary wrapper | active-read-model | keep |

## 006 CLI Retirement Delta

CLI legacy dispatch now imports only `agentflow_core::active` read models.
`agentflow-cli/src/retirement.rs` classifies commands and disables old write
flows before they can reach archived functions.

| Command group | Category | Action |
| --- | --- | --- |
| `metrics`, `projects`, `search` | active-read-model | keep temporary read-only execution |
| all other old CLI commands | cli-retired | parse, print retirement message, perform no work |

Full command classification lives in
`docs/architecture/legacy-cli-retirement-plan.md`.

## Legacy Compatibility Symbols After 006

The named legacy modules now expose only active read-model functions/DTOs or
DTOs required by those read-model return shapes.

| Module | Exposed surface after 006 | Category | Action |
| --- | --- | --- | --- |
| `legacy::goal_protocol` | `GoalLoop*`, `ProjectDefinition*`, `ProjectGoal` DTOs | active-read-model nested DTO | keep temporarily |
| `legacy::product_feature` | no public entrypoint | retired writer surface | no re-export |
| `legacy::team_project_milestone_issue` | local project snapshot readers and DTOs | active-read-model | keep |
| `legacy::workflow_control` | desktop / metrics / search readers and DTOs | active-read-model | keep |
| `legacy::run_verify_review` | `AgentRun`, `CommandRecord`, `HumanGate`, `RunOutputs`, `ValidationSpec` DTOs | active-read-model nested DTO | keep temporarily |
| `legacy::eligibility_lease` | no public entrypoint | retired writer surface | no re-export |
| `legacy::project_closure` | no public entrypoint | retired writer surface | no re-export |
| `legacy::project_audit_docs_refresh` | no public entrypoint | retired writer surface | no re-export |
| `legacy::saved_view` | `SavedView`, `SavedViewFilter`, `ViewFilterV1Preview` DTOs | active-read-model nested DTO | keep temporarily |
| `legacy::sqlite_index` | no public entrypoint | retired writer surface | no re-export |

## Legacy Re-export Audit

| Compatibility module | Status | Classification | Action |
| --- | --- | --- | --- |
| `legacy::archive_2026_05` | no longer public | unused public exposure | removed public module visibility |
| `legacy::goal_protocol` | DTO-only re-export list | active-read-model nested DTO | keep temporarily |
| `legacy::product_feature` | no re-exported entrypoints | retired writer surface | no public export |
| `legacy::team_project_milestone_issue` | read-model re-export list | active-read-model | keep temporarily |
| `legacy::workflow_control` | read-model re-export list | active-read-model | keep temporarily |
| `legacy::run_verify_review` | DTO-only re-export list | active-read-model nested DTO | keep temporarily |
| `legacy::eligibility_lease` | no re-exported entrypoints | retired writer surface | no public export |
| `legacy::project_closure` | no re-exported entrypoints | retired writer surface | no public export |
| `legacy::project_audit_docs_refresh` | no re-exported entrypoints | retired writer surface | no public export |
| `legacy::saved_view` | DTO-only re-export list | active-read-model nested DTO | keep temporarily |
| `legacy::sqlite_index` | no re-exported entrypoints | retired writer surface | no public export |
| `legacy::evidence` | no active/CLI/Desktop import | unused | deleted |

## Deleted Legacy Surface

The following legacy public exposure was removed in this slice:

| Removed item | Previous behavior | Reason |
| --- | --- | --- |
| `pub use legacy::*` in `agentflow-core/src/lib.rs` | Old code appeared at the crate root. | Too broad; new requirements must not see old workflow as product core. |
| `pub mod archive_2026_05` in `legacy/mod.rs` | Entire archived implementation was directly importable. | Too broad; callers must use named compatibility modules. |
| `pub use archive_2026_05::*` in `legacy/mod.rs` | Every archived symbol was visible as `agentflow_core::legacy::*`. | Too broad; prevents meaningful deletion/audit. |
| `legacy/evidence.rs` module | Re-exported `AepIssueProtocol`, `IndexedRun`, `IndexedUpdate`. | No active/CLI/Desktop import outside the private archive. |

No archived implementation function was deleted because the remaining candidates
are either used by active read models, archived tests, or nested DTO/serde
shapes that need a replacement requirement before removal.

006 removed CLI reachability to those functions. The private archive now has an
explicit `dead_code` allowance because retired functions remain only as private,
test-covered historical implementation until a later requirement prunes tests
and replacement read DTOs.

## Test-only and Private Archive Symbols

The following groups remain in the private archive because they are test-covered
or internally needed by archived functions:

| Group | Examples | Category | Action |
| --- | --- | --- | --- |
| Evidence/index DTOs | `AepIssueProtocol`, `IndexedRun`, `IndexedUpdate` | test-only / internal archive | keep private, no public compatibility module |
| Run data shapes | `AgentRun`, `RunOutputs`, `CommandRecord`, `ValidationSummary`, `ReviewSummary` | active-read-model nested DTO / test-only | keep through DTO-only re-export where needed |
| Closure/audit DTOs | `ProjectClosureStateSnapshot`, `ProjectCodeAuditSnapshot`, `ProjectDocsRefreshSnapshot` | test-only / private archive | keep private |
| Status normalization | `ProjectStatus`, `IssueStatus`, `canonical_*` functions | active-read-model | keep temporarily |
| Local project read DTOs | `LocalWorkspace`, `LocalProject`, `LocalMilestone`, `LocalProjectIssueRef`, `V1*` DTOs | active-read-model | keep temporarily |

## Degraded / Fallback Code Classification

| Area | Code | Classification | Action |
| --- | --- | --- | --- |
| Graph watcher fallback | `crates/graph/src/watcher/fallback.rs` | necessary degraded backup | keep |
| Graph native watcher | `crates/graph/src/watcher/native.rs` | current graph capability | keep |
| Project File Reader fallback states | `apps/desktop/src/features/project-files/**` and `apps/desktop/src-tauri/src/project_files/**` | browser preview / unsupported file / large text fallback | keep |
| Browser preview mock data | Desktop frontend runtime mock path | required for browser verification without Tauri | keep |
| Desktop active transitional read model | `crates/agentflow-core/src/active/**` | required read-only compatibility | keep |
| Legacy root exports | `pub use legacy::*`, `pub use archive_2026_05::*` | obsolete exposure | delete |
| Legacy evidence compatibility module | `legacy/evidence.rs` | unused public compatibility export | delete |
| Legacy GoalLoop / eligibility / lease / closure writers | private archive only | cli-retired / test-only | no public re-export; not authorized for new product flows |

## CLI Boundary Result

`crates/agentflow-cli/src/legacy.rs` now imports only active read models:

```rust
use agentflow_core::active::{read_local_metrics_snapshot, read_local_project_model_snapshot, read_local_search_snapshot};
```

Retired commands are classified and gated by:

```rust
crates/agentflow-cli/src/retirement.rs
```

It no longer calls old writer symbols through named legacy compatibility modules.

## Desktop Boundary Result

`apps/desktop/src-tauri/src/commands/legacy_core.rs` imports transitional read
models from:

```rust
agentflow_core::active
```

Tauri command names are unchanged. Desktop remains read-only.

## Residual Risk

- The archived implementation remains large because archived unit tests still
  exercise it.
- A later requirement may retire old archive tests, then delete the
  corresponding private implementation group.
- DTOs with public fields should not be deleted one by one until the caller
  surface is replaced, because serde and Rust return types may rely on nested
  shapes.

## Verification Snapshot

Verification after 006 CLI retirement:

```text
cargo test -p agentflow-cli: pass, 2 tests
```

Full verification is recorded in `verification.md`.
