# Delivery

更新日期：2026-06-30
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
| [releases/v1.1.1/README.md](releases/v1.1.1/README.md) | 当前发布基线：Product Contract Data-driven hardening |
| [releases/v1.1.1/AGENTFLOW_V1_1_1_PRODUCT_CONTRACT_DATA_DRIVEN_TASKS_V1.md](releases/v1.1.1/AGENTFLOW_V1_1_1_PRODUCT_CONTRACT_DATA_DRIVEN_TASKS_V1.md) | v1.1.1 Product contract data-driven tasks |
| [releases/v1.1.0/README.md](releases/v1.1.0/README.md) | 上一发布基线：Product Surface Hardening |
| [releases/v1.1.0/AGENTFLOW_V1_1_0_PRODUCT_SURFACE_HARDENING_TASKS_V1.md](releases/v1.1.0/AGENTFLOW_V1_1_0_PRODUCT_SURFACE_HARDENING_TASKS_V1.md) | v1.1.0 Product Surface hardening tasks |
| [releases/v1.0.9/README.md](releases/v1.0.9/README.md) | 上一发布基线：Software Dev Reference App Boundary Certification |
| [releases/v1.0.9/AGENTFLOW_V1_0_9_SOFTWARE_DEV_REFERENCE_APP_TASKS_V1.md](releases/v1.0.9/AGENTFLOW_V1_0_9_SOFTWARE_DEV_REFERENCE_APP_TASKS_V1.md) | v1.0.9 Software Dev Reference App tasks |
| [releases/v1.0.8/README.md](releases/v1.0.8/README.md) | 上一发布基线：Core Projection Kernel |
| [releases/v1.0.8/AGENTFLOW_V1_0_8_PROJECTION_KERNEL_TASKS_V1.md](releases/v1.0.8/AGENTFLOW_V1_0_8_PROJECTION_KERNEL_TASKS_V1.md) | v1.0.8 Projection Kernel tasks |

## Next Release Planning

| 路径 | 作用 |
| --- | --- |
| `v1.1.x` | 下一版计划：Product Surface follow-up hardening |

## Historical Delivery

更早版本记录已放入：

```text
docs/project/history/2026-06-current-baseline-history/versions/
```

## Rules

- `docs/delivery/**` 不定义新的产品方向。
- 新版本计划必须先有对应的 project / requirement / architecture 上下文。
- Release 文档不能绕过 confirmed Spec Bundle 和 `.agentflow/spec/**` 执行合同。
