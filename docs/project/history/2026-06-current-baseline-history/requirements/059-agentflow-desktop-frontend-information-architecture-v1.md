# 059 - AgentFlow Desktop Frontend Information Architecture V1

> 文档类型：开发需求
> 日期：2026-06-19
> 执行者：Codex
> 状态：Ready for Development

---

## 1. 背景

AgentFlow 当前已经逐步形成新的产品基线：

- `Project` 是顶层入口；
- `Workflow State` 是流程骨架；
- `Event Flow` 是事实流；
- `Agent` 是执行角色，不是主导航对象。

但前端客户端的信息架构还没有彻底定型，仍然存在以下问题：

1. 页面职责有历史残留；
2. 执行 / 交付仍然容易被看成独立栏目，而不是任务生命周期的一部分；
3. 项目页、任务页、审计页之间的职责边界还不够稳定；
4. Agent 入口位置还没有完全收口；
5. 用户仍可能在多个页面之间来回寻找“当前发生了什么”。

需要用一份统一的信息架构需求，把桌面客户端的页面职责、导航边界、状态流展示规则和 Agent 入口位置一次性定下来。

---

## 2. 用户目标

用户打开 AgentFlow Desktop 后，不需要理解底层 runtime 模块，也不需要理解一堆 agent prompt 或目录结构，就能直接回答：

- 当前项目在哪个阶段？
- 当前应该推进哪条任务？
- 这条任务现在走到哪个状态？
- 它已经产出了什么？
- 是否需要审计、交付或人工决策？

一句话：

```text
AgentFlow Desktop 要呈现成一个以 Project 为入口、以 Task 为主工作面、以 Workflow State 为骨架、以 Event Flow 为事实流、以 Audit 为独立验收面的本地 Agent 项目控制台。
```

---

## 3. 范围

### 3.1 必须做

1. 冻结桌面端一级导航信息架构。
2. 冻结各页面职责边界。
3. 明确任务页为主工作面。
4. 明确执行与交付并入任务页。
5. 明确审计页作为独立验收面保留。
6. 明确 Agent 入口跟随页面对象，而不是成为独立主栏目。
7. 明确任务状态流、事件流、交付槽位的展示规则。
8. 明确项目页、任务页、审计页、文件页、高级页的数据语义。

### 3.2 本需求主要作用

本需求先定义 **前端信息架构和产品表面规则**，不要求在本需求里直接完成全部实现。

它是后续页面重构、组件层调整、projection 收口和 browser preview 重构的统一依据。

### 3.3 涉及模块

- `apps/desktop/src/**`
- `docs/product/**`
- `docs/foundation/**`
- `docs/architecture/**`
- `docs/requirements/**`

---

## 4. 一级导航定义

桌面端一级栏目固定为：

1. `工作台`
2. `任务`
3. `审计`
4. `文件`
5. `高级`

明确规则：

- 不保留独立 `执行` 栏目；
- 不保留独立 `交付` 栏目；
- 执行与交付视图都并入任务页；
- 审计保持独立，因为它是独立流程；
- Agent 不成为一级导航对象。

---

## 5. 全局布局规则

整套桌面客户端固定为三层：

### 5.1 左侧

左侧负责：

- 添加项目；
- 项目列表；
- 当前项目下的页面导航；
- 项目 / 任务树选择。

### 5.2 中间

中间是当前页面主内容区，承担主阅读和主操作。

### 5.3 顶部 / 底部

顶部和底部只展示轻量全局状态：

- 当前项目名；
- 全局阶段摘要；
- 搜索 / 刷新 / 命令入口；
- 本地模式；
- 快捷键提示；
- 简短环境状态。

不允许在顶部 / 底部重复堆叠完整工作流细节。

---

## 6. 页面职责定义

### 6.1 工作台

工作台是 `Project Home`，不是任务列表页。

它要回答：

- 这个项目是什么；
- 当前项目阶段是什么；
- 下一步该做什么；
- 当前是否有活跃任务；
- Goal / Plan / Decisions 是否健康。

建议区块：

- 项目摘要；
- 当前项目阶段；
- 下一步卡片；
- 当前活跃任务；
- Project Brain 摘要；
- Goal / Plan / Decisions 状态；
- 风险 / blockers 摘要；
- Agent 角色使用说明。

工作台是 `Goal Agent / Spec Agent` 的主入口页。

### 6.2 任务

任务页是整个客户端的主工作面。

它要回答：

- 当前项目任务结构是什么；
- 当前选中任务是什么；
- 任务现在处于哪个状态；
- 这个状态已经发生了什么；
- 后面还会发生什么；
- 最终交付槽位是什么；
- 是否需要进入审计。

任务页要承载：

- 任务树；
- 状态时间线；
- 事件流；
- 当前阶段详情；
- 交付槽位；
- 审计入口；
- 任务信息折叠区。

### 6.3 审计

审计页是独立验收面。

它要回答：

- 哪些任务需要审计；
- 当前审计状态是什么；
- Findings 是什么；
- Evidence 是否完整；
- 是否通过 / 阻断 / 返工。

### 6.4 文件

文件页是只读事实面。

它要回答：

- 项目文档在哪；
- requirement / product / foundation / architecture 文档如何读取；
- 本地结构化事实如何读取。

默认优先展示对人类有意义的文档，而不是直接把 JSON Reader 作为主阅读面。

### 6.5 高级

高级页是调试与底层事实面。

它要回答：

- runtime 现在发生了什么；
- workflow / projection / event 的原始事实是什么；
- provider session 是否正常；
- 当前角色边界和只读规则是什么。

高级页是高级用户与调试场景入口，不是普通用户主工作面。

---

## 7. 任务页结构规则

任务页固定拆成：

### 7.1 左侧任务树

左侧来自 `Project Projection / Task Projection`，而不是直接读原始 spec 文件。

必须包含：

- project 分组；
- 单项任务分组；
- issue 排序；
- issue 状态点；
- priority badge；
- 当前选中态；
- 当前 / 过去 / 未来语义。

### 7.2 右侧任务详情

右侧必须以状态流为主，而不是以静态卡片堆叠为主。

固定层次：

1. 标题区
2. 元信息区
3. 状态时间线
4. 状态下挂事件流
5. 当前阶段详情
6. 交付槽位
7. 审计入口
8. 折叠附加信息

---

## 8. 状态流展示规则

任务页右侧固定使用统一任务状态：

- `backlog`
- `todo`
- `in_progress`
- `in_review`
- `done`
- `blocked`
- `cancel`

展示规则：

### 8.1 当前状态

- 展示实时事件流；
- 展示当前阶段详情；
- 可以看到最新日志和产物引用。

### 8.2 已完成状态

- 展示历史事件；
- 展示关键结果与产物；
- 不显示伪实时占位。

### 8.3 未来状态

- 只展示等待；
- 不展示假日志；
- 不生成未发生的阶段详情。

### 8.4 blocked / cancel

- 必须展示原因；
- 不得继续伪装成推进中。

---

## 9. 执行与交付并入任务页

任务页必须吸收原“执行 / 交付”理解职责。

明确规则：

- `执行` 不是独立页面对象，而是 `todo -> in_progress -> in_review` 的状态流推进；
- `交付` 不是独立主导航对象，而是任务从 `in_review -> done` 附近的交付槽位和公开交付结果；
- 用户不需要离开任务页，就能理解：
  - 当前任务做了什么；
  - 跑了哪些验证；
  - 形成了什么交付；
  - 是否已经写回完成。

---

## 10. 审计挂载规则

审计保持独立流程，但任务页必须能解释它和任务的关系。

任务页只负责：

- 显示是否需要审计；
- 显示审计摘要；
- 提供进入审计页入口。

真正的审计结论、findings、evidence map 和修复建议统一收在审计页。

---

## 11. Agent 呈现规则

Agent 是执行角色，不是主导航对象。

### 11.1 Agent 主入口位置

- `工作台`：Goal Agent / Spec Agent
- `任务页`：Work Agent / Delivery Agent / Audit Agent
- `高级 / 命令面板`：调试和手动入口

### 11.2 不允许做的事

- 不新增独立 `Agent Center`
- 不把五个 Agent 做成五个平行页面
- 不要求用户先选 Agent，再决定下一步

对用户来说，入口应该是：

```text
先看对象，再做动作。
Project 页面决定方向。
Task 页面决定推进。
Audit 页面决定验收。
```

---

## 12. 视觉语义规则

### 12.1 客户端整体风格

应保持：

- 原生桌面感；
- 低噪音；
- 强结构线；
- 少卡片；
- 小圆角；
- 轻状态色；
- 本地工程控制台气质。

### 12.2 状态点语义

建议统一为：

- 绿色点：完成
- 红色点：取消 / 失败终止
- 蓝色点：未来 / 未开始 / 就绪
- 黄色动态点：当前进行中
- 灰色点：未激活或占位

不需要再用大量文字重复解释状态含义。

---

## 13. 非目标

本需求不做：

- 不做聊天框首页；
- 不做 Agent 独立主栏目；
- 不做 BPMN 式流程编辑器；
- 不把 JSON Reader 作为主工作面；
- 不把项目降级成单纯任务列表；
- 不在本需求里重做整套视觉设计系统；
- 不在本需求里直接定义完整 runtime 实现细节。

---

## 14. 依赖

- [038-agentflow-project-operating-system-runtime-foundation-v1.md](/Users/mac/Documents/AgentFlow/docs/requirements/038-agentflow-project-operating-system-runtime-foundation-v1.md)
- [039-project-brain-runtime-entry-v1.md](/Users/mac/Documents/AgentFlow/docs/requirements/039-project-brain-runtime-entry-v1.md)
- [042-task-page-productization-v1.md](/Users/mac/Documents/AgentFlow/docs/requirements/042-task-page-productization-v1.md)
- [043-project-page-productization-v1.md](/Users/mac/Documents/AgentFlow/docs/requirements/043-project-page-productization-v1.md)
- [044-task-tree-and-sequence-ux-v1.md](/Users/mac/Documents/AgentFlow/docs/requirements/044-task-tree-and-sequence-ux-v1.md)
- [046-audit-flow-productization-v1.md](/Users/mac/Documents/AgentFlow/docs/requirements/046-audit-flow-productization-v1.md)
- [047-delivery-flow-productization-v1.md](/Users/mac/Documents/AgentFlow/docs/requirements/047-delivery-flow-productization-v1.md)

---

## 15. 完成定义

满足以下条件，视为这份信息架构正式冻结：

1. 一级导航固定为工作台 / 任务 / 审计 / 文件 / 高级。
2. 执行与交付不再作为独立主栏目。
3. 任务页被明确为主工作面。
4. 项目页被明确为 Project Home。
5. 审计页被明确为独立验收面。
6. Agent 入口跟随页面对象，而不是成为独立对象。
7. 状态流 / 事件流 / 交付槽位三者关系被写清楚。
8. 后续页面实现和交互重构以本信息架构为统一依据。

---

## 16. 验收标准

- [ ] 用户能从工作台理解项目方向与下一步。
- [ ] 用户能从任务页完整理解单任务生命周期。
- [ ] 用户不需要跳到独立执行页或交付页才能理解任务主链路。
- [ ] 审计作为独立验收面保留，但不再和执行/交付混杂。
- [ ] Agent 入口位置明确，不会让用户先“选 Agent”再工作。
- [ ] 项目 / 任务 / 审计 / 文件 / 高级 五类页面职责不再冲突。

---

## 17. 验证命令

- `git diff --check`

本需求当前是信息架构文档，不要求在本轮直接跑 build / test。
