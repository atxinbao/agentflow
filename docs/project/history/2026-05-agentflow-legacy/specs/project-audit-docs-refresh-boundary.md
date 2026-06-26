# Project Audit / Docs Refresh v0 Boundary

创建日期：2026-05-26
执行者：Codex
状态：implemented / boundary-only

## 目标

Project Audit / Docs Refresh v0 定义 AgentFlow Project 进入最终 closure 前必须经过的边界：

```text
active
-> audit
-> docs-refresh
-> final-review
-> done
```

本阶段只定义 Code Audit、Root Docs Refresh、Final Evidence Summary 和 Human Final Approval 的输入、输出、gate 和禁止项；不实现自动审计器，不修改 Desktop UI，不接入远程 PR / GitHub / Linear。

## 位置

Project closure 发生在 Workflow Control Core v0 之后：

```text
Project
-> Milestones done
-> Milestone evidence summaries
-> Project Audit
-> Root Docs Refresh
-> Final Evidence Summary
-> Human Final Approval
-> Project Done
```

Issue 仍是执行原子；Project closure 不是新的任务执行入口。任何代码修改、文档刷新或审计修复都必须回到 IssueContract 和 Workflow Control Core 链路。

## Project 状态边界

允许的最小状态迁移：

| From | To | Gate |
| --- | --- | --- |
| `active` | `audit` | 所有 milestones 已完成，并且 milestone evidence summaries 存在 |
| `audit` | `docs-refresh` | Code Audit 已生成，必修 findings 已转成后续 issue 或被 human accepted |
| `docs-refresh` | `final-review` | Root Docs Refresh 已生成，必要 root docs 已检查 |
| `final-review` | `done` | Final Evidence Summary 已生成，并且 Human Final Approval 明确通过 |

禁止迁移：

```text
active -> done
audit -> done
docs-refresh -> done
final-review -> done without human approval
```

## Code Audit Boundary

Code Audit 是 closure gate，不是自动修复器。

检查范围：

- duplicate code
- temporary code
- unused code
- dead exports
- TODO / FIXME
- security / auth / permission risk
- performance risk
- architecture drift
- test gaps
- unexpected public API changes

输入：

- `.agentflow/projects/{project-id}.json`
- `.agentflow/issues/*.json`
- `.agentflow/runs/*/run.json`
- `.agentflow/evidence/*.md`
- `.agentflow/reviews/*.md`
- `.agentflow/updates/PROJECT-UPDATE-*.md`
- repository diff / source tree read-only scan

输出候选：

```text
.agentflow/audits/{project-id}-code-audit.md
.agentflow/audits/{project-id}-code-audit.json
```

最小输出字段：

```yaml
projectId:
status: passed | failed | accepted-risk
findings:
  - id:
    severity: low | medium | high
    category:
    summary:
    evidence:
    requiredFix:
    followUpIssueIntent:
acceptedRisks:
  - findingId:
    acceptedBy:
    reason:
generatedAt:
```

Gate 规则：

- `failed` audit 不能进入 `docs-refresh`。
- high-risk finding 不能被系统自动接受。
- audit finding 需要修改代码时，必须生成后续 IssueContract intent，不能在 audit 阶段直接改代码。
- audit 只读扫描可以自动执行；修复不能自动执行。

## Root Docs Refresh Boundary

Root Docs Refresh 是文档一致性 gate，不是任意文档重写入口。

检查范围：

- `README.md`
- `ROADMAP.md`
- `docs/specs/mvp-spec.md`
- architecture docs
- contracts
- validation docs
- runbook / known limitations
- `verification.md`

输入：

- Project goal
- completed milestones
- completed issues
- latest runs / validations
- evidence / reviews
- Code Audit output
- current root docs

输出候选：

```text
.agentflow/audits/{project-id}-docs-refresh.md
.agentflow/audits/{project-id}-docs-refresh.json
```

最小输出字段：

```yaml
projectId:
status: passed | failed
checkedDocs:
  - path:
    status: current | updated-needed | missing | intentionally-absent
    reason:
requiredUpdates:
  - path:
    summary:
    followUpIssueIntent:
knownLimitations:
generatedAt:
```

Gate 规则：

- 必要 root docs 缺失时不能进入 `final-review`。
- 文档需要修改时必须走 IssueContract，不能从 closure gate 直接改写。
- `verification.md` 继续 append-only，不能重写历史记录。
- latest verification summary 可以被刷新，但刷新动作必须被明确授权。

## Final Evidence Summary Boundary

Final Evidence Summary 汇总 Project 的本地交付证据。

必须包含：

- project goal
- completed milestones
- completed issues
- runs / validations
- evidence / reviews
- milestone evidence summaries
- code audit result
- docs refresh result
- known gaps
- deferred work
- final recommendation

输出候选：

```text
.agentflow/evidence/PROJECT-{project-id}-final-evidence-summary.md
.agentflow/evidence/PROJECT-{project-id}-final-evidence-summary.json
```

最小输出字段：

```yaml
projectId:
projectGoal:
milestonesCompleted:
issuesCompleted:
runs:
validations:
evidence:
reviews:
codeAudit:
docsRefresh:
knownGaps:
deferredWork:
recommendation: approve | hold
generatedAt:
```

Gate 规则：

- Final Evidence Summary 不能替代 Code Audit 或 Docs Refresh。
- recommendation 为 `hold` 时，Project 不能 `done`。
- summary 必须可追溯到本地 `.agentflow/` 事实源。

## Human Final Approval Boundary

Human Final Approval 是 Project Done 前的最后 gate。

最小字段：

```yaml
projectId:
approved: true | false
approver:
approvedAt:
approvalNotes:
acceptedRisks:
```

规则：

- Agent 可以生成 approval packet，但不能替用户批准。
- Human final approval 缺失时，Project 只能停在 `final-review`。
- Human approval 不能让 Project 跳过 audit / docs-refresh / final evidence。

## CLI Boundary

当前阶段不新增 CLI 命令实现。

后续实现候选：

```bash
agentflow project audit
agentflow project docs-refresh
agentflow project final-summary
agentflow project close
```

v0 实现顺序建议：

1. `Project Closure State v0 实现` - implemented
2. `Project Code Audit Snapshot v0 只读实现` - implemented
3. `Root Docs Refresh Snapshot v0 只读实现` - implemented
4. `Project Final Evidence Summary v0 实现`
5. `Human Final Approval Gate v0 实现`

`Project Code Audit Snapshot v0` 已实现：

```bash
agentflow project code-audit
```

该命令只读生成：

```text
.agentflow/state/project-code-audit.json
.agentflow/updates/PROJECT-CODE-AUDIT-SUMMARY.md
```

Snapshot 汇总 Project closure、project / milestone / issue / run / evidence / review / update 事实源和源码树扫描结果。它只生成 audit input package，不创建 `.agentflow/audits/`，不自动修复 findings，不修改代码或文档，不标记 Project done。`agentflow project closure` 在 snapshot 存在时将 code audit gate 显示为 `snapshot-ready`，但该状态仍不等于 final Code Audit passed。

`Root Docs Refresh Snapshot v0` 已实现：

```bash
agentflow project docs-refresh
```

该命令只读生成：

```text
.agentflow/state/project-docs-refresh.json
.agentflow/updates/PROJECT-DOCS-REFRESH-SUMMARY.md
```

Snapshot 汇总 closure state、code audit snapshot、README、ROADMAP、MVP Spec、architecture docs、contracts、validation docs 和 `verification.md` 的 docs refresh 输入、缺口、required updates 和 blockers。它只生成 docs refresh input package，不创建 `.agentflow/audits/`，不修改文档，不调用模型，不标记 Project done。`agentflow project closure` 在 snapshot 存在时将 docs refresh gate 显示为 `snapshot-ready`，但该状态仍不等于 final Root Docs Refresh passed。

## Desktop Boundary

当前阶段不新增 Desktop UI。

后续 Desktop 只能只读展示：

- Project closure status
- Code audit snapshot
- Docs refresh snapshot
- Final evidence summary
- Human approval required badge

Desktop 不允许：

- 执行 audit
- 执行 docs refresh
- 标记 Project done
- 创建或修改 issue
- 调用模型
- 创建远程 PR / issue

## 不允许

- 自动修改代码。
- 自动修复 audit findings。
- 自动标记 Project done。
- 自动接受 high-risk finding。
- 调用模型。
- 创建远程 PR / GitHub issue / Linear issue。
- 从 Desktop 执行 audit / docs refresh。
- 绕过 Workflow Control Core。
- 绕过 IssueContract 写 `.agentflow/`。

## 验证要求

边界定义阶段必须验证：

- boundary spec 存在。
- README / ROADMAP / MVP Spec / Workflow Control Core Spec / AI Delivery Workflow Contract 已引用该边界。
- `agentflow goal next` 指向下一条 closure gate 实现切片。
- readiness script 包含 Project Audit / Docs Refresh anchor。
- 不新增 Desktop UI。
- 不创建 `.agentflow/audits/`。
- 不创建 project closure writer 输出。

## 当前下一步

`Project Code Audit Snapshot v0` 已完成后，下一候选实现切片：

```bash
agentflow plan "Root Docs Refresh Snapshot v0 只读实现"
```
