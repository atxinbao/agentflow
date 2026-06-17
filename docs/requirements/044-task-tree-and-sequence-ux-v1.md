# 044 - Task Tree And Sequence UX V1

> 文档类型：开发需求
> 日期：2026-06-18
> 执行者：Codex
> 状态：Ready for Development

---

## 1. 背景

任务树已经有 project / issue 分组，但执行顺序、依赖顺序、当前 / 过去 / 未来关系还需要更稳定、更可读。

---

## 2. 用户目标

用户看左侧任务树时，能直接看懂：

- 哪些已经做完；
- 哪些正在做；
- 哪些还在等待；
- 为什么等待；
- 项目内部 issue 和独立任务的关系。

---

## 3. 范围

### 3.1 必须做

1. 优化 project / issue 树结构。
2. 优化 standalone issue 分组。
3. 让依赖顺序和执行顺序可见。
4. 让当前 / 过去 / 未来三种状态组织稳定。
5. 让排序规则统一且可解释。

### 3.2 涉及模块

- `apps/desktop/src/**`
- `crates/projection/**`
- `crates/state/**`

---

## 4. 关键设计要求

### 4.1 顺序来自依赖，不来自随机时间

- 排序要以可执行顺序为主。
- 最少依赖、最先可执行的任务优先。

### 4.2 project 与 standalone 边界清晰

- project 下 issue 是子层；
- standalone issue 是平级单项任务；
- 不混成一团。

### 4.3 状态点与结构配合

- 当前、过去、未来的状态提示要和树结构一起工作。
- 不再只是散乱 badge。

---

## 5. 非目标

- 不做图形化 DAG 编辑器。
- 不在本需求里扩展全局搜索系统。

---

## 6. 依赖

- [042-task-page-productization-v1.md](/Users/mac/Documents/AgentFlow/docs/requirements/042-task-page-productization-v1.md)
- [043-project-page-productization-v1.md](/Users/mac/Documents/AgentFlow/docs/requirements/043-project-page-productization-v1.md)

---

## 7. 验收标准

- [ ] 左侧任务树排序和组织稳定。
- [ ] 用户能看出当前 / 过去 / 未来任务关系。
- [ ] project / issue / standalone issue 层级清晰。
- [ ] 依赖顺序能被用户直接感知。

---

## 8. 验证命令

- `npm --prefix apps/desktop run build`
- `git diff --check`
- Browser Preview / Desktop smoke：左侧任务树排序与分组
