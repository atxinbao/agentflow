# 041 - Goal Recheck And Completion Decision Runtime V1

> 文档类型：开发需求
> 日期：2026-06-18
> 执行者：Codex
> 状态：Ready for Development

---

## 1. 背景

当前系统的“完成”还容易退化成：

```text
issue done 数量累计
```

这不够。项目执行一轮后，必须回到 Goal Agent，重新判断：

- 继续；
- 调整；
- 暂停；
- 接受交付；
- 进入下一阶段。

---

## 2. 用户目标

项目不是“任务都 done 了就自动结束”，而是 Goal Agent 基于执行结果重新判断项目是否真的完成。

一句话：

```text
Completion 必须成为正式 runtime 决策，而不是 issue 计数器。
```

---

## 3. 范围

### 3.1 必须做

1. 建立 Goal Recheck runtime。
2. 建立 Completion Decision 事实模型。
3. 支持 continue / adjust / pause / accept / next-stage 决策输出。
4. 让 Project 页面展示 completion hint。
5. 让 Project 完成判断依赖 completion 决策，而不是简单统计。

### 3.2 涉及模块

- `crates/spec/**`
- `crates/workflow-runtime/**`
- `crates/projection/**`
- `crates/state/**`
- `apps/desktop/src/**`

---

## 4. 关键设计要求

### 4.1 Delivery 后回 Goal

- 任务执行结束后，项目必须能回到 Goal Recheck。
- Delivery 不是最终 authority。

### 4.2 Completion 是单独判断

- `accepted / complete` 必须有单独决策事实。
- 不能由 issue done、PR merged、release 产物任一单点直接替代。

### 4.3 对用户可解释

用户要能看懂：

- 为什么还不能算完成；
- 为什么可以继续下一阶段；
- 为什么项目被接受或暂停。

---

## 5. 非目标

- 不直接实现 release runtime。
- 不直接替代 Audit Flow。
- 不绕过 Goal Agent 做自动 completion。

---

## 6. 依赖

- [039-project-brain-runtime-entry-v1.md](/Users/mac/Documents/AgentFlow/docs/requirements/039-project-brain-runtime-entry-v1.md)
- [040-requirement-to-goal-plan-preview-runtime-v1.md](/Users/mac/Documents/AgentFlow/docs/requirements/040-requirement-to-goal-plan-preview-runtime-v1.md)

---

## 7. 验收标准

- [ ] Delivery 后可以回到 Goal Recheck。
- [ ] Completion Decision 有正式事实模型和投影。
- [ ] Project 完成不再只是 issue done 简单累计。
- [ ] Project 页面能解释 completion hint。
- [ ] 用户可以理解接受、继续、调整、暂停这些决策。

---

## 8. 验证命令

- `cargo test --workspace`
- `npm --prefix apps/desktop run build`
- `git diff --check`
