# AGENTS.md - AgentFlow Source Agent Entry

日期：2026-06-26
执行者：Codex

## Purpose

This is the stable Agent entry included in the release source archive.
It points agents to tracked documentation. It is not a local runtime fact file.

## Read First

1. Project goal: `docs/product/006-spec-driven-software-dev-product-goal-v1.md`
2. Core capabilities: `docs/foundation/021-ai-os-project-core-capabilities-v1.md`
3. Docs map: `docs/README.md`
4. Architecture contracts: `docs/architecture/README.md`
5. Current release baseline: `docs/v1.0.1/README.md`
6. Historical archive: `docs/archive/2026-06-current-baseline-history/README.md`

## Authority Boundary

- `docs/product/**` defines product direction.
- `docs/foundation/**` defines AI OS Project core capabilities.
- `docs/requirements/**` stores confirmed Spec Bundles.
- `.agentflow/spec/**` is the execution contract fact source.
- `.agentflow/events/**` is the task event stream.
- `.agentflow/tasks/<issue-id>/**` stores local run and evidence facts.

Do not treat GitHub issues, Codex threads, local chat history, or archived docs as AgentFlow task authority.

## Runtime Boundary

Do not commit local runtime records as release source:

- `.agentflow/runs/**`
- `.agentflow/tmp/**`
- `.agentflow/tasks/**`
- `.agentflow/index.sqlite`
- `.agentflow/index.sqlite-*`

Tracked documentation owns the source entry. Materialized `.agentflow/define/agent/**` files are runtime outputs and must not replace this file.
