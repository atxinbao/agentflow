# 060 - Task Page State Timeline And Event Stream V1

> 文档类型：开发需求
> 日期：2026-06-19
> 执行者：Codex
> 状态：Ready for Development

---

## 1. 背景

任务页已经是 AgentFlow Desktop 的主工作面，但当前右侧详情仍然混有静态卡片式信息组织，状态流和事件流还没有成为真正的主骨架。

059 已经明确：

- 任务页必须成为主工作台；
- 状态流是任务阅读主骨架；
- 事件流是任务事实流；
- 当前 / 已完成 / 未来状态必须用不同语义展示。

因此需要先单独完成任务页右侧主结构重构，把“状态时间线 + 事件流”收成统一阅读模型。

---

## 2. 用户目标

用户点开一条任务后，不需要跳去别的页面，就能直接看懂：

- 当前在哪个状态；
- 这个状态已经发生了什么；
- 前面哪些阶段已经完成；
- 后面哪些阶段还在等待。

一句话：

```text
任务页右侧要从静态详情面板升级成状态时间线和事件流主视图。
```

---

## 3. 范围

### 3.1 必须做

1. 用统一状态时间线重构任务页右侧骨架。
2. 每个状态节点下挂事件流。
3. 当前状态支持实时事件显示。
4. 已完成状态支持历史事件显示。
5. 未来状态只显示等待，不显示假日志。
6. blocked / cancel 状态必须展示原因。

### 3.2 涉及模块

- `apps/desktop/src/**`
- `crates/projection/**`
- `crates/state/**`

---

## 4. 关键设计要求

### 4.1 状态是骨架

右侧详情必须围绕统一状态展开：

- `backlog`
- `todo`
- `in_progress`
- `in_review`
- `done`
- `blocked`
- `cancel`

### 4.2 事件是事实流

状态下不展示抽象说明卡片堆，而是展示真实事件流：

- 事件标题
- 事件时间
- actor / authority
- artifact refs
- 当前阶段日志片段

### 4.3 当前 / 已完成 / 未来展示边界明确

- 当前：实时事件
- 已完成：历史事件
- 未来：只显示等待

---

## 5. 非目标

- 不在本需求里合并交付槽位。
- 不在本需求里重构审计页。
- 不在本需求里重做左侧任务树排序。

---

## 6. 依赖

- [059-agentflow-desktop-frontend-information-architecture-v1.md](/Users/mac/Documents/AgentFlow/docs/requirements/059-agentflow-desktop-frontend-information-architecture-v1.md)
- [042-task-page-productization-v1.md](/Users/mac/Documents/AgentFlow/docs/requirements/042-task-page-productization-v1.md)

---

## 7. 验收标准

- [ ] 右侧详情主骨架变为状态时间线。
- [ ] 每个状态节点下可以展示对应事件流。
- [ ] 当前状态显示实时事件。
- [ ] 已完成状态显示历史事件。
- [ ] 未来状态不显示假日志。
- [ ] blocked / cancel 有明确原因。

---

## 8. 验证命令

- `npm --prefix apps/desktop run build`
- `git diff --check`
- Browser Preview / Desktop smoke：任务页状态时间线与事件流展示
