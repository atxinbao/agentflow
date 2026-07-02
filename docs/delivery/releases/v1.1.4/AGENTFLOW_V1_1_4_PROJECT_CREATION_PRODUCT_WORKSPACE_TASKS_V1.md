# AgentFlow v1.1.4 Project Creation and Product Workspace Tasks

更新日期：2026-07-02
执行者：Codex

## Purpose

This document records the public delivery traceability for `v1.1.4`.

`v1.1.4` moves AgentFlow from Product command submission into Product-selected project workspace creation. It certifies dry-run receipt binding, Desktop confirm-submit behavior, generic Product bridge boundaries, Product workspace initialization, active workspace projection, failure / duplicate / recovery states and the Software Dev default workspace golden path.

## Task Traceability

| Task | GitHub Issue | Title | Status | Release Gate Artifact |
| --- | ---: | --- | --- | --- |
| V114-001 | #785 | Product Submit Receipt Binding | 状态：done | `runtime/v114-product-submit-receipt-binding.json` |
| V114-002 | #786 | Desktop Confirm-submit Interaction Proof | 状态：done | `runtime/v114-desktop-confirm-submit-interaction.json` |
| V114-003 | #787 | Product Bridge Semantic Pollution Scanner | 状态：done | `runtime/v114-product-bridge-semantic-pollution-scan.json` |
| V114-004 | #788 | Project Workspace Creation Contract | 状态：done | `runtime/v114-project-workspace-creation-contract.json` |
| V114-005 | #789 | Product-selected Workspace Bootstrap | 状态：done | `runtime/v114-product-selected-workspace-bootstrap.json` |
| V114-006 | #790 | Standard Docs and AgentFlow Fact Source Initialization | 状态：done | `runtime/v114-standard-docs-agentflow-fact-source-init.json` |
| V114-007 | #791 | Active Product Workspace State and Projection | 状态：done | `runtime/v114-active-product-workspace-projection.json` |
| V114-008 | #792 | Workspace Init Failure / Duplicate / Recovery | 状态：done | `runtime/v114-workspace-init-failure-recovery.json` |
| V114-009 | #793 | Software Dev Default Workspace Golden Path | 状态：done | `runtime/v114-software-dev-workspace-golden-path.json` |
| V114-010 | #794 | v1.1.4 Release Certification | 状态：done | `runtime/v114-release-certification.json` |

## Acceptance

`v1.1.4` is accepted only when:

1. workspace, Desktop and Tauri versions are `1.1.4`;
2. `CHANGELOG.md` contains the `v1.1.4` entry;
3. `AGENTS.md`, `docs/README.md` and `docs/delivery/README.md` point to this release baseline;
4. all release gate artifacts listed above are present and have `status: passed`;
5. Product bridge semantic pollution scan passes;
6. Project workspace duplicate, partial, invalid-root and missing-product states are certified;
7. GitHub issues `#785` through `#794` are closed through the release PR.

## Non-goals

- Do not introduce marketplace installation.
- Do not make Desktop bypass Runtime API submit.
- Do not treat Product-specific constants as Core bridge authority.
- Do not write runtime facts outside the Product workspace initialization contract.
