# Software Dev AgentFlow Product

更新日期：2026-06-26
执行者：Codex

## Purpose

Software Dev 是 AgentFlow 当前唯一商业产品主线。

它把软件团队的自然语言意图转成可确认的 Spec Bundle，再由 Spec 派生计划、任务、Agent 执行、证据、完成判定、交付和反馈。

## Product Formula

```text
Software Dev AgentFlow Product
= Core Runtime
+ Software Dev Domain Pack
+ Software Dev Surface Pack
+ Software Dev Connector Pack
```

## Documents

| 文档 | 作用 |
| --- | --- |
| [product-goal.md](product-goal.md) | 行业产品目标和非目标 |
| [domain-pack.md](domain-pack.md) | 软件开发行业对象、动作和状态 |
| [surface-pack.md](surface-pack.md) | 软件开发工作台、读模型和页面 |
| [connector-pack.md](connector-pack.md) | Git、GitHub、Codex、Claude Code 等外部连接器 |
| [spec-workflow.md](spec-workflow.md) | Software Dev 的 Spec-Driven 主流程 |
| [evidence-and-decision.md](evidence-and-decision.md) | 证据标准和 Done 判定规则 |
| [delivery-model.md](delivery-model.md) | PR、release、diff、test log、decision record 等交付包 |
| [examples/README.md](examples/README.md) | 标准场景样例 |

## Non-goals

- 不定义第二行业产品。
- 不把 Audit 放回主业务链。
- 不把 GitHub issues 当作 AgentFlow authority。
- 不把 Codex / Claude Code session 当作项目事实源。
- 不在 `docs/industries/**` 写机器执行 JSON。
