# Workflow Control Core v0

创建日期：2026-05-26
执行者：Codex
状态：implemented / local-control-core

## 目标

Workflow Control Core v0 将 AgentFlow 的受控交付主链路从状态检查推进为本地可执行控制闭环：

```text
Project
-> Milestone
-> Issue
-> Eligibility
-> Lease
-> Execution Run
-> Validation
-> Evidence
-> Review
-> Milestone Summary
-> Next Milestone
```

它不是 UI，也不是 Linear clone。它的职责是让本地 agent 执行前必须经过明确的 workflow gates。

## CLI

```bash
agentflow state check
agentflow eligibility
agentflow eligibility ISSUE-XXXX
agentflow lease
agentflow run ISSUE-XXXX --dry-run
agentflow verify ISSUE-XXXX
agentflow review ISSUE-XXXX
agentflow goal next
```

## Eligibility Engine v0

新增对象：

- `WorkflowEligibilitySnapshot`
- `WorkflowEligibilityCandidate`
- `WorkflowEligibilitySummary`
- `WorkflowEligibilityCheckSummary`

输出：

- `.agentflow/state/eligibility.json`
- `.agentflow/updates/ELIGIBILITY-SUMMARY.md`

规则：

- Eligible 是计算结果，不是人工状态字段。
- Ready 只表示 issue contract 基本完整。
- Eligible 必须同时满足 active project、active milestone、issue projectLink、validation commands、scope / non-goals / execution plan / evidence requirements / riskLevel / rollbackPlan。
- 当前 project 内已有 active lease 时，其他 code-changing issue 不能 eligible。
- active milestone 下没有 issue 时，输出明确原因，并推荐下一条 plan command。

## Lease / Lock v0

新增对象：

- `WorkflowLeaseRecord`
- `WorkflowLeaseSnapshot`
- `WorkflowLeaseSummary`

输出：

- `.agentflow/leases/LEASE-*.json`
- `.agentflow/state/leases.json`
- `.agentflow/updates/LEASE-SUMMARY.md`

规则：

- `agentflow run ISSUE-XXXX --dry-run` 会先检查 eligibility。
- eligibility 通过后自动 acquire local lease。
- lease 记录 issue / project / milestone / owner / leased_at / expires_at / status。
- 同一 project 默认只允许一个 code-changing active lease。
- stale lease 只检测，不自动恢复。
- `agentflow review ISSUE-XXXX` 完成后释放对应 lease。

## Execution Run Gate

`AgentRun` 现在记录：

- `projectId`
- `milestoneId`
- `leaseId`
- `runPlan`

run 前置链路：

```text
IssueContract
-> Eligibility candidate
-> Lease acquired
-> ScopeState claimed
-> RUN-XXXX
```

run 仍是本地 dry-run，不调用模型、不编辑代码、不创建远程 PR。

Product Feature Controlled Run v0 在此基础上补齐 dry-run 的可读执行计划：

- `agentflow run ISSUE-XXXX --dry-run` 不带 `--dry-run` 会失败。
- CLI 输出 goal、expected files、blocked files / areas、validation commands、evidence requirements 和 rollback plan。
- `.agentflow/runs/RUN-XXXX/run.json` 的 `runPlan` 与 CLI 输出一致。
- `feature status` 显示 dry-run recorded、latest run plan、expected files、blocked files、validation commands 和 evidence requirements。
- `feature next` 在 dry-run 后推荐 `agentflow verify ISSUE-XXXX`。

## Evidence-based Done

Issue Done 仍由 `review` 收口：

```text
run
-> verify
-> review
-> evidence
-> review artifact
-> project update
-> issue completed
-> lease released
-> milestone summary if complete
```

Project 不允许在 milestones 全部完成后直接跳到最终 Done。后续没有 next milestone 时，project 应进入 audit / docs refresh 闭环。

Project Audit / Docs Refresh v0 已将该闭环定义为：

```text
active -> audit -> docs-refresh -> final-review -> done
```

该边界只授权 closure gate 定义，不授权自动审计器或 Desktop 写入口。`Project Closure State v0`、`Project Code Audit Snapshot v0` 和 `Root Docs Refresh Snapshot v0` 已把 closure 前的本地状态检查、只读 audit input package、只读 docs refresh input package 接入 CLI；它们仍不创建 `.agentflow/audits/`，不修改代码或文档，不标记 Project done。

## GoalLoop 更新

`goal next` 现在在 active milestone queue 中读取 eligibility：

- 1 个 eligible issue：推荐 `agentflow run ISSUE --dry-run`。
- 0 个 eligible issue 且有 open issue：推荐 `agentflow eligibility` 并说明失败原因。
- 多个 eligible issue：进入 `wait-human`，要求收敛 WIP=1。
- active milestone 没有 issue：推荐 active milestone 的 next issue intent。

GoalLoop 仍只推荐，不执行。

## Product Feature Creation Flow v0 接入

`agentflow feature create "<feature goal>" --write --yes` 现在可以创建新的 active Project、默认 milestones 和 IssueContracts。Workflow Control Core 不为它放宽执行规则：

- 新 issue 仍必须通过 `agentflow eligibility`。
- `agentflow run ISSUE-XXXX --dry-run` 仍先 acquire lease。
- `goal next` 只推荐 active project / active milestone 的第一条 eligible issue。
- Desktop 仍不能创建 feature，也不能执行 recommended command。

Product Feature Execution Flow v0 又在创建入口之后增加只读状态层：

- `agentflow feature status` 展示 active Product Feature Project、active milestone、当前 issue、eligibility 和 latest run / validation / evidence / review。
- `agentflow feature next` 复用 `issue_next_step` 决策，只推荐 run / verify / review / wait-human。
- 该层不写执行状态，不 acquire lease，不创建 run，不调用模型，不创建远程对象。

## 当前边界

允许：

- 写 `.agentflow/state/eligibility.json`。
- 写 `.agentflow/state/leases.json`。
- 写 `.agentflow/leases/LEASE-*.json`。
- 在 `run` 前本地 acquire lease。
- 在 `review` 后本地 release lease。

不允许：

- 新增 Desktop UI。
- 接入远程 PR / GitHub / Linear。
- 调用模型。
- 做 SaaS、账号、支付、云同步。
- 自动恢复 stale lease。
- 手动写 issue eligible 状态绕过计算。

## 下一阶段

当前 active milestone 仍为：

```text
workflow-core-closure-gates
```

下一候选：

```bash
agentflow feature next
```
