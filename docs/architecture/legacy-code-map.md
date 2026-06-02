# Legacy Code Map

创建日期：2026-06-02  
执行者：Codex

## Purpose

This document records the compatibility surface that still belongs to the archived 2026-05 AgentFlow workflow. It supports `004 - Legacy Cleanup and New Module Split` and `005 - Legacy and Degraded Code Removal` by making old code visible before it is quarantined, narrowed, or removed.

## Legacy Command Surface

The following CLI commands are treated as legacy compatibility unless a new requirement re-authorizes them:

- `goal`
- `feature`
- `team`
- `project create`
- `project closure`
- `project code-audit`
- `project docs-refresh`
- `milestone`
- `issue`
- `run`
- `verify`
- `review`
- `eligibility`
- `lease`
- `index`
- `view`
- `update`
- `metrics`
- `project-seed`
- `issue-link`
- `review-assistant`
- `state`

The command names are currently preserved for compatibility. New product flows must not be added to this legacy command surface.

## Legacy Core Areas

The current `agentflow-core` compatibility layer includes archived concepts from the old product direction:

- Goal Protocol
- Product Feature creation and execution
- Team / Project / Milestone / Issue writer contracts
- GoalLoop
- Eligibility and Lease snapshots
- Run / Verify / Review records
- Evidence and Review artifacts
- Saved View and local SQLite index
- Project Closure, Code Audit, and Docs Refresh
- Issue project link compatibility

These areas must stay behind `legacy/` or transitional `active/` read-model wrappers. They are not authorized as the new AgentFlow product core.

Current quarantine layout:

- `crates/agentflow-core/src/lib.rs` no longer re-exports `legacy::*`.
- `crates/agentflow-core/src/legacy/archive_2026_05.rs` contains the archived implementation and is private to the `legacy` module.
- `legacy/goal_protocol.rs` exposes archived goal protocol compatibility symbols.
- `legacy/product_feature.rs` exposes archived product feature compatibility symbols.
- `legacy/team_project_milestone_issue.rs` exposes archived Team / Project / Milestone / Issue compatibility symbols.
- `legacy/workflow_control.rs` exposes archived read-model and workflow state compatibility symbols.
- `legacy/run_verify_review.rs` exposes archived plan / run / verify / review compatibility symbols.
- `legacy/eligibility_lease.rs` exposes archived eligibility and lease compatibility symbols.
- `legacy/project_closure.rs` exposes archived closure state compatibility symbols.
- `legacy/project_audit_docs_refresh.rs` exposes archived audit and docs refresh compatibility symbols.
- `legacy/saved_view.rs` exposes archived saved-view compatibility symbols.
- `legacy/sqlite_index.rs` exposes archived SQLite index compatibility symbols.

Removed in 005:

- the crate-root `pub use legacy::*` compatibility export;
- the `legacy/mod.rs` `pub use archive_2026_05::*` compatibility export;
- the public `legacy/evidence.rs` compatibility module, because no active read model, CLI legacy command, Desktop command, or non-archive source imported it.

Detailed reachability and removal classification lives in `docs/architecture/legacy-removal-audit.md`.

## Desktop Transitional Read Models

Desktop still needs read-only data to render the current UI. These APIs are transitional, not new workflow authorization:

- `read_desktop_workbench_snapshot`
- `read_local_metrics_snapshot`
- `read_local_project_model_snapshot`
- `read_project_milestone_issue_view_model_snapshot`
- `read_local_search_snapshot`
- `WorkbenchBoundary`

They may remain available through `active/` wrappers while the new Goal Tree model is still undefined.

## Legacy Data Paths

The following paths are historical compatibility data locations:

- `.agentflow/issues/`
- `.agentflow/runs/`
- `.agentflow/evidence/`
- `.agentflow/reviews/`
- `.agentflow/updates/`
- `.agentflow/state/`
- `.agentflow/views/`
- `.agentflow/index.db`
- `.agentflow/index.json`

New requirements should introduce new data paths explicitly instead of implicitly reusing these locations.

## Removal Rule

Do not delete legacy code or data handling only because it is old. It can be removed only when:

- no current command, desktop screen, test, or compatibility reader imports it;
- the replacement requirement is explicit;
- `cargo test`, desktop build, and relevant smoke checks pass.

005 deletion result:

- deleted unused public compatibility exposure, not active behavior;
- retained CLI legacy commands temporarily;
- retained active Desktop read models;
- retained Graph watcher fallback;
- retained Project File Reader fallback and browser-preview mock data.
