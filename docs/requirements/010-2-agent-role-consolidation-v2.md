# 010.2 - Agent Role Consolidation V2

创建日期：2026-06-04  
执行者：Codex  
状态：待开发  
版本：revised-final-draft

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
= 受控执行流水线

output/
= evidence / audit / release / logs / backup

state/
= 健康 / 锁 / 会话 / 索引状态
```

当前 V1 顶层 Agent 角色已经从早期的：

```text
Spec Agent
Build Agent
Release Agent
Audit Agent
```

准备压缩为：

```text
Spec Agent
Build Agent
Audit Agent
```

但上一版有一个错误：

```text
Release Agent 被移除后，release 能力被说成 future reserved。
```

这不符合现在的产品方向。

正确方向是：

```text
Release Agent 不再作为独立角色存在。
Release 能力直接并入 Build Agent。
Build Agent 负责从 input issue 到 execute，再到 evidence，再到 release delivery 的完整开发闭环。
```

大白话：

> **不是取消 Release 能力，而是取消独立 Release Agent。**  
> **Build Agent = 实现 + 执行 + 验证 + 证据 + PR / 发布交付准备。**

---

## 一句话定义

> **Agent Role Consolidation V2 负责把 V1 顶层 Agent 角色压成 3 个：Spec / Build / Audit。其中 Release Agent 不再独立存在，但 Release 能力直接赋予 Build Agent。Build Agent 负责完整开发交付链路：从 input issue 启动 execute，完成 patch / command / validation，产出 evidence，并继续生成 release delivery artifacts，包括 PR draft / PR metadata / review request material / changelog / release note / delivery record。**

---

# 1. 核心原则

## 1.1 顶层角色只保留 3 个

最终角色：

```text
1. Spec Agent
2. Build Agent
3. Audit Agent
```

中文：

```text
1. 需求规格 Agent
2. 实现交付 Agent
3. 代码审计 Agent
```

---

## 1.2 Release Agent 删除，但 Release 能力不删除

需要明确：

```text
Release Agent
= 不再是 V1 顶层 Agent 角色

Release capability
= 并入 Build Agent
```

也就是说：

```text
Build Agent owns release delivery.
```

不能再写：

```text
Release 仅作为未来预留
Release 不启用
Release Agent future only
```

正确表述：

```text
Release manual is read by Build Agent.
output/release is written by Build Agent.
Release delivery is part of Build Agent's full development delivery flow.
```

---

## 1.3 Build Agent 负责完整开发交付闭环

Build Agent 的主链路：

```text
input issue
→ execute run
→ output/evidence
→ output/release
```

也可以写成：

```text
Issue
→ Implementation
→ Validation
→ Evidence
→ Delivery
```

大白话：

> **Build Agent 不只是“写代码”的 Agent，它负责把一个 issue 做到可以交付。**

---

## 1.4 Audit Agent 在 Build Agent 之后

Audit Agent 不负责实现，也不负责交付。

Audit Agent 读取：

```text
Approved SPEC
input issue
execute run
patch diff
validation result
output/evidence
output/release
```

然后输出：

```text
output/audit
```

---

# 2. 新的 Agent 角色定义

## 2.1 Spec Agent / 需求规格 Agent

状态：

```text
enabled for Input Model V1
```

职责：

```text
需求沟通
需求过滤
SPEC Draft Preview
product.md
tech.md
approval.json
Approved SPEC
direct issues
project -> issues
```

主要写：

```text
.agentflow/input/**
```

允许：

```text
读取 define/
读取 panel/
读取 input/
写 Requirement Intake Result
写 SPEC Draft Preview
在人类确认后写 Approved SPEC
在人类确认后生成 direct issues 或 project issues
```

禁止：

```text
不能写用户源码
不能执行命令
不能启动 execute run
不能写 output/evidence
不能写 output/release
不能写 output/audit
不能创建 PR
不能 merge
不能 deploy
不能审计
```

一句话：

> **Spec Agent 把需求变成施工单。**

---

## 2.2 Build Agent / 实现交付 Agent

状态：

```text
enabled for Execute + Release Delivery V1
```

Build Agent 合并原来的：

```text
Build Agent
+
Release Agent
```

职责分为两段。

---

### 第一段：Execute

负责：

```text
从 input issue 启动 execute
执行 preflight
获取 lease
生成 run plan
创建 checkpoint
应用 patch
记录 command
执行 validation
写 result
写 output/evidence
```

主要写：

```text
.agentflow/execute/**
.agentflow/output/evidence/**
```

---

### 第二段：Release Delivery

负责：

```text
基于 result / evidence 生成 PR draft
生成 PR metadata
生成 review request material
生成 changelog entry
生成 release note
生成 delivery record
把交付材料写入 output/release
```

主要写：

```text
.agentflow/output/release/**
```

Release delivery 当前不是独立 Agent 流程，而是 Build Agent 在完成 execute 和 evidence 后继续完成的交付阶段。

---

### Build Agent 可以做

```text
读取 define/
读取 panel/
读取 input issue
读取 Approved SPEC
创建 execute run
执行 preflight
获取 / 释放 lease
写 execute plan
创建 checkpoint
应用受控 patch
执行 allowed command
写 result
写 output/evidence
写 output/release
生成 PR draft
生成 PR metadata
生成 review material
生成 changelog / release note
```

---

### Build Agent 禁止做

```text
不能修改 input issue
不能修改 Approved SPEC
不能绕过 preflight
不能绕过 checkpoint
不能绕过 lease
不能写未授权路径
不能执行危险命令
不能绕过 high risk issue 的人类确认
不能 merge
不能 deploy
不能直接发布生产
不能调用模型
不能写 output/audit
```

---

### 关于 PR

Build Agent 可以准备 PR 交付材料：

```text
PR draft
PR title
PR body
changed files summary
validation evidence links
review checklist
```

是否真实创建远程 PR，需要由后续 GitHub 集成能力控制。

如果当前仓库已经有 GitHub connector / PR 创建能力，Build Agent 可以作为该能力的使用者，但必须满足：

```text
execute completed
evidence exists
release delivery record exists
high risk issue 已确认
危险命令未执行
```

如果当前版本还没有远程 PR 创建实现，本需求至少要先把 PR delivery artifact 模型落地到：

```text
.agentflow/output/release/**
```

---

## 2.3 Audit Agent / 代码审计 Agent

状态：

```text
not authorized yet
```

职责：

```text
读取 Approved SPEC
读取 input issue
读取 execute run
读取 patch diff
读取 validation result
读取 output/evidence
读取 output/release
审查是否符合 SPEC
审查是否越界
审查证据是否完整
审查 release delivery 是否完整
审查是否需要补测
审查是否有安全 / 架构 / 数据风险
```

主要写：

```text
.agentflow/output/audit/**
```

禁止：

```text
不能写用户源码
不能修改 input issue
不能修改 Approved SPEC
不能修改 execute patch
不能修改 release delivery
不能执行命令
不能创建 PR
不能 merge
不能 deploy
```

一句话：

> **Audit Agent 审查 Build Agent 交付出来的证据和交付材料。**

---

# 3. 新主流程

角色流程：

```text
Spec Agent
  ↓
Build Agent
  ↓
Audit Agent
```

目录流程：

```text
input/
  ↓
execute/
  ↓
output/evidence/
  ↓
output/release/
  ↓
output/audit/
```

大白话：

```text
Spec Agent
= 把需求变成施工单

Build Agent
= 按施工单实现、验证、留证据、准备交付

Audit Agent
= 审查证据和交付是否合格
```

---

# 4. Release 能力如何并入 Build Agent

## 4.1 RELEASE.md 仍然保留，但归 Build Agent 使用

保留：

```text
.agentflow/define/release/RELEASE.md
```

但它不是 Release Agent 的手册。

它应该改成：

```text
Build Agent's release delivery manual
```

也就是：

```text
Build Agent 在 execute 完成后，读取 RELEASE.md，生成 output/release 交付材料。
```

---

## 4.2 output/release 正式启用

保留并启用：

```text
.agentflow/output/release/
```

它不再是 future reserved。

它是：

```text
Build Agent 的 release delivery 输出区
```

可以包含：

```text
.agentflow/output/release/
├── <run-id>/
│   ├── delivery.json
│   ├── pr-draft.md
│   ├── pr-metadata.json
│   ├── review-checklist.md
│   ├── changelog.md
│   └── release-note.md
```

---

## 4.3 Build Agent 的完成定义

Build Agent 完成一个 issue，不再只看：

```text
execute result
evidence
```

而是看：

```text
execute result
evidence
release delivery artifacts
```

也就是说：

```text
Build Agent completed
= result + evidence + release delivery ready
```

---

# 5. Agentflow.md 修改要求

修改位置：

```text
crates/agent-manual/src/templates.rs
```

`Agentflow.md` 里的 Agent Roles 章节应改为：

```md
## Agent Roles

### 1. Spec Agent / 需求规格 Agent

Status: enabled for Input Model V1.

Owns requirement intake, SPEC Gate, Approved SPEC, direct issues, and project issues under `.agentflow/input/**`.

It cannot execute issues, write source code, run commands, write output evidence, write release delivery, create PRs, merge, deploy, or audit.

### 2. Build Agent / 实现交付 Agent

Status: enabled for Execute + Release Delivery V1.

Owns controlled development delivery from `.agentflow/input/issues/<issue-id>.json` into `.agentflow/execute/runs/<run-id>/`, `.agentflow/output/evidence/<run-id>.json`, and `.agentflow/output/release/<run-id>/`.

It performs preflight, lease, plan, checkpoint, patch, command record, validation, result, evidence, PR draft, PR metadata, review material, changelog, release note, and delivery record.

It cannot modify input issues, modify Approved SPEC, bypass preflight, bypass checkpoint, bypass lease, write unauthorized paths, execute dangerous commands, bypass high-risk human confirmation, merge, deploy, call models, or write audit reports.

### 3. Audit Agent / 代码审计 Agent

Status: not authorized yet.

Future role for reviewing Approved SPEC, input issue, execute run, patch diff, validation result, output evidence, and release delivery artifacts against AgentFlow boundaries.

It cannot modify source code, modify input facts, modify execute patches, modify release delivery, execute commands, create PRs, merge, or deploy.
```

---

# 6. RELEASE.md 修改要求

修改位置：

```text
crates/agent-manual/src/layout.rs
```

或当前 release manual template 所在位置。

RELEASE.md 应改为 Build Agent 的交付手册：

```md
# RELEASE.md

Release delivery is owned by Build Agent in V1.

There is no standalone Release Agent in V1.

## Purpose

RELEASE.md tells Build Agent how to prepare development delivery artifacts after execute result and evidence are available.

## Build Agent Release Delivery

After a successful execute run, Build Agent may prepare:

- PR draft
- PR metadata
- Review checklist
- Changelog entry
- Release note
- Delivery record

These artifacts are written under:

`.agentflow/output/release/<run-id>/`

## V1 Boundary

Build Agent may prepare release delivery artifacts.

Build Agent must not:

- merge
- deploy
- release to production
- run dangerous commands
- bypass high-risk confirmation
- modify Approved SPEC
- modify input issue facts
- write audit reports

## Required Inputs

- input issue
- Approved SPEC
- execute result
- output evidence
- changed-files summary
- validation result

## Required Outputs

- delivery.json
- pr-draft.md
- pr-metadata.json
- review-checklist.md
- changelog.md
- release-note.md
```

---

# 7. Execute / Output 模型调整

## 7.1 Execute result next 字段

上一版建议删除 `readyForRelease`，现在需要调整。

因为 Release 能力并入 Build Agent，所以不能完全删除 release 语义。

建议把：

```json
"next": {
  "readyForRelease": false,
  "needsAudit": true
}
```

改成：

```json
"next": {
  "readyForDelivery": true,
  "needsAudit": true
}
```

原因：

```text
delivery 比 release 更适合当前 V1
```

但 output 目录仍然使用：

```text
output/release/
```

因为其中包含 PR draft / changelog / release note 等 release delivery artifacts。

---

## 7.2 新增 Release Delivery 模型

新增：

```text
OutputReleaseDelivery
```

路径：

```text
.agentflow/output/release/<run-id>/delivery.json
```

建议字段：

```json
{
  "version": "output-release-delivery.v1",
  "runId": "run-001",
  "issueId": "iss-001",
  "sourceSpecId": "spec-001",
  "riskLevel": "medium",
  "evidencePath": ".agentflow/output/evidence/run-001.json",
  "status": "drafted",
  "artifacts": {
    "prDraft": ".agentflow/output/release/run-001/pr-draft.md",
    "prMetadata": ".agentflow/output/release/run-001/pr-metadata.json",
    "reviewChecklist": ".agentflow/output/release/run-001/review-checklist.md",
    "changelog": ".agentflow/output/release/run-001/changelog.md",
    "releaseNote": ".agentflow/output/release/run-001/release-note.md"
  },
  "createdBy": "Build Agent",
  "createdAt": 1780360000
}
```

---

## 7.3 新增 Build Agent delivery API

在 execute / output 相关 crate 中新增：

```text
prepare_release_delivery
load_release_delivery
```

或者如果还没有 output crate，可以暂时放在：

```text
crates/execute/src/delivery.rs
```

因为当前 delivery 是 execute 完成后的 Build Agent 阶段。

建议：

```text
crates/execute/src/delivery.rs
```

新增：

```rust
prepare_release_delivery(project_root, run_id) -> OutputReleaseDelivery
```

要求：

```text
run completed
result exists
evidence exists
changed files summary exists
validation passed or failed result recorded
```

输出：

```text
.agentflow/output/release/<run-id>/delivery.json
.agentflow/output/release/<run-id>/pr-draft.md
.agentflow/output/release/<run-id>/pr-metadata.json
.agentflow/output/release/<run-id>/review-checklist.md
.agentflow/output/release/<run-id>/changelog.md
.agentflow/output/release/<run-id>/release-note.md
```

---

# 8. output/ 语义更新

当前 output 语义应更新为：

```text
output/evidence/
= Build Agent 执行证据

output/release/
= Build Agent 交付材料

output/audit/
= Audit Agent 审计报告
```

不要再表达成：

```text
output/release 是未来预留
```

正确表达：

```text
output/release 是 Build Agent 的交付输出。
```

---

# 9. 需要修改的文件

建议修改：

```text
crates/agent-manual/src/templates.rs
crates/agent-manual/src/layout.rs

crates/execute/src/model.rs
crates/execute/src/result.rs
crates/execute/src/evidence.rs
crates/execute/src/delivery.rs
crates/execute/src/lib.rs

apps/desktop/src/types/status.ts
apps/desktop/src/browserPreviewData.ts
apps/desktop/src/features/status-channel/statusAdapters.ts

docs/requirements/010-execute-patch-checkpoint-v1.md
docs/requirements/010-2-agent-role-consolidation-v1.md
docs/requirements/README.md
docs/requirements/next-requirements.md

README.md
GOAL.md
ROADMAP.md
verification.md
```

如有以下文案，也要同步：

```text
Release Agent
发布交付 Agent
Release future reserved
output/release reserved
readyForRelease
ready_for_release
```

---

# 10. 不允许做

本需求只做角色收敛和 Build Agent release delivery 能力归并。

不允许做：

```text
不恢复独立 Release Agent
不改 input 模型
不改 panel 模型
不改 riskLevel 规则
不绕过 execute preflight
不绕过 evidence
不 merge
不 deploy
不直接发布生产
不调用模型
不启动 Audit Agent
```

---

# 11. 测试要求

需要新增 / 更新测试：

```text
1. Agentflow.md template only lists Spec / Build / Audit roles.
2. Agentflow.md template does not list standalone Release Agent.
3. Build Agent section includes release delivery responsibility.
4. RELEASE.md says release delivery is owned by Build Agent.
5. RELEASE.md does not say release is future reserved.
6. output/release is described as Build Agent output.
7. ExecuteResultNext uses readyForDelivery, not readyForRelease.
8. prepare_release_delivery requires completed run.
9. prepare_release_delivery requires evidence.
10. prepare_release_delivery writes delivery.json / pr-draft.md / pr-metadata.json / review-checklist.md / changelog.md / release-note.md.
11. Existing execute tests still pass.
```

---

# 12. 验收标准

```text
- [ ] 新增 / 更新 docs/requirements/010-2-agent-role-consolidation-v1.md。
- [ ] 顶层 Agent 角色只剩 Spec / Build / Audit。
- [ ] Agentflow.md 不再把 Release Agent 作为顶层角色。
- [ ] Spec Agent 职责仍覆盖 input。
- [ ] Build Agent 职责覆盖 execute + output/evidence + output/release。
- [ ] Audit Agent 保持 not authorized yet。
- [ ] RELEASE.md 明确 release delivery is owned by Build Agent。
- [ ] RELEASE.md 明确 V1 没有 standalone Release Agent。
- [ ] RELEASE.md 不再说 release future reserved / not active。
- [ ] output/release/ 正式作为 Build Agent 交付输出。
- [ ] output/evidence/ 明确属于 Build Agent 执行证据。
- [ ] output/audit/ 明确属于 Audit Agent 审计输出。
- [ ] ExecuteResultNext 不再暴露 readyForRelease。
- [ ] ExecuteResultNext 改为 readyForDelivery + needsAudit，或等价 delivery 语义。
- [ ] 新增 OutputReleaseDelivery 模型。
- [ ] 新增 prepare_release_delivery API。
- [ ] prepare_release_delivery 写 output/release/<run-id>/delivery.json。
- [ ] prepare_release_delivery 写 pr-draft.md。
- [ ] prepare_release_delivery 写 pr-metadata.json。
- [ ] prepare_release_delivery 写 review-checklist.md。
- [ ] prepare_release_delivery 写 changelog.md。
- [ ] prepare_release_delivery 写 release-note.md。
- [ ] 不改 input 模型。
- [ ] 不恢复独立 Release Agent。
- [ ] 不 merge。
- [ ] 不 deploy。
- [ ] 不直接发布生产。
- [ ] 不调用模型。
- [ ] cargo fmt --check 通过。
- [ ] cargo test -p agentflow-agent-manual 通过。
- [ ] cargo test -p agentflow-execute 通过。
- [ ] cargo test -p agentflow-desktop 通过。
- [ ] cargo test 通过。
- [ ] npm --prefix apps/desktop run build 通过。
- [ ] git diff --check 通过。
```

---

# 13. 验证命令

```bash
cargo fmt --check
cargo test -p agentflow-agent-manual
cargo test -p agentflow-execute
cargo test -p agentflow-desktop
cargo test
npm --prefix apps/desktop run build
git diff --check
```

可选检查：

```bash
rg -n "Release Agent|发布交付 Agent|future reserved|not active|readyForRelease|ready_for_release" crates apps docs README.md GOAL.md ROADMAP.md verification.md
```

允许例外：

```text
历史 verification 记录
本需求文档中说明“Release Agent 不再独立存在”的段落
```

---

# 14. PR 说明要求

PR 描述必须说明：

```text
1. 为什么 Release Agent 从 V1 顶层角色中移除。
2. 为什么 Release 能力并入 Build Agent，而不是变成 future reserved。
3. Build Agent 如何负责 execute + evidence + release delivery。
4. RELEASE.md 如何变成 Build Agent 的 release delivery manual。
5. output/evidence / output/release / output/audit 的新语义。
6. ExecuteResultNext 如何从 readyForRelease 改为 readyForDelivery。
7. OutputReleaseDelivery 模型新增了什么。
8. 本次没有恢复独立 Release Agent。
9. 本次没有 merge / deploy / 生产发布。
10. 本次没有调用模型。
11. 验证命令和结果。
```

---

# 15. Codex 执行指令

```md
请执行 010.2 - Agent Role Consolidation V2。

目标：
把 V1 顶层 Agent 角色从 Spec / Build / Release / Audit 收敛为 Spec / Build / Audit。但注意：Release 能力不是 future reserved，而是直接并入 Build Agent。Build Agent 负责完整开发交付闭环：input issue -> execute run -> output/evidence -> output/release。Release Agent 不再作为独立角色存在，RELEASE.md 成为 Build Agent 的 release delivery manual，output/release 成为 Build Agent 的交付输出区。

必须遵守：
1. 顶层 Agent 角色只保留 Spec Agent / Build Agent / Audit Agent。
2. Release Agent 不再作为 V1 顶层角色。
3. Release 能力必须并入 Build Agent。
4. Build Agent 负责 execute + output/evidence + output/release。
5. Audit Agent 仍然 not authorized yet。
6. RELEASE.md 保留，并改为 Build Agent release delivery manual。
7. output/release 正式启用为 Build Agent delivery output。
8. execute result 不再包含 readyForRelease。
9. execute result 使用 readyForDelivery + needsAudit，或等价 delivery 语义。
10. 新增 OutputReleaseDelivery 模型。
11. 新增 prepare_release_delivery API。
12. 不改 input 模型。
13. 不恢复独立 Release Agent。
14. 不 merge。
15. 不 deploy。
16. 不直接发布生产。
17. 不调用模型。

实现范围：
- 新增 / 更新 docs/requirements/010-2-agent-role-consolidation-v1.md。
- 更新 Agentflow.md template 的 Agent Roles 章节。
- 更新 RELEASE.md template，改为 Build Agent release delivery manual。
- 更新 ExecuteResultNext，移除 readyForRelease，加入 readyForDelivery。
- 新增 OutputReleaseDelivery 模型。
- 新增 prepare_release_delivery API。
- 输出 output/release/<run-id>/delivery.json。
- 输出 output/release/<run-id>/pr-draft.md。
- 输出 output/release/<run-id>/pr-metadata.json。
- 输出 output/release/<run-id>/review-checklist.md。
- 输出 output/release/<run-id>/changelog.md。
- 输出 output/release/<run-id>/release-note.md。
- 更新 TypeScript 类型 / Browser Preview mock / status channel 如有相关字段。
- 更新 README / GOAL / ROADMAP / requirements / verification。
- 增加测试，防止 Release Agent 回到顶层角色。
- 增加测试，防止 release 被标记为 future reserved。
- 增加测试，覆盖 prepare_release_delivery。

验证命令：
- cargo fmt --check
- cargo test -p agentflow-agent-manual
- cargo test -p agentflow-execute
- cargo test -p agentflow-desktop
- cargo test
- npm --prefix apps/desktop run build
- git diff --check
```

---

# 16. 完成定义

本需求完成后，AgentFlow V1 的 Agent 角色为：

```text
1. Spec Agent
2. Build Agent
3. Audit Agent
```

主链路为：

```text
Spec Agent
→ input/

Build Agent
→ execute/
→ output/evidence/
→ output/release/

Audit Agent
→ output/audit/
```

Release 语义为：

```text
Release Agent
= V1 不存在

Release capability
= Build Agent 的交付能力

define/release/RELEASE.md
= Build Agent release delivery manual

output/release/
= Build Agent delivery artifacts
```

最终一句话：

> **V1 不需要独立 Release Agent，但 Release 能力必须归入 Build Agent。Build Agent 负责完整开发交付闭环：实现、验证、证据、PR 草案、评审材料、changelog、release note 和 delivery record。**
