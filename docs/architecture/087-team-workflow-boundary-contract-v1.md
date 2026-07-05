# Team Workflow Boundary Contract v1

更新日期：2026-07-05  
执行者：Codex

## Purpose

`v1.2.1` 的团队工作流先定义本地和轻量协作边界，不进入云多租户。

这个合同回答：

- 项目共享能展示什么；
- 角色、权限、交接能展示什么；
- 团队反馈如何进入 Feedback Loop；
- 交付和决策历史如何给团队读取；
- 哪些能力明确不属于本版本。

## Runtime Contract

Runtime API 提供只读合同：

```text
team_workflow_boundary_contract()
```

版本：

```text
agentflow-team-workflow-boundary.v1
```

Desktop / SDK 只能读取这个合同。写入仍必须走 Runtime command 或已有事实源，不能把 UI 输出当成团队权限事实源。

## Included

| Capability | Read Model | Write Boundary |
| --- | --- | --- |
| Project sharing | project-sharing-read-model | Runtime command / local workspace fact |
| Role / permission / handoff | role-permission-handoff-view | Runtime command / local workspace fact |
| Team feedback | feedback-loop-read-model | Runtime command |
| Delivery and decision history | delivery-decision-history-read-model | Task / decision / delivery facts |

## Excluded

这些不属于 `v1.2.1`：

- cloud multi-tenant workspace；
- payment or entitlement management；
- public commercial launch；
- external account administration；
- industry-specific Core authority。

## Role Boundary

| Role | Can Do | Cannot Do |
| --- | --- | --- |
| Human Owner | 确认共享边界、确认交付、决定下一轮 | 绕过 Runtime 直接写任务事实 |
| Spec Agent | 整理团队输入、生成 preview、等待确认后 materialize | 直接执行工作任务、代替 Human Owner 接受交付 |
| Build Agent | 消费已确认任务、生成证据、写回交付事实 | 修改产品层共享策略、跳过验收门 |
| Audit Agent | 读取交付和决策历史、生成审计报告 | 自动修改执行结果、替代交付事实源 |

## Reference App Boundary

Software Dev Reference App 可以消费这个合同来展示团队工作流示例。

但是：

- Reference App 行为不是 Core authority；
- Core 不保存用户、账单、外部账户或 tenant policy；
- 行业 Product 只能通过 Runtime API / Projection 读取该合同。

## Acceptance

- `crates/runtime-api::team_workflow_boundary_contract()` 返回上述结构化合同；
- Desktop Tauri 暴露只读命令；
- API plane 将该合同标记为 readonly projection；
- 测试覆盖本地边界、排除项、角色权限和 Product-neutral handoff。
