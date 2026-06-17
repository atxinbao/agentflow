# 055 - External Audit And Review Surface V1

> 文档类型：开发需求
> 日期：2026-06-18
> 执行者：Codex
> 状态：Ready for Development

---

## 1. 背景

外部 reviewer / 审计者不应该必须进入本地 runtime 事实目录，才能理解一次交付。

需要一层统一 surface，表达：

- 目标；
- 范围；
- 交付；
- 证据；
- 风险；
- 审计结论。

---

## 2. 用户目标

外部 reviewer 可以直接拿到统一阅读面，而不是被迫理解 `.agentflow` 内部结构。

---

## 3. 范围

### 3.1 必须做

1. 建立 external review summary。
2. 建立 audit summary surface。
3. 建立 evidence index surface。
4. 建立 review handoff package。

### 3.2 涉及模块

- `crates/audit/**`
- `crates/release/**`
- `crates/projection/**`
- `docs/**`

---

## 4. 关键设计要求

### 4.1 面向外部读者

- 不假设外部 reviewer 理解本地 runtime 目录。

### 4.2 信息够用，但不暴露内部噪音

- 重点是目标、范围、交付、证据、风险。
- 不把内部中间态一股脑倒出去。

### 4.3 不新做门户

- 可以是统一文档、summary、handoff package。
- 不需要新建外部门户产品。

---

## 5. 非目标

- 不做新的外部门户产品。
- 不要求外部 reviewer 进入本地 `.agentflow` 目录。

---

## 6. 依赖

- [046-audit-flow-productization-v1.md](/Users/mac/Documents/AgentFlow/docs/requirements/046-audit-flow-productization-v1.md)
- [053-public-delivery-standardization-v1.md](/Users/mac/Documents/AgentFlow/docs/requirements/053-public-delivery-standardization-v1.md)
- [054-release-runtime-v1.md](/Users/mac/Documents/AgentFlow/docs/requirements/054-release-runtime-v1.md)

---

## 7. 验收标准

- [ ] 外部 reviewer 能读懂一次交付。
- [ ] 不需要直接进入本地运行事实目录。
- [ ] 目标、范围、交付、证据和风险都有统一阅读面。

---

## 8. 验证命令

- `cargo test --workspace`
- `git diff --check`
