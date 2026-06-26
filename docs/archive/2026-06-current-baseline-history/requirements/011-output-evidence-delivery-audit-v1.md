# 011 - Output Evidence / Delivery / Audit V1

创建日期：2026-06-04  
执行者：Codex  
状态：待开发  
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
= 需求实时源头，包含 Spec Gate / Projects / Issues

execute/
= 受控执行流水线，包含 run / preflight / lease / plan / checkpoint / patch / command / validation / result

output/
= 当前已有 evidence / release / audit / logs / backup / cache / tmp

state/
= 健康 / 锁 / 会话 / 索引状态
```

PR #24 已经完成：

```text
Build Agent
= input issue -> execute run -> output/evidence -> output/release

Release Agent
= 不再作为独立角色存在

Release capability
= 并入 Build Agent
```

现在需要把：

```text
output/
```

正式收口成 AgentFlow 的：

```text
交付与证据层
```

大白话：

> `execute/` 记录施工过程。  
> `output/` 记录施工之后能给人看、给 Agent 审、给后续交付使用的材料。  
> 所以 output 不是杂物目录，也不是普通日志目录。  
> 它应该清楚分成 evidence、release、audit 三类。

---

## 一句话定义

> **Output Evidence / Delivery / Audit V1 负责把 `.agentflow/output/` 正式定义为 AgentFlow 的交付与证据层。它包含 Build Agent 产出的执行证据 `evidence/`，Build Agent 产出的交付材料 `release/`，以及 Audit Agent 未来产出的审计结果 `audit/`。Output 不定义需求、不描述项目现场、不记录执行过程原始状态；它只整理执行完成后的证据、交付材料和审计结果。**

---

# 1. 核心原则

## 1.1 output 是交付与证据层

```text
output/
= 执行完成后的证据
= 交付材料
= 审计结果
= 日志 / 备份 / 缓存 / 临时文件
```

它回答：

```text
这个 issue 是否完成？
完成依据是什么？
验证结果在哪里？
改动摘要在哪里？
PR 草案在哪里？
release note 在哪里？
审计报告在哪里？
后续人类 / Agent 如何复查？
```

---

## 1.2 output 不做什么

`output/` 不负责：

```text
不定义需求
不写 SPEC
不生成 issue
不描述项目现场
不执行 patch
不运行命令
不修改用户源码
不修改 input issue
不修改 Approved SPEC
不修改 execute run
不创建远程 PR
不 merge
不 deploy
不调用模型
```

这些分别属于：

```text
input/
panel/
execute/
```

---

## 1.3 output 必须可追溯

所有 output 产物都必须能追溯到：

```text
sourceSpecId
issueId
runId
panelSnapshotId / contextPackId
execute result
checkpoint
changed files
validation result
```

也就是说：

```text
output 不能孤立存在。
```

---

## 1.4 output 不复制大日志

命令原始输出保留在：

```text
.agentflow/execute/runs/<run-id>/commands/
```

output 只保留：

```text
摘要
路径引用
验证结果
证据链接
```

不要把大段 stdout / stderr 复制到 output 中。

---

# 2. 参考原则

## 2.1 从 Warp 吸收：交付必须有证明

Warp 的协作模式强调：

```text
issue 先 triage
feature 先 spec
spec approved 后才 implement
PR 需要测试证明
review 需要依据
```

AgentFlow 的对应设计是：

```text
每个 completed execute run 必须有 evidence。
每个 delivery 都必须引用 evidence。
```

大白话：

> 不能只说“完成了”，必须说“为什么可以认为完成了”。

---

## 2.2 从 Zed 吸收：输出必须能复查改动

Zed Agent Panel 的关键思路包括：

```text
checkpoint
review changes
changed files
diff summary
可恢复 / 可复查
```

AgentFlow 的对应设计是：

```text
evidence 必须引用 checkpoint
release delivery 必须引用 changed-files / diff-summary
audit 必须读取 patch / evidence / delivery
```

大白话：

> output 不是只看最后结果，还要能回看“改之前是什么样，改了什么，怎么验证”。

---

# 3. output 目标结构

目标结构：

```text
.agentflow/output/
├── manifest.json
├── index.json
│
├── evidence/
│   └── <run-id>.json
│
├── release/
│   └── <run-id>/
│       ├── delivery.json
│       ├── pr-draft.md
│       ├── pr-metadata.json
│       ├── review-checklist.md
│       ├── changelog.md
│       └── release-note.md
│
├── audit/
│   └── <run-id>/
│       ├── audit-report.md
│       ├── audit.json
│       ├── findings.json
│       └── checklist.md
│
├── logs/
├── backup/
├── cache/
└── tmp/
```

---

# 4. output 三个核心分区

## 4.1 evidence/

```text
.agentflow/output/evidence/
```

归属：

```text
Build Agent
```

职责：

```text
执行完成证明
验证结果摘要
命令结果摘要
改动路径引用
checkpoint 引用
diff 引用
```

一句话：

> **evidence 是 Build Agent 证明自己完成 issue 的证据包。**

---

## 4.2 release/

```text
.agentflow/output/release/
```

归属：

```text
Build Agent
```

职责：

```text
PR 草案
PR metadata
review checklist
changelog
release note
delivery record
```

一句话：

> **release 是 Build Agent 准备给人类 / 后续系统交付的材料包。**

注意：

```text
output/release 不等于已经创建远程 PR。
output/release 不等于已经 merge。
output/release 不等于已经 deploy。
```

它只是本地交付材料。

---

## 4.3 audit/

```text
.agentflow/output/audit/
```

归属：

```text
Audit Agent
```

当前状态：

```text
not authorized yet
```

职责：

```text
审计报告
审计 JSON
问题发现
审计 checklist
```

一句话：

> **audit 是 Audit Agent 未来审查 Build Agent 交付结果的地方。**

---

# 5. Output Manifest

路径：

```text
.agentflow/output/manifest.json
```

职责：

```text
记录 output 当前状态
记录 canonical paths
记录 evidence / release / audit 数量
记录最近 output 更新时间
```

示例：

```json
{
  "version": "output-manifest.v1",
  "projectRoot": "/path/to/project",
  "status": "ready",
  "paths": {
    "evidence": ".agentflow/output/evidence",
    "release": ".agentflow/output/release",
    "audit": ".agentflow/output/audit",
    "logs": ".agentflow/output/logs",
    "backup": ".agentflow/output/backup",
    "cache": ".agentflow/output/cache",
    "tmp": ".agentflow/output/tmp"
  },
  "summary": {
    "evidence": 0,
    "releaseDeliveries": 0,
    "audits": 0,
    "logs": 0,
    "backups": 0
  }
}
```

---

# 6. Output Index

路径：

```text
.agentflow/output/index.json
```

职责：

```text
快速索引 evidence / release / audit 产物
```

示例：

```json
{
  "version": "output-index.v1",
  "updatedAt": 1780360000,
  "evidence": [],
  "releaseDeliveries": [],
  "audits": []
}
```

Index entry 建议：

```json
{
  "runId": "run-001",
  "issueId": "iss-001",
  "sourceSpecId": "spec-001",
  "path": ".agentflow/output/evidence/run-001.json",
  "status": "ready",
  "updatedAt": 1780360000
}
```

---

# 7. Evidence 模型

## 7.1 路径

```text
.agentflow/output/evidence/<run-id>.json
```

## 7.2 职责

Evidence 回答：

```text
这个 run 做了什么？
是否完成？
验证是否通过？
改了哪些文件？
命令结果在哪里？
checkpoint 在哪里？
diff 在哪里？
是否有人工验证说明？
```

## 7.3 建议结构

```json
{
  "version": "output-evidence.v1",
  "runId": "run-001",
  "issueId": "iss-001",
  "sourceSpecId": "spec-001",
  "riskLevel": "medium",
  "completedAt": 1780360100,

  "summary": "Implemented issue and validated successfully.",

  "input": {
    "issuePath": ".agentflow/input/issues/iss-001.json",
    "specPath": ".agentflow/input/specs/approved/spec-001"
  },

  "panel": {
    "snapshotId": "panel-snapshot-001",
    "contextPackId": "ctx-001"
  },

  "execute": {
    "run": ".agentflow/execute/runs/run-001/run.json",
    "preflight": ".agentflow/execute/runs/run-001/preflight.json",
    "plan": ".agentflow/execute/runs/run-001/plan.json",
    "result": ".agentflow/execute/runs/run-001/result.json",
    "checkpoint": ".agentflow/execute/runs/run-001/checkpoints/chk-001.json",
    "diff": ".agentflow/execute/runs/run-001/patches/worktree.diff",
    "changedFiles": ".agentflow/execute/runs/run-001/patches/changed-files.json",
    "diffSummary": ".agentflow/execute/runs/run-001/review/diff-summary.json"
  },

  "commands": [
    {
      "commandId": "cmd-001",
      "label": "cargo test -p agentflow-execute",
      "exitCode": 0,
      "recordPath": ".agentflow/execute/runs/run-001/commands/cmd-001.json",
      "stdoutPath": ".agentflow/execute/runs/run-001/commands/cmd-001.stdout.txt",
      "stderrPath": ".agentflow/execute/runs/run-001/commands/cmd-001.stderr.txt"
    }
  ],

  "validation": {
    "passed": true,
    "failedCommands": [],
    "skipped": []
  },

  "manualProof": {
    "required": false,
    "notes": [],
    "screenshots": [],
    "recordings": []
  }
}
```

---

# 8. Evidence 完成规则

一个 execute run 只有满足以下条件，才可以算 completed：

```text
result.json exists
output/evidence/<run-id>.json exists
evidence references run.json
evidence references preflight.json
evidence references result.json
evidence references changed-files.json if patch exists
evidence references worktree.diff if patch exists
evidence references command records if commands exist
```

如果缺 evidence：

```text
run 不应算 completed
```

---

# 9. Release Delivery 模型

## 9.1 路径

```text
.agentflow/output/release/<run-id>/
```

## 9.2 文件

```text
delivery.json
pr-draft.md
pr-metadata.json
review-checklist.md
changelog.md
release-note.md
```

---

## 9.3 delivery.json

职责：

```text
交付材料总入口
```

示例：

```json
{
  "version": "output-release-delivery.v1",
  "runId": "run-001",
  "issueId": "iss-001",
  "sourceSpecId": "spec-001",
  "riskLevel": "medium",
  "status": "drafted",
  "createdBy": "Build Agent",
  "createdAt": 1780360100,
  "evidencePath": ".agentflow/output/evidence/run-001.json",
  "executeResultPath": ".agentflow/execute/runs/run-001/result.json",
  "diffSummaryPath": ".agentflow/execute/runs/run-001/review/diff-summary.json",
  "artifacts": {
    "prDraft": ".agentflow/output/release/run-001/pr-draft.md",
    "prMetadata": ".agentflow/output/release/run-001/pr-metadata.json",
    "reviewChecklist": ".agentflow/output/release/run-001/review-checklist.md",
    "changelog": ".agentflow/output/release/run-001/changelog.md",
    "releaseNote": ".agentflow/output/release/run-001/release-note.md"
  }
}
```

---

## 9.4 pr-draft.md

必须包含：

```text
Summary
Linked SPEC
Linked Issue
Run ID
Changed files
Validation evidence
Risk level
Manual proof if required
Non-goals
Reviewer checklist
```

示例：

```md
# PR Draft

## Summary

<本次变更做了什么>

## Linked Records

- SPEC: `.agentflow/input/specs/approved/<spec-id>/`
- Issue: `.agentflow/input/issues/<issue-id>.json`
- Run: `.agentflow/execute/runs/<run-id>/`
- Evidence: `.agentflow/output/evidence/<run-id>.json`

## Changed Files

<changed files summary>

## Validation

<validation summary>

## Risk

riskLevel: medium

## Notes

No merge, deploy, or production release was performed.
```

---

## 9.5 pr-metadata.json

建议结构：

```json
{
  "version": "output-pr-metadata.v1",
  "runId": "run-001",
  "issueId": "iss-001",
  "sourceSpecId": "spec-001",
  "title": "Implement <issue title>",
  "branchName": null,
  "remotePrUrl": null,
  "status": "draft-only",
  "createdRemotePr": false
}
```

注意：

```text
V1 不创建远程 PR。
```

---

## 9.6 review-checklist.md

必须检查：

```text
是否符合 product.md
是否符合 tech.md
是否符合 issue acceptanceCriteria
是否只修改 allowedWritePaths
是否有 checkpoint
是否有 changed-files
是否有 evidence
是否有 validation result
高风险 issue 是否有 confirmation
是否没有 merge / deploy
```

---

## 9.7 changelog.md

职责：

```text
记录可用于 changelog 的条目草案
```

V1 只生成草案，不写项目正式 changelog。

---

## 9.8 release-note.md

职责：

```text
记录 release note 草案
```

V1 只生成草案，不发布。

---

# 10. Audit Output 模型

## 10.1 当前状态

Audit Agent 当前：

```text
not authorized yet
```

但 output V1 需要创建 audit 结构和 schema skeleton。

---

## 10.2 路径

```text
.agentflow/output/audit/<run-id>/
```

## 10.3 文件

```text
audit-report.md
audit.json
findings.json
checklist.md
```

---

## 10.4 audit.json

建议结构：

```json
{
  "version": "output-audit.v1",
  "runId": "run-001",
  "issueId": "iss-001",
  "sourceSpecId": "spec-001",
  "status": "pending",
  "checks": {
    "specAligned": null,
    "issueAcceptanceCovered": null,
    "allowedPathsOnly": null,
    "evidenceComplete": null,
    "releaseDeliveryComplete": null
  },
  "findings": []
}
```

V1 可以只创建 skeleton，不运行真实 audit。

---

## 10.5 audit-report.md

模板：

```md
# Audit Report

Status: pending

This audit report is a skeleton for future Audit Agent execution.

## Inputs

- Approved SPEC:
- Issue:
- Execute Run:
- Evidence:
- Release Delivery:

## Checks

- SPEC alignment:
- Issue acceptance coverage:
- Allowed paths:
- Evidence completeness:
- Release delivery completeness:

## Findings

None yet.
```

---

# 11. output 和 Agent 角色的关系

## 11.1 Build Agent

Build Agent 写：

```text
output/evidence/
output/release/
```

Build Agent 不写：

```text
output/audit/
```

---

## 11.2 Audit Agent

Audit Agent 未来写：

```text
output/audit/
```

Audit Agent 读取：

```text
input/
execute/
output/evidence/
output/release/
panel/
```

Audit Agent 不写：

```text
input/
execute/
output/evidence/
output/release/
用户源码
```

---

# 12. Output 状态模型

新增：

```text
OutputStatusSnapshot
```

建议字段：

```ts
type OutputStatusSnapshot = {
  version: "output-status.v1";
  projectRoot: string;
  status: "missing" | "ready" | "degraded" | "failed" | "blocked";
  ready: boolean;
  manifestExists: boolean;
  indexExists: boolean;
  summary: {
    evidence: number;
    releaseDeliveries: number;
    audits: number;
    incompleteEvidence: number;
    incompleteDeliveries: number;
  };
  missingPaths: string[];
  warnings: string[];
  errors: string[];
};
```

---

# 13. Output validation

必须检查：

```text
output/manifest.json exists
output/index.json exists
output/evidence/ exists
output/release/ exists
output/audit/ exists
output/logs/ exists
output/backup/ exists
output/cache/ exists
output/tmp/ exists
```

对 evidence 检查：

```text
每个 evidence 必须有 runId / issueId / sourceSpecId
evidence 引用的 execute result 必须存在
evidence 引用的 run 必须存在
如果 evidence.validation.passed = true，result.validation.passed 必须一致
```

对 release delivery 检查：

```text
delivery.json 必须存在
pr-draft.md 必须存在
pr-metadata.json 必须存在
review-checklist.md 必须存在
changelog.md 必须存在
release-note.md 必须存在
delivery 必须引用 evidence
delivery.createdBy = Build Agent
```

对 audit skeleton 检查：

```text
audit 目录可为空
如果存在 audit/<run-id>/，必须包含 audit.json 或 audit-report.md
```

---

# 14. Rust 模块建议

建议新增 crate：

```text
crates/output/
```

package：

```text
agentflow-output
```

结构：

```text
crates/output/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── model.rs
    ├── manager.rs
    ├── storage.rs
    ├── evidence.rs
    ├── release.rs
    ├── audit.rs
    ├── validate.rs
    └── index.rs
```

如果暂时不新增 crate，也可以先把 output status / validation 放进：

```text
crates/execute/src/output.rs
```

但 V1 推荐单独 crate：

```text
agentflow-output
```

原因：

```text
output 是 execute 之后的独立交付层
后续 Audit Agent 会依赖 output
```

---

# 15. Tauri commands

建议新增：

```text
prepare_output_workspace
load_output_status
load_output_manifest
load_output_index
load_output_snapshot
validate_output
load_output_evidence
load_release_delivery
load_audit_output
```

注意：

```text
Desktop human UI V1 只读展示 output 状态。
```

写入类：

```text
write_evidence
write_release_delivery
write_audit_report
```

应为 agent-only 或由 Build / Audit Agent 内部调用，不暴露成人类手动编辑 UI。

---

# 16. Desktop UI

V1 只读展示：

```text
Output status
Evidence count
Release deliveries count
Audit count
Incomplete evidence
Incomplete deliveries
Latest evidence
Latest delivery
Latest audit status
```

状态通道新增：

```text
交付输出
```

指标：

```text
Evidence
Delivery
Audit
Incomplete
```

---

# 17. 写入边界

## 17.1 允许写

```text
.agentflow/output/manifest.json
.agentflow/output/index.json
.agentflow/output/evidence/**
.agentflow/output/release/**
.agentflow/output/audit/**
.agentflow/output/logs/**
.agentflow/output/cache/**
.agentflow/output/tmp/**
```

## 17.2 不允许写

```text
.agentflow/input/**
.agentflow/panel/**
.agentflow/execute/**
用户源码
远程 PR
远程 issue
生产服务
```

注意：

```text
Build Agent 在 execute 阶段写 execute/
Build Agent 在 output 阶段写 output/evidence 和 output/release
Audit Agent 在未来 audit 阶段写 output/audit
```

---

# 18. 非目标

本需求不做：

```text
不修改 input 模型
不修改 execute 主流程
不创建远程 PR
不 merge
不 deploy
不发布生产
不运行真实 Audit Agent
不实现 hunk review UI
不上传截图 / 录屏
不调用模型
不写用户源码
```

---

# 19. 开发切片

## Slice 1：Output layout

```text
output/manifest.json
output/index.json
evidence/
release/
audit/
logs/
backup/
cache/
tmp/
```

---

## Slice 2：Output models

```text
OutputManifest
OutputIndex
OutputStatusSnapshot
OutputEvidence
OutputReleaseDelivery
OutputAuditSkeleton
```

---

## Slice 3：Evidence validation

```text
验证 evidence 是否引用 run / result / command / diff
验证 validation 状态一致
```

---

## Slice 4：Release delivery validation

```text
验证 delivery.json
验证 PR draft / metadata / checklist / changelog / release note
验证 evidence 引用
```

---

## Slice 5：Audit skeleton

```text
创建 output/audit/<run-id>/ skeleton API
不运行真实 audit
```

---

## Slice 6：Desktop status

```text
Output status channel
Browser Preview mock
只读展示
```

---

# 20. 测试要求

必须新增测试：

```text
1. prepare_output_workspace creates manifest / index / required directories.
2. output status is ready after prepare.
3. evidence validation passes with valid run / result references.
4. evidence validation fails when referenced run is missing.
5. release delivery validation passes with all required files.
6. release delivery validation fails when pr-draft.md is missing.
7. delivery must reference existing evidence.
8. output/audit skeleton can be created for run.
9. audit skeleton does not execute audit.
10. output prepare does not write input / panel / execute.
```

---

# 21. 验收标准

```text
- [ ] 新增 docs/requirements/011-output-evidence-delivery-audit-v1.md。
- [ ] 新增 crates/output package agentflow-output。
- [ ] 创建 .agentflow/output/manifest.json。
- [ ] 创建 .agentflow/output/index.json。
- [ ] output/evidence/ 存在。
- [ ] output/release/ 存在。
- [ ] output/audit/ 存在。
- [ ] output/logs/ 存在。
- [ ] output/backup/ 存在。
- [ ] output/cache/ 存在。
- [ ] output/tmp/ 存在。
- [ ] OutputManifest 包含 evidence / release / audit paths。
- [ ] OutputIndex 索引 evidence / release / audit。
- [ ] OutputStatusSnapshot 暴露 evidence / releaseDeliveries / audits / incomplete counts。
- [ ] evidence schema 包含 input / panel / execute / commands / validation / manualProof。
- [ ] evidence 不复制大 stdout / stderr，只引用 command paths。
- [ ] release delivery schema 包含 delivery.json / pr-draft.md / pr-metadata.json / review-checklist.md / changelog.md / release-note.md。
- [ ] delivery.createdBy = Build Agent。
- [ ] release delivery 必须引用 evidence。
- [ ] audit skeleton schema 存在。
- [ ] Audit Agent 仍然 not authorized yet。
- [ ] output/evidence 属于 Build Agent。
- [ ] output/release 属于 Build Agent。
- [ ] output/audit 属于 Audit Agent。
- [ ] Desktop 只读展示 Output status。
- [ ] Browser Preview mock 更新。
- [ ] 不修改 input facts。
- [ ] 不修改 execute run facts。
- [ ] 不写用户源码。
- [ ] 不创建 PR。
- [ ] 不 merge。
- [ ] 不 deploy。
- [ ] 不调用模型。
- [ ] cargo fmt --check 通过。
- [ ] cargo test -p agentflow-output 通过。
- [ ] cargo test -p agentflow-execute 通过。
- [ ] cargo test -p agentflow-desktop 通过。
- [ ] cargo test 通过。
- [ ] npm --prefix apps/desktop run build 通过。
- [ ] git diff --check 通过。
```

---

# 22. 验证命令

```bash
cargo fmt --check
cargo test -p agentflow-output
cargo test -p agentflow-execute
cargo test -p agentflow-desktop
cargo test
npm --prefix apps/desktop run build
git diff --check
```

---

# 23. PR 说明要求

PR 描述必须说明：

```text
1. output/ 为什么定义为交付与证据层。
2. output/evidence 属于 Build Agent 的执行证明。
3. output/release 属于 Build Agent 的交付材料。
4. output/audit 属于 Audit Agent 的未来审计结果。
5. output 和 execute 的边界是什么。
6. evidence 如何引用 execute artifacts。
7. release delivery 如何引用 evidence。
8. audit skeleton 为什么不启用 Audit Agent。
9. 本次没有修改 input / execute facts。
10. 本次没有创建 PR / merge / deploy。
11. 本次没有调用模型。
12. 验证命令和结果。
```

---

# 24. Codex 执行指令

```md
请执行 011 - Output Evidence / Delivery / Audit V1。

目标：
将 `.agentflow/output/` 正式收口为 AgentFlow 的交付与证据层。output 包含三类核心产物：Build Agent 的 execution evidence、Build Agent 的 release delivery artifacts、Audit Agent 未来的 audit outputs。新增 output manifest / index / status / validation，让 Desktop 能只读展示 output 状态。

必须遵守：
1. output/evidence 属于 Build Agent。
2. output/release 属于 Build Agent。
3. output/audit 属于 Audit Agent。
4. output 不定义需求。
5. output 不描述项目现场。
6. output 不记录执行过程原始状态。
7. output 只引用 execute artifacts，不复制大 stdout / stderr。
8. evidence 必须引用 run / preflight / result / checkpoint / diff / changed-files / command records。
9. release delivery 必须引用 evidence。
10. audit skeleton 可以创建，但 Audit Agent 不启用。
11. 不修改 input facts。
12. 不修改 execute run facts。
13. 不写用户源码。
14. 不创建远程 PR。
15. 不 merge。
16. 不 deploy。
17. 不调用模型。

实现范围：
- 新增 docs/requirements/011-output-evidence-delivery-audit-v1.md。
- 新增 crates/output package agentflow-output。
- 新增 output manifest / index / status / snapshot。
- 新增 evidence schema / validation。
- 新增 release delivery schema / validation。
- 新增 audit skeleton schema。
- 新增 prepare_output_workspace / validate_output / load_output_status / load_output_snapshot。
- 新增 load evidence / load release delivery / load audit output。
- Project Workspace prepare 接入 output prepare。
- Desktop status channel 增加 Output status。
- Browser Preview mock 更新。
- README / GOAL / ROADMAP / requirements / verification 更新。

验证命令：
- cargo fmt --check
- cargo test -p agentflow-output
- cargo test -p agentflow-execute
- cargo test -p agentflow-desktop
- cargo test
- npm --prefix apps/desktop run build
- git diff --check
```

---

# 25. 完成定义

本需求完成后：

```text
output/
= AgentFlow 的交付与证据层
```

三块语义：

```text
output/evidence/
= Build Agent 执行证明

output/release/
= Build Agent 交付材料

output/audit/
= Audit Agent 审计结果
```

主链路：

```text
input issue
→ execute run
→ output/evidence
→ output/release
→ output/audit
```

最终一句话：

> **Output V1 把 AgentFlow 的结果层收口成 evidence、release、audit：Build Agent 交证据和交付材料，Audit Agent 后续审查这些材料。output 不再是杂项目录，而是完整的交付证据层。**
