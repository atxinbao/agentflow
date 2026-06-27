# v0.18.0 Issue Preview

更新日期：2026-06-28
执行者：Codex
状态：confirmed issue preview

## Dependency Order

```text
#618
-> #619
-> #620
-> (#621, #622)
-> #623
-> (#624, #625)
-> #626
-> #627
```

## Issue Contracts

| Issue | Depends on | Deliverable |
| --- | --- | --- |
| `#618` | none | Core 4-D stage contract |
| `#619` | `#618` | Intent Packet / Deconstruct schema |
| `#620` | `#618` | Gap model and route policy |
| `#621` | `#620` | Clarify interaction contract |
| `#622` | `#620` | Research evidence contract |
| `#623` | `#619`, `#620` | Core Spec Bundle slices |
| `#624` | `#623` | Industry mapping contract |
| `#625` | `#623` | Materialization boundary |
| `#626` | `#624`, `#625` | Cross-industry fixtures |
| `#627` | `#618`-`#626` | Release certification |

## Implementation Strategy

This preview authorizes a single cohesive implementation because the issues are one Core Kernel slice and share one acceptance gate.

The implementation must not create `.agentflow/**` facts.
