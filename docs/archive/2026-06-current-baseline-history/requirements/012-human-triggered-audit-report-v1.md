# 012 - Human-triggered Audit Report V1

创建日期：2026-06-04  
执行者：Codex  
状态：待开发  
版本：final-draft

---

## 用户目标

当前 AgentFlow 的主流程已经收敛为：

```text
define/
→ panel/
→ input/
→ execute/
→ output/
→ state/
```

其中：

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
= 交付与证据层，包含 evidence / release / audit

state/
= 健康 / 锁 / 会话 / 索引状态
```

Audit 不应该成为每个 issue、每个 run、每个 project 的默认必经流程。

正确定位是：

```text
Audit
= 人类主动触发的完整审计报告
```

也就是说：

```text
日常闭环：
input issue
→ execute
→ output/evidence
→ output/release
→ done

可选审计：
human trigger
→ output/audit/<audit-id>/
→ 完整审计报告包
```

本需求目标是：

> **只要人类触发审计，不管审计对象是某个 run、某个 project、某批交付、某个高风险范围，都统一输出一份完整审计报告包。不再拆 run audit / project audit / batch audit 三套模型。**

---

## 一句话定义

> **Human-triggered Audit Report V1 是 AgentFlow 的人类触发审计能力。它不会在每次 execute / output 后自动生成审计报告；只有人类明确触发 audit 时，才在 `.agentflow/output/audit/<audit-id>/` 下生成一份完整、可追溯、可复查的审计报告包。**

---

# 1. 核心原则

## 1.1 Audit 不自动执行

以下动作不会自动生成 audit report：

```text
execute completed
output/evidence generated
output/release generated
Build Agent completed delivery
low / medium issue completed
high issue completed
```

也就是说：

```text
Build Agent 完成 evidence + release delivery
= 日常闭环完成

Audit
= 人类额外触发
```

---

## 1.2 Audit 只在人类触发时生成

只有明确的人类触发动作，才允许创建：

```text
.agentflow/output/audit/<audit-id>/
```

触发来源可以是：

```text
人类要求审计某次交付
人类要求审计某个 Project
人类要求审计某个大版本
人类要求审计一组高风险 issues
人类要求审计某个 run
```

但数据模型不分：

```text
run audit
project audit
batch audit
```

统一都是：

```text
Human-triggered Audit Report
```

---

## 1.3 不按 run / project / batch 分目录

不要创建这种结构：

```text
.agentflow/output/audit/runs/<run-id>/
.agentflow/output/audit/projects/<project-id>/
.agentflow/output/audit/batches/<batch-id>/
```

统一使用：

```text
.agentflow/output/audit/<audit-id>/
```

审计对象放在：

```text
audit-request.json
```

里面记录。

---

## 1.4 Audit 是复查，不是重新执行

Audit Agent 只读取证据链并生成报告。

Audit 不允许：

```text
不写用户源码
不修改 input issue
不修改 Approved SPEC
不修改 execute run
不修改 patch
不修改 evidence
不修改 release delivery
不执行命令
不创建 PR
不 merge
不 deploy
不调用模型
```

Audit 只能写：

```text
.agentflow/output/audit/<audit-id>/**
```

---

# 2. Audit V1 固定审计清单

Audit V1 只审 Build Agent 的执行证据链是否完整、是否越界、是否可追溯。

固定 7 项检查：

```text
1. 改之前有没有 checkpoint？
2. 改了哪些文件？
3. 是否只改 allowedWritePaths？
4. 命令是否都被记录？
5. 高风险 issue 是否有人类确认？
6. evidence 是否完整？
7. release delivery 是否完整？
```

这 7 项是 Audit V1 的核心，不做泛泛代码质量审查。

---

# 3. Audit 目标结构

人类触发一次 audit 后，生成：

```text
.agentflow/output/audit/
├── manifest.json
├── index.json
│
└── <audit-id>/
    ├── audit-request.json
    ├── audit.json
    ├── audit-report.md
    ├── findings.json
    ├── checklist.md
    ├── evidence-map.json
    └── traceability.json
```

注意：

```text
output/audit/manifest.json 和 output/audit/index.json 可以在 prepare 阶段创建。
output/audit/<audit-id>/ 只能在人类触发 audit 时创建。
```

---

# 4. Audit Manifest

路径：

```text
.agentflow/output/audit/manifest.json
```

职责：

```text
记录 audit 子系统状态
记录 audit report 数量
记录最近 audit
```

示例：

```json
{
  "version": "audit-manifest.v1",
  "projectRoot": "/path/to/project",
  "status": "ready",
  "paths": {
    "auditRoot": ".agentflow/output/audit",
    "index": ".agentflow/output/audit/index.json"
  },
  "summary": {
    "audits": 0,
    "passed": 0,
    "passedWithWarnings": 0,
    "failed": 0,
    "cancelled": 0
  }
}
```

---

# 5. Audit Index

路径：

```text
.agentflow/output/audit/index.json
```

职责：

```text
快速索引所有人类触发的 audit report
```

示例：

```json
{
  "version": "audit-index.v1",
  "updatedAt": 1780360000,
  "audits": [
    {
      "auditId": "audit-001",
      "status": "passed",
      "requestedBy": "human",
      "requestedAt": 1780360000,
      "reportPath": ".agentflow/output/audit/audit-001/audit-report.md",
      "auditPath": ".agentflow/output/audit/audit-001/audit.json"
    }
  ]
}
```

---

# 6. Audit Request

## 6.1 路径

```text
.agentflow/output/audit/<audit-id>/audit-request.json
```

## 6.2 职责

记录：

```text
谁触发审计
什么时候触发
为什么触发
审计范围是什么
```

注意：

```text
不要使用 auditType = run / project / batch 这种分类。
```

使用统一的：

```text
scope
```

来记录审计范围。

## 6.3 示例

```json
{
  "version": "audit-request.v1",
  "auditId": "audit-001",
  "requestedBy": "human",
  "requestedAt": 1780360000,
  "reason": "Human requested full review before accepting this delivery.",
  "scope": {
    "description": "Review Build Agent delivery for ownership guard implementation.",
    "refs": [
      {
        "kind": "spec",
        "id": "spec-001",
        "path": ".agentflow/input/specs/approved/spec-001/"
      },
      {
        "kind": "issue",
        "id": "iss-001",
        "path": ".agentflow/input/issues/iss-001.json"
      },
      {
        "kind": "execute-run",
        "id": "run-001",
        "path": ".agentflow/execute/runs/run-001/"
      },
      {
        "kind": "evidence",
        "id": "run-001",
        "path": ".agentflow/output/evidence/run-001.json"
      },
      {
        "kind": "release-delivery",
        "id": "run-001",
        "path": ".agentflow/output/release/run-001/"
      }
    ]
  }
}
```

---

# 7. audit.json

## 7.1 路径

```text
.agentflow/output/audit/<audit-id>/audit.json
```

## 7.2 职责

机器可读审计结论。

## 7.3 状态

```text
passed
passed-with-warnings
failed
cancelled
```

## 7.4 示例

```json
{
  "version": "output-audit.v1",
  "auditId": "audit-001",
  "requestedBy": "human",
  "requestedAt": 1780360000,
  "status": "passed",
  "summary": {
    "checks": 7,
    "passed": 7,
    "warnings": 0,
    "failed": 0,
    "findings": 0
  },
  "checks": {
    "checkpointExists": "passed",
    "changedFilesRecorded": "passed",
    "allowedWritePathsOnly": "passed",
    "commandsRecorded": "passed",
    "highRiskConfirmedIfNeeded": "passed",
    "evidenceComplete": "passed",
    "releaseDeliveryComplete": "passed"
  },
  "paths": {
    "request": ".agentflow/output/audit/audit-001/audit-request.json",
    "report": ".agentflow/output/audit/audit-001/audit-report.md",
    "findings": ".agentflow/output/audit/audit-001/findings.json",
    "checklist": ".agentflow/output/audit/audit-001/checklist.md",
    "evidenceMap": ".agentflow/output/audit/audit-001/evidence-map.json",
    "traceability": ".agentflow/output/audit/audit-001/traceability.json"
  }
}
```

---

# 8. audit-report.md

## 8.1 路径

```text
.agentflow/output/audit/<audit-id>/audit-report.md
```

## 8.2 职责

人类可读完整审计报告。

## 8.3 模板

```md
# Audit Report

## 1. Summary

- Audit ID:
- Status:
- Requested By:
- Requested At:
- Reason:
- Scope:
- Final Verdict:

## 2. Audit Scope

本次审计范围：

- Approved SPEC:
- Input Issue:
- Execute Run:
- Evidence:
- Release Delivery:
- Panel Snapshot:
- Context Pack:

不在本次审计范围：

- 不重新写代码
- 不重新执行 patch
- 不修改 input facts
- 不修改 Approved SPEC
- 不创建 PR
- 不 merge
- 不 deploy

## 3. Traceability

| Layer | Path | Status |
|---|---|---|
| Approved SPEC |  |  |
| Issue |  |  |
| Execute Run |  |  |
| Evidence |  |  |
| Release Delivery |  |  |

## 4. Core Checks

### 4.1 Checkpoint

- Checkpoint exists:
- Checkpoint path:
- Git head before:
- Dirty files before:

Verdict:

### 4.2 Changed Files

- changed-files.json exists:
- worktree.diff exists:
- diff-summary.json exists:

Changed files:

| File | Change Type | Insertions | Deletions |
|---|---|---:|---:|

Verdict:

### 4.3 Allowed Write Paths

- plan.json exists:
- allowedWritePaths exists:
- all changed files inside allowedWritePaths:

Verdict:

### 4.4 Command Records

Commands executed:

| Command ID | Program | Args | Exit Code | Record |
|---|---|---|---:|---|

Checks:

- Every command has JSON record:
- Every command has stdout path:
- Every command has stderr path:

Verdict:

### 4.5 High Risk Confirmation

- issue riskLevel:
- high risk confirmation required:
- high risk confirmation exists:
- confirmation path:

Verdict:

### 4.6 Evidence Completeness

Evidence file:

- path:
- exists:
- references run:
- references preflight:
- references result:
- references checkpoint:
- references diff:
- references changed files:
- references command records:
- validation status matches result:

Verdict:

### 4.7 Release Delivery Completeness

Release delivery path:

Required files:

- delivery.json:
- pr-draft.md:
- pr-metadata.json:
- review-checklist.md:
- changelog.md:
- release-note.md:

Checks:

- delivery references evidence:
- delivery createdBy = Build Agent:
- PR draft exists:
- review checklist exists:
- no merge / deploy performed:

Verdict:

## 5. Findings

| Severity | Category | Finding | Recommendation |
|---|---|---|---|

## 6. Final Verdict

Status:

Reason:

Recommended next action:
```

---

# 9. findings.json

## 9.1 路径

```text
.agentflow/output/audit/<audit-id>/findings.json
```

## 9.2 职责

结构化问题列表。

## 9.3 finding severity

```text
low
medium
high
critical
```

注意：

```text
finding.severity 不是 issue.riskLevel
```

## 9.4 示例

```json
{
  "version": "audit-findings.v1",
  "auditId": "audit-001",
  "findings": [
    {
      "findingId": "finding-001",
      "severity": "high",
      "category": "allowed-write-paths",
      "title": "Changed file outside allowedWritePaths",
      "detail": "Patch modified README.md, but README.md was not listed in run plan allowedWritePaths.",
      "evidencePath": ".agentflow/execute/runs/run-001/patches/changed-files.json",
      "recommendation": "Review the patch manually and either reject the change or regenerate the run plan with explicit approval."
    }
  ]
}
```

---

# 10. checklist.md

## 10.1 路径

```text
.agentflow/output/audit/<audit-id>/checklist.md
```

## 10.2 模板

```md
# Audit Checklist

## Core Checks

- [ ] Checkpoint exists before patch / command.
- [ ] Changed files are recorded.
- [ ] Changed files are inside allowedWritePaths.
- [ ] Commands are fully recorded.
- [ ] High risk issue has human confirmation if required.
- [ ] Evidence is complete.
- [ ] Release delivery is complete.

## Result

- [ ] Passed
- [ ] Passed with warnings
- [ ] Failed

## Notes

<Audit Agent notes>
```

---

# 11. evidence-map.json

## 11.1 路径

```text
.agentflow/output/audit/<audit-id>/evidence-map.json
```

## 11.2 职责

记录审计报告使用了哪些证据。

## 11.3 示例

```json
{
  "version": "audit-evidence-map.v1",
  "auditId": "audit-001",
  "inputs": {
    "approvedSpec": ".agentflow/input/specs/approved/spec-001/",
    "issue": ".agentflow/input/issues/iss-001.json",
    "run": ".agentflow/execute/runs/run-001/run.json",
    "preflight": ".agentflow/execute/runs/run-001/preflight.json",
    "plan": ".agentflow/execute/runs/run-001/plan.json",
    "checkpoint": ".agentflow/execute/runs/run-001/checkpoints/chk-001.json",
    "changedFiles": ".agentflow/execute/runs/run-001/patches/changed-files.json",
    "diff": ".agentflow/execute/runs/run-001/patches/worktree.diff",
    "result": ".agentflow/execute/runs/run-001/result.json",
    "evidence": ".agentflow/output/evidence/run-001.json",
    "releaseDelivery": ".agentflow/output/release/run-001/delivery.json"
  }
}
```

---

# 12. traceability.json

## 12.1 路径

```text
.agentflow/output/audit/<audit-id>/traceability.json
```

## 12.2 职责

记录从需求到审计的完整链路。

## 12.3 示例

```json
{
  "version": "audit-traceability.v1",
  "auditId": "audit-001",
  "chain": [
    {
      "layer": "spec",
      "id": "spec-001",
      "path": ".agentflow/input/specs/approved/spec-001/"
    },
    {
      "layer": "issue",
      "id": "iss-001",
      "path": ".agentflow/input/issues/iss-001.json"
    },
    {
      "layer": "execute-run",
      "id": "run-001",
      "path": ".agentflow/execute/runs/run-001/"
    },
    {
      "layer": "evidence",
      "id": "run-001",
      "path": ".agentflow/output/evidence/run-001.json"
    },
    {
      "layer": "release-delivery",
      "id": "run-001",
      "path": ".agentflow/output/release/run-001/"
    },
    {
      "layer": "audit",
      "id": "audit-001",
      "path": ".agentflow/output/audit/audit-001/"
    }
  ]
}
```

---

# 13. 审计结论规则

Audit V1 结论：

```text
passed
passed-with-warnings
failed
cancelled
```

规则：

```text
7 项核心检查全部 passed
→ passed

没有 failed，但有 warning / low finding
→ passed-with-warnings

任意核心检查 failed
→ failed

人类取消
→ cancelled
```

---

# 14. Agent 角色边界

## 14.1 Audit Agent

Audit Agent 负责：

```text
读取 input
读取 execute
读取 output/evidence
读取 output/release
生成完整 audit report package
```

Audit Agent 写：

```text
.agentflow/output/audit/<audit-id>/**
```

Audit Agent 不写：

```text
.agentflow/input/**
.agentflow/execute/**
.agentflow/output/evidence/**
.agentflow/output/release/**
用户源码
```

---

## 14.2 Build Agent

Build Agent 不自动触发 audit。

Build Agent 只负责：

```text
execute
evidence
release delivery
```

---

## 14.3 人类

只有人类可以触发 audit。

---

# 15. API 设计

建议新增：

```text
request_human_audit
load_audit_report
load_audit_index
load_audit_status
```

---

## 15.1 request_human_audit

```text
request_human_audit(projectRoot, reason, scopeRefs)
```

行为：

```text
1. 创建 auditId
2. 创建 output/audit/<audit-id>/
3. 写 audit-request.json
4. 读取 scopeRefs 里的 evidence chain
5. 执行 7 项核心检查
6. 写 audit.json
7. 写 audit-report.md
8. 写 findings.json
9. 写 checklist.md
10. 写 evidence-map.json
11. 写 traceability.json
12. 更新 output/audit/index.json
```

---

## 15.2 load_audit_report

```text
load_audit_report(projectRoot, auditId)
```

读取：

```text
audit.json
audit-report.md
findings.json
checklist.md
evidence-map.json
traceability.json
```

---

# 16. Rust 模块建议

如果 011 已经新增：

```text
crates/output
```

则在其中新增：

```text
crates/output/src/audit.rs
```

如果 011 尚未实现，也可以先在 012 中扩展：

```text
crates/output
```

建议模块：

```text
crates/output/
├── src/
│   ├── audit.rs
│   ├── audit_model.rs
│   ├── audit_report.rs
│   ├── audit_checks.rs
│   └── audit_index.rs
```

---

# 17. Tauri commands

新增：

```text
request_human_audit
load_audit_report
load_audit_index
load_audit_status
```

注意：

```text
Desktop human UI 可以触发 audit。
```

这是少数允许人类主动触发写入 output/audit 的动作。

---

# 18. Desktop UI

V1 支持：

```text
人类点击触发 Audit
查看 audit report
查看 findings
查看 checklist
查看 evidence map
查看 traceability
```

不支持：

```text
编辑 audit report
修改 findings
修改 input / execute / output evidence / release
```

---

# 19. 不允许做

本需求不做：

```text
不自动每次 execute 后审计
不按 run/project/batch 分三套模型
不写用户源码
不修改 input issue
不修改 Approved SPEC
不修改 execute run
不修改 evidence
不修改 release delivery
不执行命令
不创建 PR
不 merge
不 deploy
不调用模型
不自动生成 follow-up issue
```

---

# 20. 测试要求

必须新增测试：

```text
1. prepare audit space only creates manifest / index, not audit report.
2. request_human_audit creates output/audit/<audit-id>/.
3. request_human_audit writes audit-request.json.
4. request_human_audit writes audit.json.
5. request_human_audit writes audit-report.md.
6. request_human_audit writes findings.json.
7. request_human_audit writes checklist.md.
8. request_human_audit writes evidence-map.json.
9. request_human_audit writes traceability.json.
10. audit checks fail when checkpoint is missing.
11. audit checks fail when changed-files is missing.
12. audit checks fail when changed file is outside allowedWritePaths.
13. audit checks fail when command record is incomplete.
14. high risk issue without confirmation fails audit.
15. missing evidence fails audit.
16. missing release delivery fails audit.
17. valid evidence chain passes audit.
18. audit does not modify input / execute / evidence / release.
```

---

# 21. 验收标准

```text
- [ ] 新增 docs/requirements/012-human-triggered-audit-report-v1.md。
- [ ] Audit 不自动随 execute / output 生成。
- [ ] prepare 只创建 output/audit manifest / index / root space。
- [ ] 不创建 output/audit/<audit-id>/，除非人类触发。
- [ ] 不使用 output/audit/runs/、output/audit/projects/、output/audit/batches/ 分类目录。
- [ ] 人类触发 audit 后生成 output/audit/<audit-id>/。
- [ ] audit package 包含 audit-request.json。
- [ ] audit package 包含 audit.json。
- [ ] audit package 包含 audit-report.md。
- [ ] audit package 包含 findings.json。
- [ ] audit package 包含 checklist.md。
- [ ] audit package 包含 evidence-map.json。
- [ ] audit package 包含 traceability.json。
- [ ] Audit V1 固定检查 7 项。
- [ ] 检查 checkpoint 是否存在。
- [ ] 检查 changed files 是否记录。
- [ ] 检查 changed files 是否在 allowedWritePaths。
- [ ] 检查 command records 是否完整。
- [ ] 检查 high risk confirmation。
- [ ] 检查 evidence 是否完整。
- [ ] 检查 release delivery 是否完整。
- [ ] audit-report.md 是完整人类可读报告。
- [ ] audit.json 是机器可读结论。
- [ ] findings.json 是结构化问题列表。
- [ ] evidence-map.json 记录审计证据。
- [ ] traceability.json 记录完整追溯链。
- [ ] Desktop 可以触发 human audit。
- [ ] Desktop 可以只读展示 audit report。
- [ ] Audit 不修改 input。
- [ ] Audit 不修改 execute。
- [ ] Audit 不修改 evidence。
- [ ] Audit 不修改 release delivery。
- [ ] Audit 不写用户源码。
- [ ] Audit 不执行命令。
- [ ] Audit 不创建 PR / merge / deploy。
- [ ] Audit 不调用模型。
- [ ] cargo fmt --check 通过。
- [ ] cargo test -p agentflow-output 通过。
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
cargo test -p agentflow-desktop
cargo test
npm --prefix apps/desktop run build
git diff --check
```

---

# 23. PR 说明要求

PR 描述必须说明：

```text
1. Audit 为什么不是每次 execute 后自动执行。
2. 为什么不按 run / project / batch 分三套模型。
3. 人类触发 audit 后会生成哪些完整报告文件。
4. Audit V1 的 7 项核心检查是什么。
5. audit-report.md 包含哪些人类可读内容。
6. audit.json / findings.json / evidence-map.json / traceability.json 的作用。
7. Audit 不修改 input / execute / evidence / release。
8. Audit 不执行命令。
9. Audit 不创建 PR / merge / deploy。
10. Audit 不调用模型。
11. 验证命令和结果。
```

---

# 24. Codex 执行指令

```md
请执行 012 - Human-triggered Audit Report V1。

目标：
实现 AgentFlow 的人类触发审计报告能力。Audit 不自动随 execute / output 生成；只有人类明确触发 audit 时，才生成 `.agentflow/output/audit/<audit-id>/` 下的一整套完整审计报告包。不按 run / project / batch 分类建三套模型，而是统一生成一个 Human-triggered Audit Report。

必须遵守：
1. Audit 不自动每次 execute 后运行。
2. Audit 不自动每次 output 后运行。
3. prepare 只创建 audit manifest / index / root space。
4. 人类触发后才创建 output/audit/<audit-id>/。
5. 不使用 output/audit/runs/、projects/、batches/ 分类目录。
6. audit package 必须包含 audit-request.json / audit.json / audit-report.md / findings.json / checklist.md / evidence-map.json / traceability.json。
7. Audit V1 固定检查 7 项：checkpoint、changed files、allowedWritePaths、command records、high risk confirmation、evidence completeness、release delivery completeness。
8. audit-report.md 必须是完整人类可读报告。
9. audit.json 必须是机器可读结论。
10. Audit 不修改 input facts。
11. Audit 不修改 execute facts。
12. Audit 不修改 evidence。
13. Audit 不修改 release delivery。
14. Audit 不写用户源码。
15. Audit 不执行命令。
16. Audit 不创建 PR。
17. Audit 不 merge。
18. Audit 不 deploy。
19. Audit 不调用模型。

实现范围：
- 新增 docs/requirements/012-human-triggered-audit-report-v1.md。
- 在 agentflow-output 中新增 audit model / audit request / audit report / findings / checklist / evidence map / traceability。
- 新增 request_human_audit。
- 新增 load_audit_report。
- 新增 load_audit_index。
- 新增 load_audit_status。
- Desktop 支持人类触发 audit。
- Desktop 支持只读查看 audit report。
- Browser Preview mock 更新。
- README / GOAL / ROADMAP / requirements / verification 更新。
- 增加测试覆盖完整 audit package 和 7 项检查。

验证命令：
- cargo fmt --check
- cargo test -p agentflow-output
- cargo test -p agentflow-desktop
- cargo test
- npm --prefix apps/desktop run build
- git diff --check
```

---

# 25. 完成定义

本需求完成后：

```text
Audit
= 人类触发的完整审计报告
```

不会自动生成：

```text
每次 execute 后自动 audit
每次 output 后自动 audit
```

只会在人类触发时生成：

```text
.agentflow/output/audit/<audit-id>/
├── audit-request.json
├── audit.json
├── audit-report.md
├── findings.json
├── checklist.md
├── evidence-map.json
└── traceability.json
```

最终一句话：

> **Audit 不是默认流水线步骤；但一旦人类触发，就必须生成一份完整、可追溯、可复查的正式审计报告。**
