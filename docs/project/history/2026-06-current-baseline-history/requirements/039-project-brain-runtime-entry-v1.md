# 039 - Project Brain Runtime Entry V1

> 文档类型：开发需求
> 日期：2026-06-18
> 执行者：Codex
> 状态：Ready for Development

---

## 1. 背景

`038` 已经把 runtime foundation 定住，但 Project Brain 目前仍偏文档层事实：

```text
GOAL.md
PLAN.md
DECISIONS.md
PROJECT_HEALTH.md
```

系统还不能把它们稳定接入 runtime，导致：

- Goal Agent 还不是正式上游入口；
- Project 页面无法基于 Project Brain 解释下一步；
- Requirement -> Goal -> Spec 之间缺少正式桥梁。

---

## 2. 用户目标

用户打开项目后，AgentFlow 应该能直接回答三件事：

1. 这个项目当前目标是什么；
2. 当前计划推进到哪里；
3. 下一步为什么是这一步。

一句话：

```text
Project Brain 要从“文档集合”变成“runtime 可读取、可投影、可解释的正式上游事实”。
```

---

## 3. 范围

### 3.1 必须做

1. 建立 Project Brain 读模型。
2. 让 `GOAL.md / PLAN.md / DECISIONS.md / PROJECT_HEALTH.md` 进入 runtime snapshot。
3. 建立 Goal Agent runtime entry。
4. 建立 Project Brain -> Spec runtime bridge。
5. 让 Project 页面能读取 Project Brain 派生出的 next actions。

### 3.2 涉及模块

- `crates/spec/**`
- `crates/projection/**`
- `crates/workflow-runtime/**`
- `crates/state/**`
- `apps/desktop/src/**`

如果暂时不新增独立 `project-brain` crate，也必须在上述模块中把 authority 边界收清楚。

---

## 4. 关键设计要求

### 4.1 Project Brain 是上游 authority

- Goal / Plan / Decisions / Health 是项目级 authority。
- Issue / Run / Session 都不是 Project Brain 的替代品。

### 4.2 先读，再投影，再驱动

- 先形成稳定 snapshot。
- 再形成 projection。
- 最后由 Project Flow 派生 next actions。

### 4.3 不直接开工

本需求只解决 Project Brain 的 runtime entry，不直接启动 Work Agent，也不直接执行 issue。

---

## 5. 非目标

- 不直接实现 Requirement intake preview。
- 不直接启动 Build Agent。
- 不直接做 Completion Decision。
- 不新增新的自由聊天入口。

---

## 6. 依赖

- [038-agentflow-project-operating-system-runtime-foundation-v1.md](/Users/mac/Documents/AgentFlow/docs/requirements/038-agentflow-project-operating-system-runtime-foundation-v1.md)

---

## 7. 验收标准

- [ ] Project Brain 可以被 runtime 稳定读取。
- [ ] Goal Agent 成为正式上游入口。
- [ ] Project 页面能基于 Project Brain 给出 next actions。
- [ ] 系统能解释当前项目目标、当前计划和下一步原因。
- [ ] 不恢复旧 `input / execute / output` 架构作为过渡路径。

---

## 8. 验证命令

- `cargo test --workspace`
- `npm --prefix apps/desktop run build`
- `git diff --check`
