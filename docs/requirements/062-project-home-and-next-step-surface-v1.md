# 062 - Project Home And Next Step Surface V1

> 文档类型：开发需求
> 日期：2026-06-19
> 执行者：Codex
> 状态：Ready for Development

---

## 1. 背景

工作台页已经不应只是一个概览页，而应该成为真正的 `Project Home`。

它需要从项目视角解释：

- 当前处于哪个项目阶段；
- 当前活跃任务是什么；
- 为什么下一步是这个动作；
- Goal / Plan / Decisions 是否健康；
- 是否需要 Goal Recheck / completion 决策。

059 已明确，工作台页是项目方向入口，不是任务堆叠页。

---

## 2. 用户目标

用户打开项目后，第一屏就能看明白：

- 项目当前在哪个阶段；
- 为什么停在这里；
- 下一步该做什么；
- 谁会接管下一步。

一句话：

```text
工作台要成为 Project Home，而不是任务页的前置空壳。
```

---

## 3. 范围

### 3.1 必须做

1. 建立项目阶段摘要区。
2. 建立下一步卡片。
3. 展示当前活跃任务。
4. 展示 blockers / next actions。
5. 展示 Goal / Plan / Decisions 摘要。
6. 展示 Goal Recheck / completion hint。
7. 明确 Goal Agent / Spec Agent 主入口都在项目页。

### 3.2 涉及模块

- `apps/desktop/src/**`
- `crates/projection/**`
- `crates/state/**`
- `crates/spec/**`

---

## 4. 关键设计要求

### 4.1 项目优先，不重复任务页

工作台必须从 Project Flow 角度组织信息，而不是把任务列表复制一遍。

### 4.2 下一步卡片是项目页主入口

用户不应该先选 Agent。

用户应该先看到：

- 当前阶段
- 下一步建议
- 为什么是这一步
- 一个主按钮

### 4.3 接住 Project Brain

工作台必须能读 Project Brain 相关信息：

- Goal
- Plan
- Decisions
- Project Health / Brain Status

---

## 5. 非目标

- 不在本需求里重做任务页状态时间线。
- 不在本需求里实现完整 Goal 文档编辑器。
- 不在本需求里实现 release / audit 全流程。

---

## 6. 依赖

- [059-agentflow-desktop-frontend-information-architecture-v1.md](/Users/mac/Documents/AgentFlow/docs/requirements/059-agentflow-desktop-frontend-information-architecture-v1.md)
- [039-project-brain-runtime-entry-v1.md](/Users/mac/Documents/AgentFlow/docs/requirements/039-project-brain-runtime-entry-v1.md)
- [041-goal-recheck-and-completion-decision-runtime-v1.md](/Users/mac/Documents/AgentFlow/docs/requirements/041-goal-recheck-and-completion-decision-runtime-v1.md)
- [043-project-page-productization-v1.md](/Users/mac/Documents/AgentFlow/docs/requirements/043-project-page-productization-v1.md)

---

## 7. 验收标准

- [ ] 工作台能解释当前项目阶段。
- [ ] 工作台能解释为什么下一步是这个动作。
- [ ] 工作台能展示当前活跃任务与 blockers。
- [ ] 用户不需要读底层 facts 就能理解项目推进状态。
- [ ] Goal Agent / Spec Agent 的主入口落在项目页。

---

## 8. 验证命令

- `npm --prefix apps/desktop run build`
- `git diff --check`
- Browser Preview / Desktop smoke：工作台阶段与下一步卡片
