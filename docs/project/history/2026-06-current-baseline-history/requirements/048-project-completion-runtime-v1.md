# 048 - Project Completion Runtime V1

> 文档类型：开发需求
> 日期：2026-06-18
> 执行者：Codex
> 状态：Ready for Development

---

## 1. 背景

项目完成判断不能只靠：

```text
所有 issue done
```

因为项目是否真正完成，还要看：

- 交付是否成立；
- 审计是否通过；
- Goal Recheck 怎么判断。

---

## 2. 用户目标

项目完成必须是一个正式 runtime 决策，而不是“任务都绿了”的错觉。

---

## 3. 范围

### 3.1 必须做

1. 建立 Completion evaluation。
2. 建立 Completion Decision。
3. 支持 `accepted / complete / continue / adjust / pause` 等结果。
4. 让 Project 状态依赖 completion runtime，而不是简单统计。

### 3.2 涉及模块

- `crates/workflow-runtime/**`
- `crates/projection/**`
- `crates/state/**`
- `apps/desktop/src/**`

---

## 4. 关键设计要求

### 4.1 Delivery 不等于 Completion

- delivery produced 不是 project done。

### 4.2 Audit 不等于 Completion

- audit 通过是完成判断的一部分，但不是唯一条件。

### 4.3 Goal Recheck 参与最终接受

- Goal Agent 要参与 completion decision。

---

## 5. 非目标

- 不直接做 Release Runtime。
- 不把项目完成简化成 PR 合并。

---

## 6. 依赖

- [041-goal-recheck-and-completion-decision-runtime-v1.md](/Users/mac/Documents/AgentFlow/docs/requirements/041-goal-recheck-and-completion-decision-runtime-v1.md)
- [046-audit-flow-productization-v1.md](/Users/mac/Documents/AgentFlow/docs/requirements/046-audit-flow-productization-v1.md)
- [047-delivery-flow-productization-v1.md](/Users/mac/Documents/AgentFlow/docs/requirements/047-delivery-flow-productization-v1.md)

---

## 7. 验收标准

- [ ] Project 完成判断具备独立 runtime 规则。
- [ ] 项目不会因为单纯 issue done 就直接 complete。
- [ ] completion decision 能被 Project 页面解释。

---

## 8. 验证命令

- `cargo test --workspace`
- `npm --prefix apps/desktop run build`
- `git diff --check`
