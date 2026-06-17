# 004 - Event And Projection Model V1

创建日期：2026-06-18
执行者：Codex

## Purpose

本文定义 AgentFlow 的事件模型、状态派生模型和 Projection 读模型。

目标是让后续任何页面、CLI、外部 adapter 都建立在统一事实流之上，而不是各读各的散文件。

## 核心原则

1. Event 是事实记录，不是页面描述。
2. State 是 Workflow 对 Event 的解释结果。
3. Projection 是给产品表面消费的只读视图。
4. 页面不直接读 provider session 或临时 artifacts 细节。

## Event Envelope

统一事件外壳建议包含：

```json
{
  "eventId": "evt-000001",
  "eventType": "work.run.started",
  "flowType": "work",
  "flowId": "issue-AF-123",
  "projectId": "project-001",
  "issueId": "AF-123",
  "runId": "run-001",
  "role": "work-agent",
  "actor": "system|agent|human|provider",
  "timestamp": "2026-06-18T10:00:00Z",
  "correlationId": "corr-001",
  "causationId": "evt-000000",
  "payload": {}
}
```

## Event 分类

建议固定六大类：

### 1. Contract Events

- spec.project.confirmed
- spec.issue.created
- spec.issue.updated

### 2. Workflow Events

- project.state.entered
- work.state.entered
- audit.state.entered
- delivery.state.entered

### 3. Runtime Events

- context.pack.requested
- context.pack.ready
- run.created
- run.started
- checkpoint.created
- verification.completed

### 4. Session Events

- agent.launch.requested
- agent.launch.claimed
- agent.session.created
- agent.session.running
- agent.session.failed
- agent.session.cancelled

### 5. Review / Audit / Delivery Events

- review.prepared
- audit.started
- audit.passed
- audit.needs_repair
- delivery.started
- delivery.published

### 6. Completion Events

- merge.confirmed
- issue.completed
- project.goal_recheck.requested
- project.accepted

## State 派生规则

状态不是事件本身，而是 workflow runtime 对事件序列的派生。

例如：

- `run.started` 不等于 `in_progress`
- `pr.created` 不等于 `in_review`
- `merge.confirmed` 不等于 `done`

只有当 workflow guard / action 满足后，runtime 才能追加：

- `work.state.entered: in_progress`
- `work.state.entered: in_review`
- `work.state.entered: done`

## Projection 视图

Projection 至少要重建三类视图：

### 1. Task Projection

用于任务页：

- 当前状态
- 状态时间线
- 当前阶段详情
- 历史事件流
- evidence summary
- audit summary
- delivery summary

### 2. Project Projection

用于项目页：

- 当前 project stage
- issues grouped by project
- 当前活跃 issue
- next actions
- blockers
- completion hints

### 3. Session / Runtime Projection

用于执行与调试：

- provider session 状态
- run summary
- latest command / log snippet
- retry / failure hints

## UI 展示语义

页面展示应该固定成三种语义：

### 当前状态

- 显示实时事件流
- 显示当前阶段详情
- 可以看到最新日志和产物引用

### 已完成状态

- 显示历史日志
- 显示关键结果与产物
- 不显示伪实时占位

### 未来状态

- 只显示等待
- 不显示假日志
- 不生成未发生的阶段详情

## Task Timeline 读模型

任务页右侧时间线建议按状态节点展示：

- backlog / todo / in_progress / in_review / done / blocked / cancel

每个状态节点下挂事件流：

- 已完成状态：展示历史事件
- 当前状态：展示实时事件
- 未来状态：展示空等待

## Project 列表读模型

左侧 Project / Issue 树建议来自 Projection，而不是直接读 spec 文件。

需要包含：

- project title
- project status
- issue ordering
- issue status dot
- priority badge
- active selection
- current / history / future grouping

## Rebuild / Replay

Projection 必须支持：

- 从全量 events 重建
- 从 checkpoint + events 重建
- 按 project / issue 局部重建

规则：

- replay 不改 contract；
- replay 不调用 provider；
- replay 只生成 read model。

## 本地与公开交付边界

本地投影只保留运行事实与索引：

- `.agentflow/events/**`
- `.agentflow/tasks/**`
- `.agentflow/audit/**`
- `.agentflow/projections/**`

公开交付不通过 projection 写回本地 delivery 目录，而是进入：

- PR/MR body
- `CHANGELOG.md`
- release notes
- public delivery summary

## 不做事项

- 不把 projection 当 authority。
- 不让 UI 自己维护一套独立状态。
- 不让 provider session snapshot 直接替代 event/projection。
- 不让“页面刷新”成为流程推进机制。
