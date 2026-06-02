# Legacy Removal Audit

创建日期：2026-06-02  
执行者：Codex

## Purpose

This audit implements `005 - Legacy and Degraded Code Removal`.

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
| `cli-legacy` | Old CLI command still compiles against this compatibility path. | Keep through a named `agentflow_core::legacy::<area>` module. |
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

## CLI Legacy Compatibility Symbols

CLI legacy dispatch now imports explicit named modules instead of relying on
crate-root legacy exports.

| Module | Key symbols | Used by | Category | Action |
| --- | --- | --- | --- | --- |
| `legacy::goal_protocol` | `init_from_goal`, `bootstrap_goal_protocol`, `check_goal_readiness`, `write_goal_next` | `agentflow init`, `agentflow goal ...` | cli-legacy | keep temporarily |
| `legacy::product_feature` | `create_product_feature`, `read_product_feature_execution_status`, `read_product_feature_execution_next`, `ProductFeatureDraft` | `agentflow feature ...` | cli-legacy | keep temporarily |
| `legacy::team_project_milestone_issue` | `create_team`, `create_project`, `create_milestone`, `create_issue`, `read_local_project_seed_preview`, `write_local_project_seed`, `read_issue_project_link_preview`, `write_issue_project_link`, draft/write DTOs | old create/project-seed/issue-link commands | cli-legacy and active-read-model | keep temporarily |
| `legacy::run_verify_review` | `plan_issue`, `write_context`, `run_issue`, `verify_issue`, `review_issue`, `write_project_summary`, `write_review_assistant`, `ControlledRunPlan` | old plan/context/run/verify/review/update/review-assistant commands | cli-legacy | keep temporarily |
| `legacy::sqlite_index` | `rebuild_index` | `agentflow index rebuild` | cli-legacy | keep temporarily |
| `legacy::saved_view` | `save_view`, `show_view`, `SavedViewFilter` | `agentflow view ...` | cli-legacy | keep temporarily |
| `legacy::eligibility_lease` | `write_workflow_eligibility`, `write_workflow_lease_snapshot` | `agentflow eligibility`, `agentflow lease` | cli-legacy | keep temporarily |
| `legacy::workflow_control` | `write_workflow_state_check` | `agentflow state check` | cli-legacy | keep temporarily |
| `legacy::project_closure` | `write_project_closure_state` | `agentflow project closure` | cli-legacy | keep temporarily |
| `legacy::project_audit_docs_refresh` | `write_project_code_audit_snapshot`, `write_project_docs_refresh_snapshot` | old audit/docs-refresh commands | cli-legacy | keep temporarily |

## Legacy Re-export Audit

| Compatibility module | Status | Classification | Action |
| --- | --- | --- | --- |
| `legacy::archive_2026_05` | no longer public | unused public exposure | removed public module visibility |
| `legacy::goal_protocol` | explicit re-export list | cli-legacy | keep temporarily |
| `legacy::product_feature` | explicit re-export list | cli-legacy | keep temporarily |
| `legacy::team_project_milestone_issue` | explicit re-export list | cli-legacy and active-read-model | keep temporarily |
| `legacy::workflow_control` | explicit re-export list | active-read-model and cli-legacy | keep temporarily |
| `legacy::run_verify_review` | explicit re-export list | cli-legacy | keep temporarily |
| `legacy::eligibility_lease` | explicit re-export list | cli-legacy | keep temporarily |
| `legacy::project_closure` | explicit re-export list | cli-legacy | keep temporarily |
| `legacy::project_audit_docs_refresh` | explicit re-export list | cli-legacy | keep temporarily |
| `legacy::saved_view` | explicit re-export list | cli-legacy | keep temporarily |
| `legacy::sqlite_index` | explicit re-export list | cli-legacy | keep temporarily |
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
are either used by CLI compatibility, active read models, archived tests, or
nested DTO/serde shapes that need a replacement requirement before removal.

## Test-only and Private Archive Symbols

The following groups remain in the private archive because they are test-covered
or internally needed by archived functions:

| Group | Examples | Category | Action |
| --- | --- | --- | --- |
| Evidence/index DTOs | `AepIssueProtocol`, `IndexedRun`, `IndexedUpdate` | test-only / internal archive | keep private, no public compatibility module |
| Run data shapes | `AgentRun`, `RunOutputs`, `CommandRecord`, `ValidationSummary`, `ReviewSummary` | cli-legacy / test-only | keep through `run_verify_review` where needed |
| Closure/audit DTOs | `ProjectClosureStateSnapshot`, `ProjectCodeAuditSnapshot`, `ProjectDocsRefreshSnapshot` | cli-legacy / test-only | keep temporarily |
| Status normalization | `ProjectStatus`, `IssueStatus`, `canonical_*` functions | active-read-model / cli-legacy | keep temporarily |
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
| Legacy GoalLoop / eligibility / lease / closure writers | archive + named legacy modules | cli-legacy | keep temporarily; not authorized for new product flows |

## CLI Boundary Result

`crates/agentflow-cli/src/legacy.rs` now imports exactly the compatibility areas it
uses:

```rust
use agentflow_core::active::{...};
use agentflow_core::legacy::goal_protocol::{...};
use agentflow_core::legacy::team_project_milestone_issue::{...};
```

It no longer calls `agentflow_core::<symbol>` through crate-root legacy exports.

## Desktop Boundary Result

`apps/desktop/src-tauri/src/commands/legacy_core.rs` imports transitional read
models from:

```rust
agentflow_core::active
```

Tauri command names are unchanged. Desktop remains read-only.

## Residual Risk

- The archived implementation remains large because CLI compatibility and
  archived unit tests still exercise it.
- A later requirement may retire specific CLI legacy commands, then delete the
  corresponding named legacy module and implementation group.
- DTOs with public fields should not be deleted one by one until the caller
  surface is replaced, because serde and Rust return types may rely on nested
  shapes.

## Verification Snapshot

Initial verification after export narrowing:

```text
cargo test -p agentflow-core: pass, 61 tests
cargo test -p agentflow-cli: pass, 0 tests
```

Full verification is recorded in `verification.md`.
