# 009 - Issue Preflight Boundary V1

创建日期：2026-06-17
执行者：Codex

## Source Foundation

本文从以下文档派生：

- [../product/005-goal-to-project-loop-flow-v1.md](../product/005-goal-to-project-loop-flow-v1.md)
- [007-project-loop-entry-from-confirmed-plan-v1.md](007-project-loop-entry-from-confirmed-plan-v1.md)
- [008-project-loop-scheduler-boundary-v1.md](008-project-loop-scheduler-boundary-v1.md)

## Purpose

本文定义 Issue Preflight 的边界。

Issue Preflight 位于 Project Loop Scheduler 之后，负责检查 Scheduler 推荐的候选 Issue 是否具备进入后续 Work Loop 的条件。

它是执行前检查，不是执行器。

它回答：

- Scheduler 推荐的 candidate issue 是否仍然有效？
- Issue Contract 是否完整？
- dependencies 是否满足？
- scope / non-goals / boundary 是否足够清楚？
- validation / evidence 是否可机械检查？
- 是否需要人工确认？
- 下一步应该进入 Work Loop Entry，还是回到计划修订 / 人工确认 / Scheduler 复查？

## Position In Flow

```text
Project Loop Scheduler
-> Candidate Issue
-> Issue Preflight
-> Issue Preflight Result
-> Work Loop Entry Proposal
```

Issue Preflight 是 Project Loop 到 Work Loop 之间的最后一道只读 gate。

通过 preflight 不等于执行。

## Inputs

Issue Preflight 读取：

```text
.agentflow/spec/projects/<project-id>.json
.agentflow/spec/issues/<issue-id>.json
docs/projects/<project-id>/GOAL.md
docs/projects/<project-id>/PLAN.md
docs/projects/<project-id>/DECISIONS.md
ProjectLoopSchedulerSnapshot
```

可选读取：

```text
.agentflow/events/**
.agentflow/projections/**
docs/projects/<project-id>/PROJECT_HEALTH.md
docs/projects/<project-id>/EVIDENCE.md
docs/projects/<project-id>/DELIVERY.md
```

## Preflight Result

建议对象：

```text
IssuePreflightResult
```

字段：

```text
projectId
issueId
candidateFromScheduler
contractComplete
dependenciesSatisfied
scopeClear
nonGoalsClear
acceptanceCriteriaComplete
validationCommandsPresent
evidenceRequirementsPresent
boundaryComplete
riskLevel
humanConfirmationRequired
checks
blockedReasons
requiredNextGate
status
readonly
```

## Preflight Status

```text
passed
blocked
needs-human-confirmation
needs-contract-revision
not-candidate
```

### passed

表示 Issue 已满足进入后续 Work Loop Entry 的条件。

仍然不代表：

- 已 acquire lease。
- 已开始执行。
- 已修改代码。
- 已生成 checkpoint。
- 已写 evidence。

### blocked

表示 Issue 当前不能进入 Work Loop Entry。

常见原因：

- Issue 缺失。
- Issue 不属于当前 Project。
- dependencies 未完成。
- Issue 已 done / cancelled / deferred。
- Issue Contract 缺少关键字段。
- 存在无法解释的冲突状态。

### needs-human-confirmation

表示 Issue 结构基本完整，但必须先由人确认。

常见原因：

- high-risk issue。
- scope change。
- forbidden file exception。
- 外部依赖或权限。
- 需要拆分 Project。
- acceptance criteria 存在歧义。

### needs-contract-revision

表示 Issue Contract 需要回到 Plan / Spec 层修订。

常见原因：

- goal 不可验收。
- scope / non-goals 冲突。
- validation commands 缺失。
- evidence requirements 缺失。
- boundary 太弱，无法限制 Agent 行为。

### not-candidate

表示输入 Issue 不是 Scheduler 当前推荐候选。

V1 不允许用户或 Agent 绕过 Scheduler 手动指定任意 Issue 进入 preflight。

## Checks

建议对象：

```text
IssuePreflightCheck
```

字段：

```text
name
status
message
blocking
source
```

`status` 取值：

```text
passed
warning
blocked
not-applicable
```

## Required Checks

V1 必须检查：

| Check | Passed 条件 | Blocked 条件 |
| --- | --- | --- |
| scheduler-candidate | Issue 等于 Scheduler candidateIssue | 不是当前候选 |
| project-match | Issue 属于当前 Project | Project 不一致 |
| issue-state | Issue 未 done / cancelled / deferred | Issue 已结束或延期 |
| dependencies | dependencies 均完成或明确跳过 | 存在未完成依赖 |
| goal | Issue goal 可验收 | goal 空或不可验证 |
| scope | scope 明确 | scope 空或过宽 |
| non-goals | non-goals 明确 | non-goals 缺失且风险不可控 |
| acceptance-criteria | 验收标准可检查 | 缺失或不可验证 |
| validation | 验证命令或验证方式存在 | 缺失 |
| evidence | evidence requirements 存在 | 缺失 |
| boundary | boundary 明确 | 缺失禁止动作 / 禁止区域 |
| risk | riskLevel 可识别 | riskLevel 缺失或未知 |
| human-gate | 高风险 / scope change 已确认 | 需要人工确认但未确认 |

## Required Next Gate

`requiredNextGate` 可以是：

```text
work-loop-entry
human-confirmation
contract-revision
scheduler-recheck
blocked
```

### work-loop-entry

表示 Issue 可以交给后续 Work Loop Entry 生成执行提案。

不表示自动执行。

### human-confirmation

表示必须先由人确认风险或边界。

### contract-revision

表示必须回到 Project Brain / Plan Draft / Spec 进行修订。

### scheduler-recheck

表示当前候选已经失效，需要重新运行 Project Loop Scheduler。

### blocked

表示存在阻塞项，不能继续推进。

## Human Confirmation Rules

Issue Preflight 必须要求人工确认，如果：

- `riskLevel = high`。
- Issue 需要 scope change。
- Issue 需要修改原本 forbidden 的路径。
- Issue 涉及密钥、生产环境、外部系统权限。
- Issue 的 acceptance criteria 不可机械验证。
- Issue 暗示需要拆分新 Project。
- Issue 与当前 Goal / Plan 出现方向冲突。

人工确认只解除 gate，不授权 Work Loop 自动执行。

## Readonly Boundary

Issue Preflight V1 只读。

允许：

- 读取 SpecProject / SpecIssue。
- 读取 Project Brain 文档。
- 读取 Scheduler Snapshot。
- 读取事件、projection、evidence 摘要。
- 输出 IssuePreflightResult。
- 输出下一步 gate 建议。

不允许：

- 不写 `.agentflow/`。
- 不写 `docs/projects/**`。
- 不修改 Issue status。
- 不把 Issue 提升为 todo / ready。
- 不 acquire lease。
- 不创建 checkpoint。
- 不生成 patch。
- 不运行命令。
- 不写 evidence。
- 不启动 Agent。
- 不生成 Audit Report。
- 不生成 Delivery Report。
- 不调用模型。
- 不创建远程 PR / GitHub Issue / Linear Issue。

## Relationship With Scheduler

Scheduler 负责从项目整体判断“下一个候选是谁”。

Issue Preflight 负责判断“这个候选是否可进入执行入口”。

二者分工：

| Layer | Responsibility |
| --- | --- |
| Project Loop Scheduler | 项目级排序、阶段判断、候选推荐 |
| Issue Preflight | 候选 Issue 合同完整性和执行边界检查 |
| Work Loop Entry | 后续执行提案，不在本文定义 |

## Relationship With Work Loop

Issue Preflight 不属于 Work Loop。

它不能：

- 创建 run。
- 创建 lease。
- 执行 patch。
- 执行验证命令。
- 产出最终 evidence。

它只允许把结果交给后续 Work Loop Entry：

```text
IssuePreflightResult(status=passed)
-> WorkLoopEntryProposal
```

## Blocked Conditions

Issue Preflight 必须 blocked，如果：

- Candidate Issue 缺失。
- Candidate Issue 不属于当前 Project。
- Candidate Issue 不等于 Scheduler 推荐候选。
- dependencies 未完成。
- Issue Contract 缺少 goal。
- Issue Contract 缺少 scope。
- Issue Contract 缺少 acceptance criteria。
- Issue Contract 缺少 validation commands / validation method。
- Issue Contract 缺少 evidence requirements。
- Issue Contract 缺少 boundary。
- high-risk issue 未经过人工确认。
- Issue 与 Goal / Plan 冲突。
- Issue 已 done / cancelled / deferred。

## Output

V1 输出：

```text
IssuePreflightResult
IssuePreflightCheck[]
BlockedReasons
RequiredNextGate
```

不输出：

```text
Run
Lease
Checkpoint
Patch
Command Result
Evidence
Audit Report
Delivery Report
```

## Not In Scope

本文不定义：

- Work Loop Entry 的写入。
- Queue promotion。
- Lease。
- Checkpoint。
- Patch。
- Command execution。
- Verification execution。
- Evidence writer。
- Audit execution。
- Delivery generation。
- Desktop write UI。
- Model call。
- Remote PR / GitHub / Linear。

## Acceptance Criteria

- [ ] Issue Preflight 的位置和职责明确。
- [ ] IssuePreflightResult 字段明确。
- [ ] Preflight status 语义明确。
- [ ] Required checks 明确。
- [ ] `passed` 不等于执行。
- [ ] high-risk issue 必须进入 human-confirmation。
- [ ] contract 不完整必须 blocked 或 needs-contract-revision。
- [ ] Scheduler 与 Preflight 分工明确。
- [ ] Preflight 与 Work Loop 分界明确。
- [ ] V1 不写 `.agentflow/`。
- [ ] V1 不修改 docs。
- [ ] V1 不 acquire lease。
- [ ] V1 不运行命令。
- [ ] V1 不调用模型。
