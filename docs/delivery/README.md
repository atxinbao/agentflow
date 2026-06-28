# Delivery

更新日期：2026-06-28
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
| [releases/v1.0.4/README.md](releases/v1.0.4/README.md) | 当前发布基线：Core Ontology Kernel |
| [releases/v1.0.4/AGENTFLOW_V1_0_4_CORE_ONTOLOGY_KERNEL_TASKS_V1.md](releases/v1.0.4/AGENTFLOW_V1_0_4_CORE_ONTOLOGY_KERNEL_TASKS_V1.md) | v1.0.4 Core Ontology Kernel tasks |

## Next Release Planning

| 路径 | 作用 |
| --- | --- |
| [releases/v1.0.5/README.md](releases/v1.0.5/README.md) | 下一版计划：Core Runtime Kernel |
| [releases/v1.0.5/AGENTFLOW_V1_0_5_CORE_RUNTIME_KERNEL_TASKS_V1.md](releases/v1.0.5/AGENTFLOW_V1_0_5_CORE_RUNTIME_KERNEL_TASKS_V1.md) | v1.0.5 Core Runtime Kernel tasks |

## Historical Delivery

更早版本记录已放入：

```text
docs/project/history/2026-06-current-baseline-history/versions/
```

## Rules

- `docs/delivery/**` 不定义新的产品方向。
- 新版本计划必须先有对应的 project / requirement / architecture 上下文。
- Release 文档不能绕过 confirmed Spec Bundle 和 `.agentflow/spec/**` 执行合同。
