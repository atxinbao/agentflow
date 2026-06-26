# AgentFlow v0.5.1 Release Hygiene & Authority Closure Tasks V1

日期：2026-06-21
执行者：Codex
状态：Historical Remediation Plan / Folded Into v0.6.0 Release Path

## 1. Purpose

本文档沉淀 AgentFlow `v0.5.1` 的修复任务链。

`v0.5.1` 的目标不是继续扩版本范围，而是收掉 `v0.5.0` 发布后暴露出的两类问题：

1. release hygiene 不完整；
2. Runtime authority 顺序没有完全闭合。

## 2. Main Chain

`v0.5.1` 的修复主链：

```text
Release Metadata
-> Tag / Release Gate
-> Spec Loop Gate
-> Arbitration Before Materialization
-> Durable Runtime Records
-> Spec Authority Manifest
-> Documentation Closeout
```

## 3. Issue Preview

| Issue | Title | Dependency | Priority |
| --- | --- | --- | --- |
| `V051-001` | Release Metadata Repair | 无 | P0 |
| `V051-002` | Tag Release Gate | `V051-001` | P0 |
| `V051-003` | Spec Loop Gate | `V051-001` | P0 |
| `V051-004` | Arbitration-before-Materialization | `V051-001` | P0 |
| `V051-005` | Durable Runtime Command Records | `V051-004` | P1 |
| `V051-006` | Spec Authority Manifest | `V051-004`, `V051-005` | P1 |
| `V051-007` | Documentation Closeout | `V051-001`, `V051-004` | P1 |

## 4. What This Version Must Prove

`v0.5.1` 完成后，必须证明：

- 发布事实和版本元数据不再漂移；
- tag / release 场景有正式 gate；
- Spec Loop 不会在未仲裁前直接写 authority；
- runtime 写入能留下 durable records；
- Spec authority、preview artifact、projection 的边界清晰；
- 文档状态、GitHub tag、Release notes、CHANGELOG 口径一致。

## 5. Version Boundary

`v0.5.1` 不进入这些范围：

- 不新增 `v0.6.0` Work Loop handoff 功能；
- 不引入新的多 Agent orchestration；
- 不改写 `v0.5.0` 的 Spec Loop 主目标；
- 不把文档修复伪装成新功能发布。

## 6. Release Rule

历史规则要求 `V051-001` 到 `V051-007` 完成后，`v0.6.0` 才能从规划进入实现。

当前事实是：`v0.6.0` 已经发布。本文档不再作为阻塞已发布版本的活跃 gate；遗留 release hygiene 和 authority closure 问题统一进入 `docs/v0.6.1/**`。
