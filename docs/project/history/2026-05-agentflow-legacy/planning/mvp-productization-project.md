# MVP Productization Project

创建日期：2026-05-26
执行者：Codex

## 目标

把 AgentFlow 从“功能切片连续实现”收敛为一个可推进的本地 MVP 项目：

```text
Workspace
  -> Teams
      -> Projects
      -> Issues
      -> Views
  -> Projects
      -> Milestones
          -> IssueContracts
  -> GoalLoop
      -> active milestone next issue
```

## Linear 参考

Linear 的最小可借鉴关系：

- Workspace 是组织内 issues 和 interactions 的 home；创建 workspace 时会有默认 team。
- Team 侧边栏默认承载 Issues、Projects、Views。
- Issue 必须属于单个 team，title 和 status 是必需字段。
- Project milestones 用于把项目生命周期拆成阶段，并可把 issues 加入 milestone。
- Views / display options 用于围绕 issues 和 projects 做不同展示，不替代事实源。

参考来源：

- https://linear.app/docs/workspaces
- https://linear.app/docs/default-team-pages
- https://linear.app/docs/creating-issues
- https://linear.app/docs/project-milestones
- https://linear.app/docs/display-options

## AgentFlow MVP 取舍

AgentFlow 只复制 Linear 的最小产品骨架，不复制完整协作平台：

- 保留：workspace / team / project / milestone / issue / view 的本地关系。
- 保留：team 下有项目、任务、视图三个入口。
- 保留：project 用 milestone 切阶段。
- 保留：issue contract 是唯一执行授权。
- 不做：账号、权限、成员协作、云同步、远程 issue、远程 PR、完整看板。

## 执行链路模型

MVP 的产品化链路固定为：

```text
Project
  -> Milestones
      -> Issues
          -> ExecutionRuns
              -> VerificationEvidence
              -> ReviewEvidence
              -> ProjectUpdate
```

Issue 仍然是唯一执行原子；Milestone 只负责阶段归属、阶段完成标准和阶段 evidence 收口。当前本地 MVP 不创建远程 PR，`PullRequestEvidence` 后置为未来 artifact；v0 只展示本地 run / validation / evidence / review / project update。

## MVP 最小工作流

AgentFlow MVP 锁定为以下 8 步，不继续扩展完整项目管理平台：

1. Human 创建/确认 Project。
2. Project 下拆 Milestones。
3. 每个 Milestone 下挂 Issues。
4. AgentFlow 做 active milestone queue preflight。
5. 只推进当前 milestone 中唯一 eligible issue。
6. Issue 完成后记录本地 run / validation / evidence / review / project update；PR / checks / merge 后置为未来远程 artifact。
7. Milestone 全部 Done 后自动生成 `MILESTONE-*-evidence-summary.md`，并把 project seed 推进到下一个 planned milestone。
8. Project 全部 milestones Done 后进入 Stage Code Audit + Root Docs Refresh。

## 本地事实源

当前已从只读派生模型进入本地 seed 事实源：

```text
.agentflow/workspace.json
.agentflow/teams/core.json
.agentflow/projects/agentflow-local-execution.json
```

`read_local_project_model_snapshot` 现在优先读取这些 seed；没有 seed 时才回退到派生模型。

## MVP Project

Project: `agentflow-local-execution`

Goal:

> 把 AgentFlow 产品化为一个可实际使用的本地 MVP：先构建项目和 milestones，再基于 milestones 规划 issue contracts，最后按 WIP=1 推进开发、验证、证据和审查闭环。

## Milestones

| Milestone | 状态 | 目的 | 当前 issue |
| --- | --- | --- | --- |
| `mvp-foundation-archive` | completed | 归档此前 0.x 能力切片 | none |
| `mvp-project-foundation` | completed | 让 project seed / milestones 成为可读取事实源 | `ISSUE-0037` |
| `mvp-issue-planning` | completed | 基于 active milestone 自动规划和归属 IssueContract | `ISSUE-0038` |
| `mvp-execution-loop` | completed | 按 WIP=1 推进 run / verify / evidence / review / update | `ISSUE-0039` |
| `mvp-desktop-polish` | completed | 打磨桌面 MVP 的总览、团队、项目、任务、视图 | `ISSUE-0040` |
| `mvp-release-readiness` | completed | 验收 README、安装、启动、验证和演示路径 | `ISSUE-0041` |

## Issue 链路

已创建：

- `ISSUE-0037 Project Seed Fact Source v0 实现`
  - Milestone: `mvp-project-foundation`
  - Status: completed
  - Evidence: `.agentflow/evidence/ISSUE-0037-evidence.md`
- `ISSUE-0038 Milestone-aware Issue Planning v0 实现`
  - Milestone: `mvp-issue-planning`
  - Status: completed
  - Evidence: `.agentflow/evidence/ISSUE-0038-evidence.md`
- `ISSUE-0039 MVP Execution Loop v0 收敛`
  - Milestone: `mvp-execution-loop`
  - Status: completed
  - Current run: `RUN-0037`
  - Evidence: `.agentflow/evidence/ISSUE-0039-evidence.md`
- `ISSUE-0040 Desktop MVP Productization v0 收敛`
  - Milestone: `mvp-desktop-polish`
  - Status: completed
  - Current run: `RUN-0038`
  - Evidence: `.agentflow/evidence/ISSUE-0040-evidence.md`
  - Milestone summary: `.agentflow/evidence/MILESTONE-mvp-desktop-polish-evidence-summary.md`
- `ISSUE-0041 AgentFlow AI Delivery Workflow Contract v1 边界定义`
- `ISSUE-0042 Workflow State Machine v0 边界定义`
- `Workflow Control Core v0 完整可用闭环`
  - Milestone: `workflow-core-eligibility-engine`
  - Status: completed / goal-level
  - Outputs: `.agentflow/state/eligibility.json`, `.agentflow/state/leases.json`, `.agentflow/updates/ELIGIBILITY-SUMMARY.md`, `.agentflow/updates/LEASE-SUMMARY.md`
- `Project Audit / Docs Refresh v0 边界定义`
  - Milestone: `workflow-core-closure-gates`
  - Status: completed / boundary-only
  - Spec: `docs/specs/project-audit-docs-refresh-boundary.md`
- `Project Closure State v0 实现`
  - Milestone: `workflow-core-closure-gates`
  - Status: implemented / local-closure-state
  - Outputs: `.agentflow/state/project-closure.json`, `.agentflow/updates/PROJECT-CLOSURE-SUMMARY.md`
- `Project Code Audit Snapshot v0 只读实现`
  - Milestone: `workflow-core-closure-gates`
  - Status: implemented / read-only-audit-snapshot
  - Outputs: `.agentflow/state/project-code-audit.json`, `.agentflow/updates/PROJECT-CODE-AUDIT-SUMMARY.md`

`ISSUE-0039` 是第一个通过 active milestone 自动创建的后续开发 issue：`agentflow plan "MVP Execution Loop v0 收敛"` 已自动写入 issue `projectLink`，并同步更新 `.agentflow/teams/core.json` 与 `.agentflow/projects/agentflow-local-execution.json` 的 `issueIds`。

`agentflow projects` 现在会在每个 milestone 下展示 issue 的 execution state、latest run、validation、evidence 和 project update，用来回答“当前阶段卡在哪一步”。

后续 issue 必须来自 active milestone 或显式用户指令，不能再从松散 roadmap 直接跳转。

当前下一步：

```bash
agentflow project closure
```

## 推进规则

- GoalLoop 先看 goal readiness。
- 若存在 active issue，先完成当前 issue，保持 WIP=1。
- 若 active milestone 有 issue 队列，`goal next` 只推进该 milestone 下唯一 eligible issue。
- 若 active milestone 下有多个未完成 issue，`goal next` 进入 wait-human，要求先收敛唯一 eligible issue。
- 若无未完成 issue，从 active project / active milestone 的 `nextIssueIntent` 推荐下一条 issue。
- Review 在 milestone 全部 issue completed 后生成 milestone evidence summary，并自动激活下一个 planned milestone。
- 每个开发切片必须先有 IssueContract。
- Desktop 仍然只读，不从 UI 执行 run / verify / review。

## AI Delivery Workflow Contract v1

当前 MVP 不继续扩展 UI，而是先把 AgentFlow 的受控交付合同定稳：

```text
Workspace / Team
  -> Project
      -> Milestone
          -> Issue
              -> Lease
              -> Execution Run
              -> PR / Checks
              -> Evidence
      -> Milestone Review
  -> Project Audit
  -> Root Docs Refresh
```

`AgentFlow AI Delivery Workflow Contract v1` 已明确 AgentFlow 不是 Linear issue runner，而是 AI coding agent delivery system。后续实现按五个能力面推进：

1. Workflow State Machine。
2. Eligibility Engine。
3. Lease / Lock。
4. Execution Evidence。
5. Milestone / Project Closure。

`Workflow State Machine v0` 已通过 `agentflow state check` 落成本地可用切片：它读取 `.agentflow/` project / milestone / issue 事实源，输出 `.agentflow/state/workflow-state.json` 和 `.agentflow/updates/WORKFLOW-STATE-SUMMARY.md`，并只做状态不变量检查，不执行 run / verify / review，不创建远程 issue / PR。

`Workflow Control Core v0` 已继续完成 goal 级推进：`agentflow eligibility` 计算 eligible issue 和失败原因，`agentflow lease` 展示 active / stale leases，`agentflow run` 前必须通过 eligibility 并 acquire lease，`agentflow review` 完成后释放 lease。当前 active milestone 已切到 `workflow-core-closure-gates`。

`Project Code Audit Snapshot v0` 已完成只读审计输入包：`agentflow project code-audit` 生成 `.agentflow/state/project-code-audit.json` 和 `.agentflow/updates/PROJECT-CODE-AUDIT-SUMMARY.md`。Snapshot 只汇总候选风险和缺口，不创建 `.agentflow/audits/`，不修复代码，不刷新文档，不标记 Project done。

`Root Docs Refresh Snapshot v0` 已完成只读文档刷新输入包：`agentflow project docs-refresh` 生成 `.agentflow/state/project-docs-refresh.json` 和 `.agentflow/updates/PROJECT-DOCS-REFRESH-SUMMARY.md`。Snapshot 只汇总 checked docs、missing docs、updated-needed docs、required updates 和 blockers，不创建 `.agentflow/audits/`，不修改文档，不调用模型，不标记 Project done。下一候选是 `Product Feature Creation Flow v0`。

PRD / ARC / AIE 顺序固定为：`@003 / PRD` 输出合同，`@005 / ARC` 审查状态机、eligible、lease、evidence 和数据模型，`@000 / AIE` 再落仓为 issue contract 并执行验证。
