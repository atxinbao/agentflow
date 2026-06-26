# 040 - Requirement To Goal / Plan Preview Runtime V1

> 文档类型：开发需求
> 日期：2026-06-18
> 执行者：Codex
> 状态：Ready for Development

---

## 1. 背景

Project Brain 接入 runtime 后，下一步不是直接生成执行合同，而是把原始需求先转成：

```text
Goal Draft Preview
Plan Draft Preview
```

当前系统还缺这一层正式 preview-first / confirm-first runtime。

---

## 2. 用户目标

用户提出一个需求后，AgentFlow 先帮用户整理目标和计划草案，用户确认后，才继续 materialize 成 `SpecProject / SpecIssue`。

一句话：

```text
原始需求不能直接变成任务，必须先经过 Goal / Plan Preview。
```

---

## 3. 范围

### 3.1 必须做

1. Requirement intake 进入 Goal Draft Preview。
2. Goal Draft Preview 进入 Plan Draft Preview。
3. 建立明确的 confirmation gate。
4. 确认后才能 materialize 成 `SpecProject / SpecIssue`。
5. 取消或修改时，旧 preview 不得误入 Work Flow。

### 3.2 涉及模块

- `crates/spec/**`
- `crates/workflow-runtime/**`
- `crates/projection/**`
- `apps/desktop/src/**`

---

## 4. 关键设计要求

### 4.1 Preview First

- 用户先看 Goal / Plan Draft。
- 系统不能绕过 preview 直接写执行合同。

### 4.2 Confirm First

- 只有确认后，才允许写入 `SpecProject / SpecIssue`。
- 未确认状态不得推进到任务执行。

### 4.3 可解释

- Goal Draft 要能回答“为什么这样理解需求”。
- Plan Draft 要能回答“为什么拆成这些步骤”。

---

## 5. 非目标

- 不直接执行任务。
- 不直接进入 Build Agent run loop。
- 不在本需求里做 Goal Recheck。

---

## 6. 依赖

- [039-project-brain-runtime-entry-v1.md](/Users/mac/Documents/AgentFlow/docs/requirements/039-project-brain-runtime-entry-v1.md)

---

## 7. 验收标准

- [ ] 原始需求不会直接进入 Work Flow。
- [ ] Goal Draft Preview 与 Plan Draft Preview 都有正式投影。
- [ ] 确认前不会写执行合同。
- [ ] 确认后才能 materialize 成 `SpecProject / SpecIssue`。
- [ ] 取消、重做、修改不会污染后续 runtime。

---

## 8. 验证命令

- `cargo test --workspace`
- `npm --prefix apps/desktop run build`
- `git diff --check`
