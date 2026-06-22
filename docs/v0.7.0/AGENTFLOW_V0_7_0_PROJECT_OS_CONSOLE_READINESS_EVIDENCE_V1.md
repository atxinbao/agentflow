# AgentFlow v0.7.0 Project OS Console Readiness Evidence V1

Date: 2026-06-22  
Executor: Codex

## Conclusion

`v0.7.0` 的 Project OS Console 已具备软件开发场景的只读可解释闭环：

```text
Project
-> Spec
-> Task
-> Work
-> Acceptance
-> Delivery
-> Audit read-only
-> Command Surface
```

Console 只消费 projection / view model / browser preview facts，不直接写事实源。

## Covered Surfaces

| Surface | Readiness proof |
| --- | --- |
| Project Home | `ProjectHomeViewModel` exposes project identity, stage, issue counts, current issue, and command readiness. |
| Spec Workbench | `SpecWorkbenchViewModel` exposes stages, preview issues, authority layers, and runtime action proposals. |
| Task Workbench | `TaskWorkbenchViewModel` exposes project tree, selected issue, state counts, timeline states, evidence graph, and command surface readiness. |
| Event Timeline / Evidence Graph | Task projection timeline and artifact refs are mapped into read-only view models. |
| Acceptance / Delivery / Audit | `AcceptanceDeliveryAuditViewModel` exposes acceptance state, delivery state, audit state, evidence count, public record count, and findings count. |
| Command Surface | Command readiness remains a Runtime API / Action Proposal bridge, not a fact writer. |

## State Coverage

The readiness check covers:

- `ready`
- `done`
- `missing`
- `stale`
- `conflict`

The broader Browser Preview smoke also covers loading-capable shell paths, empty project registry, task empty states, current / past / future task timeline boundaries, audit read-only states, and runtime diagnostics.

## Validation Commands

Formal local validation for this issue:

```bash
npm --prefix apps/desktop run build
npm --prefix apps/desktop run preview:smoke
npm --prefix apps/desktop run console:readiness
git diff --check
```

## Non-authority Boundary

The following remain read-only presentation layers:

- Desktop View Models
- Browser Preview data
- Project OS Console
- Command Surface labels
- Advanced Runtime Diagnostics

Fact writes must continue to go through runtime APIs, accepted action proposals, or existing authority files. The Console must not become a new authority source.
