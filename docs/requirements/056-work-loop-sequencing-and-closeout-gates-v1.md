# 056 - Work Loop Sequencing And Closeout Gates V1

> 文档类型：开发需求
> 日期：2026-06-18
> 执行者：Codex
> 状态：Ready for Development

---

## 1. 背景

`#208` 到 `#231` 的 v0.3.0 交付链，代码结果已经完成，但流程审计没有通过。

问题不在功能缺失，而在底层 loop 的门禁还不够硬：

1. project issue 的顺序约束没有被 runtime 强制执行；
2. Build Agent 的 `in_review -> done` 只检查 merge proof，没有把 issue closeout 当成硬前置；
3. GitHub / GitLab provider 只有 PR/MR 能力，没有 issue close / issue closed query 能力；
4. projection 会把 `issue.completed` 直接投影成 `done`，但 `issue.completed` 写入时机本身还不够严格。

一句话：

```text
当前链路是“结果能交付”，但还不是“流程可审计”。
```

---

## 2. 用户目标

后续每一条 issue 都必须严格按顺序推进，并且只有在完整 closeout 之后，才允许写入 `done` 并启动下一条任务。

一句话：

```text
任务链要从“能跑通”升级成“顺序不可绕过、闭环不可跳过”。
```

---

## 3. 范围

### 3.1 必须做

1. 为 project issue 增加严格顺序门禁。
2. 为 Build Agent 增加 closeout gate。
3. 为 GitHub / GitLab provider 增加 issue close / closed query 能力。
4. 调整 projection 和状态文案，让 UI 展示与真实事实一致。
5. 增加 acceptance tests，覆盖顺序门禁和闭环门禁。

### 3.2 涉及模块

- `crates/task-loop/**`
- `crates/cli/**`
- `crates/mcp/**`
- `crates/projection/**`
- `crates/state/**`
- `crates/acceptance/**`
- `docs/**`

---

## 4. 关键设计要求

### 4.1 顺序门禁必须成为 runtime 硬约束

对属于 project 的 issue：

- 不能只靠 `blocked_by`；
- 必须额外检查它在 `SpecProject.issue_ids` 里的前序 issue；
- 只有当前序 issue 全部为 `done` 或 `cancel` 时，当前 issue 才允许：
  - `backlog -> todo`
  - `todo -> in_progress`

同一个 project 在任意时刻只允许一条 issue 处于：

- `todo`
- `in_progress`
- `in_review`

不能同时存在多条活跃 issue。

### 4.2 `done` 必须等到 closeout 完成后再写

Build Agent 当前的 `complete` 不能只看 merge proof。

`in_review -> done` 至少需要同时满足：

1. validation passed
2. PR/MR created
3. PR/MR merged
4. GitHub / GitLab issue 已关闭
5. public delivery 已写出

只要其中任意一项缺失，任务必须保持在 `in_review`，不能写 `issue.completed`。

### 4.3 `done` 之后才能启动下一条 issue

当前 issue 完整 closeout 之前：

- 不允许 project loop `tick` 下一条
- 不允许自动生成下一条 launch request

只有 `issue.completed` 已成功写入，才允许 project loop 继续推进。

### 4.4 Provider 必须显式支持关单

不要再依赖 PR body 里的 `closes #xxx` 作为唯一关单路径。

Provider capability 至少要新增：

- `issue.close`
- `issue.closed_query`

Build Agent closeout 需要显式调用 provider，把当前 issue 关闭，然后再复核 closed 状态。

### 4.5 Projection 只能展示 authoritative state

UI 的时间线和状态摘要必须服从统一语义：

- `backlog`：任务已生成，等待进入执行顺序
- `todo`：已轮到当前任务，等待执行线程接管
- `in_progress`：代码与本地验证进行中
- `in_review`：验证通过，等待 PR/MR merge 与关单收口
- `done`：closeout 已完成，允许推进下一条
- `blocked` / `cancel`：异常分支

不要再出现“PR 已 merge 但 issue 未关，页面却显示 done”的情况。

---

## 5. 详细方案

### 5.1 Task Loop 顺序门禁

建议在 `task-loop` 内新增统一 guard helper，供：

- `schedule_next_issue`
- `start_issue`
- `tick`

共同使用。

至少需要四条 guard：

1. `issue.contract.complete`
2. `dependencies.done`
3. `project.sequence.predecessors.done`
4. `project.serial_slot.free`

其中：

- `project.sequence.predecessors.done`
  - 基于 `SpecProject.issue_ids` 顺序检查前序 issue
- `project.serial_slot.free`
  - 同 project 不允许多条活跃 issue 并行

### 5.2 Build Agent Closeout Proof

建议把当前 merge proof 升级成 closeout proof。

proof 至少包含：

- `provider`
- `mergeMode`
- `prUrl`
- `mergeCommit`
- `merged`
- `issueClosed`
- `closedAt`
- `publicDeliveryWritten`

`complete_build_agent_issue_from_request` 只能在 closeout proof 完整时写：

- `issue.completed`

### 5.3 Provider Capability 扩展

GitHub / GitLab provider 需要具备：

- issue close
- issue closed query

并把这两个能力写进 provider capability registry。

### 5.4 Projection 与文案收口

projection 的 authoritative state 映射要收紧：

- `issue.completed` 才能映射为 `done`
- `issue.pr.merged` 仍然只属于 `in_review`
- `in_review` 的说明要明确包含：
  - merge
  - issue close
  - public delivery closeout

### 5.5 Acceptance Tests

至少新增以下回归：

1. 前序 issue 未完成时，后序 issue 不能 schedule。
2. 手动 `start_issue` 不能绕过顺序门禁。
3. PR merged 但 issue 未关闭时，任务保持 `in_review`。
4. issue 关闭后，才允许写 `done`。
5. `done` 写回前，project loop 不能启动下一条。

---

## 6. 非目标

- 不在本需求里重构整个 workflow schema。
- 不在本需求里引入新的 provider 类型。
- 不在本需求里调整产品页面布局。
- 不在本需求里修改 requirement / goal / completion 的上游输入模型。

---

## 7. 依赖

- [045-work-loop-hardening-v1.md](/Users/mac/Documents/AgentFlow/docs/requirements/045-work-loop-hardening-v1.md)
- [049-codex-provider-hardening-v1.md](/Users/mac/Documents/AgentFlow/docs/requirements/049-codex-provider-hardening-v1.md)
- [051-provider-capability-matrix-v1.md](/Users/mac/Documents/AgentFlow/docs/requirements/051-provider-capability-matrix-v1.md)
- [052-agent-session-governance-v1.md](/Users/mac/Documents/AgentFlow/docs/requirements/052-agent-session-governance-v1.md)
- [053-public-delivery-standardization-v1.md](/Users/mac/Documents/AgentFlow/docs/requirements/053-public-delivery-standardization-v1.md)

---

## 8. 建议 issue 拆分

### 8.1 Issue A - Task Loop Strict Sequencing Guards

目标：

- 把顺序门禁落到 `task-loop`

范围：

- `crates/task-loop/**`
- `crates/state/**`

验收：

- project issue 不能抢跑
- 同一 project 不会同时有多条活跃 issue

### 8.2 Issue B - Provider Closeout Capability And Build Agent Done Gate

目标：

- 把 closeout proof 和 provider 关单能力补齐

范围：

- `crates/mcp/**`
- `crates/cli/**`

验收：

- PR merged 但 issue 未关时，不能 done
- issue closed 后，才允许 done

### 8.3 Issue C - Projection Semantics And Acceptance Audit

目标：

- 收口状态投影、文案和 acceptance 回归

范围：

- `crates/projection/**`
- `crates/acceptance/**`
- `docs/**`

验收：

- UI 时间线与 authoritative state 一致
- acceptance tests 覆盖顺序门禁与闭环门禁

---

## 9. 验收标准

- [ ] project issue 的顺序约束成为 runtime 硬门禁。
- [ ] `done` 只在完整 closeout 后写入。
- [ ] provider 具备 issue close / closed query 能力。
- [ ] project loop 不会在 closeout 未完成时推进下一条。
- [ ] projection / UI 文案与 authoritative state 一致。
- [ ] acceptance tests 能覆盖这次审计暴露出的流程缺陷。

---

## 10. 验证命令

- `cargo test --workspace`
- `npm --prefix apps/desktop run build`
- `git diff --check`
