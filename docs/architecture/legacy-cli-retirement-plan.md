# Legacy CLI Retirement Plan

创建日期：2026-06-02  
执行者：Codex

## Purpose

This document implements `006 - Legacy CLI Retirement and Archive Pruning`.

The old 2026-05 CLI command names still parse, but old write/automation
implementations are no longer executed. The current active CLI surface keeps
only temporary read-only inspection commands until the new Goal Tree / AgentRun
workflow is defined by a future requirement.

## Policy

Every legacy CLI command is classified as one of:

| Disposition | Meaning |
| --- | --- |
| `keep-temporary` | Still allowed because it is read-only and supports current verification. |
| `hide-from-help` | Reserved for future migration; not used in code in 006. |
| `disable-with-message` | Command parses, prints a retirement message, and performs no work. |
| `delete` | Ready for source removal when no tests or active read models depend on it. |
| `defer-until-goal-tree` | Concept may return later, but the old implementation is not inherited. |

Runtime disabled-message invariant:

```text
This command belongs to the archived 2026-05 AgentFlow workflow.
It is disabled in the new requirements track.
The new Goal Tree / AgentRun workflow has not been defined yet.
No files were written and no command was executed.
```

## Active Temporary Commands

| CLI command | Classification | Current behavior |
| --- | --- | --- |
| `metrics` | `keep-temporary` | Reads the transitional local metrics snapshot. |
| `projects` | `keep-temporary` | Reads the transitional local project model snapshot. |
| `search` | `keep-temporary` | Reads the transitional local search snapshot. |

These commands are read-only compatibility paths. They are not new product
workflow authorization.

## Retired Command Matrix

| CLI command | Classification | Runtime behavior | Reason |
| --- | --- | --- | --- |
| `init` | `defer-until-goal-tree` | disabled message | Old goal bootstrap is not the new Project Workspace flow. |
| `goal bootstrap/check/next` | `disable-with-message` | disabled message | Old GoalLoop is not inherited. |
| `feature create/status/next` | `disable-with-message` | disabled message | Old Product Feature flow is not inherited. |
| `team create` | `disable-with-message` | disabled message | Old Team writer is not authorized. |
| `project create` | `disable-with-message` | disabled message | Old Project writer is not authorized. |
| `project closure` | `disable-with-message` | disabled message | Old Project Closure is not inherited. |
| `project code-audit` | `disable-with-message` | disabled message | Old Code Audit flow is not inherited. |
| `project docs-refresh` | `disable-with-message` | disabled message | Old Docs Refresh flow is not inherited. |
| `milestone create` | `disable-with-message` | disabled message | Old Milestone writer is not authorized. |
| `issue create` | `disable-with-message` | disabled message | Old IssueContract writer is not authorized. |
| `context` | `delete` | disabled message | Graph read models replace the old context direction. |
| `plan` | `disable-with-message` | disabled message | Old issue planning is not authorized. |
| `run` | `disable-with-message` | disabled message | New AgentRun has not been defined. |
| `verify` | `disable-with-message` | disabled message | Old verification writer is not authorized. |
| `review` | `disable-with-message` | disabled message | Old review/evidence writer is not authorized. |
| `eligibility` | `disable-with-message` | disabled message | Old eligibility snapshot writer is not inherited. |
| `lease` | `disable-with-message` | disabled message | Old lease snapshot writer is not inherited. |
| `index rebuild` | `disable-with-message` | disabled message | Old SQLite index writer is not authorized. |
| `view save/show` | `disable-with-message` | disabled message | Old saved-view writer is not authorized. |
| `update summary` | `disable-with-message` | disabled message | Old project summary writer is not authorized. |
| `project-seed` | `disable-with-message` | disabled message | Old local project seed writer is not authorized. |
| `issue-link` | `disable-with-message` | disabled message | Old issue link writer is not authorized. |
| `review-assistant` | `disable-with-message` | disabled message | Old review assistant writer is not authorized. |
| `state check` | `disable-with-message` | disabled message | Old workflow state writer is not inherited. |

## Code Boundary

Implemented CLI files:

| File | Responsibility |
| --- | --- |
| `crates/cli/src/args.rs` | Preserves legacy command parsing. |
| `crates/cli/src/retirement.rs` | Classifies each legacy command and prints the retirement message. |
| `crates/cli/src/legacy.rs` | Runs only temporary read-only commands and gates all retired commands. |
| `crates/cli/src/active.rs` | Placeholder active CLI boundary; no write commands authorized in 006. |
| `crates/cli/src/print.rs` | Retired output-helper boundary kept for module clarity. |

## Archive Pruning Result

The named legacy modules no longer publicly re-export old writer entrypoints
for:

- Goal protocol writers;
- Product Feature writers/readers;
- Team / Project / Milestone / Issue writers;
- plan / context / run / verify / review writers;
- SQLite index writer;
- saved view writer;
- eligibility / lease writers;
- closure / audit / docs-refresh writers;
- workflow state writer.

The private `archive_2026_05.rs` implementation is retained in 006 because its
old internal tests still exercise those functions and the active read-model
wrappers still rely on some archived DTO shapes.

## Non-goals

- No Goal Tree implementation.
- No AgentRun implementation.
- No model invocation.
- No command execution against user projects.
- No Desktop UI behavior change.
- No Tauri command rename.
- No `.agentflow/` runtime writes.
