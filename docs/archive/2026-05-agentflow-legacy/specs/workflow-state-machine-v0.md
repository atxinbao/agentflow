# Workflow State Machine v0

创建日期：2026-05-26
执行者：Codex
状态：implemented / local-state-check

## 目标

Workflow State Machine v0 将 `AgentFlow AI Delivery Workflow Contract v1` 中的 Project / Milestone / Issue 状态规则落成一个本地可执行检查。

它不负责调度、不执行代码、不创建远程对象，只回答一个问题：

```text
当前 `.agentflow/` 事实源是否满足最小工作流状态不变量？
```

## CLI

```bash
agentflow state check
```

命令行为：

- 读取 `.agentflow/projects/*.json`、`.agentflow/issues/*.json`、evidence / review 本地事实源。
- 生成 `.agentflow/state/workflow-state.json`。
- 生成 `.agentflow/updates/WORKFLOW-STATE-SUMMARY.md`。
- 若存在 error 级失败检查，命令返回失败。
- warning 级结果用于指出历史数据缺口，不阻断当前 active milestone 的执行链路。

## 已实现对象

Core 中新增：

- `WorkflowStateSnapshot`
- `WorkflowStateCounts`
- `WorkflowStateCheck`
- `WorkflowTransitionGuard`
- `WorkflowStateCheckSummary`
- `write_workflow_state_check`

## 检查范围

Project 检查：

- project status 必须属于已知状态。
- 每个 project 最多只能有一个 active milestone。
- active milestone id 必须指向存在的 milestone。
- done project 不能仍包含非 completed milestone。

Milestone 检查：

- milestone status 必须属于已知状态。
- completedIssueIds 必须是 issueIds 的子集。
- completed milestone 下的 issue 必须都是 completed。
- active milestone 必须匹配 project.activeMilestoneId。
- active milestone 当前最多只能有一个未完成 issue。

Issue 检查：

- issue status 必须属于已知状态。
- issue contract 必须保留 scope、non-goals、execution plan、validation commands、evidence requirements。
- 已关联 projectLink 的 issue 必须指向存在的 team / project / milestone。
- 已关联 projectLink 的 issue 必须被列入目标 milestone.issueIds。
- 未完成 issue 不能挂在 completed milestone 下。
- completed issue 必须有 evidence 和 review artifact。

## 状态迁移守卫

v0 输出固定 transition guard 表，用于让后续 Eligibility Engine / Lease / Evidence-based Done 复用同一批状态边界。

关键禁止项：

- `project active -> done` 禁止，必须先走 audit / docs refresh。
- `milestone active -> done` 禁止，必须先生成 milestone review。
- `issue ready -> leased` 禁止，必须先通过 eligibility。
- `issue leased -> done` 禁止，必须先有 run / checks / merge / evidence。
- `issue merged -> done` 禁止，必须先 capture evidence。

## 边界

允许：

- 写 `.agentflow/state/workflow-state.json`。
- 写 `.agentflow/updates/WORKFLOW-STATE-SUMMARY.md`。
- 作为 readiness / review 的本地检查输入。

不允许：

- 执行 `run / verify / review`。
- 调用模型。
- 创建远程 issue / PR。
- 写 projectLink。
- 迁移历史 issue。
- 修改 Desktop UI。

## 下一阶段

Workflow State Machine v0 完成后，Workflow Control Core v0 已继续补齐：

```text
Eligibility Engine
Lease / Lock
Execution Run gate
Evidence-based Done
GoalLoop eligibility check
```

后续继续进入 Project Audit / Docs Refresh closure gate。
