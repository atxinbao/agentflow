# AgentFlow Filesystem Workflow Architecture V1

创建日期：2026-06-20
执行者：Codex

## Source Product Baseline

本文从以下产品设计和运行时基线派生：

- [../project/README.md](../project/README.md)
- [../project/goal.md](../project/goal.md)
- [021-ai-os-project-core-capabilities-v1.md](021-ai-os-project-core-capabilities-v1.md)
- [../project/history/2026-06-current-baseline-history/product/001-project-agent-role-model-v1.md](../project/history/2026-06-current-baseline-history/product/001-project-agent-role-model-v1.md)
- [../project/history/2026-06-current-baseline-history/product/002-project-lifecycle-v1.md](../project/history/2026-06-current-baseline-history/product/002-project-lifecycle-v1.md)
- [../project/history/2026-06-current-baseline-history/product/003-goal-plan-document-model-v1.md](../project/history/2026-06-current-baseline-history/product/003-goal-plan-document-model-v1.md)
- [../project/history/2026-06-current-baseline-history/architecture/009-runtime-foundation-closeout-baseline-v1.md](../project/history/2026-06-current-baseline-history/architecture/009-runtime-foundation-closeout-baseline-v1.md)
- [../project/history/2026-06-current-baseline-history/versions/v0.5.0/AGENTFLOW_V0_5_0_SPEC_LOOP_PRODUCTIZATION_TASKS_V1.md](../project/history/2026-06-current-baseline-history/versions/v0.5.0/AGENTFLOW_V0_5_0_SPEC_LOOP_PRODUCTIZATION_TASKS_V1.md)

外部参考：

- [Vercel eve](https://vercel.com/eve)
- [vercel/eve GitHub repository](https://github.com/vercel/eve)
- [Introducing eve](https://vercel.com/blog/introducing-eve)

## 1. Core Conclusion

AgentFlow 的下一阶段架构基准是：

```text
AgentFlow = 项目驱动的 Agent Workflow 文件系统
```

它不是：

- 画布工具；
- 单纯 Agent Runner；
- Eve clone；
- 一个巨大 `workflow.json` / `workflow.yaml` 配置文件；
- 一个把所有行业塞进同一个 UI 的通用软件壳。

AgentFlow 的核心对象是 Project。

```text
Project 是顶层。
Agent 是角色能力。
Flow 是工作路径。
Session 是执行过程。
Evidence 是交付证明。
```

## 2. Why Filesystem-first

AgentFlow 需要 filesystem-first，不是因为文件比数据库更高级，而是因为项目工作天然需要：

- 可读；
- 可审计；
- 可 diff；
- 可版本化；
- 可迁移；
- 可被 Agent 和人类同时理解；
- 可在本地优先运行；
- 可在未来接入云端 Runtime。

一个项目的 Agent Workflow 不应该藏在一个巨大的配置对象里。

更清晰的方式是：

```text
目录表达边界
文件表达合同
事件表达事实
Projection 表达只读视图
```

这和 Eve 的启发一致：目录结构本身应该让人一眼看出 Agent 能做什么、知道什么、从哪里进入、如何被调度。

但 AgentFlow 不能止步于“可读”。它还必须做到：

- 可裁决；
- 可回放；
- 可审计；
- 可证明；
- 可被 Project Runtime 执行。

## 3. Current Runtime Boundary

当前 AgentFlow 的 runtime boundary 已经在 `v0.4.0` 收敛为：

```text
Requirement
-> Spec Project / Spec Issue Contract
-> Runtime Command
-> Action Proposal
-> Arbitration Decision
-> Runtime Event
-> Projection Read Model
-> Desktop / CLI / Provider Query Surface
```

filesystem-first 不能推翻这条主链。

它的职责是让这条主链在文件系统上更清晰：

```text
定义在哪里
合同在哪里
执行过程在哪里
证据在哪里
交付在哪里
审计在哪里
```

## 4. `.agentflow/` Directory Responsibilities

当前 `.agentflow/` 是 AgentFlow 的本地控制平面。

短期不能直接迁移目录结构。任何新结构都必须通过正式 requirement 和 SPEC 进入。

当前版本边界要再压一次：

```text
v0.5.0 只做 Spec Loop 的 filesystem-first 试点。
不在这一版完整落地 codeflow / designflow / Eve adapter。
```

建议把未来目录职责收敛为：

```text
.agentflow/
  project/
    goal.md
    plan.md
    state.json

  agents/
    spec/
      instructions.md
      skills/
    work/
      instructions.md
      skills/
    audit/
      instructions.md
      skills/

  codeflow/
    instructions.md
    workflow.md
    tasks/
    patches/
    evidence/
    handoff/

  designflow/
    prd.md
    directions/
    wireframes/
    hifi/
    design-system/
    pages/
    handoff/

  sessions/
  events/
  reports/
```

这不是当前可直接写入的 runtime contract。

这是下一阶段 filesystem contract 的目标形态，用于指导后续 requirements 和 migration。

## 5. Current Path Mapping

在正式迁移前，必须尊重当前事实源：

| 当前路径 | 当前职责 | 未来映射方向 |
| --- | --- | --- |
| `docs/requirements/**` | public requirement record | `project/` 和 `spec` 入口的公开需求层 |
| `.agentflow/spec/projects/**` | project contract authority | `project/` 的机器可读合同层 |
| `.agentflow/spec/issues/**` | issue contract authority | `codeflow/tasks/` 的执行合同来源 |
| `.agentflow/tasks/<issue-id>/**` | task run and evidence | `sessions/`、`codeflow/evidence/`、`handoff/` 的来源 |
| `.agentflow/events/**` | task status event stream | `events/` authority |
| `.agentflow/projections/**` | read model | `reports/` 和 UI query surface 的只读来源 |
| `.agentflow/audit/**` | independent audit facts | audit flow 的事实来源 |

规则：

1. 当前路径是 authority，不能被目标形态直接替换。
2. 新目录形态必须先进入 SPEC，再迁移。
3. Projection 永远只读。
4. Event Store / event stream 是事实沉淀，不是 UI 缓存。

## 6. CodeFlow Contract

`codeflow` 是技术实现路径。

它处理：

- 技术实现；
- issue 执行；
- patch 生成；
- 本地验证；
- evidence 收集；
- handoff；
- delivery 准备。

建议目录：

```text
codeflow/
  instructions.md
  workflow.md
  tasks/
  patches/
  evidence/
  handoff/
```

### `instructions.md`

定义 CodeFlow 的执行原则：

- 只执行确认后的 spec issue；
- 不从聊天直接执行；
- 不绕过 preflight；
- 不绕过 Action Contract；
- 不绕过 Arbitration；
- 不自动触发 Audit。

### `workflow.md`

定义 CodeFlow 主流程：

```text
Issue Preflight
-> Test Design
-> Implementation
-> Sandbox Verification
-> PR / Delivery Preparation
-> Done Writeback
```

### `tasks/`

承载执行任务的派生视图。

它不能替代 `.agentflow/spec/issues/**` 的 authority。

### `patches/`

保存 patch proposal 或 patch summary。

它不是事实源，只有通过 Runtime / Git / Event 写入后的结果才是事实。

### `evidence/`

保存验证证据：

- command output 摘要；
- test result；
- build result；
- screenshot；
- browser smoke result；
- release gate result。

### `handoff/`

保存给下一阶段 Agent 的交接材料。

handoff 是 derived transport snapshot，不是 task authority。

## 7. DesignFlow Contract

`designflow` 是产品设计路径。

它处理：

- 产品需求整理；
- 信息架构；
- 页面设计；
- 设计系统；
- Figma / visual handoff；
- design-only issue 拆分；
- 设计证据和交付材料。

建议目录：

```text
designflow/
  prd.md
  directions/
  wireframes/
  hifi/
  design-system/
  pages/
  handoff/
```

### `prd.md`

保存设计目标、用户场景、页面范围、非目标、验收口径。

### `directions/`

保存方向探索和设计判断。

### `wireframes/`

保存低保真结构。

### `hifi/`

保存高保真方案说明和版本。

### `design-system/`

保存组件、token、布局、状态、交互约束。

### `pages/`

按页面或工作台拆分设计说明。

### `handoff/`

保存从设计到实现的交接材料。

规则：

1. DesignFlow 可以生成 design-only issue。
2. DesignFlow 不直接授权代码实现。
3. DesignFlow 输出进入 CodeFlow 前，必须经过 SPEC 或确认门。
4. Figma 是设计 surface，不是 AgentFlow 事实源。

## 8. Agent Package Contract

Agent 不应该只是一个聊天入口。

Agent 应该被文件化为可组合 package：

```text
agents/<role>/
  instructions.md
  policy.md
  skills/
  tools/
  subagents/
  channels/
  schedules/
  state/
  evidence/
```

### `instructions.md`

定义 Agent 的身份、任务边界和输出风格。

### `policy.md`

定义 Agent 能读什么、能写什么、能执行什么、必须产出什么、不能做什么。

### `skills/`

定义可按需加载的流程能力。

例如：

- requirement-intake；
- spec-gate；
- issue-preflight；
- sandbox-verification；
- audit-reporting。

### `tools/`

定义 Agent 可调用工具。

AgentFlow 不应只按文件名信任工具。

工具必须绑定：

- Action Contract；
- Role Policy；
- Object State precondition；
- Evidence requirement；
- Arbitration rule。

### `subagents/`

定义可委派的子 Agent。

子 Agent 不是事实 authority。

它只能产出 proposal、evidence 或 report。

### `channels/`

定义入口：

- conversation；
- CLI；
- Desktop；
- GitHub；
- Slack / Teams / Discord；
- API / SDK。

channel 只负责输入输出，不拥有项目 authority。

### `schedules/`

定义定时触发。

定时触发必须进入 Runtime Command，不能直接改事实源。

### `state/`

保存 Agent session state。

它不是 Project state authority。

### `evidence/`

保存 Agent 产出的证据。

证据必须能追溯到对应 action、run、issue、session。

## 9. Skill / Subagent / Channel / Schedule Boundaries

### Skill

Skill 是流程知识。

它回答：

```text
这类任务应该怎么做？
```

Skill 不能自己成为事实源。

### Subagent

Subagent 是委派执行者。

它回答：

```text
谁来处理这个子问题？
```

Subagent 不能绕过主 Agent 的 policy 和 Arbitration。

### Channel

Channel 是入口和出口。

它回答：

```text
请求从哪里来，结果发到哪里？
```

Channel 不决定项目状态。

### Schedule

Schedule 是定时触发器。

它回答：

```text
什么时候自动提出一个 command？
```

Schedule 只能提交 command / proposal，不能直接写状态。

## 10. Durable Session / Evidence / Handoff Rules

AgentFlow 需要 durable session，但 durable session 不能替代项目事实。

建议规则：

1. `sessions/` 保存执行过程。
2. `events/` 保存事实变化。
3. `evidence/` 保存证明材料。
4. `handoff/` 保存交接快照。
5. `reports/` 保存面向人类的只读总结。

关系：

```text
Session 记录过程
Event 记录事实
Evidence 证明结果
Handoff 转交上下文
Report 给人阅读
```

约束：

- session 可以恢复；
- event 必须可回放；
- evidence 必须可追溯；
- handoff 不能成为 authority；
- report 不能回写事实。

## 11. Eve Adapter Boundary

Eve 是 runtime framework。

AgentFlow 是 project workflow product system。

因此当前不迁移到 Eve。

正确顺序是：

```text
先定义 AgentFlow 自己的 filesystem contract
再判断是否需要 Eve adapter / Vercel adapter
```

未来可选 adapter：

```text
AgentFlow Agent Package
-> Eve agent directory
-> Vercel runtime
-> Runtime Command / Projection callback
```

## 12. v0.5.0 Pilot Boundary

`v0.5.0` 只把上面的 filesystem-first 方向收成第一条可执行主链：

```text
Raw Human Request
-> Intake Artifact
-> Classification Artifact
-> Context Artifact
-> Boundary Artifact
-> Route Artifact
-> Preview Artifact
-> Confirmation Artifact
-> Requirement / Spec / Issue Authority
-> Runtime Action Proposal
```

这意味着：

- `v0.5.0` 先把 Spec Loop 文件化；
- `codeflow`、`designflow` 在这一版只作为未来 contract 方向保留；
- 不在 `v0.5.0` 完整落地多 Flow 并行 runtime；
- 不在 `v0.5.0` 进入 Eve / Vercel adapter 主线。

adapter 必须遵守：

- AgentFlow Project authority 不外包；
- Eve session 不替代 AgentFlow Event Store；
- Eve tool 不绕过 Action Contract；
- Eve subagent 不绕过 Role Policy；
- Eve schedule 不直接写 AgentFlow facts；
- Eve channel 不决定 AgentFlow state。

## 12. Migration Strategy

不能一次性迁移 `.agentflow/`。

建议分三步：

### Step 1 - Document Contract

先把 filesystem contract 写成 foundation 文档。

本文件就是 Step 1。

### Step 2 - Spec Loop Pilot

在 `v0.5.0` Spec Loop 中试点：

```text
Requirement Intake
Classifier
Context Resolver
Boundary Checker
Route Decider
Preview Generator
Confirmation Gate
```

目标是让 Spec Loop 的每一步都有清晰文件边界。

### Step 3 - Flow Split Pilot

后续版本再把工作路径拆成：

```text
codeflow
designflow
```

先做软件开发和产品设计两个现场。

不要一开始就做所有行业。

## 13. Acceptance Direction

本 foundation 成立时，应满足：

- 能解释 AgentFlow 为什么 filesystem-first；
- 能说明 `.agentflow/` 各类目录职责；
- 能区分 Project、Agent、Flow、Session、Evidence；
- 能定义 CodeFlow 和 DesignFlow 的第一层合同；
- 能定义 Agent Package 的文件组成；
- 能说明 skill、tool、subagent、channel、schedule 的边界；
- 能说明 durable session、evidence、handoff 的存放规则；
- 能明确 Eve adapter 是未来可选项，不进入当前主线；
- 不破坏当前 `.agentflow/spec/**`、`.agentflow/tasks/**`、`.agentflow/events/**` 的 authority。

## 14. Non-goals

本文不做：

- 不修改 `.agentflow/` 当前目录；
- 不生成正式 SPEC issue；
- 不授权 Build Agent 执行；
- 不迁移到 Eve；
- 不引入 Vercel runtime 依赖；
- 不定义云端部署；
- 不定义所有行业 Pack；
- 不替代 `v0.5.0` Spec Loop 任务规划。

## 15. Next Step

下一步应该把本文件作为 `v0.5.0` 和后续 Pack System 的架构参考。

进入开发时，仍然必须按 AgentFlow 正式流程走：

```text
SPEC Draft Preview
-> Project Preview
-> Issues Preview
-> Human Confirmation
-> docs/requirements/**
-> .agentflow/spec/projects/**
-> .agentflow/spec/issues/**
```

确认前，本文件只是 foundation 架构基准，不是可执行任务合同。
