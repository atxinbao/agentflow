# v0.18.0 Decision Record

更新日期：2026-06-28
执行者：Codex
状态：accepted

## Decision

Accept `Core 4-D Spec Intake` as the generic Core Spec Kernel entry.

## Rationale

The current project goal defines AgentFlow as:

```text
Spec-Driven AI OS Project
```

Therefore Core intake cannot be Software Dev-only. It must define generic stages, slices, routes, artifact boundaries, and industry mapping fixtures.

## Boundary

Accepted:

- generic 4-D stages；
- generic route policy；
- generic Spec Bundle slices；
- Draft / Preview / Confirmed / Materialized boundary；
- cross-industry reference fixtures。

Rejected:

- making Software Dev objects Core authority；
- treating GitHub issues as AgentFlow authority；
- materializing unconfirmed preview；
- writing runtime facts during this documentation/contract slice。

## Consequence

Future Spec Loop work must build on `core_intake` rather than adding more Software Dev-only intake concepts to Core.
