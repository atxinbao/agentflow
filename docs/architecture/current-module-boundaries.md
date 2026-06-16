# Current Module Boundaries

创建日期：2026-06-02  
最后更新：2026-06-14
执行者：Codex

## Purpose

This document records the active module boundaries for AgentFlow.

The current architecture is moving from the older `input / execute / output /
state` directory pipeline to a task-centered runtime:

```text
docs/requirements
  -> .agentflow/spec
  -> .agentflow/workflows
  -> .agentflow/events
  -> .agentflow/projections
  -> .agentflow/tasks
```

The implementation baseline is
`docs/requirements/034-agentflow-task-workflow-yaml-runtime-v1.md`.

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

## MCP Provider Adapter

Scope:

- external provider health and capability discovery;
- provider launch plan generation;
- provider session snapshot persistence;
- provider session polling and log tail;
- provider-specific command mapping for GitHub, GitLab, Codex, Browser Preview, and future external coding agents.

Non-goals:

- no ownership of Project Loop or Issue Loop state transitions;
- no ownership of task artifacts, validation evidence, or public delivery records;
- no external provider authority over AgentFlow task ordering.

Implemented backend modules:

- `crates/mcp/src/model.rs`
- `crates/mcp/src/provider.rs`
- `crates/mcp/src/registry.rs`
- `crates/mcp/src/storage.rs`
- `crates/mcp/src/github.rs`
- `crates/mcp/src/gitlab.rs`
- `crates/mcp/src/codex.rs`
- `crates/mcp/src/browser.rs`
- `crates/mcp/src/events.rs`
- `crates/mcp/src/health.rs`
- `crates/mcp/src/error.rs`

Notes:

- `crates/mcp` is the provider adapter layer for the current codebase.
- `task-loop` decides whether an issue can launch; `mcp` defines how a provider is probed and how a provider session is represented.
- `mcp` reads task/session state from `projection` and appends provider lifecycle facts to `event-store`.
- `mcp` no longer depends on the legacy `input`, `execute`, or `workflow-events` crates.
- `mcp` can append provider lifecycle facts such as `agent.session.running` and `agent.session.failed`, but it still does not own issue status transitions.
- Desktop reads provider sessions through `apps/desktop/src-tauri/src/commands/mcp.rs`, including snapshot polling and session log chunks.

## Spec

Scope:

- read public requirement records from `docs/requirements/<requirement-id>.md`;
- manage internal task contracts under `.agentflow/spec/projects/**` and `.agentflow/spec/issues/**`;
- validate issue fields, priority, dependency links, workflow references, allowed paths, forbidden paths, and expected outputs.

Non-goals:

- no runtime status transitions;
- no event log writes;
- no task execution;
- no provider launch;
- no UI projection generation.

Implemented backend modules:

- `crates/spec/src/model.rs`
- `crates/spec/src/storage.rs`
- `crates/spec/src/lib.rs`

Notes:

- `spec` is the replacement target for the old `input` task fact source.
- `spec` owns task contracts only. Runtime facts must be written as task events.

## Workflow Core

Scope:

- parse and validate task workflow YAML;
- define workflow states, transitions, guards, actions, and terminal states;
- keep workflow definitions independent from any project or issue instance.

Non-goals:

- no issue loading;
- no event store writes;
- no action execution;
- no Desktop-facing projection.

Implemented backend modules:

- `crates/workflow-core/src/model.rs`
- `crates/workflow-core/src/parser.rs`
- `crates/workflow-core/src/validation.rs`
- `crates/workflow-core/src/lib.rs`

Notes:

- Workflow YAML describes allowed transitions. It does not run shell commands and does not become a CI system.

## Event Store

Scope:

- append task events to `.agentflow/events/task-events.jsonl`;
- provide deterministic event IDs, idempotency keys, correlation IDs, and replay;
- import old workflow events only when explicitly requested by migration code.

Non-goals:

- no UI projection generation;
- no status decision logic;
- no task execution;
- no provider calls.

Implemented backend modules:

- `crates/event-store/src/model.rs`
- `crates/event-store/src/storage.rs`
- `crates/event-store/src/lib.rs`

Notes:

- Event log is the runtime fact source.
- Projection can be rebuilt from this log.

## Workflow Runtime

Scope:

- read workflow definition and current projection;
- match incoming events to allowed transitions;
- run registered guards and actions;
- append state transition events;
- reject illegal state jumps.

Non-goals:

- no arbitrary shell execution from YAML;
- no public release writing;
- no provider implementation;
- no Desktop-specific rendering.

Implemented backend modules:

- `crates/workflow-runtime/src/runtime.rs`
- `crates/workflow-runtime/src/lib.rs`

Notes:

- Runtime owns state-machine correctness. Build Agent, Desktop, and external providers must not mutate issue status directly.

## Task Artifacts

Scope:

- manage `.agentflow/tasks/<issue-id>/runs/<run-id>/**`;
- manage `.agentflow/tasks/<issue-id>/evidence/**`;
- record command output, validation output, checkpoints, plans, and local evidence.

Non-goals:

- no `.agentflow/tasks/<issue-id>/delivery/**`;
- no PR/MR body writes;
- no CHANGELOG or release notes writes;
- no issue scheduling.

Implemented backend modules:

- `crates/task-artifacts/src/model.rs`
- `crates/task-artifacts/src/storage.rs`
- `crates/task-artifacts/src/lib.rs`

Notes:

- Local `.agentflow` runtime artifacts end at evidence. Public delivery records live in PR/MR bodies and later release notes.

## Task Loop

Scope:

- read spec projects and issues;
- sort issues by dependencies, priority, and issue number;
- schedule the next eligible issue by appending task events;
- emit launch requests for Agent Dispatcher.

Non-goals:

- no code execution;
- no provider-specific launch mechanics;
- no direct Desktop rendering;
- no public release writing.

Implemented backend modules:

- `crates/task-loop/src/model.rs`
- `crates/task-loop/src/scheduler.rs`
- `crates/task-loop/src/launcher.rs`
- `crates/task-loop/src/loop_runtime.rs`
- `crates/task-loop/src/lib.rs`

Notes:

- Project Loop is a scheduler, not an executor.
- User-triggered project loop buttons should call this layer instead of the old `loop` crate.

## Agent Dispatcher

Scope:

- consume `agent.launch.requested` events;
- create external agent sessions through `mcp` provider adapters;
- translate session creation and launch-time status into task events;
- coordinate with `mcp` provider adapters.

Non-goals:

- no task ordering;
- no issue status mutation outside workflow events;
- no direct code editing;
- no release note generation.

Implemented backend modules:

- `crates/agent-dispatcher/src/lib.rs`

Notes:

- Agent Dispatcher is the session orchestrator. It does not decide what task should run next.

## Projection

Scope:

- rebuild task projections from task events;
- rebuild project projections from task events and spec contracts;
- generate issue-status indexes for the task page;
- provide Desktop read models.

Non-goals:

- no spec issue writes;
- no event writes except explicit rebuild markers when needed;
- no provider calls;
- no local command execution.

Implemented backend modules:

- `crates/projection/src/model.rs`
- `crates/projection/src/projector.rs`
- `crates/projection/src/storage.rs`
- `crates/projection/src/lib.rs`

Notes:

- Desktop should read projection and indexes instead of reading old `input`, `execute`, `output`, or `state` files directly.

## Release

Scope:

- collect public delivery records from completed task projections and PR/MR metadata;
- generate CHANGELOG and release notes for version-level delivery;
- write public release documents when explicitly invoked.

Non-goals:

- no single-task status transitions;
- no Build Agent loop execution;
- no local `.agentflow/output/**` replacement;
- no audit decision making.

Implemented backend modules:

- `crates/release/src/model.rs`
- `crates/release/src/summary.rs`
- `crates/release/src/writer.rs`
- `crates/release/src/lib.rs`

Notes:

- Release is a batch public-record layer, not the owner of task delivery.

## Legacy CLI Read Model

Scope:

- keep temporary CLI read-only inspection commands working while current product flows move to input / panel / execute / output / state modules;
- expose transitional snapshots only through explicit compatibility wrappers.

Non-goals:

- no new write flows;
- no new Goal / Milestone / Issue execution model;
- no new AgentRun semantics.

Implemented modules:

- `crates/core/src/active/`
- `crates/core/src/legacy/`
- `crates/core/src/shared/`

Active read-model wrappers:

- `active/local_metrics.rs`
- `active/local_project_model.rs`
- `active/local_search.rs`
- `active/boundary.rs`

Legacy compatibility areas:

- `legacy/archive_2026_05.rs` keeps the archived implementation for compatibility and is private to `legacy`.
- `legacy/team_project_milestone_issue.rs`
- `legacy/workflow_control.rs`

005 boundary tightening:

- `agentflow-core` no longer exposes `pub use legacy::*` at the crate root.
- `legacy/mod.rs` no longer exposes `pub use archive_2026_05::*`.
- `legacy/evidence.rs` was removed because it had no active/CLI/Desktop import.
- CLI legacy dispatch no longer imports old writer compatibility modules.
- Desktop no longer registers legacy read-model Tauri commands; tasks come from spec contracts and task projections.

Shared neutral boundaries:

- `shared/fs_paths.rs`
- `shared/json_io.rs`
- `shared/markdown.rs`
- `shared/ids.rs`
- `shared/time.rs`

## Legacy CLI

Scope:

- preserve old command names only as migration-visible parse targets;
- disable archived 2026-05 write/automation commands with a clear retirement message;
- keep only temporary read-only inspection commands: `metrics`, `projects`, `search`;
- keep argument definitions separate from command retirement policy and dispatch.

Non-goals:

- no Goal Tree command surface in 006;
- no AgentRun command surface in 006;
- no old workflow writes through CLI;
- no `.agentflow/` runtime writes through retired commands.

Implemented modules:

- `crates/cli/src/args.rs`
- `crates/cli/src/retirement.rs`
- `crates/cli/src/legacy.rs`
- `crates/cli/src/active.rs`
- `crates/cli/src/print.rs`

006 boundary tightening:

- `legacy.rs` imports only `agentflow_core::active` read-only snapshots.
- Retired old writer commands print a message and return success without writing files.
- Named legacy core modules no longer publicly re-export old writer entrypoints unless active read models need DTOs.
- `docs/architecture/legacy-cli-retirement-plan.md` records per-command disposition.
