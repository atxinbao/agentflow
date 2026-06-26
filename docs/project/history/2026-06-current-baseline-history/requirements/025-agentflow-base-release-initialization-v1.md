# 025 - AgentFlow Base Release Initialization V1

> 建议保存路径：`docs/requirements/025-agentflow-base-release-initialization-v1.md`
> 目标版本：`v0.1.0-base`
> 类型：Base Release 初始化 / 产品体验 / 本地数据策略 / 前端真实数据过渡需求
> 状态：Ready for Codex implementation

---

## 1. 背景

AgentFlow 当前底层能力已经基本闭环，前端页面也已经进入最新 SVG / GitHub 代码对齐阶段。

在切 `v0.1.0-base` 之前，还需要处理 Base Release 的初始化体验。

当前用户反馈了两个关键问题：

```text
1. 当前代码需要去掉登录和首次引导页面逻辑。
2. 打开 App 后：
   - 如果是新项目，需要构造 3-5 条 mock 数据，让用户可以体验 AgentFlow 流程。
   - 如果是已有项目，需要从 Git PR 里获取 5-10 条项目上下文 PR，方便承接真实需求。
```

本需求定义 Base Release 初始化策略。
完成后，AgentFlow 才适合作为后续 dogfood 的基础版本。

---

## 2. 一句话目标

> **Base Release 打开后不再让用户先登录或走长引导，而是直接进入项目工作台。新项目用本地示例数据帮助用户理解流程；已有项目用 Git 历史 PR 生成上下文，让用户能马上接着真实项目继续提需求。**

---

## 3. 范围

### 3.1 本需求包含

```text
1. 去掉登录入口阻断
2. 去掉首次引导阻断
3. App 启动后直接进入项目选择 / 项目工作台
4. 新项目初始化 3-5 条本地示例数据
5. 已有项目提取 5-10 条 Git PR / merge / commit 上下文
6. 初始化 workspace / issue / delivery / audit / state 的可展示数据
7. 保持所有数据写入 .agentflow/**
8. 保持 AGENTS.md 本地生成和 .gitignore 处理
9. 前端根据真实数据 / 初始化数据展示工作台
```

### 3.2 本需求不包含

```text
1. 不做真实登录
2. 不接 OAuth
3. 不调用模型
4. 不自动执行 Codex
5. 不创建远程 PR
6. 不写用户源码
7. 不修改 Git 历史
8. 不做完整在线同步
9. 不做多人协作
```

---

## 4. Base Release 启动体验

### 4.1 当前要去掉的逻辑

Base Release 不再默认显示：

```text
LoginModal
FirstRunModal
Provider 选择窗口
首次引导 5 步流程
```

也不要让这些状态阻断进入 App：

```text
not-authenticated
first-run
providerConnected = false
onboardingComplete = false
```

### 4.2 Base Release 启动流程

新的启动流程：

```text
打开 App
  ↓
检查最近项目
  ↓
如果有最近项目：
  直接进入该项目工作台

如果没有最近项目：
  显示项目选择 / 添加项目入口
  用户选择本地项目
  准备 .agentflow 工作区
  进入工作台
```

### 4.3 登录能力保留方式

登录和首次引导相关代码不要强行删除到无法恢复。
建议：

```text
1. 暂时不在启动流程使用
2. 可以保留组件文件
3. 不作为默认路由
4. 不阻断 App 进入工作台
5. 后续版本再重新接入
```

---

## 5. 新项目初始化策略

### 5.1 新项目定义

满足以下条件时，视为新项目：

```text
1. 项目没有 .agentflow/input/issues
2. 或 .agentflow/input/issues 为空
3. 或 .agentflow/state/indexes/issue-status.json 不存在
4. 或没有可展示的 delivery / audit / recent activity
```

### 5.2 新项目初始化目标

用户第一次打开一个新项目时，不应该看到空白页面。

应该看到：

```text
1. 工作台有项目状态
2. 任务页有 3-5 条示例任务
3. 交付页有示例交付
4. 审计页有示例审计
5. 高级页能看到初始化状态
6. 底部状态栏显示项目已准备
```

### 5.3 新项目 mock 数据数量

初始化 3-5 条示例任务：

```text
建议 5 条，覆盖完整状态流：
1. Backlog
2. Ready
3. In Progress
4. Review
5. Done
```

如果只生成 3 条，至少覆盖：

```text
Ready
In Progress
Review
```

### 5.4 示例任务建议

```text
AF-DEMO-001
标题：整理第一个需求
状态：Backlog / 待办
说明：先把一个模糊想法整理成可以确认的需求。

AF-DEMO-002
标题：生成 Codex 任务包
状态：Ready / 就绪
说明：这个任务已经准备好，可以复制任务包交给 Codex。

AF-DEMO-003
标题：等待 Codex 写回结果
状态：In Progress / 进行中
说明：任务包已经交给 Codex，等待写回交付结果。

AF-DEMO-004
标题：核对交付证据
状态：Review / 待审阅
说明：Codex 已写回交付，需要检查验证结果和证据。

AF-DEMO-005
标题：完成一次审计
状态：Done / 已完成
说明：交付已经通过审计，可以作为完成样例。
```

### 5.5 示例任务字段

每条示例任务必须包含：

```text
issueId
title
summary
displayStatus
riskLevel
scope
nonGoals
acceptanceCriteria
validationHints
system.path
system.revision
```

### 5.6 示例交付

至少生成 1 条示例交付：

```text
DEL-DEMO-001
关联任务：AF-DEMO-004
状态：ready
说明：这是一次本地示例交付，用来展示交付页如何阅读交付包。
```

内容包括：

```text
交付摘要
关联任务
验证结果
证据列表
变更文件摘要
```

### 5.7 示例审计

至少生成 1 条示例审计：

```text
AUD-DEMO-001
关联交付：DEL-DEMO-001
状态：passed-with-warnings
说明：这是一次示例审计，用来展示审计报告、发现项和证据链。
```

内容包括：

```text
审计结论
发现项
证据链
处理建议
```

### 5.8 示例数据标记

所有 mock / demo 数据必须带标记：

```json
{
  "source": "agentflow-demo",
  "demo": true
}
```

或者在 system 字段里写：

```json
{
  "createdBy": "agentflow-demo"
}
```

这样后续用户真实数据产生后，可以：

```text
1. 隐藏 demo 数据
2. 删除 demo 数据
3. 不把 demo 数据误认为真实交付
```

---

## 6. 已有项目初始化策略

### 6.1 已有项目定义

满足以下条件时，视为已有项目：

```text
1. 项目是 Git 仓库
2. 有 commit 历史
3. 有 merge commit / PR merge 记录 / GitHub remote
4. 或已存在 .agentflow/**
```

### 6.2 已有项目目标

打开已有项目时，AgentFlow 不应该只显示空白。
它应该先帮助用户理解：

```text
这个项目最近在做什么？
最近合并了哪些功能？
有哪些可以承接的新需求？
哪些模块可能需要继续迭代？
```

### 6.3 Git PR 上下文来源

优先级：

```text
1. GitHub PR API（如果已有 remote 且可访问）
2. Git log 中的 merge commit
3. Git log 中的 conventional commit / commit message
4. 本地 .agentflow 历史数据
```

### 6.4 需要获取的数据数量

获取 5-10 条最近上下文。

优先：

```text
最近 5-10 个已合并 PR
```

如果没有 PR：

```text
最近 5-10 个 merge commit
```

如果没有 merge commit：

```text
最近 5-10 个普通 commit
```

### 6.5 PR 上下文字段

每条上下文记录包含：

```text
prNumber
title
summary
mergedAt
author
changedFiles
labels
relatedCommitSha
sourceUrl
```

如果来自 commit，则包含：

```text
commitSha
title
summary
committedAt
author
changedFiles
```

### 6.6 生成项目上下文

将 5-10 条 PR / commit 转成用户能看懂的上下文：

```text
最近完成了什么
涉及哪些模块
可能还能继续做什么
是否适合作为下一条需求
```

示例：

```text
最近合并：docs: add human-agent guided experience design
影响范围：docs/requirements
可承接需求：把这份设计落成前端交互页面

最近合并：feat: workflow state gate orchestration
影响范围：state / desktop status
可承接需求：让工作台显示真实 next action 和 blockers
```

### 6.7 写入位置

建议写入：

```text
.agentflow/input/intake/git-context.json
.agentflow/input/projects/context-prs.json
.agentflow/state/indexes/recent-project-context.json
```

如果当前结构不支持这些路径，可以先写入：

```text
.agentflow/input/intake/git-context.json
```

并在 manifest / index 中登记。

### 6.8 不允许

```text
不修改 Git 历史
不创建 GitHub Issue
不评论 PR
不修改远程 PR
不把 PR 数据提交到仓库
```

---

## 7. 工作台展示规则

### 7.1 新项目工作台

新项目工作台展示：

```text
项目已准备好
这是一个新项目
AgentFlow 已生成示例任务，帮助你体验流程
```

展示模块：

```text
项目状态
当前任务：AF-DEMO-003 等待 Codex 写回结果
最近活动：示例任务 / 示例交付 / 示例审计
```

### 7.2 已有项目工作台

已有项目工作台展示：

```text
项目已准备好
已读取最近项目上下文
发现 5-10 条最近 PR / commit
```

展示模块：

```text
项目状态
当前建议：从最近 PR 中选择一个需求继续
最近活动：最近 PR / commit 摘要
```

### 7.3 普通用户文案

不要写：

```text
读取 workflow.json
生成 state index
load git context
```

要写：

```text
已读取最近项目记录
已准备好任务上下文
可以开始整理下一条需求
```

---

## 8. 任务页展示规则

### 8.1 新项目

任务页显示 demo 任务：

```text
待办
就绪
进行中
待审阅
已完成
```

每条任务展示：

```text
任务标题
状态
风险
一句话说明
```

### 8.2 已有项目

任务页根据 PR / commit 上下文生成建议任务草稿：

```text
从最近 PR 承接需求
补充测试
补齐交付证据
统一页面体验
修复状态显示
```

这些任务可以是：

```text
context suggestion
```

不要直接当作 approved Issue，除非用户确认。

### 8.3 关键规则

```text
PR 上下文生成的是“建议”，不是正式 Issue。
正式开发仍需要用户确认 SPEC / Issue。
```

---

## 9. 交付页展示规则

### 9.1 新项目

显示示例交付：

```text
这是示例交付，用来展示 Codex 写回结果会如何出现在这里。
```

### 9.2 已有项目

如果有真实 output：

```text
展示真实 Delivery
```

如果没有真实 output：

```text
展示空态：
还没有交付结果。Codex 写回后会显示在这里。
```

不要用 PR context 伪装成 Delivery。

---

## 10. 审计页展示规则

### 10.1 新项目

显示示例审计：

```text
这是示例审计，用来展示人工审计报告会如何呈现。
```

### 10.2 已有项目

如果有真实 audit：

```text
展示真实 Audit
```

如果没有：

```text
显示空态：
还没有审计报告。交付完成后可以请求审计。
```

不要把 PR context 当成 Audit。

---

## 11. 高级页展示规则

高级页展示：

```text
初始化类型
demo 数据是否存在
Git 上下文是否读取成功
state / panel / input / output 当前状态
```

可以显示 raw JSON，但必须标注：

```text
只读诊断
```

---

## 12. AGENTS.md 与 Git 忽略

Base Release 必须继续保证：

```text
1. 根目录 AGENTS.md 自动生成
2. .gitignore 自动加入 AGENTS.md
3. AGENTS.md 不被 Git 跟踪
4. 如果 AGENTS.md 已被跟踪，给出明确提示，不自动 rm --cached
```

提示文案：

```text
AGENTS.md 已生成，并已加入 .gitignore。
如果它之前已经被 Git 跟踪，请手动执行：
git rm --cached AGENTS.md
```

---

## 13. Mock / Demo 数据边界

### 13.1 允许

```text
Browser Preview 使用 mock
新项目首次体验使用 demo 数据
测试 fixture 使用 demo 数据
```

### 13.2 不允许

```text
真实已有项目 silent fallback 到 mock
真实 Delivery 缺失时用 demo Delivery 假装真实
真实 Audit 缺失时用 demo Audit 假装真实
```

### 13.3 标识要求

所有 demo 数据必须可识别：

```text
demo = true
source = agentflow-demo
createdBy = agentflow-demo
```

---

## 14. 数据刷新规则

触发初始化 / 刷新：

```text
打开项目
切换项目
点击刷新
首次准备项目
检测到 .agentflow 缺失
检测到 input/issues 为空
```

不触发初始化：

```text
查看文件
打开高级页
切换 Light / Dark
复制任务包
```

---

## 15. 建议实现位置

### 15.1 后端 / Rust

建议新增或扩展：

```text
crates/input
crates/state
crates/output
crates/workflow-acceptance
apps/desktop/src-tauri commands
```

可能新增 command：

```rust
initialize_base_release_project(project_root)
load_project_initialization_status(project_root)
load_recent_git_context(project_root)
```

### 15.2 前端

建议新增：

```text
apps/desktop/src/features/initialization
apps/desktop/src/interaction/initializationViewModel.ts
```

ViewModel：

```ts
type ProjectInitializationState = {
  projectKind: "new" | "existing";
  initialized: boolean;
  demoDataCreated: boolean;
  gitContextLoaded: boolean;
  recentContextCount: number;
  message: string;
}
```

---

## 16. Base Release 验收标准

必须满足：

```text
1. 打开 App 不再被登录阻断。
2. 打开 App 不再强制走首次引导。
3. 没有最近项目时，能选择本地项目。
4. 新项目打开后生成 3-5 条 demo 任务。
5. 新项目能看到示例交付和示例审计。
6. 已有 Git 项目能读取 5-10 条最近 PR / commit 上下文。
7. PR / commit 上下文不被当成正式 Issue。
8. 真实已有项目没有 Delivery 时，不展示假 Delivery。
9. 真实已有项目没有 Audit 时，不展示假 Audit。
10. AGENTS.md 自动加入 .gitignore。
11. 真实客户端不能 silent fallback 到 browser mock。
12. Browser Preview 仍可使用 mock。
13. cargo check 通过。
14. npm build 通过。
```

---

## 17. Codex 实现指令

```text
你现在只做这个任务：AgentFlow Base Release Initialization V1。

背景：
当前 AgentFlow 准备切 v0.1.0-base。base 前需要去掉登录/首次引导阻断，并为新项目和已有项目提供可用初始化体验。

目标：
1. App 启动不再强制登录。
2. App 启动不再强制首次引导。
3. 新项目生成 3-5 条 demo 任务、示例交付、示例审计。
4. 已有 Git 项目读取 5-10 条最近 PR / commit 上下文。
5. 所有初始化数据写入 .agentflow/**
6. AGENTS.md 自动加入 .gitignore。
7. 不污染 Git。
8. 前端能展示初始化状态。

范围：
- apps/desktop/src/**
- apps/desktop/src-tauri/**
- crates/input/**
- crates/state/**
- crates/output/**
- docs/requirements/**
- 相关测试

禁止：
- 不要调用模型。
- 不要真实登录。
- 不要创建远程 PR。
- 不要修改 Git 历史。
- 不要写用户源码。
- 不要把 demo 数据当成真实 Delivery / Audit。
- 不要让真实客户端 fallback 到 browser mock。
- 不要删除登录/引导组件，只从默认启动流程移除。

步骤：
1. 阅读本需求文档。
2. 找到 App.tsx 中登录和首次引导阻断逻辑。
3. 改成 Base Release 默认直接进入项目工作台 / 项目选择。
4. 实现新项目 demo 数据初始化。
5. 实现已有项目 Git context 读取。
6. 确保 demo 数据有 demo/source 标记。
7. 确保 AGENTS.md 写入 .gitignore。
8. 前端展示初始化状态和上下文。
9. 增加测试或 fixture。
10. 跑 cargo check 和 npm build。

输出：
- 改了哪些文件
- Base Release 启动流程如何变化
- demo 数据写入哪里
- Git context 写入哪里
- AGENTS.md 如何处理
- 验证命令结果
```

---

## 18. 后续版本规则提示

Base Release 之后，后续需求必须走：

```text
需求
→ SPEC
→ Issue
→ Codex Handoff
→ Evidence
→ Delivery
→ Audit
```

不能再从聊天需求直接跳到代码修改。

`.agentflow/input` 是事实源。
`docs/requirements` 是归档。
`Figma` 是设计参考。
`GitHub PR` 是代码变更载体。
