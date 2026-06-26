# AgentFlow AI Delivery Workflow Contract v1

创建日期：2026-05-26
执行者：Codex
状态：contract-defined

正式 PRD 合同文档：`docs/contracts/agentflow-ai-delivery-workflow-contract-v1.md`。本文件作为 specs 侧实现引用入口，保留核心边界、数据模型候选和后续切片顺序。

## 目标

AgentFlow 不做完整 Linear clone，也不只是 Linear issue runner。AgentFlow 的 MVP 主干是 AI coding agent 的受控交付系统：

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

核心判断：

```text
Human controls scope
Milestone controls phase
Issue controls execution
Eligibility controls when
Lease controls who
PR / checks / evidence controls Done
Audit / docs controls closure
```

## 不变量

- Project 必须由 Human 创建或确认，Agent 只能建议修改范围。
- Milestone 是阶段 gate，不是复杂调度器。
- Issue 是 Agent 唯一执行原子。
- Eligible 必须是自动计算结果，不是手工标签。
- Lease 防止多个 agent 或多次运行抢同一个 code-changing issue。
- Done 必须基于 merge / checks / evidence captured，不能只看本地测试或口头完成。
- Desktop MVP 不执行 run / verify / review / merge，只展示本地事实源。
- 当前本地 v1 不创建远程 PR、不自动 merge、不接入 CI；PR / Checks 是合同字段和后续 adapter 边界。

## 角色分工

| Role | 职责 | 输出 |
| --- | --- | --- |
| `@003 / PRD` | 定义产品目标、用户工作流、状态含义、Human gate 和非目标 | Workflow Contract / Product Contract |
| `@005 / ARC` | 审查状态机、eligible 规则、lease 机制、evidence 标准和数据模型边界 | Architecture Review / Boundary Notes |
| `@000 / AIE` | 将通过审查的合同落到仓库、生成 issue contract、执行验证和 evidence | Repo docs / issue / run / evidence |

v1 流程必须先由 PRD 输出合同，再由 ARC 审查，最后由 AIE 落仓。不能跳过合同直接实现 UI 或 writer。

## 五个能力面

### 1. Workflow State Machine

定义 Project / Milestone / Issue 的最小状态机，避免 Ready、Eligible、Leased、Done 混用。

```text
Project:
Draft -> Confirmed -> Active -> Audit -> DocsRefresh -> FinalReview -> Done

Milestone:
Draft -> Ready -> Active -> Review -> Done

Issue:
Draft -> Ready -> Eligible -> Leased -> InProgress -> PR -> ChecksPassing -> Merged -> EvidenceCaptured -> Done
                                     -> Blocked
                                     -> Failed
```

状态含义：

| Entity | State | 含义 |
| --- | --- | --- |
| Project | Draft | Human 或 Agent 草拟，不能执行 |
| Project | Confirmed | Human 已确认目标、非目标、成功标准和边界 |
| Project | Active | 至少一个 milestone 可推进 |
| Project | Audit | 所有 milestones done，进入 code audit |
| Project | DocsRefresh | code audit 通过后刷新根文档 |
| Project | FinalReview | final evidence summary 完成，等待 Human final approval |
| Project | Done | final evidence summary 和 Human final approval 完成 |
| Milestone | Draft | 阶段草案，不能执行 |
| Milestone | Ready | 阶段合同完整，但尚未成为当前阶段 |
| Milestone | Active | 当前唯一推进阶段 |
| Milestone | Review | 所有 issue done，等待 milestone review gate |
| Milestone | Done | Human 确认 review，可进入下一 milestone |
| Issue | Draft | 任务草案，不能执行 |
| Issue | Ready | contract 内容完整，但未必可执行 |
| Issue | Eligible | 自动规则确认当前可执行 |
| Issue | Leased | 被某个 agent 领取，其他 agent 不能执行 |
| Issue | InProgress | agent 正在运行 |
| Issue | PR | 已产生 PR 或本地 PR evidence stub |
| Issue | ChecksPassing | checks / validation 全部通过 |
| Issue | Merged | PR merged 或本地 merge-equivalent 记录完成 |
| Issue | EvidenceCaptured | evidence 完整归档 |
| Issue | Done | Done 标准全部满足 |
| Issue | Blocked | 依赖、环境、风险或 Human gate 阻塞 |
| Issue | Failed | 执行失败，需要诊断或重新规划 |

### 2. Eligibility Engine

Eligible 是自动计算结果。Issue 只有同时满足以下条件才可进入 `Eligible`：

| Rule | 要求 | v1 处理 |
| --- | --- | --- |
| Active milestone | Issue 属于当前 active milestone | hard gate |
| Dependencies | 所有 dependencies 已 Done | hard gate |
| Acceptance criteria | 验收标准明确 | hard gate |
| Test plan | 测试命令或验证方式明确 | hard gate |
| Risk level | risk level 已标注 | hard gate |
| Scope | expected files / areas 明确 | hard gate |
| Blockers | 没有 unresolved blocker | hard gate |
| PR conflict | 没有 open PR 正在改同一区域 | future adapter gate |
| Repo state | repo clean / base latest | future adapter gate |
| Issue size | 没超过 agent 可控范围 | human-review gate |
| High risk | high-risk issue 需要 Human approve | human gate |
| Preflight | AgentFlow queue preflight 通过 | hard gate |

Ready 与 Eligible 的区别：

```text
Ready = issue contract 写完整了。
Eligible = 在当前 project / milestone / repo / lease 环境下确实能被 agent 执行。
```

### 3. Lease / Lock

Lease 是执行锁，不是权限系统。它只解决“谁正在执行这个 issue”和“是否允许第二个 agent 抢任务”。

最小字段：

```yaml
issue_id: ISSUE-123
lease_owner: codex-agent-1
lease_started_at: 2026-05-26T12:00:00Z
lease_expires_at: 2026-05-26T12:30:00Z
lease_status: active
lease_scope: code-changing
```

规则：

- 一个 active milestone 默认只能有一个 code-changing lease。
- read-only investigation、docs-only、test-only、review/evidence summary 可以后续定义 parallel-safe lease。
- stale lease 必须可恢复，但不能静默抢占；需要记录 recovery evidence。
- lease 不授权扩大 issue scope。
- Agent 可以 propose scope change，但 Human 必须 approve。

### 4. Execution Evidence

Issue Done 不能只看 merge，也不能只看 local test pass。Done 必须至少包含：

| Evidence | 要求 |
| --- | --- |
| PR link | 当前本地 v1 可为 future artifact / local stub |
| Commit hash | 合并 commit 或本地 equivalent |
| Checks result | lint / test / typecheck / security check 结果 |
| Changed files | 实际变更文件列表 |
| Acceptance coverage | 每条 acceptance criteria 的满足情况 |
| Test output | 关键命令输出摘要 |
| Behavior proof | 截图、日志、API response 或 manual notes，按 issue 类型决定 |
| Deferred work | 未完成或延期内容 |
| Risk | 遗留风险 |
| Rollback plan | 如何回滚 |

Done 标准：

```text
PR / merge-equivalent completed
checks passing
review completed
evidence captured
rollback plan recorded
issue update written
```

### 5. Milestone / Project Closure

Milestone review 是 gate，不只是总结。所有 issue Done 后，Milestone 进入 `Review`，必须生成 review / evidence summary，并由 Human 确认后进入 `Done`。

Milestone Review 必须覆盖：

- original milestone goal
- completed issues
- PR / checks / evidence 链路
- acceptance criteria coverage
- skipped tests / TODO / known gaps
- schema / env / config / API / docs 变化
- user-visible behavior changes
- risks / open questions
- recommendation: proceed / hold

Project 结束后分两阶段：

```text
Stage 1: Project Code Audit
Stage 2: Root Docs Refresh
```

`Project Audit / Docs Refresh v0` 已将 closure gate 细化为：

```text
active -> audit -> docs-refresh -> final-review -> done
```

边界文档见 `docs/specs/project-audit-docs-refresh-boundary.md`。后续实现必须先落 `Project Closure State v0 实现`，把 project 不能直接 done 的本地状态守卫接入 workflow core；任何 audit finding 修复或 docs refresh 修改都必须回到 IssueContract，不允许从 closure gate 直接执行。

Project Code Audit 检查：

- 重复实现
- 临时代码
- 未使用代码
- 安全问题
- 性能退化
- 错误处理缺口
- 测试缺口
- 架构偏离
- 破坏性 API 变化

Root Docs Refresh 检查：

- README
- ARCHITECTURE
- CONTRIBUTING
- CHANGELOG
- API docs
- ENV example
- Migration guide
- Runbook
- Known limitations

## 数据模型候选

### ProjectDeliveryContract

```json
{
  "projectId": "agentflow-local-execution",
  "state": "Active",
  "charter": {
    "goal": "Human confirmed product goal",
    "nonGoals": [],
    "successCriteria": [],
    "allowedAreas": [],
    "riskLevel": "medium",
    "humanGates": []
  },
  "activeMilestoneId": "mvp-release-readiness",
  "milestoneIds": []
}
```

### MilestoneDeliveryContract

```json
{
  "milestoneId": "mvp-release-readiness",
  "projectId": "agentflow-local-execution",
  "state": "Active",
  "goal": "Stage goal",
  "gateCriteria": [],
  "issueIds": [],
  "review": {
    "required": true,
    "status": "pending",
    "summaryPath": null,
    "humanApprovedAt": null
  }
}
```

### IssueDeliveryContract

```json
{
  "issueId": "ISSUE-0041",
  "milestoneId": "mvp-release-readiness",
  "state": "Ready",
  "dependencies": [],
  "acceptanceCriteria": [],
  "expectedFiles": [],
  "blockedFiles": [],
  "riskLevel": "medium",
  "testPlan": [],
  "evidenceRequired": [],
  "rollbackPlan": "",
  "lease": null
}
```

### EligibilitySnapshot

```json
{
  "issueId": "ISSUE-0041",
  "eligible": false,
  "state": "Ready",
  "checks": [
    {"name": "active-milestone", "status": "pass"},
    {"name": "acceptance-criteria", "status": "missing"}
  ],
  "blockingReasons": [],
  "recommendedCommand": "agentflow projects"
}
```

### ExecutionEvidence

```json
{
  "issueId": "ISSUE-0041",
  "runId": "RUN-0039",
  "pr": {"url": null, "status": "future-adapter"},
  "checks": [],
  "commitHash": null,
  "changedFiles": [],
  "acceptanceCoverage": [],
  "rollbackPlan": "",
  "evidencePath": ".agentflow/evidence/ISSUE-0041-evidence.md"
}
```

## Preflight 输出合同

后续 `agentflow preflight ISSUE-XXXX` 应输出结构化结果：

```yaml
issue_id: ISSUE-0041
project_id: agentflow-local-execution
milestone_id: mvp-release-readiness
eligible: true
risk: medium
lease_required: true
lease_owner: null
lease_expires_at: null
acceptance_criteria: []
expected_files: []
blocked_files: []
test_plan: []
rollback_plan: ""
blocking_reasons: []
recommended_next: "agentflow lease ISSUE-0041 --yes"
```

## Product Feature Creation Flow v0

第一个产品功能入口是：

```bash
agentflow feature create "<feature goal>"
agentflow feature create "<feature goal>" --write --yes
```

该入口把 Human 输入的产品功能目标确定性落成本地 Project、Milestones 和 IssueContracts。默认 preview 不写事实源；只有 `--write --yes` 才更新 `.agentflow/workspace.json`、`.agentflow/teams/{team-id}.json`、`.agentflow/projects/{feature-project-id}.json`、`.agentflow/issues/ISSUE-XXXX.{json,md}` 和 `.agentflow/index.json`。

Product Feature Creation Flow v0 不调用模型、不创建远程 PR / GitHub issue / Linear issue、不从 Desktop 执行创建。写入后的第一条 issue 仍必须通过 `goal next -> eligibility -> lease -> run`，不能绕过 IssueContract。

Product Feature Execution Flow v0 提供只读执行入口：

```bash
agentflow feature status
agentflow feature next
```

它只读取 active Product Feature Project、active milestone、current issue、eligibility 和 latest run / validation / evidence / review，并推荐 run / verify / review / wait-human。它不自动执行命令，不 acquire lease，不创建 run，不调用模型。

## Human Gates

| Gate | Human 必须确认 |
| --- | --- |
| Project Charter | Project goal、non-goals、success criteria、allowed areas |
| Milestone Contract | milestone goal、gate criteria、issue list |
| High Risk Issue | auth / billing / permissions / data deletion / migration / infra |
| Scope Change | Agent 提议扩大或改变 issue 范围 |
| Stale Lease Recovery | 接管或释放已有 lease |
| Milestone Review | 是否进入下一个 milestone |
| Final Project Approval | Project 是否 Done |

## v1 实现顺序

1. Workflow State Machine v0 边界定义。
2. Eligibility Engine v0 边界定义。
3. Lease / Lock v0 边界定义。
4. Evidence-based Done v0 实现。
5. Milestone Human Gate v0 边界定义。
6. Project Audit / Root Docs Refresh v0 边界定义。

## 当前阶段不做

- 不实现 UI。
- 不创建完整 Linear clone。
- 不接入远程 PR provider。
- 不自动 merge。
- 不实现 lease writer。
- 不改写 `goal_loop_decision`。
- 不把 PR / checks 伪装成已接入能力。
- 不让 Agent 自己批准 Project / Milestone / scope change。

## 验收标准

- Contract 能解释 AgentFlow 与 Linear issue runner 的区别。
- 五个能力面定义清楚。
- 状态机、Eligible、Lease、Evidence Done、Milestone Review、Project Closure 的边界清楚。
- 后续实现顺序明确。
- 当前仓库仍保持 Project -> Milestone -> Issue -> Evidence 的 MVP 主链路。
