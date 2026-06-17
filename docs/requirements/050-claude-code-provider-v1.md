# 050 - Claude Code Provider V1

> 文档类型：开发需求
> 日期：2026-06-18
> 执行者：Codex
> 状态：Ready for Development

---

## 1. 背景

AgentFlow 不能把 provider 能力绑死在 Codex 上。

在不新增第二套 authority 的前提下，需要把 Claude Code provider 接进同一套 runtime。

---

## 2. 用户目标

同一条 issue flow，在更换 provider 时，业务 authority 和任务状态逻辑不变化。

---

## 3. 范围

### 3.1 必须做

1. 接入 Claude Code provider adapter。
2. 支持 launch / poll / cancel / logs。
3. 接入统一 capability matrix。
4. 接入统一 session governance。

### 3.2 涉及模块

- `crates/mcp/**`
- `crates/agent-dispatcher/**`
- `crates/workflow-runtime/**`
- `crates/projection/**`

---

## 4. 关键设计要求

### 4.1 同 authority

- Claude provider 不能单独发明一套业务状态。

### 4.2 同 workflow

- 相同 issue 在 Codex / Claude 下，workflow 语义一致。

### 4.3 同 projection

- Desktop 看到的是同一套 projection，不看 provider 私有状态。

---

## 5. 非目标

- 不为 Claude 单独做一套业务流程。
- 不引入新的聊天驱动多 Agent 模式。

---

## 6. 依赖

- [049-codex-provider-hardening-v1.md](/Users/mac/Documents/AgentFlow/docs/requirements/049-codex-provider-hardening-v1.md)
- [051-provider-capability-matrix-v1.md](/Users/mac/Documents/AgentFlow/docs/requirements/051-provider-capability-matrix-v1.md)

---

## 7. 验收标准

- [ ] Claude Code 可以作为等价 provider 接入。
- [ ] 同一条 issue flow 不因 provider 不同而改变 authority。
- [ ] launch / poll / cancel / logs 能进入统一 runtime 语义。

---

## 8. 验证命令

- `cargo test --workspace`
- `npm --prefix apps/desktop run build`
- `git diff --check`
