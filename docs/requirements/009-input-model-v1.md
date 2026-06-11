# 009 - Input Model V1

创建日期：2026-06-04  
执行者：Codex  
状态：已实现  
版本：final-draft

---

## 用户目标

当前 AgentFlow 已经完成了：

```text
define/
= Agent 工作手册 / 规则

panel/
= 项目工作现场

execute/
= 未来执行过程

output/
= 证据 / 审计 / 发布 / 日志

state/
= 健康 / 锁 / 会话 / 索引状态
```

现在需要进入下一阶段：

```text
input/
= 需求实时源头
```

原来有两个顶层目录：

```text
spec/
goal-tree/
```

现在需要统一收敛成：

```text
input/
```

原因：

```text
spec/ 和 goal-tree/ 本质都是“输入侧”的内容。
spec/ 负责需求说明书。
goal-tree/ 负责需求拆出来的施工清单。
这两者都属于 Agent 后续执行前的“输入事实源”。
```

新的主结构应该是：

```text
.agentflow/
├── define/
├── panel/
├── input/
├── execute/
├── output/
└── state/
```

大白话：

> **define 是规矩，panel 是现场，input 是要做什么。**

---

## 一句话定义

> **当前修订：Issue 对人和调度暴露 `priority: p0 | p1 | p2 | p3`，用于决定处理顺序；技术执行风险改名为 `executionRisk: low | medium | high`，只用于执行前安全确认。旧 `priority: high | normal | low` 和 `riskLevel` 由 prepare 自动迁移。**

---

# 1. 核心原则

## 1.1 `input/` 是需求事实源

```text
input/
= 需求沟通
= 需求过滤
= 需求说明书
= 需求施工清单
```

它回答：

```text
用户要做什么？
这个需求是否已经说清楚？
需求说明书是什么？
是否已经人类确认？
后续要生成哪些 issues？
复杂需求是否需要 project 容器？
issues 之间有没有关系？
```

---

## 1.2 `input/` 不做执行

`input/` 不负责：

```text
不写用户源码
不执行命令
不跑测试
不启动 AgentRun
不创建 PR
不发布
不审计执行结果
```

这些以后属于：

```text
execute/
output/
```

---

## 1.3 `spec/` 和 `goal-tree/` 不再作为顶层主路径

从 009 开始，新的 canonical path 是：

```text
.agentflow/input/
```

旧路径：

```text
.agentflow/spec/
.agentflow/goal-tree/
```

后续作为 legacy 处理。

本需求 V1 推荐：

```text
不强制删除旧 spec/
不强制删除旧 goal-tree/
不强迁移旧数据
但新写入只允许进入 .agentflow/input/**
```

---

# 2. Input 主流程

Input 的主流程用大白话表示是：

```text
输入准备
→ 需求沟通与过滤
→ 需求说明书
→ 需求施工清单
```

对应英文：

```text
Input Prepare
→ Intake
→ Spec Gate
→ Issue Generation
```

---

## 2.1 输入准备

对应：

```text
.agentflow/input/manifest.json
.agentflow/input/index.json
```

职责：

```text
准备 input 目录
准备 input manifest
准备 input index
确认 define ready
确认 panel ready / degraded
确认 ownership 已通过
确认 Spec Agent 当前能力边界
```

它不生成正式需求，也不生成 issue。

---

## 2.2 需求沟通与过滤

对应：

```text
.agentflow/input/intake/<intake-id>.json
```

职责：

```text
记录用户原始输入
记录需求类型
记录 Agent 初步理解
记录澄清问题
记录是否 ready-for-spec
记录是否 blocked-by-boundary
```

状态：

```text
needs-clarification
answer-only
blocked-by-boundary
ready-for-spec
```

大白话：

> **先把人话听懂，再判断能不能进入需求说明书。**

---

## 2.3 需求说明书

对应：

```text
.agentflow/input/specs/
```

需求说明书不是一个文件，而是一组文件：

```text
product.md
tech.md
approval.json
spec.json
```

这一步是所有 issue 的共同前置。

也就是：

```text
所有正式 issue，都必须从 Approved SPEC 生成。
```

---

## 2.4 需求施工清单

对应：

```text
.agentflow/input/issues/
.agentflow/input/projects/
.agentflow/input/relations/
```

它负责把已经批准的需求说明书，变成后续 Agent 能处理的施工清单。

V1 只支持两种模型：

```text
简单需求：
Approved SPEC
→ Issues

复杂需求：
Approved SPEC
→ Project
→ Issues
```

不做三层：

```text
Goal / Milestone / Issue
```

V1 把旧的 goal 理解成新的 project 结构层。

---

# 3. Input 目录结构

目标结构：

```text
.agentflow/input/
├── manifest.json
├── index.json
│
├── intake/
│   └── <intake-id>.json
│
├── specs/
│   ├── drafts/
│   │   └── <spec-id>/
│   │       ├── product.md
│   │       ├── tech.md
│   │       └── spec.json
│   │
│   ├── approved/
│   │   └── <spec-id>/
│   │       ├── product.md
│   │       ├── tech.md
│   │       ├── approval.json
│   │       └── spec.json
│   │
│   └── archive/
│
├── projects/
│   └── <project-id>.json
│
├── issues/
│   └── <issue-id>.json
│
├── relations/
│   ├── issue-relations.json
│   └── dependency-graph.json
│
└── views/
    ├── active.json
    ├── blocked.json
    ├── by-spec.json
    └── by-project.json
```

---

# 4. Spec Gate

## 4.1 Spec Gate 是什么

Spec Gate 是所有 issue 生成前的共同前置条件。

它由三部分组成：

```text
product.md
tech.md
approval.json
```

大白话：

```text
product.md
= 要做什么，为什么做，做到什么算完成

tech.md
= 现在代码是什么情况，准备怎么改，怎么验证

approval.json
= 人类确认：这版说明书可以生成施工清单
```

没有 approval：

```text
不能生成 issue
不能生成 project
不能进入 execute
```

---

## 4.2 product.md

路径：

```text
.agentflow/input/specs/drafts/<spec-id>/product.md
.agentflow/input/specs/approved/<spec-id>/product.md
```

职责：

```text
从用户 / 产品角度定义需求
```

建议包含：

```text
Summary
Problem
Goals
Non-goals
User / Agent behavior
Edge cases
Success criteria
Validation
Open questions
```

---

## 4.3 tech.md

路径：

```text
.agentflow/input/specs/drafts/<spec-id>/tech.md
.agentflow/input/specs/approved/<spec-id>/tech.md
```

职责：

```text
从工程 / 代码角度定义方案
```

建议包含：

```text
Context
Relevant files
Current state
Proposed changes
Data model
API / command changes
Risks
Testing and validation
Follow-ups
```

---

## 4.4 approval.json

路径：

```text
.agentflow/input/specs/approved/<spec-id>/approval.json
```

职责：

```text
记录人类确认
记录允许生成什么
记录不允许做什么
```

示例：

```json
{
  "version": "input-spec-approval.v1",
  "specId": "spec-001",
  "approvedBy": "human",
  "approvedAt": 1780360000,
  "approvedProductHash": "<sha256>",
  "approvedTechHash": "<sha256>",
  "issueGenerationMode": "project",
  "authorizedOutputs": [
    "project",
    "issues"
  ],
  "notAuthorized": [
    "execute",
    "sourceWrites",
    "remotePr",
    "merge",
    "release"
  ]
}
```

---

# 5. 两种 issue 落地模型

## 5.1 简单需求：Direct Issues

适合：

```text
小 bug
小 polish
单点修复
文档修正
轻量清理
单模块小功能
```

流程：

```text
Approved SPEC
→ Issues
```

落地：

```text
.agentflow/input/issues/iss-001.json
```

不生成：

```text
project
goal
milestone
```

Issue 里：

```json
{
  "issueModel": "direct",
  "projectId": null
}
```

---

## 5.2 复杂需求：Project -> Issues

适合：

```text
系统级功能
多模块改造
多步骤开发
需要拆多张任务卡
需要多个 Agent 后续执行
```

流程：

```text
Approved SPEC
→ Project
→ Issues
```

落地：

```text
.agentflow/input/projects/proj-001.json
.agentflow/input/issues/iss-001.json
.agentflow/input/issues/iss-002.json
```

Project 里包含多个 issue：

```json
{
  "projectId": "proj-001",
  "issueIds": [
    "iss-001",
    "iss-002"
  ]
}
```

Issue 里挂回 Project：

```json
{
  "issueModel": "project",
  "projectId": "proj-001"
}
```

---

# 6. Project 模型

路径：

```text
.agentflow/input/projects/<project-id>.json
```

示例：

```json
{
  "version": "input-project.v1",
  "projectId": "proj-001",
  "sourceSpecId": "spec-001",
  "title": "Workspace Ownership Guard",
  "summary": "为 .agentflow/ 增加归属权保护。",
  "objective": "防止 AgentFlow 误写入非自己管理的 .agentflow 目录。",
  "scope": [
    "ownership check",
    "foreign blocked",
    "takeover API",
    "Panel prepare guard"
  ],
  "nonGoals": [
    "不实现 OpenSpec Authoring",
    "不启动 AgentRun"
  ],
  "successCriteria": [
    "foreign .agentflow 不写入",
    "managed-current 可以 repair",
    "takeover 显式触发"
  ],
  "issueIds": [
    "iss-001",
    "iss-002",
    "iss-003"
  ],
  "status": "planned",
  "panel": {
    "snapshotId": "panel-snapshot-001",
    "contextPackId": "ctx-001"
  },
  "system": {
    "createdBy": "Spec Agent",
    "createdAt": 1780360000,
    "updatedAt": 1780360000,
    "path": ".agentflow/input/projects/proj-001.json",
    "revision": 1
  }
}
```

---

# 7. Issue 模型

路径：

```text
.agentflow/input/issues/<issue-id>.json
```

Issue 是后续执行的统一入口。

不管是简单需求还是复杂需求，最终都落到：

```text
input/issues/
```

---

## 7.1 Issue 字段

Issue V1 字段：

```text
version
issueId
issueModel
sourceSpecId
projectId

title
summary
kind
priority
status
executionRisk

scope
nonGoals
acceptanceCriteria
validationHints

relations
panel
system
```

---

## 7.2 priority 与 executionRisk

Issue 的处理顺序使用：

```json
{
  "priority": "p1"
}
```

取值：

```text
p0
p1
p2
p3
```

执行阶段另保留内部技术风险：

```json
{
  "executionRisk": "medium"
}
```

`priority` 由 Spec Agent 根据任务紧急程度和业务顺序生成；`executionRisk` 只决定是否需要执行前人工确认。二者不能互相替代。

`displayStatus` 面向前端展示，可取：

```text
backlog
blocked
ready
in-progress
review
done
cancel
```

其中 `blocked` 是 state 派生状态。Spec Agent 默认只生成 backlog / ready / done / cancel 这类事实状态；依赖未完成、Git provider 预检失败或执行预检失败时，由 state/gates 派生为 blocked。

---

## 7.3 人类确认规则

V1 只定义这一条：

```text
low
= 不需要人类确认

medium
= 不需要人类确认

high
= 需要人类确认
```

也就是说：

> **只有 high risk issue 执行前需要人类确认。**

---

## 7.4 不新增复杂自动化字段

Issue V1 不新增：

```text
automation
humanGates
prAutomation
requiresHumanConfirmation
allowedAgentActions
blockedAgentActions
riskReasons
riskFactors
```

原因：

```text
Issue 是施工清单事实源，不是自动化策略文件。
```

后续 Agent 读到：

```json
"executionRisk": "high"
```

就知道执行前需要人类确认。

规则写在：

```text
define/agent/Agentflow.md
define/tdd/TDD.md
define/release/RELEASE.md
define/audit/AUDIT.md
```

不是写进 issue 数据里。

---

## 7.5 Issue 示例

```json
{
  "version": "input-issue.v1",
  "issueId": "iss-001",
  "issueModel": "project",
  "sourceSpecId": "spec-001",
  "projectId": "proj-001",

  "title": "新增 WorkspaceOwnershipStatus 模型",
  "summary": "定义 .agentflow ownership 状态、marker、recommended action。",
  "kind": "feature",
  "priority": "p2",
  "status": "planned",
  "executionRisk": "medium",

  "scope": [
    "新增 WorkspaceOwnershipStatus",
    "新增 WorkspaceOwnershipState",
    "新增 WorkspaceOwnershipMarker"
  ],
  "nonGoals": [
    "不做 UI takeover 按钮",
    "不启动 AgentRun"
  ],
  "acceptanceCriteria": [
    "状态包含 none / managed-current / managed-legacy / foreign / corrupted / blocked",
    "可序列化给 Desktop 状态通道",
    "foreign .agentflow 不允许自动写入"
  ],
  "validationHints": [
    "cargo test -p agentflow-agent-manual"
  ],

  "relations": {
    "blockedBy": [],
    "blocks": [],
    "related": [],
    "duplicateOf": null
  },

  "panel": {
    "snapshotId": "panel-snapshot-001",
    "contextPackId": "ctx-001"
  },

  "system": {
    "createdBy": "Spec Agent",
    "createdAt": 1780360000,
    "updatedAt": 1780360000,
    "path": ".agentflow/input/issues/iss-001.json",
    "revision": 1
  }
}
```

---

# 8. Issue Relations

即使 V1 压成 Project -> Issues 两层，也需要保留 issue relations。

路径：

```text
.agentflow/input/relations/issue-relations.json
```

关系类型：

```text
blocked-by
blocks
related
duplicate-of
```

示例：

```json
{
  "version": "input-issue-relations.v1",
  "relations": [
    {
      "fromIssueId": "iss-002",
      "toIssueId": "iss-001",
      "type": "blocked-by"
    },
    {
      "fromIssueId": "iss-003",
      "toIssueId": "iss-001",
      "type": "related"
    }
  ]
}
```

---

# 9. Views

views 是只读派生视图，方便 Desktop 展示。

路径：

```text
.agentflow/input/views/
```

包含：

```text
active.json
blocked.json
by-spec.json
by-project.json
```

注意：

```text
views 是派生结果
不是事实源
可以重建
```

---

# 10. manifest.json

路径：

```text
.agentflow/input/manifest.json
```

职责：

```text
记录 input 当前状态
记录 counts
记录 canonical paths
记录 legacy paths
记录当前 input version
```

示例：

```json
{
  "version": "input-manifest.v1",
  "projectRoot": "/path/to/project",
  "status": "ready",
  "paths": {
    "intake": ".agentflow/input/intake",
    "specs": ".agentflow/input/specs",
    "projects": ".agentflow/input/projects",
    "issues": ".agentflow/input/issues",
    "relations": ".agentflow/input/relations",
    "views": ".agentflow/input/views"
  },
  "summary": {
    "intake": 0,
    "draftSpecs": 0,
    "approvedSpecs": 0,
    "projects": 0,
    "issues": 0
  }
}
```

---

# 11. index.json

路径：

```text
.agentflow/input/index.json
```

职责：

```text
快速索引所有 input fact
```

包含：

```text
specs
projects
issues
relations
active views
```

示例：

```json
{
  "version": "input-index.v1",
  "updatedAt": 1780360000,
  "specs": [],
  "projects": [],
  "issues": []
}
```

---

# 12. 旧路径处理

旧路径：

```text
.agentflow/spec/
.agentflow/goal-tree/
```

V1 处理：

```text
不删除
不强迁移
不再作为新写入路径
workspace-manifest 标记为 legacy
Desktop 逐步切换到 input/
```

新写入只允许：

```text
.agentflow/input/**
```

---

# 13. Rust 模块建议

建议新增 crate：

```text
crates/input/
```

package：

```text
agentflow-input
```

结构建议：

```text
crates/input/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── model.rs
    ├── manager.rs
    ├── storage.rs
    ├── validate.rs
    ├── repair.rs
    ├── spec_gate.rs
    ├── issue.rs
    ├── project.rs
    ├── relations.rs
    └── views.rs
```

---

# 14. Tauri commands

新增只读 / agent-only commands。

V1 Desktop 只读，不允许人类 UI 写 input facts。

建议 commands：

```text
prepare_input_workspace
load_input_status
load_input_manifest
load_input_index
load_input_snapshot
validate_input
```

Agent-only / future commands：

```text
write_intake_result
write_spec_draft
approve_spec
generate_direct_issues_from_spec
generate_project_issues_from_spec
```

注意：

```text
V1 可以先实现 prepare / load / validate。
写入类命令如未授权，可以只定义模型，不暴露给 Desktop human UI。
```

---

# 15. Desktop UI

V1 Desktop 只读展示：

```text
Input status
Intake count
Draft Specs
Approved Specs
Projects
Issues
Blocked issues
High risk issues
```

不提供人类直接创建 / 编辑 input facts 的 UI。

大白话：

> **人类通过会话确认需求，不在 UI 里手动改 input 文件。**

---

# 16. Agent 角色边界

当前四个 Agent 角色：

```text
Spec Agent
Build Agent
Release Agent
Audit Agent
```

009 只涉及：

```text
Spec Agent
```

Spec Agent 允许：

```text
生成 Requirement Intake Result
生成 SPEC Draft Preview
在人类确认后写 Approved SPEC
在 Approved SPEC 后生成 Issues 或 Project -> Issues
```

但 009 仍不启用：

```text
Build Agent
Release Agent
Audit Agent
AgentRun
```

---

# 17. 写入边界

009 允许写：

```text
.agentflow/input/**
```

不允许写：

```text
.agentflow/execute/**
.agentflow/output/evidence/**
.agentflow/output/audit/**
.agentflow/output/release/**
用户源码
远程系统
```

旧路径不再新写：

```text
.agentflow/spec/**
.agentflow/goal-tree/**
```

---

# 18. 非目标

本需求不做：

```text
不启动 AgentRun
不写用户源码
不执行命令
不跑测试
不创建 PR
不发布
不审计执行结果
不连接远程 issue 系统
不保留 Linear 远程依赖
不做 milestone 三层模型
不做复杂 automation / humanGates 字段
```

---

# 19. 开发切片

## Slice 1：Input layout

```text
创建 .agentflow/input/
创建 intake / specs / projects / issues / relations / views
创建 manifest.json / index.json
workspace-manifest paths 加 input
spec / goal-tree 标记 legacy
```

---

## Slice 2：Input models

```text
InputManifest
InputIndex
InputIntakeResult
InputSpec
InputSpecApproval
InputProject
InputIssue
InputIssueRelation
```

---

## Slice 3：Spec Gate

```text
product.md
tech.md
approval.json
spec.json
draft -> approved
```

---

## Slice 4：Issue generation model

```text
direct issue 模式
project -> issues 模式
priority 与 executionRisk 字段
relations
views
```

---

## Slice 5：Validation

```text
所有 issue 必须有 sourceSpecId
所有 issue 必须有 priority
priority 必须是 p0 / p1 / p2 / p3
executionRisk 必须是 low / medium / high
high risk issue 标记为需要人类确认
direct issue projectId = null
project issue projectId 必须存在
project.issueIds 必须能找到对应 issues
relations 不能引用不存在的 issue
```

---

## Slice 6：Desktop / status

```text
Input 状态通道
Browser Preview mock
只读 input snapshot
state/gates 派生 blocked displayStatus
```

---

# 20. 验收标准

```text
- [ ] 新增 docs/requirements/009-input-model-v1.md。
- [ ] 创建 .agentflow/input/ canonical layout。
- [ ] workspace-manifest paths 增加 input。
- [ ] workspace-manifest 将 spec / goal-tree 标记为 legacy。
- [ ] 新写入不再进入 .agentflow/spec/。
- [ ] 新写入不再进入 .agentflow/goal-tree/。
- [ ] input/manifest.json 生成。
- [ ] input/index.json 生成。
- [ ] input/intake/ 目录存在。
- [ ] input/specs/drafts/ 目录存在。
- [ ] input/specs/approved/ 目录存在。
- [ ] input/specs/archive/ 目录存在。
- [ ] input/projects/ 目录存在。
- [ ] input/issues/ 目录存在。
- [ ] input/relations/issue-relations.json 生成。
- [ ] input/relations/dependency-graph.json 生成。
- [ ] input/views/ 目录存在。
- [ ] Spec Gate 使用 product.md + tech.md + approval.json。
- [ ] approval.json 支持 issueGenerationMode = direct。
- [ ] approval.json 支持 issueGenerationMode = project。
- [ ] Direct issue 模型支持 projectId = null。
- [ ] Project issue 模型支持 projectId。
- [ ] Project 模型包含 issueIds。
- [ ] Issue 模型包含 priority。
- [ ] priority 只允许 p0 / p1 / p2 / p3。
- [ ] executionRisk 只允许 low / medium / high。
- [ ] low / medium 不需要人类确认。
- [ ] high 需要人类确认。
- [ ] 不新增 automation / humanGates / prAutomation 字段。
- [ ] issue relations 支持 blocked-by / blocks / related / duplicate-of。
- [ ] 未完成 blockedBy 依赖会在 state/gates/blockers.json 记录阻断，并把相关 issue 派生显示为 blocked。
- [ ] Git provider 预检失败会在 state/gates/blockers.json 记录阻断，并把相关 issue 派生显示为 blocked。
- [ ] Desktop 只读展示 input 状态。
- [ ] 不启动 AgentRun。
- [ ] 不写用户源码。
- [ ] 不执行项目命令。
- [ ] 不创建 PR。
- [ ] 不调用模型。
- [ ] cargo fmt --check 通过。
- [ ] cargo test -p agentflow-input 通过。
- [ ] cargo test -p agentflow-desktop 通过。
- [ ] cargo test 通过。
- [ ] npm --prefix apps/desktop run build 通过。
- [ ] git diff --check 通过。
```

---

# 21. 验证命令

```bash
cargo fmt --check
cargo test -p agentflow-input
cargo test -p agentflow-desktop
cargo test
npm --prefix apps/desktop run build
git diff --check
```

---

# 22. PR 说明要求

PR 描述必须说明：

```text
1. 为什么 spec/ 和 goal-tree/ 合并为 input/。
2. input/ 的职责是什么。
3. Spec Gate 为什么是 product.md + tech.md + approval.json。
4. 两种 issue 模型是什么：direct issues / project -> issues。
5. 为什么 V1 不做 milestone 三层。
6. Issue priority 与 executionRisk 如何定义。
7. 为什么不新增 automation / humanGates 字段。
8. 旧 spec/ 和 goal-tree/ 如何处理。
9. 本次没有启动 AgentRun。
10. 本次没有写用户源码。
11. 验证命令和结果。
```

---

# 23. Codex 执行指令

```md
请执行 009 - Input Model V1。

目标：
把原来的 `.agentflow/spec/` 和 `.agentflow/goal-tree/` 收敛为统一的 `.agentflow/input/` 需求事实源。Input 负责需求沟通、需求说明书、需求施工清单。所有正式 issues 都必须先经过 Spec Gate，即 product.md + tech.md + approval.json。Approved SPEC 后支持两种落地模型：简单需求直接生成 issues；复杂需求生成 project，再由 project 包含多个 issues。

必须遵守：
1. input/ 是新的 canonical 需求事实源。
2. spec/ 和 goal-tree/ 不再作为新写入路径。
3. product.md + tech.md + approval.json 是所有 issue 的共同前置。
4. V1 只支持两层模型：direct issues 或 project -> issues。
5. V1 不做 milestone 三层模型。
6. Issue 模型必须包含 priority。
7. priority 只允许 p0 / p1 / p2 / p3。
8. low / medium 不需要人类确认。
9. high 需要人类确认。
10. priority 由 Spec Agent 生成 issue 时判断。
11. 不新增 automation / humanGates / prAutomation 字段。
12. 不启动 AgentRun。
13. 不写用户源码。
14. 不执行项目命令。
15. 不创建 PR。
16. 不调用模型。

实现范围：
- 新增 docs/requirements/009-input-model-v1.md。
- 新增 crates/input package agentflow-input。
- 新增 .agentflow/input/ layout prepare / validate。
- 新增 input manifest / index。
- 新增 intake / specs / projects / issues / relations / views 目录。
- 新增 Spec Gate 模型：product.md / tech.md / approval.json / spec.json。
- 新增 InputProject 模型。
- 新增 InputIssue 模型。
- 新增 priority 与 executionRisk 字段。
- 新增 relations 模型。
- 新增 views 派生视图。
- workspace-manifest 增加 input path，并将 spec / goalTree 标记为 legacy。
- Desktop status 增加 Input 状态，只读展示。
- Browser Preview mock 更新。
- 更新 verification。

验证命令：
- cargo fmt --check
- cargo test -p agentflow-input
- cargo test -p agentflow-desktop
- cargo test
- npm --prefix apps/desktop run build
- git diff --check
```

---

# 24. 完成定义

本需求完成后，AgentFlow 的输入层是：

```text
input/
= 需求实时源头
```

内部流程是：

```text
输入准备
→ 需求沟通与过滤
→ 需求说明书
→ 需求施工清单
```

最终模型是：

```text
简单需求：
Approved SPEC
→ issues

复杂需求：
Approved SPEC
→ project
→ issues
```

Issue 优先级是：

```text
priority = p0 | p1 | p2 | p3
```

执行风险是：

```text
executionRisk = low | medium | high

low / medium
= 不需要人类确认

high
= 需要人类确认
```

最终一句话：

> **Input Model V1 把需求输入、SPEC 确认和施工清单统一到 input/；所有 issues 都来自 Approved SPEC；简单需求直接 issues，复杂需求 project 包含 issues；Issue 使用 priority 决定顺序，executionRisk 控制执行确认。**
