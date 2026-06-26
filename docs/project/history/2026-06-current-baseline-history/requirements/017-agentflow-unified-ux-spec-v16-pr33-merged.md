# AgentFlow Unified UX Spec (V16 + PR#33 merged)

> 建议保存路径：`docs/requirements/017-agentflow-unified-ux-spec-v16-pr33-merged.md`
> 类型：前端开发需求 / UX 统一规范 / PR#33 冲突收口文档
> 状态：Ready for implementation
> 设计源：Figma AgentFlow v3
> Figma 地址：https://www.figma.com/design/d1yy8aPLVr4M45FPxt1Wsw/Agentflow-v3?node-id=129-2&t=GUBsGkt8feJZCwPu-1

---

## 1. 背景

AgentFlow 之前已经有 PR#33 里的产品体验设计文档，目标是描述 Human-Agent Guided Experience。
现在设计已经继续演进到 V16，并且统一放在 Figma 在线文件里：

```text
Figma:
https://www.figma.com/design/d1yy8aPLVr4M45FPxt1Wsw/Agentflow-v3?node-id=129-2&t=GUBsGkt8feJZCwPu-1
```

因此需要一份新的统一开发需求文档，解决 PR#33 与当前 V16 设计之间的冲突。

本需求的作用：

```text
1. 把 PR#33 里的体验设计收口为历史参考。
2. 把 V16 作为当前前端实现基线。
3. 明确页面、组件、状态、数据依赖、按钮行为和验收标准。
4. 给 Codex / 前端 Agent 一个可以直接执行的开发任务说明。
```

---

## 2. 设计基线

### 2.1 当前唯一设计源

后续前端实现以这个 Figma 文件为准：

```text
AgentFlow v3
https://www.figma.com/design/d1yy8aPLVr4M45FPxt1Wsw/Agentflow-v3?node-id=129-2&t=GUBsGkt8feJZCwPu-1
```

不要再以本地历史 Figma 包作为实现依据。
本地历史包只作为设计讨论记录。

### 2.2 V16 定稿方向

V16 的核心方向：

```text
登录模块：独立浮动窗口
首次引导：独立浮动窗口
项目工作台：原生桌面客户端风格
任务页：看板 / 列表，不默认显示固定 Inspector
文件页：文件树 + 只读 Reader，不默认显示固定 Inspector
交付页：Delivery / Evidence / Validation / Changed files
审计页：Findings / Evidence Map / Traceability
高级页：State / Panel / Input / Execute / Output / Audit / Settings
底部状态栏：所有项目页固定存在
Light / Dark：跟随系统主题
Companion Mode：保留，作为 Codex 旁边的窄窗口模式
```

---

## 3. PR#33 冲突处理原则

### 3.1 PR#33 的定位

PR#33 里的 Human-Agent Guided Experience 是早期产品体验设计，保留为参考资料。
当前开发不再直接按 PR#33 的页面结构实现。

### 3.2 冲突处理规则

| 冲突项 | PR#33 倾向 | V16 当前方案 | 最终决策 |
|---|---|---|---|
| 首次引导 | 较完整的叙事流程 | 独立浮动窗口，5 步流程 | 使用 V16，保留 PR#33 的引导意图 |
| Agent 展示 | 三个 Agent 卡片较重 | 登录后引导里轻量说明 | 使用 V16，Agent 不做主导航 |
| 项目首页 | 偏引导式首页 | 原生客户端工作台 | 使用 V16 |
| 任务页 | 有固定详情区倾向 | 看板/列表默认不显示固定 Inspector | 使用 V16 |
| 文件页 | 偏项目现场 + 详情 | 参考当前代码：文件树 + 只读 Reader | 使用 V16 |
| 状态栏 | PR#33 未完整定义 | V16 有固定底部状态栏 | 使用 V16 |
| 语言风格 | 需要人话化 | 已纳入 agentLocale + plain-work-style | 使用当前语言策略 |
| Companion | PR#33 未作为重点 | V16 保留为窄窗口模式 | 使用 V16 |

### 3.3 文档处理要求

需要新增本文件，并在需求索引里明确：

```text
PR#33 / docs/requirements/015-human-agent-guided-experience-v1.md
= historical design reference

docs/requirements/017-agentflow-unified-ux-spec-v16-pr33-merged.md
= current frontend implementation baseline
```

不要删除 PR#33 文档，避免丢失产品演进记录。

---

## 4. 产品定位

AgentFlow 不是：

```text
AI IDE
代码编辑器
完整 Agent 平台
Linear 替代品
自动开发平台
```

AgentFlow 是：

```text
本地 Agent 工作流控制面
Codex / Claude Code / DeepSeek 这类工具的配套编排面板
把用户想法变成 SPEC、Issue、任务包、交付证据和审计结果的工具
```

一句话：

```text
Codex 负责干活，AgentFlow 负责让 Codex 知道该干什么、不能干什么、做到什么算完成。
```

---

## 5. 总体信息架构

### 5.1 登录前

```text
App 启动
→ 登录浮窗
→ 选择入口：ChatGPT / Claude / DeepSeek
→ 登录成功
→ 首次引导浮窗
```

### 5.2 首次引导

```text
选择项目
→ 环境准备
→ 认识 Agent
→ 确认意图
→ 完成引导
→ 进入项目工作台
```

### 5.3 常规项目工作区

```text
项目工作台
├── 工作台
├── 任务
├── 文件
├── 交付
├── 审计
└── 高级
```

### 5.4 Companion Mode

```text
窄窗口 Companion
├── 当前项目
├── Issue Queue
├── 当前 Issue
├── Handoff
├── Writeback Check
└── Action Bar
```

---

## 6. 全局布局规范

### 6.1 标准项目窗口结构

```text
┌────────────────────────────────────────────┐
│ 顶部标题栏 / 当前项目 / 搜索 / 命令入口       │
├─────────────┬──────────────────┬───────────┤
│ 项目 Tree    │ 主内容区           │ 可选详情区 │
│             │                  │           │
│ my-web-app  │ 任务 / 文件 / 交付  │ 详情 / 动作│
│  工作台      │                  │           │
│  任务        │                  │           │
│  文件        │                  │           │
│  交付        │                  │           │
│  审计        │                  │           │
│  高级        │                  │           │
├─────────────┴──────────────────┴───────────┤
│ 底部状态栏：环境状态 / 工作流状态 / 工具状态  │
└────────────────────────────────────────────┘
```

### 6.2 固定区

所有项目页必须有：

```text
Titlebar
ProjectTree
Toolbar
Main Content
StatusBar
```

按页面需要决定是否显示：

```text
Inspector
Detail Drawer
Inline Detail Pane
```

### 6.3 页面默认 Inspector 规则

```text
工作台：可以显示 Inspector，用于下一步动作
任务看板：默认不显示固定 Inspector
任务列表：默认不显示固定 Inspector，点击行后右侧显示任务详情
文件页：不显示固定 Inspector
交付页：列表 + 详情区，不用固定 Inspector
审计页：列表 + 报告详情区，不用固定 Inspector
高级页：分类 + 状态列表 + JSON Reader，不用固定 Inspector
```

---

## 7. 登录模块

### 7.1 目标

登录模块是独立浮动窗口，不属于项目页面。

### 7.2 支持入口

```text
ChatGPT
Claude
DeepSeek
```

### 7.3 页面内容

登录浮窗只显示：

```text
AgentFlow
连接大模型入口
ChatGPT
Claude
DeepSeek
登录说明
```

不要显示：

```text
Project Tree
工作台
任务
文件
交付
审计
高级
```

### 7.4 登录状态展示

登录成功后，常规页面只在顶部标题栏显示当前入口：

```text
ChatGPT connected
Claude connected
DeepSeek connected
```

不要在以下区域重复展示登录信息：

```text
Sidebar
Inspector
任务卡片
文件详情
交付详情
审计详情
Companion 主内容区
```

### 7.5 不做事项

V16 不做：

```text
多登录会话历史
多账号同时在线状态
复杂权限系统
远程分析设置页
```

---

## 8. 首次引导模块

### 8.1 流程

```text
选择项目
→ 环境准备
→ 认识 Agent
→ 确认意图
→ 完成引导
```

### 8.2 Step 1：选择项目

页面只保留一个主标题：

```text
选择项目
```

说明：

```text
打开一个本地项目，AgentFlow 会准备工作规则和项目现场。
```

按钮：

```text
打开本地项目
下一步
```

规则：

```text
未选择项目：下一步 disabled
已选择项目：下一步 enabled
```

不要出现重复标题：

```text
打开一个本地项目
打开一个本地项目
```

### 8.3 Step 2：环境准备

展示：

```text
当前项目名
当前项目路径
准备进度条
准备步骤列表
```

准备步骤：

```text
检测项目结构
创建 Agent 工作规则
创建 .agentflow 目录结构
读取项目现场
初始化项目状态
验证环境
```

按钮：

```text
上一步
下一步
```

环境未完成时：

```text
下一步 disabled
```

### 8.4 Step 3：认识 Agent

标题：

```text
认识 Agent
```

内容：

```text
Spec Agent
= 确认需求 / SPEC / Issue

Build Agent
= 任务包 / 执行 / 写回

Audit Agent
= 审计 / Evidence / Report
```

按钮：

```text
上一步
下一步
```

### 8.5 Step 4：确认意图

意图入口：

```text
我要开发 APP
我要重构代码
我要新增功能
我要修复 BUG
我要理解项目
```

包含：

```text
给 ChatGPT / Codex 的启动说明
复制说明
```

按钮：

```text
上一步
下一步
```

### 8.6 Step 5：完成引导

只保留一个主按钮：

```text
进入工作台
```

删除：

```text
打开 Codex
上一步
其他多余按钮
```

推荐内容：

```text
完成引导
一切准备就绪
以后打开 App，会直接进入项目工作台。
进入工作台
```

---

## 9. 工作台页面

### 9.1 目标

工作台回答三个问题：

```text
项目现在怎么样？
现在最该处理什么？
项目现场是否正常？
```

### 9.2 Toolbar

只显示：

```text
工作台
刷新图标
```

不要显示：

```text
扫描按钮
workspace.home
```

内部术语可以进底部状态栏或高级页。

### 9.3 主内容

工作台主内容分三块：

#### A. 当前下一步

数据来源：

```text
state/next-actions
state/blockers
state/gates
```

展示：

```text
当前建议动作
原因
关联对象
阻断原因
主按钮
```

#### B. 今日待处理

数据来源：

```text
input/issues
output/delivery
output/audit
state/next-actions
```

展示：

```text
待确认
可交给 Codex
等待写回
待审计
```

#### C. 项目现场摘要

数据来源：

```text
panel/output/manifest.json
panel/output/git.json
panel/output/diagnostics.json
panel/output/tests.json
panel/output/languages.json
```

展示：

```text
文件数
语言数
Git 状态
Diagnostics
Tests
Context Pack
最近索引时间
```

### 9.4 工作台 Inspector

工作台可保留 Inspector。

默认展示：

```text
下一步详情
```

动作根据状态变化：

```text
需求未确认 → 去确认需求
可交给 Codex → 复制任务包
等待写回 → 检查写回
交付可审计 → 请求审计
无阻断 → 查看任务
```

---

## 10. 任务页面

### 10.1 目标

任务页是 Project / Issues 编排主页面。

### 10.2 Toolbar

只保留：

```text
任务
看板 / 列表切换
搜索
刷新图标
```

删除：

```text
新建 Issue
筛选
issues.board
issues.list
```

### 10.3 看板模式

默认不显示固定 Inspector。

看板列：

```text
待确认
可交给 Codex
等待写回
待审计
已完成
```

点击 Issue 后：

```text
打开右侧临时详情抽屉
或进入任务详情页
```

### 10.4 列表模式

默认不显示固定 Inspector。

布局：

```text
左侧：任务列表
右侧：选中任务详情
```

列表字段：

```text
Issue
状态
Agent
风险
更新时间
动作
```

### 10.5 任务详情

展示：

```text
Issue 标题
状态
Agent
风险
来源 SPEC
范围
非目标
验收标准
相关文件
验证命令
证据要求
```

动作：

```text
确认需求
复制任务包
我已交给 Codex
检查写回
请求审计
```

动作区固定在详情区底部。

---

## 11. 文件页面

### 11.1 目标

文件页是只读项目现场。

### 11.2 布局

参考当前代码模块：

```text
ProjectLocalFilesPage
ProjectFileBrowser
ProjectFileReader
```

页面结构：

```text
左侧：文件树 / 搜索 / 全部 / 源码 / 最近
右侧：只读文件阅读器
```

不显示固定 Inspector。

### 11.3 内容

文件浏览器展示：

```text
搜索文件
全部 / 源码 / 最近
文件树
Git 状态标记
Diagnostics 标记
```

文件阅读器展示：

```text
文件名
路径
read-only
modified
代码内容
行号
```

### 11.4 不做事项

文件页不做：

```text
代码编辑
新建文件
删除文件
重命名文件
Git stage / commit
```

---

## 12. 交付页面

### 12.1 目标

交付页用于查看 Codex 写回结果。

回答：

```text
Codex 写回了什么？
有没有 evidence？
验证有没有通过？
哪些文件改了？
是否可以审计？
```

### 12.2 数据来源

```text
output/evidence
output/release
output/delivery
execute/runs
execute/results
panel/output/git.json
panel/output/tests.json
panel/output/diagnostics.json
```

### 12.3 页面结构

```text
左侧：交付列表
右侧：交付详情
底部：状态栏
```

### 12.4 交付列表字段

```text
Delivery ID
关联 Issue
关联 SPEC
状态
执行 Agent
验证状态
Evidence 数量
更新时间
```

### 12.5 交付详情

展示：

```text
交付摘要
Changed files
Validation commands
Validation result
Evidence files
Release note
Missing evidence
Out-of-scope check
```

动作：

```text
查看证据
请求审计
补证据
重新检查写回
```

---

## 13. 审计页面

### 13.1 目标

审计页用于检查交付是否符合 SPEC 和 Issue。

回答：

```text
交付是否符合需求？
有没有越界？
证据是否完整？
是否需要返工？
```

### 13.2 数据来源

```text
output/audit
output/human-audit
output/evidence
output/release
input/spec
input/issues
execute/runs
state/audit
```

### 13.3 页面结构

```text
左侧：审计列表
右侧：审计报告详情
底部：状态栏
```

### 13.4 审计列表字段

```text
Audit ID
关联 Delivery
关联 Issue
状态
风险等级
审计人 / 审计类型
更新时间
```

状态：

```text
未请求
进行中
通过
通过但有警告
返工
补证据
```

### 13.5 审计详情

展示：

```text
审计结论
Findings
Evidence Map
Traceability
Scope check
Validation check
Risk summary
建议动作
```

Traceability：

```text
SPEC → Issue → Execute Run → Evidence → Delivery → Audit Report
```

动作：

```text
接受
返工
补证据
查看证据
```

---

## 14. 高级页面

### 14.1 目标

高级页给开发者 / 高级用户 / 调试使用。

回答：

```text
系统内部状态是什么？
索引是否正常？
Gate 为什么阻断？
哪些文件记录在 .agentflow？
```

### 14.2 数据来源

```text
state/*
state/gates
state/blockers
state/locks
state/sessions
state/indexes
panel/output/manifest.json
panel/output/file-tree.json
panel/output/languages.json
panel/output/symbols.json
panel/output/relations.json
panel/output/diagnostics.json
panel/output/git.json
panel/output/tests.json
input/spec
input/issues
execute/runs
output/index
```

### 14.3 页面结构

```text
左侧：高级分类
中间：状态文件 / 索引列表
右侧：JSON / 详情阅读器
底部：状态栏
```

分类：

```text
状态
Panel
Input
Execute
Output
Audit
设置
```

---

## 15. Companion Mode

### 15.1 目标

Companion Mode 是 Codex 旁边的窄窗口。

### 15.2 布局

```text
顶部：项目 + 工作流状态
中间：Issue Queue
下方：当前 Issue
底部：Action Bar
```

### 15.3 宽度

```text
420px = 最小
520px = 推荐
720px = 展开
960px+ = Full Workspace
```

### 15.4 不显示

Companion 默认不显示：

```text
完整 Project Tree
完整高级页
完整交付详情
完整审计详情
```

---

## 16. 底部状态栏

所有项目页面都有底部状态栏。

结构：

```text
左侧：环境状态
中间：工作流状态
右侧：工具状态
```

示例：

```text
● ready  my-web-app  main  dirty:8
waiting-for-codex ISSUE-204 last-scan:2m
ChatGPT  Full  local-only  ⌘K
```

页面差异：

```text
工作台：项目整体状态
任务：当前任务流状态
文件：当前文件 / read-only / git 状态
交付：delivery / validation / evidence 状态
审计：audit / risk / evidence map 状态
高级：raw state / index / manifest 状态
```

---

## 17. 组件清单

必须拆组件：

```text
AppShell
TitleBar
ProjectTree
Toolbar
StatusBar
InspectorPanel
ActionBar
StatePill
StatusDot
TerminalRow
CompactTable
TaskBoard
TaskList
TaskDetail
FileBrowser
FileReader
DeliveryList
DeliveryDetail
AuditList
AuditReport
AdvancedStateViewer
JsonReader
LoginModal
FirstRunModal
CompanionShell
```

---

## 18. ViewModel 建议

前端不要让页面直接读零散 JSON。
建议做 ViewModel 层：

```text
WorkspaceHomeViewModel
TaskBoardViewModel
TaskListViewModel
TaskDetailViewModel
FilePanelViewModel
DeliveryViewModel
AuditViewModel
AdvancedStateViewModel
StatusBarViewModel
NextActionViewModel
CompanionViewModel
```

---

## 19. 按钮真实语义

```text
复制任务包
= 只复制到剪贴板

我已交给 Codex
= 用户确认已粘贴后，本地改状态

检查写回
= 扫描 .agentflow/output 和 .agentflow/execute

请求审计
= 本地生成 audit package

进入工作台
= 完成首次引导并进入项目工作台
```

不要让按钮表现成：

```text
自动控制 Codex
自动执行代码
自动修复问题
```

---

## 20. 验收标准

必须满足：

```text
1. 首次引导没有重复标题。
2. 首次引导按钮不再错位。
3. 完成页只保留“进入工作台”。
4. 工作台顶部只保留刷新图标。
5. 任务看板默认不显示固定 Inspector。
6. 任务列表默认不显示固定 Inspector。
7. 文件页不显示固定 Inspector。
8. 任务列表点击后右侧显示任务详情。
9. 文件页是文件树 + 只读 Reader。
10. 交付页展示 delivery / evidence / validation / changed files。
11. 审计页展示 findings / evidence map / traceability。
12. 高级页展示 state / panel / input / execute / output / audit / settings。
13. 所有项目页面都有底部状态栏。
14. Light / Dark 页面结构一致。
15. 所有按钮和文字对齐，不出现明显错位。
```

---

## 21. 不做事项

V16 不做：

```text
不新增模型调用能力
不自动执行 Codex
不做完整 IDE 编辑器
不做 Git commit / push / stage
不做多账号会话历史
不做复杂权限管理
不在普通页面暴露 raw JSON
不删除 PR#33 文档
```

高级页可以展示 raw JSON。

---

## 22. 开发阶段

建议按阶段推进：

```text
Phase 1：AppShell + Design Tokens
Phase 2：登录 + 首次引导
Phase 3：工作台
Phase 4：任务页
Phase 5：文件页
Phase 6：交付 / 审计 / 高级
Phase 7：Companion Mode
```

---

## 23. Codex 实现指令

```text
你现在只做这个任务：实现 AgentFlow Unified UX Spec V16 的前端基础结构。

背景：
PR#33 中已有早期体验设计，但当前实现以 Figma AgentFlow v3 和本需求为准。

目标：
先实现 AppShell + 页面骨架，不要一次性实现所有业务逻辑。

范围：
- 只改 apps/desktop 前端相关文件
- 新增必要组件
- 不改 Rust 后端
- 不新增模型调用能力
- 不自动执行 Codex

步骤：
1. 阅读本需求文档。
2. 阅读当前 apps/desktop/src/App.tsx。
3. 找到已有 status-channel、agent-manual、project-files、output、state 模块。
4. 建立 AppShell / TitleBar / ProjectTree / Toolbar / StatusBar 基础组件。
5. 把左侧菜单固定为：
   工作台 / 任务 / 文件 / 交付 / 审计 / 高级
6. 工作台先接入已有状态和 mock 数据。
7. 文件页沿用当前 ProjectLocalFilesPage / ProjectFileBrowser / ProjectFileReader 逻辑。
8. 不实现自动 Codex 执行。
9. 保证 TypeScript build 通过。

禁止：
- 不要顺手重构无关业务
- 不要删除 PR#33 文档
- 不要把内部 raw JSON 放到普通页面
- 不要把任务看板和文件页固定 Inspector 做回来
- 不要新增后端能力

验证：
- npm / pnpm build 通过
- 前端 preview 能看到：
  登录
  首次引导
  工作台
  任务
  文件
  交付
  审计
  高级
- Light / Dark 基础 token 可用
- 底部状态栏存在
```

---

## 24. 需要更新的文档索引

实现本需求时，更新：

```text
docs/requirements/README.md
docs/requirements/next-requirements.md
```

加入：

```text
017-agentflow-unified-ux-spec-v16-pr33-merged.md
```

并标记：

```text
015-human-agent-guided-experience-v1.md = historical reference
017-agentflow-unified-ux-spec-v16-pr33-merged.md = current implementation baseline
```
