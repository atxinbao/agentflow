# Product Requirements

创建日期：2026-05-21
最近压缩：2026-05-22
执行者：Codex

## 定义

AgentFlow 是本地 AI 研发执行工作台，不替代项目管理平台。核心问题：

> AI Agent 写代码时，如何在本地约束任务边界、执行验证、生成 evidence，并交给人审查。

## 用户

| 用户 | 需求 |
| --- | --- |
| 独立开发者 | 快速把需求变成可执行任务，保留 AI 执行和验证记录 |
| 2-10 人小团队 | 统一 issue contract，用 evidence 替代口头汇报 |
| 小型外包团队 | 客户项目本地留痕，生成交付说明和验证记录 |

## Jobs

| 场景 | 输出 |
| --- | --- |
| 给出目标 | `/goal` 编译成 ProjectGoal 和第一候选任务 |
| 新需求 | `agentflow plan` 生成 issue contract |
| 执行前 | scope / non-goals / validation 审查 |
| 执行后 | Evidence Report |
| 查看项目 | Project Update Summary |
| 筛选历史 | Saved Views |
| 准备交付 | Review checklist + PR / handoff |

## MVP 必须有

- Flow 0.1 / 0.2 / 0.3 初始化。
- `/goal` 编译为 `goal.{md,json}`。
- 创建和编辑 issue contract。
- dry-run / run gate。
- validation commands。
- evidence report。
- project update summary。
- saved views / filters。
- review / PR 文案。
- 本地模型和 OpenAI-compatible endpoint 配置。
- Desktop Workbench 首版只读 `.agentflow/` 事实源。

## MVP 不做

- SaaS、账号、支付、云同步。
- 多人实时协作。
- 完整看板、燃尽图、企业报表。
- 自动 merge。
- SaaS workspace / team / permission hierarchy。
- cycles / initiatives / enterprise roadmap。
- 远程 GraphQL API、webhook 平台和公开 SDK。
- Desktop v0 直接触发执行或修改 contract。

## 指标

- 10 分钟内完成初始化。
- 首次 issue contract 成功率。
- 首次 evidence report 成功率。
- 7 日内重复打开同一项目。
- 用户编辑 contract / 复制 review 文案的比例。

## 风险

| 风险 | 应对 |
| --- | --- |
| 太像项目管理平台 | 聚焦 Agent 执行闭环 |
| 太像聊天工具 | 强制 contract + evidence |
| 照搬 Linear | 只吸收对象模型、Views、Updates |
| 模型配置复杂 | 提供国内和本地模型预设 |
| 本地文件难懂 | Markdown/JSON 可读 |
