# Legacy Code Map

创建日期：2026-06-02  
执行者：Codex

## Purpose

This document records the compatibility surface that still belongs to the archived 2026-05 AgentFlow workflow. It supports `004 - Legacy Cleanup and New Module Split` and `005 - Legacy and Degraded Code Removal` by making old code visible before it is quarantined, narrowed, or removed.

## Legacy Command Surface

The following CLI commands are legacy. As of 006, only the temporary read-only
commands remain executable:

```text
metrics
projects
search
```

All other old CLI commands parse only to show the retirement message:

```text
This command belongs to the archived 2026-05 AgentFlow workflow.
It is disabled in the new requirements track.
The new Goal Tree / AgentRun workflow has not been defined yet.
No files were written and no command was executed.
```

Retired commands:

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

The command names are currently preserved for explicit migration feedback. New
product flows must not be added to this legacy command surface. Detailed
classification lives in `docs/architecture/legacy-cli-retirement-plan.md`.

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

- `crates/core/src/lib.rs` no longer re-exports `legacy::*`.
- `crates/core/src/legacy/archive_2026_05.rs` contains the archived implementation and is private to the `legacy` module.
- `legacy/team_project_milestone_issue.rs` exposes only the temporary CLI `projects` read model.
- `legacy/workflow_control.rs` exposes only the temporary CLI `metrics` and `search` read models.

Removed in 005:

- the crate-root `pub use legacy::*` compatibility export;
- the `legacy/mod.rs` `pub use archive_2026_05::*` compatibility export;
- the public `legacy/evidence.rs` compatibility module, because no active read model, CLI legacy command, Desktop command, or non-archive source imported it.

Retired in 006:

- CLI write/automation dispatch for all old commands except `metrics`, `projects`, and `search`;
- public compatibility re-exports for old legacy writer entrypoints;
- old CLI output helpers that formatted retired writer summaries.

Detailed reachability and removal classification lives in `docs/architecture/legacy-removal-audit.md`.

## Desktop Legacy Read Models

Desktop no longer reads legacy workbench or Project/Milestone/Issue snapshots. The
current task list is derived from `.agentflow/input/issues/**` through the input
snapshot and state status index.

Removed from Desktop runtime in the current cleanup slice:

- `apps/desktop/src-tauri/src/commands/legacy_core.rs`
- Tauri command registration for `load_workbench_snapshot`
- Tauri command registration for `load_project_milestone_issue_view_model_snapshot`
- Browser Preview workbench / metrics / project-model / search mock snapshots
- frontend fallback that converted old `IssueContract` records into current task rows

The remaining active legacy read models are CLI-only temporary inspection paths:

- `read_local_metrics_snapshot`
- `read_local_project_model_snapshot`
- `read_local_search_snapshot`

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

Current deletion result:

- deleted unused public compatibility exposure, not active behavior;
- retired CLI legacy write/automation commands and kept only temporary read-only commands;
- removed Desktop legacy read-model commands and task fallback;
- retained Graph watcher fallback;
- retained Project File Reader fallback and current Browser Preview mock data.
