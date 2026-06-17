# 046 - Audit Flow Productization V1

> 文档类型：开发需求
> 日期：2026-06-18
> 执行者：Codex
> 状态：Ready for Development

---

## 1. 背景

Audit 现在已经是独立流程方向，但还需要正式产品化，尤其是：

- findings；
- evidence gap；
- repair recommendation；
- 任务页 / 项目页中的 audit summary。

---

## 2. 用户目标

用户能把 Audit 当成真正独立的一条流程来理解，而不是 Work Loop 的尾巴。

---

## 3. 范围

### 3.1 必须做

1. 建立正式 Audit Flow runtime。
2. 建立 audit result read model。
3. 在任务页 / 项目页展示 audit summary。
4. 清晰表达 findings、evidence gap、repair recommendation。

### 3.2 涉及模块

- `crates/audit/**`
- `crates/projection/**`
- `crates/workflow-runtime/**`
- `apps/desktop/src/**`

---

## 4. 关键设计要求

### 4.1 Audit 独立

- Audit 不直接执行修复。
- Audit 不混入 Work Agent run loop。

### 4.2 对用户可读

- findings 必须可读；
- gap 必须可解释；
- repair recommendation 必须清楚边界。

### 4.3 接得住 Completion

- Audit 结果必须能进入 Completion 决策。

---

## 5. 非目标

- 不让 Audit Agent 自动修代码。
- 不做新的外部审计平台。

---

## 6. 依赖

- [045-work-loop-hardening-v1.md](/Users/mac/Documents/AgentFlow/docs/requirements/045-work-loop-hardening-v1.md)

---

## 7. 验收标准

- [ ] Audit 作为独立阶段可运行、可展示。
- [ ] findings 和 repair recommendation 对用户可读。
- [ ] Audit 结果能被任务页、项目页和 Completion 读取。

---

## 8. 验证命令

- `cargo test --workspace`
- `npm --prefix apps/desktop run build`
- `git diff --check`
