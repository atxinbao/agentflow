# V101 Software Dev Pack Usage Baseline V1

日期：2026-06-26
执行者：Codex

## Goal

把 Software Dev Pack 从 stable contract 推进到可读的使用基线。

这份文档不新增 Pack 能力，只说明一个真实软件开发任务如何穿过 v1 stable core：

```text
Requirement
-> Spec
-> Issue
-> Run
-> Evidence
-> Acceptance
-> Delivery
-> Done
```

Audit 仍是独立 sidecar：

```text
Done / Delivery
-> Optional Audit Request
-> Audit Report
-> Finding
-> Follow-up Proposal
```

## Authority Boundary

Software Dev Pack 不能把外部系统变成 AgentFlow authority。
不能把 GitHub issue 当成 AgentFlow authority。

明确禁止：

- GitHub issue 作为任务 authority；
- GitHub PR 作为 completion authority；
- provider session 作为 task truth；
- Audit finding 直接阻断主链；
- connector 直接写 `.agentflow/spec/**`、`.agentflow/events/**` 或 `.agentflow/tasks/**`。

正确关系：

| 对象 | 角色 |
| --- | --- |
| Requirement / Spec / Issue | AgentFlow 项目事实源 |
| Runtime API | command admission |
| Event Store | durable authority |
| Projection | read model |
| Evidence Pack | 验证证据 |
| Acceptance Gate | 完成判断 |
| Delivery | 公开交付记录 |
| Audit | 独立 sidecar |

## Example Flow

### 1. Requirement

人类输入需求，Spec Loop 生成 requirement artifact，并在确认后写入正式 Requirement Record。

### 2. Spec

Spec Agent 将 requirement 拆成可执行 Spec 和 Issue。Spec 必须说明范围、非目标、验收标准、路径边界和预期输出。

### 3. Issue

Issue 是 Work Agent 的唯一任务合同。GitHub issue 只能是 planning mirror，不能替代 AgentFlow Issue。

### 4. Run

Work Agent 通过 Runtime API 创建 run。run 必须绑定 issue、branch、executor session 和 acceptance target。

### 5. Evidence

Work Agent 本地执行验证，写入 Evidence Pack。证据必须包含命令、退出码、关键输出、变更文件和必要的 UI / browser proof。

### 6. Acceptance

Acceptance Gate 读取 Evidence Pack 和 Issue contract。通过后才允许进入 delivery / closeout。

### 7. Delivery

Delivery 记录公开交付摘要：PR/MR body、验证结果、变更摘要、风险、回滚方式和 release note 入口。

### 8. Done

Completion Commit 只在 Acceptance Gate 和 Delivery 都满足后写入 Done。

## Runtime API Mapping

Software Dev Pack 使用以下稳定面：

- Runtime command API：创建 run、准备 review、写 closeout proof、complete；
- Projection query API：读取 Task Workbench、Delivery Package、Audit Surface；
- Evidence / Acceptance contract：判断任务是否可完成；
- Executor Adapter contract：隔离 provider session 和 task truth。

## Audit Sidecar

Audit 只读取 delivery 和 evidence，输出 audit report / finding / follow-up proposal。

Audit 不自动：

- 改 issue 状态；
- 改 completion；
- 创建主链 blocker；
- 启动 provider；
- 修改 release authority。

## Release Gate Binding

release gate 必须生成：

```text
runtime/software-dev-pack-usage-baseline.json
```

并证明：

- usage flow 完整；
- GitHub issue 不是 authority；
- Audit sidecar 独立；
- 映射到 Runtime API、Evidence、Acceptance 和 Projection；
- 不新增行业 Pack。
