# 007.1 - Goal Tree V1 Agent-Only Boundary Fix

创建日期：2026-06-02  
执行者：Codex

## 用户目标

PR #11 已经实现了 Goal Tree V1 的主体能力：

```text
Goal / Milestone / Issue 新模型
.agentflow/define/** 本地事实源
Goal Tree storage
integrity validation
Tauri commands
Desktop Goal Tree 页面
Graph Context Pack 关联
Browser Preview mock
```

但当前实现方向需要修正一个关键产品边界：

```text
Goal Tree 是给 Agent 使用的，不是给人类在 UI 里手动创建、编辑、写入和执行的。
```

大白话：

> Goal Tree 是 AgentFlow 给 Agent 准备的“目标工作地图”。  
> 人类可以看、审查、理解它，但不能在 Goal Tree 页面里直接创建 Goal、写 Milestone、改 Issue、归档、排序或触发执行。  
> 后续写入应该来自 Agent / 系统授权流程，而不是人类在桌面 UI 里手动操作。

---

## 一句话定义

> **007.1 Goal Tree V1 Agent-Only Boundary Fix 是对 PR #11 的边界修正：保留 Goal Tree 本地模型和事实源，但把 Desktop 人类界面改成只读，把写入能力收敛到未来 Agent-only / system-only 通道。**

---

## 背景

PR #11 中，Desktop Goal Tree 页面已经支持：

```text
创建 Goal
创建 Milestone
创建 Issue
编辑 Goal / Milestone / Issue
归档
准备 Graph Context
```

这对普通人类用户来说过于开放。

新的产品判断是：

```text
Goal Tree 是 Agent 使用的目标树，不是人类手动编辑器。
```

因此需要调整为：

```text
人类端：
  只读查看 Goal Tree
  查看完整性提示
  查看 Graph 推荐上下文
  打开推荐文件
  不创建、不编辑、不归档、不排序、不执行

Agent / system 端：
  未来可通过授权通道写入 .agentflow/define/**
  当前 V1.1 只固定边界，不实现 Agent 执行
```

---

## 与 007 的关系

007 已经完成 Goal Tree V1 主体。

007.1 不推翻 007 的模型和存储，只修正使用边界：

```text
保留：
- crates/goal-tree
- Goal / Milestone / Issue 模型
- .agentflow/define/** 存储
- integrity validation
- Goal Tree snapshot
- Graph Context Pack 关联模型
- Browser Preview mock

调整：
- Desktop UI 从可写改成只读
- Tauri Desktop 不暴露人类可调用的写命令
- create/update/archive/reorder/prepare context 不再由人类 UI 触发
- “human editable contract” 改成 “human confirmed contract / contract layer”
```

---

## 核心原则

## 1. Goal Tree 是 Agent 工作地图

Goal Tree 的作用是：

```text
给 Agent 知道：
- 当前项目目标是什么
- 阶段是什么
- Issue 是什么
- 验收标准是什么
- 依赖是什么
- 边界是什么
- 相关代码上下文在哪里
```

Goal Tree 不是：

```text
人类手动项目管理工具
Linear clone
OpenSpec editor
Issue tracker
Agent 执行按钮面板
```

---

## 2. 人类只能查看，不能写入

Desktop 人类界面只能：

```text
查看 Goal Tree
查看 Goal / Milestone / Issue contract
查看 system state
查看 Agent draft
查看 integrity warnings
查看 Graph context 信息
打开推荐文件
刷新只读 snapshot
```

Desktop 人类界面不能：

```text
创建 Goal
创建 Milestone
创建 Issue
编辑 Goal
编辑 Milestone
编辑 Issue
归档 Goal
归档 Milestone
归档 Issue
调整排序
准备 / 生成 Graph Context Pack
执行 Issue
启动 Agent
运行测试
调用模型
写 .agentflow/define/**
写 .agentflow/output/**
写用户源码
```

---

## 3. “human contract” 不是 UI 可编辑字段

当前 JSON 中有：

```json
"human": {}
```

这个 key 可以暂时保留，避免大规模 schema 变更。

但语义要改清楚：

```text
human
= human confirmed contract
= 人类确认过的目标约束
= 可以被 Agent 使用
= 不代表人类可以在 Goal Tree UI 里直接编辑
```

UI 展示名称建议改成：

```text
Contract
Confirmed Contract
目标约束
```

不要显示成：

```text
Human editable
可编辑合同
```

---

## 4. 写入必须走 Agent-only / System-only 通道

当前阶段不实现 Agent 写入，但要先定边界。

未来允许写入的来源是：

```text
Agent planning flow
System migration
Internal tests
Explicitly authorized automation
```

不允许写入的来源是：

```text
Desktop human UI
Browser preview
普通文件浏览器操作
旧 CLI legacy command
```

---

## 5. Goal Tree 不执行

Goal Tree V1 / 007.1 都不执行任何东西。

不允许：

```text
执行项目命令
运行测试
启动 Agent
claim issue
lease issue
生成 PR
调用模型
```

这些属于未来 AgentRun / Execution 层。

---

# 范围

本需求包含 7 个修正范围：

```text
1. Desktop Goal Tree UI 改为只读
2. Tauri Goal Tree command surface 收窄
3. Goal Tree write APIs 标记为 agent-only/internal
4. Graph Context 准备入口改为非人类触发
5. 文案和类型语义修正
6. Browser Preview 只读化
7. 验证和文档更新
```

---

# 非目标

本需求不做以下事情：

```text
不删除 crates/goal-tree
不删除 Goal / Milestone / Issue 模型
不删除 .agentflow/define/** 存储
不实现 AgentRun
不实现 Agent 写入流程
不接 OpenSpec 工具链
不接 Superpowers
不接 gstack
不实现人类编辑器
不实现人工创建 Issue
不新增 CLI Goal Tree 写命令
不把旧 CLI 写命令恢复
不写用户源码
不执行项目命令
不调用模型
```

---

# 1. Desktop Goal Tree UI 改为只读

## 当前问题

当前 `GoalTreePage` 暴露了人类可操作按钮：

```text
创建 Goal
创建 Milestone
创建 Issue
编辑 Goal / Milestone / Issue
归档
准备 Graph Context
```

这需要修改。

---

## 目标状态

Desktop Goal Tree 页面只读。

页面允许：

```text
刷新 Goal Tree snapshot
选择 Goal / Milestone / Issue
查看 Contract
查看 Agent Draft
查看 System State
查看 Integrity warnings
查看 Context Pack 状态
打开推荐文件
```

页面不允许：

```text
创建
编辑
保存
归档
排序
准备 Context
执行
```

---

## UI 调整

### 移除 / 禁用按钮

移除这些按钮：

```text
+ Goal
+ Milestone
+ Issue
创建
保存
归档
准备 Graph Context
```

如果产品需要展示未来能力，可以显示 disabled pill：

```text
Agent-only
由 Agent 准备
只读
```

但不要提供可点击写入操作。

---

## 页面状态

如果 Goal Tree 为空：

```text
当前项目还没有 Agent 准备的 Goal Tree。
Goal Tree 将由后续 Agent planning flow 写入。
```

不要显示：

```text
创建 Goal
```

---

## 编辑器改名

把：

```text
GoalEditor
MilestoneEditor
IssueEditor
```

改成或包装成只读组件：

```text
GoalContractViewer
MilestoneContractViewer
IssueContractViewer
```

如果不重命名文件，至少组件行为必须只读。

---

## 验收标准

```text
- [ ] Desktop Goal Tree 页面没有创建 Goal 按钮。
- [ ] Desktop Goal Tree 页面没有创建 Milestone 按钮。
- [ ] Desktop Goal Tree 页面没有创建 Issue 按钮。
- [ ] Desktop Goal Tree 页面没有保存按钮。
- [ ] Desktop Goal Tree 页面没有归档按钮。
- [ ] Desktop Goal Tree 页面没有排序按钮。
- [ ] Desktop Goal Tree 页面没有“准备 Graph Context”写入按钮。
- [ ] 选中 Goal 只能查看 Contract / Agent Draft / System State。
- [ ] 选中 Milestone 只能查看 Contract / Agent Draft / System State。
- [ ] 选中 Issue 只能查看 Contract / Agent Draft / System State / Context。
- [ ] 空状态不引导人类创建 Goal。
```

---

# 2. Tauri Goal Tree command surface 收窄

## 当前问题

PR #11 已注册很多 Tauri commands：

```text
create_goal_tree_goal
update_goal_tree_goal
archive_goal_tree_goal
create_goal_tree_milestone
update_goal_tree_milestone
archive_goal_tree_milestone
create_goal_tree_issue
update_goal_tree_issue
archive_goal_tree_issue
reorder_goal_tree
prepare_goal_tree_issue_context
```

这些命令如果暴露给 Desktop UI，就等于人类可写。

---

## 目标状态

Desktop Tauri command surface 应只暴露人类只读命令：

```text
load_goal_tree_snapshot
validate_goal_tree
```

可选只读命令：

```text
load_goal_tree_issue_context_snapshot
```

不暴露给人类 UI 的写命令：

```text
create_goal_tree_goal
update_goal_tree_goal
archive_goal_tree_goal
create_goal_tree_milestone
update_goal_tree_milestone
archive_goal_tree_milestone
create_goal_tree_issue
update_goal_tree_issue
archive_goal_tree_issue
reorder_goal_tree
prepare_goal_tree_issue_context
```

---

## 处理策略

### 推荐策略：从 Desktop Tauri handler 移除写命令

在：

```text
apps/desktop/src-tauri/src/main.rs
```

只注册：

```text
commands::goal_tree::load_goal_tree_snapshot
commands::goal_tree::validate_goal_tree
```

不要注册写命令。

写命令可以保留在 `agentflow-goal-tree` crate 中，作为未来 Agent-only API。

---

### 如果必须保留 Tauri command

如果短期为了测试必须保留写命令，则必须加 guard：

```text
caller = "agent"
capability = "goal-tree-write"
humanDesktop = false
```

人类 UI 不应持有该能力。

但 V1.1 推荐直接移除注册，避免误用。

---

## 验收标准

```text
- [ ] Tauri Desktop 只注册 Goal Tree read commands。
- [ ] `create_goal_tree_goal` 不再从 Desktop invoke handler 暴露。
- [ ] `update_goal_tree_goal` 不再从 Desktop invoke handler 暴露。
- [ ] `archive_goal_tree_goal` 不再从 Desktop invoke handler 暴露。
- [ ] `create_goal_tree_milestone` 不再从 Desktop invoke handler 暴露。
- [ ] `update_goal_tree_milestone` 不再从 Desktop invoke handler 暴露。
- [ ] `archive_goal_tree_milestone` 不再从 Desktop invoke handler 暴露。
- [ ] `create_goal_tree_issue` 不再从 Desktop invoke handler 暴露。
- [ ] `update_goal_tree_issue` 不再从 Desktop invoke handler 暴露。
- [ ] `archive_goal_tree_issue` 不再从 Desktop invoke handler 暴露。
- [ ] `reorder_goal_tree` 不再从 Desktop invoke handler 暴露。
- [ ] `prepare_goal_tree_issue_context` 不再从 Desktop invoke handler 暴露。
- [ ] Desktop build 通过。
```

---

# 3. Goal Tree write APIs 标记为 agent-only / internal

## 当前状态

`agentflow-goal-tree` crate 已有写 API：

```text
create_goal
update_goal
archive_goal
create_milestone
update_milestone
archive_milestone
create_issue
update_issue
archive_issue
reorder_goal_tree
record_issue_graph_context_path
```

这些可以保留，但必须标记为：

```text
Agent-only / system-only / internal API
```

---

## 目标状态

Rust crate 中要明确分层：

```text
read API
write API
validation API
agent-only API
```

建议模块：

```text
crates/goal-tree/src/
├── reader.rs
├── writer.rs
├── validation.rs
├── agent_api.rs
```

如果不拆文件，至少文档注释要明确。

---

## 注释要求

所有写 API 顶部或模块顶部加注释：

```rust
//! Agent-only Goal Tree write API.
//!
//! These functions mutate `.agentflow/define/**` and must not be exposed
//! through the human Desktop UI. They are reserved for a future authorized
//! Agent planning flow, migrations, and internal tests.
```

---

## 验收标准

```text
- [ ] 写 API 有 agent-only / system-only 注释。
- [ ] Desktop UI 不 import 写 API。
- [ ] Desktop Tauri 不暴露写 API。
- [ ] tests 可以继续使用写 API 创建 fixture。
- [ ] cargo test -p agentflow-goal-tree 通过。
```

---

# 4. Graph Context 准备入口改为非人类触发

## 当前问题

PR #11 的 Goal Tree context panel 有：

```text
准备 Graph Context
```

这会触发：

```text
prepare_goal_tree_issue_context
```

并写入：

```text
Issue.system.graphContextPackPath
.agentflow/output/graph/context-packs/**
```

这仍然是写行为，不应该由人类 UI 触发。

---

## 目标状态

人类 UI 只能读取已存在的 context 信息。

允许显示：

```text
已有 Context Pack
recommendedFiles
recommendedTests
warnings
```

不允许按钮触发：

```text
prepare context
generate context
refresh context pack
record context path
```

---

## UI 行为

如果 Issue 没有 Context Pack：

```text
暂无 Agent 准备的 Graph Context。
后续 Agent planning flow 会准备上下文。
```

如果 Graph 缺失：

```text
Graph 尚未 ready，Context 推荐可能不可用。
```

---

## 验收标准

```text
- [ ] GoalTreeContextPanel 不显示“准备 Graph Context”按钮。
- [ ] 人类点击页面不会调用 prepare_goal_tree_issue_context。
- [ ] UI 能显示已有 graphContextPackPath。
- [ ] UI 能显示 Issue.agentDraft.suggestedFiles。
- [ ] Graph 缺失时显示只读 warning。
- [ ] 不写 .agentflow/output/graph/context-packs/**。
```

---

# 5. 文案和语义修正

## 目标

把“人类可编辑”语义改成“人类确认的 contract”。

---

## 修改文案

把 UI 中类似：

```text
Human editable contract
编辑
保存
创建
归档
```

改成：

```text
Contract
Confirmed Contract
目标约束
只读
Agent Draft
System State
由 Agent 准备
```

---

## 类型注释

TypeScript / Rust 类型可以暂不改字段名，但注释要说明：

```text
human = human confirmed contract, not human UI editable state
```

未来可考虑 schema v2 改名：

```text
contract
```

但本次不做 breaking schema change。

---

## 验收标准

```text
- [ ] UI 不出现 “Human editable”。
- [ ] UI 不出现让人类误解可写的 “保存 / 创建 / 归档”。
- [ ] 文案清楚说明 Goal Tree 由 Agent 准备。
- [ ] Issue validationCommands 显示为只读，不执行。
```

---

# 6. Browser Preview 只读化

## 当前状态

Browser Preview 已经不会写真实 `.agentflow/`，但 UI 仍可能展示创建入口。

---

## 目标状态

Browser Preview 也是只读。

允许：

```text
展示 mock Goal Tree
展示 mock warning
展示 mock recommended files
```

不允许：

```text
创建
编辑
归档
排序
准备 context
```

---

## 验收标准

```text
- [ ] Browser Preview 没有创建按钮。
- [ ] Browser Preview 没有编辑/保存按钮。
- [ ] Browser Preview 没有准备 Graph Context 按钮。
- [ ] Browser Preview 只显示 mock read-only Goal Tree。
```

---

# 7. 测试和验证

## 必须补测试 / 检查

```text
1. Desktop build 通过。
2. Goal Tree crate tests 通过。
3. Tauri command registry 不包含 human-write Goal Tree commands。
4. UI 无 create/save/archive/prepare context 按钮。
5. Browser Preview 只读。
```

---

## 建议测试

### Rust

```text
cargo test -p agentflow-goal-tree
```

继续允许 crate 内部测试使用 write API 创建 fixture。

### Frontend / build

```text
npm --prefix apps/desktop run build
```

如果有前端测试框架，增加：

```text
GoalTreePage does not render create buttons
GoalTreeContextPanel does not render prepare context button
```

---

# 8. 开发切片

## Slice 1：Desktop UI read-only

目标：

```text
移除 GoalTreePage 创建 / 编辑 / 保存 / 归档入口
GoalEditor / MilestoneEditor / IssueEditor 改成只读 viewer
```

验收：

```text
页面只读
build 通过
```

---

## Slice 2：Tauri command surface 收窄

目标：

```text
从 Desktop invoke handler 移除 Goal Tree 写命令
只保留 load / validate
```

验收：

```text
main.rs 不再注册 create/update/archive/reorder/prepare context
cargo test -p agentflow-desktop 通过
```

---

## Slice 3：Write API agent-only 标注

目标：

```text
给 agentflow-goal-tree 写 API 加 agent-only 注释
未来 Agent planning flow 才能调用
```

验收：

```text
文档和代码注释清楚
```

---

## Slice 4：Context panel read-only

目标：

```text
移除 prepare Graph Context 按钮
只显示已有 context / warning / suggested files
```

验收：

```text
人类 UI 不触发 context pack 写入
```

---

## Slice 5：Docs and verification

目标：

```text
新增 007.1 需求
更新 007 文档状态
更新 verification
```

验收：

```text
所有验证命令通过
```

---

# 9. 总验收标准

```text
- [ ] 新增 docs/requirements/007-1-goal-tree-agent-only-boundary-fix.md。
- [ ] Goal Tree Desktop UI 只读。
- [ ] 没有创建 Goal / Milestone / Issue 按钮。
- [ ] 没有编辑 / 保存 / 归档按钮。
- [ ] 没有排序按钮。
- [ ] 没有准备 Graph Context 按钮。
- [ ] Desktop Tauri 不注册 Goal Tree 写命令。
- [ ] Desktop Tauri 只保留 Goal Tree read commands。
- [ ] agentflow-goal-tree 写 API 标记 agent-only / system-only。
- [ ] Browser Preview Goal Tree 只读。
- [ ] Human contract 文案改为 confirmed contract / contract layer。
- [ ] 不启动 Agent。
- [ ] 不执行项目命令。
- [ ] 不调用模型。
- [ ] 不写用户源码。
- [ ] 人类 UI 不写 .agentflow/define/**。
- [ ] 人类 UI 不写 .agentflow/output/graph/context-packs/**。
- [ ] cargo fmt --check 通过。
- [ ] cargo test -p agentflow-goal-tree 通过。
- [ ] cargo test 通过。
- [ ] npm --prefix apps/desktop run build 通过。
- [ ] git diff --check 通过。
```

---

# 10. 验证命令

必须执行：

```bash
cargo fmt --check
cargo test -p agentflow-goal-tree
cargo test
npm --prefix apps/desktop run build
git diff --check
```

如果改 Tauri command registry：

```bash
cargo test -p agentflow-desktop
```

---

# 11. PR 说明要求

PR 描述必须说明：

```text
1. Goal Tree 是否仍然可由人类创建：必须说明不能。
2. 哪些写命令从 Desktop Tauri 移除。
3. Goal Tree write API 为什么保留：agent-only / internal tests / future planning flow。
4. Desktop UI 现在能做什么。
5. Desktop UI 现在不能做什么。
6. 是否启动 Agent：必须说明没有。
7. 是否执行命令：必须说明没有。
8. 是否调用模型：必须说明没有。
9. 是否写 .agentflow/define/**：必须说明人类 UI 不写。
10. 验证命令和结果。
```

---

# 12. Codex 执行指令

```md
请执行 007.1 - Goal Tree V1 Agent-Only Boundary Fix。

目标：
修正 Goal Tree V1 的产品边界。Goal Tree 是给 Agent 使用的本地目标树，不是给人类在 Desktop UI 中创建、编辑、写入或执行的工具。人类 UI 只能只读查看 Goal Tree、完整性提示、已有上下文和推荐文件。

必须遵守：
1. 不删除 crates/goal-tree。
2. 不删除 Goal / Milestone / Issue 模型。
3. 不删除 .agentflow/define/** 存储能力。
4. Desktop UI 不能创建 Goal / Milestone / Issue。
5. Desktop UI 不能编辑 / 保存 / 归档 / 排序。
6. Desktop UI 不能准备 Graph Context Pack。
7. Desktop Tauri 不暴露 Goal Tree 写命令给人类 UI。
8. Goal Tree 写 API 保留为 agent-only / system-only / internal tests。
9. 不启动 Agent。
10. 不执行项目命令。
11. 不调用模型。
12. 不写用户源码。
13. 人类 UI 不写 .agentflow/define/**。
14. 人类 UI 不写 .agentflow/output/graph/context-packs/**。
15. Browser Preview 只读。
16. Graph 失败只显示 warning，不阻塞只读查看。

实现范围：
- 移除 GoalTreePage 中创建 / 保存 / 归档 / 准备 context 入口。
- GoalEditor / MilestoneEditor / IssueEditor 改成只读 viewer 或替换为 viewer 组件。
- GoalTreeContextPanel 移除 prepare context 按钮。
- 从 Tauri invoke handler 移除 create/update/archive/reorder/prepare context commands。
- 保留 load_goal_tree_snapshot / validate_goal_tree。
- 给 agentflow-goal-tree 写 API 加 agent-only 注释。
- 更新 UI 文案：human contract -> confirmed contract / contract layer。
- 更新 docs/requirements 和 verification。

验证命令：
- cargo fmt --check
- cargo test -p agentflow-goal-tree
- cargo test
- npm --prefix apps/desktop run build
- git diff --check
```

---

# 13. 完成定义

本需求完成后，Goal Tree V1 的边界应变为：

```text
Goal Tree 是 Agent 使用的目标树事实源。
人类 Desktop UI 是只读观察窗口。
人类不能在 Goal Tree 内创建、写入、归档、排序或执行。
写入能力保留给未来 Agent planning flow。
当前阶段仍不启动 Agent、不执行命令、不调用模型。
```

最终一句话：

> **007.1 把 Goal Tree 从“人类可编辑目标树”修正为“Agent 使用、人类只读查看的目标树事实源”。**
