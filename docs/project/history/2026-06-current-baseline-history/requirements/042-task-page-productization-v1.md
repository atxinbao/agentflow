# 042 - Task Page Productization V1

> 文档类型：开发需求
> 日期：2026-06-18
> 执行者：Codex
> 状态：Ready for Development

---

## 1. 背景

任务页已经能展示任务，但还带有明显的状态调试面板痕迹。

后续主链路要在任务页承载：

- 当前状态；
- 事件流；
- 历史日志；
- 交付信息；
- 审计摘要。

---

## 2. 用户目标

用户打开任务页时，不需要跳到别的旧栏目，就能看懂一个任务从开始到结束发生了什么。

一句话：

```text
任务页要从“状态面板”升级成真正的任务主工作台。
```

---

## 3. 范围

### 3.1 必须做

1. 重构任务页主布局。
2. 右侧以状态流 / 事件流为主视图。
3. 当前状态展示实时信息。
4. 已完成状态展示历史日志。
5. 未来状态只展示等待，不展示假日志。
6. 交付与审计摘要整合回任务页。

### 3.2 涉及模块

- `apps/desktop/src/App.tsx`
- `apps/desktop/src/AppShell.css`
- `apps/desktop/src/features/**`
- `apps/desktop/src/browserPreviewData.ts`

---

## 4. 关键设计要求

### 4.1 右侧以状态流为中心

- 不是一堆静态卡片。
- 是任务从 `backlog -> todo -> in_progress -> in_review -> done / blocked / cancel` 的可读过程。

### 4.2 当前 / 过去 / 未来区分明确

- 当前：看实时事件；
- 过去：看已发生日志；
- 未来：只看等待状态。

### 4.3 交付信息并入任务页

- 不再让旧执行页 / 旧交付页承担主链路解释。
- 任务页就是工作流阅读中心。

---

## 5. 非目标

- 不重做整套设计系统。
- 不在本需求里重做全局导航。
- 不新增自由聊天入口。

---

## 6. 依赖

- [038-agentflow-project-operating-system-runtime-foundation-v1.md](/Users/mac/Documents/AgentFlow/docs/requirements/038-agentflow-project-operating-system-runtime-foundation-v1.md)

---

## 7. 验收标准

- [ ] 任务页足以承载主链路理解。
- [ ] 用户能从任务页看懂执行、交付、审计过程。
- [ ] 当前 / 历史 / 未来三种信息展示边界清晰。
- [ ] 旧执行页 / 旧交付页不再是理解任务主链路的必要入口。

---

## 8. 验证命令

- `npm --prefix apps/desktop run build`
- `git diff --check`
- Browser Preview / Desktop smoke：任务页状态流显示
