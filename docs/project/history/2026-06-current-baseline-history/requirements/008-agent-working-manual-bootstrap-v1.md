# 008 - Agent Working Manual Bootstrap V1

创建日期：2026-06-02  
执行者：Codex

## 用户目标

AgentFlow 当前已经完成：

```text
Project Workspace Manager
Graph
Project File Reader
Goal Tree V1
Goal Tree Agent-only 边界修正
Legacy cleanup
```

但现在还缺一个更上游、更关键的能力：

```text
Agent 工作环境和 SPEC 工作手册。
```

大白话：

> Agent 进入一个本地项目之前，不能自由发挥。  
> 它必须先知道：这个项目由 AgentFlow 管理、入口在哪里、规则是什么、技能有哪些、哪些事情能做、哪些事情绝对不能做。  
> 所以我们要在项目初始化 / 打开 / 切换 / Agent 会话开始前，自动准备和校验 Agent 工作环境。  
> 环境坏了要自动修复；修不好要阻止 Agent 继续工作。

---

## 一句话定义

> **008 Agent Working Manual Bootstrap V1 负责接管项目根目录的 `AGENT.MD`，写入 AgentFlow 的 Agent 工作手册、内置 skills 和 `skills-lock.json`，并在 App 打开、项目切换、Agent 工作前持续检查和修复 Agent 工作环境。**

---

## 背景

我们参考 Warp 的项目初始化逻辑：

```text
bootstrap
  → 安装 / 校验项目需要的 agent skills
  → 让 Agent 读 WARP.md
  → issue 先 triage
  → feature 先 spec
  → product.md + tech.md approved
  → 再进入 code PR
```

AgentFlow 也需要类似机制，但形式是本地优先：

```text
AGENT.MD
  → .agentflow/define/agent/Agentflow.md
  → .agentflow/define/agent/skills/**
  → .agentflow/define/agent/skills-lock.json
```

其中：

```text
AGENT.MD
= Agent 进入项目后最先读的入口文件

Agentflow.md
= AgentFlow 总工作手册

skills/**
= AgentFlow 内置 Agent 工作技能

skills-lock.json
= 锁定技能版本、路径、hash，防止规则漂移
```

---

## 当前问题

如果没有 Agent 工作手册，Agent 可能会：

```text
随便改源码
绕过 OpenSpec
直接写 Goal Tree
直接执行命令
直接启动 AgentRun
直接创建 PR
复用旧 legacy 流程
```

这些都不允许。

因此，AgentFlow 必须先完成：

```text
Agent 入口接管
Agent 规则写入
Agent skills 锁定
Agent 环境健康检查
Agent 环境自动修复
Agent 行为 gating
```

---

## 与 Goal Tree 的关系

Goal Tree 已经存在，但已经被修正为：

```text
Agent-only 事实源
Human Desktop UI 只读
```

008 不继续扩展 Goal Tree。

正确顺序是：

```text
Agent Working Manual
  ↓
OpenSpec Authoring
  ↓
OpenSpec Approval
  ↓
Goal Tree Materialization
  ↓
AgentRun Preflight
```

Goal Tree 不是当前流程入口。  
当前流程入口是：

```text
AGENT.MD + Agentflow.md + skills-lock.json
```

---

# 1. 范围

本需求包含 10 个能力：

```text
1. 接管项目根目录 AGENT.MD
2. 写入 Agentflow.md
3. 写入 Agent 内置 skills
4. 写入 skills-lock.json
5. 写入 bootstrap / validation state
6. 环境健康检查
7. 环境自动修复
8. Project Workspace prepare 接入
9. Agent Environment 状态通道
10. Desktop 只读展示 Agent 工作环境状态
```

---

# 2. 非目标

本需求不做以下事情：

```text
不写 OpenSpec changes
不生成 OpenSpec Draft
不生成 Goal Tree
不启动 AgentRun
不执行项目命令
不运行测试
不调用模型
不修改用户源码
不创建 PR
不创建 GitHub Issue
不创建 Linear Issue
不接外部 skills marketplace
不恢复旧 CLI 写命令
不写旧 .agentflow/issues
不写旧 .agentflow/runs
不写旧 .agentflow/evidence
不写旧 .agentflow/reviews
不写旧 .agentflow/updates
不写旧 .agentflow/views
```

---

# 3. 合格 Agent 工作环境定义

一个 AgentFlow 项目环境必须具备：

```text
<project-root>/AGENT.MD

.agentflow/define/agent/
├── Agentflow.md
├── skills-lock.json
├── skills/
│   ├── request-triage/
│   │   └── SKILL.md
│   ├── openspec-authoring/
│   │   └── SKILL.md
│   ├── goal-tree-materialization/
│   │   └── SKILL.md
│   ├── boundary-check/
│   │   └── SKILL.md
│   └── validation/
│       └── SKILL.md
└── state/
    ├── bootstrap.json
    └── validation.json
```

说明：

```text
AGENT.MD
= Agent 入口文件，必须在项目根目录

Agentflow.md
= AgentFlow 总工作手册

skills/**
= 每个 Agent 工作技能说明

skills-lock.json
= 锁定技能版本、路径、hash

bootstrap.json
= 最近一次 bootstrap / repair 结果

validation.json
= 最近一次环境校验结果
```

---

# 4. AGENT.MD 接管规则

## 4.1 如果没有 AGENT.MD

自动创建：

```text
<project-root>/AGENT.MD
```

写入 AgentFlow managed 入口内容。

---

## 4.2 如果已有 AGENT.MD

必须：

```text
1. 读取现有 AGENT.MD
2. 备份原文件
3. 重写为 AgentFlow managed 版本
4. 写入 managed marker
5. 写入 validation warning
```

备份路径：

```text
.agentflow/output/backup/agent-md/AGENT.MD.<timestamp>.bak.md
```

原因：

```text
AgentFlow 要接管 Agent 入口，但不能无声破坏用户已有内容。
```

---

## 4.3 如果 AGENT.MD 被 Git 跟踪

仍然重写，但必须记录 warning：

```text
AGENT.MD is tracked by Git. AgentFlow rewrote it as the managed Agent entry.
Review your Git diff before committing.
```

该 warning 不阻止项目进入 ready，但必须在 Agent Environment 状态里可见。

---

## 4.4 如果 AGENT.MD 是 symlink

规则：

```text
symlink 指向 project root 内:
  可以重写目标，但记录 warning

symlink 指向 project root 外:
  不自动重写
  status = blocked
  Agent 功能不可用
```

---

## 4.5 AGENT.MD managed marker

AGENT.MD 必须包含：

```md
<!-- AGENTFLOW:MANAGED version=agent-entry.v1 -->
...
<!-- AGENTFLOW:END -->
```

环境检查通过 marker 判断：

```text
是否由 AgentFlow 管理
版本是否过旧
内容是否被改坏
是否需要修复
```

---

# 5. AGENT.MD 内容模板

`AGENT.MD` 必须保持很薄，只做入口。

建议模板：

```md
# AGENT.MD

<!-- AGENTFLOW:MANAGED version=agent-entry.v1 -->

This project is managed by AgentFlow.

Every Agent MUST read and follow:

1. `.agentflow/define/agent/Agentflow.md`
2. `.agentflow/define/agent/skills-lock.json`
3. All skills referenced by `skills-lock.json`

## Hard Rules

- Do not write source code unless AgentFlow rules explicitly allow it.
- Do not execute project commands unless AgentFlow rules explicitly allow it.
- Do not create or edit Goal Tree directly.
- Do not bypass OpenSpec.
- Do not create PRs, issues, or remote objects unless explicitly authorized.
- Human conversation is for confirmation and feedback, not direct Goal Tree editing.

## Current Flow

Conversation with human
→ OpenSpec Draft
→ Human confirmation
→ Approved OpenSpec
→ Goal Tree materialization
→ Future AgentRun

If any rule conflicts, AgentFlow rules win.

<!-- AGENTFLOW:END -->
```

---

# 6. Agentflow.md

路径：

```text
.agentflow/define/agent/Agentflow.md
```

作用：

```text
AgentFlow 总工作手册
```

内容必须包含：

```text
Role
Required reading order
Current project facts
Allowed actions
Forbidden actions
Required workflow
OpenSpec first rule
Goal Tree rule
Execution boundary
Validation rule
```

建议结构：

```md
# Agentflow.md

## Role

You are an Agent working inside an AgentFlow-managed local project.

## Required Reading Order

1. `<project-root>/AGENT.MD`
2. `.agentflow/define/agent/Agentflow.md`
3. `.agentflow/define/agent/skills-lock.json`
4. All required skills listed in `skills-lock.json`

## Current Project Facts

- Project Workspace is local-first.
- `.agentflow/` is the local AgentFlow runtime and definition space.
- Goal Tree is agent-only and human read-only.
- OpenSpec is the requirement source.
- Goal Tree is derived from approved OpenSpec.
- AgentRun is not authorized yet.

## Allowed Actions

- Read project files.
- Read Graph status.
- Read Project File Reader metadata.
- Read Goal Tree snapshot.
- Read existing OpenSpec drafts / approvals when they exist.
- Ask human clarification questions.
- Produce OpenSpec Draft previews in conversation.

## Forbidden Actions

- Do not write user source code.
- Do not execute project commands.
- Do not run tests.
- Do not create or edit Goal Tree directly.
- Do not write approved OpenSpec without human confirmation.
- Do not start AgentRun.
- Do not create PRs or remote issues.
- Do not use legacy workflow paths.

## Required Workflow

Conversation
→ Request triage
→ OpenSpec Draft
→ Human confirmation
→ Approved OpenSpec
→ Goal Tree materialization
→ Future AgentRun

## Boundary

If the requested action is outside the current authorized stage, stop and ask for confirmation or wait for the next AgentFlow requirement.
```

---

# 7. 内置 Agent Skills

V1 内置 5 个 skills。

---

## 7.1 request-triage

路径：

```text
.agentflow/define/agent/skills/request-triage/SKILL.md
```

作用：

```text
判断用户请求类型，并决定是否需要 OpenSpec。
```

必须支持分类：

```text
bug
feature
refactor
docs
research
cleanup
question
```

规则：

```text
feature:
  必须进入 OpenSpec Authoring

unclear change:
  必须先问问题，不能写事实源

bug:
  必须要求复现信息、当前行为、期望行为

cleanup:
  必须限定范围和非目标

question:
  只回答，不写入

research:
  输出调研结论，不写事实源，除非人类确认进入 OpenSpec
```

---

## 7.2 openspec-authoring

路径：

```text
.agentflow/define/agent/skills/openspec-authoring/SKILL.md
```

作用：

```text
指导 Agent 如何基于会话生成 OpenSpec Draft Preview。
```

OpenSpec Draft Preview 至少包含：

```text
Summary
Problem
Goals
Non-goals
User behavior
Edge cases
Acceptance criteria
Risks
Open questions
Product spec draft
Tech spec draft
Tasks draft
Validation plan
```

硬规则：

```text
没有人类确认，不写 .agentflow/define/openspec/**
OpenSpec 是需求源
Goal Tree 是派生产物
```

---

## 7.3 goal-tree-materialization

路径：

```text
.agentflow/define/agent/skills/goal-tree-materialization/SKILL.md
```

作用：

```text
指导 approved OpenSpec 如何转 Goal / Milestone / Issue。
```

硬规则：

```text
不能从聊天直接生成 Goal Tree
必须从 approved OpenSpec 生成
写入来源必须是 agent-system
Goal Tree 人类只读
Goal Tree 不执行
```

映射规则：

```text
OpenSpec objective
  -> Goal.objective

OpenSpec scope / non-goals
  -> Goal.scope / Goal.nonGoals

OpenSpec phases / design stages
  -> Milestone.stageGoal

OpenSpec tasks
  -> Issue.goal

OpenSpec acceptance criteria
  -> Issue.acceptanceCriteria

OpenSpec constraints / boundaries
  -> Issue.boundary

OpenSpec task dependencies
  -> Issue.dependencies
```

---

## 7.4 boundary-check

路径：

```text
.agentflow/define/agent/skills/boundary-check/SKILL.md
```

作用：

```text
Agent 每次行动前检查是否越界。
```

必须检查：

```text
是否要写用户源码？
是否要执行命令？
是否要写 OpenSpec？
是否有人类确认？
是否要写 Goal Tree？
是否已有 approved OpenSpec？
是否要启动 AgentRun？
是否要创建远程对象？
是否涉及 legacy 路径？
```

如果越界：

```text
停止
说明原因
请求人类确认或等待后续能力
```

---

## 7.5 validation

路径：

```text
.agentflow/define/agent/skills/validation/SKILL.md
```

作用：

```text
Agent 每次输出 / 写入前自检。
```

检查：

```text
是否读取 AGENT.MD？
是否读取 Agentflow.md？
是否读取 skills-lock？
是否遵守 OpenSpec first？
是否错误写入 Goal Tree？
是否错误执行命令？
是否包含待确认问题？
是否需要停止？
```

---

# 8. skills-lock.json

路径：

```text
.agentflow/define/agent/skills-lock.json
```

格式：

```json
{
  "version": "agentflow-skills-lock.v1",
  "managedBy": "AgentFlow",
  "updatedAt": 1780291200,
  "entry": {
    "path": "AGENT.MD",
    "version": "agent-entry.v1",
    "managed": true,
    "hash": "<sha256>"
  },
  "manual": {
    "path": ".agentflow/define/agent/Agentflow.md",
    "version": "agentflow-manual.v1",
    "hash": "<sha256>"
  },
  "skills": {
    "request-triage": {
      "version": "v1",
      "path": ".agentflow/define/agent/skills/request-triage/SKILL.md",
      "hash": "<sha256>"
    },
    "openspec-authoring": {
      "version": "v1",
      "path": ".agentflow/define/agent/skills/openspec-authoring/SKILL.md",
      "hash": "<sha256>"
    },
    "goal-tree-materialization": {
      "version": "v1",
      "path": ".agentflow/define/agent/skills/goal-tree-materialization/SKILL.md",
      "hash": "<sha256>"
    },
    "boundary-check": {
      "version": "v1",
      "path": ".agentflow/define/agent/skills/boundary-check/SKILL.md",
      "hash": "<sha256>"
    },
    "validation": {
      "version": "v1",
      "path": ".agentflow/define/agent/skills/validation/SKILL.md",
      "hash": "<sha256>"
    }
  }
}
```

---

# 9. Agent Environment 状态模型

新增状态：

```json
{
  "version": "agent-environment-status.v1",
  "projectRoot": "/path/to/project",
  "status": "ready",
  "ready": true,
  "checkedAt": 1780291200,
  "repairedAt": null,
  "agentMd": {
    "exists": true,
    "managed": true,
    "version": "agent-entry.v1",
    "hash": "<sha256>",
    "backedUp": false,
    "trackedByGit": false
  },
  "manual": {
    "exists": true,
    "path": ".agentflow/define/agent/Agentflow.md",
    "hash": "<sha256>"
  },
  "skillsLock": {
    "exists": true,
    "valid": true,
    "path": ".agentflow/define/agent/skills-lock.json",
    "skillCount": 5
  },
  "skills": [
    {
      "name": "request-triage",
      "path": ".agentflow/define/agent/skills/request-triage/SKILL.md",
      "exists": true,
      "hashMatches": true,
      "version": "v1"
    }
  ],
  "repairs": [],
  "warnings": [],
  "errors": []
}
```

状态枚举：

```text
missing
checking
repairing
ready
repaired
degraded
failed
blocked
```

含义：

```text
ready
= 环境完整，无需修复

repaired
= 本次检查发现问题并已自动修复

degraded
= 有 warning，但 Agent 可继续

failed
= 检查失败，原因可见

blocked
= 无法修复，Agent 不允许继续
```

---

# 10. 环境检查时机

环境不是初始化一次就完事。

必须在以下时机检查。

---

## 10.1 App 启动时

App 启动后检查：

```text
当前 active project
最近选择 project
```

不要求扫描所有历史项目，避免启动变慢。

---

## 10.2 添加项目时

添加本地项目时必须执行：

```text
prepare_local_project_workspace
→ prepare_agent_working_manual
→ validate_agent_working_manual
```

如果坏了：

```text
自动修复
```

修不好：

```text
项目可只读打开
Agent 功能 blocked
```

---

## 10.3 切换项目时

每次切换 projectRoot：

```text
先检查 Agent 工作环境
再显示 Agent 状态
```

---

## 10.4 Agent 会话开始前

未来任何 Agent 会话开始前必须执行：

```text
assert_agent_environment_ready(projectRoot)
```

不通过：

```text
禁止 Agent 继续
```

---

## 10.5 OpenSpec 写入前

未来写 OpenSpec 前必须检查：

```text
AGENT.MD ready
Agentflow.md ready
skills-lock ready
openspec-authoring skill ready
boundary-check skill ready
validation skill ready
```

---

## 10.6 Goal Tree materialization 前

未来 approved OpenSpec 转 Goal Tree 前必须检查：

```text
goal-tree-materialization skill ready
boundary-check skill ready
approved OpenSpec exists
```

---

## 10.7 AgentRun 前

未来 AgentRun 前必须检查：

```text
Agent working manual ready
Graph ready / degraded allowed
Approved OpenSpec exists
Goal Tree ready
Issue ready
```

---

# 11. 自动修复规则

## 11.1 可以自动修复

以下情况自动修复：

```text
AGENT.MD 缺失
AGENT.MD 不是 AgentFlow managed 版本
AGENT.MD managed marker 版本过旧
AGENT.MD hash 不匹配
Agentflow.md 缺失
Agentflow.md hash 不匹配
skills-lock.json 缺失
skills-lock.json 格式错误
某个 SKILL.md 缺失
某个 SKILL.md hash 不匹配
state/bootstrap.json 缺失
state/validation.json 缺失
```

修复动作：

```text
备份现有 AGENT.MD
重写 AGENT.MD
重写 Agentflow.md
重写 skills/**
重写 skills-lock.json
重算 hash
重写 bootstrap.json
重写 validation.json
记录 repairs
```

---

## 11.2 不能自动修复

以下情况不能静默修复：

```text
项目根目录不可写
.agentflow/ 不可写
AGENT.MD 是只读文件
AGENT.MD 是 symlink 且指向项目外
备份失败
磁盘写入失败
权限不足
```

处理：

```text
status = blocked
ready = false
Agent 相关功能不可用
UI 显示明确错误
```

---

# 12. 本轮允许写入路径

允许写：

```text
<project-root>/AGENT.MD
.agentflow/define/agent/**
.agentflow/output/backup/agent-md/**
.agentflow/output/logs/**
```

不允许写：

```text
用户源码
OpenSpec changes
Goal Tree
AgentRun
旧 .agentflow/issues
旧 .agentflow/runs
旧 .agentflow/evidence
旧 .agentflow/reviews
旧 .agentflow/updates
旧 .agentflow/views
.gitignore
远程服务
```

特别说明：

```text
AGENT.MD 是本轮唯一允许写到项目根目录的文件。
```

---

# 13. Rust 模块设计

建议新增 crate：

```text
crates/agent-manual/
```

Cargo package：

```text
agentflow-agent-manual
```

原因：

```text
Agent 工作手册是 AgentFlow 新主链路基础能力
不应该放到 legacy core
不应该混入 graph / goal-tree
```

结构：

```text
crates/agent-manual/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── manager.rs
    ├── templates.rs
    ├── lock.rs
    ├── validate.rs
    ├── repair.rs
    ├── git.rs
    ├── hash.rs
    └── model.rs
```

---

## 13.1 模块职责

```text
manager.rs
= prepare / validate / repair 总入口

templates.rs
= AGENT.MD、Agentflow.md、SKILL.md 模板

lock.rs
= skills-lock.json 生成和读取

validate.rs
= 环境校验

repair.rs
= 自动修复和备份

git.rs
= 判断 AGENT.MD 是否被 Git 跟踪

hash.rs
= sha256 计算

model.rs
= AgentEnvironmentStatus / SkillLock / SkillStatus 等类型
```

---

# 14. Project Workspace 接入

`prepare_local_project_workspace` 必须接入：

```text
agentflow_agent_manual::prepare_agent_working_manual(project_root)
```

执行顺序建议：

```text
prepare_local_project_workspace
  → ensure .agentflow base dirs
  → ensure define / execute / output dirs
  → ensure git protection
  → ensure graph dirs
  → prepare_agent_working_manual
  → validate_agent_working_manual
```

---

# 15. Tauri commands

新增 command wrapper：

```text
apps/desktop/src-tauri/src/commands/agent_manual.rs
```

命令：

```text
prepare_agent_working_manual
load_agent_environment_status
repair_agent_working_manual
validate_agent_working_manual
```

---

## 15.1 prepare_agent_working_manual

```ts
prepare_agent_working_manual(projectRoot: string): AgentEnvironmentStatus
```

行为：

```text
创建 / 修复 AGENT.MD 和 .agentflow/define/agent/**
```

---

## 15.2 load_agent_environment_status

```ts
load_agent_environment_status(projectRoot: string): AgentEnvironmentStatus
```

行为：

```text
只读读取最近状态
如果没有 validation.json，可即时 validate
```

---

## 15.3 repair_agent_working_manual

```ts
repair_agent_working_manual(projectRoot: string): AgentEnvironmentStatus
```

行为：

```text
强制修复
```

注意：

```text
UI 可以不提供按钮，主要给系统调用。
```

---

## 15.4 validate_agent_working_manual

```ts
validate_agent_working_manual(projectRoot: string): AgentEnvironmentStatus
```

行为：

```text
只检查，不修复
```

---

# 16. Desktop UI

## 16.1 状态通道

新增状态：

```text
Agent Manual
```

显示：

```text
ready
repaired
degraded
blocked
failed
```

示例：

```text
Agent Manual: Ready
AGENT.MD: Managed
Skills: 5/5 locked
Warnings: AGENT.MD tracked by Git
```

---

## 16.2 Project 页面只读卡片

可以在 Project 页面展示：

```text
Agent 工作手册
- Status
- AGENT.MD managed
- Skills lock
- Last check
- Repairs
- Warnings
- Errors
```

不做编辑器。

---

## 16.3 自动检查

UI 不能要求用户手动点检测。

必须在：

```text
App 启动
项目添加
项目切换
```

自动执行或自动排队执行。

---

# 17. Browser Preview

Browser Preview 不写真实文件。

返回 mock：

```text
createBrowserPreviewAgentEnvironmentStatus
```

显示：

```text
Agent Manual: Preview Ready
AGENT.MD: mock managed
Skills: 5/5
```

---

# 18. 开发切片

## Slice 1：Agent Manual crate scaffold

目标：

```text
新增 crates/agent-manual
定义模型
定义模板
```

验收：

```text
cargo test -p agentflow-agent-manual 通过
```

---

## Slice 2：AGENT.MD 接管

目标：

```text
检测 / 创建 / 备份 / 重写 AGENT.MD
managed marker
tracked by Git warning
```

验收：

```text
无 AGENT.MD 时创建
有 AGENT.MD 时备份并重写
AGENT.MD tracked by Git 时 warning
```

---

## Slice 3：Agentflow.md + skills 写入

目标：

```text
写 Agentflow.md
写 5 个 skills/SKILL.md
```

验收：

```text
文件存在
内容包含必要硬规则
```

---

## Slice 4：skills-lock + hash validation

目标：

```text
生成 skills-lock.json
计算 hash
校验 hash
```

验收：

```text
篡改 SKILL.md 后 validate 发现 mismatch
repair 后恢复
```

---

## Slice 5：环境状态和自动修复

目标：

```text
AgentEnvironmentStatus
bootstrap.json
validation.json
auto repair
blocked 状态
```

验收：

```text
缺文件可自动修复
权限错误进入 blocked
```

---

## Slice 6：Project Workspace 接入

目标：

```text
prepare_local_project_workspace 自动调用 prepare_agent_working_manual
```

验收：

```text
添加 / 打开项目后自动具备 Agent 工作手册
```

---

## Slice 7：Tauri + Desktop status

目标：

```text
Tauri commands
Agent Manual status channel
Browser preview mock
```

验收：

```text
Desktop 显示 Agent Manual 状态
```

---

## Slice 8：Docs and verification

目标：

```text
新增 008 需求文档
更新 README / next requirements
更新 verification
```

验收：

```text
验证命令通过
```

---

# 19. 总验收标准

```text
- [ ] 新增 docs/requirements/008-agent-working-manual-bootstrap-v1.md。
- [ ] 新增 crates/agent-manual。
- [ ] Project Workspace prepare 接入 Agent Manual bootstrap。
- [ ] 打开项目时自动检查 Agent 工作环境。
- [ ] 添加项目时自动准备 Agent 工作环境。
- [ ] 切换项目时自动检查 Agent 工作环境。
- [ ] 缺少 AGENT.MD 时自动创建。
- [ ] 已有 AGENT.MD 时自动备份并重写。
- [ ] AGENT.MD 有 AgentFlow managed marker。
- [ ] AGENT.MD 被 Git 跟踪时记录 warning。
- [ ] AGENT.MD 指向 Agentflow.md 和 skills-lock.json。
- [ ] Agentflow.md 写入完整工作手册。
- [ ] 5 个内置 skills 写入完成。
- [ ] skills-lock.json 写入完成。
- [ ] skills-lock.json 包含 hash。
- [ ] validate 能发现缺失文件。
- [ ] validate 能发现 hash mismatch。
- [ ] repair 能自动修复缺失 / mismatch。
- [ ] 无法修复时 status = blocked。
- [ ] state/bootstrap.json 写入。
- [ ] state/validation.json 写入。
- [ ] Tauri commands 可用。
- [ ] Desktop 状态通道显示 Agent Manual。
- [ ] Browser Preview 有 mock 状态。
- [ ] 本轮只允许写 AGENT.MD 和 .agentflow/define/agent/** 等授权路径。
- [ ] 不写用户源码。
- [ ] 不写 OpenSpec changes。
- [ ] 不写 Goal Tree。
- [ ] 不启动 Agent。
- [ ] 不执行项目命令。
- [ ] 不调用模型。
- [ ] cargo fmt --check 通过。
- [ ] cargo test -p agentflow-agent-manual 通过。
- [ ] cargo test 通过。
- [ ] npm --prefix apps/desktop run build 通过。
- [ ] git diff --check 通过。
```

---

# 20. 验证命令

必须执行：

```bash
cargo fmt --check
cargo test -p agentflow-agent-manual
cargo test
npm --prefix apps/desktop run build
git diff --check
```

如果改 Desktop Tauri：

```bash
cargo test -p agentflow-desktop
```

---

# 21. PR 说明要求

PR 描述必须说明：

```text
1. AGENT.MD 是否会被创建 / 重写。
2. 已有 AGENT.MD 如何备份。
3. AGENT.MD 是否可能被 Git 跟踪，如何提示。
4. Agentflow.md 写入了什么。
5. 内置 skills 有哪些。
6. skills-lock 如何校验。
7. 什么时候自动检查环境。
8. 环境坏了如何自动修复。
9. 修不好如何阻止 Agent。
10. 本轮写入了哪些路径。
11. 本轮没有写入哪些路径。
12. 验证命令和结果。
```

---

# 22. Codex 执行指令

```md
请执行 008 - Agent Working Manual Bootstrap V1。

目标：
在项目打开 / 添加 / 切换 / Agent 会话前，自动准备、校验和修复 Agent 工作环境。接管项目根目录 AGENT.MD，写入 AgentFlow 工作手册 Agentflow.md、内置 Agent skills 和 skills-lock.json，确保 Agent 必须在 AgentFlow 规则之下工作。

必须遵守：
1. AGENT.MD 是本轮唯一允许写入项目根目录的文件。
2. 如果 AGENT.MD 不存在，创建它。
3. 如果 AGENT.MD 已存在，先备份，再重写为 AgentFlow managed 版本。
4. 如果 AGENT.MD 是 symlink 且指向项目外，禁止重写并进入 blocked。
5. 写入 .agentflow/define/agent/Agentflow.md。
6. 写入 5 个内置 skills。
7. 写入 skills-lock.json，并包含 hash。
8. App 打开 / 项目添加 / 项目切换 / Agent 会话前都必须检查 Agent 环境。
9. 缺失或 hash mismatch 要自动修复。
10. 修复失败要进入 blocked，并阻止 Agent 继续。
11. 不写用户源码。
12. 不写 OpenSpec changes。
13. 不写 Goal Tree。
14. 不启动 Agent。
15. 不执行项目命令。
16. 不调用模型。
17. 不写旧 .agentflow/issues/runs/evidence/reviews/updates/views。

实现范围：
- 新增 crates/agent-manual。
- 新增 AgentEnvironmentStatus / SkillsLock / SkillStatus 等模型。
- 新增 AGENT.MD / Agentflow.md / SKILL.md 模板。
- 新增 prepare / validate / repair API。
- Project Workspace prepare 接入 Agent Manual bootstrap。
- 新增 Tauri commands。
- Desktop status channel 显示 Agent Manual。
- Browser Preview mock。
- 更新 docs 和 verification。

验证命令：
- cargo fmt --check
- cargo test -p agentflow-agent-manual
- cargo test -p agentflow-desktop
- cargo test
- npm --prefix apps/desktop run build
- git diff --check
```

---

# 23. 完成定义

本需求完成后，AgentFlow 应达到：

```text
每个打开的本地项目都有 AgentFlow 管理的 AGENT.MD
每个项目都有 Agentflow.md 总工作手册
每个项目都有锁定的 Agent skills
每个项目都有 skills-lock.json
App 每次进入项目都会检查环境
环境坏了会自动修复
修不好会阻止 Agent 工作
Agent 不再能绕过规则自由行动
后续 OpenSpec / Goal Tree / AgentRun 都必须建立在这个工作手册之上
```

最终一句话：

> **008 不是初始化文件，而是建立 Agent 的工作环境健康闭环：Agent 入口、规则、skills、lock、检查、修复、阻断，一个都不能少。**
