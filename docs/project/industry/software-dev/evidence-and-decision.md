# Software Dev Evidence and Decision

更新日期：2026-06-26
执行者：Codex

## Purpose

Evidence 证明发生过什么。Decision 判定是否完成。

验证命令通过不是 Done。证据完整也不是 Done。Done 必须由 Decision Gate 给出。

## Evidence Types

| Evidence | 示例 |
| --- | --- |
| Source Diff | git diff、changed files、allowed surface check |
| Verification Log | cargo test、npm build、browser smoke、release gate |
| Runtime Proof | command result、provider smoke、capability proof |
| Delivery Proof | PR URL、release URL、artifact manifest、handoff |
| Visual Proof | screenshot、browser preview、UI interaction result |
| Decision Record | accepted / rejected / deferred / needs-fix reason |

## Decision Gates

| Gate | 判断 |
| --- | --- |
| Verification Gate | 验证命令是否通过 |
| Evidence Gate | 证据是否完整、可追溯 |
| Contract Gate | 是否满足 issue contract |
| Boundary Gate | 是否越界修改 allowed surface |
| State Gate | 当前对象状态是否允许 Done |
| Decision Gate | 汇总以上结果，给出最终判定 |

## Decision Values

```text
accepted
rejected
deferred
needs_fix
blocked
```

## Audit Boundary

Audit 是独立 sidecar。默认主链到 Decision / Delivery 后结束。

Audit 只能从独立 Audit Issue 或明确的人类 audit request 触发。
