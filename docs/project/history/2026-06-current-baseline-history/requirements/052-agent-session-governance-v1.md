# 052 - Agent Session Governance V1

> 文档类型：开发需求
> 日期：2026-06-18
> 执行者：Codex
> 状态：Ready for Development

---

## 1. 背景

Provider 能力扩展后，必须统一治理 session：

- claim；
- timeout；
- cancel；
- takeover；
- retry。

否则 session 很容易反过来污染业务状态。

---

## 2. 用户目标

用户不需要关心 provider 的会话细节，也不会因为 session 出问题而丢失项目主链状态。

---

## 3. 范围

### 3.1 必须做

1. 定义 session policy。
2. 定义 timeout policy。
3. 定义 takeover policy。
4. 定义 retry / cancel policy。
5. 建立 session-level governance facts。

### 3.2 涉及模块

- `crates/agent-dispatcher/**`
- `crates/mcp/**`
- `crates/workflow-runtime/**`
- `crates/projection/**`

---

## 4. 关键设计要求

### 4.1 Session 受治理，不是自由生长

- session 生命周期必须进入统一规则。

### 4.2 失败不破坏业务状态

- session 死掉，不代表 issue authority 失真。

### 4.3 takeover 可解释

- 谁接管；
- 为什么接管；
- 接管后状态如何恢复。

---

## 5. 非目标

- 不让 session 自己升级成业务 authority。
- 不引入多 Agent 自由群聊协调。

---

## 6. 依赖

- [049-codex-provider-hardening-v1.md](/Users/mac/Documents/AgentFlow/docs/requirements/049-codex-provider-hardening-v1.md)
- [050-claude-code-provider-v1.md](/Users/mac/Documents/AgentFlow/docs/requirements/050-claude-code-provider-v1.md)
- [051-provider-capability-matrix-v1.md](/Users/mac/Documents/AgentFlow/docs/requirements/051-provider-capability-matrix-v1.md)

---

## 7. 验收标准

- [ ] 多 session 生命周期受统一治理。
- [ ] session 失败不会破坏业务状态。
- [ ] claim / timeout / cancel / takeover / retry 规则清晰。

---

## 8. 验证命令

- `cargo test --workspace`
- `git diff --check`
