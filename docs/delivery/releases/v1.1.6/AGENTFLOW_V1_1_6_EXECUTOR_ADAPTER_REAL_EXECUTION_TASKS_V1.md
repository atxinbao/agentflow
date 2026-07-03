# AgentFlow v1.1.6 Executor Adapter Real Execution Tasks

更新日期：2026-07-03
执行者：Codex

## Purpose

This document records the public delivery traceability for `v1.1.6`.

`v1.1.6` closes the Executor Adapter Real Execution path. It keeps `.agentflow/spec/issues/**` as task authority, generates stable executor handoff packages, checks allowed surfaces before writeback, captures execution evidence, normalizes issue/run statuses and certifies the Software Dev real executor golden path.

## Task Traceability

| Task | GitHub Issue | Title | Status | Release Gate Artifact |
| --- | --- | --- | --- | --- |
| V116-001 | #808 | Next Release Authority Alignment | done | `runtime/v116-next-release-authority-alignment.json` |
| V116-002 | #809 | Core Route Next-action Semantics Hardening | done | `runtime/v116-core-route-next-action-semantics.json` |
| V116-003 | #810 | Product Spec Intake Desktop Invocation Bridge | done | `runtime/v116-product-spec-intake-desktop-invocation-bridge.json` |
| V116-004 | #811 | Executor Adapter Handoff Package | done | `runtime/v116-executor-adapter-handoff-package.json` |
| V116-005 | #812 | Allowed Surface and Diff Boundary Check | done | `runtime/v116-allowed-surface-diff-boundary-check.json` |
| V116-006 | #813 | Executor Evidence Capture | done | `runtime/v116-executor-evidence-capture.json` |
| V116-007 | #814 | Executor Result to Issue / Run Status Writeback | done | `runtime/v116-executor-result-issue-run-writeback.json` |
| V116-008 | #815 | Failure / Timeout / Cancel / Retry Semantics | done | `runtime/v116-failure-timeout-cancel-retry-semantics.json` |
| V116-009 | #816 | Software Dev Real Executor Golden Path | done | `runtime/v116-software-dev-real-executor-golden-path.json` |
| V116-010 | #817 | v1.1.6 Release Certification | done | `runtime/v116-release-certification.json` |

## Acceptance Contract

`v1.1.6` is accepted only when:

1. `Cargo.toml`, `Cargo.lock`, Desktop package metadata and Tauri metadata are `1.1.6`;
2. `CHANGELOG.md` contains the `v1.1.6` entry;
3. `docs/delivery/README.md` points to `v1.1.6` as current baseline;
4. route next-actions prove `clarify` and `research` cannot confirm or materialize;
5. Desktop has runtime command bridge coverage for Spec Intake and Executor Adapter actions;
6. executor handoff packages are written under run-scoped launch artifacts;
7. out-of-scope diffs are blocked before writeback;
8. evidence capture binds handoff refs, command results and diff boundary reports;
9. writeback can only mark an issue/run complete after evidence and boundary checks pass;
10. failure, timeout, cancel and retry receipts are durable and retry creates a new run;
11. Software Dev golden path proves Spec Issue -> handoff -> executor result -> evidence -> boundary -> writeback -> projection;
12. release-gate publishes `runtime/v116-release-certification.json` with all V116 artifacts passed.

## Boundaries

- GitHub issues are traceability records, not AgentFlow task authority.
- Executor session transcripts are not completion authority.
- Provider launch supervision remains outside this release.
- Desktop task page redesign remains outside this release.
