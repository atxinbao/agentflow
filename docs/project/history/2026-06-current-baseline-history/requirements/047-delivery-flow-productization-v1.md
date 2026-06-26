# 047 - Delivery Flow Productization V1

> 文档类型：开发需求
> 日期：2026-06-18
> 执行者：Codex
> 状态：Ready for Development

---

## 1. 背景

Delivery 不应继续混在 Work Loop 里，也不应再退回旧 `output/release` 目录模型。

当前需要把 Delivery 定义为：

- 一条独立流程；
- 一组统一公开交付记录；
- 一套清晰的 summary / changelog / release note 组织方式。

---

## 2. 用户目标

用户完成一个任务后，能清楚看到最终交付整理结果，而不是只剩一堆底层运行事实。

---

## 3. 范围

### 3.1 必须做

1. 建立 Delivery Flow runtime。
2. 建立 Delivery Summary 模型。
3. 把公开交付统一到 PR/MR body、`CHANGELOG.md`、release notes。
4. 让任务页与项目页读取 delivery summary，而不是直接读散乱事实。

### 3.2 涉及模块

- `crates/release/**`
- `crates/projection/**`
- `crates/workflow-runtime/**`
- `apps/desktop/src/**`

---

## 4. 关键设计要求

### 4.1 Delivery 独立于 Work Loop

- Work Loop 负责执行。
- Delivery Flow 负责整理和公开交付。

### 4.2 本地事实与公开交付分层

- 本地 runtime facts 留在 `.agentflow`。
- 对外读者看的交付记录进入 PR/MR body、`CHANGELOG.md`、release notes。

### 4.3 任务页可读

- 用户在任务页能看懂本次交付是什么，而不是只看到内部路径。

---

## 5. 非目标

- 不把 Delivery 再塞回 Work Loop。
- 不做新的发布平台。

---

## 6. 依赖

- [046-audit-flow-productization-v1.md](/Users/mac/Documents/AgentFlow/docs/requirements/046-audit-flow-productization-v1.md)

---

## 7. 验收标准

- [ ] Delivery 有清晰产物模型。
- [ ] 本地事实与公开交付边界稳定。
- [ ] 任务页、项目页可以读取 delivery summary。
- [ ] 不恢复旧 `.agentflow/output/release/**` 交付模型。

---

## 8. 验证命令

- `cargo test --workspace`
- `npm --prefix apps/desktop run build`
- `git diff --check`
