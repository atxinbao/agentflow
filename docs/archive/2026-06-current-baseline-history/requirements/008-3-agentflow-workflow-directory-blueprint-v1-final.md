# 008.3 - AgentFlow Workflow Directory Blueprint V1

创建日期：2026-06-03
执行者：Codex
状态：待开发
版本：final-draft

---

## 用户目标

当前 AgentFlow 已经完成：

```text
001 Project Workspace Manager
002 Graph
003 Project File Reader
004-006 Legacy Cleanup
007 Goal Tree V1
007.1 Goal Tree Agent-only Boundary Fix
008 Agent Working Manual Bootstrap V1
008.1 Agent Working Manual Health Polish
008.2 Requirement Intake Filter Skill V1
```

现在需要把 `.agentflow/` 的整体目录结构定下来。

当前问题是：

```text
.agentflow/ 已经开始承载 Agent 入口、Agent 工作手册、skills、Graph、Goal Tree、状态、输出等内容，
但整体目录还没有形成完整、清晰、可持续扩展的 Agent Workflow Control Plane。
```

本需求目标是：

```text
把 .agentflow/ 从“本地运行目录”升级成 AgentFlow 的本地 Agent 工作流控制面。
```

大白话：

> `.agentflow/` 不能只是随便放文件的地方。
> 它应该清楚区分：规则在哪里、需求在哪里、目标树在哪里、Graph 现场在哪里、TDD 工作手册在哪里、发布手册在哪里、审计手册在哪里、执行状态在哪里、证据在哪里。
> Agent 后续所有动作都必须落在这个结构里，不能自由写、不能混着写、不能绕过流程。

---

## 一句话定义

> **008.3 AgentFlow Workflow Directory Blueprint V1 负责重新定义 `.agentflow/` 目录结构，把 `define/` 收敛为工作手册区，只包含 `agent/spec/tdd/release/audit`；把 SPEC 产物、Goal Tree、Graph、执行、输出、状态从 `define/` 中拆出来；将项目根 Agent 入口统一为 `AGENTS.md`；加入 Root Agent Entry Shadow Guard；为后续 OpenSpec、TDD、Release、Audit、AgentRun 建立清晰的本地工作流骨架。**

---

# 1. 核心原则

## 1.1 项目根 Agent 入口统一为 `AGENTS.md`

AgentFlow 后续的项目根入口统一使用：

```text
<project-root>/AGENTS.md
```

`AGENTS.md` 是所有 Agent 进入项目后第一眼应该读取的入口文件。

它的职责很薄：

```text
告诉 Agent：这个项目由 AgentFlow 管理
告诉 Agent：必须读取 .agentflow/define/agent/Agentflow.md
告诉 Agent：必须读取 skills-lock.json
告诉 Agent：必须遵守 AgentFlow 规则和边界
```

真正完整规则仍然放在：

```text
.agentflow/define/agent/Agentflow.md
```

---

## 1.2 `AGENT.MD` 变成 legacy compatibility

PR #13 已经实现过 `AGENT.MD` 入口。

从 008.3 开始：

```text
AGENTS.md = canonical root agent entry
AGENT.MD = legacy compatibility entry
```

本需求不强制删除已有 `AGENT.MD`，也不强制清理历史备份。

但从 008.3 开始，Project Workspace prepare / repair 的目标应该变成：

```text
优先保证 AGENTS.md 是 AgentFlow managed entry
保留 AGENT.MD 作为 legacy compatibility entry
workspace-manifest.json 明确 canonicalAgentEntry = AGENTS.md
```

如果项目里已经存在 `AGENT.MD`：

```text
不直接删除
不把它作为 canonical
可保留为 legacy compatibility
如需重写，必须备份
```

如果项目里没有 `AGENTS.md`：

```text
创建 AGENTS.md
```

如果项目里已有 `AGENTS.md`：

```text
备份原 AGENTS.md
重写为 AgentFlow managed entry
```

备份路径建议继续使用：

```text
.agentflow/output/backup/agent-md/
```

或者后续统一改成：

```text
.agentflow/output/backup/agent-entry/
```

V1 可以保持已有 backup 目录不变，避免迁移过重。

---

## 1.3 Root Agent Entry Shadow Guard

只创建 `AGENTS.md` 还不够。

一些外部 Agent / 编辑器可能会优先读取其他根目录规则文件，例如：

```text
.rules
.cursorrules
.windsurfrules
.clinerules
.github/copilot-instructions.md
AGENT.md
CLAUDE.md
GEMINI.md
```

这些文件可能会“遮挡” `AGENTS.md`，导致外部 Agent 没有首先读取 AgentFlow 规则。

因此 V1 新增：

```text
Root Agent Entry Shadow Guard
```

职责：

```text
检查项目根目录中是否存在可能优先于 AGENTS.md 的 Agent 规则文件
如果存在，记录 warning
不默认重写这些 shadow files
不默认删除这些 shadow files
不阻塞 AgentFlow 内部流程
但必须让用户 / Agent 可见
```

Shadow Guard 检查对象：

```text
.rules
.cursorrules
.windsurfrules
.clinerules
.github/copilot-instructions.md
AGENT.md
CLAUDE.md
GEMINI.md
```

注意：

```text
AGENT.MD 是 AgentFlow legacy compatibility entry，不等同于 AGENT.md。
AGENT.md 是外部生态常见入口，应该作为 shadow candidate 检查。
```

如果发现 shadow file，warning 示例：

```text
Agent entry shadow detected: .rules exists and may be read before AGENTS.md by some tools. AgentFlow uses AGENTS.md as canonical entry.
```

如果发现 `AGENT.md`：

```text
Agent entry shadow detected: AGENT.md may be read before AGENTS.md by some tools. AgentFlow uses AGENTS.md as canonical entry.
```

V1 不自动重写 shadow files，原因：

```text
.rules / CLAUDE.md / GEMINI.md 等可能是用户为其他工具准备的规则
直接接管会破坏用户预期
```

---

## 1.4 `define/` 只放工作手册和规则

新的原则：

```text
define/
= Agent 要读的规则、手册、模板、技能定义
```

`define/` 不放真实业务事实。

`define/` 只包含：

```text
agent/
spec/
tdd/
release/
audit/
```

对应五本工作手册：

```text
Agentflow.md
SPEC.md
TDD.md
RELEASE.md
AUDIT.md
```

大白话：

```text
define = 工作手册区
```

---

## 1.5 Goal Tree 不属于 `define/`

Goal Tree 是 approved SPEC 派生出来的事实源。

所以 Goal Tree 的 canonical path 应该是：

```text
.agentflow/goal-tree/
```

而不是：

```text
.agentflow/define/goal-tree/
.agentflow/define/goals/
.agentflow/define/milestones/
.agentflow/define/issues/
```

原因：

```text
define = 规则 / 手册
goal-tree = 事实 / 产物
```

所以：

```text
Goal Tree 必须从 define 中拿出来。
```

---

## 1.6 SPEC 也要区分“手册”和“产物”

有两种 SPEC。

### SPEC 工作手册

路径：

```text
.agentflow/define/spec/SPEC.md
```

职责：

```text
告诉 Agent 如何写 SPEC
SPEC 必须包含什么
什么时候可以进入 approval
验收标准怎么写
OpenSpec Draft Preview 如何生成
```

### SPEC 真实产物

路径：

```text
.agentflow/spec/
```

职责：

```text
保存真正生成出来的需求规格、草案、审批记录和索引
```

大白话：

```text
define/spec/SPEC.md
= 写规格的说明书

spec/changes/**
= 真正写出来的规格
```

---

## 1.7 Superpowers 只落地为 TDD 工作手册

之前容易把 Superpowers 说成：

```text
质量规则 / TDD 工作手册
```

现在要收窄：

```text
Superpowers = TDD 工作手册
```

因为：

```text
需求质量 / 验收标准已经由 SPEC.md 管
需求过滤已经由 requirement-intake-filter 管
边界检查已经由 boundary-check 管
```

所以 TDD 只负责：

```text
写代码前，测试怎么先行
怎么从 SPEC / Issue 推导测试
怎么红绿重构
怎么防回归
```

路径：

```text
.agentflow/define/tdd/TDD.md
```

---

## 1.8 gstack 简化为 RELEASE.md

不再使用复杂名：

```text
gstack
delivery
deploy
```

统一落地为：

```text
release/
RELEASE.md
```

原因：

```text
release 比 delivery 更好理解
release 比 deploy 更宽
```

`RELEASE.md` 未来负责：

```text
提交
PR
Review
Changelog
Release note
部署
回滚
发布证据
```

路径：

```text
.agentflow/define/release/RELEASE.md
```

---

## 1.9 Audit V1 直接进入 define/audit/

代码审计已经是顶层 Agent 角色之一，所以 V1 直接新增：

```text
.agentflow/define/audit/
```

对应手册：

```text
AUDIT.md
```

原因：

```text
既然顶层已经确定有“代码审计 Agent”，那它应该有独立工作手册。
```

V1 不实现审计执行，只写审计工作手册骨架。

必须区分：

```text
define/audit/AUDIT.md
= 审计工作手册，V1 必须有

output/audit/
= 审计报告输出，V1 只建目录，不生成报告
```

---

## 1.10 `.agentflow/inspect/` 改成 `.agentflow/graph/`

不要引入新的 `inspect` 概念。

当前项目里“现场资源索引”的核心名字已经是：

```text
Graph
```

所以 canonical path 应该是：

```text
.agentflow/graph/
```

而不是：

```text
.agentflow/inspect/
```

大白话：

> 项目现场地图就叫 Graph，不要再多套一层 inspect。

---

# 2. 顶层 5 个 Agent 角色

AgentFlow 顶层只定义 5 个 Agent 角色：

```text
1. 需求接待 Agent
2. 规格计划 Agent
3. 实现执行 Agent
4. 发布交付 Agent
5. 代码审计 Agent
```

英文对应：

```text
1. Intake Agent
2. Spec Planning Agent
3. Build Agent
4. Release Agent
5. Audit Agent
```

---

## 2.1 需求接待 Agent / Intake Agent

职责：

```text
接住人类输入
判断请求类型
做需求过滤
提出澄清问题
判断是否能进入 SPEC
阻断越界请求
```

使用：

```text
request-triage
requirement-intake-filter
boundary-check
validation
```

输出：

```text
Requirement Intake Result
```

状态：

```text
ready-for-openspec
needs-clarification
answer-only
blocked-by-boundary
defer
```

禁止：

```text
不能写 SPEC 文件
不能写 Goal Tree
不能写源码
不能执行命令
不能启动 AgentRun
```

当前状态：

```text
enabled
```

---

## 2.2 规格计划 Agent / Spec Planning Agent

这是合并后的角色：

```text
规格整理 Agent
+
计划生成 Agent
```

职责分两段。

### 第一段：SPEC

```text
根据 ready-for-openspec 的 Intake Result
生成 SPEC Draft Preview
整理目标
整理范围
整理非目标
整理验收标准
整理任务草案
整理风险和开放问题
等待人类确认
```

### 第二段：Plan / Goal Tree

```text
从 Approved SPEC 生成 Goal
从 Approved SPEC 生成 Milestone
从 Approved SPEC 生成 Issue
写入 Goal Tree
记录 SPEC source 追溯
```

使用：

```text
SPEC.md
openspec-authoring
goal-tree-materialization
boundary-check
validation
```

输出：

```text
SPEC Draft Preview
Approved SPEC
Goal Tree
Goal / Milestone / Issue
```

禁止：

```text
不能跳过 Intake
不能绕过人类确认
不能从聊天直接生成 Goal Tree
不能从未批准 SPEC 生成 Goal Tree
不能执行 Issue
不能写代码
不能跑测试
```

当前状态：

```text
planned
```

---

## 2.3 实现执行 Agent / Build Agent

职责：

```text
未来真正写代码、跑测试、实现 Issue
但必须在 TDD 规则下执行
```

使用：

```text
TDD.md
test-first
red-green-refactor
spec-to-test
regression-guard
boundary-check
validation
```

未来输出：

```text
AgentRun
test plan
implementation patch
validation result
evidence
```

当前状态：

```text
not authorized yet
```

禁止：

```text
不能在没有 Approved SPEC 时执行
不能在没有 Goal Tree Issue 时执行
不能绕过 TDD
不能直接改代码
不能删除测试来通过验证
不能降低验收标准
```

---

## 2.4 发布交付 Agent / Release Agent

职责：

```text
未来处理提交、PR、Review、发布、回滚
```

使用：

```text
RELEASE.md
release-lock
release skills
boundary-check
validation
```

未来输出：

```text
PR draft
release note
delivery evidence
deploy record
rollback plan
```

当前状态：

```text
not authorized yet
```

禁止：

```text
不能在 Build / Validation 未完成前发布
不能绕过 Review
不能直接创建远程 PR
不能直接部署
```

---

## 2.5 代码审计 Agent / Audit Agent

职责：

```text
交付后做代码审计和风险复盘
```

它不是实现者，也不是发布者。

它做：

```text
审查本次变更是否符合 SPEC
审查实现是否越界
审查是否破坏架构边界
审查是否有安全 / 权限 / 路径 / 数据写入风险
审查测试是否覆盖关键验收标准
审查是否有 legacy 代码回流
审查是否有未授权执行 / 写入 / 模型调用
审查 evidence 是否完整
输出 audit report
```

未来使用：

```text
AUDIT.md
audit-lock
audit skills
boundary-check
validation
Graph impact
TDD evidence
Release evidence
```

当前状态：

```text
not authorized yet
```

V1 新增 `define/audit/` 和 `AUDIT.md`，但不执行审计。

---

# 3. 新的 Agent 主流程

调整后的主流程是：

```text
Conversation
  ↓
需求接待 Agent
  ↓
Requirement Intake Result
  ↓
规格计划 Agent
  ↓
SPEC Draft Preview
  ↓
Human Confirmation
  ↓
Approved SPEC
  ↓
Goal Tree
  ↓
实现执行 Agent
  ↓
TDD / AgentRun / Evidence
  ↓
发布交付 Agent
  ↓
PR / Release
  ↓
代码审计 Agent
  ↓
Audit Report
```

当前只开放到：

```text
需求接待 Agent
```

下一步才进入：

```text
规格计划 Agent
```

Build / Release / Audit 均为：

```text
not authorized yet
```

---

# 4. 新的 `.agentflow/` 总目录结构

目标结构：

```text
.agentflow/
├── workspace.yaml
├── config.yaml
├── workspace-manifest.json
│
├── define/
│   ├── agent/
│   │   ├── Agentflow.md
│   │   ├── skills-lock.json
│   │   ├── skills/
│   │   └── state/
│   │
│   ├── spec/
│   │   ├── SPEC.md
│   │   ├── spec-lock.json
│   │   ├── skills/
│   │   └── templates/
│   │
│   ├── tdd/
│   │   ├── TDD.md
│   │   ├── tdd-lock.json
│   │   ├── skills/
│   │   └── templates/
│   │
│   ├── release/
│   │   ├── RELEASE.md
│   │   ├── release-lock.json
│   │   ├── skills/
│   │   └── templates/
│   │
│   └── audit/
│       ├── AUDIT.md
│       ├── audit-lock.json
│       ├── skills/
│       └── templates/
│
├── spec/
│   ├── changes/
│   ├── approvals/
│   ├── drafts/
│   └── index.json
│
├── goal-tree/
│   ├── goal-tree.json
│   ├── goals/
│   ├── milestones/
│   ├── issues/
│   └── materialization/
│
├── graph/
│   ├── manifest.json
│   ├── symbols.json
│   ├── relations.json
│   ├── context-packs/
│   ├── search/
│   ├── files/
│   └── index/
│
├── execute/
│   ├── runs/
│   ├── leases/
│   └── commands/
│
├── output/
│   ├── evidence/
│   ├── audit/
│   ├── backup/
│   ├── logs/
│   ├── cache/
│   └── tmp/
│
└── state/
    ├── health/
    ├── locks/
    ├── sessions/
    └── indexes/
```

---

# 5. `define/` 结构

`define/` 只放工作手册：

```text
.agentflow/define/
├── agent/
├── spec/
├── tdd/
├── release/
└── audit/
```

不放：

```text
Goal Tree
SPEC changes
AgentRun
Evidence
Audit output
Graph data
```

---

# 6. `.agentflow/spec/`

真实 SPEC 产物放这里：

```text
.agentflow/spec/
├── changes/
│   └── <change-id>/
│       ├── proposal.md
│       ├── product.md
│       ├── tech.md
│       ├── tasks.md
│       └── specs/
├── approvals/
│   └── <change-id>.json
├── drafts/
│   └── <draft-id>.json
└── index.json
```

职责：

```text
OpenSpec Draft
Approved SPEC
SPEC approvals
SPEC index
```

V1 只建目录，不写真实 SPEC。

---

# 7. `.agentflow/goal-tree/`

Goal Tree 新 canonical path：

```text
.agentflow/goal-tree/
├── goal-tree.json
├── goals/
├── milestones/
├── issues/
└── materialization/
```

职责：

```text
Approved SPEC 派生出来的目标树事实源
```

注意：

```text
Goal Tree 不属于 define
```

V1 只建目录，不迁移旧数据，不写事实。

---

# 8. `.agentflow/graph/`

Graph 新 canonical path：

```text
.agentflow/graph/
├── manifest.json
├── symbols.json
├── relations.json
├── context-packs/
├── search/
├── files/
└── index/
```

职责：

```text
项目现场图谱
文件索引
符号索引
关系索引
搜索索引
Context Pack
Graph manifest
```

注意：

```text
.agentflow/graph/
= 当前项目现场

.agentflow/output/
= 某次运行后的结果 / 证据 / 日志
```

V1 不强迁移旧 Graph 数据。

如果当前 Graph 仍在：

```text
.agentflow/output/graph/
```

本阶段保留兼容。

---

# 9. `.agentflow/execute/`

未来 AgentRun 使用：

```text
.agentflow/execute/
├── runs/
├── leases/
└── commands/
```

当前只建目录，不启用执行。

---

# 10. `.agentflow/output/`

运行输出和证据：

```text
.agentflow/output/
├── evidence/
├── audit/
├── backup/
├── logs/
├── cache/
└── tmp/
```

职责：

```text
Evidence
Audit Report
Backups
Logs
Cache
Tmp
```

注意：

```text
Graph canonical path 是 .agentflow/graph/
output/graph 只作为兼容路径保留
```

---

# 11. `.agentflow/state/`

系统当前状态：

```text
.agentflow/state/
├── health/
├── locks/
├── sessions/
└── indexes/
```

职责：

```text
健康状态
全局锁
会话状态
索引状态
```

---

# 12. workspace-manifest.json

新增：

```text
.agentflow/workspace-manifest.json
```

示例：

```json
{
  "version": "agentflow-workspace-manifest.v1",
  "layoutVersion": "agentflow-layout.v1",
  "projectRoot": "/path/to/project",
  "rootEntries": {
    "canonicalAgentEntry": "AGENTS.md",
    "legacyAgentEntry": "AGENT.MD",
    "shadowChecked": [
      ".rules",
      ".cursorrules",
      ".windsurfrules",
      ".clinerules",
      ".github/copilot-instructions.md",
      "AGENT.md",
      "CLAUDE.md",
      "GEMINI.md"
    ]
  },
  "activeLayers": [
    "workspace",
    "agent-manual",
    "graph",
    "project-file-reader",
    "requirement-intake"
  ],
  "plannedLayers": [
    "spec",
    "goal-tree",
    "tdd",
    "execution",
    "release",
    "audit"
  ],
  "paths": {
    "agentEntry": "AGENTS.md",
    "legacyAgentEntry": "AGENT.MD",
    "defineAgent": ".agentflow/define/agent",
    "defineSpec": ".agentflow/define/spec",
    "defineTdd": ".agentflow/define/tdd",
    "defineRelease": ".agentflow/define/release",
    "defineAudit": ".agentflow/define/audit",
    "spec": ".agentflow/spec",
    "goalTree": ".agentflow/goal-tree",
    "graph": ".agentflow/graph",
    "execute": ".agentflow/execute",
    "output": ".agentflow/output",
    "state": ".agentflow/state"
  },
  "compat": {
    "legacyGraphOutput": ".agentflow/output/graph",
    "legacyGoalTreeDefine": ".agentflow/define",
    "legacyAgentEntry": "AGENT.MD",
    "agentflowSkills": ".agentflow/define/agent/skills",
    "zedProjectSkills": ".agents/skills",
    "skillsExport": "planned"
  },
  "warnings": []
}
```

作用：

```text
Agent 知道目录布局
系统知道哪些层已启用
未来迁移有依据
明确 AGENTS.md 是 canonical agent entry
保留 AGENT.MD legacy compatibility
记录 shadow guard 检查对象
预留未来 Zed project skills export，但 V1 不导出
```

---

# 13. 008.3 做什么

本需求只做目录蓝图和手册骨架。

做：

```text
1. 新增 workspace-manifest.json
2. 将项目根 canonical Agent entry 设为 AGENTS.md
3. 创建 / 修复 AGENTS.md
4. 保留 AGENT.MD legacy compatibility，不强删除
5. 检查 Root Agent Entry Shadow Guard 并记录 warning
6. 创建 define/spec/SPEC.md
7. 创建 define/tdd/TDD.md
8. 创建 define/release/RELEASE.md
9. 创建 define/audit/AUDIT.md
10. 创建 spec/ 目录
11. 创建 goal-tree/ 顶层目录
12. 创建 graph/ 顶层目录
13. 创建 execute/ 目录
14. 创建 output/ 标准目录
15. 创建 state/ 目录
16. Agentflow.md 增加 5 个 Agent 角色章节
17. Project Workspace prepare 接入 workflow layout prepare / repair
18. validation 检查目录布局
```

---

# 14. 008.3 不做什么

不做：

```text
不实现 OpenSpec Authoring
不写 SPEC change
不生成 Goal Tree fact
不迁移已有 Goal Tree 数据
不迁移已有 Graph 数据
不启动 AgentRun
不实现 TDD
不执行测试命令
不实现 Release
不执行代码审计
不导出 .agents/skills
不创建 PR
不调用模型
不写用户源码
不写旧 .agentflow paths
不恢复 legacy workflow
```

---

# 15. 关于 AGENTS.md 迁移

因为 PR #13 已经使用过：

```text
AGENT.MD
```

008.3 不做破坏性迁移。

本阶段策略：

```text
AGENTS.md 是新 canonical entry
AGENT.MD 是 legacy compatibility entry
不删除 AGENT.MD
不要求用户手动清理 AGENT.MD
workspace-manifest.json 记录两者关系
```

建议 prepare 行为：

```text
如果 AGENTS.md 缺失：
  创建 AGENTS.md

如果 AGENTS.md 已存在：
  备份后重写为 AgentFlow managed entry

如果 AGENT.MD 存在：
  不删除
  可保留为 legacy compatibility

如果 AGENT.MD 缺失：
  V1 不强制创建 AGENT.MD
```

如项目短期仍需要兼容旧 AgentFlow 逻辑，也可以让 `AGENT.MD` 保留一个薄跳转入口，但 canonical 仍必须是 `AGENTS.md`。

---

# 16. 关于 shadow files

V1 只检查，不强接管。

检查对象：

```text
.rules
.cursorrules
.windsurfrules
.clinerules
.github/copilot-instructions.md
AGENT.md
CLAUDE.md
GEMINI.md
```

处理规则：

```text
如果不存在：
  无 warning

如果存在且是 AgentFlow managed：
  无 warning

如果存在且不是 AgentFlow managed：
  warnings 记录 shadow detected
  status 可以 degraded，但不 blocked
```

不做：

```text
不重写 .rules
不重写 CLAUDE.md
不重写 GEMINI.md
不重写 AGENT.md
不删除任何 shadow file
```

---

# 17. 关于 Goal Tree 迁移

当前已有 Goal Tree 可能还使用旧路径：

```text
.agentflow/define/goal-tree.json
.agentflow/define/goals/
.agentflow/define/milestones/
.agentflow/define/issues/
```

008.3 不做强迁移。

本阶段只做：

```text
创建新规范路径 .agentflow/goal-tree/
更新 workspace-manifest
文档标记新路径为 canonical
保留旧路径兼容
```

后续如有必要再做：

```text
Goal Tree Path Migration
```

---

# 18. 关于 Graph 迁移

当前 Graph 可能还使用旧路径：

```text
.agentflow/output/graph/
```

008.3 不做强迁移。

本阶段只做：

```text
创建新规范路径 .agentflow/graph/
更新 workspace-manifest
文档标记 .agentflow/graph/ 为 canonical
保留 .agentflow/output/graph/ 兼容
```

后续如有必要再做：

```text
Graph Path Migration
```

---

# 19. 写入边界

本轮允许写：

```text
<project-root>/AGENTS.md
.agentflow/workspace-manifest.json
.agentflow/define/spec/**
.agentflow/define/tdd/**
.agentflow/define/release/**
.agentflow/define/audit/**
.agentflow/spec/**
.agentflow/goal-tree/**
.agentflow/graph/**
.agentflow/execute/**
.agentflow/output/**
.agentflow/state/**
```

可选兼容写入：

```text
<project-root>/AGENT.MD
```

但只有在明确需要 legacy compatibility 时才写。

本轮只允许写：

```text
目录
空占位文件
工作手册模板
manifest
health state
```

不允许写业务事实：

```text
SPEC change
Approved SPEC
Goal
Milestone
Issue
AgentRun
Evidence
Audit report
Release record
```

不允许写或重写：

```text
.rules
.cursorrules
.windsurfrules
.clinerules
.github/copilot-instructions.md
AGENT.md
CLAUDE.md
GEMINI.md
```

这些只做 shadow detection。

---

# 20. Rust / Project Workspace 实现建议

可以在现有 workspace prepare 中新增一个布局准备模块。

建议放在：

```text
apps/desktop/src-tauri/src/project_workspace/layout.rs
```

或者如果更适合复用，也可以放在：

```text
crates/agent-manual/src/layout.rs
```

V1 推荐：

```text
project_workspace/layout.rs
```

原因：

```text
目录布局属于 Project Workspace prepare 管线
不是单纯 Agent Manual
```

建议函数：

```rust
prepare_agentflow_workspace_layout(project_root: impl AsRef<Path>) -> Result<WorkspaceLayoutStatus>
```

职责：

```text
创建标准目录
写 workspace-manifest.json
写 SPEC.md / TDD.md / RELEASE.md / AUDIT.md
创建 / 修复 AGENTS.md
检查 shadow files
返回 layout status
```

---

# 21. Agentflow.md 更新

在：

```text
.agentflow/define/agent/Agentflow.md
```

新增章节：

```md
## Agent Roles

### 1. Intake Agent / 需求接待 Agent
...

### 2. Spec Planning Agent / 规格计划 Agent
...

### 3. Build Agent / 实现执行 Agent
...

### 4. Release Agent / 发布交付 Agent
...

### 5. Audit Agent / 代码审计 Agent
...
```

其中 Build / Release / Audit 必须明确：

```text
not authorized yet
```

同时 Required Reading Order 应调整为：

```text
1. `<project-root>/AGENTS.md`
2. `.agentflow/define/agent/Agentflow.md`
3. `.agentflow/define/agent/skills-lock.json`
4. All required skills listed in `skills-lock.json`
```

---

# 22. AGENTS.md 模板

建议模板：

```md
# AGENTS.md

<!-- AGENTFLOW:MANAGED version=agent-entry.v2 -->

This project is managed by AgentFlow.

Every Agent MUST read and follow:

1. `.agentflow/define/agent/Agentflow.md`
2. `.agentflow/define/agent/skills-lock.json`
3. All skills referenced by `skills-lock.json`

## Hard Rules

- Do not write source code unless AgentFlow rules explicitly allow it.
- Do not execute project commands unless AgentFlow rules explicitly allow it.
- Before producing an OpenSpec Draft, every Agent MUST run the requirement-intake-filter skill.
- Do not create or edit Goal Tree directly.
- Do not bypass SPEC.
- Do not create PRs, issues, or remote objects unless explicitly authorized.
- Human conversation is for confirmation and feedback, not direct Goal Tree editing.

## Current Flow

Conversation with human
→ Request triage
→ Requirement intake filter
→ SPEC Draft Preview
→ Human confirmation
→ Approved SPEC
→ Goal Tree materialization
→ Future AgentRun

If any rule conflicts, AgentFlow rules win.

<!-- AGENTFLOW:END -->
```

---

# 23. 手册模板内容要求

## 23.1 SPEC.md

路径：

```text
.agentflow/define/spec/SPEC.md
```

必须说明：

```text
SPEC 是需求 / 验收工作手册
Requirement Intake Result 是 SPEC 前置条件
只有 ready-for-openspec 才能生成 SPEC Draft Preview
人类确认前不能写 SPEC 事实源
Approved SPEC 才能生成 Goal Tree
```

V1 只写手册，不写真实 SPEC。

---

## 23.2 TDD.md

路径：

```text
.agentflow/define/tdd/TDD.md
```

必须说明：

```text
TDD 是测试先行工作手册
质量标准来自 SPEC 的验收标准
TDD 不重新定义质量
Build Agent 当前 not authorized yet
未来实现前必须先做 spec-to-test
```

V1 只写手册，不执行 TDD。

---

## 23.3 RELEASE.md

路径：

```text
.agentflow/define/release/RELEASE.md
```

必须说明：

```text
Release 是发布交付工作手册
Release Agent 当前 not authorized yet
未来负责 commit / PR / review / release / rollback
当前不能创建 PR
当前不能部署
```

V1 只写手册，不执行发布。

---

## 23.4 AUDIT.md

路径：

```text
.agentflow/define/audit/AUDIT.md
```

必须说明：

```text
Audit 是代码审计工作手册
Audit Agent 当前 not authorized yet
未来负责 SPEC 对齐、边界检查、测试覆盖、legacy 回流、未授权执行 / 写入 / 模型调用检查
当前不生成 audit report
```

V1 只写手册，不执行审计。

---

# 24. 验收标准

```text
- [ ] 新增 docs/requirements/008-3-agentflow-workflow-directory-blueprint-v1.md。
- [ ] Project Workspace prepare 创建 / 修复 AGENTS.md。
- [ ] AGENTS.md 是新的 canonical root agent entry。
- [ ] workspace-manifest.json 记录 canonicalAgentEntry = AGENTS.md。
- [ ] workspace-manifest.json 记录 legacyAgentEntry = AGENT.MD。
- [ ] Project Workspace prepare 检查 Root Agent Entry Shadow Guard。
- [ ] shadow files 存在时记录 warning，不自动重写。
- [ ] prepare Project Workspace 时创建 workspace-manifest.json。
- [ ] workspace-manifest.json 包含 defineAgent / defineSpec / defineTdd / defineRelease / defineAudit。
- [ ] workspace-manifest.json 包含 spec / goalTree / graph / execute / output / state。
- [ ] workspace-manifest.json 包含 rootEntries.shadowChecked。
- [ ] workspace-manifest.json 包含 compat.zedProjectSkills = .agents/skills。
- [ ] workspace-manifest.json 包含 legacyGraphOutput compat。
- [ ] workspace-manifest.json 包含 legacyGoalTreeDefine compat。
- [ ] prepare 时创建 define/spec/SPEC.md。
- [ ] prepare 时创建 define/tdd/TDD.md。
- [ ] prepare 时创建 define/release/RELEASE.md。
- [ ] prepare 时创建 define/audit/AUDIT.md。
- [ ] prepare 时创建顶层 spec/。
- [ ] prepare 时创建顶层 goal-tree/。
- [ ] prepare 时创建顶层 graph/。
- [ ] prepare 时不创建 inspect/。
- [ ] prepare 时创建 execute/。
- [ ] prepare 时创建 output/evidence。
- [ ] prepare 时创建 output/audit。
- [ ] prepare 时创建 state/health。
- [ ] define/ 只包含 agent/spec/tdd/release/audit 作为工作手册区。
- [ ] Goal Tree 新 canonical path 为 .agentflow/goal-tree/。
- [ ] Graph 新 canonical path 为 .agentflow/graph/。
- [ ] .agentflow/output/graph/ 仅作为兼容路径保留。
- [ ] 不强迁移旧 Goal Tree 数据。
- [ ] 不强迁移旧 Graph 数据。
- [ ] Agentflow.md Required Reading Order 指向 AGENTS.md。
- [ ] Agentflow.md 增加 5 个角色章节。
- [ ] 5 个角色为：需求接待、规格计划、实现执行、发布交付、代码审计。
- [ ] Build / Release / Audit 明确 not authorized yet。
- [ ] SPEC.md / TDD.md / RELEASE.md / AUDIT.md 均为手册骨架，不启用执行能力。
- [ ] 不实现 OpenSpec。
- [ ] 不写 SPEC change。
- [ ] 不写 Goal Tree fact。
- [ ] 不启动 AgentRun。
- [ ] 不执行项目命令。
- [ ] 不调用模型。
- [ ] 不写用户源码。
- [ ] 不导出 .agents/skills。
- [ ] cargo fmt --check 通过。
- [ ] cargo test -p agentflow-agent-manual 通过。
- [ ] cargo test -p agentflow-desktop 通过。
- [ ] cargo test 通过。
- [ ] npm --prefix apps/desktop run build 通过。
- [ ] git diff --check 通过。
```

---

# 25. 验证命令

```bash
cargo fmt --check
cargo test -p agentflow-agent-manual
cargo test -p agentflow-desktop
cargo test
npm --prefix apps/desktop run build
git diff --check
```

---

# 26. PR 说明要求

PR 描述必须说明：

```text
1. 为什么 root agent entry 改为 AGENTS.md。
2. AGENT.MD 如何作为 legacy compatibility 处理。
3. shadow files 检查了哪些。
4. shadow files 是否会自动重写：必须说明不会。
5. define/ 为什么只保留 agent/spec/tdd/release/audit。
6. audit 为什么 V1 必须有手册，但不启用执行能力。
7. Goal Tree 为什么从 define 中拿出来。
8. Graph 为什么从 inspect 命名改为 graph。
9. 新 canonical Goal Tree 路径是什么。
10. 新 canonical Graph 路径是什么。
11. 是否迁移旧数据：必须说明不迁移，只保留兼容。
12. 5 个 Agent 角色是什么。
13. Build / Release / Audit 当前是否启用：必须说明未授权。
14. 本次只写目录和手册，不写业务事实。
15. 本次没有 OpenSpec / Goal Tree / AgentRun 新功能。
16. 验证命令和结果。
```

---

# 27. Codex 执行指令

```md
请执行 008.3 - AgentFlow Workflow Directory Blueprint V1。

目标：
重新规划 `.agentflow/` 目录结构，把 `define/` 收敛成工作手册区，只包含 agent/spec/tdd/release/audit；把 Goal Tree 从 define 中拿出来，作为 `.agentflow/goal-tree/` 顶层事实源；把 inspect 命名改为 `.agentflow/graph/`，作为项目现场图谱 canonical path；将项目根 Agent 入口统一调整为 `AGENTS.md`；加入 Root Agent Entry Shadow Guard；新增 workspace-manifest.json；在 Agentflow.md 中定义 5 个顶层 Agent 角色。

必须遵守：
1. root canonical Agent entry 是 `AGENTS.md`。
2. `AGENT.MD` 只作为 legacy compatibility。
3. define/ 只放工作手册、规则、模板。
4. define/ 只包含 agent/spec/tdd/release/audit。
5. Goal Tree 新 canonical 路径是 `.agentflow/goal-tree/`。
6. Graph 新 canonical 路径是 `.agentflow/graph/`。
7. 不创建 `.agentflow/inspect/`。
8. Root Agent Entry Shadow Guard 只记录 warning，不自动重写 shadow files。
9. 不导出 `.agents/skills`，只在 manifest 中预留 compat。
10. 本次不强迁移旧 Goal Tree 数据。
11. 本次不强迁移旧 Graph 数据。
12. 本次只创建目录和手册骨架。
13. 不写 SPEC change。
14. 不生成 Goal Tree fact。
15. 不启动 AgentRun。
16. 不执行项目命令。
17. 不调用模型。
18. 不写用户源码。
19. 不创建远程 PR / Issue。
20. 不恢复 legacy workflow。

Root Agent Entry Shadow Guard 检查：
- `.rules`
- `.cursorrules`
- `.windsurfrules`
- `.clinerules`
- `.github/copilot-instructions.md`
- `AGENT.md`
- `CLAUDE.md`
- `GEMINI.md`

5 个 Agent 角色：
- 需求接待 Agent / Intake Agent
- 规格计划 Agent / Spec Planning Agent
- 实现执行 Agent / Build Agent
- 发布交付 Agent / Release Agent
- 代码审计 Agent / Audit Agent

实现范围：
- 新增 docs/requirements/008-3-agentflow-workflow-directory-blueprint-v1.md。
- Project Workspace prepare 新增 workflow layout prepare / repair。
- 新增 / 修复 `<project-root>/AGENTS.md`。
- 将 `AGENTS.md` 作为 canonical root agent entry 写入 workspace-manifest.json。
- 保留 `AGENT.MD` legacy compatibility 说明。
- 新增 Root Agent Entry Shadow Guard，并把 warnings 写入 layout status / manifest。
- 新增 `.agentflow/workspace-manifest.json`。
- 新增 `.agentflow/define/spec/SPEC.md`。
- 新增 `.agentflow/define/tdd/TDD.md`。
- 新增 `.agentflow/define/release/RELEASE.md`。
- 新增 `.agentflow/define/audit/AUDIT.md`。
- 创建 `.agentflow/spec/`。
- 创建 `.agentflow/goal-tree/`。
- 创建 `.agentflow/graph/`。
- 创建 `.agentflow/execute/`。
- 创建 `.agentflow/state/`。
- Agentflow.md Required Reading Order 改为 AGENTS.md。
- Agentflow.md 增加 Agent Roles 章节。
- Browser Preview mock 如有 Agent Manual 状态，也同步展示新 layout ready。
- 更新 verification。

验证命令：
- cargo fmt --check
- cargo test -p agentflow-agent-manual
- cargo test -p agentflow-desktop
- cargo test
- npm --prefix apps/desktop run build
- git diff --check
```

---

# 28. 完成定义

本需求完成后，`.agentflow/` 的语义应变成：

```text
AGENTS.md
= 项目根 canonical Agent 入口

define/
= 工作手册区

spec/
= 真实 SPEC 产物区

goal-tree/
= 真实目标树事实源

graph/
= 项目现场图谱

execute/
= 未来执行过程

output/
= 证据 / 审计报告 / 备份 / 日志

state/
= 健康 / 锁 / 会话 / 索引状态
```

最终一句话：

> **008.3 把 `.agentflow/` 从“本地文件夹”升级成 Agent 工作流控制面：AGENTS.md 作为项目根 canonical Agent 入口，define 只放五本手册，Goal Tree 和 Graph 都作为顶层事实源 / 现场源独立出来，Root Agent Entry Shadow Guard 防止外部 Agent 入口遮挡 AgentFlow 规则，未来执行、发布、审计都有明确位置。**
