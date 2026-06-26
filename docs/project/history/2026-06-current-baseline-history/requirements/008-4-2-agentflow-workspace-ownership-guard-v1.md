# 008.4.2 - AgentFlow Workspace Ownership Guard V1

创建日期：2026-06-04  
执行者：Codex  
状态：已实现  
版本：final-draft

---

## 用户目标

当前 AgentFlow 在打开 / 添加本地项目时，会自动准备：

```text
AGENTS.md
.agentflow/
.agentflow/workspace-manifest.json
.agentflow/define/**
.agentflow/panel/**
.agentflow/spec/**
.agentflow/goal-tree/**
.agentflow/execute/**
.agentflow/output/**
.agentflow/state/**
```

但现在缺少一个关键安全判断：

```text
如果项目里已经存在 .agentflow/，它不一定是 AgentFlow 生成的。
```

这个 `.agentflow/` 可能来自：

```text
1. AgentFlow 当前版本
2. AgentFlow 旧版本
3. 其他工具
4. 用户自己创建
5. 损坏的 AgentFlow 目录
6. symlink 或不可写目录
```

所以不能看到 `.agentflow/` 就直接写。

本需求目标是：

> **在任何 Project Workspace prepare / Agent Manual prepare / Panel prepare 写入 `.agentflow/` 之前，先检查 `.agentflow/` 的归属权。AgentFlow 只能自动修复自己管理的 `.agentflow/`。如果 `.agentflow/` 已存在但无法确认归属，必须停止写入并 blocked，除非用户明确选择备份并接管。**

---

## 一句话定义

> **AgentFlow Workspace Ownership Guard 是 `.agentflow/` 写入前的归属权检查。它负责判断 `.agentflow/` 是当前 AgentFlow 管理、旧版 AgentFlow 管理、外部未知目录、损坏目录，还是不可安全处理目录，并决定是否允许 prepare / repair。**

---

# 1. 背景

当前 AgentFlow 的 Project Workspace prepare 已经开始承担很多职责：

```text
创建 / 修复 .agentflow/
创建 / 修复 AGENTS.md
创建 / 修复 Agent 工作手册
创建 / 修复 skills-lock
创建 / 修复 workflow directory layout
创建 / 修复 Panel
```

这些动作都会写入：

```text
.agentflow/**
```

如果项目中已经有一个不是 AgentFlow 管理的 `.agentflow/`，直接写入会产生风险：

```text
覆盖其他工具数据
污染用户已有目录
误判当前项目为 AgentFlow managed
让 Agent 在错误规则下工作
破坏用户本地状态
```

因此 `.agentflow/` 需要一个前置归属检查。

---

# 2. 核心原则

## 2.1 没有确认归属前，不能写 `.agentflow/`

规则：

```text
看到 .agentflow/ 存在时，不能直接创建子目录。
不能直接写 workspace-manifest.json。
不能直接写 Agentflow.md。
不能直接写 Panel。
不能直接 repair。
```

必须先执行：

```text
check_agentflow_workspace_ownership(projectRoot)
```

---

## 2.2 能确定是自己的，才自动 repair

允许自动 repair 的情况：

```text
没有 .agentflow/
AgentFlow 当前版本 managed
AgentFlow 旧版本 managed
AgentFlow managed 但部分文件缺失
AgentFlow managed 但 hash mismatch
AgentFlow managed 但 layout 缺失
```

不允许自动 repair 的情况：

```text
.agentflow/ 存在但没有任何 AgentFlow marker
.agentflow/ 是项目外 symlink
.agentflow/ 不可写
.agentflow/ 备份失败
.agentflow/ manifest 明显属于其他工具
```

---

## 2.3 foreign `.agentflow/` 必须 blocked

如果 `.agentflow/` 存在，但无法确认是 AgentFlow 生成：

```text
status = foreign
ready = false
agentBlocked = true
```

行为：

```text
不写入
不修复
不创建子目录
不覆盖文件
项目仍可只读打开
Agent 能力 blocked
Panel prepare blocked
Spec / Goal Tree / Execute blocked
```

---

## 2.4 用户确认后才能 takeover

如果用户明确选择接管：

```text
Backup and Take Over
```

才允许：

```text
1. 把原 .agentflow/ 重命名为 .agentflow.unmanaged.<timestamp>.bak/
2. 创建新的 .agentflow/
3. 写入 AgentFlow workspace-manifest.json
4. 正常 prepare / repair
```

不能静默接管。

---

# 3. Ownership 状态

新增 `.agentflow/` ownership 状态：

```text
none
managed-current
managed-legacy
foreign
corrupted
blocked
```

---

## 3.1 none

含义：

```text
项目中没有 .agentflow/
```

处理：

```text
创建 .agentflow/
写 workspace-manifest.json
正常 prepare
```

状态：

```text
ready / repaired
```

---

## 3.2 managed-current

含义：

```text
.agentflow/ 是当前 AgentFlow 版本管理的
```

判断依据：

```text
.agentflow/workspace-manifest.json 存在
managedBy = AgentFlow
version = agentflow-workspace-manifest.v1
layoutVersion = agentflow-layout.v1
```

处理：

```text
validate
repair missing files
repair hash mismatch
repair layout
repair AGENTS.md
repair skills-lock
repair Panel layout
```

状态：

```text
ready / repaired / degraded
```

---

## 3.3 managed-legacy

含义：

```text
.agentflow/ 看起来是 AgentFlow 旧版本生成的
```

可能依据：

```text
.agentflow/define/agent/Agentflow.md 存在
.agentflow/define/agent/skills-lock.json 存在
AGENT.MD 或 AGENTS.md 包含 AGENTFLOW:MANAGED marker
旧 workspace-manifest.json 存在
旧 layoutVersion
旧路径存在：.agentflow/define/goals
旧路径存在：.agentflow/output/graph
旧路径存在：.agentflow/graph
```

处理：

```text
1. 备份关键状态
2. 写 migration record
3. 创建新 layout skeleton
4. 不删除旧数据
5. 不强迁移 Goal Tree / Graph / Panel 事实
```

状态：

```text
repaired / degraded
```

---

## 3.4 foreign

含义：

```text
.agentflow/ 存在，但没有任何 AgentFlow marker
```

判断：

```text
没有 workspace-manifest.json
没有 define/agent/Agentflow.md
没有 skills-lock.json
没有 AGENTFLOW:MANAGED marker
没有 managedBy = AgentFlow
```

处理：

```text
不写入
不 repair
不创建任何新文件
status = foreign
agentBlocked = true
项目只读打开
```

---

## 3.5 corrupted

含义：

```text
看起来像 AgentFlow，但关键文件损坏
```

例如：

```text
workspace-manifest.json 存在但 JSON 解析失败
managedBy = AgentFlow 但 layoutVersion 缺失
skills-lock.json 损坏
Agentflow.md marker 存在但内容严重不完整
```

处理规则：

```text
如果能确认 managedBy = AgentFlow:
  备份损坏文件
  repair

如果无法确认:
  blocked
```

---

## 3.6 blocked

含义：

```text
无法安全处理
```

例如：

```text
.agentflow/ 是 symlink 且指向项目外
.agentflow/ 不可写
备份失败
权限不足
磁盘写入失败
```

处理：

```text
不写入
Agent blocked
UI 显示错误
```

---

# 4. Ownership Marker

## 4.1 workspace-manifest.json 必须增加 managedBy

当前 workspace manifest 需要新增：

```json
{
  "managedBy": "AgentFlow"
}
```

完整建议：

```json
{
  "version": "agentflow-workspace-manifest.v1",
  "managedBy": "AgentFlow",
  "layoutVersion": "agentflow-layout.v1",
  "projectRoot": "/path/to/project",
  "ownership": {
    "status": "managed-current",
    "createdBy": "AgentFlow",
    "createdAt": 1780360000,
    "lastValidatedAt": 1780360000
  }
}
```

---

## 4.2 迁移后的 manifest

如果是旧版迁移：

```json
{
  "version": "agentflow-workspace-manifest.v1",
  "managedBy": "AgentFlow",
  "layoutVersion": "agentflow-layout.v1",
  "projectRoot": "/path/to/project",
  "ownership": {
    "status": "managed-current",
    "migratedFrom": "agentflow-layout.v0",
    "migrationRecord": ".agentflow/state/migrations/20260604-layout-v0-to-v1.json"
  }
}
```

---

## 4.3 AgentFlow managed marker

可用于辅助判断：

```text
<!-- AGENTFLOW:MANAGED version=... -->
```

可能出现在：

```text
AGENTS.md
AGENT.MD
.agentflow/define/agent/Agentflow.md
```

但 marker 不能单独决定 current version，只能辅助判断：

```text
managed-legacy
corrupted managed
```

---

# 5. Prepare 流程调整

当前 Project Workspace prepare 必须改成：

```text
resolve project root
  ↓
check duplicate project
  ↓
check .agentflow ownership
  ↓
if none:
    create .agentflow and prepare
  ↓
if managed-current:
    validate / repair
  ↓
if managed-legacy:
    backup / migration record / repair
  ↓
if foreign:
    stop prepare, mark blocked
  ↓
if corrupted:
    repair only if managed marker is trusted
  ↓
if blocked:
    stop prepare
```

关键：

```text
ownership check 必须发生在所有 .agentflow 写入之前。
```

---

# 6. Foreign `.agentflow/` 处理

## 6.1 默认不接管

如果发现 foreign `.agentflow/`：

```text
不写入
不创建 AGENTS.md
不创建 define/
不创建 panel/
不创建 workspace-manifest.json
不创建 backup 到 .agentflow/output/
```

原因：

```text
既然 .agentflow/ 不是我们的，就不能假设能往里面写 backup。
```

---

## 6.2 UI / Status 提示

状态文案：

```text
检测到已有 .agentflow/ 目录，但无法确认它由 AgentFlow 管理。
为避免覆盖其他工具或用户数据，AgentFlow 不会自动接管该目录。
你可以选择备份并接管，或手动清理后重新打开项目。
```

---

## 6.3 项目仍可只读打开

foreign 状态下允许：

```text
Project File Reader 只读打开
基础文件浏览
```

禁止：

```text
Agent Manual prepare
Panel prepare
Spec
Goal Tree
Execute
Release
Audit
```

---

# 7. 用户确认接管流程

本需求可以先实现后端能力，不一定做 UI 按钮。

建议 API：

```text
take_over_agentflow_workspace(projectRoot): WorkspaceOwnershipStatus
```

行为：

```text
1. 再次确认 .agentflow/ 是 foreign。
2. 将原 .agentflow/ 重命名为 .agentflow.unmanaged.<timestamp>.bak/
3. 创建新的 .agentflow/
4. 写入 workspace-manifest.json
5. 正常 prepare
```

备份路径：

```text
<project-root>/.agentflow.unmanaged.<timestamp>.bak/
```

不要备份到：

```text
.agentflow/output/backup/
```

原因：

```text
原 .agentflow/ 还不是我们的，不能假设里面可以写。
```

如果 rename 失败：

```text
status = blocked
不继续 prepare
```

---

# 8. Symlink 规则

## 8.1 `.agentflow/` 是普通目录

正常检查 ownership。

---

## 8.2 `.agentflow/` 是 symlink，指向项目内

允许继续，但记录 warning：

```text
.agentflow is a symlink inside project root.
```

继续 ownership check。

---

## 8.3 `.agentflow/` 是 symlink，指向项目外

必须 blocked：

```text
.agentflow is a symlink outside project root.
```

不写入，不 repair。

---

# 9. 数据模型

新增：

```rust
WorkspaceOwnershipStatus
```

建议字段：

```rust
pub struct WorkspaceOwnershipStatus {
    pub version: String,
    pub project_root: String,
    pub status: WorkspaceOwnershipState,
    pub ready_for_prepare: bool,
    pub agent_blocked: bool,
    pub agentflow_path: String,
    pub marker: WorkspaceOwnershipMarker,
    pub detected_files: Vec<String>,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub recommended_action: WorkspaceOwnershipAction,
}
```

状态：

```rust
pub enum WorkspaceOwnershipState {
    None,
    ManagedCurrent,
    ManagedLegacy,
    Foreign,
    Corrupted,
    Blocked,
}
```

推荐动作：

```rust
pub enum WorkspaceOwnershipAction {
    Create,
    ValidateRepair,
    MigrateRepair,
    AskUserToTakeOver,
    Stop,
}
```

Marker：

```rust
pub struct WorkspaceOwnershipMarker {
    pub manifest_exists: bool,
    pub manifest_managed_by_agentflow: bool,
    pub manifest_version: Option<String>,
    pub layout_version: Option<String>,
    pub agent_manual_exists: bool,
    pub skills_lock_exists: bool,
    pub managed_entry_exists: bool,
}
```

---

# 10. Rust 实现建议

建议新增模块：

```text
crates/agent-manual/src/ownership.rs
```

原因：

```text
.agentflow ownership 与 Agent Manual / workspace manifest 强相关
Project Workspace prepare 可以直接调用
未来 CLI / Tauri 也可复用
```

新增 API：

```rust
pub fn check_agentflow_workspace_ownership(
    project_root: impl AsRef<Path>
) -> Result<WorkspaceOwnershipStatus>;

pub fn assert_agentflow_workspace_owned_or_creatable(
    project_root: impl AsRef<Path>
) -> Result<WorkspaceOwnershipStatus>;

pub fn take_over_agentflow_workspace(
    project_root: impl AsRef<Path>
) -> Result<WorkspaceOwnershipStatus>;
```

---

# 11. Project Workspace prepare 接入

在：

```text
prepare_local_project_workspace_at
```

中最前面接入：

```rust
let ownership = check_agentflow_workspace_ownership(&root)?;

match ownership.status {
    None | ManagedCurrent | ManagedLegacy | Corrupted if ownership.ready_for_prepare => {
        continue_prepare()
    }
    Foreign | Blocked => {
        return summary_with_agent_blocked(ownership)
    }
}
```

注意：

```text
不要在 ownership check 前创建 .agentflow/
```

如果当前代码已经先创建 `.agentflow/`，必须调整顺序。

---

# 12. Agent Manual / Panel prepare 不能绕过 ownership

所有会写 `.agentflow/` 的入口都必须遵守 ownership guard。

必须覆盖：

```text
Project Workspace prepare
Agent Manual prepare
Panel prepare
repair_agent_working_manual
prepare_project_panel
```

规则：

```text
如果 ownership status 是 foreign / blocked：
  不允许继续写入
```

---

# 13. Tauri commands

新增：

```text
load_agentflow_workspace_ownership
take_over_agentflow_workspace
```

---

## 13.1 load_agentflow_workspace_ownership

```ts
load_agentflow_workspace_ownership(projectRoot: string): WorkspaceOwnershipStatus
```

只读检查，不写。

---

## 13.2 take_over_agentflow_workspace

```ts
take_over_agentflow_workspace(projectRoot: string): WorkspaceOwnershipStatus
```

需要用户明确触发。

V1 可以不在 UI 暴露按钮，但 command 可以先准备好；如果不暴露 command，则至少 backend API 要有。

---

# 14. Desktop UI

状态通道新增或合并展示：

```text
Workspace Ownership
```

状态：

```text
Managed
Created
Migrated
Foreign - Blocked
Corrupted
Blocked
```

如果 foreign：

```text
Agent Manual: Blocked
Panel: Blocked
Spec: Blocked
```

但 Project File Reader 仍可以只读打开。

---

# 15. 写入边界

## 15.1 ownership check 允许读取

```text
.agentflow/**
AGENTS.md
AGENT.MD
```

---

## 15.2 foreign 状态禁止写入

如果 status = foreign：

```text
不写任何 .agentflow/**
不写 AGENTS.md
不写 AGENT.MD
不写 backup 到 .agentflow/output/**
```

---

## 15.3 takeover 允许写入

只有用户明确 takeover 时允许：

```text
rename .agentflow -> .agentflow.unmanaged.<timestamp>.bak
create new .agentflow/
write AgentFlow layout
```

---

# 16. 测试要求

必须新增测试：

```text
1. no .agentflow -> ownership none -> prepare creates.
2. current managed manifest -> managed-current -> repair allowed.
3. legacy AgentFlow markers -> managed-legacy -> repair allowed.
4. foreign .agentflow without marker -> foreign -> prepare blocked and writes nothing.
5. corrupted manifest with managedBy AgentFlow -> corrupted but repair allowed.
6. corrupted manifest without managedBy -> blocked.
7. .agentflow symlink outside project -> blocked.
8. .agentflow symlink inside project -> warning but continue.
9. takeover foreign .agentflow renames old dir and creates new managed .agentflow.
```

特别检查：

```text
foreign .agentflow 下不能新增任何文件。
```

---

# 17. 验收标准

```text
- [ ] 新增 docs/requirements/008-4-2-agentflow-workspace-ownership-guard-v1.md。
- [ ] workspace-manifest.json 增加 managedBy = AgentFlow。
- [ ] workspace-manifest.json 增加 ownership 字段。
- [ ] 新增 WorkspaceOwnershipStatus 模型。
- [ ] 新增 WorkspaceOwnershipState 枚举。
- [ ] 新增 WorkspaceOwnershipMarker 模型。
- [ ] prepare 前先检查 .agentflow ownership。
- [ ] 没有 .agentflow 时正常创建。
- [ ] managed-current 时允许 validate / repair。
- [ ] managed-legacy 时允许 migration repair。
- [ ] foreign .agentflow 不自动写入。
- [ ] foreign .agentflow 使 Agent prepare blocked。
- [ ] foreign .agentflow 项目仍可只读打开。
- [ ] corrupted managed manifest 可备份后 repair。
- [ ] corrupted unknown manifest blocked。
- [ ] .agentflow symlink 指向项目外时 blocked。
- [ ] .agentflow symlink 指向项目内时 warning。
- [ ] 用户确认 takeover 时，原目录重命名为 .agentflow.unmanaged.<timestamp>.bak。
- [ ] takeover 后创建新的 managed .agentflow。
- [ ] Tauri 或 backend API 可读取 ownership status。
- [ ] 不在 ownership check 前创建 .agentflow。
- [ ] Agent Manual prepare 不得绕过 ownership guard。
- [ ] Panel prepare 不得绕过 ownership guard。
- [ ] foreign .agentflow 下不能新增任何文件。
- [ ] cargo fmt --check 通过。
- [ ] cargo test -p agentflow-agent-manual 通过。
- [ ] cargo test -p agentflow-desktop 通过。
- [ ] cargo test 通过。
- [ ] npm --prefix apps/desktop run build 通过。
- [ ] git diff --check 通过。
```

---

# 18. 验证命令

```bash
cargo fmt --check
cargo test -p agentflow-agent-manual
cargo test -p agentflow-desktop
cargo test
npm --prefix apps/desktop run build
git diff --check
```

---

# 19. PR 说明要求

PR 描述必须说明：

```text
1. 为什么需要 ownership guard。
2. 如何判断 .agentflow 是 managed-current / managed-legacy / foreign / corrupted / blocked。
3. foreign .agentflow 为什么不自动写入。
4. foreign 项目是否仍可只读打开。
5. takeover 如何备份旧 .agentflow。
6. workspace-manifest 新增了哪些 ownership 字段。
7. symlink 如何处理。
8. Agent Manual / Panel prepare 如何避免绕过 ownership guard。
9. 本次没有写 SPEC / Goal Tree / AgentRun。
10. 本次没有写用户源码。
11. 验证命令和结果。
```

---

# 20. Codex 执行指令

```md
请执行 008.4.2 - AgentFlow Workspace Ownership Guard V1。

目标：
在任何 Project Workspace prepare / Agent Manual prepare / Panel prepare 写入 `.agentflow/` 之前，先检查 `.agentflow/` 的归属权。AgentFlow 只能自动修复自己管理的 `.agentflow/`。如果 `.agentflow/` 已存在但无法确认归属，必须停止写入并 blocked，除非用户明确选择备份并接管。

必须遵守：
1. ownership check 必须发生在任何 .agentflow 写入之前。
2. 没有 .agentflow 时可以创建。
3. managed-current 可以 validate / repair。
4. managed-legacy 可以备份 / migration record / repair。
5. foreign .agentflow 不允许自动写入。
6. foreign .agentflow 必须 blocked。
7. foreign 项目仍可只读打开。
8. takeover 必须显式触发。
9. takeover 必须把旧 .agentflow 重命名为 .agentflow.unmanaged.<timestamp>.bak。
10. .agentflow symlink 指向项目外必须 blocked。
11. .agentflow symlink 指向项目内可以继续，但必须 warning。
12. workspace-manifest.json 必须增加 managedBy = AgentFlow。
13. workspace-manifest.json 必须增加 ownership 字段。
14. Agent Manual prepare 不得绕过 ownership guard。
15. Panel prepare 不得绕过 ownership guard。
16. 不写 SPEC。
17. 不写 Goal Tree。
18. 不启动 AgentRun。
19. 不写用户源码。
20. 不执行项目命令。
21. 不调用模型。

实现范围：
- 新增 docs/requirements/008-4-2-agentflow-workspace-ownership-guard-v1.md。
- 新增 WorkspaceOwnershipStatus / WorkspaceOwnershipState / WorkspaceOwnershipMarker 模型。
- 新增 ownership check API。
- 新增 takeover API 或 backend function。
- Project Workspace prepare 前置 ownership check。
- Agent Manual prepare 不得绕过 ownership check。
- Panel prepare 不得绕过 ownership check。
- workspace-manifest 增加 managedBy 和 ownership。
- Desktop status 展示 ownership。
- 增加测试覆盖 none / managed-current / managed-legacy / foreign / corrupted / symlink / takeover。
- 更新 verification。

验证命令：
- cargo fmt --check
- cargo test -p agentflow-agent-manual
- cargo test -p agentflow-desktop
- cargo test
- npm --prefix apps/desktop run build
- git diff --check
```

---

# 21. 完成定义

本需求完成后，AgentFlow 的 `.agentflow/` 处理规则是：

```text
没有 .agentflow/
  -> 创建

是 AgentFlow 当前版本
  -> repair

是 AgentFlow 旧版本
  -> migration repair

不是 AgentFlow 的
  -> blocked，不写

用户确认 takeover
  -> 备份旧目录，再创建新的 managed .agentflow
```

最终一句话：

> **AgentFlow 只能自动修自己的 `.agentflow/`；看到归属不明的 `.agentflow/`，必须先停，不能静默接管。**
