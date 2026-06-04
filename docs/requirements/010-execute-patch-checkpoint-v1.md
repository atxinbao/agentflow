# 010 - Execute Patch / Checkpoint V1

创建日期：2026-06-04
执行者：Codex
状态：已实现
版本：final-draft

---

## 用户目标

当前 AgentFlow 已经完成 / 正在推进：

```text
define/
= Agent 工作手册 / 规则

panel/
= 项目工作现场

input/
= 需求实时源头，包含 Spec Gate、Project、Issues

execute/
= 未来执行过程

output/
= 证据 / 审计 / 发布 / 日志

state/
= 健康 / 锁 / 会话 / 索引状态
```

现在需要进入：

```text
execute/
```

本轮不再拆成：

```text
010.1 Execute Readiness V1
010.2 Execute Command Runner V1
010.3 Execute Patch / Checkpoint V1
```

而是直接一次性做到：

```text
010 - Execute Patch / Checkpoint V1
```

大白话：

> `execute/` 不是“让 Agent 随便执行”的按钮。
> 它是一个受控施工流水线。
> 它必须从 `input/issues/` 启动，先做开工前检查，再加锁，再生成执行计划，再做 checkpoint，再执行 patch / command，再记录 diff / validation / result，最后把证据写到 `output/evidence/`。

---

## 一句话定义

> **Execute Patch / Checkpoint V1 是 AgentFlow 的受控执行层。它以 `input/issues/<issue-id>.json` 为唯一入口，创建一次 `run`，执行 preflight、lease、plan、checkpoint、patch、command record、validation、result 和 evidence。低 / 中风险 issue 可以按手册推进；高风险 issue 执行前必须有人类确认。**

---

# 1. 核心原则

## 1.1 execute 只能从 input issue 启动

允许入口：

```text
.agentflow/input/issues/<issue-id>.json
```

不允许入口：

```text
聊天消息
临时自然语言
未批准 SPEC
未登记 issue
旧 goal-tree issue
手动拼接 run
```

也就是说：

```text
No input issue, no execute.
```

---

## 1.2 execute 不是需求源

`execute/` 不负责定义需求。

需求来自：

```text
.agentflow/input/specs/approved/<spec-id>/
.agentflow/input/issues/<issue-id>.json
```

execute 只负责：

```text
执行过程
执行记录
执行证据
```

---

## 1.3 高风险 issue 执行前必须人类确认

Issue 里只保留：

```json
{
  "riskLevel": "low"
}
```

取值：

```text
low
medium
high
```

规则：

```text
low
= 不需要人类确认

medium
= 不需要人类确认

high
= 执行前需要人类确认
```

注意：

```text
Issue 模型里不新增 automation / humanGates / prAutomation。
```

人类确认记录属于执行过程，所以放在：

```text
.agentflow/execute/runs/<run-id>/confirmations/
```

不写回：

```text
.agentflow/input/issues/<issue-id>.json
```

---

## 1.4 每次写入前必须 checkpoint

只要 execute 要执行以下动作：

```text
写文件
改文件
删文件
覆盖文件
应用 patch
运行可能改变项目状态的命令
```

必须先生成 checkpoint。

大白话：

> **动手前先拍照。**

---

## 1.5 每次执行必须有证据

执行完成后必须写：

```text
.agentflow/execute/runs/<run-id>/result.json
.agentflow/output/evidence/<run-id>.json
```

没有 evidence 的 run 不能算完成。

---

## 1.6 execute 不负责发布

本轮不做：

```text
创建 PR
merge
release
deploy
rollback
remote issue
remote API
```

这些属于后续：

```text
release/
output/release/
```

---

# 2. execute 在整体流程里的位置

完整链路：

```text
define/
  ↓
panel/
  ↓
input/
  ↓
execute/
  ↓
output/
  ↓
state/
```

大白话：

```text
define = 规矩
panel = 现场
input = 施工单
execute = 施工过程
output = 施工证据
state = 当前状态
```

---

# 3. execute 主流程

```text
Input Issue
  ↓
Run Create
  ↓
Preflight
  ↓
Lease
  ↓
Risk Check
  ↓
Run Plan
  ↓
Checkpoint
  ↓
Patch / Commands
  ↓
Validation
  ↓
Diff / Review Summary
  ↓
Result
  ↓
Evidence
```

大白话：

```text
拿施工单
→ 开工前检查
→ 占坑上锁
→ 看风险
→ 写执行计划
→ 动手前拍照
→ 改文件 / 跑命令
→ 验证
→ 记录改动
→ 写结果
→ 留证据
```

---

# 4. 目录结构

目标结构：

```text
.agentflow/execute/
├── manifest.json
├── index.json
│
├── runs/
│   └── <run-id>/
│       ├── run.json
│       ├── preflight.json
│       ├── plan.json
│       ├── confirmations/
│       │   └── high-risk-confirmation.json
│       ├── commands/
│       │   ├── <command-id>.json
│       │   ├── <command-id>.stdout.txt
│       │   └── <command-id>.stderr.txt
│       ├── checkpoints/
│       │   └── <checkpoint-id>.json
│       ├── patches/
│       │   ├── proposed.patch
│       │   ├── applied.patch
│       │   ├── worktree.diff
│       │   └── changed-files.json
│       ├── review/
│       │   ├── diff-summary.json
│       │   └── review-state.json
│       └── result.json
│
├── leases/
│   └── <issue-id>.json
│
└── queue/
    ├── pending.json
    ├── active.json
    └── blocked.json
```

同时 output 里生成：

```text
.agentflow/output/evidence/
└── <run-id>.json
```

---

# 5. Execute Manifest

路径：

```text
.agentflow/execute/manifest.json
```

职责：

```text
记录 execute 当前状态
记录 canonical paths
记录 run / lease / queue 数量
```

示例：

```json
{
  "version": "execute-manifest.v1",
  "projectRoot": "/path/to/project",
  "status": "ready",
  "paths": {
    "runs": ".agentflow/execute/runs",
    "leases": ".agentflow/execute/leases",
    "queue": ".agentflow/execute/queue",
    "evidence": ".agentflow/output/evidence"
  },
  "summary": {
    "runs": 0,
    "activeRuns": 0,
    "blockedRuns": 0,
    "completedRuns": 0,
    "activeLeases": 0
  }
}
```

---

# 6. Execute Index

路径：

```text
.agentflow/execute/index.json
```

职责：

```text
快速索引 run / issue / lease / result
```

示例：

```json
{
  "version": "execute-index.v1",
  "updatedAt": 1780360000,
  "runs": [],
  "leases": []
}
```

---

# 7. Run 模型

路径：

```text
.agentflow/execute/runs/<run-id>/run.json
```

一个 run 是：

> **某个 issue 的一次执行尝试。**

一个 issue 可以有多个 run：

```text
run-001
= 第一次执行

run-002
= 修复后再次执行
```

示例：

```json
{
  "version": "execute-run.v1",
  "runId": "run-001",
  "issueId": "iss-001",
  "sourceSpecId": "spec-001",
  "projectId": "proj-001",
  "riskLevel": "medium",

  "status": "preflight",
  "agentRole": "Build Agent",
  "createdBy": "agent",
  "createdAt": 1780360000,
  "updatedAt": 1780360000,

  "input": {
    "issuePath": ".agentflow/input/issues/iss-001.json",
    "specPath": ".agentflow/input/specs/approved/spec-001",
    "panelSnapshotId": "panel-snapshot-001",
    "contextPackId": "ctx-001"
  },

  "paths": {
    "preflight": ".agentflow/execute/runs/run-001/preflight.json",
    "plan": ".agentflow/execute/runs/run-001/plan.json",
    "result": ".agentflow/execute/runs/run-001/result.json",
    "evidence": ".agentflow/output/evidence/run-001.json"
  }
}
```

---

# 8. Run 状态机

V1 状态：

```text
queued
preflight
blocked
planned
checkpointed
patching
running
validating
completed
failed
cancelled
```

解释：

```text
queued
= 排队中

preflight
= 开工前检查

blocked
= 被阻断

planned
= 执行计划已生成

checkpointed
= 已生成 checkpoint

patching
= 正在应用 patch

running
= 正在执行命令或步骤

validating
= 正在验证

completed
= 完成

failed
= 失败

cancelled
= 取消
```

---

# 9. Preflight

路径：

```text
.agentflow/execute/runs/<run-id>/preflight.json
```

Preflight 是：

> **开工前体检。**

必须检查：

```text
ownership ready
define ready
panel ready / degraded
input ready
issue exists
issue has sourceSpecId
sourceSpec exists
approval.json exists
issue has riskLevel
riskLevel is low / medium / high
high risk confirmation exists if riskLevel = high
lease available
working tree readable
validation hints available or panel tests available
```

示例：

```json
{
  "version": "execute-preflight.v1",
  "runId": "run-001",
  "issueId": "iss-001",
  "status": "ready",
  "checks": [
    {
      "name": "ownership",
      "status": "passed"
    },
    {
      "name": "define",
      "status": "passed"
    },
    {
      "name": "panel",
      "status": "passed"
    },
    {
      "name": "approved-spec",
      "status": "passed"
    },
    {
      "name": "risk",
      "status": "passed",
      "riskLevel": "medium",
      "humanConfirmationRequired": false
    }
  ],
  "blockedReason": null
}
```

高风险未确认：

```json
{
  "status": "blocked",
  "checks": [
    {
      "name": "risk",
      "status": "blocked",
      "riskLevel": "high",
      "humanConfirmationRequired": true,
      "confirmed": false
    }
  ],
  "blockedReason": "High risk issue requires human confirmation before execute."
}
```

---

# 10. 高风险确认

路径：

```text
.agentflow/execute/runs/<run-id>/confirmations/high-risk-confirmation.json
```

仅当：

```text
issue.riskLevel = high
```

时需要。

示例：

```json
{
  "version": "execute-human-confirmation.v1",
  "runId": "run-001",
  "issueId": "iss-001",
  "riskLevel": "high",
  "confirmedBy": "human",
  "confirmedAt": 1780360000,
  "confirmationText": "I approve executing this high risk issue.",
  "scope": "execute-run"
}
```

注意：

```text
确认记录不写回 input issue。
```

---

# 11. Lease

路径：

```text
.agentflow/execute/leases/<issue-id>.json
```

Lease 是：

> **占坑锁，防止多个 Agent 同时执行同一个 issue。**

示例：

```json
{
  "version": "execute-lease.v1",
  "issueId": "iss-001",
  "runId": "run-001",
  "status": "active",
  "createdAt": 1780360000,
  "expiresAt": null,
  "lockedFiles": [
    "crates/input/src/model.rs"
  ]
}
```

规则：

```text
同一个 issue 同一时间只能有一个 active lease。
run completed / failed / cancelled 后释放 lease。
```

---

# 12. Run Plan

路径：

```text
.agentflow/execute/runs/<run-id>/plan.json
```

Run Plan 是：

> **这次执行准备怎么做。**

它不是 SPEC，也不是 issue。

示例：

```json
{
  "version": "execute-plan.v1",
  "runId": "run-001",
  "issueId": "iss-001",
  "steps": [
    {
      "stepId": "step-001",
      "kind": "edit",
      "target": "crates/input/src/model.rs",
      "summary": "Add InputIssue riskLevel field"
    },
    {
      "stepId": "step-002",
      "kind": "validate",
      "command": "cargo test -p agentflow-input"
    }
  ],
  "allowedWritePaths": [
    "crates/input/src/model.rs",
    "docs/requirements/009-input-model-v1.md"
  ],
  "allowedCommands": [
    "cargo test -p agentflow-input",
    "cargo fmt --check",
    "git diff --check"
  ]
}
```

---

# 13. Checkpoint

路径：

```text
.agentflow/execute/runs/<run-id>/checkpoints/<checkpoint-id>.json
```

Checkpoint 是：

> **动手前的现场快照。**

必须在以下动作之前生成：

```text
应用 patch
写文件
删文件
覆盖文件
运行可能改变状态的命令
```

示例：

```json
{
  "version": "execute-checkpoint.v1",
  "checkpointId": "chk-001",
  "runId": "run-001",
  "createdAt": 1780360000,
  "gitHead": "abc123",
  "dirtyFilesBefore": [],
  "panelSnapshotId": "panel-snapshot-001",
  "fileHashesBefore": [
    {
      "path": "crates/input/src/model.rs",
      "hash": "<sha256>"
    }
  ]
}
```

---

# 14. Patch

路径：

```text
.agentflow/execute/runs/<run-id>/patches/
```

Patch 负责记录：

```text
计划改什么
实际应用了什么
工作区最终 diff 是什么
改了哪些文件
```

文件：

```text
proposed.patch
applied.patch
worktree.diff
changed-files.json
```

---

## 14.1 proposed.patch

Agent 计划应用的 patch。

```text
.agentflow/execute/runs/<run-id>/patches/proposed.patch
```

---

## 14.2 applied.patch

实际成功应用的 patch。

```text
.agentflow/execute/runs/<run-id>/patches/applied.patch
```

---

## 14.3 worktree.diff

应用后当前工作区 diff。

```text
.agentflow/execute/runs/<run-id>/patches/worktree.diff
```

---

## 14.4 changed-files.json

示例：

```json
{
  "version": "execute-changed-files.v1",
  "runId": "run-001",
  "files": [
    {
      "path": "crates/input/src/model.rs",
      "changeType": "modified",
      "insertions": 24,
      "deletions": 2
    }
  ]
}
```

---

# 15. Command Records

路径：

```text
.agentflow/execute/runs/<run-id>/commands/<command-id>.json
```

Command Record 是：

> **命令执行记录，不是普通日志。**

示例：

```json
{
  "version": "execute-command.v1",
  "commandId": "cmd-001",
  "runId": "run-001",
  "label": "cargo test -p agentflow-input",
  "program": "cargo",
  "args": [
    "test",
    "-p",
    "agentflow-input"
  ],
  "cwd": "<project-root>",
  "source": "issue.validationHints",
  "startedAt": 1780360000,
  "finishedAt": 1780360030,
  "exitCode": 0,
  "stdoutPath": ".agentflow/execute/runs/run-001/commands/cmd-001.stdout.txt",
  "stderrPath": ".agentflow/execute/runs/run-001/commands/cmd-001.stderr.txt"
}
```

---

## 15.1 命令边界

V1 允许记录和执行结构化命令，但必须满足：

```text
来自 issue.validationHints
或来自 panel/tests.json
或来自 define/tdd/TDD.md 推荐
或来自 run plan allowedCommands
```

禁止任意 shell：

```text
sh -c "<freeform>"
bash -c "<freeform>"
```

禁止危险命令：

```text
rm -rf
git reset --hard
git clean -fd
git push
git commit
git checkout
deploy
release
curl 写远程
生产服务命令
```

---

# 16. Validation

Validation 是：

> **检查这个 issue 是否完成。**

来源：

```text
issue.validationHints
panel/tests.json
define/tdd/TDD.md
run plan allowedCommands
```

结果写入：

```text
.agentflow/execute/runs/<run-id>/result.json
.agentflow/output/evidence/<run-id>.json
```

---

# 17. Review Summary

路径：

```text
.agentflow/execute/runs/<run-id>/review/
```

文件：

```text
diff-summary.json
review-state.json
```

`diff-summary.json` 示例：

```json
{
  "version": "execute-diff-summary.v1",
  "runId": "run-001",
  "changedFiles": 2,
  "insertions": 44,
  "deletions": 8,
  "riskLevel": "medium",
  "notes": [
    "Patch only touched allowed write paths."
  ]
}
```

`review-state.json` 示例：

```json
{
  "version": "execute-review-state.v1",
  "runId": "run-001",
  "status": "pending-review",
  "hunkReviewEnabled": false,
  "notes": [
    "V1 stores reviewable diff artifacts but does not implement hunk accept/reject UI."
  ]
}
```

---

# 18. Result

路径：

```text
.agentflow/execute/runs/<run-id>/result.json
```

Result 是：

> **本次 run 的最终结论。**

示例：

```json
{
  "version": "execute-result.v1",
  "runId": "run-001",
  "issueId": "iss-001",
  "status": "completed",
  "riskLevel": "medium",
  "changedFiles": [
    "crates/input/src/model.rs"
  ],
  "commands": [
    "cmd-001"
  ],
  "validation": {
    "passed": true,
    "evidencePath": ".agentflow/output/evidence/run-001.json"
  },
  "next": {
    "readyForDelivery": true,
    "needsAudit": true
  }
}
```

---

# 19. Evidence

路径：

```text
.agentflow/output/evidence/<run-id>.json
```

Evidence 是：

> **给后续 Release / Audit看的证据包。**

示例：

```json
{
  "version": "output-evidence.v1",
  "runId": "run-001",
  "issueId": "iss-001",
  "sourceSpecId": "spec-001",
  "riskLevel": "medium",
  "completedAt": 1780360100,
  "summary": "Implemented InputIssue riskLevel and validation.",
  "changedFiles": [],
  "commands": [],
  "validationPassed": true,
  "artifacts": {
    "run": ".agentflow/execute/runs/run-001/run.json",
    "preflight": ".agentflow/execute/runs/run-001/preflight.json",
    "result": ".agentflow/execute/runs/run-001/result.json",
    "diff": ".agentflow/execute/runs/run-001/patches/worktree.diff"
  }
}
```

---

# 20. Queue

路径：

```text
.agentflow/execute/queue/
```

文件：

```text
pending.json
active.json
blocked.json
```

V1 只需要支持本地派生视图，不需要真正并发调度。

---

# 21. Tauri / Rust 模块建议

建议新增 crate：

```text
crates/execute/
```

package：

```text
agentflow-execute
```

结构：

```text
crates/execute/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── model.rs
    ├── manager.rs
    ├── storage.rs
    ├── preflight.rs
    ├── lease.rs
    ├── plan.rs
    ├── checkpoint.rs
    ├── patch.rs
    ├── command.rs
    ├── validation.rs
    ├── result.rs
    └── evidence.rs
```

依赖：

```text
agentflow-agent-manual
agentflow-panel
agentflow-input
```

---

# 22. Tauri commands

建议新增：

```text
prepare_execute_workspace
load_execute_status
load_execute_manifest
load_execute_index
create_execute_run
load_execute_run
execute_run_preflight
confirm_high_risk_execute_run
acquire_execute_lease
release_execute_lease
write_execute_plan
create_execute_checkpoint
apply_execute_patch
run_execute_command
load_execute_result
validate_execute_run
complete_execute_run
cancel_execute_run
```

注意：

```text
Desktop human UI V1 可以只读展示。
真正写入 / patch / command API 应标注 agent-only。
```

---

# 23. Desktop UI

V1 只读展示：

```text
Execute status
Runs
Active lease
Blocked runs
Run detail
Preflight result
Changed files
Validation result
Evidence path
```

人类 UI 可以做：

```text
高风险确认按钮
取消 run
查看 diff
查看 evidence
```

但不能手动编辑 run 文件。

---

# 24. 权限边界

## 24.1 允许写

```text
.agentflow/execute/**
.agentflow/output/evidence/**
```

## 24.2 允许受控写用户源码

本需求是 Patch / Checkpoint V1，所以允许受控写用户源码，但必须满足：

```text
来自 input issue
preflight passed
lease acquired
run plan exists
checkpoint exists
patch 只修改 allowedWritePaths
high risk 已确认
changed-files 记录完整
result / evidence 记录完整
```

## 24.3 不允许

```text
不写 .agentflow/input/**
不修改 Approved SPEC
不修改 input issue
不修改 panel facts
不创建 PR
不 merge
不 release
不 deploy
不调用模型
不执行危险命令
不执行远程操作
```

---

# 25. Non-goals

本需求不做：

```text
不实现 Release Agent
不创建 PR
不自动 merge
不发布
不部署
不做 Audit report
不做 hunk accept / reject UI
不做 Git worktree isolation
不做多人并发 Agent
不连接远程服务
不调用模型
不修改 input facts
```

---

# 26. 测试要求

必须覆盖：

```text
1. 没有 input issue 时不能 create run。
2. issue 没有 sourceSpecId 时 preflight blocked。
3. source Approved SPEC 不存在时 preflight blocked。
4. low risk issue 不需要确认。
5. medium risk issue 不需要确认。
6. high risk issue 没确认时 preflight blocked。
7. high risk issue 确认后 preflight pass。
8. 同一个 issue 不能同时获取两个 active lease。
9. completed / failed / cancelled 后释放 lease。
10. checkpoint 在 patch 前创建。
11. patch 只能修改 allowedWritePaths。
12. patch 修改未授权路径时 blocked。
13. command record 写 stdout / stderr / exitCode。
14. dangerous command 被 blocked。
15. result 必须指向 evidence。
16. completed run 必须有 result 和 evidence。
17. execute 不写 input issue。
18. execute 不写 Approved SPEC。
```

---

# 27. 验收标准

```text
- [ ] 新增 docs/requirements/010-execute-patch-checkpoint-v1.md。
- [ ] 新增 crates/execute package agentflow-execute。
- [ ] 创建 .agentflow/execute/manifest.json。
- [ ] 创建 .agentflow/execute/index.json。
- [ ] 创建 .agentflow/execute/runs/。
- [ ] 创建 .agentflow/execute/leases/。
- [ ] 创建 .agentflow/execute/queue/。
- [ ] create_execute_run 只能从 input issue 启动。
- [ ] preflight 检查 ownership / define / panel / input / approved spec / risk / lease。
- [ ] high risk issue 未确认时 blocked。
- [ ] high risk issue 确认后可继续。
- [ ] low / medium issue 不需要确认。
- [ ] lease 防止同 issue 并发执行。
- [ ] run plan 支持 allowedWritePaths / allowedCommands。
- [ ] checkpoint 在 patch / write 前生成。
- [ ] proposed.patch / applied.patch / worktree.diff 生成。
- [ ] changed-files.json 生成。
- [ ] command record 结构化记录 program / args / cwd / exitCode / stdout / stderr。
- [ ] dangerous command 被 blocked。
- [ ] validation result 写入 result.json。
- [ ] evidence 写入 .agentflow/output/evidence/<run-id>.json。
- [ ] completed run 必须有 evidence。
- [ ] review/diff-summary.json 生成。
- [ ] review/review-state.json 生成。
- [ ] 不写 input issue。
- [ ] 不写 Approved SPEC。
- [ ] 不创建 PR。
- [ ] 不 merge。
- [ ] 不 release。
- [ ] 不 deploy。
- [ ] 不调用模型。
- [ ] Desktop 只读展示 execute 状态。
- [ ] cargo fmt --check 通过。
- [ ] cargo test -p agentflow-execute 通过。
- [ ] cargo test -p agentflow-desktop 通过。
- [ ] cargo test 通过。
- [ ] npm --prefix apps/desktop run build 通过。
- [ ] git diff --check 通过。
```

---

# 28. 验证命令

```bash
cargo fmt --check
cargo test -p agentflow-execute
cargo test -p agentflow-desktop
cargo test
npm --prefix apps/desktop run build
git diff --check
```

---

# 29. PR 说明要求

PR 描述必须说明：

```text
1. execute 为什么只能从 input issue 启动。
2. preflight 检查了哪些 gate。
3. high risk issue 如何要求人类确认。
4. low / medium 为什么不需要确认。
5. lease 如何防止同 issue 并发执行。
6. checkpoint 在什么动作前创建。
7. patch 如何限制 allowedWritePaths。
8. command record 如何结构化记录。
9. dangerous command 如何 blocked。
10. result 和 evidence 的关系。
11. 本次没有修改 input facts。
12. 本次没有创建 PR / merge / release。
13. 本次没有调用模型。
14. 验证命令和结果。
```

---

# 30. Codex 执行指令

```md
请执行 010 - Execute Patch / Checkpoint V1。

目标：
一次性实现 AgentFlow 的受控执行流水线。execute 只能从 `.agentflow/input/issues/<issue-id>.json` 启动，创建 run，完成 preflight、lease、risk check、run plan、checkpoint、patch、command record、validation、result 和 evidence。低 / 中风险 issue 不需要人类确认，高风险 issue 执行前必须人类确认。

必须遵守：
1. execute 只能从 input issue 启动。
2. 没有 Approved SPEC 的 issue 不能执行。
3. riskLevel 只读取 low / medium / high。
4. low / medium 不需要人类确认。
5. high 需要人类确认。
6. 同一个 issue 只能有一个 active lease。
7. patch / write 前必须 checkpoint。
8. patch 只能改 allowedWritePaths。
9. command 必须结构化记录。
10. dangerous command 必须 blocked。
11. completed run 必须有 result 和 evidence。
12. 不写 input issue。
13. 不写 Approved SPEC。
14. 不创建 PR。
15. 不 merge。
16. 不 release。
17. 不 deploy。
18. 不调用模型。
19. Desktop human UI 只读展示 execute 状态；写入 / patch / command API 标注 agent-only。

实现范围：
- 新增 docs/requirements/010-execute-patch-checkpoint-v1.md。
- 新增 crates/execute package agentflow-execute。
- 新增 execute manifest / index。
- 新增 run / preflight / lease / plan / checkpoint / patch / command / result / evidence 模型。
- 新增 execute workspace prepare / validate。
- 新增 create_execute_run。
- 新增 execute_run_preflight。
- 新增 confirm_high_risk_execute_run。
- 新增 acquire / release lease。
- 新增 create checkpoint。
- 新增 apply patch。
- 新增 run command。
- 新增 validate / complete run。
- 新增 output/evidence 写入。
- 新增 Desktop status 只读展示。
- 更新 Browser Preview mock。
- 更新 verification。

验证命令：
- cargo fmt --check
- cargo test -p agentflow-execute
- cargo test -p agentflow-desktop
- cargo test
- npm --prefix apps/desktop run build
- git diff --check
```

---

# 31. 完成定义

本需求完成后：

```text
execute/
= Issue 的受控施工流水线
```

完整流程：

```text
input issue
→ run
→ preflight
→ lease
→ plan
→ checkpoint
→ patch / command
→ validation
→ result
→ evidence
```

最终一句话：

> **Execute Patch / Checkpoint V1 让 AgentFlow 具备受控执行能力：不是直接干活，而是每次执行都带 issue 来源、开工检查、锁、计划、checkpoint、patch、命令记录、验证、结果和证据。**
