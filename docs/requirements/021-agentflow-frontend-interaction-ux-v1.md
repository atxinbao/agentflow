# 021 - AgentFlow Frontend Interaction UX V1

## 1. 文档定位

这是 **前端交互总纲**。

它不是 Phase 1，也不是单独的“第二步实现任务”。
它用于指导 Phase 2 之后所有页面的交互逻辑。

关系如下：

```text
020 Design System & App Shell
= 第一步，先搭风格和组件基座

021 Frontend Interaction UX
= 交互总纲，指导后续登录、引导、工作台、任务、文件、交付、审计、高级、Companion 的实现
```

如果要严格拆开发阶段：

```text
Phase 1 = 020 App Shell + Design System
Phase 2 = 登录 + 首次引导
Phase 3 = 工作台
Phase 4 = 任务页
Phase 5 = 文件页
Phase 6 = 交付 / 审计 / 高级
Phase 7 = Companion
```

---

## 2. 背景

前面已经完成了 Figma 风格与页面主体内容设计。
现在需要进入真实前端交互：

```text
用户怎么走
页面怎么响应
按钮什么时候可点
点击后状态怎么变
页面怎么跳转
数据什么时候刷新
错误怎么提示
空态怎么展示
```

---

## 3. 总体目标

实现 AgentFlow 的前端交互体验：

```text
1. 首次使用动线清楚。
2. 日常打开直接进入项目工作台。
3. 任务从选择、查看、复制任务包、交给 Codex、检查写回到交付审计，路径清楚。
4. 文件页只读，不误导用户编辑。
5. 交付页展示结果和证据。
6. 审计页展示报告和证据链。
7. 高级页只读诊断。
8. Companion 可以放在 Codex 旁边使用。
```

---

## 4. 全局交互原则

### 4.1 每个阶段只给一个主动作

例如：

```text
选择项目 → 打开本地项目
完成引导 → 进入工作台
Ready 任务 → 复制任务包
等待写回 → 检查写回
Delivery ready → 请求审计
```

### 4.2 普通页面不展示 raw JSON

普通页面只展示人能看懂的信息。

```text
不要展示 workflow.json
不要展示 gates.json
不要展示 manifest.json
不要展示 sourceSpecId
不要展示 system.path
```

高级页例外。

### 4.3 所有自动感按钮必须说清楚

这些按钮不是自动控制 Codex：

```text
复制任务包
我已交给 Codex
检查写回
请求审计
```

必须明确：

```text
复制任务包 = 复制到剪贴板
我已交给 Codex = 本地状态标记
检查写回 = 扫描本地 .agentflow/output 和 .agentflow/execute
请求审计 = 本地生成 audit package
```

---

## 5. App 状态机

### 5.1 顶层状态

```ts
type AppLifecycleState =
  | "not-authenticated"
  | "first-run"
  | "project-loading"
  | "workspace-ready"
  | "workspace-blocked"
  | "error"
```

### 5.2 转换

```text
打开 App
→ not-authenticated
→ 登录成功
→ first-run 或 workspace-ready
→ 选择项目
→ project-loading
→ workspace-ready
```

如果用户已完成首次引导：

```text
打开 App
→ workspace-ready
```

---

## 6. 首次使用动线

```text
打开 App
→ 登录入口
→ 选择大模型入口
→ 选择本地项目
→ 准备环境
→ 认识 Agent
→ 确认意图
→ 完成引导
→ 进入工作台
```

### 6.1 登录

按钮：

```text
连接 ChatGPT
连接 Claude
连接 DeepSeek
```

状态：

```text
not-connected
connecting
connected
expired
error
```

### 6.2 选择项目

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

### 6.3 环境准备

状态：

```text
checking
ready
degraded
failed
```

准备完成前：

```text
下一步 disabled
```

### 6.4 完成引导

只保留：

```text
进入工作台
```

不要有：

```text
打开 Codex
上一步
```

---

## 7. 日常使用动线

```text
打开 App
→ 进入最近项目工作台
→ 查看当前任务 / 最近活动
→ 切换任务 / 文件 / 交付 / 审计 / 高级
```

必须支持：

```text
添加项目
切换项目
恢复最近项目
恢复选中的页面
刷新当前项目状态
```

---

## 8. 工作台交互

### 8.1 页面目标

工作台回答：

```text
项目能不能用？
当前正在处理什么？
最近发生了什么？
下一步应该做什么？
```

### 8.2 可交互元素

```text
刷新图标
当前任务行
最近活动行
左侧导航
底部状态栏
```

### 8.3 点击行为

```text
点击当前任务
→ 进入任务页并选中该任务

点击最近活动中的交付
→ 进入交付页并选中交付

点击最近活动中的审计
→ 进入审计页并选中审计

点击刷新
→ 重新读取 state / input / output 摘要
```

---

## 9. 任务页交互

### 9.1 页面结构

最新 UX 是：

```text
左侧：任务流转列表
右侧：任务合约详情
```

不是旧式多列 Kanban。

### 9.2 任务状态

使用新的 6 个状态：

```text
Backlog
Ready
In Progress
Review
Done
Cancel
```

中文：

```text
待办
就绪
进行中
待审阅
已完成
已取消
```

### 9.3 选中规则

```text
优先选中 activeIssueId
没有 activeIssueId 时选中最近更新时间最高的 In Progress / Ready 任务
没有任务时显示空态
```

### 9.4 任务详情动作

根据状态显示动作：

```text
Backlog
→ 查看需求

Ready
→ 复制任务包

In Progress
→ 我已交给 Codex / 检查写回

Review
→ 查看交付 / 请求审计

Done
→ 查看交付 / 查看审计

Cancel
→ 只读查看
```

### 9.5 检查写回

点击：

```text
检查写回
```

行为：

```text
扫描 .agentflow/output
扫描 .agentflow/execute
刷新 delivery / evidence / state
```

结果：

```text
发现交付 → 跳到交付页或更新任务为 Review
未发现交付 → 提示“还没有检测到 Codex 写回结果”
```

---

## 10. 文件页交互

### 10.1 页面结构

```text
左侧或右侧：项目文件列表
主体：只读文件 Reader
```

以最新 SVG 为准。

### 10.2 点击文件

```text
点击文件
→ 读取内容
→ Reader 展示
```

### 10.3 只读规则

必须明确：

```text
只能读，不能改。
```

不提供：

```text
编辑
保存
删除
重命名
Git 操作
```

---

## 11. 交付页交互

### 11.1 页面结构

```text
左侧：交付列表
右侧：交付包详情
```

### 11.2 点击交付

```text
点击交付
→ 展示交付详情
```

详情展示：

```text
交付摘要
关联任务
验证结果
证据
变更文件
缺失证据
```

### 11.3 请求审计

按钮：

```text
请求审计
```

行为：

```text
生成本地 audit package
刷新 audit 列表
跳到审计页并选中新 audit
```

---

## 12. 审计页交互

### 12.1 页面结构

```text
左侧：审计列表
右侧：审计报告详情
```

### 12.2 点击审计

```text
点击审计
→ 展示报告详情
```

详情展示：

```text
审计结论
发现项
证据链
处理建议
```

### 12.3 当前版本限制

如果后端没有真实接受 / 返工 / 补证据状态写入，本轮不要放可点击按钮。

可以显示只读建议：

```text
建议：补充证据
建议：返工
建议：接受
```

---

## 13. 高级页交互

### 13.1 页面结构

```text
左侧：分类
中间：状态文件列表
右侧：JSON Reader + 人话解释
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

### 13.2 点击状态文件

```text
点击状态文件
→ 右侧展示 JSON
→ 展示中文解释
```

### 13.3 禁止动作

高级页不做：

```text
自动修复
清理锁
继续执行
触发审计
删除状态文件
编辑 JSON
```

---

## 14. Companion 交互

### 14.1 目标

Companion 放在 Codex 旁边。

### 14.2 展示

```text
当前项目
工作流状态
任务队列
当前任务
底部动作
```

### 14.3 按钮

```text
检查写回
任务包
打开文件
```

---

## 15. 页面状态

所有页面都要支持：

```text
loading
empty
ready
blocked
error
stale
```

示例：

```text
任务页 empty：
还没有任务。请先确认需求，生成任务。

文件页 empty：
还没有读取到项目文件。请刷新项目现场。

交付页 empty：
还没有交付结果。Codex 写回后会显示在这里。

审计页 empty：
还没有审计报告。交付完成后可以请求审计。

高级页 empty：
还没有状态快照。请先准备项目环境。
```

---

## 16. 按钮状态

按钮必须支持：

```text
enabled
disabled
loading
success
error
```

示例：

```text
复制任务包：
enabled = 任务 Ready
disabled = 缺少任务合约或 SPEC
success = 已复制
error = 生成任务包失败
```

---

## 17. 数据刷新规则

会触发刷新：

```text
打开项目
切换项目
切换页面
点击刷新
检查写回
请求审计成功
完成首次引导
```

不会触发业务刷新：

```text
复制任务包
查看文件
查看高级 JSON
切换 Light / Dark
```

---

## 18. ViewModel 要求

需要建立：

```text
AppInteractionState
WorkspaceInteractionState
TaskInteractionState
FileInteractionState
DeliveryInteractionState
AuditInteractionState
AdvancedInteractionState
CompanionInteractionState
```

页面只消费 ViewModel，不要直接到处拼底层 JSON。

---

## 19. 验收标准

```text
1. 首次引导流程完整。
2. 日常打开能直接进入工作台。
3. 工作台能跳到任务 / 交付 / 审计。
4. 任务页能选中任务并显示任务合约。
5. 文件页只能读文件。
6. 交付页能选中交付并显示详情。
7. 审计页能选中审计并显示报告。
8. 高级页能选中状态文件并显示 JSON。
9. 所有按钮状态正确。
10. 所有空态 / 错误态 / 阻断态有文案。
```

---

## 20. Codex 实现指令

```text
你现在只做这个任务：实现 AgentFlow Frontend Interaction UX V1。

背景：
020 已定义 Design System 和 AppShell。
本需求定义用户动线、页面状态、按钮行为和刷新规则。

目标：
把静态页面变成可交互的前端体验。

范围：
- 只改 apps/desktop/src/**
- 不改 Rust 后端
- 可以先用 mock ViewModel
- 不自动执行 Codex

步骤：
1. 阅读 020 设计系统文档。
2. 阅读本交互文档。
3. 建立 AppInteractionState。
4. 建立页面 ViewModel。
5. 实现首次引导状态流转。
6. 实现日常项目进入流程。
7. 实现任务选择和详情展示。
8. 实现文件选择和只读 Reader。
9. 实现交付 / 审计 / 高级选择详情。
10. 实现刷新、检查写回、请求审计的前端行为占位。
11. 保证 TypeScript build 通过。

禁止：
- 不要改 Rust 后端。
- 不要自动执行 Codex。
- 不要把 raw JSON 放到普通页面。
- 不要做文件编辑。
- 不要做 Git 操作。

输出：
- 改了哪些文件
- 完成哪些交互
- 哪些仍是 mock
- build 结果
```
