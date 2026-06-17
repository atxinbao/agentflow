# 053 - Public Delivery Standardization V1

> 文档类型：开发需求
> 日期：2026-06-18
> 执行者：Codex
> 状态：Ready for Development

---

## 1. 背景

任务的本地 runtime 事实在 `.agentflow` 内部，但外部读者真正能看到的交付，应当统一收口到公开表面：

- PR/MR body；
- `CHANGELOG.md`；
- release notes。

---

## 2. 用户目标

不进入本地 runtime 目录，外部 reviewer 也能看懂一次交付做了什么、验证了什么、影响是什么。

---

## 3. 范围

### 3.1 必须做

1. 统一 public delivery format。
2. 统一 changelog conventions。
3. 统一 release note template。
4. 统一 delivery summary template。

### 3.2 涉及模块

- `crates/release/**`
- `crates/projection/**`
- `docs/**`
- PR/MR 模板与公开说明模板

---

## 4. 关键设计要求

### 4.1 公开交付不依赖隐藏目录

- 外部读者不需要进入 `.agentflow` 才能理解交付。

### 4.2 任务级和版本级边界清晰

- PR/MR body 是任务级；
- `CHANGELOG.md` / release notes 是版本级。

### 4.3 与本地事实对齐

- 公开交付记录要和本地事实一致，但不直接暴露内部路径。

---

## 5. 非目标

- 不在本阶段做完整发布平台整合。
- 不新增独立公开门户。

---

## 6. 依赖

- [047-delivery-flow-productization-v1.md](/Users/mac/Documents/AgentFlow/docs/requirements/047-delivery-flow-productization-v1.md)
- [048-project-completion-runtime-v1.md](/Users/mac/Documents/AgentFlow/docs/requirements/048-project-completion-runtime-v1.md)

---

## 7. 验收标准

- [ ] 公开交付格式统一。
- [ ] 外部读者能理解一次交付结果。
- [ ] 本地事实与公开记录边界稳定。

---

## 8. 验证命令

- `cargo test --workspace`
- `git diff --check`
