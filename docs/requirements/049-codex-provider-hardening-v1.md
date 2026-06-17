# 049 - Codex Provider Hardening V1

> 文档类型：开发需求
> 日期：2026-06-18
> 执行者：Codex
> 状态：Ready for Development

---

## 1. 背景

Codex provider 已经打通基础链路，但距离长期稳定运行还差几件事：

- launch 稳定性；
- poll / logs 一致性；
- merge / writeback 收口；
- resume / recovery 可靠性。

---

## 2. 用户目标

用户把任务交给 Codex provider 后，不需要担心 provider 自己把业务状态打乱。

---

## 3. 范围

### 3.1 必须做

1. 强化 Codex launch 计划与状态同步。
2. 强化 session lifecycle。
3. 强化 logs / merge proof / writeback 流程。
4. 强化 resume / recovery。
5. 让 provider 失败和重试路径可解释、可重试。

### 3.2 涉及模块

- `crates/mcp/**`
- `crates/agent-dispatcher/**`
- `crates/workflow-runtime/**`
- `crates/projection/**`

---

## 4. 关键设计要求

### 4.1 Provider 不是 authority

- provider 只负责执行与回传事实。
- 状态 authority 仍在 workflow / event / projection。

### 4.2 生命周期可治理

- launch、poll、resume、cancel、merge、writeback 都要进入统一治理语义。

### 4.3 故障路径可恢复

- 失败不是静默丢失。
- 恢复要能回到主链。

---

## 5. 非目标

- 不引入第二套 authority。
- 不新增新的 provider 类型。

---

## 6. 依赖

- [045-work-loop-hardening-v1.md](/Users/mac/Documents/AgentFlow/docs/requirements/045-work-loop-hardening-v1.md)

---

## 7. 验收标准

- [ ] Codex provider 生命周期稳定。
- [ ] launch / poll / logs / merge / writeback / recovery 可解释。
- [ ] provider 故障不会污染业务状态 authority。

---

## 8. 验证命令

- `cargo test --workspace`
- `npm --prefix apps/desktop run build`
- `git diff --check`
