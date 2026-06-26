# 014 - AgentFlow End-to-End Workflow Acceptance V1

创建日期：2026-06-05  
执行者：Codex  
状态：已开发  
版本：final

---

## 用户目标

当前 AgentFlow 已经陆续完成了多层基础能力：

```text
define/
= Agent 工作手册 / 规则 / skills / AGENTS.md

panel/
= 项目工作现场 / 文件 / 语言 / 符号 / Git / 诊断 / 测试 / Context Pack

input/
= 需求实时源头 / SPEC Gate / Projects / Issues

execute/
= 受控执行流水线 / run / preflight / lease / plan / checkpoint / patch / command / validation / result

output/
= 证据 / release delivery / human audit

state/
= 全局派生状态 / gates / next actions / blockers / locks / sessions / indexes
```

现在需要做一次端到端验收。

大白话：

> 单个模块通过测试还不够。  
> 我们要证明 AgentFlow 从“打开本地项目”到“交付后人工审计”这条主链路真的能串起来。  
> 这次不是新增功能，而是用一个本地 fixture 项目，把 define → panel → input → execute → output → state → human audit 全链路跑通，确认路径、状态、gate、写入边界和 Desktop / Browser Preview 都一致。

---

## 一句话定义

> **014 AgentFlow End-to-End Workflow Acceptance V1 是 AgentFlow 的系统级验收需求。它用本地 fixture 项目验证 define / panel / input / execute / output / state / human audit 的完整闭环，确保各模块之间的状态流转、路径约定、gate 判断、写入边界和 Desktop 展示都正确。**

---

# 1. 背景

PR #27 已完成：

```text
012.1 - Desktop Human Audit Entry Polish
013 - Workflow State / Gate Orchestration V1
```

PR #27 已经实现：

```text
agentflow-state
.agentflow/state/**
workflow gates
next actions
blockers
sessions
locks
events
indexes
Desktop 工作流状态通道
Desktop 人工审计入口
```

但 PR #27 仍有一个验证缺口：

```text
Browser Preview 可视核对被 net::ERR_BLOCKED_BY_CLIENT 拦截
```

因此在进入 014 前，建议先完成：

```text
013.1 - Browser Preview Verification Polish
```

014 不重复做 013.1 的 smoke 修复，但会把 Browser Preview smoke 作为端到端验收的一部分。

---

# 2. 核心原则

## 2.1 014 是验收，不是新功能

本需求不新增新的产品能力。

它只做：

```text
端到端 fixture
端到端测试
状态一致性检查
路径一致性检查
写入边界检查
Desktop / Browser Preview smoke 验证
文档和 verification 更新
```

不做：

```text
不新增 OpenSpec Authoring
不新增 AgentRun 自动执行
不调用模型
不创建远程 PR
不 merge
不 deploy
不写用户源码
不改 TDD / Release / Audit 核心模型
```

---

## 2.2 014 要验证真实链路，不只验证单模块

必须覆盖：

```text
Project Workspace prepare
Define prepare
Panel prepare
Input prepare
Execute prepare
Output prepare
State prepare
Approved SPEC fixture
Issue fixture
Execute run fixture
Evidence fixture
Release delivery fixture
State gate 推导
Human Audit request
Audit report load
Browser Preview read-only
```

---

## 2.3 014 必须验证写入边界

整个验收链路只允许写：

```text
AGENTS.md
.agentflow/**
```

不允许写：

```text
用户源码
远程 PR
远程 Issue
deploy 记录
未授权 SPEC
未授权 Goal Tree
未授权 AgentRun
```

如果测试需要 fixture 源码文件，只能在临时 fixture 项目初始化阶段创建，后续 AgentFlow 流程不能修改它。

---

# 3. 目标链路

014 要验证的主链路：

```text
创建本地 fixture 项目
  ↓
Project Workspace prepare
  ↓
Define ready
  ↓
Panel ready / degraded
  ↓
Input ready
  ↓
Execute ready
  ↓
Output ready
  ↓
State ready
  ↓
写入 approved SPEC fixture
  ↓
写入 issue fixture
  ↓
创建 execute run fixture
  ↓
执行 preflight / plan / checkpoint / command / validation fixture
  ↓
写入 evidence fixture
  ↓
写入 release delivery fixture
  ↓
refresh state
  ↓
state 推导 delivery-ready
  ↓
Desktop / Tauri 可看到 request-human-audit next action
  ↓
request_human_audit 生成 audit package
  ↓
load_audit_report 可读取 audit-report.md
  ↓
refresh state
  ↓
state 推导 audit status
```

---

# 4. 范围

本需求包含 8 个范围：

```text
1. End-to-end fixture project
2. Workspace prepare acceptance
3. Workflow state transition acceptance
4. Human audit acceptance
5. Write boundary acceptance
6. Desktop / Tauri command acceptance
7. Browser Preview smoke acceptance
8. Documentation and verification update
```

---

# 5. 非目标

本需求不做以下事情：

```text
不新增 OpenSpec Authoring V1
不新增 SPEC 编辑器
不新增 Goal Tree materializer
不新增 Agent 自动执行
不新增多 Agent 调度
不新增 Lease 抢任务
不新增真实远程 PR
不新增 merge
不新增 deploy
不调用模型
不执行用户项目测试命令
不修改用户源码
不做 LSP diagnostics
不做 CRDT
不做多人协作
不做 Review Changes UI
不做 checkpoint restore UI
不自动触发 audit
```

---

# 6. 建议实现位置

优先方案：

```text
crates/workflow-acceptance/
```

Cargo package：

```text
agentflow-workflow-acceptance
```

原因：

```text
014 是跨 crate 的端到端验收
不应该塞进 state / output / execute 单个 crate
独立 crate 更清晰
可以依赖 agent-manual / panel / input / execute / output / state
```

建议加入 workspace：

```toml
members = [
  ...
  "crates/workflow-acceptance"
]
```

如果项目不希望新增 crate，也可以放在：

```text
tests/workflow_acceptance/
```

但推荐新增 crate，因为后续端到端验收可能会持续扩展。

---

## 6.1 推荐 crate 结构

```text
crates/workflow-acceptance/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── fixture.rs
    ├── acceptance.rs
    ├── boundaries.rs
    ├── browser_preview.rs
    └── assertions.rs
```

---

## 6.2 依赖建议

```toml
[dependencies]
agentflow-agent-manual = { path = "../agent-manual" }
agentflow-panel = { path = "../panel" }
agentflow-input = { path = "../input" }
agentflow-execute = { path = "../execute" }
agentflow-output = { path = "../output" }
agentflow-state = { path = "../state" }
anyhow = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }

[dev-dependencies]
tempfile = { workspace = true }
```

---

# 7. End-to-End Fixture Project

## 7.1 Fixture 项目内容

创建一个临时项目：

```text
fixture-project/
├── README.md
├── Cargo.toml
└── src/
    └── lib.rs
```

示例源码：

```rust
pub fn value() -> u8 {
    1
}
```

注意：

```text
这个源码只在 fixture 初始化时创建
AgentFlow 流程不能修改它
```

---

## 7.2 Fixture 初始化

函数建议：

```rust
fn create_fixture_project() -> TempDir
```

写入：

```text
README.md
Cargo.toml
src/lib.rs
```

然后记录初始源码 hash：

```rust
let source_hash_before = hash_file("src/lib.rs")
```

后续验收时确认：

```rust
source_hash_after == source_hash_before
```

---

# 8. Workspace Prepare Acceptance

## 8.1 目标

验证打开项目后，工作环境完整准备。

调用链：

```rust
agentflow_agent_manual::prepare_agent_working_manual(project_root)
agentflow_panel::prepare_project_panel(project_root, Blocking)
agentflow_input::prepare_input_workspace(project_root)
agentflow_execute::prepare_execute_workspace(project_root)
agentflow_output::prepare_output_workspace(project_root)
agentflow_state::prepare_state_workspace(project_root)
```

如果已有 Project Workspace prepare API 能一键调用，则优先使用真实 prepare API。

---

## 8.2 必须断言

```text
AGENTS.md 存在
.agentflow/workspace-manifest.json 存在
.agentflow/define/agent/Agentflow.md 存在
.agentflow/define/agent/skills-lock.json 存在
.agentflow/define/spec/SPEC.md 存在
.agentflow/define/tdd/TDD.md 存在
.agentflow/define/release/RELEASE.md 存在
.agentflow/define/audit/AUDIT.md 存在
.agentflow/panel/manifest.json 存在
.agentflow/input/manifest.json 存在
.agentflow/execute/manifest.json 存在
.agentflow/output/manifest.json 存在
.agentflow/state/manifest.json 存在
.agentflow/state/gates/workflow.json 存在
```

---

## 8.3 断言状态

```text
define ready
panel ready 或 degraded
input ready
execute ready
output ready
state ready 或 degraded
```

---

# 9. Workflow State Transition Acceptance

## 9.1 目标

验证 state 可以正确推导主流程状态。

---

## 9.2 需要覆盖的阶段

至少覆盖：

```text
workspace-ready
panel-ready
input-ready
issue-ready
execute-ready
execute-running
execute-completed
evidence-ready
delivery-ready
audit-completed
```

如果某些状态因为当前实现合并或跳过，测试可以验证当前实现实际可达状态，但必须记录原因。

---

## 9.3 Approved SPEC fixture

写入 approved SPEC fixture：

```text
.agentflow/input/specs/approved/spec-001/
├── product.md
├── tech.md
├── spec.json
└── approval.json
```

要求：

```text
不能通过 conversation 直接生成 Goal Tree
不能写 .agentflow/goal-tree/**
只是准备 input fixture
```

---

## 9.4 Issue fixture

写入 issue fixture：

```text
.agentflow/input/issues/iss-001.json
```

字段应包含：

```text
issueId
sourceSpecId
title
summary
status = ready-for-execute
riskLevel = low
validationHints
scope
```

---

## 9.5 Execute run fixture

通过 execute crate 创建 run：

```text
create_execute_run
execute_run_preflight
acquire_execute_lease
write_execute_plan
create_execute_checkpoint
run_execute_command
validate_execute_run
```

命令只能用安全命令：

```text
printf ok
```

不允许运行项目测试命令。

---

## 9.6 Evidence fixture

写入：

```text
.agentflow/output/evidence/<run-id>.json
```

必须引用：

```text
input issue
approved spec
execute run
preflight
plan
result
checkpoint
command output
validation summary
```

---

## 9.7 Release delivery fixture

写入：

```text
.agentflow/output/release/<run-id>/delivery.json
.agentflow/output/release/<run-id>/pr-draft.md
.agentflow/output/release/<run-id>/review-checklist.md
.agentflow/output/release/<run-id>/changelog.md
.agentflow/output/release/<run-id>/release-note.md
.agentflow/output/release/<run-id>/pr-metadata.json
```

必须是本地 draft，不创建远程 PR。

---

## 9.8 State refresh

调用：

```rust
agentflow_state::refresh_state(project_root)
```

断言：

```text
currentStage = delivery-ready
auditStatus = not-requested
nextActions 包含 request-human-audit
nextActions 包含 start-new-input
blockers 不包含自动 audit required
```

---

# 10. Human Audit Acceptance

## 10.1 目标

验证 human audit 只能由人类显式触发，且触发后能读取报告。

---

## 10.2 请求 audit

调用：

```rust
agentflow_output::request_human_audit(project_root, draft)
```

draft 必须包含：

```text
reason 非空
scope refs 自动构造
```

refs 包含：

```text
spec
issue
execute-run
evidence
release-delivery
```

---

## 10.3 必须断言

```text
.agentflow/output/audit/<audit-id>/audit-request.json 存在
.agentflow/output/audit/<audit-id>/audit.json 存在
.agentflow/output/audit/<audit-id>/audit-report.md 存在
.agentflow/output/audit/<audit-id>/findings.json 存在
.agentflow/output/audit/<audit-id>/checklist.md 存在
.agentflow/output/audit/<audit-id>/evidence-map.json 存在
.agentflow/output/audit/<audit-id>/traceability.json 存在
```

---

## 10.4 load audit report

调用：

```rust
agentflow_output::load_audit_report(project_root, audit_id)
```

断言：

```text
reportMarkdown 非空
audit.auditId 匹配
audit.status 是 passed / passed-with-warnings / failed / cancelled 之一
findings 可读
checklist 可读
evidenceMap 可读
traceability 可读
```

---

## 10.5 State refresh after audit

调用：

```rust
agentflow_state::refresh_state(project_root)
```

断言：

```text
auditStatus != not-requested
currentStage 是 audit-completed 或 audit-requested / audit-running / audit-completed 中符合当前实现的状态
```

如果当前 audit 是同步生成报告，则应期望：

```text
currentStage = audit-completed
```

---

# 11. Write Boundary Acceptance

## 11.1 目标

验证端到端流程没有写未授权路径。

---

## 11.2 必须记录 baseline

在执行链路前记录：

```text
src/lib.rs hash
README.md hash
Cargo.toml hash
```

链路完成后再次计算。

断言：

```text
hash unchanged
```

---

## 11.3 禁止存在

```text
.git/refs/heads/agentflow-*
.remote-pr
deploy record
用户源码变更
```

如果 fixture 没有 git，也不要自动 git init，除非已有 prepare 逻辑要求。  
如需测试 Git 状态，可单独创建 git fixture，但不能污染主验收。

---

## 11.4 允许写入

```text
AGENTS.md
.agentflow/**
```

---

# 12. Desktop / Tauri Acceptance

## 12.1 目标

验证 Tauri commands 能读到端到端状态。

如果在 Rust 层不方便直接测 Tauri command，可以通过 `agentflow-desktop` tests 验证 command wrappers。

---

## 12.2 必须覆盖 commands

```text
load_state_status
load_workflow_gates
load_next_actions
load_blockers
load_output_status
load_output_index
load_audit_index
load_audit_report
```

---

## 12.3 必须断言

```text
load_state_status.currentStage 可读
load_next_actions 包含 request-human-audit
load_output_index.releaseDeliveries 非空
load_audit_index.audits 非空
load_audit_report.reportMarkdown 非空
```

---

# 13. Browser Preview Acceptance

## 13.1 前置

014 默认要求 013.1 已经补齐 deterministic Browser Preview smoke。

---

## 13.2 必须验证

```text
Browser Preview 可渲染
工作流状态可见
人工审计入口可见
Browser Preview 下 Request Human Audit 按钮不会执行真实写入
Browser Preview 不调用 request_human_audit
Browser Preview 不写 .agentflow/output/audit
```

---

## 13.3 验证方式

如果 013.1 新增了：

```text
npm --prefix apps/desktop run preview:smoke
```

014 必须执行该命令。

如果 013.1 使用前端测试，则执行对应测试命令。

---

# 14. 测试建议

## 14.1 Rust E2E tests

建议新增测试：

```text
full_workflow_reaches_delivery_ready
human_audit_request_updates_state
write_boundary_keeps_user_source_unchanged
high_risk_issue_creates_blocker
stale_lease_is_reported
```

---

## 14.2 State gate tests

覆盖：

```text
delivery-ready
audit-completed
high-risk-blocked
stale-lock
```

---

## 14.3 Desktop tests

覆盖：

```text
state commands readable
audit report commands readable
```

---

## 14.4 Browser Preview smoke

覆盖：

```text
workflow state visible
human audit visible
preview-only no-write
```

---

# 15. 文档更新

需要更新：

```text
docs/requirements/014-agentflow-end-to-end-workflow-acceptance-v1.md
docs/requirements/README.md
docs/requirements/next-requirements.md
verification.md
```

可选更新：

```text
README.md
ROADMAP.md
GOAL.md
```

如果更新 README / ROADMAP / GOAL，必须只描述验收，不新增产品能力承诺。

---

# 16. 写入边界

014 允许写：

```text
docs/requirements/014-agentflow-end-to-end-workflow-acceptance-v1.md
crates/workflow-acceptance/**
tests/**
verification.md
docs/requirements/README.md
docs/requirements/next-requirements.md
```

测试运行时允许写临时 fixture 的：

```text
AGENTS.md
.agentflow/**
```

不允许测试写真实用户项目。

不允许写：

```text
真实项目源码
远程 PR
远程 issue
deploy record
model output
```

---

# 17. 验收标准

```text
- [ ] 新增 docs/requirements/014-agentflow-end-to-end-workflow-acceptance-v1.md。
- [ ] 新增端到端 fixture 项目测试。
- [ ] Workspace prepare 验证 AGENTS.md / define / panel / input / execute / output / state 均 ready 或符合预期。
- [ ] State 能推导 delivery-ready。
- [ ] delivery-ready 时 nextActions 包含 request-human-audit。
- [ ] audit 默认不是 required。
- [ ] request_human_audit 生成完整 audit package。
- [ ] load_audit_report 可读取 audit-report.md。
- [ ] refresh_state 后 auditStatus 不再是 not-requested。
- [ ] 高风险 issue 会形成 blocker。
- [ ] stale lease 会进入 stale locks 或 cleanup candidates。
- [ ] 用户源码 hash 在流程前后保持不变。
- [ ] 不创建远程 PR。
- [ ] 不 deploy。
- [ ] 不调用模型。
- [ ] 不自动触发 audit。
- [ ] Browser Preview smoke 可确认工作流状态可见。
- [ ] Browser Preview smoke 可确认人工审计入口可见。
- [ ] Browser Preview smoke 可确认不写 audit。
- [ ] cargo fmt --check 通过。
- [ ] cargo test -p agentflow-workflow-acceptance 通过。
- [ ] cargo test -p agentflow-state 通过。
- [ ] cargo test -p agentflow-desktop 通过。
- [ ] cargo test 通过。
- [ ] npm --prefix apps/desktop run build 通过。
- [ ] npm --prefix apps/desktop run preview:smoke 通过，如果 013.1 已新增。
- [ ] git diff --check 通过。
```

---

# 18. 验证命令

必须执行：

```bash
cargo fmt --check
cargo test -p agentflow-workflow-acceptance
cargo test -p agentflow-state
cargo test -p agentflow-desktop
cargo test
npm --prefix apps/desktop run build
git diff --check
```

如果 013.1 已新增 Browser Preview smoke：

```bash
npm --prefix apps/desktop run preview:smoke
```

如果没有新增独立 crate，而是用其他位置放 E2E tests，则把：

```bash
cargo test -p agentflow-workflow-acceptance
```

替换为实际测试命令，并在 PR 说明里解释。

---

# 19. PR 说明要求

PR 描述必须说明：

```text
1. 014 是端到端验收，不是新功能。
2. 覆盖了哪些链路。
3. fixture 项目如何构造。
4. 如何验证 state delivery-ready。
5. 如何验证 human audit request。
6. 如何验证 audit report 可读。
7. 如何验证用户源码未被修改。
8. 如何验证 Browser Preview 不写 audit。
9. 是否新增 crate：如果新增，说明为什么。
10. 本次没有调用模型。
11. 本次没有执行用户项目命令。
12. 本次没有创建远程 PR / merge / deploy。
13. 验证命令和结果。
```

---

# 20. Codex 执行指令

```md
请执行 014 - AgentFlow End-to-End Workflow Acceptance V1。

目标：
新增系统级端到端验收，使用本地 fixture 项目验证 define → panel → input → execute → output → state → human audit 的完整闭环。014 是验收，不是新功能。

必须遵守：
1. 不新增 OpenSpec Authoring。
2. 不新增 Goal Tree materializer。
3. 不新增 Agent 自动执行。
4. 不调用模型。
5. 不创建远程 PR。
6. 不 merge。
7. 不 deploy。
8. 不写真实用户源码。
9. 不自动触发 audit。
10. 不改 state / audit / execute / output 核心模型。
11. 测试只能使用临时 fixture 项目。
12. 允许 fixture 初始化时创建 README.md / Cargo.toml / src/lib.rs。
13. AgentFlow 流程执行后必须证明 fixture 源码 hash 未改变。

实现范围：
- 新增 docs/requirements/014-agentflow-end-to-end-workflow-acceptance-v1.md。
- 新增端到端 fixture。
- 建议新增 crates/workflow-acceptance，package 为 agentflow-workflow-acceptance。
- 验证 workspace prepare。
- 验证 state gate 流转到 delivery-ready。
- 验证 delivery-ready next action 包含 request-human-audit。
- 验证 audit 默认不是 required。
- 验证 request_human_audit 可生成 audit package。
- 验证 load_audit_report 可读。
- 验证 audit 后 refresh_state。
- 验证高风险 issue blocker。
- 验证 stale lease。
- 验证 write boundary。
- 接入 Browser Preview smoke，如果 013.1 已完成。
- 更新 requirements index 和 verification。

验证命令：
- cargo fmt --check
- cargo test -p agentflow-workflow-acceptance
- cargo test -p agentflow-state
- cargo test -p agentflow-desktop
- cargo test
- npm --prefix apps/desktop run build
- npm --prefix apps/desktop run preview:smoke
- git diff --check
```

---

# 21. 完成定义

完成后，AgentFlow 应能证明：

```text
打开本地项目
→ define ready
→ panel ready
→ input ready
→ execute ready
→ output ready
→ state ready
→ approved SPEC fixture
→ issue fixture
→ execute run fixture
→ evidence fixture
→ release delivery fixture
→ state delivery-ready
→ human audit request
→ audit report readable
→ state audit status updated
```

并且证明：

```text
没有写用户源码
没有调用模型
没有创建远程 PR
没有 deploy
没有自动 audit
Browser Preview 只读
```

最终一句话：

> **014 用端到端验收把 AgentFlow 现有工作流证明跑通：不是继续加能力，而是确认 define / panel / input / execute / output / state / audit 能稳定闭环。**
