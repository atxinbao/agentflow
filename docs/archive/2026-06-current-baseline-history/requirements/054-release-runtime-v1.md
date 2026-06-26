# 054 - Release Runtime V1

> 文档类型：开发需求
> 日期：2026-06-18
> 执行者：Codex
> 状态：Ready for Development

---

## 1. 背景

release 不能继续是零散人工动作拼接。

在 `v0.3.0` 之后，release 应该成为：

- completion 之后的正式阶段；
- 有正式 gate；
- 有正式 release facts；
- 有正式 public note generation。

---

## 2. 用户目标

项目完成后，release 是一条正式流程，而不是手工补动作。

---

## 3. 范围

### 3.1 必须做

1. 建立 release runtime。
2. 建立 release gate。
3. 建立 release facts。
4. 建立 public note generation。

### 3.2 涉及模块

- `crates/release/**`
- `crates/workflow-runtime/**`
- `crates/projection/**`
- `docs/**`

---

## 4. 关键设计要求

### 4.1 Release 在 Completion 之后

- 不是任何任务一完成就能直接 release。

### 4.2 Release Facts 与 Public Delivery 一致

- release facts、public delivery、completion state 必须一致。

### 4.3 仍然是项目级动作

- release 是项目级阶段，不是单条 task 的附属动作。

---

## 5. 非目标

- 不做所有渠道分发自动化。
- 不做完整发布平台接入。

---

## 6. 依赖

- [053-public-delivery-standardization-v1.md](/Users/mac/Documents/AgentFlow/docs/requirements/053-public-delivery-standardization-v1.md)

---

## 7. 验收标准

- [ ] release 成为 project completion 后的正式阶段。
- [ ] release facts 与 public delivery 一致。
- [ ] release 不再依赖零散人工动作拼接。

---

## 8. 验证命令

- `cargo test --workspace`
- `git diff --check`
