# AgentFlow GitHub Code × Latest SVG Page Repair Requirement V1

> 建议保存路径：`docs/requirements/023-agentflow-github-code-latest-svg-page-repair-v1.md`  
> 类型：前端 / 后端数据模型 / 页面 UX 修复需求  
> 状态：Ready for Codex implementation  
> 设计源：用户上传的 `figma-agentflow-v3-svg.zip`  
> 代码源：GitHub `atxinbao/agentflow` 当前代码  
> 目标：逐页对比最新 SVG 与 GitHub 当前实现，修复产品体验和技术问题。

---

## 1. 背景

当前 AgentFlow 的前端页面和后端能力已经都上了 GitHub。  
用户上传了最新 Figma SVG 设计包。现在需要做一轮 **代码 × 页面 × 产品体验** 的对齐。

本需求不再讨论 Figma 风格是否正确。  
本需求只做：

```text
1. 对照当前 GitHub 代码，找出和最新 SVG 页面不一致的问题。
2. 按页面逐个列出产品问题和技术问题。
3. 给 Codex 一份可执行的修复需求。
```

---

## 2. 设计源和代码源

### 2.1 设计源

```text
figma-agentflow-v3-svg.zip
```

核心页面：

```text
03_workspace_home_light / dark
04_tasks_board_light / dark
07_file_panel_light / dark
08_delivery_light / dark
09_audit_light / dark
10_advanced_light / dark
11_companion_light / dark
12_tokens_light / dark
```

### 2.2 代码源

重点检查：

```text
apps/desktop/src/App.tsx
apps/desktop/src/AppShell.css
apps/desktop/src/interaction/viewModels.ts
apps/desktop/src/types/status.ts
apps/desktop/src/browserPreviewData.ts
apps/desktop/src/features/project-files/ProjectLocalFilesPage.tsx
apps/desktop/src/features/project-files/**
crates/input/src/issue.rs
crates/state/src/indexes.rs
```

---

## 3. 总体结论

当前代码已经不是空白状态，已经具备：

```text
登录 Modal
首次引导 Modal
AppShell
TitleBar
ProjectTree
Toolbar
StatusBar
工作台
任务列表 + 任务详情
文件页
交付页
审计页
高级页
Companion
ViewModel 基础
IssueDisplayStatus 前端类型
```

但还有几个关键问题：

```text
P0：后端 input issue 模型和 state index 代码存在 displayStatus 不一致风险。
P1：工作台页面还偏旧结构，没有完全按最新 SVG 的“项目状态 / 当前任务 / 最近活动”填内容。
P1：任务页已接近最新 SVG，但文案、任务合约区、动作区还需要收口。
P1：文件页还有空壳 FileBrowser / FileReader 组件。
P1：交付 / 审计页面已经有列表和详情，但用户可见内容仍偏技术字段，需要转成用户能理解的交付包 / 审计报告。
P1：高级页混入 DesignSystemPreview，不应该出现在产品页面。
P2：Companion 目前嵌在工作台里，应该作为独立窄窗口模式或响应式模式处理。
```

---

## 4. P0：后端 DisplayStatus 模型不一致

### 4.1 问题

前端已经定义了：

```ts
IssueDisplayStatus =
  | "backlog"
  | "ready"
  | "in-progress"
  | "review"
  | "done"
  | "cancel"
```

`browserPreviewData.ts` 也已经在 mock issue 里写入 `displayStatus`。

`crates/state/src/indexes.rs` 已经导入：

```rust
use agentflow_input::issue::{DisplayStatus, InputIssue, InputIssueStatus};
```

并且通过 `display_status(...) -> DisplayStatus` 生成 `issue-status.json`。

但当前 `crates/input/src/issue.rs` 里只看到：

```rust
InputIssueStatus {
  Planned,
  Blocked,
  ReadyForExecute,
  Done,
  Canceled,
}
```

没有完整看到 `DisplayStatus` 和 `InputIssue.display_status` 字段。

这会造成：

```text
1. Rust 编译可能失败。
2. 后端 issue-status index 无法稳定生成 displayStatus。
3. 前端任务页依赖 displayStatus，但真实数据可能没有。
```

### 4.2 修复要求

在 `crates/input/src/issue.rs` 中补齐：

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DisplayStatus {
    Backlog,
    Ready,
    InProgress,
    Review,
    Done,
    Cancel,
}

impl Default for DisplayStatus {
    fn default() -> Self {
        Self::Backlog
    }
}

impl DisplayStatus {
    pub fn from_input_status(status: &InputIssueStatus) -> Self {
        match status {
            InputIssueStatus::Planned => Self::Backlog,
            InputIssueStatus::Blocked => Self::Backlog,
            InputIssueStatus::ReadyForExecute => Self::Ready,
            InputIssueStatus::Done => Self::Done,
            InputIssueStatus::Canceled => Self::Cancel,
        }
    }
}
```

并在 `InputIssue` 中加入：

```rust
pub display_status: DisplayStatus,
```

默认值：

```rust
display_status: DisplayStatus::default(),
```

### 4.3 兼容要求

已有旧 issue JSON 可能没有 `displayStatus`。  
必须保证旧文件可读。

建议：

```rust
#[serde(default)]
pub display_status: DisplayStatus,
```

### 4.4 Index 生成规则

`crates/state/src/indexes.rs` 仍然可以根据 execute / output / audit 派生真实 display status。

但写入 `.agentflow/state/indexes/issue-status.json` 时必须稳定输出：

```text
backlog
ready
in-progress
review
done
cancel
```

### 4.5 测试要求

新增测试：

```text
1. 老 issue json 缺 displayStatus 也能 parse。
2. DisplayStatus 序列化为 kebab-case。
3. State issue-status index 能生成 displayStatus。
4. displayStatus 推导覆盖：
   planned → backlog
   ready-for-execute → ready
   execute running → in-progress
   delivery/evidence ready → review
   audit passed → done
   canceled → cancel
```

---

## 5. App Shell 与全局结构修复

### 5.1 当前情况

当前 `App.tsx` 已经有：

```text
activePage
ProjectTree
Toolbar
StatusBar
Inspector home-only
LoginModal
FirstRunModal
```

页面枚举：

```text
home
tasks
files
delivery
audit
advanced
```

### 5.2 问题

当前 AppShell 基本可用，但要明确：

```text
1. 任务页、文件页、交付页、审计页、高级页默认不使用固定 Inspector。
2. 工作台是否显示 Inspector 要按最新 SVG 决定，不能再塞旧版 NextStepCard + 通用 Inspector。
3. Toolbar 只保留页面标题、任务搜索、刷新图标。
4. 不要恢复“扫描”“新建 Issue”“筛选”等按钮。
```

### 5.3 修复要求

保持：

```tsx
inspector={activePage === "home" ? <InspectorPanel ... /> : null}
```

但如果最新 SVG 工作台没有右侧 Inspector，则改为：

```tsx
inspector={null}
```

如果保留工作台 Inspector，则其内容必须只展示：

```text
下一步详情
当前任务
阻断原因
主动作
```

不要展示技术 JSON。

---

## 6. 登录页面修复

### 6.1 当前情况

已有：

```text
LoginModal
ChatGPT / Claude / DeepSeek
未登录状态
not-authenticated
```

### 6.2 修复要求

登录页保持独立，不显示项目内容。

需要补齐用户说明：

```text
连接大模型入口
选择你将用来配合 AgentFlow 的入口。
登录是独立模块，不展示项目内容；完成后进入首次引导。
```

### 6.3 不做

```text
不做真实 OAuth
不做多账号历史
不在侧栏显示登录信息
```

---

## 7. 首次引导修复

### 7.1 当前情况

当前已有 5 步：

```text
选择项目
环境准备
认识智能体
确认意图
完成引导
```

### 7.2 问题

文案需要统一：

```text
认识智能体 → 认识 Agent
需求助手 value 里“整理计划”不如“整理规格”
执行助手 value 混用了 · 和 /
审计助手 value 仍有 slash 风格
```

### 7.3 修复要求

步骤名：

```text
选择项目
环境准备
认识 Agent
确认意图
完成引导
```

Agent 卡片：

```text
需求助手
确认需求 · 整理规格 · 生成任务

执行助手
任务打包 · 执行改动 · 写回结果

审计助手
审计交付 · 核对证据 · 生成报告
```

完成页只保留：

```text
进入工作台
```

不要有：

```text
打开 Codex
上一步
```

---

## 8. 工作台页面修复

### 8.1 最新 SVG 结构

最新工作台不是旧版的 MetricCard + NextStepCard 页面。  
最新结构是三块主体：

```text
项目状态
当前任务
最近活动
```

### 8.2 当前代码问题

当前 `ProjectHomePage` 仍然包括：

```text
PageHeader: 项目工作台
NextStepCard
MetricCard: 待确认 / 可交给 Codex / 等待写回 / 待审计
当前任务和最近活动 Panel
项目现场摘要 Panel
CompanionShell
```

这和最新 SVG 有差异。

### 8.3 产品修复目标

工作台一眼回答：

```text
项目是否可用？
当前任务是什么？
最近发生了什么？
```

### 8.4 修复后的页面结构

```text
左列：项目状态
中列：当前任务
右列：最近活动
```

### 8.5 项目状态列

展示：

```text
项目
my-web-app
等待 Codex 写回
就绪

工作台 Shell
本地只读客户端
已就绪

项目文件
浏览器预览 / 客户端真实读取
只读
```

数据来源：

```text
stateStatusState
projectPanelState
projectFilesState
projectRoot
connectedProvider
```

不要直接展示：

```text
define ready
panel ready
input ready
manifest.json
workflow.json
```

### 8.6 当前任务列

展示：

```text
AF-104 / ISSUE ID
任务标题
风险
状态
```

数据来源：

```text
selectedTask
stateStatusState.status.activeIssueId
issueStatusIndexState.index
inputSnapshotState.snapshot.issues
```

状态中文：

```text
backlog → 待办
ready → 就绪
in-progress → 进行中
review → 待审阅
done → 已完成
cancel → 已取消
```

点击当前任务：

```text
进入任务页并选中该任务
```

### 8.7 最近活动列

展示用户能懂的事件：

```text
任务页面压缩完成
审计页面同步结构
交付页面同步结构
高级页面清理
```

数据来源：

```text
state/events/timeline.jsonl
outputBundle.outputIndex
outputBundle.auditIndex
workspaceData.workbench?.projectUpdates
```

如果暂时没有 timeline API，可以先从 mock / output index 构造。

### 8.8 CompanionShell 处理

当前 `ProjectHomePage` 里直接渲染 `CompanionShell`。

最新 UX 里 Companion 是独立窄窗口模式，不应该长期塞在工作台主内容下面。

修复：

```text
1. 从工作台默认主内容中移除 CompanionShell。
2. 后续单独做 Companion mode 或 responsive narrow mode。
3. 如果暂时保留，必须放到折叠调试区，不作为工作台主体。
```

---

## 9. 任务页面修复

### 9.1 当前情况

当前任务页已经是：

```text
左侧任务队列
右侧任务详情
```

这基本符合最新 SVG。

### 9.2 问题

当前命名和内容仍偏旧：

```text
任务队列
任务详情
来源规格显示 projectId
CopyableCodeBlock 直接暴露完整任务包
ActionBar 中按钮可能太多
```

### 9.3 修复要求

标题：

```text
任务流转
任务合约
```

右侧标题：

```text
任务合约：<task-id>
```

不要叫：

```text
任务详情
```

### 9.4 任务列表字段

每行展示：

```text
任务 ID
任务标题
状态
风险
```

不要展示：

```text
sourceSpecId
projectId
system.path
```

### 9.5 任务合约内容

右侧内容分区：

```text
目标
范围
非目标
验收标准
证据要求
验证命令
相关文件
```

`Codex 任务包` 不要默认展开成大块代码。  
改为：

```text
折叠区：Codex 任务包
按钮：复制任务包
```

### 9.6 动作按钮

按状态显示：

```text
Backlog → 查看需求
Ready → 复制任务包
In Progress → 我已交给 Codex / 检查写回
Review → 查看交付 / 请求审计
Done → 查看交付 / 查看审计
Cancel → 只读查看
```

### 9.7 已有 ViewModel 可保留

当前 `viewModels.ts` 已有：

```text
TaskInteractionAction
taskActionsForStatus
displayStatusLabelZh
pickTaskId
```

这些保留，但需要配合后端 DisplayStatus 修复。

---

## 10. 文件页面修复

### 10.1 当前情况

`FilesPage` 当前代码里有：

```tsx
<FileBrowser />
<FileReader />
<ProjectLocalFilesPage ... />
```

但：

```tsx
function FileBrowser() { return null; }
function FileReader() { return null; }
```

这两个空壳会造成实现混乱。

### 10.2 当前底层正确组件

`ProjectLocalFilesPage` 已经包含：

```text
只读文件页提示
ProjectFileBrowser
ProjectFileReader
```

它已经是正确入口。

### 10.3 修复要求

删除空壳：

```tsx
<FileBrowser />
<FileReader />
```

保留：

```tsx
<ProjectLocalFilesPage ... />
```

### 10.4 最新 SVG 布局要求

最新 SVG 文件页是：

```text
左侧：只读文件 Reader
右侧：项目文件列表
```

而当前 `ProjectLocalFilesPage` JSX 顺序是：

```text
ProjectFileBrowser
ProjectFileReader
```

需要按最新 SVG 调整布局。

可选实现：

```text
方案 A：改 CSS grid，把 reader 放左，browser 放右
方案 B：调整 JSX 顺序，先 Reader 后 Browser
```

建议：

```text
优先用 CSS grid order，不破坏 ProjectLocalFilesPage 逻辑。
```

### 10.5 文件页内容

用户看到：

```text
只读文件页
只能读，不能改。
文件内容
项目文件列表
```

不允许：

```text
编辑
保存
删除
重命名
Git 操作
```

---

## 11. 交付页面修复

### 11.1 当前情况

当前已经有：

```text
DeliveryList
DeliveryDetail
证据数
验证命令数
变更文件数
缺失证据
请求审计
查看证据
```

### 11.2 问题

当前列表主要显示：

```text
runId
issueId
status
path
```

这对普通用户仍偏技术。

### 11.3 修复要求

列表显示：

```text
DEL-001 / run 简短 ID
任务标题
状态：就绪 / 待核对 / 待办
更新时间
```

详情显示：

```text
交付包：DEL-001
任务合约页面交付记录

状态
证据
模式：只读

交付摘要
关联记录
验证结果
```

### 11.4 数据映射

```text
runId → DEL 显示 ID
issueId → 关联任务
sourceSpecId → 不直接展示，改为“关联规格”
path → 不在普通页面展示，放高级页
```

### 11.5 按钮

```text
请求审计
查看证据
```

如果 `查看证据` 暂无真实跳转逻辑：

```text
要么实现跳转到交付详情证据区域
要么先禁用
不要放一个无效按钮
```

---

## 12. 审计页面修复

### 12.1 当前情况

当前审计页结构：

```text
AuditList
AuditReport
OutputAuditPanel
```

### 12.2 问题

最新 SVG 是：

```text
左侧：审计列表
右侧：审计报告
```

`OutputAuditPanel` 是请求审计入口，放在审计页下面会破坏页面结构。

### 12.3 修复要求

默认审计页只读：

```text
审计列表
审计报告详情
```

把 `OutputAuditPanel` 移除或折叠到：

```text
交付页的“请求审计”流程
或审计页空态里的“请求审计入口”
```

不要常驻在审计页底部。

### 12.4 报告内容

用户看到：

```text
审计结论
发现项
证据链
处理建议
当前版本限制
```

不要直接展示 raw `evidenceMap` / `traceability` JSON。

当前 `JsonSummary` 可以保留，但要渲染成人话表格或折叠区。

---

## 13. 高级页面修复

### 13.1 当前情况

当前高级页包含：

```tsx
<AdvancedStateViewer />
<DesignSystemPreview />
```

### 13.2 问题

`DesignSystemPreview` 是设计系统预览，不应该出现在真实产品高级页。

### 13.3 修复要求

删除：

```tsx
<DesignSystemPreview />
```

高级页结构必须为：

```text
左侧：分类
中间：状态文件列表
右侧：JSON Reader + 中文说明
```

### 13.4 分类

```text
状态
Panel
Input
Execute
Output
Audit
设置
```

### 13.5 状态文件列表

每个分类展示文件和中文说明：

```text
workflow.json
当前阶段与下一动作

gates.json
门禁检查结果

blockers.json
阻塞项快照

locks.json
本地锁状态

sessions.json
智能体会话记录

next-actions.json
下一步候选动作
```

### 13.6 JSON Reader

只读展示。

禁止：

```text
编辑 JSON
修复状态
清理锁
继续执行
触发审计
```

---

## 14. Companion 修复

### 14.1 当前情况

当前 `CompanionShell` 嵌在 `ProjectHomePage` 内。

### 14.2 问题

最新 SVG 有独立 Companion 页面。  
Companion 应该是窄窗口模式，不应该默认占工作台空间。

### 14.3 修复要求

短期：

```text
从工作台默认页面移除 CompanionShell。
```

中期：

```text
当窗口宽度小于某个阈值时，进入 Companion Mode。
```

或者增加：

```text
Companion 独立路由 / 独立 preview 页面
```

### 14.4 Companion 内容

保留：

```text
当前项目
工作流状态
今日队列
当前任务
检查写回
任务包
打开文件
```

---

## 15. Toolbar 修复

### 15.1 当前情况

Toolbar 当前：

```text
标题
任务页搜索
刷新图标
```

这方向是正确的。

### 15.2 修复要求

不要增加：

```text
扫描
新建 Issue
筛选
导出
复杂更多菜单
```

任务页搜索可保留。  
其他页面只保留刷新图标。

---

## 16. Browser Preview Mock 修复

### 16.1 当前情况

`browserPreviewData.ts` 已经有六个 displayStatus mock issue：

```text
backlog
ready
in-progress
review
done
cancel
```

这是正确的。

### 16.2 修复要求

保留这些 mock 数据。  
需要让它们真正覆盖任务页所有状态展示和按钮行为。

至少在浏览器预览里验证：

```text
每个状态都有任务
每个状态都有正确中文标签
每个状态对应按钮正确
```

---

## 17. 样式修复

### 17.1 当前问题

当前 CSS 已经有较多 `.v16-*` 样式。  
但页面结构继续变动后，必须避免：

```text
旧样式残留
空壳组件样式
工作台旧 MetricCard 大面积撑开
高级页 DesignSystemPreview 样式污染
```

### 17.2 修复要求

清理或确认不再使用的类：

```text
v16-panel-grid
v16-next-step-card
v16-current-work-panel
v16-companion-shell 在工作台中的默认布局
```

如果保留类，必须确认仍有页面使用且符合最新 SVG。

---

## 18. 验收标准

### 18.1 编译

```text
cargo check
npm --prefix apps/desktop run build
```

必须通过。

### 18.2 页面验收

```text
1. 登录页独立，不显示项目内容。
2. 首次引导文案统一，完成页只保留进入工作台。
3. 工作台为项目状态 / 当前任务 / 最近活动三列。
4. 工作台不再默认嵌入 Companion。
5. 任务页是任务流转 + 任务合约。
6. 任务包默认不大面积展开。
7. 文件页无空壳组件，无固定 Inspector。
8. 文件页按最新 SVG：Reader + 文件列表。
9. 交付页隐藏 raw path，展示用户友好的交付包。
10. 审计页不常驻 OutputAuditPanel。
11. 审计证据链是人话表格，不是 raw JSON。
12. 高级页删除 DesignSystemPreview。
13. 高级页有分类、文件列表、JSON Reader 和中文解释。
14. Companion 不是工作台默认主体内容。
15. Browser Preview 可覆盖六个任务状态。
```

---

## 19. 建议修复顺序

```text
P0-1：修复 DisplayStatus 后端模型不一致。
P0-2：跑 cargo check / npm build，确认基础可编译。

P1-1：修复工作台三列内容和移除 Companion 默认嵌入。
P1-2：修复任务页文案、任务合约和任务包折叠。
P1-3：修复文件页空壳和布局顺序。
P1-4：修复交付页用户化展示。
P1-5：修复审计页移除 OutputAuditPanel 常驻。
P1-6：修复高级页移除 DesignSystemPreview。

P2-1：清理旧样式。
P2-2：完善 Browser Preview mock 状态覆盖。
P2-3：补空态 / 错误态 / stale 态。
```

---

## 20. Codex 修复指令

```text
你现在只做这个任务：AgentFlow GitHub Code × Latest SVG Page Repair V1。

背景：
当前 GitHub 代码已经有前后端页面实现，但和用户上传的最新 SVG 还有差异。
你需要按本需求逐页修复，不重新设计 UI。

设计源：
- 用户上传 figma-agentflow-v3-svg.zip
- Figma Agentflow v3

目标：
1. 修复后端 DisplayStatus 不一致。
2. 修复工作台主体内容。
3. 修复任务页任务合约体验。
4. 修复文件页空壳和布局。
5. 修复交付页用户化展示。
6. 修复审计页结构。
7. 修复高级页。
8. 保证 build 通过。

范围：
- apps/desktop/src/**
- crates/input/src/issue.rs
- crates/state/src/indexes.rs
- 相关测试

禁止：
- 不要改无关 Rust crate。
- 不要自动执行 Codex。
- 不要新增模型调用。
- 不要做 Git commit / push / stage UI。
- 不要把 raw JSON 放到普通页面。
- 不要把 Companion 塞回工作台主体。
- 不要恢复任务页/文件页固定 Inspector。

步骤：
1. 先修复 DisplayStatus 类型和兼容读取。
2. 跑 cargo check。
3. 修复工作台三列。
4. 修复任务页文案和任务包折叠。
5. 修复文件页只保留 ProjectLocalFilesPage，并按最新 SVG 调整布局。
6. 修复交付页字段展示。
7. 修复审计页，移除常驻 OutputAuditPanel。
8. 修复高级页，移除 DesignSystemPreview，补状态文件列表。
9. 更新 Browser Preview mock 验证 6 个任务状态。
10. 跑 npm build。
11. 输出修改文件、完成点、剩余风险。

验收：
- cargo check 通过
- npm --prefix apps/desktop run build 通过
- Browser Preview 可打开
- 六个主页面与最新 SVG 页面结构一致
```
