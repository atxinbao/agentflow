# Software Dev Spec Workflow

更新日期：2026-06-26
执行者：Codex

## Purpose

Software Dev 的主链是 Spec-Driven，不是 Agent 自由执行。

## Main Flow

```text
Intent
-> Spec Bundle
-> Route
-> Plan / Tasks
-> Agent Execution
-> Evidence
-> Decision
-> Delivery
-> Feedback
-> Spec Evolution
```

## Route Types

| Route | 说明 |
| --- | --- |
| answer | 直接回答，不写事实源 |
| research | 调研并输出可引用结论 |
| design-only | 只做产品 / UI 设计，不授权 Build Agent |
| spec | 生成 confirmed Spec Bundle |
| build | 派生可执行 issue 并进入 Build Agent |
| release | 进入 release closeout / certification |
| audit | 独立 Audit sidecar，不进入主业务链 |

## Spec Bundle Must Define

- 背景和人类意图；
- 范围和非目标；
- PRD / 产品判断；
- 技术方案或架构计划；
- issues / tasks；
- allowed surface；
- expected outputs；
- evidence policy；
- decision criteria；
- delivery model；
- feedback and evolution rule。

## Execution Boundary

Agent 只执行当前确认的 Issue。没有 confirmed Spec Bundle 或 `.agentflow/spec/**` 执行合同，Build Agent 不能开始实现。
