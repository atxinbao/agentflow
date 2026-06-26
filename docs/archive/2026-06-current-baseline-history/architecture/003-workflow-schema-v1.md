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
    skillPack: execution-skills
  todo:
    role: work-agent
    skillPack: execution-skills
  in_progress:
    role: work-agent
    skillPack: execution-skills
  in_review:
    role: work-agent
    skillPack: execution-skills
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
    handoff:
      fromRole: work-agent
      toRole: system
      mode: ownership-transfer
      payloadRef: mergeProofRef
      expectedState: done
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

补充规则：

- 只要 transition 会改变 authority role，就必须显式声明 handoff。
- `ownership-transfer` 表示 authority 从一个主角色交给另一个角色。
- `bounded-capability-call` 表示 authority 不变，只允许调用 `specialist`。
- `bounded-capability-call` 不能把 target state 的 authority 改成 `specialist`。

## Project Flow 与 Work Flow 的分界

必须定死：

- Project Flow 负责“哪个阶段、哪条 issue、什么时候进入交付或目标回看”
- Work Flow 负责“单条 issue 如何从 backlog 走到 done”

Project Flow 不能代替 Work Flow 执行单任务。

补充边界：

- Audit Flow 是独立 Sidecar Loop，不在 Project 主链上阻断 Work Done、Delivery Package 或 Completion Commit。
- Project Flow 可以展示 audit summary，但不能把 `audit.passed` 作为进入交付或完成的前置条件。
- Audit Finding 只能回流为 Follow-up Proposal，不能直接修改 issue / project / delivery authority。
Work Flow 不能自己决定下一条 issue。

## 不做事项

- 不用自由群聊当 workflow。
- 不把 provider session 状态直接当 workflow state。
- 不让 Desktop 页面点击直接改业务状态。
- 不让 Agent 自己声明“我已经 done”而绕过 runtime。
