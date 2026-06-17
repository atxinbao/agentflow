# 043 - Project Page Productization V1

> 文档类型：开发需求
> 日期：2026-06-18
> 执行者：Codex
> 状态：Ready for Development

---

## 1. 背景

项目页后续不应只是项目基础信息展示，而应成为 Project Flow 的主页面。

它要能回答：

- 当前项目在哪个阶段；
- 当前任务是什么；
- 下一步为什么还没开始；
- 当前阻断是什么；
- Completion 有没有信号。

---

## 2. 用户目标

用户打开项目页时，能从项目视角理解整个工作流，而不是只能看单个任务。

一句话：

```text
项目页要成为 Project Flow 的主工作台。
```

---

## 3. 范围

### 3.1 必须做

1. 建立项目阶段摘要区。
2. 展示当前活跃 issue 和 next actions。
3. 展示 blockers。
4. 展示 Goal Recheck / completion hint。
5. 保留 loop trigger / project action area。

### 3.2 涉及模块

- `apps/desktop/src/**`
- `crates/projection/**`
- `crates/state/**`

---

## 4. 关键设计要求

### 4.1 项目优先，不是任务拼盘

- 项目页展示的是阶段与节奏。
- 不是简单把任务列表再重复一遍。

### 4.2 能解释“为什么”

项目页要能解释：

- 为什么下一条 issue 还没开始；
- 为什么项目停在当前阶段；
- 为什么还不能 completion。

### 4.3 接住 Goal Recheck

- Goal Recheck 和 completion hint 应该能在项目页被读懂。

---

## 5. 非目标

- 不让项目页直接修改 authority。
- 不在本需求里实现完整 release 页面。

---

## 6. 依赖

- [039-project-brain-runtime-entry-v1.md](/Users/mac/Documents/AgentFlow/docs/requirements/039-project-brain-runtime-entry-v1.md)
- [041-goal-recheck-and-completion-decision-runtime-v1.md](/Users/mac/Documents/AgentFlow/docs/requirements/041-goal-recheck-and-completion-decision-runtime-v1.md)
- [042-task-page-productization-v1.md](/Users/mac/Documents/AgentFlow/docs/requirements/042-task-page-productization-v1.md)

---

## 7. 验收标准

- [ ] 项目页能解释当前阶段。
- [ ] 项目页能解释当前 issue、阻断和下一步。
- [ ] 项目页能展示 completion hint。
- [ ] 用户不需要读底层事实目录，就能理解项目当前状态。

---

## 8. 验证命令

- `npm --prefix apps/desktop run build`
- `git diff --check`
- Browser Preview / Desktop smoke：项目页阶段与 next actions
