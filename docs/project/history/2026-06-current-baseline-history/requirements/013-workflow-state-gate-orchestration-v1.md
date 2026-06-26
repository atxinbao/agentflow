# 013 - Workflow State / Gate Orchestration V1

创建日期：2026-06-04  
执行者：Codex  
状态：待开发  
版本：final-draft

---

## 用户目标

当前 AgentFlow 已经完成 / 正在推进：

```text
define/
= Agent 工作手册 / 规则

panel/
= 项目工作现场

input/
= 需求实时源头，包含 Spec Gate / Projects / Issues

execute/
= 受控执行流水线，包含 run / preflight / lease / plan / checkpoint / patch / command / validation / result

output/
= 交付与证据层，包含 evidence / release / audit

state/
= 当前只有基础目录，还没有成为真正的总控状态层
```

现在需要推进：

```text
state/
```

目标不是再新增一个业务事实源，而是把已经成型的：

```text
define
panel
input
execute
output
audit
```

统一聚合成一个可读、可判断、可展示的工作流状态层。

大白话：

> **state 不是新业务数据，不替代 input / execute / output。**  
> **state 是 AgentFlow 的总控看板：告诉 Agent 和 Desktop 当前系统走到哪、下一步能做什么、为什么不能做、谁正在做、有没有需要人类确认。**

---

## 一句话定义

> **Workflow State / Gate Orchestration V1 是 AgentFlow 的全局状态总控层。它聚合 define / panel / input / execute / output / audit 的健康状态，生成 workflow gates、next actions、blockers、session、locks、events 和 indexes，让 Agent 和 Desktop 能清楚知道当前阶段、允许动作、阻断原因、活跃 issue / run、delivery 状态和 audit 状态。**

---

# 1. 参考原则

## 1.1 从 Warp 吸收：显式化工作前置条件和 gate

Warp 的项目工作方式非常明确：

```text
bootstrap
skills-lock
development commands
presubmit
testing
PR workflow
review gate
```

对 AgentFlow 的启发：

```text
不要等 Agent 执行时才发现环境坏了。
应该提前在 state 中显式记录：
- 环境是否 ready
- 技能 / 手册是否 ready
- panel 是否 ready
- input 是否 ready
- execute 是否 ready
- output 是否 ready
- 当前是否允许进入下一步
```

对应 AgentFlow：

```text
state/health/
state/gates/
```

需要回答：

```text
define ready?
panel ready?
input ready?
execute ready?
output ready?
ownership safe?
git protected?
next action allowed?
blocked reason?
```

---

## 1.2 从 Zed 吸收：显式化 session / queue / checkpoint / review / lock 状态

Zed 的 Agent 工作心智里有：

```text
agent threads
queued messages
tool permissions
checkpoint
review changes
terminal task state
parallel worktree isolation
```

对 AgentFlow 的启发：

```text
state 要记录当前正在发生什么，而不是只保存最终结果。
```

对应 AgentFlow：

```text
当前 active session 是哪个？
当前 active issue 是哪个？
当前 active run 是哪个？
有没有 queued action？
有没有 waiting human confirmation？
有没有 stale lease？
有没有 checkpoint 可恢复？
有没有 delivery ready？
有没有 audit requested？
```

---

# 2. 核心原则

## 2.1 state 是派生状态，不是事实源

state 不存：

```text
需求事实
项目现场事实
执行过程事实
交付证据事实
审计报告事实
```

这些已经分别属于：

```text
input/
panel/
execute/
output/
```

state 只存：

```text
健康状态
当前 gate
当前 session
当前 lock 摘要
当前 next action
当前 blocked reason
当前 timeline
当前 indexes
```

一句话：

> **state 是看板和闸门，不是事实源。**

---

## 2.2 state 不自动执行动作

state 只判断和记录，不直接执行。

不允许：

```text
不自动执行下一个 issue
不自动触发 audit
不自动清理 stale locks
不自动恢复 checkpoint
不自动创建 PR
不自动 merge
不自动 deploy
不自动调用模型
```

允许：

```text
聚合状态
判断 gate
记录 session
记录 lock summary
追加 timeline event
生成索引
给 Desktop 展示
```

---

## 2.3 audit 不是默认 required

Audit 已经定为：

```text
人类触发才生成完整审计报告
```

所以 state 不应该把每次 delivery 后都标成：

```text
needsAudit = true
```

而应该记录：

```text
auditStatus = not-requested
```

只有人类触发后才变为：

```text
requested
running
passed
passed-with-warnings
failed
cancelled
```

---

# 3. state 目标结构

```text
.agentflow/state/
├── manifest.json
├── index.json
│
├── health/
│   ├── workspace.json
│   ├── define.json
│   ├── panel.json
│   ├── input.json
│   ├── execute.json
│   ├── output.json
│   └── audit.json
│
├── gates/
│   ├── workflow.json
│   ├── next-actions.json
│   └── blockers.json
│
├── sessions/
│   └── <session-id>.json
│
├── locks/
│   ├── active.json
│   ├── stale.json
│   └── cleanup-candidates.json
│
├── events/
│   └── timeline.jsonl
│
└── indexes/
    ├── workspace-status.json
    ├── issue-status.json
    ├── run-status.json
    └── output-status.json
```

---

# 4. state manifest

## 4.1 路径

```text
.agentflow/state/manifest.json
```

## 4.2 职责

记录 state 层自身状态：

```text
state 是否 ready
canonical paths
summary
updatedAt
```

## 4.3 示例

```json
{
  "version": "state-manifest.v1",
  "projectRoot": "/path/to/project",
  "status": "ready",
  "updatedAt": 1780360000,
  "paths": {
    "health": ".agentflow/state/health",
    "gates": ".agentflow/state/gates",
    "sessions": ".agentflow/state/sessions",
    "locks": ".agentflow/state/locks",
    "events": ".agentflow/state/events",
    "indexes": ".agentflow/state/indexes"
  },
  "summary": {
    "healthReady": true,
    "currentStage": "delivery-ready",
    "allowedNextActions": 2,
    "blockedActions": 1,
    "activeSessions": 1,
    "activeLocks": 0,
    "staleLocks": 0,
    "auditStatus": "not-requested"
  }
}
```

---

# 5. state index

## 5.1 路径

```text
.agentflow/state/index.json
```

## 5.2 职责

快速索引 state 产物：

```text
health
gates
sessions
locks
events
indexes
```

## 5.3 示例

```json
{
  "version": "state-index.v1",
  "updatedAt": 1780360000,
  "health": [
    {
      "module": "input",
      "status": "ready",
      "path": ".agentflow/state/health/input.json"
    }
  ],
  "sessions": [],
  "locks": [],
  "events": {
    "timeline": ".agentflow/state/events/timeline.jsonl"
  }
}
```

---

# 6. health/

## 6.1 作用

```text
health/
= 各模块健康状态
```

它聚合：

```text
ownership
define
panel
input
execute
output
audit
```

大白话：

> **health 告诉你每一块有没有坏。**

---

## 6.2 health 文件

```text
.agentflow/state/health/workspace.json
.agentflow/state/health/define.json
.agentflow/state/health/panel.json
.agentflow/state/health/input.json
.agentflow/state/health/execute.json
.agentflow/state/health/output.json
.agentflow/state/health/audit.json
```

---

## 6.3 HealthSnapshot 示例

```json
{
  "version": "state-health.v1",
  "module": "input",
  "status": "ready",
  "ready": true,
  "sourcePath": ".agentflow/input/manifest.json",
  "checkedAt": 1780360000,
  "warnings": [],
  "errors": []
}
```

---

## 6.4 health 聚合规则

```text
ownership foreign / blocked
→ workspace health = blocked

define failed
→ workspace health = failed

panel degraded
→ workspace health = degraded，但不一定 blocked

input missing
→ input health = missing

execute active run exists
→ execute health = working

output incomplete evidence / delivery
→ output health = degraded

audit not requested
→ audit health = idle
```

---

# 7. gates/

## 7.1 作用

```text
gates/
= 工作流闸门判断
```

它回答：

```text
现在能不能写 input？
现在能不能 execute？
现在能不能 delivery？
现在能不能 request audit？
现在卡在哪里？
```

大白话：

> **gates 告诉 Agent：下一步能干什么，不能干什么。**

---

## 7.2 workflow.json

路径：

```text
.agentflow/state/gates/workflow.json
```

示例：

```json
{
  "version": "state-workflow-gates.v1",
  "currentStage": "delivery-ready",
  "auditStatus": "not-requested",
  "activeIssueId": "iss-001",
  "activeRunId": "run-001",
  "latestEvidencePath": ".agentflow/output/evidence/run-001.json",
  "latestDeliveryPath": ".agentflow/output/release/run-001/delivery.json",
  "allowedNextActions": [
    "request-human-audit",
    "start-new-input"
  ],
  "blockedActions": [
    {
      "action": "execute-issue",
      "reason": "No selected issue."
    }
  ],
  "updatedAt": 1780360000
}
```

---

## 7.3 next-actions.json

路径：

```text
.agentflow/state/gates/next-actions.json
```

示例：

```json
{
  "version": "state-next-actions.v1",
  "actions": [
    {
      "action": "request-human-audit",
      "label": "Request human audit",
      "allowed": true,
      "reason": "Release delivery is ready and audit has not been requested."
    },
    {
      "action": "start-new-input",
      "label": "Start new requirement intake",
      "allowed": true,
      "reason": "Current delivery is complete."
    }
  ]
}
```

---

## 7.4 blockers.json

路径：

```text
.agentflow/state/gates/blockers.json
```

示例：

```json
{
  "version": "state-blockers.v1",
  "blockers": [
    {
      "action": "execute-issue",
      "reason": "High risk issue requires human confirmation.",
      "sourcePath": ".agentflow/execute/runs/run-001/preflight.json"
    }
  ]
}
```

---

# 8. Workflow Gate 规则

## 8.1 currentStage

V1 支持：

```text
workspace-missing
workspace-blocked
workspace-ready
panel-ready
input-ready
issue-ready
execute-ready
execute-running
execute-blocked
execute-completed
evidence-ready
delivery-ready
audit-requested
audit-running
audit-completed
failed
```

---

## 8.2 gate 关系

```text
ownership + define ready
→ workspace-ready

panel ready / degraded
→ panel-ready

input has approved spec + issue
→ issue-ready

issue low / medium
→ execute-ready

issue high + confirmation missing
→ execute-blocked

issue high + confirmation exists
→ execute-ready

execute completed + result exists
→ execute-completed

evidence exists and valid
→ evidence-ready

release delivery exists and valid
→ delivery-ready

human requested audit
→ audit-requested
```

---

## 8.3 auditStatus

取值：

```text
not-requested
requested
running
passed
passed-with-warnings
failed
cancelled
```

默认：

```text
not-requested
```

重要：

```text
delivery-ready 不等于 needsAudit。
delivery-ready 只表示可以请求审计。
```

---

# 9. sessions/

## 9.1 作用

```text
sessions/
= 当前人类 / Agent 会话状态
```

它记录：

```text
sessionId
projectRoot
activeRole
activeIssueId
activeRunId
status
waitingForHuman
lastAction
updatedAt
```

大白话：

> **sessions 记录这次工作对话现在走到哪。**

---

## 9.2 session 示例

```json
{
  "version": "state-session.v1",
  "sessionId": "session-001",
  "projectRoot": "/path/to/project",
  "activeRole": "Build Agent",
  "activeIssueId": "iss-001",
  "activeRunId": "run-001",
  "status": "waiting-human-confirmation",
  "waitingForHuman": true,
  "lastAction": "execute.preflight.blocked",
  "updatedAt": 1780360000
}
```

---

## 9.3 V1 session 行为

V1 支持：

```text
create / update session
load current session
mark waitingForHuman
mark idle
```

V1 不做：

```text
多 Agent 并发调度
自动恢复 session
自动继续执行
```

---

# 10. locks/

## 10.1 作用

```text
locks/
= 全局锁状态摘要
```

注意：

```text
execute/leases/
= 真实 lock 事实源

state/locks/
= lock 看板
```

---

## 10.2 文件

```text
.agentflow/state/locks/active.json
.agentflow/state/locks/stale.json
.agentflow/state/locks/cleanup-candidates.json
```

---

## 10.3 active.json 示例

```json
{
  "version": "state-locks.v1",
  "active": [
    {
      "kind": "execute-lease",
      "issueId": "iss-001",
      "runId": "run-001",
      "sourcePath": ".agentflow/execute/leases/iss-001.json"
    }
  ]
}
```

---

## 10.4 stale lock 判断

V1 可以只判断：

```text
lease.status = active
但对应 run 不存在
→ stale

lease.status = active
但 run.status = completed / failed / cancelled
→ stale

lease 文件不可读
→ cleanup candidate
```

V1 不自动清理。

---

# 11. events/

## 11.1 作用

```text
events/timeline.jsonl
= 工作流事件时间线
```

大白话：

> **events 是系统流水账，方便回放发生过什么。**

---

## 11.2 路径

```text
.agentflow/state/events/timeline.jsonl
```

---

## 11.3 事件示例

```json
{"ts":1780360000,"event":"workspace.prepared","projectRoot":"/path/to/project"}
{"ts":1780360020,"event":"panel.indexed","snapshotId":"panel-snapshot-001"}
{"ts":1780360100,"event":"input.issue.created","issueId":"iss-001"}
{"ts":1780360200,"event":"execute.run.created","issueId":"iss-001","runId":"run-001"}
{"ts":1780360300,"event":"execute.checkpoint.created","runId":"run-001","checkpointId":"chk-001"}
{"ts":1780360400,"event":"output.evidence.written","runId":"run-001"}
{"ts":1780360500,"event":"output.release.prepared","runId":"run-001"}
{"ts":1780360600,"event":"audit.requested","auditId":"audit-001"}
```

---

## 11.4 V1 事件行为

V1 只做：

```text
append timeline event
load timeline
```

不做：

```text
event sourcing
事件驱动自动执行
事件回放恢复状态
```

---

# 12. indexes/

## 12.1 作用

```text
indexes/
= 快速查询状态索引
```

它不是事实源，只是聚合视图。

---

## 12.2 文件

```text
.agentflow/state/indexes/workspace-status.json
.agentflow/state/indexes/issue-status.json
.agentflow/state/indexes/run-status.json
.agentflow/state/indexes/output-status.json
```

---

## 12.3 issue-status.json 示例

```json
{
  "version": "state-issue-status-index.v1",
  "issues": [
    {
      "issueId": "iss-001",
      "riskLevel": "medium",
      "latestRunId": "run-001",
      "executeStatus": "completed",
      "evidenceStatus": "ready",
      "deliveryStatus": "drafted",
      "auditStatus": "not-requested"
    }
  ]
}
```

---

# 13. State Status Snapshot

新增：

```text
StateStatusSnapshot
```

建议字段：

```json
{
  "version": "state-status.v1",
  "projectRoot": "/path/to/project",
  "status": "ready",
  "currentStage": "delivery-ready",
  "auditStatus": "not-requested",
  "activeIssueId": "iss-001",
  "activeRunId": "run-001",
  "health": {
    "workspace": "ready",
    "define": "ready",
    "panel": "ready",
    "input": "ready",
    "execute": "ready",
    "output": "ready",
    "audit": "idle"
  },
  "nextActions": [
    "request-human-audit",
    "start-new-input"
  ],
  "blockers": [],
  "updatedAt": 1780360000
}
```

---

# 14. State 和 Desktop

Desktop 最后应该看：

```text
state/status
```

而不是每次自己拼：

```text
define status
panel status
input status
execute status
output status
```

V1 Desktop 展示：

```text
Workflow State
- Current Stage
- Next Actions
- Blockers
- Active Issue
- Active Run
- Evidence
- Delivery
- Audit Status
```

状态通道新增：

```text
工作流状态
```

指标：

```text
Stage
Next Actions
Blockers
Audit Status
Active Run
```

---

# 15. State 和 Audit 的关系

Audit 不自动跑。

state 只记录：

```text
auditStatus
```

默认：

```text
not-requested
```

人类触发 audit 后：

```text
requested
```

Audit 执行中：

```text
running
```

Audit 完成：

```text
passed
passed-with-warnings
failed
cancelled
```

---

# 16. Rust 模块建议

建议新增 crate：

```text
crates/state/
```

package：

```text
agentflow-state
```

结构：

```text
crates/state/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── model.rs
    ├── manager.rs
    ├── storage.rs
    ├── health.rs
    ├── gates.rs
    ├── sessions.rs
    ├── locks.rs
    ├── events.rs
    └── indexes.rs
```

依赖：

```text
agentflow-agent-manual
agentflow-panel
agentflow-input
agentflow-execute
agentflow-output
```

---

# 17. Tauri commands

新增：

```text
prepare_state_workspace
load_state_status
load_state_manifest
load_state_index
load_workflow_gates
load_next_actions
load_blockers
load_state_timeline
append_state_event
load_state_session
update_state_session
load_state_locks
refresh_state
```

Desktop V1：

```text
只读展示为主
允许 append event / update session 由 Agent 系统调用
不允许人类手动修改状态文件
```

---

# 18. Project Workspace prepare 接入

Project Workspace prepare 应该最终顺序：

```text
ownership
define
panel
input
execute
output
state
```

state prepare 应该在最后，因为它要聚合前面所有层。

---

# 19. 写入边界

## 19.1 允许写

```text
.agentflow/state/**
```

## 19.2 允许读

```text
.agentflow/define/**
.agentflow/panel/**
.agentflow/input/**
.agentflow/execute/**
.agentflow/output/**
```

## 19.3 不允许写

```text
.agentflow/input/**
.agentflow/panel/**
.agentflow/execute/**
.agentflow/output/**
用户源码
远程 PR
远程 issue
生产服务
```

---

# 20. 非目标

本需求不做：

```text
不自动执行 issue
不自动触发 audit
不自动清理 stale locks
不自动恢复 checkpoint
不创建 PR
不 merge
不 deploy
不调用模型
不实现复杂调度器
不做事件回放状态恢复
不写用户源码
```

---

# 21. 开发切片

## Slice 1：State layout

```text
state/manifest.json
state/index.json
state/health/
state/gates/
state/sessions/
state/locks/
state/events/
state/indexes/
```

---

## Slice 2：Health aggregation

读取：

```text
agent manual status
panel status
input status
execute status
output status
ownership status
```

写：

```text
state/health/*.json
state/indexes/workspace-status.json
```

---

## Slice 3：Gate evaluation

写：

```text
state/gates/workflow.json
state/gates/next-actions.json
state/gates/blockers.json
```

输出：

```text
currentStage
allowedNextActions
blockedActions
auditStatus
```

---

## Slice 4：Session state

写：

```text
state/sessions/<session-id>.json
```

支持：

```text
load current session
update activeRole
update activeIssueId
update activeRunId
mark waitingForHuman
mark idle
```

---

## Slice 5：Lock summary

读取：

```text
execute/leases
```

写：

```text
state/locks/active.json
state/locks/stale.json
state/locks/cleanup-candidates.json
```

---

## Slice 6：Event timeline

写：

```text
state/events/timeline.jsonl
```

支持：

```text
append event
load timeline
```

---

## Slice 7：Desktop state status

新增：

```text
load_state_status
load_workflow_gates
load_state_timeline
```

Desktop 展示：

```text
当前阶段
下一步建议
阻断原因
active issue / run
audit status
```

---

# 22. 测试要求

必须新增测试：

```text
1. prepare_state_workspace creates manifest / index / required directories.
2. state prepare does not write input / panel / execute / output.
3. health aggregation reads define / panel / input / execute / output statuses.
4. workflow gate returns workspace-ready when core layers are ready.
5. workflow gate returns delivery-ready when evidence and release delivery exist.
6. auditStatus defaults to not-requested.
7. auditStatus does not become required automatically after delivery.
8. high risk blocked preflight appears in blockers.
9. active execute lease appears in state/locks/active.json.
10. released lease does not appear in active locks.
11. stale active lease appears in stale.json when run is completed / failed / cancelled.
12. append_state_event writes timeline.jsonl.
13. session update writes state/sessions/<session-id>.json.
14. indexes/issue-status.json aggregates issue / run / evidence / delivery / audit status.
```

---

# 23. 验收标准

```text
- [ ] 新增 docs/requirements/013-workflow-state-gate-orchestration-v1.md。
- [ ] 新增 crates/state package agentflow-state。
- [ ] 创建 .agentflow/state/manifest.json。
- [ ] 创建 .agentflow/state/index.json。
- [ ] 创建 state/health/。
- [ ] 创建 state/gates/。
- [ ] 创建 state/sessions/。
- [ ] 创建 state/locks/。
- [ ] 创建 state/events/timeline.jsonl。
- [ ] 创建 state/indexes/。
- [ ] StateManifest 实现。
- [ ] StateIndex 实现。
- [ ] StateStatusSnapshot 实现。
- [ ] WorkflowHealthSnapshot 实现。
- [ ] WorkflowGateSnapshot 实现。
- [ ] StateSession 实现。
- [ ] StateLockSnapshot 实现。
- [ ] StateTimelineEvent 实现。
- [ ] health 聚合 define / panel / input / execute / output。
- [ ] gates 生成 currentStage。
- [ ] gates 生成 allowedNextActions。
- [ ] gates 生成 blockedActions。
- [ ] auditStatus 默认 not-requested。
- [ ] delivery-ready 不自动变成 needsAudit。
- [ ] active lease 出现在 state/locks/active.json。
- [ ] released lease 不出现在 active locks。
- [ ] stale lock 出现在 stale.json。
- [ ] timeline.jsonl 支持 append。
- [ ] sessions 支持 update / load。
- [ ] indexes 聚合 issue / run / output 状态。
- [ ] Desktop 只读展示 state status。
- [ ] Browser Preview mock 更新。
- [ ] Project Workspace prepare 最后接入 state prepare。
- [ ] state 不写 input。
- [ ] state 不写 panel。
- [ ] state 不写 execute。
- [ ] state 不写 output。
- [ ] state 不写用户源码。
- [ ] state 不自动执行 issue。
- [ ] state 不自动触发 audit。
- [ ] state 不调用模型。
- [ ] cargo fmt --check 通过。
- [ ] cargo test -p agentflow-state 通过。
- [ ] cargo test -p agentflow-desktop 通过。
- [ ] cargo test 通过。
- [ ] npm --prefix apps/desktop run build 通过。
- [ ] git diff --check 通过。
```

---

# 24. 验证命令

```bash
cargo fmt --check
cargo test -p agentflow-state
cargo test -p agentflow-desktop
cargo test
npm --prefix apps/desktop run build
git diff --check
```

---

# 25. PR 说明要求

PR 描述必须说明：

```text
1. state 为什么是派生状态，不是事实源。
2. state 如何聚合 define / panel / input / execute / output。
3. workflow gates 如何判断 next actions / blockers。
4. auditStatus 为什么默认 not-requested。
5. delivery-ready 为什么不自动 required audit。
6. sessions / locks / events / indexes 各自的职责。
7. state 不自动执行 issue。
8. state 不自动触发 audit。
9. state 不修改 input / execute / output。
10. state 不调用模型。
11. 验证命令和结果。
```

---

# 26. Codex 执行指令

```md
请执行 013 - Workflow State / Gate Orchestration V1。

目标：
实现 AgentFlow 的全局状态总控层 `.agentflow/state/`。state 是派生状态，不是事实源。它聚合 define / panel / input / execute / output / audit 的健康状态，生成 workflow gates、next actions、blockers、sessions、locks、events 和 indexes，让 Desktop 和 Agent 能知道当前阶段、允许动作、阻断原因、活跃 issue / run、delivery 状态和 audit 状态。

必须遵守：
1. state 是派生状态，不是事实源。
2. state 可以读 define / panel / input / execute / output。
3. state 只写 .agentflow/state/**。
4. state 不写 input。
5. state 不写 panel。
6. state 不写 execute。
7. state 不写 output。
8. state 不写用户源码。
9. state 不自动执行 issue。
10. state 不自动触发 audit。
11. state 不自动清理 stale locks。
12. state 不自动恢复 checkpoint。
13. auditStatus 默认 not-requested。
14. delivery-ready 不等于 needsAudit。
15. Desktop 只读展示 state status。
16. 不调用模型。

实现范围：
- 新增 docs/requirements/013-workflow-state-gate-orchestration-v1.md。
- 新增 crates/state package agentflow-state。
- 新增 state manifest / index / status / snapshot。
- 新增 health aggregation。
- 新增 workflow gates / next actions / blockers。
- 新增 session state。
- 新增 lock summary。
- 新增 timeline event。
- 新增 state indexes。
- 新增 Tauri read commands。
- Project Workspace prepare 最后接入 state prepare。
- Desktop 状态通道新增 Workflow State。
- Browser Preview mock 更新。
- README / GOAL / ROADMAP / requirements / verification 更新。

验证命令：
- cargo fmt --check
- cargo test -p agentflow-state
- cargo test -p agentflow-desktop
- cargo test
- npm --prefix apps/desktop run build
- git diff --check
```

---

# 27. 完成定义

本需求完成后：

```text
state/
= AgentFlow 的总控状态层
```

它能回答：

```text
现在在哪里？
下一步能做什么？
为什么不能做？
谁正在做？
有没有需要人类确认？
有没有 active / stale lock？
delivery 是否 ready？
audit 是否 requested？
```

主流程依旧是：

```text
define
→ panel
→ input
→ execute
→ output
→ state
```

最终一句话：

> **State V1 把 AgentFlow 从“各层都有状态”升级为“全局可判断的工作流状态”：健康、闸门、会话、锁、事件、索引全部聚合到 state，但 state 不替代任何事实源，也不自动执行任何动作。**
