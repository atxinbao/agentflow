# AgentFlow Human-Agent Guided Experience V1

创建日期：2026-06-05
执行者：ChatGPT
状态：产品设计草案
版本：v1

---

## 0. 阅读情况和边界

已阅读并作为本设计依据的文件：

```text
README.md
ROADMAP.md
GOAL.md
verification.md
docs/requirements/008-3-agentflow-workflow-directory-blueprint-v1-final.md
docs/requirements/008-4-project-panel-v1.md
docs/requirements/013-workflow-state-gate-orchestration-v1.md
docs/requirements/014-agentflow-end-to-end-workflow-acceptance-v1.md
docs/requirements/014-1-014-2-agent-locale-and-voice-style-policy-v1.md
apps/desktop/src/App.tsx
apps/desktop/src/features/status-channel/statusAdapters.ts
apps/desktop/src/features/agent-manual/hooks/useAgentManual.ts
apps/desktop/src/features/project-files/hooks/useProjectPanel.ts
apps/desktop/src/features/output/OutputAuditPanel.tsx
apps/desktop/src/features/state/hooks/useStateStatus.ts
apps/desktop/src-tauri/src/commands/input.rs
apps/desktop/src-tauri/src/commands/execute.rs
apps/desktop/src-tauri/src/commands/output.rs
apps/desktop/src-tauri/src/commands/state.rs
apps/desktop/src/types/status.ts
```

未看到这个文件：

```text
docs/requirements/008-3-agentflow-workflow-directory-blueprint-v1.md
```

仓库里实际存在的是：

```text
docs/requirements/008-3-agentflow-workflow-directory-blueprint-v1-final.md
```

以下设计基于已看到的文件。

本文件是前端体验设计，不直接授权新增模型调用、自动执行 Codex、远程 PR、merge、deploy 或后端新能力。后续实现需要再拆成明确开发切片。

---

## 1. 设计结论

AgentFlow V1 的人类入口不要做成“复杂工作流控制台”。

它应该做成一条简单动线：

```text
打开项目
  ↓
看项目是否准备好
  ↓
告诉 Agent 想做什么
  ↓
确认 SPEC
  ↓
复制 Codex 指令
  ↓
回填交付材料
  ↓
请求人工审计
  ↓
接受 / 返工 / 补证据
```

用户不需要先理解：

```text
define/
panel/
input/
execute/
output/
state/
gates/
blockers/
leases/
sessions/
```

用户只需要知道：

```text
项目准备好了吗？
Agent 看懂项目了吗？
现在下一步做什么？
为什么还不能交给 Codex？
怎么把任务交给 Codex？
Codex 做完后怎么检查？
```

V1 前端的核心不是“把所有内部能力展示出来”，而是：

```text
把内部 Agent workflow 翻译成人类能走的下一步。
```

---

## 2. 产品定位

### 2.1 AgentFlow 是什么

AgentFlow 不是 AI IDE，也不是代码编辑器，也不是 Linear 替代品。

AgentFlow 是：

```text
本地 Agent 工作流控制面。
```

大白话：

```text
Codex 负责干活。
AgentFlow 负责让 Codex 知道该干什么、不能干什么、做到什么算完成。
```

### 2.2 早期版本靠什么生存

早期版本不要和 Codex / Claude Code / Cursor 抢位置。

早期版本的生存方式是：

```text
寄生在大模型厂商工具里生存。
```

也就是：

```text
不替代 Codex。
不替代 Claude Code。
不替代 Cursor。
只帮它们更清楚、更安全、更可控地干活。
```

### 2.3 当前已有底层能力

| 内部模块 | 大白话含义 | 普通用户默认不看 |
| --- | --- | --- |
| `define/` | Agent 工作手册、规则、skills、AGENTS.md | skills lock、layout、manual hash |
| `panel/` | 项目工作现场 | manifest、symbols、relations、search index |
| `input/` | 需求源头、SPEC Gate、Projects、Issues | input manifest、index、snapshot |
| `execute/` | 受控执行流水线 | run、preflight、lease、checkpoint、patch |
| `output/` | 证据、release delivery、human audit | output index、audit package paths |
| `state/` | 全局派生状态、gate、next actions、blockers | locks、sessions、indexes、timeline |

### 2.4 人类真正要完成什么

人类用户来到 AgentFlow，不是为了学习目录结构。

他们要完成 5 件事：

```text
1. 打开一个本地项目。
2. 说清楚一个需求。
3. 确认 Agent 整理出来的 SPEC。
4. 把确认后的任务交给 Codex。
5. 检查 Codex 交付结果是否靠谱。
```

### 2.5 不能直接暴露给普通用户的概念

默认不要把这些放到主界面：

```text
manifest.json
skills-lock.json
workflow.json
locale.json
style.json
blockers.json
locks
sessions
indexes
state/health/*
state/gates/*
output/audit/<audit-id>/**
execute/runs/<run-id>/**
```

它们可以放到“高级详情”里。

主界面只展示：

```text
Ready
Warning
Blocked
Next action
```

并且必须翻译成人话。

---

## 3. 用户主流程

### 3.1 Diagnose：当前体验问题

| 问题 | 用户会怎么卡住 | 设计修复方向 |
| --- | --- | --- |
| 首页像内部模块集合 | 第一次打开不知道下一步做什么 | 首页只回答“项目是否准备好”和“下一步做什么” |
| 状态词偏系统内部 | 看见 `workflow gate` / `input ready` 不知道含义 | 状态翻译成人话，例如“需求还没有确认成 SPEC” |
| define / panel / input / execute / output / state 太底层 | 以为必须学习目录结构才能用 | 主界面只展示工作手册、项目现场、需求、交付、审计 |
| 空白聊天框门槛高 | 不知道该怎么描述需求 | 第一屏给 5 个意图入口和示例 |
| SPEC / Issue 概念不清楚 | 不知道为什么不能直接交给 Codex | 在 Spec Agent 里解释“确认后才会成为 Codex 的执行依据” |
| Build Agent 容易被误解成自动执行 | 用户期待 AgentFlow 直接改代码 | Build Agent 只展示 Codex Handoff，不执行代码 |
| Audit 入口太像底层表单 | 不知道为什么要选择 delivery 和 reason | 包装成 Audit Agent 流程：选择交付、说明原因、请求人工审计 |
| 高级 JSON 太吵 | 普通用户被 manifest / locks / sessions 干扰 | 默认折叠到 Advanced Details |
| 阻断原因偏机器话 | 用户不知道该修什么 | 只显示第一个最重要阻断，并给一个主动作 |
| 人类确认门不够显眼 | 用户不知道哪些步骤需要自己拍板 | SPEC approved、Codex handoff、audit request、返工都要明确确认 |

### 3.2 主流程

```text
Project Onboarding
  ↓
Project Home
  ↓
Next Step Card
  ↓
Spec Agent Flow
  ↓
SPEC Review
  ↓
Codex Handoff
  ↓
Output / Delivery Summary
  ↓
Human Audit
  ↓
Audit Result
```

### 3.3 三个 Agent 的用户心智

V1 对用户只展示 3 个 Agent：

| Agent | 用户理解 | 用户动作 |
| --- | --- | --- |
| Spec Agent | 帮我把想法整理成可确认的需求 | 描述需求、回答问题、确认 SPEC |
| Build Agent | 把确认后的需求打包成 Codex 能执行的任务 | 复制 Codex 指令、交给 Codex、回填交付 |
| Audit Agent | 检查 Codex 做完的结果是否符合原需求 | 选择 delivery、填写原因、请求审计、看报告 |

### 3.4 每个阶段只有一个主动作

| 阶段 | 主按钮 |
| --- | --- |
| 项目未打开 | 打开本地项目 |
| 项目已准备 | 告诉 Agent 你想做什么 |
| 需求理解中 | 确认理解 |
| SPEC 草稿可生成 | 生成 SPEC Draft |
| SPEC 草稿已生成 | 确认 SPEC |
| 任务可交给 Codex | 复制 Codex 指令 |
| 有交付材料 | 请求人工审计 |
| 审计完成 | 接受结果 |

次要动作只放到弱按钮或链接：

```text
查看高级详情
返回修改 SPEC
查看上下文包
复制调试信息
```

---

## 4. 信息架构

### 4.1 顶层结构

```text
AgentFlow Desktop
├── Project Onboarding
│   ├── Open Local Project
│   └── Workspace Prepare Result
│
├── Project Home
│   ├── Project Status Bar
│   ├── Next Step Card
│   ├── Agent Cards
│   └── Advanced Details
│
├── Spec Agent
│   ├── Intent Entry
│   ├── Requirement Input
│   ├── Understanding Review
│   ├── Clarifying Questions
│   ├── SPEC Draft Preview
│   └── SPEC Review
│
├── Build Agent
│   ├── Approved SPEC Summary
│   ├── Issue Summary
│   ├── Panel Context Pack
│   └── Codex Handoff Package
│
├── Output
│   ├── Delivery Summary
│   ├── Evidence Summary
│   └── Missing Evidence Warning
│
└── Audit Agent
    ├── Human Audit Request
    ├── Audit Report
    ├── Findings
    ├── Evidence Map
    └── Traceability
```

### 4.2 默认隐藏的信息

默认隐藏：

```text
JSON 原文
manifest
index
locks
sessions
state timeline
raw blocker sourcePath
execute run internals
audit package paths
Panel search raw result
```

可以展开：

```text
查看高级详情
查看上下文包
查看 evidence map
查看 traceability
查看原始状态
复制调试信息
```

### 4.3 主导航建议

V1 不需要复杂导航。

建议主导航压成 4 个入口：

```text
项目首页
需求
交给 Codex
审计
```

高级入口：

```text
高级详情
```

旧的 overview / projects / goal-tree / issues / evidence / reviews / views 可以先降噪，逐步合并到这条人类动线里。

---

## 5. 页面清单

| 页面 | 目标 | 主按钮 |
| --- | --- | --- |
| Project Onboarding | 让用户打开项目并知道 AgentFlow 做了什么准备 | 打开本地项目 |
| Project Home | 告诉用户现在项目状态和下一步 | 根据 next action 动态变化 |
| Next Step Card | 把 state nextActions 翻译成一个主动作 | 动态变化 |
| Agent Cards | 让用户理解 Spec / Build / Audit 三个角色 | 进入当前可用 Agent |
| Spec Agent Flow | 把想法变成 SPEC Draft | 确认理解 / 生成 SPEC Draft |
| SPEC Review | 让人类拍板 approved SPEC | 确认 SPEC |
| Codex Handoff | 生成可复制的 Codex 指令 | 复制 Codex 指令 |
| Output / Delivery Summary | 展示 Codex 交付材料和证据情况 | 去审计 |
| Human Audit | 触发和查看人工审计 | 请求人工审计 |
| Status Channel | 展示系统状态，但说人话 | 无主按钮 |
| Advanced Details | 给高级用户看内部状态 | 复制调试信息 |

---

## 6. 页面详细设计

### 6.1 Project Onboarding

#### 页面目标

让用户第一次打开 AgentFlow 时知道：

```text
这是一个给 Codex / Claude Code / Cursor 配套的本地工作流控制面。
第一步是打开本地项目。
AgentFlow 会准备规则和项目现场。
```

#### 页面结构

```text
顶部说明
  AgentFlow 帮你把需求整理成 Codex 能执行的任务。

项目入口
  [打开本地项目]

准备说明
  1. 准备 Agent 工作手册
  2. 读取项目现场
  3. 建立工作流状态
  4. 告诉你下一步该做什么

边界提示
  AgentFlow 不会直接改你的源码。
```

#### 状态

| 状态 | 文案 |
| --- | --- |
| empty | 还没有打开项目。先选择一个本地项目。 |
| loading | 正在准备项目。 |
| ready | 项目已准备好。 |
| warning | 项目可以使用，但有一些提醒。 |
| blocked | 项目还不能使用。需要先处理阻断原因。 |

#### 主按钮

```text
打开本地项目
```

#### 数据依赖

```text
Project Workspace Manager
useProjectFiles
useAgentManual
useProjectPanel
useStateStatus
prepare_agent_working_manual
prepare_project_panel
load_state_status
```

---

### 6.2 Project Home

#### 页面目标

项目打开后，不让用户面对空页面。

Project Home 只回答 3 个问题：

```text
项目准备好了吗？
我下一步做什么？
可以用哪个 Agent？
```

#### 页面结构

```text
顶部：项目状态条
  工作手册：已就绪
  项目现场：已读取
  需求：等待确认
  交付：暂无
  审计：未请求
  语言：zh-CN
  风格：plain-work-style

中间：下一步卡片
  标题
  说明
  主按钮
  阻断原因

下面：三个 Agent 卡片
  Spec Agent
  Build Agent
  Audit Agent

右侧 / 下方：高级详情
  Panel summary
  State blockers
  Output summary
```

#### 主按钮

主按钮由 `state.nextActions` 和 `state.currentStage` 决定。

| 状态 | 主按钮 |
| --- | --- |
| workspace-ready / panel-ready | 告诉 Agent 你想做什么 |
| input-ready / issue missing | 继续整理 SPEC |
| issue-ready / execute-ready | 复制 Codex 指令 |
| delivery-ready | 请求人工审计 |
| audit-completed | 查看审计报告 |

#### 页面跳转

```text
告诉 Agent 你想做什么 -> Spec Agent Flow
继续整理 SPEC -> Spec Agent Flow / SPEC Review
复制 Codex 指令 -> Codex Handoff
请求人工审计 -> Human Audit
查看审计报告 -> Human Audit Report
查看高级详情 -> Advanced Details
```

#### 数据依赖

```text
agentManualState.status
projectPanelState.status
projectPanelState.manifest
inputStatusState.status
executeStatusState.status
outputStatusState.status
stateStatusState.status
StateStatusSnapshot.currentStage
StateStatusSnapshot.nextActions
StateStatusSnapshot.blockers
AgentEnvironmentStatus.locale.agentLocale
AgentEnvironmentStatus.style.styleId
```

---

### 6.3 Spec Agent Flow

#### 页面目标

把用户一句模糊想法，整理成能确认的 SPEC Draft。

#### 入口不要是空白聊天框

第一屏给意图入口：

```text
我要做一个新功能
我要修一个 bug
我要重构一块代码
我要理解这个项目
我要审计一次交付
```

每个入口下面给一句示例：

```text
例：给订单列表加一个按状态筛选的功能。
例：修复登录后页面白屏的问题。
例：把支付模块里的重复校验逻辑收敛成一个函数。
```

#### 流程

```text
选择意图入口
  ↓
输入需求
  ↓
Agent 复述理解
  ↓
最多问 3 个问题
  ↓
生成 SPEC Draft Preview
  ↓
人类确认
  ↓
Approved SPEC + Issue
```

#### 页面组件

```text
IntentEntryGrid
RequirementInputBox
UnderstandingSummary
KnownInfoList
MissingInfoQuestions
ScopePanel
NonGoalsPanel
AcceptanceCriteriaList
RiskHint
SpecDraftPreview
GeneratedIssuePreview
HumanConfirmGate
```

#### 状态

| 状态 | 页面表现 |
| --- | --- |
| empty | 选择一个入口，不显示空白聊天框 |
| drafting | 显示“正在整理需求” |
| need-more-info | 显示最多 3 个问题 |
| draft-ready | 显示 SPEC Draft Preview |
| approved | 显示 Approved SPEC + Issue |
| blocked | 用人话说明不能继续的原因 |

#### 主按钮

```text
确认理解
生成 SPEC Draft
确认 SPEC
```

#### 不能假设的能力

V1 不假设已经有模型调用。

如果没有模型能力，Spec Agent Flow 可以先做成：

```text
结构化表单
本地模板化草稿
人工可编辑 SPEC Draft Preview
```

---

### 6.4 SPEC Review

#### 页面目标

让人类清楚知道：

```text
这是 AgentFlow 理解后的需求。
确认后，才能交给 Codex。
```

#### 页面结构

```text
左侧：SPEC 内容
  需求摘要
  背景
  范围
  非目标
  验收标准
  风险
  受影响区域

右侧：确认门
  我确认这个 SPEC 可以作为 Codex 执行依据
  主按钮：确认 SPEC
```

#### 人类确认门

确认前必须明确提示：

```text
确认后，AgentFlow 会把这个需求作为 approved SPEC。
Codex Handoff 会基于它生成。
后续如果要改需求，需要回到 SPEC 再修改。
```

#### 主按钮

```text
确认 SPEC
```

#### 次按钮

```text
返回修改
保存草稿
```

#### 数据依赖

```text
Input status
load_input_status
load_input_index
load_input_snapshot
Approved SPEC path: .agentflow/input/specs/approved/<spec-id>/
Issue path: .agentflow/input/issues/<issue-id>.json
Panel Context Pack
```

---

### 6.5 Codex Handoff

#### 页面目标

Build Agent 页面不是“执行代码”。

它只做一件事：

```text
生成 Codex 能复制执行的任务包。
```

#### 页面内容

```text
Issue title
Source SPEC
Risk level
Allowed paths
Forbidden actions
Validation commands
Panel Context Pack
Copyable Codex instruction
```

#### 页面结构

```text
顶部：任务摘要
  Issue
  SPEC
  风险
  当前状态

中间：Codex Handoff Package
  目标
  范围
  禁止事项
  必须读取的上下文
  验证命令
  输出要求

底部：可复制指令
  [复制 Codex 指令]
```

#### 主按钮

```text
复制 Codex 指令
```

#### 次按钮

```text
查看上下文包
返回修改 SPEC
```

#### Codex 指令模板

```text
你是 Codex。请在当前项目中执行 AgentFlow handoff 任务。

任务：
<issue title>

来源：
- Approved SPEC: <spec ref>
- Issue: <issue ref>
- Panel Context Pack: <context pack ref>

你可以做：
- <allowed paths>
- <allowed actions>

你不能做：
- 不要改未授权路径
- 不要创建远程 PR
- 不要 merge
- 不要 deploy
- 不要绕过验证
- 不要删除 AgentFlow 管理文件

完成标准：
- <acceptance criteria>

必须运行或说明不能运行的验证：
- <validation commands>

交付时请返回：
- 修改摘要
- 关键 diff
- 验证结果
- evidence
- release delivery
- 任何未完成项
```

#### 数据依赖

```text
Approved SPEC
Issue
Panel Context Pack
Input index
Panel status
Execute status
State gates
State nextActions
State blockers
```

---

### 6.6 Human Audit

#### 页面目标

把现有 Human Audit Panel 包装成人类更容易理解的 Audit Agent 流程。

#### 流程

```text
选择 release delivery
  ↓
填写 audit reason
  ↓
自动生成 scope refs
  ↓
请求人工审计
  ↓
查看 audit report
  ↓
接受 / 返工 / 补证据
```

#### 页面内容

```text
审计状态
主要发现
风险
证据映射
追溯关系
建议动作
```

#### 主按钮

```text
请求人工审计
```

#### 结果动作

```text
接受
返工
补证据
```

V1 只做展示和请求，不做自动修复。

#### 已有代码可复用

当前已有 `OutputAuditPanel`，已经包含：

```text
选择 release delivery
填写 audit reason
request_human_audit
load_audit_report
audit report
findings
evidence map
traceability
```

V1 体验优化不应该重写这个能力，而应该：

```text
把它放进 Audit Agent 页面
把状态文案翻译成人话
把 JSON details 默认折叠
把主按钮保持为“请求人工审计”
```

#### 数据依赖

```text
load_output_status
load_output_index
load_audit_index
load_audit_report
request_human_audit
OutputStatusSnapshot
OutputIndex
AuditIndex
HumanAuditReport
```

---

### 6.7 Status Channel

#### 页面目标

状态通道负责把内部系统状态翻译成人话。

当前代码已有 `buildAgentStatusItems`，它把 workspace / ownership / worksite / input / execute / output / workflow-state / agent-manual 聚合成状态项。

V1 需要进一步减少技术噪音。

#### 用户可见状态

| 内部状态 | 用户看到 |
| --- | --- |
| define ready | 工作手册：已就绪 |
| panel ready | 项目现场：已读取 |
| input ready | 需求：已准备 |
| approved spec missing | 需求：等待确认 |
| issue ready | 任务：已就绪 |
| delivery-ready | 交付：可审计 |
| audit not-requested | 审计：未请求 |
| audit running | 审计：进行中 |
| blockers.length > 0 | 有阻断 |

#### 必须展示的策略状态

```text
语言：<agentLocale>
风格：plain-work-style
```

示例：

```text
语言：zh-CN
风格：plain-work-style
```

#### 文案翻译规则

坏：

```text
Workflow gate blocked: missing input approved spec.
```

好：

```text
还不能交给 Codex。原因是：这个需求还没有确认成 SPEC。
```

#### 数据依赖

```text
buildAgentStatusItems
AgentStatusChannelItem
AgentEnvironmentStatus.locale.agentLocale
AgentEnvironmentStatus.style.styleId
StateStatusSnapshot.currentStage
StateStatusSnapshot.auditStatus
StateStatusSnapshot.blockers
```

---

### 6.8 Advanced Details

#### 页面目标

高级详情给开发者和 Agent 使用，不打扰普通用户。

#### 默认折叠内容

```text
Agent Manual raw status
Panel manifest
Input index
Execute index
Output index
State health
Workflow gates
Next actions raw
Blockers raw
Locks
Sessions
Audit report raw fields
```

#### 交互

```text
展开
复制 JSON
复制调试信息
复制 source path
```

#### 原则

```text
高级详情不能成为主路径。
主路径永远只给一个下一步动作。
```

---

## 7. 状态与空态设计

### 7.1 Next Step Card 规则

Next Step Card 根据 `stateStatusState.status` 渲染。

#### 项目刚准备好

```text
标题：项目已准备好
说明：AgentFlow 已经准备好规则和项目现场。
主按钮：告诉 Agent 你想做什么
```

#### 需求未确认

```text
标题：先确认需求
说明：还不能交给 Codex。原因是：需求还没有确认成 SPEC。
主按钮：继续整理 SPEC
```

#### 可以交给 Codex

```text
标题：可以交给 Codex 了
说明：这个任务已经有 approved SPEC 和 Issue。
主按钮：复制 Codex 指令
```

#### 可以审计

```text
标题：可以审计交付结果
说明：Codex 已经返回交付材料，可以请求人工审计。
主按钮：请求人工审计
```

#### 被阻断

```text
标题：现在还不能继续
说明：<把第一个 blocker.reason 翻译成人话>
主按钮：查看需要处理的事项
```

### 7.2 Loading

```text
正在读取项目现场。
这一步只读取本地信息，不会改你的源码。
```

### 7.3 Warning

```text
项目可以继续，但有提醒。
建议先看一下高级详情。
```

### 7.4 Blocked

```text
现在不能继续。
原因是：<人话原因>。
```

### 7.5 Empty

```text
还没有任务。
先告诉 Agent 你想做什么。
```

---

## 8. 组件拆分

### 8.1 页面级组件

```text
ProjectOnboardingPage
ProjectHomePage
SpecAgentPage
SpecReviewPage
CodexHandoffPage
OutputDeliveryPage
HumanAuditPage
AdvancedDetailsPage
```

### 8.2 核心业务组件

```text
ProjectStatusBar
NextStepCard
AgentCards
AgentCard
IntentEntryGrid
RequirementInputPanel
UnderstandingSummary
ClarifyingQuestions
SpecDraftPreview
HumanConfirmGate
CodexHandoffPackage
CopyableInstruction
DeliverySummaryCard
AuditRequestPanel
AuditReportPanel
StatusChannel
AdvancedJsonDetails
```

### 8.3 Hooks 建议

复用当前 hooks：

```text
useAgentManual
useProjectPanel
useInputStatus
useExecuteStatus
useOutputStatus
useStateStatus
```

新增前端组合 hook：

```text
useHumanNextStep
useAgentCards
useCodexHandoffDraft
useReadableBlockers
```

这些 hook 只做前端组合和文案翻译，不新增后端能力。

---

## 9. 数据依赖映射

### 9.1 Project Onboarding

依赖：

```text
Project Workspace Manager
prepare_agent_working_manual
prepare_project_panel
load_state_status
```

前端状态：

```text
projectFilesState
agentManualState
projectPanelState
stateStatusState
```

### 9.2 Project Home

依赖：

```text
Agent Manual status
Panel status
Input status
Execute status
Output status
State status
Workflow gates
Next actions
Blockers
```

当前可用数据：

```text
agentManualState.status
projectPanelState.status
projectPanelState.manifest
inputStatusState.status
executeStatusState.status
outputStatusState.status
stateStatusState.status
```

### 9.3 Next Step Card

依赖：

```text
load_state_status
load_next_actions
load_blockers
```

当前可先用：

```text
StateStatusSnapshot.currentStage
StateStatusSnapshot.nextActions
StateStatusSnapshot.blockers
StateStatusSnapshot.auditStatus
StateStatusSnapshot.activeIssueId
StateStatusSnapshot.activeRunId
```

### 9.4 Agent Cards

依赖：

```text
Agent Manual status
Input status
Output status
State status
```

映射：

```text
Spec Agent -> inputStatus + approved SPEC / issue status
Build Agent -> issue-ready / execute-ready / panel context pack
Audit Agent -> outputStatus + auditStatus + release delivery
```

### 9.5 Spec Agent Flow

依赖：

```text
load_input_status
load_input_index
load_input_snapshot
Panel status
Panel Context Pack
```

注意：

```text
V1 不假设模型调用。
没有模型时，用结构化表单和人工可编辑 draft。
```

### 9.6 Codex Handoff

依赖：

```text
Approved SPEC
Issue
Panel Context Pack
State gates
State blockers
Execute status
```

可用命令 / 能力：

```text
load_input_index
load_input_snapshot
build_panel_context_pack
load_execute_status
load_state_status
load_workflow_gates
load_next_actions
load_blockers
```

### 9.7 Output / Delivery Summary

依赖：

```text
load_output_status
load_output_index
load_output_snapshot
OutputStatusSnapshot.summary.evidence
OutputStatusSnapshot.summary.releaseDeliveries
OutputStatusSnapshot.summary.incompleteEvidence
OutputStatusSnapshot.summary.incompleteDeliveries
```

### 9.8 Human Audit

依赖：

```text
load_output_status
load_output_index
load_audit_index
load_audit_report
request_human_audit
```

数据：

```text
OutputIndex.releaseDeliveries
AuditIndex.audits
HumanAuditReport.reportMarkdown
HumanAuditReport.findings
HumanAuditReport.evidenceMap
HumanAuditReport.traceability
```

### 9.9 Status Channel

依赖：

```text
buildAgentStatusItems
AgentStatusChannelItem
AgentEnvironmentStatus
PanelStatusSnapshot
InputStatusSnapshot
ExecuteStatusSnapshot
OutputStatusSnapshot
StateStatusSnapshot
```

---

## 10. 文案示例

### 10.1 首页说明

```text
AgentFlow 帮你把想法整理成 Codex 能执行的任务。
先把需求说清楚，再交给 Codex 做。
做完后，你可以按证据审计结果。
```

### 10.2 Project Home

```text
项目已准备好
AgentFlow 已经准备好工作手册和项目现场。
下一步：告诉 Agent 你想做什么。
```

### 10.3 Spec Agent

```text
先说你想改什么。
不用一次说完整。Agent 会帮你整理范围、非目标和验收标准。
```

### 10.4 SPEC Review

```text
请确认这份 SPEC。
确认后，它会成为 Codex 执行任务的依据。
```

### 10.5 Codex Handoff

```text
可以交给 Codex 了。
这个任务已经有 confirmed SPEC、Issue 和项目上下文。
复制下面的指令到 Codex。
```

### 10.6 Audit

```text
可以审计交付结果。
选择一次 delivery，填写为什么要审计，然后请求人工审计。
```

### 10.7 Blocked

```text
还不能交给 Codex。
原因是：这个需求还没有确认成 SPEC。
```

### 10.8 Browser Preview

```text
浏览器预览只读，不会写入 .agentflow/output/audit。
请在 Tauri Desktop 中请求人工审计。
```

---

## 11. 实现优先级

### P0：收敛首页和状态动线

目标：

```text
用户打开项目后知道下一步做什么。
```

范围：

```text
Project Home
Project Status Bar
Next Step Card
Agent Cards
Readable blocker copy
Advanced Details 折叠
```

不做：

```text
不新增模型调用
不新增自动执行
不新增后端能力
```

### P1：Spec Agent Flow 壳层

目标：

```text
用户能从意图入口进入需求整理。
```

范围：

```text
Intent Entry
Requirement Input
Understanding Review
Clarifying Questions
SPEC Draft Preview
SPEC Review
```

不做：

```text
不自动生成高质量 SPEC
不假设 LLM
不自动写 approved SPEC
```

### P2：Codex Handoff

目标：

```text
把 approved SPEC + Issue + Panel Context Pack 转成可复制指令。
```

范围：

```text
Codex Handoff Package
Copyable instruction
Allowed / forbidden / validation / evidence requirements
```

不做：

```text
不自动打开 Codex
不自动执行 Codex
不创建远程 PR
```

### P3：Audit Agent 包装

目标：

```text
把现有 OutputAuditPanel 变成更清楚的 Audit Agent 页面。
```

范围：

```text
Delivery selector
Audit reason
Request human audit
Audit report
Findings / evidence map / traceability
Result actions placeholder
```

不做：

```text
不自动修复
不自动返工
不自动触发 audit
```

### P4：视觉和组件整理

目标：

```text
低噪音、开发者工具风、状态清楚。
```

范围：

```text
布局
间距
卡片层级
按钮优先级
空态 / loading / warning / blocked / ready
```

---

## 12. 不做事项

V1 不做：

```text
不直接生成 React 代码
不把内部目录结构直接当页面
不把所有功能都放首页
不新增后端能力
不假设已经有模型调用
不假设可以自动执行 Codex
不做 AI IDE
不做代码编辑器
不做 Linear 替代品
不做完整 Agent 平台
不自动创建远程 PR
不自动 merge
不自动 deploy
不自动写用户源码
不把 Browser Preview 变成写入入口
不把 Audit 变成默认自动流程
不展示 manifest / locks / sessions 作为默认主内容
```

---

## 13. Figma AI 生成提示词

```text
你是一个开发者工具产品设计师。请为 AgentFlow 设计一套桌面端前端页面。

产品定位：
AgentFlow 是给 Codex / Claude Code / Cursor 配套的本地 Agent 工作流控制面。Codex 负责干活，AgentFlow 负责让 Codex 知道该干什么、不能干什么、做到什么算完成。

设计目标：
让用户第一次打开 AgentFlow 就知道：
1. AgentFlow 是什么
2. 项目准备好了吗
3. 下一步该做什么
4. 为什么不能直接执行
5. 怎么把任务交给 Codex
6. 怎么检查 Codex 做得对不对

视觉风格：
专业、干净、低噪音、开发者工具风。参考 Linear 的清晰状态、Warp 的 Agent 工作流纪律、Zed 的项目现场感、Codex 的执行工具感。不要营销型 SaaS 首页，不要游戏化。

页面：
1. Project Onboarding
2. Project Home
3. Next Step Card
4. Agent Cards
5. Spec Agent Flow
6. SPEC Review
7. Codex Handoff
8. Output / Delivery Summary
9. Human Audit
10. Status Channel
11. Advanced Details

Project Home 布局：
顶部：项目状态条
- 工作手册
- 项目现场
- 需求
- 交付
- 审计
- 语言
- 风格

中间：下一步卡片
- 当前建议动作
- 为什么是这个动作
- 一个主按钮
- 阻断原因

下面：三个 Agent 卡片
- Spec Agent：帮你把想法整理成可确认的需求
- Build Agent：把确认后的需求打包成 Codex 能执行的任务
- Audit Agent：检查 Codex 做完的结果是否符合原需求

右侧或下方：高级详情
- Panel summary
- State blockers
- Output summary
默认折叠 JSON / manifest / locks / sessions。

Next Step Card 文案示例：
- 项目已准备好：AgentFlow 已经准备好规则和项目现场。主按钮：告诉 Agent 你想做什么
- 先确认需求：还不能交给 Codex。原因是：需求还没有确认成 SPEC。主按钮：继续整理 SPEC
- 可以交给 Codex 了：这个任务已经有 approved SPEC 和 Issue。主按钮：复制 Codex 指令
- 可以审计交付结果：Codex 已经返回交付材料，可以请求人工审计。主按钮：请求人工审计

Spec Agent Flow：
不要空白聊天框。第一屏给 5 个意图入口：
- 我要做一个新功能
- 我要修一个 bug
- 我要重构一块代码
- 我要理解这个项目
- 我要审计一次交付

Codex Handoff 页面：
展示 Issue title、Source SPEC、Risk level、Allowed paths、Forbidden actions、Validation commands、Panel Context Pack、Copyable Codex instruction。主按钮：复制 Codex 指令。

Human Audit 页面：
展示 delivery selector、audit reason、request human audit、audit report、findings、evidence map、traceability。主按钮：请求人工审计。结果动作：接受、返工、补证据。

文案风格：
使用中文大白话，短句，少废话。不要写“赋能”“全链路闭环”“智能化编排”。
```

---

## 14. Codex 前端实现指令

```text
你是 Codex。请在 atxinbao/agentflow 仓库里实现 AgentFlow Human-Agent Guided Experience V1 的前端体验切片。

先阅读：
- docs/requirements/015-human-agent-guided-experience-v1.md
- README.md
- GOAL.md
- ROADMAP.md
- apps/desktop/src/App.tsx
- apps/desktop/src/features/status-channel/statusAdapters.ts
- apps/desktop/src/features/agent-manual/hooks/useAgentManual.ts
- apps/desktop/src/features/project-files/hooks/useProjectPanel.ts
- apps/desktop/src/features/input/**
- apps/desktop/src/features/execute/**
- apps/desktop/src/features/output/OutputAuditPanel.tsx
- apps/desktop/src/features/state/hooks/useStateStatus.ts
- apps/desktop/src/types/status.ts

目标：
把 Desktop 首页从“底层模块展示”收敛为人类可走的 AgentFlow guided workflow。

必须实现：
1. Project Home 新结构：
   - 顶部 Project Status Bar
   - 中间 Next Step Card
   - 下方 Spec / Build / Audit 三个 Agent Card
   - Advanced Details 默认折叠

2. Next Step Card：
   - 基于 StateStatusSnapshot.currentStage / nextActions / blockers / auditStatus 渲染
   - 每次只突出一个主按钮
   - blocker.reason 必须翻译成人话
   - 支持 ready / warning / blocked / loading / empty

3. Agent Cards：
   - Spec Agent：帮你把想法整理成可确认的需求
   - Build Agent：把确认后的需求打包成 Codex 能执行的任务
   - Audit Agent：检查 Codex 做完的结果是否符合原需求
   - 根据 state/input/output 状态显示可用 / 等待 / 阻断

4. Status Channel 降噪：
   - 默认展示工作手册、项目现场、需求、交付、审计、语言、风格
   - manifest / locks / sessions / raw JSON 放入 Advanced Details

5. Codex Handoff 页面壳层：
   - 展示 Issue / SPEC / Risk / Allowed / Forbidden / Validation / Context Pack
   - 提供 Copyable Codex instruction
   - 不自动执行 Codex

6. Audit Agent 页面：
   - 复用 OutputAuditPanel 能力
   - 包装成更清楚的 Audit Agent 页面
   - JSON details 默认折叠
   - 主按钮保持“请求人工审计”

不允许做：
- 不新增模型调用
- 不新增后端能力，除非另有需求文档
- 不自动执行 Codex
- 不调用 execute 的写入命令作为 human UI 主流程
- 不创建远程 PR
- 不 merge
- 不 deploy
- 不写用户业务源码
- 不把 Browser Preview 变成写入入口
- 不恢复独立 Release Agent
- 不把内部目录结构直接当页面

可用数据：
- useAgentManual
- useProjectPanel
- useInputStatus
- useExecuteStatus
- useOutputStatus
- useStateStatus
- load_state_status
- load_next_actions
- load_blockers
- load_output_index
- load_audit_index
- load_audit_report
- request_human_audit

验证命令：
- npm --prefix apps/desktop run build
- npm --prefix apps/desktop run preview:smoke
- cargo test -p agentflow-desktop
- cargo test -p agentflow-state
- cargo test -p agentflow-output
- git diff --check

交付要求：
- PR description 写清楚用户可见变化
- 写清楚没有新增模型调用、没有自动执行 Codex、没有远程 PR/merge/deploy
- 写清楚验证命令和结果
```
