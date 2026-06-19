# 063 - Audit Page Independent Acceptance Surface V1

> 文档类型：开发需求
> 日期：2026-06-19
> 执行者：Codex
> 状态：Ready for Development

---

## 1. 背景

AgentFlow 已经明确：

- 审计是独立流程；
- 审计不属于执行内部；
- 审计不属于交付内部；
- 审计页要成为独立验收面。

当前需要进一步收口的是：任务页只挂审计入口和审计摘要，而真正的审计结论、evidence gap、repair recommendation 必须统一留在审计页。

---

## 2. 用户目标

用户进入审计页时，可以直接看懂：

- 哪些任务需要审计；
- 当前审计状态是什么；
- 风险点是什么；
- 证据缺口是什么；
- 是否通过、阻断或返工。

一句话：

```text
审计页要成为独立验收面，而不是执行页或交付页的附属区。
```

---

## 3. 范围

### 3.1 必须做

1. 明确审计页作为独立验收面。
2. 审计列表展示待审计对象和状态。
3. 审计详情展示 findings、evidence gap、结论、后续动作。
4. 任务页只展示审计摘要和入口。
5. 任务页与审计页职责边界固定。

### 3.2 涉及模块

- `apps/desktop/src/**`
- `crates/audit/**`
- `crates/projection/**`

---

## 4. 关键设计要求

### 4.1 审计是独立流程

任务页不能替代审计页给出完整审计结论。

### 4.2 审计页以判断为中心

审计页必须围绕这些内容组织：

- 审计状态
- Findings
- Evidence Gap
- 通过 / 阻断 / 返工
- Repair Recommendation

### 4.3 与任务页关系清晰

任务页负责：

- 显示是否需要审计
- 显示审计摘要
- 提供入口

审计页负责：

- 完整判断
- 完整审计事实
- 完整后续动作

---

## 5. 非目标

- 不在本需求里重构 release 页面。
- 不在本需求里修改 work loop 执行逻辑。
- 不在本需求里改 public delivery 标准。

---

## 6. 依赖

- [059-agentflow-desktop-frontend-information-architecture-v1.md](/Users/mac/Documents/AgentFlow/docs/requirements/059-agentflow-desktop-frontend-information-architecture-v1.md)
- [046-audit-flow-productization-v1.md](/Users/mac/Documents/AgentFlow/docs/requirements/046-audit-flow-productization-v1.md)
- [055-external-audit-and-review-surface-v1.md](/Users/mac/Documents/AgentFlow/docs/requirements/055-external-audit-and-review-surface-v1.md)

---

## 7. 验收标准

- [ ] 审计页是独立验收面，不再是执行页附属区。
- [ ] 任务页只保留审计摘要与入口。
- [ ] 审计页能完整展示审计状态、结论和后续动作。
- [ ] 用户能明确区分“执行完成”和“审计通过”。

---

## 8. 验证命令

- `npm --prefix apps/desktop run build`
- `git diff --check`
- Browser Preview / Desktop smoke：审计页列表与详情展示
