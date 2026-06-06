# 029 - AgentFlow Dogfood Cutover Cleanup & Start Workflow V1

> 建议保存路径：`docs/requirements/029-agentflow-dogfood-cutover-cleanup-and-start-v1.md`
> 类型：Base Release 前置收口 / Dogfood 启动规则 / 清理任务 / 后续开发流程
> 状态：Ready for Codex implementation
> 目标：在 `027-agentflow-agent-role-descriptor-and-issue-guard-v1` 完成后，正式切到 `.agentflow` 规则驱动的开发流程。

---

## 1. 背景

AgentFlow 当前已经完成大量底层能力和前端页面修复：

```text
define/
panel/
input/
execute/
output/
state/
desktop frontend
project tree
issue status
release delivery
audit trigger
agent role guard
```

`027-agentflow-agent-role-descriptor-and-issue-guard-v1` 完成后，AgentFlow 的核心规则会变成：

```text
Spec Issue 只能由 Build Agent 执行
Audit Issue 只能由 Audit Agent 执行
Spec Agent 只整理需求和生成 Issue，不执行任务
Release 后生成 Audit Issue
Audit Agent 写审计报告
```

这意味着 AgentFlow 已经可以开始正式 dogfood：

```text
用 AgentFlow 管 AgentFlow 自己的需求。
```

从这个 cutover 之后，不能再继续用“聊天里直接说需求 → Codex 直接改代码”的方式推进。

---

## 2. 一句话目标

> **完成最后一轮 Base 前清理，把旧设计稿、旧文档、旧 mock、旧单项目状态、旧审计入口和旧开发习惯收口。之后所有 AgentFlow 新需求都必须进入 `.agentflow/input`，通过 SPEC → Issue → Handoff → Delivery → Audit 的真实流程执行。**

---

## 3. Cutover 前置条件

必须先完成：

```text
1. 023 页面修复完成
2. 024 Base Release 初始化完成
3. 025 多项目 ProjectTree 完成
4. 026 Release 后审计规则完成
5. 027 Agent Role / Issue Guard 完成
```

其中 027 是硬前置：

```text
没有 Issue 分类和 Agent 角色隔离，就不能正式开始 dogfood。
```

---

## 4. 清理目标

本需求要清理和收口 8 类问题：

```text
1. 文档事实源混乱
2. Figma / SVG / 历史设计稿残留
3. 真实客户端 mock fallback 边界
4. 单项目 localStorage 旧 key
5. 登录 / 首次引导默认阻断
6. 审计 request-only 状态
7. AGENTS.md Git 管理
8. release / audit / issue 状态不闭环
```

---

## 5. 事实源重新定义

Cutover 后事实源如下：

| 类型 | 事实源 | 说明 |
|---|---|---|
| 需求事实 | `.agentflow/input/specs/**` | Draft / Approved SPEC |
| 任务事实 | `.agentflow/input/issues/**` | Spec Issue / Audit Issue |
| 执行事实 | `.agentflow/execute/**` | run / lease / plan / validation |
| 交付事实 | `.agentflow/output/release/**` | local release delivery |
| 证据事实 | `.agentflow/output/evidence/**` | validation / evidence |
| 审计事实 | `.agentflow/output/audit/**` | audit report / findings / evidence map |
| 派生状态 | `.agentflow/state/**` | gates / indexes / blockers |
| 设计参考 | Figma / SVG | 只作为 UI 参考，不是事实源 |
| 文档归档 | `docs/requirements/**` | 人类可读记录，不是执行事实源 |
| GitHub PR | GitHub | 代码变更载体，不是需求事实源 |

---

## 6. 文档清理

### 6.1 需要标记历史参考的文档

这些文档可以保留，但要在索引里标明：

```text
historical design reference
pre-base requirement
superseded by dogfood workflow
```

包括但不限于：

```text
015-human-agent-guided-experience-v1.md
017-agentflow-unified-ux-spec-v16-pr33-merged.md
018-agentflow-latest-svg-frontend-content-implementation.md
019-agentflow-frontend-interaction-ux-v1.md
020-agentflow-frontend-design-system-and-app-shell-v1.md
021-agentflow-frontend-interaction-ux-v1.md
023-agentflow-github-code-latest-svg-page-repair-v1.md
```

### 6.2 当前有效基线文档

Cutover 前仍有效：

```text
024-agentflow-base-release-initialization-v1.md
025-agentflow-project-tree-multi-project-navigation-v1.md
026-agentflow-release-audit-trigger-rules-v1.md
027-agentflow-agent-role-descriptor-and-issue-guard-v1.md
028-agentflow-codex-role-usage-guide-v1.md
029-agentflow-dogfood-cutover-cleanup-and-start-v1.md
```

### 6.3 README / next-requirements 更新

更新：

```text
docs/requirements/README.md
docs/requirements/next-requirements.md
```

加入：

```text
029-agentflow-dogfood-cutover-cleanup-and-start-v1.md
```

并标注：

```text
029 = dogfood cutover baseline
```

---

## 7. `.agentflow` 工作区清理

### 7.1 必须保留的目录

`.agentflow` 应保留：

```text
define/
panel/
input/
execute/
output/
state/
config.yaml
workspace.yaml
workspace-manifest.yaml
```

### 7.2 需要检查的目录

检查并确认：

```text
.agentflow/input/issues/**
.agentflow/input/specs/**
.agentflow/output/release/**
.agentflow/output/audit/**
.agentflow/state/indexes/**
.agentflow/state/gates/**
```

### 7.3 goals / milestones / issues 旧目录清理删除

如果 `.agentflow/define/` 下存在：

```text
goals/
milestones/
issues/
```

需要判断是否仍是当前工作流需要的定义资料。

规则：

```text
如果只是旧 Goal Tree / legacy planning 残留：
  直接删除
  不作为当前事实源


最终规则：

```text
当前任务事实源只能是 .agentflow/input/issues/**
```

---

## 8. Demo / Mock 数据清理

### 8.1 Browser Preview

允许继续使用 mock：

```text
apps/desktop/src/browserPreviewData.ts
```

用途：

```text
浏览器预览
UI 验证
本地 fixture
```

### 8.2 真实 Desktop 客户端

真实客户端禁止：

```text
silent fallback 到 browser mock
用 demo delivery 冒充真实 delivery
用 demo audit 冒充真实 audit
用 PR context 冒充正式 issue
```

### 8.3 Demo 数据标记

所有 demo 数据必须带：

```json
{
  "demo": true,
  "source": "agentflow-demo"
}
```

或：

```json
{
  "system": {
    "createdBy": "agentflow-demo"
  }
}
```

### 8.4 真实数据出现后的处理

如果真实 `.agentflow/input/issues` 已存在：

```text
不要再展示 demo issue 作为正式任务
```

如果真实 delivery / audit 不存在：

```text
显示空态
不要用 demo 替代
```

---

## 9. localStorage 清理

### 9.1 旧 key

旧 key：

```text
agentflow.interaction.projectRoot.v1
agentflow.interaction.activePage.v1
```

### 9.2 新 key

新 key：

```text
agentflow.projects.v1
agentflow.activeProjectRoot.v1
agentflow.expandedProjectRoots.v1
agentflow.activePageByProject.v1
```

### 9.3 迁移规则

```text
1. 如果新 registry 不存在，但旧 projectRoot 存在：
   创建一个 project ref
2. 写入 projects[]
3. 设置 activeProjectRoot
4. 保留旧 key 或清理旧 key
5. 后续读取优先使用新 registry
```

### 9.4 所有项目移除后的状态

当 `projects.length === 0`：

```text
activeProjectRoot = null
activePage = home
selectedTaskId = null
selectedDeliveryRunId = null
selectedAuditId = null
taskSearch = ""
```

UI：

```text
左侧：暂无项目
主内容：还没有项目，添加本地项目
顶部：未选择项目
底部：本地模式
```

---

## 10. 登录 / 首次引导清理

Base Release 不默认显示：

```text
LoginModal
FirstRunModal
Provider 选择窗口
```

规则：

```text
组件可以保留
默认路由不触发
不阻断进入项目工作台
后续版本再重新接入
```

禁止：

```text
打开 App 先要求登录
打开 App 强制首次引导
没有 provider 就进不了工作台
```

---

## 11. AGENTS.md 清理

### 11.1 文件位置

```text
./AGENTS.md
```

### 11.2 规则

```text
1. 项目打开 / prepare 时自动生成
2. 如果已存在，不覆盖用户修改
3. 自动写入 .gitignore
4. 不提交到 Git
```

### 11.3 如果已经被 Git 跟踪

不要自动执行危险命令。
给用户提示：

```text
AGENTS.md 已生成，并已加入 .gitignore。
如果它之前已经被 Git 跟踪，请手动执行：
git rm --cached AGENTS.md
```

---

## 12. Audit Request 清理

### 12.1 新规则

审计入口是：

```text
Audit Issue
```

不是：

```text
audit-request.json
```

### 12.2 对已有 audit-request.json 的处理

如果发现：

```text
.agentflow/output/audit/audit-001/audit-request.json
```

但没有对应 Audit Issue，则创建：

```text
.agentflow/input/issues/audit-<release-id>.json
```

### 12.3 audit-request 的定位

`audit-request.json` 可以：

```text
作为兼容 metadata 保留
作为历史记录保留
```

但不能作为 Agent 执行入口。

### 12.4 正确链路

```text
Release Delivery
→ Audit Issue
→ Audit Agent Handoff
→ Audit Report
```

---

## 13. Issue 分类清理

### 13.1 所有 Issue 必须有

```text
issueCategory
requiredAgentRole
displayStatus
```

### 13.2 兼容旧 Issue

旧 Issue 缺字段时：

```text
issueCategory = spec
requiredAgentRole = build-agent
displayStatus = 由现有 status 推导
```

### 13.3 Audit Issue

Release 后生成：

```text
issueCategory = audit
requiredAgentRole = audit-agent
displayStatus = ready
riskLevel = high
```

### 13.4 Spec Issue

普通开发 / 修复 / 文档 / 验证任务：

```text
issueCategory = spec
requiredAgentRole = build-agent
```

---

## 14. Agent Role Guard 清理

### 14.1 必须生成

```text
.agentflow/define/agent/roles.json
```

### 14.2 必须写入任务包

```text
requiredAgentRole
issueCategory
```

### 14.3 必须校验写回

写回必须声明：

```text
claimedAgentRole
issueId
issueCategory
handoffId
```

不匹配则：

```text
不接受写回
不更新 done
写 blocker
写 timeline event
```

---

## 15. Release Cutover 规则

### 15.1 本地 release delivery

Release 先本地生成：

```text
.agentflow/output/release/release-v0.1.0/
```

至少包含：

```text
delivery.json
release-note.md
changelog.md
review-checklist.md
evidence refs
validation result
```

### 15.2 GitHub Release

GitHub Release 不是第一步。
顺序：

```text
本地 release delivery
→ Audit Issue
→ Audit Report
→ 人类确认
→ Git tag / GitHub Release
```

不要在 Audit Report 缺失时发布 GitHub Release。

---

## 16. Dogfood 启动流程

从 028 合并后，后续需求必须走：

```text
1. 用户提出需求
2. Spec Agent 整理 SPEC Draft
3. 用户确认
4. 写入 approved SPEC
5. 生成 Input Issue
6. Build Agent 复制任务包 / 执行
7. 写回 Evidence / Delivery
8. Release 后生成 Audit Issue
9. Audit Agent 执行审计
10. 写回 Audit Report
11. 任务关闭
```

---

## 17. 第一个 Dogfood 任务建议

### 17.1 生成 cutover SPEC

建议创建：

```text
.agentflow/input/specs/approved/dogfood-cutover-v1/
```

内容：

```text
product.md
tech.md
spec.json
approval.json
```

### 17.2 生成第一个正式 Issue

```text
.agentflow/input/issues/AF-DOGFOOD-001.json
```

建议内容：

```text
title: 清理 pre-base 文档并启用 dogfood 工作流
issueCategory: spec
requiredAgentRole: build-agent
displayStatus: ready
riskLevel: medium
```

### 17.3 如果已有 release-v0.1.0

必须生成：

```text
.agentflow/input/issues/audit-release-v0.1.0.json
```

内容：

```text
issueCategory: audit
requiredAgentRole: audit-agent
displayStatus: ready
riskLevel: high
```

---

## 18. 前端清理

### 18.1 真实客户端

必须只读真实 `.agentflow` 数据：

```text
input/issues
state/indexes
output/release
output/audit
```

### 18.2 没数据时

显示空态，不 fallback mock：

```text
没有任务
没有交付
没有审计
```

### 18.3 页面不应出现

```text
请求审计
新建审计
重新审计
补证据
登录阻断
首次引导阻断
```

---

## 19. 验收命令

必须执行：

```bash
cargo check --workspace
cargo test --workspace
npm --prefix apps/desktop run build
git diff --check
```

可选：

```bash
npm --prefix apps/desktop run lint
```

---

## 20. Base Release 前最终验收

必须满足：

```text
1. App 可以无登录进入项目工作台。
2. 多项目 ProjectTree 可用。
3. 所有项目移除后进入 Empty Workspace。
4. AGENTS.md 已加入 .gitignore。
5. Issue 有 issueCategory / requiredAgentRole。
6. Release 后生成 Audit Issue。
7. Audit Issue 只能由 Audit Agent 执行。
8. 真实客户端不 silent fallback mock。
9. 交付页不提供请求审计按钮。
10. 审计页不提供新建审计按钮。
11. 高级页可以查看 raw state，但只读。
12. 本地 release delivery 可生成。
13. Audit report 缺失时不允许 GitHub Release。
```

---

## 21. Codex 执行指令

```text
你现在只做这个任务：AgentFlow Dogfood Cutover Cleanup & Start Workflow V1。

背景：
027 Agent Role Descriptor & Issue Guard 完成后，AgentFlow 要正式切到 dogfood 模式。之后不能再从聊天需求直接跳到代码修改，所有新需求必须进入 .agentflow/input 并通过 SPEC / Issue / Delivery / Audit 流程执行。

目标：
完成 Base Release 前最后一轮清理，并建立正式 dogfood 启动入口。

范围：
- .agentflow/**
- docs/requirements/**
- apps/desktop/src/**
- crates/input/**
- crates/state/**
- crates/output/**
- AGENTS.md 生成规则
- 相关测试

具体任务：
1. 更新 docs/requirements 索引，标记 pre-base 文档和当前基线。
2. 检查 .agentflow 目录，明确 input/issues 是任务事实源。
3. 清理或标记 legacy goals / milestones / issues 目录，不让它们替代 input/issues。
4. 确保真实客户端不 silent fallback browser mock。
5. 迁移旧 single project localStorage 到 Project Registry。
6. 确保所有项目移除后进入 Empty Workspace。
7. 确保登录 / 首次引导不再阻断 Base Release 启动。
8. 确保 AGENTS.md 自动加入 .gitignore。
9. 如果存在 release audit request 但没有 Audit Issue，生成 Audit Issue。
10. 确保所有 Issue 有 issueCategory / requiredAgentRole / displayStatus。
11. 生成或确认 .agentflow/define/agent/roles.json。
12. 生成第一个 dogfood cutover SPEC 和 AF-DOGFOOD-001。
13. 如果 release-v0.1.0 已存在，生成 audit-release-v0.1.0 Audit Issue。
14. 跑验证命令。

禁止：
- 不要创建 GitHub Release。
- 不要自动调用 Codex API。
- 不要把 docs/requirements 当事实源。
- 不要把 audit-request 当审计入口。
- 不要让真实客户端使用 browser mock。
- 不要删除用户源码。
- 不要删除 .agentflow 真实数据。
- 不要强制删除 AGENTS.md。
- 不要恢复登录 / 首次引导阻断。

验证：
- cargo check --workspace
- cargo test --workspace
- npm --prefix apps/desktop run build
- git diff --check

输出：
- 清理了哪些内容
- 生成了哪些 .agentflow 输入
- 哪些旧文档被标记为 historical
- 是否生成 dogfood cutover SPEC
- 是否生成 AF-DOGFOOD-001
- 是否生成 audit-release-v0.1.0
- 验证命令结果
```

---

## 22. 后续硬规则

从本需求合并后，所有新开发都必须遵守：

```text
没有 approved SPEC，不开发。
没有 Issue，不生成任务包。
没有 requiredAgentRole，不执行。
Build Agent 不执行 Audit Issue。
Audit Agent 不执行 Spec Issue。
没有 Delivery，不进入审计。
Release 后必须生成 Audit Issue。
没有 Audit Report，不发 GitHub Release。
```
