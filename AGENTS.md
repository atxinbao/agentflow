# AGENTS.md - AgentFlow Source Agent Entry

日期：2026-07-05
执行者：Codex

## Purpose

This is the stable Agent entry included in the release source archive.
It points agents to tracked documentation. It is not a local runtime fact file.

## Read First

1. Project goal: `docs/project/goal.md`
2. Project roadmap: `docs/project/roadmap.md`
3. Project docs map: `docs/project/README.md`
4. Project context: `docs/project/context.md`
5. Core capabilities: `docs/architecture/021-ai-os-project-core-capabilities-v1.md`
6. Built-in Pack registry boundary: `docs/architecture/builtin-pack-registry.md`
7. Docs map: `docs/README.md`
8. Architecture contracts: `docs/architecture/README.md`
9. Current module boundaries: `docs/architecture/current-module-boundaries.md`
10. Current stable contract baseline: `docs/architecture/041-v100-stable-contract-baseline-v1.md`
11. Release certification boundary: `docs/architecture/050-v100-release-certification-v1.md`
12. Current release baseline: `docs/delivery/releases/v1.2.3/README.md`
13. Previous release baseline: `docs/delivery/releases/v1.2.2/README.md`
14. Previous release baseline: `docs/delivery/releases/v1.2.1/README.md`
15. Recent release trace: `docs/delivery/releases/v1.2.0/README.md`
16. Recent release trace: `docs/delivery/releases/v1.1.9/README.md`
17. Recent release trace: `docs/delivery/releases/v1.1.8/README.md`
18. Recent release trace: `docs/delivery/releases/v1.1.7/README.md`
19. Recent release trace: `docs/delivery/releases/v1.1.6/README.md`
20. Historical release trace: `docs/delivery/releases/v1.1.5/README.md`
21. Historical release trace: `docs/delivery/releases/v1.1.4/README.md`
22. Historical release trace: `docs/delivery/releases/v1.1.3/README.md`
23. Historical release trace: `docs/delivery/releases/v1.1.2/README.md`
24. Historical archive: `docs/project/history/2026-06-current-baseline-history/README.md`

## Authority Boundary

- `docs/project/**` defines product direction.
- `docs/project/roadmap.md` defines the version route from goal to requirements, not executable issues.
- `docs/architecture/**` defines AI OS Project core capabilities.
- Built-in Pack definitions are App internal capabilities, not `docs/project/**` authority.
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
