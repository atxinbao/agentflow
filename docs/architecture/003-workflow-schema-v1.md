# 003 - Workflow Schema V1

创建日期：2026-06-18
执行者：Codex

## Purpose

本文定义 AgentFlow 的统一流程 schema。

目标是把 Project / Work / Audit / Delivery 四类流程从“散落在代码里的判断逻辑”收口为统一模型。

## 核心原则

1. Workflow 负责控制流，不负责执行业务动作。
2. State 跟着 Workflow 走，不跟着 Agent 走。
3. Event 触发状态推进，不能由页面刷新直接推进。
4. Project Flow 和 Work Flow 必须分开。

## Workflow Types

AgentFlow 第一版固定四类流程：

- `project`
- `work`
- `audit`
- `delivery`

## Flow 实例层级

### Project Flow

项目级流程，负责阶段和 issue 调度。

建议状态：

```text
intake
goal_draft
plan_draft
confirmed
working
auditing
delivering
goal_recheck
paused
accepted
blocked
```

### Work Flow

单个 issue 的执行流程。

建议状态：

```text
backlog
todo
in_progress
in_review
done
blocked
cancel
```

### Audit Flow

独立审计流程。

建议状态：

```text
pending
ready
in_progress
passed
needs_repair
blocked
cancel
```

### Delivery Flow

交付整理流程。

建议状态：

```text
pending
ready
in_progress
published
returned
blocked
cancel
```

## Workflow Schema

统一 schema 建议包含：

```yaml
id: work.workflow.v1
flowType: work
version: 1
initialState: backlog
terminalStates:
  - done
  - cancel
states:
  backlog:
    role: work-agent
  todo:
    role: work-agent
  in_progress:
    role: work-agent
  in_review:
    role: work-agent
  done:
    role: system
transitions:
  - from: backlog
    to: todo
    guard: issue_preflight_ready
    action: enter_todo
  - from: todo
    to: in_progress
    guard: context_pack_ready
    action: start_run
  - from: in_progress
    to: in_review
    guard: verification_completed
    action: prepare_review
  - from: in_review
    to: done
    guard: merge_confirmed
    action: complete_issue
```

## Guard 类型

统一支持以下 guard 类型：

- contract-complete
- dependency-ready
- context-pack-ready
- workspace-clean
- provider-capable
- evidence-complete
- audit-passed
- delivery-ready
- human-confirmed

## Action 类型

统一支持以下 action 类型：

- append-event
- create-run
- request-context-pack
- launch-agent-session
- prepare-review
- write-merge-proof
- complete-issue
- open-audit
- publish-delivery
- request-goal-recheck

## Human Gates

Workflow Schema 必须支持人类确认点。

典型 gate：

- Goal Draft confirm
- Plan Draft confirm
- Project Entry confirm
- Work Entry confirm
- Scope change confirm
- Final acceptance confirm

规则：

- Human gate 是 workflow 的一等条件；
- 不能由 UI 自己绕过；
- 不能用 provider session 替代。

## Pause / Resume / Retry / Cancel

所有 flow 都必须统一支持：

- `pause`
- `resume`
- `retry`
- `cancel`

含义：

- `pause`：暂时停止当前 flow，不丢上下文
- `resume`：从已有 checkpoint / projection 恢复
- `retry`：在同一阶段重新尝试
- `cancel`：终止 flow，进入终态

## Handoff in Workflow

handoff 必须成为 schema 显式能力，而不是 prompt 附属概念。

每次 handoff 至少要定义：

- `fromRole`
- `toRole`
- `mode`
- `payloadRef`
- `expectedState`

## Project Flow 与 Work Flow 的分界

必须定死：

- Project Flow 负责“哪个阶段、哪条 issue、什么时候进入审计或交付”
- Work Flow 负责“单条 issue 如何从 backlog 走到 done”

Project Flow 不能代替 Work Flow 执行单任务。
Work Flow 不能自己决定下一条 issue。

## 不做事项

- 不用自由群聊当 workflow。
- 不把 provider session 状态直接当 workflow state。
- 不让 Desktop 页面点击直接改业务状态。
- 不让 Agent 自己声明“我已经 done”而绕过 runtime。
