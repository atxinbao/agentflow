# Software Dev Product Goal

更新日期：2026-06-26
执行者：Codex

## Goal

Software Dev AgentFlow 的目标是：

```text
让软件开发团队用 Spec 驱动 Agent 完成需求、实现、验证、交付和反馈闭环。
```

## Target Users

- 需要把需求稳定交给 AI Agent 执行的软件团队。
- 需要保留证据、判定和交付记录的项目负责人。
- 需要把 Codex、Claude Code、GitHub、Git 和本地验证纳入同一工作流的工程团队。

## Core Jobs

1. 把人类意图整理成 Spec Bundle。
2. 从 Spec Bundle 派生计划、任务和执行合同。
3. 约束 Agent 在任务合同内执行。
4. 收集 diff、test log、PR、release、decision record 等证据。
5. 由 Decision Gate 判定是否 Done。
6. 输出可交付、可追溯、可反馈演进的结果。

## Non-goals

- 不做通用聊天 Agent。
- 不做单纯 Spec 文档生成器。
- 不替代 Codex / Claude Code 的底层执行 runtime。
- 不把 Audit 放进默认业务链路。
- 不让外部 issue 系统成为 AgentFlow authority。
