# 061 - Task Page Execution And Delivery Convergence V1

> 文档类型：开发需求
> 日期：2026-06-19
> 执行者：Codex
> 状态：Ready for Development

---

## 1. 背景

AgentFlow 已经明确不再把“执行”和“交付”作为独立一级栏目。

对用户来说，执行和交付都属于任务生命周期的一部分：

- 执行是 `todo -> in_progress -> in_review`
- 交付是 `in_review -> done` 附近形成的人类可接收结果

因此需要把原本分散在旧执行页、旧交付页里的阅读职责全部并入任务页。

---

## 2. 用户目标

用户查看一条任务时，可以在同一页里看清：

- 做了什么；
- 如何验证；
- 产出了什么；
- 当前是否已经形成可读交付；
- 完成后对人类的交付内容是什么。

一句话：

```text
执行与交付都必须跟着任务走，不能再让用户切页面找上下文。
```

---

## 3. 范围

### 3.1 必须做

1. 任务页并入执行事实阅读面。
2. 任务页并入交付槽位。
3. 当前阶段详情能解释执行中发生的关键动作。
4. `in_review / done` 能展示交付摘要。
5. 任务页可展示验证结果、证据摘要、公开交付摘要。
6. 删除“旧执行页 / 旧交付页是主阅读入口”的产品依赖。

### 3.2 涉及模块

- `apps/desktop/src/**`
- `crates/projection/**`
- `crates/task-artifacts/**`
- `crates/release/**`

---

## 4. 关键设计要求

### 4.1 执行信息跟随状态流

执行中的：

- run
- checkpoint
- verification
- session summary

都应该挂在任务当前状态之下，而不是跑去单独“执行”页。

### 4.2 交付信息是任务末端槽位

右侧详情必须有固定交付槽位，用来展示：

- PR / MR body 摘要
- evidence summary
- merge proof
- done writeback
- public delivery summary

### 4.3 交付不等于审计

交付槽位只负责说明“任务交了什么”。

审计结论仍然是独立流程，不混到交付内部。

---

## 5. 非目标

- 不在本需求里重做审计页面。
- 不在本需求里改变 public delivery 标准。
- 不在本需求里新增 release 页面。

---

## 6. 依赖

- [059-agentflow-desktop-frontend-information-architecture-v1.md](/Users/mac/Documents/AgentFlow/docs/requirements/059-agentflow-desktop-frontend-information-architecture-v1.md)
- [060-task-page-state-timeline-and-event-stream-v1.md](/Users/mac/Documents/AgentFlow/docs/requirements/060-task-page-state-timeline-and-event-stream-v1.md)
- [047-delivery-flow-productization-v1.md](/Users/mac/Documents/AgentFlow/docs/requirements/047-delivery-flow-productization-v1.md)
- [053-public-delivery-standardization-v1.md](/Users/mac/Documents/AgentFlow/docs/requirements/053-public-delivery-standardization-v1.md)

---

## 7. 验收标准

- [ ] 用户不需要离开任务页就能理解执行过程。
- [ ] 用户不需要离开任务页就能看到交付摘要。
- [ ] 交付槽位在 `in_review / done` 阶段可读。
- [ ] 执行信息和交付信息都被收进任务生命周期。
- [ ] 旧执行页 / 旧交付页不再承担主阅读职责。

---

## 8. 验证命令

- `npm --prefix apps/desktop run build`
- `git diff --check`
- Browser Preview / Desktop smoke：任务页执行事实与交付槽位
