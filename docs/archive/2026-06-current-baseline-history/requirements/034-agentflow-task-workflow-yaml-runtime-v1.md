# AgentFlow Task Workflow YAML Runtime V1

> 文档类型：底层架构方案
> 根目录用途：作为 AgentFlow 后续任务流重构的架构基线
> 需求目录副本：`docs/requirements/034-agentflow-task-workflow-yaml-runtime-v1.md`
> 日期：2026-06-14
> 执行者：Codex
> 状态：Architecture Proposal

---

## 1. 背景

AgentFlow 早期底层是按目录能力拆分的：

```text
input
execute
output
state
```

新架构会把旧 input 任务合同层切换为 spec：

```text
docs/requirements
  -> .agentflow/spec/issues
```

这种拆分有利于落地文件结构，但当产品进入 Project Loop / Issue Loop / Build Agent 自动执行阶段后，用户真正关心的不是目录，而是：

```text
任务现在处于什么状态？
这个状态为什么进入？
这个状态正在干什么？
下一个状态是什么？
最后交付了什么？
```

因此底层主线需要从“输入 / 执行 / 输出”重构为：

```text
Task
  -> State
  -> Event
  -> Projection
  -> UI
```

也就是：任务合同来自 spec issue，但运行时事实由事件驱动，页面展示来自投影。

---

## 2. 一句话目标

用 YAML 定义任务工作流，用事件日志记录任务运行事实，用状态投影驱动任务页展示。

最终模型：

```text
Issue Contract
  引用 workflowRef

Workflow YAML
  定义状态、事件、守卫、动作、产物

Runtime
  消费事件，执行状态转移

Event Store
  记录 append-only 事实

Projection
  生成任务页、项目页、执行信息和交付信息
```

---

## 3. 核心原则

```text
1. spec issue 是任务合同，不是运行时日志。
2. YAML workflow 是流程定义，不是任务实例。
3. Event log 是运行事实源。
4. Projection 是 UI read model，可以重建，不作为事实源。
5. 执行 / 交付不再是一级主流程目录，而是任务状态流里的 artifacts。
6. Build Agent 只执行当前 issue，不自己决定状态机规则。
7. Desktop 只展示 projection，不直接修改任务状态。
8. 外部 provider 只负责能力调用，不成为任务源。
```

---

## 4. 顶层架构

```text
.agentflow/spec/issues/<issue-id>.json
  任务合同
  包含 workflowRef

.agentflow/workflows/<workflow-name>.yaml
  工作流定义
  包含 states / transitions / guards / actions

.agentflow/events/task-events.jsonl
  任务事件日志
  append-only

.agentflow/projections/tasks/<issue-id>.json
  单任务状态投影

.agentflow/projections/projects/<project-id>.json
  项目状态投影

.agentflow/indexes/issue-status.json
  任务列表索引

.agentflow/tasks/<issue-id>/runs/<run-id>/**
  当前任务的执行产物

.agentflow/tasks/<issue-id>/evidence/**
  当前任务的验证证据
```

目标 crates 架构：

```text
crates/core
  共享基础层。
  只放 id、time、path、error、serde helper 等无业务工具。
  不读写 .agentflow。
  不依赖其他 AgentFlow 业务 crate。

crates/cli
  命令入口层。
  只负责参数解析和调用下层 use case。
  不直接读写 .agentflow 业务文件。

crates/spec
  任务合同层。
  读取公开需求 docs/requirements/<requirement-id>.md。
  管理 .agentflow/spec/issues/<issue-id>.json。
  管理 .agentflow/spec/projects/<project-id>.json。
  不记录运行时状态。

crates/workflow-core
  工作流定义层。
  解析 .agentflow/workflows/<workflow-name>.yaml。
  定义 WorkflowDefinition / StateDefinition / TransitionDefinition。
  定义 GuardDefinition / ActionDefinition。
  不读写 issue，不执行 action。

crates/event-store
  事件事实层。
  写入 .agentflow/events/task-events.jsonl。
  提供 append / replay / event id / causation id / correlation id。
  不生成 UI 状态。

crates/workflow-runtime
  状态机运行层。
  根据 workflow + 当前 projection + incoming event 匹配 transition。
  执行 guard registry。
  调用 action registry。
  写入状态转移事件。
  不直接服务 Desktop UI。

crates/task-loop
  调度层。
  读取 spec project / issues。
  按依赖和 priority 找下一个可执行 issue。
  只发 issue.scheduled / agent.launch.requested 等事件。
  不直接执行代码。

crates/projection
  读模型层。
  从 event-store 重建 task projection。
  从 event-store 重建 project projection。
  生成 .agentflow/indexes/issue-status.json。
  生成 UI refresh signal。
  Desktop 只能读 projection。

crates/task-artifacts
  本地任务产物层。
  管理 .agentflow/tasks/<issue-id>/runs/<run-id>/**。
  管理 .agentflow/tasks/<issue-id>/evidence/**。
  不生成 delivery 目录。

crates/agent-dispatcher
  Agent 会话编排层。
  消费 agent.launch.requested。
  创建 / 恢复 / 轮询外部 Agent session。
  把 session running / interrupted / completed 写回 event-store。

crates/mcp
  外部 provider 适配层。
  封装 codex / future agent provider。
  封装 GitHub / GitLab PR/MR provider。
  只返回能力状态和调用结果，不推进任务状态。

crates/release
  版本发布层。
  汇总已合并 PR/MR 的公开交付记录。
  写 CHANGELOG.md。
  写 GitHub/GitLab Release notes。
  不参与单个 task loop。

crates/panel
  项目现场层。
  生成 Context Pack。
  提供文件 / 符号 / 测试建议。
  不推进 issue 状态。

crates/agent-manual
  Agent 规则层。
  生成角色手册、语言、风格和本地规则。
  不参与运行时状态机。

crates/acceptance
  端到端验收测试层。
  只作为测试 harness。
  不允许被 runtime crate 依赖。
```

旧模块处理：

```text
crates/input
  由 crates/spec 替代。
  不作为新架构入口。

crates/execute
  由 crates/task-artifacts + crates/workflow-runtime 替代。
  不作为新架构入口。

crates/output
  直接删除。
  不再用本地 output 目录承载任务交付。
  任务级公开交付记录写入 PR/MR body。
  版本级公开交付记录由 release 流程写入 CHANGELOG / Release notes。
  不新增 public-delivery crate。

crates/state
  由 crates/projection 替代。
  不作为新架构入口。

crates/loop
  由 crates/task-loop + crates/workflow-runtime 拆分替代。
  迁移后只保留兼容门面或直接删除。

crates/workflow-events
  由 crates/event-store 替代。
  现有事件模型可迁移到 event-store。
```

当前目录到目标目录映射：

```text
agentflow-cli        -> 改名为 cli，作为 CLI 入口
agentflow-core       -> 改名为 core，并压缩为纯 shared primitives
agent-manual        -> 保留
panel               -> 保留
mcp                 -> 保留，降级为外部 provider adapter
workflow-acceptance -> 改名为 acceptance，只做测试 harness

input               -> 删除，由 spec 替代
execute             -> 删除，由 task-artifacts + workflow-runtime 替代
output              -> 删除，不再有本地 output crate
state               -> 删除，由 projection 替代
workflow-events     -> 删除，由 event-store 替代
loop                -> 删除，由 task-loop + workflow-runtime 替代
```

目标依赖方向：

```text
core
agent-manual
panel
spec
workflow-core
event-store
  ↓
workflow-runtime
  ↓
task-loop
agent-dispatcher
projection
task-artifacts
mcp
release
  ↓
cli / desktop commands

acceptance
  只在测试中依赖上面模块
```

禁止依赖方向：

```text
1. spec 不依赖 task-loop / runtime / projection。
2. event-store 不依赖 spec / runtime / projection。
3. workflow-core 不依赖任何业务 crate。
4. projection 不写 spec issue。
5. Desktop 不写 spec issue 或 task event。
6. mcp 不推进 issue 状态。
7. workflow-runtime 的 public record action 不写 .agentflow/tasks/<issue-id>/delivery/**。
8. task-artifacts 不写 PR/MR body、CHANGELOG 或 release notes。
9. acceptance 不被任何 runtime crate 依赖。
10. core 不反向依赖业务 crate。
11. release 不参与单个 issue 状态推进。
```

---

## 5. Issue 合同如何引用 Workflow

输入顺序必须是：

```text
1. 先写 docs/requirements/<requirement-id>.md
   这是人类需求源，也是外部审计能看到的公开输入记录。

2. Spec Agent 再把 requirement 拆成内部 issue contract
   写入 .agentflow/spec/issues/<issue-id>.json。

3. Build Agent 只消费 .agentflow/spec/issues/<issue-id>.json
   但 PR/MR body 必须引用对应的 docs/requirements/<requirement-id>.md。
```

Issue 不应该复制完整 executionPipeline。

Issue 只保留任务事实和 workflow 引用：

```json
{
  "issueId": "AF-TASK-001",
  "issueCategory": "spec",
  "requiredAgentRole": "build-agent",
  "status": "backlog",
  "workflowRef": "build-agent.issue-loop@v1",
  "sourceSpecId": "agentflow-task-workflow-v1",
  "projectId": "project-task-workflow-v1",
  "priority": "P1",
  "blockedBy": [],
  "allowedPaths": [
    "apps/desktop/src/**",
    "docs/**"
  ],
  "forbiddenPaths": [
    ".agentflow/**"
  ],
  "validationCommands": [
    "npm --prefix apps/desktop run build",
    "git diff --check"
  ],
  "expectedOutputs": {
    "taskRunDir": ".agentflow/tasks/<issue-id>/runs/<run-id>",
    "evidencePath": ".agentflow/tasks/<issue-id>/evidence/evidence.json",
    "publicDeliveryRecord": {
      "prOrMrBody": true,
      "changelogOrReleaseNotes": "required-when-release-visible"
    }
  }
}
```

规则：

```text
1. status 只能是 7 个对外状态之一。
2. workflowRef 必填。
3. workflowRef 决定状态机规则。
4. issue 只描述做什么、边界是什么、验收是什么。
5. issue 不描述每一步如何推进状态。
```

---

## 6. 统一状态模型

Issue 对外只允许 7 个状态：

```text
backlog
todo
in_progress
in_review
done
blocked
cancel
```

含义：

| 状态 | 中文 | 含义 |
| --- | --- | --- |
| backlog | 待处理 | 已生成任务，但还没有进入执行准备 |
| todo | 准备开工 | 依赖、合同、上下文已满足，等待 Build Agent 接管 |
| in_progress | 正在做 | Build Agent 已创建 run，并正在实现 / 验证 |
| in_review | 正在评审 | 已完成本地验证和 PR/MR，等待合并或写回 |
| done | 已完成 | PR/MR 已合并，证据和交付已写回 |
| blocked | 已阻断 | 外部条件导致无法继续 |
| cancel | 已取消 | 用户或系统取消该任务 |

内部 run 可以保留细粒度阶段：

```text
preflight
planning
implementing
validating
creating-pr
waiting-merge
completing
```

但这些只属于 run detail，不再作为 issue 对外状态。

---

## 7. YAML Workflow 定义

建议默认 workflow：

```yaml
apiVersion: agentflow.dev/v1
kind: TaskWorkflow

metadata:
  name: build-agent.issue-loop
  version: v1
  title: Build Agent Issue Loop

spec:
  initialState: backlog
  terminalStates:
    - done
    - cancel

  states:
    backlog:
      label: 待处理
      phase: future
    todo:
      label: 准备开工
      phase: current
    in_progress:
      label: 正在做
      phase: current
    in_review:
      label: 正在评审
      phase: current
    done:
      label: 已完成
      phase: past
    blocked:
      label: 已阻断
      phase: current
    cancel:
      label: 已取消
      phase: past

  transitions:
    - id: schedule
      from: backlog
      to: todo
      on: issue.scheduled
      guards:
        - issue.contract.complete
        - dependencies.done
        - context_pack.ready
        - workspace.clean
      actions:
        - task.context.prepare
        - task.todo.write
        - event.emit.issue_scheduled

    - id: start
      from: todo
      to: in_progress
      on: build_agent.started
      guards:
        - run.created
        - runtime.preflight.passed
        - lease.active
      actions:
        - run.plan.write
        - run.checkpoint.create
        - event.emit.task_started

    - id: request_review
      from: in_progress
      to: in_review
      on: verification.passed
      guards:
        - sandbox.validation.passed
        - pr_or_mr.created
      actions:
        - public_record.pr_body_draft
        - review.prepare
        - event.emit.review_requested

    - id: complete
      from: in_review
      to: done
      on: pr_or_mr.merged
      guards:
        - merge.proof.present
        - public_record.pr_body_ready
      actions:
        - build_agent.complete
        - public_record.pr_body_finalize
        - event.emit.task_completed

    - id: block
      from:
        - backlog
        - todo
        - in_progress
        - in_review
      to: blocked
      on: task.blocked
      actions:
        - blocker.write
        - event.emit.task_blocked

    - id: cancel
      from:
        - backlog
        - todo
        - in_progress
        - in_review
        - blocked
      to: cancel
      on: task.cancelled
      actions:
        - task.cancel.write
        - event.emit.task_cancelled
```

---

## 8. Guard Registry

Workflow YAML 只引用 guard 名称。

具体 guard 由 runtime 注册实现。

建议第一批 guard：

```text
issue.contract.complete
  issue 必填字段完整。

dependencies.done
  blockedBy 里的 issue 全部 done。

context_pack.ready
  Panel Context Pack 存在且可读；不存在时尝试补生成。

workspace.clean
  当前工作区没有未提交的用户源码改动。

run.created
  当前 issue 已有当前 run。

runtime.preflight.passed
  runtime preflight 通过。

lease.active
  当前 run 已获得 active lease。

sandbox.validation.passed
  本地验证命令通过。

pr_or_mr.created
  PR 或 MR 已创建。

merge.proof.present
  PR/MR 已合并的证明存在。

public_record.pr_body_ready
  PR/MR body 已包含本任务交付记录，并引用 docs/requirements/<requirement-id>.md。
```

Guard 失败时不要默认把任务改成 blocked。

规则：

```text
1. 缺依赖完成：保持 backlog。
2. 未到执行顺序：保持 backlog。
3. 缺 context pack：先尝试补生成，失败后保持 todo 或 blocked，取决于失败原因。
4. 工作区不干净：阻断当前 issue。
5. runtime preflight 失败：阻断当前 issue。
6. provider 自动合并不可用：保持 in_review，不进入 blocked。
```

---

## 9. Action Registry

Workflow YAML 只引用 action 名称。

具体 action 由 runtime 注册实现。

建议第一批 action：

```text
task.context.prepare
  调用 Panel 生成或刷新 Context Pack。

task.todo.write
  写入 issue todo 状态事件。

run.plan.write
  写入当前 run 的执行计划。

run.checkpoint.create
  创建执行 checkpoint。

public_record.pr_body_draft
  将验证结果、PR/MR 信息和变更摘要写入 PR/MR body 草稿。

review.prepare
  写入 in_review 阶段需要展示的信息。

build_agent.complete
  调用官方 complete 入口写回 run / evidence / done。

public_record.pr_body_finalize
  确认 PR/MR body 中的任务级公开交付记录已落地。CHANGELOG / Release notes 只在 release 流程中处理。

blocker.write
  写入 blockers 和阻断原因。

task.cancel.write
  写入取消原因和取消事件。

event.emit.*
  追加业务事件。
```

禁止在 YAML 里直接写任意 shell：

```yaml
run: npm test
```

验证命令来自 issue 的 `validationCommands`。

Workflow 只能调用受控 action：

```yaml
actions:
  - sandbox.validation.run
```

---

## 10. Event Log 格式

事件是运行事实源。

建议 JSONL 每行一个事件：

```json
{
  "eventId": "evt-20260614-000001",
  "eventVersion": "task-event.v1",
  "aggregateType": "issue",
  "aggregateId": "AF-TASK-001",
  "projectId": "project-task-workflow-v1",
  "issueId": "AF-TASK-001",
  "type": "issue.scheduled",
  "timestamp": "2026-06-14T10:00:00+08:00",
  "actor": {
    "role": "task-loop",
    "kind": "system"
  },
  "state": {
    "from": "backlog",
    "to": "todo"
  },
  "correlationId": "corr-AF-TASK-001",
  "causationId": "evt-20260614-000000",
  "payload": {
    "workflowRef": "build-agent.issue-loop@v1",
    "transitionId": "schedule",
    "guardsPassed": [
      "issue.contract.complete",
      "dependencies.done",
      "context_pack.ready",
      "workspace.clean"
    ]
  },
  "artifactRefs": [
    ".agentflow/panel/context-packs/AF-TASK-001/context-pack.json"
  ]
}
```

事件类型建议：

```text
issue.created
issue.scheduled
issue.started
issue.checkpoint.created
issue.validation.started
issue.validation.passed
issue.validation.failed
issue.review.requested
issue.pr.created
issue.pr.merged
issue.completed
issue.blocked
issue.cancelled

project.created
project.scheduled
project.issue.ready
project.issue.completed
project.completed

agent.launch.requested
agent.session.created
agent.session.running
agent.session.interrupted
agent.session.resumed
agent.session.completed
```

---

## 11. Projection 格式

Projection 是 UI read model。

可以从事件日志重建。

单任务 projection：

```json
{
  "issueId": "AF-TASK-001",
  "projectId": "project-task-workflow-v1",
  "workflowRef": "build-agent.issue-loop@v1",
  "currentState": "in_progress",
  "displayStatus": "in_progress",
  "currentTransition": "start",
  "latestRunId": "run-001",
  "branchName": "agentflow/project-task-workflow-v1/AF-TASK-001",
  "timeline": [
    {
      "state": "backlog",
      "phase": "past",
      "enteredAt": "2026-06-14T10:00:00+08:00",
      "events": [
        "issue.created"
      ],
      "summary": "任务已生成。"
    },
    {
      "state": "todo",
      "phase": "past",
      "enteredAt": "2026-06-14T10:03:00+08:00",
      "events": [
        "issue.scheduled"
      ],
      "summary": "上下文、依赖和工作区检查已通过。"
    },
    {
      "state": "in_progress",
      "phase": "current",
      "enteredAt": "2026-06-14T10:05:00+08:00",
      "events": [
        "issue.started",
        "issue.checkpoint.created"
      ],
      "summary": "Build Agent 正在执行任务。",
      "liveRefs": [
        ".agentflow/tasks/AF-TASK-001/runs/run-001/worker-state.json"
      ]
    },
    {
      "state": "in_review",
      "phase": "future",
      "summary": "等待本地验证和 PR/MR。"
    },
    {
      "state": "done",
      "phase": "future",
      "summary": "等待合并和交付写回。"
    }
  ],
  "publicDelivery": {
    "evidencePath": null,
    "prUrl": null,
    "mergeCommit": null,
    "changelogPath": null,
    "releaseNotesUrl": null
  },
  "updatedAt": "2026-06-14T10:05:00+08:00"
}
```

任务页只读这个 projection。

---

## 12. Runtime 执行规则

Runtime 接收事件后执行：

```text
1. 读取 issue。
2. 读取 workflowRef 对应 YAML。
3. 读取当前 projection，得到 currentState。
4. 根据 incoming event type 找 transition。
5. 校验 transition.from 是否匹配 currentState。
6. 执行 guards。
7. guards 全部通过后执行 actions。
8. append 状态转移事件。
9. 重建 task projection。
10. 重建 project projection。
11. 发 UI refresh signal。
```

非法转移必须拒绝：

```text
done -> in_progress
cancel -> in_progress
backlog -> in_progress
todo -> done
```

如果需要强制修复状态，只能写显式 repair event：

```text
issue.state.repaired
```

并记录原因。

---

## 13. Build Agent 如何消费 Workflow

Build Agent 不直接读 UI。

Build Agent 启动前拿到：

```text
1. issueId
2. issuePath
3. workflowRef
4. currentState
5. allowedPaths
6. forbiddenPaths
7. validationCommands
8. contextPackPath
9. runId
```

Build Agent 只能在 `todo` 状态接管任务。

接管流程：

```text
1. task-loop 发出 agent.launch.requested。
2. agent-dispatcher 创建 provider session。
3. workflow-runtime 写 issue.started。
4. issue 进入 in_progress。
5. Build Agent 执行实现和验证。
6. 验证通过后发 issue.validation.passed。
7. workflow-runtime 推进到 in_review。
8. PR/MR merged 后发 issue.pr.merged。
9. workflow-runtime 推进到 done。
```

---

## 14. Project Loop 如何调度

Project Loop 不执行代码。

Project Loop 只做：

```text
1. 读取 project issue 列表。
2. 根据 dependencies / blockedBy 排序。
3. 找到第一个可执行 backlog issue。
4. 发 issue.scheduled。
5. 让该 issue 进入 todo。
6. 如果用户点击“启动 Project Loop”，再发 agent.launch.requested。
```

排序规则：

```text
1. 依赖最少的任务优先。
2. 没有未完成 blockedBy 的任务优先。
3. 同层按 priority 排序：P0 > P1 > P2 > P3。
4. 同 priority 按 issue number 排序。
5. 新 project 在任务列表上方显示，但 project 内 issue 按依赖执行顺序显示。
```

---

## 15. 任务页展示模型

任务页左侧：

```text
Project
  Issue 1
  Issue 2
  Issue 3

单项任务
  Issue A
  Issue B
```

状态点：

```text
done       绿点
cancel     红点
backlog    未来状态色
todo       当前状态色
in_progress 当前状态色 + 动态
in_review  当前状态色
blocked    当前状态色 + 阻断提示
```

任务页右侧：

```text
标题
状态流 timeline
  backlog
    事件日志
  todo
    事件日志
  in_progress
    实时信息流
  in_review
    PR/MR / 验证 / 交付草稿
  done
    公开交付记录
```

规则：

```text
1. 右侧内容跟随状态流展示。
2. 已完成状态能展开看历史事件。
3. 当前状态展示实时信息流。
4. 未来状态只展示等待摘要。
5. 不再展示独立的执行页和交付页主导航。
6. 执行和交付都收敛到任务详情里的状态流。
```

---

## 16. 与旧 execute / output 的切换清理关系

新架构不再把 `execute` / `output` 作为一级主流程目录。

新写入路径必须以任务为中心：

```text
spec
  存任务合同。

tasks/<issue-id>/runs/<run-id>
  存当前任务的 run 产物。

tasks/<issue-id>/evidence
  存当前任务的验证证据。

projections
  存 UI read model。

indexes
  存任务列表索引。

events
  存运行事实。

workflow
  存状态机定义。
```

切换清理规则：

```text
1. 新 runtime 不读 .agentflow/execute/**。
2. 新 runtime 不写 .agentflow/execute/**。
3. 新 runtime 不读 .agentflow/output/**。
4. 新 runtime 不写 .agentflow/output/**。
5. 新 runtime 不写 .agentflow/tasks/<issue-id>/delivery/**。
6. UI 不读取 execute / output 目录，只读取 task projection。
7. 实施切换时直接清理旧 execute / output 目录和代码引用。
8. 不提供 legacy import / read-only fallback。
```

公开交付规则：

```text
1. .agentflow 内部只保存到 tasks/<issue-id>/evidence/**。
2. 任务交付记录写入 PR/MR body。
3. 版本可见变更不在 task loop 内直接写 CHANGELOG.md。
4. CHANGELOG.md 或 GitHub/GitLab Release notes 由 release 模块汇总多个 PR/MR 后生成。
5. PR/MR body 必须引用 docs/requirements/<requirement-id>.md。
6. 不新增 public-delivery crate；PR/MR body 由 workflow-runtime action 生成，mcp provider 负责提交。
```

用户看到的是：

```text
任务
  状态流
  事件
  执行动作
  验证结果
  公开交付记录
```

不是：

```text
spec
execute
output
```

`execute` / `output` 不作为旧数据迁移入口；实施切换时直接删除旧目录和旧读取链路。

---

## 17. 迁移计划

### PR 1：架构文档和目录命名

```text
1. 确认 crates 目标目录。
2. 确认旧 input / execute / output / state / loop / workflow-events 的删除边界。
3. 确认 .agentflow 新目录：spec / workflows / events / projections / indexes / tasks。
4. 确认公开输入和公开输出边界。
```

### PR 2：core / cli / acceptance 重命名

```text
1. agentflow-core -> core。
2. agentflow-cli -> cli。
3. workflow-acceptance -> acceptance。
4. 保证 core 不依赖业务 crate。
5. 保证 acceptance 只作为测试 harness。
```

### PR 3：spec

```text
1. 新增 crates/spec。
2. 从 docs/requirements/<requirement-id>.md 生成 spec project / issue contract。
3. 写入 .agentflow/spec/projects 和 .agentflow/spec/issues。
4. 删除 input 作为新任务入口。
```

### PR 4：workflow-core

```text
1. 新增 YAML 解析。
2. 新增 workflow 校验。
3. 新增状态 / transition 类型。
4. 新增 guard / action registry 类型。
```

### PR 5：event-store

```text
1. 新增 task-events.jsonl writer。
2. 新增 reader / replay。
3. 新增 event id / correlation id。
4. 迁移 workflow-events。
```

### PR 6：workflow-runtime

```text
1. 根据事件匹配 transition。
2. 执行 guard。
3. 执行 action。
4. 写状态转移事件。
```

### PR 7：task-artifacts

```text
1. 新增 .agentflow/tasks/<issue-id>/runs/<run-id>。
2. 新增 .agentflow/tasks/<issue-id>/evidence。
3. 迁移 execute 的 run / command / validation / evidence 写入职责。
4. 不生成 delivery 目录。
```

### PR 8：task-loop / agent-dispatcher / mcp

```text
1. task-loop 按 project dependency 调度 issue。
2. task-loop 发 agent.launch.requested。
3. agent-dispatcher 拉起 provider session。
4. mcp 只负责 provider 调用。
5. session 事件写回 task events。
```

### PR 9：projection / Desktop 任务页

```text
1. 从 event log 重建 task projection。
2. 从 event log 重建 project projection。
3. 生成 issue-status index。
4. 左侧任务列表读取 projection。
5. 右侧任务详情展示状态流。
6. 移除执行 / 交付主导航。
```

### PR 10：release

```text
1. 汇总多个已合并 PR/MR 的公开交付记录。
2. 写 CHANGELOG.md。
3. 写 GitHub/GitLab Release notes。
4. 不参与单个 task loop。
```

### PR 11：删除旧模块

```text
1. 删除 input。
2. 删除 execute。
3. 删除 output。
4. 删除 state。
5. 删除 loop。
6. 删除 workflow-events。
```

---

## 18. MVP 验收标准

```text
1. issue 可以通过 workflowRef 绑定 YAML workflow。
2. backlog -> todo -> in_progress -> in_review -> done 由事件驱动。
3. 每次状态变化都有事件日志。
4. 删除 projection 后可以从 event log 重建。
5. 任务页右侧能按状态流展示历史、当前和未来状态。
6. 执行和交付信息不再依赖独立栏目才能查看。
7. Build Agent 只能通过 workflow-runtime 推进状态。
8. Desktop 不直接写 issue 状态。
9. Project Loop 能按依赖关系找下一个可执行任务。
10. Workflow YAML 校验失败时不能启动任务。
```

---

## 19. 暂不做

```text
1. 不做任意 shell workflow。
2. 不做远程 CI 替代。
3. 不做多 Agent 并发抢任务。
4. 不做复杂 workflow UI 编辑器。
5. 不做跨项目 workflow 继承。
6. 不做云端事件总线。
7. 不把 GitHub Issue / GitLab Issue / Linear 当任务事实源。
```

---

## 20. 关键风险

```text
1. Event log 与旧 state index 双写期间可能不一致。
2. 旧 execute/output 代码可能仍直接驱动 UI，需要删除旧读取链路并切到 task projection。
3. Build Agent 如果绕过 runtime，状态仍会跳变。
4. YAML 过度灵活会把系统做成 CI，需要限制 action registry。
5. Projection 重建性能需要后续按 project / issue 做增量优化。
```

---

## 21. 结论

AgentFlow 后续底层不应该继续以 input / execute / output 作为用户可见主流程；任务合同层切到 spec，且不保留 execute / output 作为任务产物目录。

新的核心应该是：

```text
Task Workflow YAML
  定义流程

Task Events
  记录事实

Task Projection
  驱动展示

Build Agent
  执行当前状态允许的动作
```

这套架构能把任务、执行、交付和后续审计都串在同一条状态流里，任务页也能自然成为 AgentFlow 的主工作台。
