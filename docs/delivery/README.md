# Delivery

更新日期：2026-06-29
执行者：Codex

## Purpose

本目录记录 AgentFlow 的交付结果。

```text
docs/delivery
= release baseline
+ release certification
+ handoff / decision record
+ delivery summary
```

具体执行证据、任务事件和运行事实仍属于 `.agentflow/**`。`docs/delivery/**` 只保存人类可读的交付入口和发布记录。

## Current Delivery Records

| 路径 | 作用 |
| --- | --- |
| [releases/v1.0.6/README.md](releases/v1.0.6/README.md) | 当前发布基线：Core Evidence Kernel |
| [releases/v1.0.6/AGENTFLOW_V1_0_6_CORE_EVIDENCE_KERNEL_TASKS_V1.md](releases/v1.0.6/AGENTFLOW_V1_0_6_CORE_EVIDENCE_KERNEL_TASKS_V1.md) | v1.0.6 Core Evidence Kernel tasks |

## Next Release Planning

| 路径 | 作用 |
| --- | --- |
| [releases/v1.0.7/README.md](releases/v1.0.7/README.md) | 下一版计划：Core Decision Kernel |
| [releases/v1.0.7/AGENTFLOW_V1_0_7_DECISION_KERNEL_TASKS_V1.md](releases/v1.0.7/AGENTFLOW_V1_0_7_DECISION_KERNEL_TASKS_V1.md) | v1.0.7 Decision Kernel tasks |

## Historical Delivery

更早版本记录已放入：

```text
docs/project/history/2026-06-current-baseline-history/versions/
```

## Rules

- `docs/delivery/**` 不定义新的产品方向。
- 新版本计划必须先有对应的 project / requirement / architecture 上下文。
- Release 文档不能绕过 confirmed Spec Bundle 和 `.agentflow/spec/**` 执行合同。
