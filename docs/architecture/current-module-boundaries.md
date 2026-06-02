# Current Module Boundaries

创建日期：2026-06-02  
执行者：Codex

## Purpose

This document records the intended post-004 module boundaries for the current AgentFlow codebase. It is the implementation map for `004 - Legacy Cleanup and New Module Split`.

## Project Workspace Manager

Scope:

- prepare local project workspace metadata;
- create or reuse `.agentflow/`;
- create or reuse workspace config files;
- protect `.agentflow/` through local Git exclude handling;
- deduplicate local projects by canonical path / Git root / workspace identity.

Non-goals:

- no command execution;
- no model invocation;
- no remote object creation;
- no deletion of source projects.

Implemented backend modules:

- `apps/desktop/src-tauri/src/project_workspace/commands.rs`
- `apps/desktop/src-tauri/src/project_workspace/model.rs`
- `apps/desktop/src-tauri/src/project_workspace/prepare.rs`
- `apps/desktop/src-tauri/src/project_workspace/dedupe.rs`
- `apps/desktop/src-tauri/src/project_workspace/git.rs`
- `apps/desktop/src-tauri/src/project_workspace/ignore.rs`
- `apps/desktop/src-tauri/src/project_workspace/remove.rs`

Notes:

- `commands.rs` is the Tauri command wrapper.
- `prepare.rs`, `git.rs`, `ignore.rs`, and `model.rs` contain current behavior.
- `dedupe.rs` and `remove.rs` are explicit boundaries for future requirements and do not add behavior in 004.

## Graph

Scope:

- project file / symbol / relation index;
- graph status and manifest;
- context pack;
- preflight;
- weak impact and test recommendation;
- OS native watcher with fallback and degraded status.

Non-goals:

- no Agent execution;
- no test execution;
- no model calls;
- no writes outside `.agentflow/output/graph/`.

Implemented watcher modules:

- `crates/graph/src/watcher/mod.rs`
- `crates/graph/src/watcher/native.rs`
- `crates/graph/src/watcher/fallback.rs`
- `crates/graph/src/watcher/filter.rs`
- `crates/graph/src/watcher/state.rs`
- `crates/graph/src/watcher/debounce.rs`

## Project File Reader

Scope:

- read-only local file browser;
- file and directory content preview;
- directory pagination;
- search and quick open;
- text range loading;
- renderer selection for code, Markdown, config, media, PDF, DOCX, and fallback states.

Non-goals:

- no file writes;
- no command execution;
- no source edits;
- no model calls.

Implemented backend modules:

- `apps/desktop/src-tauri/src/project_files/commands.rs`
- `apps/desktop/src-tauri/src/project_files/model.rs`
- `apps/desktop/src-tauri/src/project_files/path_guard.rs`
- `apps/desktop/src-tauri/src/project_files/directory.rs`
- `apps/desktop/src-tauri/src/project_files/content.rs`
- `apps/desktop/src-tauri/src/project_files/search.rs`
- `apps/desktop/src-tauri/src/project_files/range.rs`
- `apps/desktop/src-tauri/src/project_files/mime.rs`

Implemented frontend modules:

- `apps/desktop/src/features/project-files/browser/`
- `apps/desktop/src/features/project-files/reader/`
- `apps/desktop/src/features/project-files/hooks/`
- `apps/desktop/src/features/project-files/model/`

Hook boundaries:

- `hooks/useProjectFiles.ts` remains the coordinator and preserves the public UI API.
- `hooks/useProjectDirectoryPages.ts` owns directory pagination loading.
- `hooks/useProjectFileSearch.ts` owns search state and search command dispatch.
- `hooks/useProjectFileTextRange.ts` owns large text range loading.
- `hooks/projectFileRuntime.ts` owns browser-preview detection and readable error text.

## Legacy Core Read Model

Scope:

- keep current Desktop read-only views rendering while the new workflow model is undefined;
- expose transitional snapshots only through explicit compatibility wrappers.

Non-goals:

- no new write flows;
- no new Goal / Milestone / Issue execution model;
- no new AgentRun semantics.

Implemented modules:

- `crates/agentflow-core/src/active/`
- `crates/agentflow-core/src/legacy/`
- `crates/agentflow-core/src/shared/`

Active read-model wrappers:

- `active/desktop_snapshot.rs`
- `active/local_metrics.rs`
- `active/local_project_model.rs`
- `active/local_search.rs`
- `active/boundary.rs`

Legacy compatibility areas:

- `legacy/archive_2026_05.rs` keeps the archived implementation for compatibility and is private to `legacy`.
- `legacy/goal_protocol.rs`
- `legacy/product_feature.rs`
- `legacy/team_project_milestone_issue.rs`
- `legacy/workflow_control.rs`
- `legacy/run_verify_review.rs`
- `legacy/eligibility_lease.rs`
- `legacy/project_closure.rs`
- `legacy/project_audit_docs_refresh.rs`
- `legacy/saved_view.rs`
- `legacy/sqlite_index.rs`

005 boundary tightening:

- `agentflow-core` no longer exposes `pub use legacy::*` at the crate root.
- `legacy/mod.rs` no longer exposes `pub use archive_2026_05::*`.
- `legacy/evidence.rs` was removed because it had no active/CLI/Desktop import.
- CLI legacy dispatch imports named compatibility modules explicitly.
- Desktop legacy Tauri commands import only `agentflow_core::active` read models.

Shared neutral boundaries:

- `shared/fs_paths.rs`
- `shared/json_io.rs`
- `shared/markdown.rs`
- `shared/ids.rs`
- `shared/time.rs`

## Legacy CLI

Scope:

- preserve old command names temporarily;
- isolate old command implementations into `legacy.rs`;
- keep argument definitions separate from command dispatch.

Non-goals:

- no renaming to `agentflow legacy ...` in this slice;
- no new product command surface;
- no command behavior change.

Implemented modules:

- `crates/agentflow-cli/src/args.rs`
- `crates/agentflow-cli/src/legacy.rs`
- `crates/agentflow-cli/src/print.rs`
