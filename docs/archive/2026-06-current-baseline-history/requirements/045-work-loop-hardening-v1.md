# 045 - Work Loop Hardening V1

> 文档类型：开发需求
> 日期：2026-06-18
> 执行者：Codex
> 状态：Ready for Development

---

## 1. 背景

Build Agent 的 Work Loop 已经能跑，但 run lifecycle、retry、resume、中断恢复、preflight / verify / review 收口还不够稳定。

---

## 2. 用户目标

任务执行过程中，即使中断、重试、恢复，也不会把业务状态打乱。

一句话：

```text
Work Loop 要从“能跑”提升到“稳定可长期运行”。
```

---

## 3. 范围

### 3.1 必须做

1. 收口 run lifecycle。
2. 收口 pause / resume / retry。
3. 收口 interruption recovery。
4. 强化 preflight / verify / review 过程一致性。
5. 让事件链、projection、issue 状态同步稳定。

### 3.2 涉及模块

- `crates/workflow-runtime/**`
- `crates/task-loop/**`
- `crates/event-store/**`
- `crates/projection/**`
- `crates/state/**`

---

## 4. 关键设计要求

### 4.1 状态推进必须稳定

- `todo -> in_progress -> in_review -> done`
- `blocked / cancel` 为意外分支

### 4.2 中断恢复不能污染主链

- dirty run、旧 run、失败 run 不能覆盖当前主链状态。

### 4.3 验证是正式阶段

- verify 不是附属脚本。
- verify 是 Work Loop 正式阶段的一部分。

---

## 5. 非目标

- 不新增新的业务流程类型。
- 不扩展更多 provider 类型。

---

## 6. 依赖

- [038-agentflow-project-operating-system-runtime-foundation-v1.md](/Users/mac/Documents/AgentFlow/docs/requirements/038-agentflow-project-operating-system-runtime-foundation-v1.md)

---

## 7. 验收标准

- [ ] 中断恢复不再打乱业务状态。
- [ ] retry / resume 有稳定事件链。
- [ ] run lifecycle、issue 状态、projection 同步一致。
- [ ] verify / review 成为正式阶段，而不是临时补丁。

---

## 8. 验证命令

- `cargo test --workspace`
- `npm --prefix apps/desktop run build`
- `git diff --check`
