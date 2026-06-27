# Requirements

更新日期：2026-06-26
执行者：Codex

## Purpose

本目录是后续 confirmed Spec Bundle 的公共记录入口。

旧模式下的 flat requirement records 已经归档，不再作为当前开发入口。

归档位置：

```text
docs/project/history/2026-06-current-baseline-history/requirements/
```

## Current Rule

新需求不能直接写成零散 issue。

后续应该使用 Spec-Driven 结构：

```text
docs/requirements/
  <spec-id>/
    spec-bundle.md
    issue-preview.md
    decision.md
    delivery.md
```

其中 `spec-bundle.md` 至少包含：

- Intent Slice；
- Domain Slice；
- Product Slice；
- Plan Slice；
- Task Slice；
- Decision Slice；
- Delivery Slice；
- Feedback Slice。

## Authority Boundary

`docs/requirements/**` 是人类可读的 confirmed Spec Bundle 记录。

`.agentflow/spec/**` 是从 confirmed Spec Bundle 派生出来的机器可执行合同。

GitHub issue 只能是外部 planning mirror，不能替代 AgentFlow authority。

## Current Confirmed Bundles

| Bundle | Purpose |
| --- | --- |
| [v0.18.0-core-4d-spec-intake/spec-bundle.md](v0.18.0-core-4d-spec-intake/spec-bundle.md) | Core 4-D Spec Intake Kernel |

## Non-goals

- 不恢复旧 flat requirements 列表；
- 不从历史归档自动派生新 issue；
- 不直接写 `.agentflow/spec/**`；
- 不跳过 Spec confirmation；
- 不把 GitHub issues 作为需求事实源。
