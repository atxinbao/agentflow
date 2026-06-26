# AgentFlow v0.2.0 Project Loop / Issue Loop MVP 开发需求文档

> 文档类型：开发需求 / 架构收口 / MVP 任务边界
> 建议路径：`docs/requirements/033-agentflow-v0.2.0-project-loop-issue-loop-mvp-v1.md`
> 版本目标：v0.2.0
> 范围：只覆盖 Project Loop / Project Audit Gate / Issue Loop，不覆盖 Reader V2、多 Agent Runtime、完整 Execute Observability V2。
> 状态：Ready for Codex implementation planning

---

## 0. 本轮架构收口调整

v0.2.0 不再继续把 Project / Issue / Git provider / Audit Gate 的流程散落在 `input`、`execute`、`output`、`state` 和 Desktop UI 里。

本轮先新增两个底层模块：

```text
crates/loop
  AgentFlow 的流程编排层。
  负责 Project Loop、Issue Loop、Project Audit Gate 和状态推进。

crates/mcp
  AgentFlow 的外部能力适配层。
  负责 GitHub、GitLab、Codex、Browser Preview 等外部工具能力发现、健康检查和调用封装。
```

模块边界：

```text
input
  事实源：Project、Issue、SPEC、relations。

panel
  项目现场：Context Pack、文件、符号、诊断、测试摘要。

mcp
  外部能力：GitHub、GitLab、Codex、Browser Preview 等 provider。
  只返回能力状态和调用结果，不生成任务，不推进任务。

loop
  编排状态机：Project Loop、Issue Loop、Project Audit Gate。
  负责调度、状态转换、审计触发和 Project done 判断。

execute
  执行记录：run、claim、branch、plan、command、validation、complete。

output
  交付产物：evidence、release delivery、audit report。

state
  派生状态：index、projection、gate、blocker、UI refresh signal。

desktop
  展示和轻量触发：不作为事实源，不直接决定流程状态。
```

关键原则：

```text
1. .agentflow/input/** 仍然是唯一任务事实源。
2. GitHub / GitLab / Codex 只能作为 MCP provider，不允许成为任务源。
3. Project Loop 负责 backlog → todo、Issue done 后审计、Project done。
4. Issue Loop 只负责 todo → in_progress → in_review → done。
5. Audit Gate 是 Project Loop 内部验收，不创建 Audit Issue。
6. Build Agent 不写 audit report / findings / evidence-map / traceability。
7. MCP 只提供能力和结果，Loop 才能推进状态。
```

v0.2.0 的最小闭环：

```text
Spec Agent 写入 Project + backlog Issues
→ Project Loop 调度 backlog → todo
→ Issue Loop 接收 todo
→ Execute 记录 run / claim / branch / validation
→ Issue Loop 创建 PR/MR 并等待合并
→ Execute / Output 写回 Evidence + Release Delivery
→ Issue 状态 done
→ Project Loop 生成 Delivery Audit Report
→ 所有 Issues done 且 audits passed
→ Project Final Audit Report
→ Project 状态 done
```

---

## 0.1 本轮实现落地规则

本轮把 GitHub issues #84 - #92 收口为以下硬规则：

```text
#84 Project-bound Issue Generation
  新生成的 Issue 默认必须属于 Project。
  issueModel=project，projectId 必填，Project issueIds 必须包含该 Issue。
  Direct Issue 只作为 legacy 可读输入，不能进入 Project Loop scheduling。

#85 Project Environment Preflight
  Project Preflight 只检测当前 repo 对应的 Git provider。
  GitHub repo 只检测 GitHub，GitLab repo 只检测 GitLab。
  Provider 状态写入 .agentflow/state/mcp/providers/<provider>.json。
  Project Preflight 不直接修改 Issue 状态。

#86 Project Issue Scheduler
  Scheduler 只扫描当前 Project 下的 backlog issues。
  通过依赖、合同、Context Pack、Project Preflight 后，把 Issue 推到 todo。
  Scheduler 不创建 run，不执行命令，不创建 PR/MR。

#87 Issue Loop Runtime Preflight
  Issue Loop 只处理 todo issue。
  backlog issue 不能被 Build Agent 直接执行。
  create_execute_run 只创建 run、agent-claim.json 和 branch.json，不代表 Issue 已经开工。
  Runtime preflight 会确认 Context Pack 可读取或可补生成，并确认当前工作区没有未提交的用户源码改动；全部通过后才把 Issue 切到 in_progress。

#88 Issue Branch Check
  默认 issue branch 为 agentflow/<project-id>/<issue-id>。
  分支检查结果写入 .agentflow/execute/runs/<run-id>/branch.json。
  分支不匹配且工作区有未提交改动时阻断当前 Issue。

#89 PR/MR Auto Merge Default + Manual Fallback
  沙箱验证后 Issue 进入 in_review。
  auto-merge-if-eligible 默认尝试 provider 自动合并。
  自动合并不可用时保持 in_review，并写 manual-merge waiting proof。
  waiting-for-merge 不是 Issue 状态。

#90 Run-based Build Agent Complete
  build-agent complete 必须传 runId。
  complete 不能重新 create run。
  complete 必须校验 agent-claim.json、branch.json、review/merge-proof.json。
  complete 成功后写 evidence / release delivery，并把 Issue 从 in_review 切到 done。

#91 Delivery Audit Report
  Issue done 后由 Project Loop 直接生成 output/audit/delivery-<run-id>/audit-report.md。
  不创建 Audit Issue。

#92 Project Final Audit Report
  所有 Project Issues done 且 Delivery Audits passed 后生成 project-final audit package。
  Project 状态切到 done。
```

---

## 1. 一句话目标

AgentFlow v0.2.0 要先完成一个 **Project 完整 MVP 闭环**：

```text
Project Loop 负责项目级环境检查、Issue 调度、交付审计和 Project 完成判断。
Issue Loop 只负责一个 todo Issue 从执行到 done。
Audit 不再生成 Audit Issue，而是由 Project Loop 直接生成 Audit Report。
```

最终主链路：

```text
Project 初始化 / 刷新
→ Project Preflight
→ Project Issue Scheduling
→ backlog Issues 调度为 todo
→ Issue Loop 执行 todo Issue
→ in_progress
→ in_review
→ done
→ Project Loop 直接审计 Delivery
→ 输出 Audit Report
→ 所有 Project Issues done 且审计通过
→ Project done
```

---

## 2. 背景和问题

当前 AgentFlow 已经有任务、交付、审计等能力雏形，但 Project / Issue 的职责边界还需要收口。

当前需要解决的问题：

```text
1. Project 级调度和 Issue 级执行混在一起。
2. backlog → todo 的调度检查不应该由 Build Agent Issue Loop 处理。
3. GitHub / GitLab 登录、权限等是 Project 级环境，不应该放在每个 Issue 的 readiness 里重复判断。
4. input issues 不能继续生成没有 Project 容器的孤立 Issue。
5. Audit 不应该作为 Audit Issue 混入任务列表，MVP 阶段应由 Project Loop 直接生成 Audit Report。
6. Issue Loop 应简化为只处理 todo → done，并保留 in_review 作为 PR/MR 阶段。
7. Issue Loop 需要检测当前 Issue 对应的本地分支，保证执行对象、分支和 PR/MR 关系清晰。
```

---

## 3. v0.2.0 MVP 范围

### 3.1 本版本要做

```text
1. 定义 Project Loop。
2. 定义 Issue Loop。
3. input issue 生成逻辑统一为 Project 容器下的 issues。
4. Project Loop 在项目启动 / 刷新时检测 GitHub / GitLab 基础环境。
5. Project Loop 负责 backlog → todo。
6. Issue Loop 只处理 todo → in_progress → in_review → done。
7. Issue Loop 增加当前 Issue 分支检测。
8. PR/MR 默认 auto merge，失败 fallback manual merge。
9. Project Loop 直接生成 Delivery Audit Report，不创建 Audit Issue。
10. Project Loop 判断所有 Project Issues done 后生成 Project Final Audit Report，并进入 Project done。
11. Desktop 任务页 / 交付页 / 审计页展示新的 Project / Issue / Audit 状态。
```

### 3.2 本版本不做

```text
1. 不做完整 Multi-Agent Runtime。
2. 不做多个 Agent 并发抢任务。
3. 不做完整 Execute Observability 页面。
4. 不做 Audit Issue。
5. 不做 Audit Agent 独立 Issue Loop。
6. 不做完整 Reader V2。
7. 不做 LSP / IDE / Terminal。
8. 不默认 deploy。
9. 不把 GitHub Issue / GitLab Issue / Linear 当作 AgentFlow 需求事实源。
10. 不把 State 当事实源。
```

---

## 4. 核心架构定义

v0.2.0 的核心只保留两个 Loop 和一个 Audit Gate：

```text
Project Loop
  项目级编排、环境检查、Issue 调度、交付审计、Project 完成判断。

Issue Loop
  单个 todo Issue 的 Build Agent 执行闭环。

Project Audit Gate
  Project Loop 内部的审计环节，直接输出 Audit Report，不生成 Audit Issue。
```

架构关系：

```text
Project Loop
  ├── Project Environment Preflight
  ├── Issue Scheduling: backlog → todo
  ├── Monitor Issue Loop
  ├── Delivery Audit Gate
  └── Project Completion Gate

Issue Loop
  ├── Runtime Preflight: todo → in_progress
  ├── Test Design
  ├── Implementation
  ├── Sandbox Verify
  ├── PR/MR Create
  ├── Auto Merge / Manual Merge Fallback
  └── Done Writeback: in_review → done
```

### 4.1 Rust crate 拆分

v0.2.0 的底层实现先拆成两个新增 crate：

```text
crates/loop
  crate name: agentflow-loop
  作用：Project Loop / Issue Loop / Project Audit Gate 的统一编排层。

crates/mcp
  crate name: agentflow-mcp
  作用：GitHub / GitLab / Codex / Browser Preview 等外部 provider 的能力适配层。
```

`agentflow-loop` 允许依赖：

```text
agentflow-input
agentflow-panel
agentflow-mcp
agentflow-execute
agentflow-output
agentflow-state
agentflow-workflow-events
```

`agentflow-mcp` 不允许依赖 `agentflow-loop`，避免外部工具反向推进 AgentFlow 状态。

### 4.2 agentflow-loop 目录建议

```text
crates/loop/
  Cargo.toml
  src/lib.rs
  src/model.rs
  src/project_loop.rs
  src/issue_loop.rs
  src/audit_gate.rs
  src/storage.rs
  src/events.rs
  src/error.rs
```

`project_loop.rs`：

```text
project.preflight
project.issue.schedule
project.delivery.audit
project.final.audit
project.done
```

`issue_loop.rs`：

```text
issue.runtime_preflight
issue.branch_check
issue.status_transition
issue.review_wait
issue.done_writeback
```

`audit_gate.rs`：

```text
delivery audit report
project final audit report
audit status projection
```

### 4.3 agentflow-mcp 目录建议

```text
crates/mcp/
  Cargo.toml
  src/lib.rs
  src/model.rs
  src/registry.rs
  src/provider.rs
  src/github.rs
  src/gitlab.rs
  src/codex.rs
  src/browser.rs
  src/health.rs
  src/events.rs
  src/error.rs
```

MCP provider 只暴露能力：

```text
GitHub
  gh 可用性、认证状态、repo 权限、PR create、PR ready、PR auto merge、PR merged 查询。

GitLab
  glab 可用性、认证状态、repo 权限、MR create、MR ready、MR auto merge、MR merged 查询。

Codex
  Codex CLI 可用性、当前 agentflow CLI 能力、build-agent complete 支持检测。

Browser Preview
  smoke、DOM snapshot、console logs、截图证据能力。
```

MCP provider 禁止：

```text
1. 不生成 Issue。
2. 不拆任务。
3. 不重排 Project。
4. 不写 issue done。
5. 不把 GitHub Issue / GitLab Issue / 外部 task 当事实源。
```

### 4.4 状态文件建议

```text
.agentflow/state/mcp/registry.json
.agentflow/state/mcp/providers/github.json
.agentflow/state/mcp/providers/gitlab.json
.agentflow/state/mcp/providers/codex.json
.agentflow/state/project/preflight.json
.agentflow/state/project/scheduler.json
.agentflow/state/loops/issues/<issue-id>.json
```

---

## 5. 数据事实源调整

### 5.1 Project 是 Issue 的必需容器

v0.2.0 开始，Spec Agent 不能再生成缺少 Project 的孤立 Issue。

必须满足：

```text
每个 Issue 必须属于一个 Project。
每个 Issue 必须有 projectId。
每个 Project 必须维护自己的 issueIds。
Project Loop 只调度属于当前 Project 的 Issues。
```

### 5.2 建议数据结构

#### Project

```json
{
  "version": "input-project.v1",
  "projectId": "project-agentflow-v020-mvp",
  "sourceSpecId": "agentflow-v020-project-loop-issue-loop-mvp",
  "title": "AgentFlow v0.2.0 Project Loop / Issue Loop MVP",
  "status": "active",
  "issueIds": [
    "AF-v020-PL-001",
    "AF-v020-IL-001"
  ],
  "system": {
    "createdBy": "spec-agent",
    "path": ".agentflow/input/projects/project-agentflow-v020-mvp.json"
  }
}
```

#### Project-bound Issue

```json
{
  "version": "input-issue.v1",
  "issueId": "AF-v020-PL-001",
  "issueModel": "project",
  "projectId": "project-agentflow-v020-mvp",
  "issueCategory": "spec",
  "requiredAgentRole": "build-agent",
  "sourceSpecId": "agentflow-v020-project-loop-issue-loop-mvp",
  "displayStatus": "backlog",
  "title": "实现 Project Loop 环境检测与调度",
  "allowedPaths": [
    "crates/state/**",
    "crates/input/**"
  ],
  "forbiddenPaths": [
    ".agentflow/output/audit/**"
  ],
  "validationCommands": [
    "cargo test -p agentflow-state",
    "cargo test -p agentflow-input"
  ],
  "expectedOutputs": {
    "executeRunDir": ".agentflow/execute/runs/<run-id>",
    "evidencePath": ".agentflow/output/evidence/<run-id>.json",
    "releaseDeliveryDir": ".agentflow/output/release/<run-id>"
  }
}
```

### 5.3 MVP 路径策略

为了减少 v0.2.0 迁移成本，本版本建议保留现有 Issue canonical path：

```text
.agentflow/input/issues/<issue-id>.json
```

但必须强制：

```text
issueModel = project
projectId != null
project.issueIds 包含该 issueId
```

也就是说，v0.2.0 先做 **逻辑 Project 容器**，不强制迁移为嵌套路径。

后续版本可再考虑：

```text
.agentflow/input/projects/<project-id>/issues/<issue-id>.json
```

---

## 6. 状态模型

### 6.1 Issue Display Status

v0.2.0 采用以下状态：

```text
backlog
  任务已生成，但未进入 Project Loop 调度队列。

todo
  Project Loop 已完成项目级调度检查，任务可以被 Issue Loop 接管。

in_progress
  Issue Loop 已开始执行，Build Agent 已接管并正在执行。

in_review
  本地验证通过，正在处理 PR/MR 创建、自动合并或等待手动合并。

done
  PR/MR 已合并，Build Agent 写回完成，Evidence 和 Release Delivery 已生成。

blocked
  当前任务被阻断，需要人类或系统修复前置问题。

cancel
  任务取消。
```

状态主链路：

```text
backlog
→ todo
→ in_progress
→ in_review
→ done
```

异常：

```text
backlog / todo / in_progress / in_review → blocked
any → cancel
```

### 6.2 Project Status

```text
active
  Project 已创建，等待 Project Loop 检查或调度。

preflight_blocked
  Project 级环境检测失败。

scheduling
  Project Loop 正在调度 backlog Issues。

executing
  至少一个 Issue Loop 正在运行。

auditing
  Project Loop 正在生成 Delivery Audit / Project Audit。

done
  所有 Issues done 且所有审计通过。

blocked
  Project 级阻断。

cancel
  Project 取消。
```

---

## 7. Project Loop 设计

### 7.1 Project Loop 职责

Project Loop 负责：

```text
1. 项目级环境检测。
2. GitHub / GitLab 基础能力检测。
3. backlog Issue 调度为 todo。
4. 监听 Issue Loop 的 done 状态。
5. Issue done 后直接生成 Delivery Audit Report。
6. 所有 Issues done 后生成 Project Final Audit Report。
7. 判断 Project done。
```

Project Loop 不负责：

```text
1. 不创建 Execute Run。
2. 不写 Agent Claim。
3. 不改源码。
4. 不运行测试。
5. 不创建 PR/MR。
6. 不执行 build-agent complete。
7. 不创建 Audit Issue。
```

---

## 8. Project Environment Preflight

Project Loop 在项目启动或刷新时统一检查 GitHub / GitLab 基础状态。

### 8.1 只检测三项

按照 MVP 范围，只检测：

```text
1. gh / glab 是否安装
2. 是否已登录
3. 是否有 repo 权限
```

### 8.2 Provider 检测规则

先根据 Git remote 判断 provider：

```text
remote 包含 github.com 或 GitHub Enterprise Host → 使用 gh
remote 包含 gitlab.com 或 GitLab Host → 使用 glab
无法判断 → project preflight blocked
```

### 8.3 GitHub 检测

```bash
# 是否安装
which gh
或 gh --version

# 是否登录
gh auth status

# 是否有 repo 权限
gh repo view --json nameWithOwner,viewerPermission,defaultBranchRef
```

通过条件：

```text
gh 可执行
gh auth status 成功
viewerPermission 至少为 READ / TRIAGE / WRITE / MAINTAIN / ADMIN
```

对于需要创建 PR/MR 的流程，建议最低要求：

```text
WRITE 或更高
```

但 v0.2.0 MVP 只要求记录权限，不强制细分权限策略。

### 8.4 GitLab 检测

```bash
# 是否安装
which glab
或 glab --version

# 是否登录
glab auth status

# 是否有 repo 权限
glab repo view
```

通过条件：

```text
glab 可执行
glab auth status 成功
glab repo view 成功
```

### 8.5 输出文件

建议输出：

```text
.agentflow/state/project/preflight.json
.agentflow/state/integrations/git-provider.json
```

示例：

```json
{
  "version": "git-provider-status.v1",
  "provider": "github",
  "cli": "gh",
  "installed": true,
  "authenticated": true,
  "repoPermissionChecked": true,
  "repoPermission": "ADMIN",
  "status": "ready",
  "checkedAt": 1760000000,
  "errors": []
}
```

---

## 9. Project Issue Scheduling：backlog → todo

Project Loop 只扫描当前 Project 下 `displayStatus = backlog` 的 Issues。

### 9.1 调度检查项

每个 backlog Issue 必须满足：

```text
1. Issue 必须是 AgentFlow input issue。
2. Issue 必须有 projectId。
3. projectId 必须指向当前 Project。
4. 当前 displayStatus 必须是 backlog。
5. issueCategory 必须是 spec。
6. requiredAgentRole 必须是 build-agent。
7. 依赖任务必须已经 done。
8. 任务合同完整。
9. sourceSpecId / sourceSpecPath 完整。
10. allowedPaths / forbiddenPaths 完整。
11. validationCommands 完整。
12. expectedOutputs 完整。
13. Panel Context Pack 存在或可生成。
14. Project Environment Preflight 已 ready。
```

通过后：

```text
Issue displayStatus = todo
Project Loop 写调度记录
State 写 Issue Loop Projection
```

失败后：

```text
Issue displayStatus = blocked
State 写 blocker
保留 reason
```

### 9.2 输出文件

建议输出：

```text
.agentflow/state/project/scheduler.json
.agentflow/state/readiness/issues/<issue-id>.json
.agentflow/state/loops/issues/<issue-id>.json
```

Readiness Record 示例：

```json
{
  "version": "issue-readiness.v1",
  "projectId": "project-agentflow-v020-mvp",
  "issueId": "AF-v020-PL-001",
  "fromStatus": "backlog",
  "toStatus": "todo",
  "checkedBy": "project-loop",
  "passed": true,
  "checks": [
    { "name": "project-bound-issue", "status": "passed" },
    { "name": "dependencies-done", "status": "passed" },
    { "name": "contract-complete", "status": "passed" },
    { "name": "context-pack", "status": "passed" },
    { "name": "project-provider", "status": "passed" }
  ]
}
```

---

## 10. Issue Loop 设计

### 10.1 Issue Loop 职责

Issue Loop 只处理：

```text
displayStatus = todo
issueCategory = spec
requiredAgentRole = build-agent
```

Issue Loop 负责：

```text
1. Runtime Preflight。
2. 当前 Issue 分支检测。
3. Test Design。
4. Implementation。
5. Sandbox Verify。
6. PR/MR 创建。
7. 默认 auto merge，失败 fallback manual merge。
8. build-agent complete。
9. Issue 状态 todo → in_progress → in_review → done。
```

Issue Loop 不负责：

```text
1. 不处理 backlog Issue。
2. 不调度其他 Issue。
3. 不判断 Project done。
4. 不生成 Audit Report。
5. 不创建 Audit Issue。
6. 不检测 gh/glab 登录状态和 repo 权限。
```

---

## 11. Issue Runtime Preflight：todo → in_progress

Issue Loop 开始时执行 Runtime Preflight。

### 11.1 检查项

```text
1. 当前 Issue displayStatus = todo。
2. issueCategory = spec。
3. requiredAgentRole = build-agent。
4. Project Environment Preflight 已 ready。
5. Context Pack 可读取。
6. Agent Manual / Skill 可读取。
7. MCP 所需能力可用，若当前任务声明需要 MCP。
8. Agent Claim 可生成。
9. Execute Run 可创建。
10. Issue Lease 可获取。
11. 当前 Issue 分支检查通过。
```

通过后：

```text
Issue displayStatus = in_progress
创建 Execute Run
写 agent-claim.json
写 trace: run.created
写 trace: agent.claimed
写 trace: issue.status.changed(todo → in_progress)
```

失败后：

```text
Issue displayStatus = blocked
写 blocker
写 trace: runtime-preflight.blocked
```

---

## 12. 当前 Issue 分支检测

Issue Loop 必须检测当前 Issue 的分支。

### 12.1 目的

保证：

```text
1. 当前代码变更发生在当前 Issue 的专属分支。
2. PR/MR 能准确绑定当前 Issue。
3. 多个 Issue 不会在同一个工作分支混写。
4. Evidence / Delivery / PR/MR metadata 能追踪到正确分支。
```

### 12.2 分支命名规则

默认分支名：

```text
agentflow/<project-id>/<issue-id>
```

示例：

```text
agentflow/project-agentflow-v020-mvp/AF-v020-PL-001
```

如果 Issue 明确声明 `branchName`，则以 Issue 中声明为准。

### 12.3 检查项

```text
1. 当前仓库必须是 Git repo。
2. 当前 base branch 可识别。
3. 当前 issue branch 可识别。
4. 当前 branch 必须等于 issue.branchName 或默认 issue branch。
5. 如果 branch 不存在，可由 Issue Loop 创建。
6. 如果当前 branch 不匹配且已有未提交改动，则 blocked。
7. 如果当前 branch 不匹配但工作区干净，可以切换 / 创建 issue branch。
```

### 12.4 输出

写入：

```text
.agentflow/execute/runs/<run-id>/branch.json
```

示例：

```json
{
  "version": "issue-branch.v1",
  "projectId": "project-agentflow-v020-mvp",
  "issueId": "AF-v020-PL-001",
  "runId": "run-001",
  "baseBranch": "main",
  "issueBranch": "agentflow/project-agentflow-v020-mvp/AF-v020-PL-001",
  "currentBranchBefore": "main",
  "currentBranchAfter": "agentflow/project-agentflow-v020-mvp/AF-v020-PL-001",
  "status": "ready"
}
```

---

## 13. Issue Loop 执行阶段

### 13.1 测试设计

```text
从 SPEC 和当前 Issue 推导测试点。
能 TDD 的任务先补失败测试。
不适合 TDD 的任务必须记录原因。
明确替代验证方式。
```

Trace：

```text
test-design.started
test-design.finished
test-design.skipped-with-reason
```

Issue 状态保持：

```text
in_progress
```

---

### 13.2 Agent 执行 Issue

```text
按测试设计和 Issue 合同执行。
只能在 allowedPaths 内改代码、配置、测试或文档。
不能执行其他 Issue。
不能重排 Project 任务。
不能写 Audit Report。
```

Trace：

```text
patch.proposed
patch.applied
patch.rejected
file.changed
diff.generated
```

Issue 状态保持：

```text
in_progress
```

---

### 13.3 沙箱验证

```text
运行 validationCommands。
收集 stdout。
收集 stderr。
收集 exit code。
收集测试 / 构建 / smoke / 截图 / DOM snapshot 证据。
运行 git diff --check。
```

Trace：

```text
command.started
command.finished
command.rejected
validation.started
validation.finished
validation.failed
```

如果失败但 Build Agent 可以继续修复：

```text
保持 in_progress
```

如果失败且无法继续：

```text
blocked
```

---

### 13.4 创建 PR/MR：in_progress → in_review

沙箱验证通过后：

```text
Issue displayStatus = in_review
```

动作：

```text
1. 推送 issue branch。
2. 按 AgentFlow Build Agent PR/MR 模板创建 PR/MR。
3. PR/MR 描述包含任务、范围、验证结果、影响、回滚和 review gate。
4. 默认尝试 auto merge。
5. auto merge 不满足时 fallback manual merge。
```

Trace：

```text
review-request.created
issue.status.changed(in_progress → in_review)
auto-merge.requested
manual-merge.fallback
```

---

### 13.5 默认自动合并，fallback 手动合并

默认策略：

```text
mergeMode.default = auto-merge-if-eligible
mergeMode.fallback = manual-merge
```

GitHub：

```bash
gh pr ready
gh pr merge --auto
```

GitLab：

```bash
glab mr update --ready
glab mr merge --auto-merge
```

如果 auto merge 不满足条件：

```text
Issue 保持 in_review
run.reviewSubstate = manual-merge-waiting
等待人类合并
```

本地检测 PR/MR merged 后继续写回。

---

### 13.6 写回 Done：in_review → done

PR/MR 确认 merged 后：

```bash
agentflow build-agent complete --run-id <run-id>
```

注意：

```text
build-agent complete 必须基于已有 runId。
不允许在 complete 阶段重新 create run。
```

写回内容：

```text
1. 校验 run。
2. 校验 agent claim。
3. 校验 branch metadata。
4. 校验 merge proof。
5. 校验 validation result。
6. 写 output/evidence。
7. 写 output/release。
8. Issue displayStatus = done。
9. Project Loop 接手审计。
```

Trace：

```text
writeback.started
writeback.accepted
delivery.created
issue.status.changed(in_review → done)
```

---

## 14. Project Audit Gate

v0.2.0 不生成 Audit Issue。

Project Loop 在 Issue done 后直接生成 Audit Report。

### 14.1 Delivery Audit

每个 Issue done 后，Project Loop 审计该 Issue 的交付。

输入：

```text
Input Project
Input Issue
Source SPEC
Execute Run
Execute Trace
Changed Files
Validation Commands
Evidence
Release Delivery
Branch Metadata
PR/MR Metadata
Merge Proof
```

输出：

```text
.agentflow/output/audit/delivery-<run-id>/audit.json
.agentflow/output/audit/delivery-<run-id>/audit-report.md
.agentflow/output/audit/delivery-<run-id>/findings.json
.agentflow/output/audit/delivery-<run-id>/evidence-map.json
.agentflow/output/audit/delivery-<run-id>/traceability.json
```

检查项：

```text
1. Issue 属于当前 Project。
2. Issue 状态为 done。
3. Evidence 存在且完整。
4. Release Delivery 存在且完整。
5. changed-files 只在 allowedPaths 内。
6. validationCommands 有结果。
7. 失败命令有说明。
8. branch metadata 存在。
9. PR/MR metadata 存在。
10. merge proof 存在。
11. Build Agent 没有写 output/audit。
```

---

### 14.2 Project Final Audit

当 Project 下所有 Issues 都 done，并且所有 Delivery Audit 通过后，Project Loop 生成 Project Final Audit。

输出：

```text
.agentflow/output/audit/project-<project-id>-final/audit.json
.agentflow/output/audit/project-<project-id>-final/audit-report.md
.agentflow/output/audit/project-<project-id>-final/findings.json
.agentflow/output/audit/project-<project-id>-final/evidence-map.json
.agentflow/output/audit/project-<project-id>-final/traceability.json
```

检查项：

```text
1. Project 下所有 Issues 均 done。
2. 每个 done Issue 都有 Evidence。
3. 每个 done Issue 都有 Release Delivery。
4. 每个 done Issue 都有 Delivery Audit Report。
5. 无 active blocker。
6. 无 active run。
7. Project 状态可切换为 done。
```

---

## 15. Desktop 页面影响

### 15.1 任务页

任务页展示 Project-bound Issues。

必须显示：

```text
Project
Issue ID
Title
Status: backlog / todo / in_progress / in_review / done / blocked / cancel
requiredAgentRole
sourceSpecId
dependencies
branchName
blocker reason
```

任务页不展示 Audit Issue，因为 v0.2.0 不创建 Audit Issue。

---

### 15.2 交付页

交付页展示：

```text
Release Delivery
Evidence refs
PR/MR metadata
Branch metadata
Merge proof
Delivery Audit status
```

---

### 15.3 审计页

审计页展示 Project Loop 生成的 Audit Reports：

```text
Delivery Audit Reports
Project Final Audit Report
Findings
Evidence Map
Traceability
```

审计页不提供创建 Audit Issue 的入口。

---

### 15.4 高级页

高级页只读展示：

```text
project preflight
scheduler
readiness records
git-provider status
issue loop projections
audit reports index
```

---

## 16. 推荐新增 / 修改模块

### 16.1 crates/input

修改：

```text
1. 禁止生成 projectId 为空的 Issue。
2. 默认 issueModel = project。
3. Project 必须维护 issueIds。
4. validate_input_workspace 检查 Issue 是否属于现有 Project。
5. 兼容旧 Direct Issue：读取时可显示 warning，但新写入禁止。
```

---

### 16.2 crates/state

新增：

```text
Project Environment Preflight state
Project Scheduler state
Issue Readiness records
Issue Loop projection
Project Audit status projection
Git provider status
```

建议路径：

```text
.agentflow/state/project/preflight.json
.agentflow/state/project/scheduler.json
.agentflow/state/integrations/git-provider.json
.agentflow/state/readiness/issues/<issue-id>.json
.agentflow/state/loops/issues/<issue-id>.json
.agentflow/state/audits/index.json
```

---

### 16.3 crates/execute

修改 / 新增：

```text
1. Issue Loop 只接受 todo Issue。
2. Runtime Preflight 增加 branch check。
3. create_execute_run 只创建 run，不代表 Issue 已经开工。
4. runtime preflight 确认 Context Pack 可读取或可补生成后，Issue 才进入 in_progress。
5. PR/MR 创建后 Issue 进入 in_review。
6. build-agent complete 改为 runId finalizer。
7. complete 后 Issue 进入 done。
8. branch.json 写入 run 目录。
```

---

### 16.4 crates/output

新增：

```text
1. Delivery Audit Report 生成。
2. Project Final Audit Report 生成。
3. Audit Index 读取。
4. 不再由 Release Delivery 生成 Audit Issue。
```

---

### 16.5 apps/desktop

修改：

```text
1. 任务页支持 Project-bound Issues。
2. 状态列支持 backlog / todo / in_progress / in_review / done / blocked / cancel。
3. 交付页显示 Delivery Audit status。
4. 审计页显示直接生成的 Audit Reports。
5. 高级页只读展示 Project Loop / Issue Loop 状态。
```

---

## 17. 开发任务拆分

GitHub Issues 已发布：

```text
#82  AF-v020-ARCH-001 新增 agentflow-loop 编排 crate
#83  AF-v020-ARCH-002 新增 agentflow-mcp 外部能力适配 crate
#84  AF-v020-PL-001 Project-bound Issue Generation
#85  AF-v020-PL-002 Project Environment Preflight
#86  AF-v020-PL-003 Project Issue Scheduler
#87  AF-v020-IL-001 Issue Loop Runtime Preflight
#88  AF-v020-IL-002 Issue Branch Check
#89  AF-v020-IL-003 PR/MR Auto Merge Default + Manual Fallback
#90  AF-v020-IL-004 Run-based Build Agent Complete
#91  AF-v020-AUD-001 Delivery Audit Report
#92  AF-v020-AUD-002 Project Final Audit Report
```

### AF-v020-ARCH-001：新增 agentflow-loop 编排 crate

目标：

```text
新增 crates/loop，作为 Project Loop、Issue Loop 和 Project Audit Gate 的统一编排模块。
```

验收：

```text
1. workspace 注册 agentflow-loop crate。
2. crate 暴露 ProjectLoop、IssueLoop、ProjectAuditGate 的基础模型和入口。
3. loop 模块只调用 input / panel / mcp / execute / output / state，不替代这些模块的事实源职责。
4. 写入 .agentflow/state/loops/** 的 storage helper。
5. 单元测试覆盖 loop state model 序列化。
6. 不接 Desktop UI，不执行真实代码修改。
```

---

### AF-v020-ARCH-002：新增 agentflow-mcp 外部能力适配 crate

目标：

```text
新增 crates/mcp，统一 GitHub / GitLab / Codex / Browser Preview 等外部 provider 的能力发现、健康检查和调用封装。
```

验收：

```text
1. workspace 注册 agentflow-mcp crate。
2. 定义 McpProvider、McpCapability、McpHealth、McpProviderStatus。
3. GitHub provider 检测 gh、auth、repo view。
4. GitLab provider 检测 glab、auth、repo view。
5. Codex provider 检测当前 agentflow CLI 是否支持 build-agent complete。
6. provider 只返回能力状态和调用结果，不生成 Issue，不推进 Issue 状态。
7. 写入 .agentflow/state/mcp/** 的 storage helper。
```

---

### AF-v020-PL-001：Project-bound Issue Generation

目标：

```text
所有新生成的 input issues 必须属于 Project。
```

验收：

```text
1. 新 Issue 必须有 projectId。
2. 新 Issue 的 issueModel 必须是 project。
3. Project issueIds 必须包含该 Issue。
4. 缺 projectId 的新 Issue 写入失败。
5. 旧 Direct Issue 可读取但提示 legacy warning。
```

---

### AF-v020-PL-002：Project Environment Preflight

目标：

```text
Project 启动 / 刷新时通过 agentflow-mcp 统一检测 GitHub / GitLab provider 基础能力。
```

验收：

```text
1. GitHub repo 只检测 GitHub provider，不要求 GitLab 同时可用。
2. GitLab repo 只检测 GitLab provider，不要求 GitHub 同时可用。
3. 未安装对应 CLI 时 Project preflight 写 preflight_blocked。
4. 未登录时 Project preflight 写 preflight_blocked。
5. repo 权限检测失败时 Project preflight 写 preflight_blocked。
6. 成功时写 .agentflow/state/mcp/providers/<provider>.json 和 git-provider status ready。
7. Project preflight 状态不直接修改 Issue 状态；Issue 是否 blocked 由 Project Scheduler 针对当前 issue 决定。
```

---

### AF-v020-PL-003：Project Issue Scheduler

目标：

```text
Project Loop 负责 backlog → todo。
```

验收：

```text
1. 只扫描当前 Project 下 backlog Issues。
2. 依赖未 done 时 blocked。
3. 合同不完整时 blocked。
4. Context Pack 不可生成时 blocked。
5. Project preflight 不 ready 时 blocked。
6. 通过后 Issue 状态为 todo。
7. 不检测 PR/MR，不创建 run，不执行命令。
```

---

### AF-v020-IL-001：Issue Loop Runtime Preflight

目标：

```text
Issue Loop 只处理 todo Issue。
```

验收：

```text
1. backlog Issue 不能被 Build Agent 执行。
2. todo Issue 可进入 Runtime Preflight。
3. create_execute_run 写 run、agent-claim.json 和 branch.json，但不切换 Issue 到 in_progress。
4. 写 agent-claim.json。
5. 写 branch.json。
6. Runtime Preflight 确认 Context Pack 可读取或可补生成。
7. Runtime Preflight 确认当前工作区没有未提交的用户源码改动。
8. Runtime Preflight 全部通过后，Issue 状态切换为 in_progress。
```

---

### AF-v020-IL-002：Issue Branch Check

目标：

```text
执行当前 Issue 前检测 / 准备 issue branch。
```

验收：

```text
1. 默认分支名为 agentflow/<project-id>/<issue-id>。
2. 当前分支不匹配且工作区干净时可创建 / 切换。
3. 当前分支不匹配且工作区有未提交改动时 blocked。
4. branch.json 记录 baseBranch / issueBranch / currentBranchBefore / currentBranchAfter。
5. PR/MR metadata 引用 issueBranch。
```

---

### AF-v020-IL-003：PR/MR Auto Merge Default + Manual Fallback

目标：

```text
PR/MR 默认自动合并，不满足条件 fallback 手动合并。
```

验收：

```text
1. 沙箱验证通过后 Issue 进入 in_review。
2. 创建 PR/MR 后默认尝试 auto merge。
3. auto merge 不满足条件时进入 manual-merge-waiting substate。
4. waiting-for-merge 不作为 Issue 状态。
5. 人类合并后本地检测 merged，再继续 writeback。
```

---

### AF-v020-IL-004：Run-based Build Agent Complete

目标：

```text
build-agent complete 改为基于 runId 的 finalizer。
```

验收：

```text
1. complete 必须传 runId。
2. complete 不重新 create run。
3. complete 校验 agent claim。
4. complete 校验 branch metadata。
5. complete 校验 merge proof。
6. complete 写 evidence / release delivery。
7. complete 把 Issue 从 in_review 切换为 done。
```

---

### AF-v020-AUD-001：Delivery Audit Report

目标：

```text
Issue done 后 Project Loop 直接生成 Delivery Audit Report。
```

验收：

```text
1. 不创建 Audit Issue。
2. 每个 done Issue 生成 output/audit/delivery-<run-id>/audit-report.md。
3. 检查 Evidence / Release Delivery / changed files / validation / branch / merge proof。
4. 审计失败时 Project 进入 blocked 或 auditing_failed projection。
5. 审计通过时 Delivery Audit status = passed。
```

---

### AF-v020-AUD-002：Project Final Audit Report

目标：

```text
所有 Project Issues done 且 Delivery Audit 通过后生成 Project Final Audit。
```

验收：

```text
1. 所有 Issues done 后触发 Project Final Audit。
2. 所有 Delivery Audits 必须 passed。
3. 输出 project-final audit package。
4. Project 状态切换为 done。
5. 审计页能展示 Project Final Audit Report。
```

---

## 18. 验收标准

v0.2.0 MVP 完成后必须满足：

```text
1. Spec Agent 不能生成没有 projectId 的 Issue。
2. Project 可以包含多个 Issues。
3. Project Loop 启动 / 刷新时检测 gh/glab 安装、登录、repo 权限。
4. Project Loop 可以把满足条件的 backlog Issue 调度为 todo。
5. Build Agent 只能处理 todo Issue。
6. Issue Loop 执行后状态按 todo → in_progress → in_review → done 推进。
7. Issue Loop 会检测当前 Issue 分支。
8. PR/MR 默认 auto merge，失败 fallback manual merge。
9. waiting-for-merge 不作为 Issue 主状态。
10. build-agent complete 基于 runId，不重新创建 run。
11. Issue done 后不生成 Audit Issue。
12. Project Loop 直接生成 Delivery Audit Report。
13. 所有 Issues done 且审计通过后生成 Project Final Audit Report。
14. Project 状态变为 done。
15. 任务页不出现 Audit Issue。
16. 审计页展示 Audit Reports。
```

---

## 19. 验证命令

基础验证：

```bash
cargo fmt --check
cargo check --workspace
cargo test --workspace
npm --prefix apps/desktop run build
npm --prefix apps/desktop run preview:smoke
git diff --check
```

建议新增专项测试：

```bash
cargo test -p agentflow-input project_bound_issue
cargo test -p agentflow-state project_loop_preflight
cargo test -p agentflow-state project_issue_scheduler
cargo test -p agentflow-execute issue_loop_runtime_preflight
cargo test -p agentflow-execute issue_branch_check
cargo test -p agentflow-output delivery_audit_report
cargo test -p agentflow-output project_final_audit_report
```

---

## 20. Codex 执行指令

```text
你现在只做 AgentFlow v0.2.0 Project Loop / Issue Loop MVP。

目标：
把 AgentFlow 收敛成一个 Project 完整 MVP：Project Loop 管项目级环境检测、Issue 调度和直接审计；Issue Loop 只处理 todo Issue 从执行到 done；不创建 Audit Issue。

核心规则：
1. 所有 input issues 必须属于 Project，不能生成缺少 projectId 的孤立 Issue。
2. Project Loop 在项目启动 / 刷新时检测 gh/glab 是否安装、是否登录、是否有 repo 权限。
3. Project Loop 负责 backlog → todo。
4. Issue Loop 只处理 todo → in_progress → in_review → done。
5. Issue Loop 必须检测当前 Issue 分支。
6. PR/MR 默认 auto merge，失败 fallback manual merge。
7. waiting-for-merge 是 run substate，不是 Issue 状态。
8. build-agent complete 必须基于 runId，不允许重新创建 run。
9. Issue done 后由 Project Loop 直接生成 Delivery Audit Report。
10. 不创建 Audit Issue。
11. 所有 Project Issues done 且 Delivery Audit 通过后生成 Project Final Audit Report，并把 Project 标记 done。

范围：
- crates/input/**
- crates/state/**
- crates/execute/**
- crates/output/**
- apps/desktop/src/**
- apps/desktop/src-tauri/**
- docs/requirements/**
- tests / fixtures

禁止：
- 不做 Multi-Agent Runtime。
- 不做完整 Execute Observability 页面。
- 不做 Reader V2。
- 不创建 Audit Issue。
- 不让 Audit Agent 处理 input issue。
- 不让 Desktop 执行命令。
- 不把 GitHub Issue / GitLab Issue / Linear 当事实源。
- 不改旧 legacy flow 作为事实源。

验证：
- cargo fmt --check
- cargo check --workspace
- cargo test --workspace
- npm --prefix apps/desktop run build
- npm --prefix apps/desktop run preview:smoke
- git diff --check

输出：
- 改了哪些模块
- Project-bound Issue 生成逻辑如何实现
- Project Preflight 检测结果结构
- backlog → todo 调度如何实现
- Issue Branch Check 如何实现
- build-agent complete 如何改为 runId finalizer
- Delivery Audit Report 如何生成
- Project Final Audit 如何生成
- 验证命令结果
```

---

## 21. 最终收口

v0.2.0 的最终定义：

```text
Project Loop 管项目：
环境、GitHub/GitLab、调度、直接审计、Project done。

Issue Loop 管任务：
只消费 todo，只负责一个 Issue 从 in_progress、in_review 到 done。

Audit 不是任务：
不创建 Audit Issue，而是 Project Loop 的交付验收 Gate，直接输出 Audit Report。
```

最终用户视角：

```text
任务页：看 Project 下所有 Issues 的状态。
执行链路：Issue Loop 负责 todo → done。
交付页：看 Evidence / Release Delivery / PR/MR / Merge Proof。
审计页：看 Project Loop 直接生成的 Audit Reports。
Project 完成：所有 Issues done 且审计通过。
```
