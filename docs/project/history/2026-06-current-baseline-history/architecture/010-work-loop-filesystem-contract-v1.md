# Work Loop Filesystem Contract V1

创建日期：2026-06-21  
执行者：Codex

## 1. Purpose

本文档定义 AgentFlow Work Loop 的文件合同。

目标只有一个：

```text
让 Work Loop 的执行阶段不再依赖聊天线程、临时脚本或隐式约定，
而是明确落到稳定的 authority / artifact / event / projection 路径。
```

它服务于 `v0.6.0 / V060-001`。

## 2. Contract Boundary

Work Loop 的权威入口仍然是：

- `.agentflow/spec/issues/<issue-id>.json`

Work Loop 只是在这个 authority 之下，补齐执行阶段的稳定合同：

- work command
- action proposal
- preflight
- handoff
- work session
- evidence
- public delivery

## 3. Path Surface

| Surface | Path / Location | Authority Class | 说明 |
| --- | --- | --- | --- |
| Spec Issue | `.agentflow/spec/issues/<issue-id>.json` | authority | Work Loop 唯一任务权威 |
| Work Command | `.agentflow/runtime/commands/<command-id>.json` | authority | runtime 执行入口 |
| Action Proposal | `.agentflow/runtime/proposals/<proposal-id>.json` | authority | 关键动作提案 |
| Proposal Decision | `.agentflow/runtime/decisions/<proposal-id>.json` | derived | proposal 是否被接受 |
| Accepted Action | `.agentflow/runtime/actions/<accepted-action-id>.json` | derived | 真正进入执行面的动作事实 |
| Work Loop Contract | `.agentflow/tasks/<issue-id>/work-loop-contract.json` | derived | 当前 issue 的文件合同快照 |
| Handoff Request | `.agentflow/tasks/<issue-id>/runs/<run-id>/launch/agent-request.json` | transport | 只做 transport snapshot，不替代 authority |
| Work Action Proposal Contract | `.agentflow/tasks/<issue-id>/runs/<run-id>/launch/work-action-proposals.json` | derived | Work Agent 关键动作提案合同 |
| Preflight Report | `.agentflow/tasks/<issue-id>/runs/<run-id>/preflight/preflight.json` | derived | 执行前检查结果 |
| Run Record | `.agentflow/tasks/<issue-id>/runs/<run-id>/run.json` | derived | work session 主记录 |
| Command Records | `.agentflow/tasks/<issue-id>/runs/<run-id>/commands/<command-id>.json` | derived | 命令与 stdout/stderr 记录 |
| Checkpoints | `.agentflow/tasks/<issue-id>/runs/<run-id>/checkpoints/<checkpoint-id>.json` | derived | durable session 恢复点 |
| Validation Record | `.agentflow/tasks/<issue-id>/runs/<run-id>/validation.json` | derived | 验证命令结果 |
| Changed Files | `.agentflow/tasks/<issue-id>/runs/<run-id>/changed-files.json` | derived | 变更文件摘要 |
| Task Evidence | `.agentflow/tasks/<issue-id>/evidence/evidence.json` | authority | 本地验证事实出口 |
| Event Stream | `.agentflow/events/task-events.jsonl` | authority | Work Loop 事件事实流 |
| Task Projection | `.agentflow/projections/tasks/<issue-id>.json` | read_model | Desktop / query 的只读视图 |
| PR / MR Body | `public://pr-or-mr-body` | public_record | 公开交付记录 |
| CHANGELOG | `CHANGELOG.md` | public_record | 版本公开交付记录 |
| Release Notes | `public://release-notes` | public_record | 发布记录 |

## 4. Stage Model

| Stage | Issue Status | Stable Inputs | Stable Outputs | Stable Evidence |
| --- | --- | --- | --- | --- |
| command | `todo` | spec issue | work command | - |
| proposal | `todo`, `in_progress` | work command, work action proposal contract | proposal, decision, accepted action | decision |
| preflight | `todo` | spec issue, accepted action | preflight report | preflight report |
| handoff | `todo`, `in_progress` | spec issue, work command | handoff request | handoff request |
| session | `in_progress` | accepted action, preflight report, handoff request | run, command records, checkpoints, task event stream | checkpoints, task event stream |
| evidence | `in_review`, `done` | run, command records, changed files | validation, task evidence | validation, task evidence |
| delivery | `in_review`, `done` | task evidence, task projection | PR/MR body, changelog, release notes | task projection |

## 5. Authority Rules

### 5.1 Issue authority

`spec issue` 永远是任务权威。

以下内容都不能替代它：

- handoff request
- runtime command bundle
- 聊天线程
- provider session
- projection

### 5.2 Handoff rule

handoff request 只是一份 transport snapshot。

它的作用是把当前 issue、run、branch、provider、workflowRef 交给外部 Work Agent。
如果 handoff 和 spec issue 冲突，必须以 spec issue 为准。

### 5.3 Evidence rule

`task evidence` 是本地验证事实出口，但不是公开交付记录。

公开交付只认：

- PR/MR body
- `CHANGELOG.md`
- release notes

因此 `.agentflow/tasks/<issue-id>/delivery/**` 不再作为 authority surface。

## 6. Naming Convergence

Work Loop 的规范角色名是：

```text
work-agent
```

当前系统中允许兼容读取：

```text
build-agent
```

规则是：

- runtime / workflow / task-loop 以 `work-agent` 为规范角色；
- 任务包、旧文档或外部 provider 若仍写 `build-agent`，视为 `work-agent` 的兼容别名；
- audit agent 不属于 Work Loop 的执行角色。

## 7. Non-goals

本合同不做这些事情：

- 不启动 Work Agent
- 不定义完整多 Agent 并发协议
- 不做 UI 呈现
- 不重新引入 `.agentflow/input/**`
- 不重新引入 `.agentflow/execute/**`
- 不重新引入 `.agentflow/output/**`

## 8. Runtime Effect

落地后，Work Loop 至少具备三层可追溯性：

```text
spec issue
-> runtime command / proposal / decision / action
-> task run / validation / evidence
-> event stream / projection
-> public delivery
```

这意味着后续 `V060-002` 到 `V060-012` 都必须沿着这组稳定路径继续推进，不能再临时发明新 authority surface。
