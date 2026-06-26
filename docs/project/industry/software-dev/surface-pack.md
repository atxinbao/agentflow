# Software Dev Surface Pack

更新日期：2026-06-26
执行者：Codex

## Purpose

Surface Pack 定义软件开发行业客户端应该展示什么，而不是定义事实源。

客户端只能读取 Projection / Read Model，不能直接写 Runtime authority。

## Standard Surfaces

| Surface | 作用 |
| --- | --- |
| Project Home | 展示项目目标、当前状态、关键任务和风险 |
| Spec Workbench | 展示 Intent、Spec Bundle、Route、Plan 和确认状态 |
| Task Workbench | 展示 Issue、Run、Agent handoff、执行状态和验证结果 |
| Evidence Graph | 展示 requirement、spec、issue、run、evidence、decision 的追溯关系 |
| Delivery View | 展示 PR、release、diff、test log、handoff 和 delivery record |
| Audit Sidecar | 只读展示独立 Audit finding、risk 和 proof |
| Command Surface | 发起受控命令，例如 request fix、accept delivery、request audit |

## Read Model Rules

- Surface 不读 `.agentflow/spec/**` 原始合同作为 UI 状态。
- Surface 不直接写 `.agentflow/events/**`。
- Surface 通过 Projection API 读取项目状态。
- Command Surface 只能通过 Runtime API 发出 command。
- Audit Surface 只能展示 sidecar 状态，不把 Audit 放回主链。
