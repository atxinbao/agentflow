# 057 - v0.3.0 发布前审计重基线 V1

> 文档类型：审计重基线
> 日期：2026-06-19
> 执行者：Codex
> 状态：Audit Baseline / Ready for follow-up requirements

---

## 1. 背景

此前的 `v0.3.0` 发布前审计，基线停在 PR `#261` 合入后的旧 `main`：

```text
ec7f0b358bde66909b53e2ec19c2052d83f1f84c
```

之后仓库已经继续合入：

- `056A` Task Loop Strict Sequencing Guards
- `056B` Provider Closeout Capability And Build Agent Done Gate
- `056C` Projection Semantics And Acceptance Audit

因此，旧审计结论不能原样继续执行。

现在需要做两件事：

1. 把已经部分修复的问题从旧 P0 里降级或改写；
2. 把仍然阻断 `v0.3.0 stable` 的问题重新排序，作为新的修复入口。

本文档的基线是当前仓库本地 `main`：

```text
5c7e936
```

---

## 2. 审计结论

### 当前发布判断

```text
NO-GO for v0.3.0 Stable
```

大白话：

> `v0.3.0` 现在已经不是“主流程完全没接起来”，而是“顺序门禁和 done gate 已经补上，但可信验证、原子事件、provider 边界和正式产品入口还没收口”。

因此，当前版本更准确的状态是：

```text
v0.3.0 stabilized development snapshot
```

还不能认定为一条真正可发布的稳定链路：

```text
Requirement
-> Goal / Plan Confirm
-> Work Loop
-> Trusted Validation
-> PR / MR Merge
-> Public Delivery
-> Audit
-> Completion
-> Release
```

---

## 3. 已部分修复，旧审计口径需要调整的项

### 3.1 顺序门禁不再是原样 P0

`056A` 已经把 project issue 的顺序门禁和单活跃槽位约束补到 runtime。

这意味着旧审计里“任务顺序仍然可以随意绕过”的表述已经不准确。

现在更准确的表述应当是：

```text
顺序门禁已经形成，但仍需要和后续可信验证、closeout attestation 一起收口成完整 stable gate。
```

### 3.2 Done Gate 不再是原样 P0

`056B` 已经把：

- merge proof
- issue close proof
- public delivery closeout

纳入 `done` 写回前置条件。

所以旧审计里“PR merge 后几乎可以直接 done”的表述也需要收回。

现在更准确的表述应当是：

```text
done gate 已形成，但 closeout proof 仍然偏调用方声明，还不是 provider 侧可信证明。
```

### 3.3 Projection 语义不再是原样 P0

`056C` 已经把 projection 语义收紧到：

- `issue.pr.merged` 仍然停留在 `in_review`
- `issue.completed` 才进入 `done`

所以旧审计里“状态投影直接误导用户”的描述，也应该改成“剩余一致性问题”，而不是原样阻断。

### 3.4 `issue.completed` 写入顺序问题已修正

当前 closeout 路径已经先写 public delivery，再写 `issue.completed`。

因此旧审计里“先 completed 再写交付记录”的原文已经过时，不应继续沿用。

---

## 4. 仍然阻断 v0.3.0 Stable 的 P0

### P0-1：Project Brain / Completion / Release 缺少正式产品入口

当前仓库已经有相关 crate 内部能力，但 CLI 和 Desktop 仍缺正式一等入口。

当前可见的正式命令仍然主要停留在：

```text
build-agent
task-loop
agent-dispatcher
projection
release summary / write-docs
```

还缺：

```text
agentflow project intake
agentflow project preview-goal
agentflow project confirm-goal
agentflow project confirm-plan
agentflow project materialize

agentflow completion inspect
agentflow completion decide

agentflow release prepare
agentflow release confirm
agentflow release publish
```

这条仍然是稳定版阻断。

### P0-2：验证结果仍然不是 runtime 自己执行

当前 `BuildAgentCompletionRequest` 仍允许调用方直接提交：

- `validationCommands`
- `exitCode`
- `stdout`
- `stderr`
- `changedFiles`

review preparation 仍然把这些调用方上报内容写入 command records，并据此生成 validation / evidence。

这意味着：

```text
当前验证更像 Agent 自己填写的结果，
不是 AgentFlow Runtime 独立执行并签发的结果。
```

这条仍然是最硬的 P0 之一。

### P0-3：Closeout Attestation 仍然不可信

虽然 closeout gate 已存在，但当前 CLI 仍然接受：

- `--merged`
- `--issue-closed`
- `--closed-at`

这些值再写入本地 closeout proof。

也就是说：

```text
done gate 已经有了，
但 merge / issue closed 的事实仍然偏调用方声明，
不是 provider 查询后的可信事实。
```

这条保留为 P0，但口径应改成：

```text
Trusted Closeout Attestation Missing
```

### P0-4：Event Store / Dispatcher 仍然缺原子 claim

当前事件追加仍是：

- 先读已有事件
- 再按 idempotency key 判断
- 再 append

dispatcher 仍是：

- 先读 launch request
- 先拉起 provider session
- 再补 `launch.claimed`

这不是 claim-before-spawn，也不是单次原子事务。

这条仍然是核心 P0。

### P0-5：Issue / Project / Release 的 ID 与路径边界仍然太弱

当前：

- `issueId` 主要只校验 `<prefix>-<digits>`
- `projectId` 主要只校验非空
- 路径 containment 还不是统一 typed model

这对于 stable runtime 还不够。

这条仍然保留。

### P0-6：Provider Runtime Boundary 仍然过松

当前 provider 默认边界仍偏宽：

- Codex 仍使用 `workspace-write`
- approval policy 仍是 `never`
- 仍带有 `ignore-user-config` / `ignore-rules` 一类放宽入口
- Claude 仍默认 `bypassPermissions`

同时，外部 provider 的隔离、超时、取消、退出证明和 worktree 约束还没有收成稳定策略。

这条仍然是 P0。

### P0-7：Work / Delivery / Audit / Completion / Release 的 saga 仍未彻底拆开

旧审计里“完成顺序错乱”这部分已经被修正。

但更大的结构问题仍在：

```text
当前 closeout 路径仍然承担过多职责，
还不是显式分层的 workflow saga。
```

因此这条保留，但口径应改成：

```text
Workflow Saga Separation Incomplete
```

### P0-8：没有 CI / 真实发布级 E2E 证据

当前仓库没有正式的 `.github/workflows/**` 发布链，也没有真实 release gate 的外部证据闭环。

因此：

```text
即使本地链路能跑，也还不能证明 v0.3.0 stable 可重复、可托管、可外部验收。
```

这条仍然保留。

---

## 5. 调整后的优先级

`v0.3.0 stable` 的修复顺序不应再沿用旧审计原顺序。

现在建议按下面顺序推进：

1. 可信验证 runtime
2. 原子 event / claim / run allocation
3. ID / 路径安全模型
4. provider runtime isolation / supervision
5. Project Brain / Completion / Release 正式入口
6. provider 查询驱动的可信 closeout attestation
7. Work / Delivery / Audit / Completion / Release saga 拆分
8. CI / release gate / 真实 E2E 证据

---

## 6. 本文档对后续开发的约束

从这份重基线开始：

1. 不再按旧审计原文逐条机械补丁。
2. 任何新的修复需求，都必须以本文档的口径为准。
3. `056A/056B/056C` 视为已完成的第一轮审计收口，不得重复开发。
4. 后续 requirement / GitHub issues 应围绕“可信验证、原子事件、provider 边界、正式入口和 stable gate”重新拆分。

---

## 7. 下一步

下一步不应继续直接改代码，而应先生成一份新的稳定版修复需求包：

```text
058 - v0.3.0 Stable Gate Remediation
```

用它来承接新的 issue 拆分、依赖顺序和验收标准。
