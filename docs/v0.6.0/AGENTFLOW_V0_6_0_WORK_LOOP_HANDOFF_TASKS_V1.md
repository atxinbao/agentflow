# AgentFlow v0.6.0 Work Loop Handoff Tasks V1

日期：2026-06-20
执行者：Codex
状态：Version Planning Draft / 开发前置文档 / 不授权 Build Agent 执行

## 1. Purpose

本文档沉淀 AgentFlow `v0.6.0` 的开发任务规划。

`v0.6.0` 的核心目标不是重新做 Spec Loop，也不是直接做完整多 Agent 平台。

它要完成这一条链：

```text
Confirmed Spec Issue
-> Work Command
-> Work Action Proposal
-> Arbitration
-> Issue Preflight
-> Lock / Queue
-> Work Session
-> Evidence Gate
-> State Transition
-> Event Store
-> Projection
-> Done Writeback
```

一句话：

```text
把 Build Loop 从“Agent 自己干活”升级成“Runtime 管住 Agent 干活”。
```

## 2. Version Boundary

`v0.6.0` 处理 Build / Work Loop。

但它只处理受控执行的第一层：

- 合法任务入口；
- 合法 Agent 行为；
- 合法执行状态；
- 合法证据；
- 合法 Done 写回；
- 基础多 Agent proposal 仲裁。

它不处理完整云端调度、Message Bus 中心化、行业 Pack、完整 OS Console 和自动审计。

## 3. Precondition

进入 `v0.6.0` 前，必须先完成 `v0.5.1` 修复：

1. release metadata / tag gate / release gate 修复；
2. Spec materialization 必须先 Arbitration，再写 authority；
3. Runtime command / proposal / decision / action records 持久化；
4. Spec authority manifest 明确；
5. `v0.5.0` 文档状态从 clean stable 改成 functional baseline 或 release closeout。

如果这些前置条件未完成，`v0.6.0` 不能进入正式开发。

## 4. Issue Preview

| Issue | Title | Dependency | Priority | First executable |
| --- | --- | --- | --- | --- |
| `V060-001` | Work Loop Filesystem Contract / CodeFlow Contract | 无 | P0 | 是 |
| `V060-002` | Spec Issue -> Work Command Handoff | `V060-001` | P0 | 否 |
| `V060-003` | Work Agent Action Proposal Contract | `V060-002` | P0 | 否 |
| `V060-004` | Issue Preflight Runtime Gate | `V060-003` | P0 | 否 |
| `V060-005` | Issue / Object Lock and Lease | `V060-004` | P0 | 否 |
| `V060-006` | Dependency Queue and Next Issue Selection | `V060-005` | P0 | 否 |
| `V060-007` | Evidence Gate and Verification Contract | `V060-004` | P0 | 否 |
| `V060-008` | Work State Transition Enforcement | `V060-005`, `V060-007` | P0 | 否 |
| `V060-009` | Durable Work Session and Recovery | `V060-008` | P1 | 否 |
| `V060-010` | Work Loop Event Model and Projection | `V060-009` | P1 | 否 |
| `V060-011` | Controlled Multi-agent Proposal Arbitration | `V060-006`, `V060-010` | P1 | 否 |
| `V060-012` | Done Writeback / Delivery / Audit Separation Acceptance | `V060-010`, `V060-011` | P1 | 否 |

## 5. Development Tasks

### V060-001 - Work Loop Filesystem Contract / CodeFlow Contract

目标：

定义 Work Loop 的文件化阶段合同，让执行过程不再只是聊天线程或临时脚本。

范围：

- 定义 Work Loop / CodeFlow 的阶段边界；
- 定义 work command、proposal、preflight、session、evidence、handoff、delivery 的文件合同；
- 明确哪些文件是 authority，哪些只是 derived artifact；
- 定义 Work Loop 与现有 `.agentflow/spec/**`、`.agentflow/tasks/**`、`.agentflow/events/**`、`.agentflow/projections/**` 的映射；
- 明确 Build Agent / Work Agent 命名关系。

验收标准：

- Work Loop 每个阶段都有稳定输入、输出、状态和证据位置；
- CodeFlow contract 不替代 `.agentflow/spec/issues/**` authority；
- Handoff 只是 transport snapshot，不成为任务权威；
- Evidence 能追溯到 issue、run、command、proposal；
- 不新增 legacy `.agentflow/input/**`、`.agentflow/execute/**`、`.agentflow/output/**`。

非目标：

- 不执行任务；
- 不启动 Build Agent；
- 不做 UI；
- 不做完整多 Agent 并发。

### V060-002 - Spec Issue -> Work Command Handoff

目标：

把确认后的 spec issue 转成 Runtime 可以理解的 Work Command。

范围：

- 读取 `.agentflow/spec/issues/**` 中的合法 spec issue；
- 校验 `issueCategory=spec`、`requiredAgentRole=build-agent` 或兼容 Work Agent role；
- 生成 Work Command；
- 保留 source requirement、source spec、workflowRef、allowedPaths、forbiddenPaths、validationCommands、expectedOutputs；
- 生成 handoff artifact，但不让 handoff 替代 spec issue authority。

验收标准：

- 只有合法 spec issue 能生成 Work Command；
- 外部 GitHub issue、聊天文本、临时计划不能直接生成 Work Command；
- Work Command 可追溯回 source spec issue；
- 缺少 required fields 时必须拒绝 handoff；
- 不启动执行。

非目标：

- 不生成新 spec issue；
- 不写源码；
- 不创建 PR。

### V060-003 - Work Agent Action Proposal Contract

目标：

让 Work / Build Agent 的每个关键动作都先成为 Action Proposal。

范围：

- 定义 Work Action Proposal schema；
- 覆盖 start work、claim issue、create run、write patch、run validation、prepare delivery、mark done 等动作；
- 绑定 Ontology object、Action Contract、Role Policy、Object State precondition；
- 定义 proposal 所需 evidence；
- 输出可被 Arbitration 消费的 proposal。

验收标准：

- Work Agent 不直接改状态；
- 每个关键写动作都有 proposal；
- proposal 里能看出作用对象、动作、角色、前置状态和期望证据；
- proposal 不通过 Arbitration 时不能写 Event Store；
- proposal 失败时能给出明确拒绝原因。

非目标：

- 不实现完整调度器；
- 不引入 Message Bus；
- 不做 UI 展示。

### V060-004 - Issue Preflight Runtime Gate

目标：

在执行前确认 issue 是否可以进入 Work Loop。

范围：

- 校验 issue 状态；
- 校验依赖是否完成；
- 校验 contract 是否完整；
- 校验 allowedPaths / forbiddenPaths；
- 校验 validationCommands；
- 校验 expectedOutputs；
- 校验当前 workspace 是否满足执行前置条件；
- 生成 preflight decision。

验收标准：

- preflight 未通过时不能创建 work session；
- 依赖未完成时不能执行；
- contract 缺字段时不能执行；
- forbidden path 冲突时不能执行；
- preflight decision 可追溯、可投影。

非目标：

- 不修改源码；
- 不运行测试；
- 不创建 PR。

### V060-005 - Issue / Object Lock and Lease

目标：

阻止多个 Agent 同时修改同一 issue 或同一关键对象。

范围：

- 定义 issue lock；
- 定义 ontology object lock；
- 定义 lease owner、lease ttl、renew、release、expire；
- 定义 stale lock 处理规则；
- 将 lock decision 接入 Arbitration。

验收标准：

- 同一 issue 同一时间只能有一个有效执行 lease；
- 冲突对象不能被并发写入；
- lease 过期后可以按规则恢复；
- lock / lease 事件可回放；
- 人类可以看懂当前谁持有锁以及为什么。

非目标：

- 不做跨机器分布式锁；
- 不做云端调度；
- 不引入中心 Message Bus。

### V060-006 - Dependency Queue and Next Issue Selection

目标：

让项目可以按依赖顺序找到下一条可执行 issue。

范围：

- 读取 spec project / spec issues；
- 根据 `blockedBy` 建立依赖队列；
- 排除 blocked、done、cancel、in_progress 的 issue；
- 识别 ready issue；
- 生成 next issue candidate；
- 输出不能执行的原因列表。

验收标准：

- 依赖未完成的 issue 不会被选中；
- done / cancel issue 不会被重复执行；
- 多个 ready issue 时有稳定排序；
- 没有 ready issue 时能解释原因；
- queue 只给建议，不绕过 preflight 和 Arbitration。

非目标：

- 不自动静默执行下一条 issue；
- 不做完整项目管理 UI；
- 不做跨项目调度。

### V060-007 - Evidence Gate and Verification Contract

目标：

阻止没有验证证据的任务被标记为 Done。

范围：

- 定义 evidence 类型；
- 定义 validation result artifact；
- 定义 command output / test result / screenshot / PR link / merge proof 等证据规则；
- 绑定 issue expectedOutputs；
- 在 Done 前检查 required evidence；
- 记录 evidence gate decision。

验收标准：

- 没有 required evidence 不能 Done；
- evidence 必须能追溯到 issue 和 run；
- failed validation 不能被当作通过证据；
- 手动无法验证时必须记录原因和风险；
- Evidence Gate 不自动触发 Audit。

非目标：

- 不替代 Audit Agent；
- 不做人工外包验证；
- 不新增远程 CI 依赖作为唯一 gate。

### V060-008 - Work State Transition Enforcement

目标：

让 Work Loop 状态迁移受合同控制。

范围：

- 定义 Work State；
- 定义 issue state 和 run state 的映射；
- 定义合法迁移；
- 定义非法迁移拒绝；
- 接入 Object State Machine；
- 接入 Arbitration decision。

验收标准：

- issue 不能跳过 preflight 直接进入 done；
- run 不能没有 session 就进入 completed；
- failed / interrupted / resumed 有明确迁移规则；
- done 必须经过 Evidence Gate；
- cancel / supersede / retry / resume 都有明确状态效果。

非目标：

- 不重写全部旧状态历史；
- 不做 UI 动画；
- 不做 release 状态机。

### V060-009 - Durable Work Session and Recovery

目标：

让 Work Loop 执行过程可以中断、恢复、重试。

范围：

- 定义 work session；
- 记录 session owner、issue、run、branch、workspace、startedAt、lastHeartbeat；
- 记录 interrupted / resumed / failed / completed；
- 支持 retry policy；
- 支持 recovery summary；
- 生成 session evidence。

验收标准：

- session 可以从事件和文件恢复当前状态；
- 中断后不会丢失 issue / run / evidence 关联；
- retry 不会覆盖旧证据；
- completed session 必须关联 Done writeback 或明确未完成原因；
- session 不是 project authority，不能替代 Event Store。

非目标：

- 不做跨设备实时同步；
- 不做复杂 worker pool；
- 不做云端 durable execution。

### V060-010 - Work Loop Event Model and Projection

目标：

让 Work Loop 的执行过程可以被 Event Store 记录，并被 Projection 只读展示。

范围：

- 定义 Work Loop event types；
- 覆盖 command created、proposal accepted、preflight passed、lock acquired、session started、validation recorded、state transitioned、done written 等事件；
- 定义 projection read model；
- 支持按 project、issue、run、session 查询；
- 支持 evidence summary。

验收标准：

- Work Loop 状态可以从事件重建；
- Projection 只读；
- UI / CLI 查询不能直接修改事实；
- event 和 projection 能解释当前 issue 为什么处于某状态；
- Done writeback 能被 projection 看见。

非目标：

- 不做完整 OS Console；
- 不做行业 Surface；
- 不做复杂图数据库。

### V060-011 - Controlled Multi-agent Proposal Arbitration

目标：

支持基础多 Agent 协作，但所有 Agent 都只能提交 proposal，由 Runtime 仲裁。

范围：

- 支持多个 Agent 提交 Work Action Proposal；
- 处理同 issue 冲突；
- 处理同 object 冲突；
- 处理依赖冲突；
- 处理 role policy 冲突；
- 输出 accept / reject / queue / requireHumanDecision / cancel / supersede；
- 保证被拒绝 proposal 不写 authority。

验收标准：

- 多 Agent 不能绕过 Runtime；
- 冲突 proposal 不会同时写入；
- queued proposal 有明确原因；
- requireHumanDecision 有明确问题和候选处理方式；
- Arbitration decision 可追溯到 proposal。

非目标：

- 不做大规模并发；
- 不做 worker autoscaling；
- 不引入 Message Bus 作为中心主链。

### V060-012 - Done Writeback / Delivery / Audit Separation Acceptance

目标：

证明 Work Loop 能完整收尾，并保持 Delivery 与 Audit 分离。

范围：

- 定义 Done writeback 合同；
- 绑定 merged PR / release note / changelog / evidence；
- 写入 delivery summary；
- 更新 issue / run 状态；
- 生成 acceptance test；
- 验证 Done 不自动创建 audit issue；
- 验证 explicit audit request 才进入 Audit Agent。

验收标准：

- Done writeback 只能在 evidence gate 通过后发生；
- delivery 有清晰证据链；
- audit 不被 Build / Work Loop 自动触发；
- explicit audit request 能被识别为独立流程；
- acceptance 覆盖从 spec issue 到 done 的完整链。

非目标：

- 不写 audit report；
- 不创建自动审计；
- 不做 release 系统大改。

## 6. Suggested Milestones

### Milestone 1 - Work Loop Contract

包含：

- `V060-001`
- `V060-002`
- `V060-003`

完成后，AgentFlow 能把确认后的 spec issue 转成受 Runtime 管理的 Work Command 和 Work Action Proposal。

### Milestone 2 - Execution Gate

包含：

- `V060-004`
- `V060-005`
- `V060-006`
- `V060-007`
- `V060-008`

完成后，AgentFlow 能判断任务能不能执行、谁能执行、证据是否足够、状态能不能迁移。

### Milestone 3 - Session and Projection

包含：

- `V060-009`
- `V060-010`

完成后，AgentFlow 能恢复执行过程，并用只读 projection 展示 Work Loop 状态。

### Milestone 4 - Controlled Multi-agent and Closeout

包含：

- `V060-011`
- `V060-012`

完成后，AgentFlow 能支持基础多 Agent proposal 仲裁，并完成 Done / Delivery / Audit 分离验收。

## 7. Completion Criteria

`v0.6.0` 完成时，必须满足：

- spec issue 不能被聊天、GitHub issue 或临时计划绕过；
- Work Agent 每个关键写动作必须先成为 Action Proposal；
- Arbitration 必须发生在状态写入前；
- preflight 未通过不能启动 work session；
- lock / lease 能阻止冲突执行；
- dependency queue 能稳定找出下一条可执行 issue；
- evidence gate 能阻止无证据 Done；
- state transition 不能非法跳转；
- work session 可以中断、恢复、重试；
- event store 可以回放 Work Loop 状态；
- projection 只读；
- 多 Agent 只能提交 proposal，不能直接写事实；
- Done writeback 不自动触发 Audit。

## 8. Verification Direction

正式 SPEC 生成时，应至少覆盖以下验证方向：

- 单 issue 从 spec issue 到 done 的 happy path；
- 依赖未完成时 preflight 拒绝；
- 缺 evidence 时 Done 拒绝；
- 两个 Agent 抢同一 issue 时 lock 拒绝或 queue；
- 被 reject 的 proposal 不写 Event Store；
- interrupted session 可以恢复；
- done writeback 后 audit 不自动生成。

## 9. Next Step

下一步不是直接执行这些任务。

正确顺序是：

```text
SPEC Draft Preview
-> Project Preview
-> Issues Preview
-> Human Confirmation
-> docs/requirements/**
-> .agentflow/spec/projects/**
-> .agentflow/spec/issues/**
```

确认前，本文件只是 `v0.6.0` 的开发前置规划。
