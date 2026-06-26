# 008.4.1 - Project Panel Finalization and Graph Removal V1

创建日期：2026-06-04  
执行者：Codex  
状态：已实现  
版本：final-draft

---

## 用户目标

PR #19 已完成 `008.4 - Project Panel V1` 的主体：

```text
Graph -> Panel 的主命名迁移
.agentflow/panel/** canonical path
Panel Rust API / Tauri command / Desktop 文案
旧 Graph 路径兼容
Panel 基础产物输出
```

但现在还需要做一轮最终收口。

本需求目标是：

```text
1. 把 Agent 顶层角色从 5 个收敛为 4 个。
2. 去掉 Graph 兼容层，彻底清理 Graph 命名和旧 Graph 路径。
3. 把 Panel V1 从“基本完成”补到“目标完成”，补齐工作现场数据完整度。
```

大白话：

> PR #19 已经把 Panel 的主体搭起来了。  
> 但现在不能继续保留“Graph 兼容”“基本完成”“后续再补”的状态。  
> 这一轮要把 Panel 收到一个干净的最终形态：角色清楚、命名统一、路径唯一、数据完整、没有 Graph 残留。

---

## 一句话定义

> **008.4.1 Project Panel Finalization and Graph Removal V1 是 Project Panel 的最终收口需求：合并 Agent 角色为 Spec / Build / Release / Audit 四类，删除 Graph 兼容层和旧路径，补齐 Panel 的 Git / Diagnostics / Tests / Snapshot / Manifest 数据，使 Project Panel V1 达到“完成”状态。**

---

# 1. 范围

本需求包含 3 个大块：

```text
1. Agent Roles Finalization
2. Graph Decompat and Cleanup
3. Panel Data Completeness
```

---

# 2. 非目标

本需求不做以下事情：

```text
不实现 OpenSpec Authoring
不写 SPEC change
不写 Approved SPEC
不生成 Goal Tree fact
不启动 AgentRun
不写用户源码
不运行项目测试命令
不执行项目 build 命令
不创建 PR
不创建远程 Issue
不调用模型
不实现 TDD 执行
不实现 Release 执行
不实现 Audit 执行
不做多人协作
不做 CRDT
不做 Review Changes UI
不做 checkpoint restore
```

---

# 3. Agent 角色最终收敛

## 3.1 当前问题

008.3 / PR #18 中定义了 5 个 Agent 角色：

```text
1. 需求接待 Agent
2. 规格计划 Agent
3. 实现执行 Agent
4. 发布交付 Agent
5. 代码审计 Agent
```

现在需要把前两个合并。

原因：

```text
需求接待 + 规格计划 都属于上游“需求 -> SPEC -> Goal Tree”链路
不需要拆成两个顶层角色
```

---

## 3.2 最终 4 个 Agent 角色

从本需求开始，AgentFlow 顶层只保留 4 个 Agent 角色：

```text
1. Spec Agent / 需求规格 Agent
2. Build Agent / 实现执行 Agent
3. Release Agent / 发布交付 Agent
4. Audit Agent / 代码审计 Agent
```

---

## 3.3 Spec Agent

Spec Agent 合并原来的：

```text
需求接待 Agent
规格计划 Agent
```

职责：

```text
接收人类输入
判断请求类型
运行 request-triage
运行 requirement-intake-filter
提出澄清问题
判断是否 ready-for-openspec
生成 SPEC Draft Preview
等待人类确认
从 Approved SPEC 生成 Goal Tree
```

使用：

```text
Agentflow.md
SPEC.md
request-triage
requirement-intake-filter
openspec-authoring
goal-tree-materialization
boundary-check
validation
```

输出：

```text
Requirement Intake Result
SPEC Draft Preview
Approved SPEC
Goal Tree
Goal / Milestone / Issue
```

当前状态：

```text
partially enabled
```

解释：

```text
需求过滤能力已启用
SPEC 写入和 Goal Tree materialization 仍需后续需求授权
```

禁止：

```text
不能跳过 requirement-intake-filter
不能绕过人类确认
不能从聊天直接生成 Goal Tree
不能从未批准 SPEC 生成 Goal Tree
不能写源码
不能执行命令
不能启动 AgentRun
```

---

## 3.4 Build Agent

职责：

```text
未来按 TDD.md 执行实现
从 SPEC / Issue 推导测试
先写测试
最小实现
红绿重构
记录 evidence
```

当前状态：

```text
not authorized yet
```

---

## 3.5 Release Agent

职责：

```text
未来处理 commit / PR / review / release note / deploy / rollback / release evidence
```

当前状态：

```text
not authorized yet
```

---

## 3.6 Audit Agent

职责：

```text
未来检查是否符合 SPEC
检查是否越界
检查架构影响
检查测试覆盖
检查 legacy 回流
检查未授权执行 / 写入 / 模型调用
检查 evidence 完整性
```

当前状态：

```text
not authorized yet
```

---

## 3.7 需要修改

需要更新：

```text
crates/agent-manual/src/templates.rs
docs/requirements/008-3-agentflow-workflow-directory-blueprint-v1-final.md
docs/requirements/008-4-project-panel-v1.md
AGENTS.md
README.md
GOAL.md
ROADMAP.md
verification.md
Browser Preview mock
```

`Agentflow.md` 中 Agent Roles 章节最终应为：

```md
## Agent Roles

### 1. Spec Agent / 需求规格 Agent

Status: partially enabled.

Receives human input, classifies request type, runs request-triage and requirement-intake-filter, asks clarification questions, produces SPEC Draft Preview after intake is ready, waits for human confirmation, and later materializes Goal Tree from Approved SPEC.

It cannot bypass intake, skip human confirmation, generate Goal Tree from chat directly, write source code, execute commands, or start AgentRun.

### 2. Build Agent / 实现执行 Agent

Status: not authorized yet.

Future role for TDD-driven implementation from approved Goal Tree issues.

### 3. Release Agent / 发布交付 Agent

Status: not authorized yet.

Future role for commit, PR, review, release note, deploy, rollback, and release evidence.

### 4. Audit Agent / 代码审计 Agent

Status: not authorized yet.

Future role for SPEC alignment, boundary compliance, architecture impact, test coverage, legacy reintroduction, unauthorized execution, and evidence completeness review.
```

---

## 3.8 角色验收标准

```text
- [ ] Agentflow.md 只定义 4 个顶层 Agent 角色。
- [ ] 角色为 Spec / Build / Release / Audit。
- [ ] 不再出现 “Intake Agent” 作为顶层角色。
- [ ] 不再出现 “Spec Planning Agent” 作为顶层角色。
- [ ] Spec Agent 明确包含需求接待、需求过滤、SPEC Draft Preview、Approved SPEC -> Goal Tree。
- [ ] Build / Release / Audit 明确 not authorized yet。
```

---

# 4. Graph 去兼容化与完全清理

## 4.1 当前问题

PR #19 仍保留了很多 Graph compatibility：

```text
crates/graph 目录
agentflow-graph 依赖别名
GraphStatus / GraphManifest / GraphContextPack 等类型 alias
prepare_project_graph 等旧 API
Tauri graph commands
TypeScript graph.ts / useProjectGraph.ts
.agentflow/output/graph 兼容路径
.agentflow/graph 兼容路径
workspace-manifest compat.legacyGraphOutput
workspace-manifest compat.legacyGraphCanonical
```

本需求要求：

```text
Graph 不再作为产品名、代码域名、路径名、API 名、UI 名存在。
```

---

## 4.2 最终目标

从本需求完成后：

```text
Panel = 唯一 canonical domain
Graph = 不再兼容
```

也就是说：

```text
不保留 Graph API alias
不保留 Graph Tauri commands
不保留 Graph TS aliases
不保留 Graph path compat
不保留 Graph wording in active UI / active docs / active code
```

---

## 4.3 Rust crate 清理

当前：

```text
crates/graph
package = agentflow-panel
```

最终应改为：

```text
crates/panel
package = agentflow-panel
```

要求：

```text
- [ ] Cargo workspace 使用 crates/panel。
- [ ] crates/graph 目录删除。
- [ ] apps/desktop/src-tauri/Cargo.toml 依赖改为 agentflow-panel = { path = "../../../crates/panel" }。
- [ ] 不再使用 agentflow-graph dependency alias。
- [ ] Rust public API 只保留 Panel 命名。
- [ ] 删除 Graph* type alias。
- [ ] 删除 prepare_project_graph / search_project_graph / build_graph_context_pack 等旧 API。
- [ ] 删除 graph_preflight / analyze_graph_impact / check_graph_git_protection 等旧命名。
```

---

## 4.4 Tauri command 清理

当前仍有：

```text
commands/graph.rs
prepare_project_graph
load_project_graph_status
load_project_graph_manifest
search_project_graph
build_graph_context_pack
load_graph_context_pack
graph_preflight
analyze_graph_impact
check_graph_git_protection
```

最终应改为：

```text
commands/panel.rs
prepare_project_panel
load_project_panel_status
load_project_panel_manifest
search_project_panel
build_panel_context_pack
load_panel_context_pack
panel_preflight
analyze_panel_impact
check_panel_git_protection
load_panel_snapshot
load_panel_git_status
load_panel_diagnostics
load_panel_tests
```

要求：

```text
- [ ] 删除 commands/graph.rs 或改名为 panel.rs。
- [ ] main.rs 只注册 panel commands。
- [ ] 不再注册 graph commands。
- [ ] 前端 invoke 只调用 panel commands。
```

---

## 4.5 TypeScript / Frontend 清理

当前仍有：

```text
apps/desktop/src/types/graph.ts
apps/desktop/src/features/project-files/hooks/useProjectGraph.ts
ProjectGraphState
GraphStatusSnapshot
GraphManifestSnapshot
GraphContextPack
```

最终应改为：

```text
apps/desktop/src/types/panel.ts
apps/desktop/src/features/project-files/hooks/useProjectPanel.ts
ProjectPanelState
PanelStatusSnapshot
PanelManifestSnapshot
PanelContextPack
```

要求：

```text
- [ ] 删除 graph.ts 或改名 panel.ts。
- [ ] 删除 useProjectGraph.ts 或改名 useProjectPanel.ts。
- [ ] 前端主类型使用 Panel*。
- [ ] 不保留 Graph* TS alias。
- [ ] UI 文案全部使用 Panel / 项目现场 / 工作现场。
```

---

## 4.6 路径清理

从本需求完成后，只允许 Panel 写入：

```text
.agentflow/panel/**
```

不允许再读取 / 写入 / 创建：

```text
.agentflow/output/graph/**
.agentflow/graph/**
.agentflow/inspect/**
.agentflow/panel/output/**
```

处理策略：

```text
- 旧路径不再兼容读取。
- prepare 不再迁移旧路径。
- workspace-manifest 不再记录 legacyGraphOutput / legacyGraphCanonical。
- 如果测试 fixture 需要旧路径，必须改成 panel 路径。
```

可选清理：

```text
如果运行时检测到旧 .agentflow/output/graph 或 .agentflow/graph，可以在 .agentflow/output/backup/legacy-graph/<timestamp>/ 下备份后删除。
```

但 V1 推荐：

```text
不主动删除用户已有旧目录，只是不再读取、不再写入、不再在 manifest 中记录。
```

---

## 4.7 文档清理

需要清理 active docs 中的 Graph 残留。

需要更新：

```text
README.md
GOAL.md
ROADMAP.md
AGENTS.md
docs/README.md
docs/requirements/008-3-agentflow-workflow-directory-blueprint-v1-final.md
docs/requirements/008-4-project-panel-v1.md
docs/requirements/README.md
docs/requirements/next-requirements.md
verification.md
```

规则：

```text
active docs / active code 中不再使用 Graph 作为当前产品概念。
```

允许保留：

```text
docs/archive/**
旧历史 requirement 中的历史描述
git history
```

但当前主线文档必须使用：

```text
Panel
项目现场
工作现场
```

---

## 4.8 Graph 清理验收标准

```text
- [ ] crates/graph 不存在。
- [ ] crates/panel 存在。
- [ ] package 为 agentflow-panel。
- [ ] Cargo workspace 不引用 crates/graph。
- [ ] apps/desktop/src-tauri/Cargo.toml 不使用 agentflow-graph alias。
- [ ] Rust public API 无 Graph* alias。
- [ ] Rust public API 无 graph_* / *_graph 命名。
- [ ] Tauri commands 无 graph command。
- [ ] Frontend invoke 无 graph command。
- [ ] TypeScript 无 Graph* primary/alias types。
- [ ] UI 文案无 “Graph”。
- [ ] workspace-manifest 无 legacyGraphOutput。
- [ ] workspace-manifest 无 legacyGraphCanonical。
- [ ] 新写入只进 .agentflow/panel/**。
- [ ] 不创建 .agentflow/output/graph。
- [ ] 不创建 .agentflow/graph。
- [ ] 不创建 .agentflow/inspect。
```

建议执行：

```bash
rg -n "Graph|graph" crates apps Cargo.toml Cargo.lock README.md GOAL.md ROADMAP.md docs/README.md docs/requirements verification.md
```

允许例外：

```text
docs/archive/**
历史旧需求文件
本需求文档中“Graph removal”说明段落
```

---

# 5. Panel V1 数据完整度收口

## 5.1 当前问题

PR #19 已生成以下文件：

```text
manifest.json
file-tree.json
languages.json
symbols.json
relations.json
diagnostics.json
git.json
tests.json
search/**
context-packs/**
snapshots/**
index/panel.db
```

但部分文件内容偏轻：

```text
git.json 信息不完整
diagnostics.json 为空数组
tests.json 只是测试文件列表
snapshot 信息偏少
manifest 仍有旧 GraphMeta 味道
```

本需求目标是：

```text
Panel V1 不再是基本完成，而是目标完成。
```

---

## 5.2 `manifest.json` 完整目标

路径：

```text
.agentflow/panel/manifest.json
```

必须使用 Panel 语义，不再保留 Graph 字段。

必须包含：

```text
version = panel-manifest.v1
projectRoot
status
backend
lastIndexedAt
activeSnapshotId
paths
summary
worktree
watcher
degradedReasons
warnings
errors
```

不得出现：

```text
graphDb
graph_db
graphRevision
```

建议结构：

```json
{
  "version": "panel-manifest.v1",
  "projectRoot": "/path/to/project",
  "status": "ready",
  "backend": "native",
  "lastIndexedAt": 1780360000,
  "activeSnapshotId": "panel-snapshot-001",
  "paths": {
    "database": ".agentflow/panel/index/panel.db",
    "fileTree": ".agentflow/panel/file-tree.json",
    "languages": ".agentflow/panel/languages.json",
    "symbols": ".agentflow/panel/symbols.json",
    "relations": ".agentflow/panel/relations.json",
    "diagnostics": ".agentflow/panel/diagnostics.json",
    "git": ".agentflow/panel/git.json",
    "tests": ".agentflow/panel/tests.json"
  },
  "summary": {
    "files": 0,
    "languages": 0,
    "symbols": 0,
    "relations": 0,
    "diagnostics": 0,
    "tests": 0
  },
  "worktree": {
    "root": "/path/to/project",
    "gitBranch": null,
    "headSha": null,
    "dirty": false
  },
  "watcher": {
    "status": "watching",
    "backend": "native"
  },
  "degradedReasons": [],
  "warnings": [],
  "errors": []
}
```

---

## 5.3 `git.json` 完整目标

路径：

```text
.agentflow/panel/git.json
```

必须包含：

```text
version = panel-git.v1
isGitRepo
branch
headSha
dirty
modifiedFiles
untrackedFiles
deletedFiles
ignoredFiles
agentflowProtected
warnings
errors
```

如果不是 Git repo：

```json
{
  "version": "panel-git.v1",
  "isGitRepo": false,
  "dirty": false,
  "warnings": ["Project is not a Git repository"]
}
```

只允许只读 Git 查询：

```text
git status --porcelain
git rev-parse --abbrev-ref HEAD
git rev-parse HEAD
git check-ignore
```

禁止：

```text
git init
git add
git commit
git reset
git checkout
```

---

## 5.4 `diagnostics.json` 完整目标

路径：

```text
.agentflow/panel/diagnostics.json
```

必须包含：

```text
version = panel-diagnostics.v1
summary
items
```

items 至少支持：

```text
scanner-error
parser-fallback
unsupported-language
large-file-skipped
binary-file-skipped
symlink-skipped
watcher-degraded
legacy-panel-input-ignored
```

如果没有诊断问题，也必须写结构化空结果：

```json
{
  "version": "panel-diagnostics.v1",
  "summary": {
    "errors": 0,
    "warnings": 0,
    "infos": 0
  },
  "items": []
}
```

不能只是：

```json
[]
```

---

## 5.5 `tests.json` 完整目标

路径：

```text
.agentflow/panel/tests.json
```

必须包含：

```text
version = panel-tests.v1
testFiles
frameworks
commandCandidates
sourceToLikelyTests
testToLikelySources
hints
```

必须支持语言 / 框架线索：

```text
Rust:
  *_test.rs
  tests/
  cargo test

TypeScript / JavaScript:
  *.test.ts
  *.spec.ts
  *.test.tsx
  *.spec.tsx
  package.json scripts.test
  vitest / jest / playwright hints

Python:
  test_*.py
  *_test.py
  pytest / unittest hints

Go:
  *_test.go
  go test ./...

Java / Kotlin:
  src/test/
  gradle test
  mvn test

Swift:
  Tests/
  swift test

Dart / Flutter:
  test/
  flutter test
```

Panel 不运行测试，只发现测试和候选命令。

---

## 5.6 `snapshots/` 完整目标

每次完整 index 后生成：

```text
.agentflow/panel/snapshots/<snapshot-id>.json
```

必须包含：

```text
version = panel-snapshot.v1
snapshotId
createdAt
projectRoot
manifestSummary
worktree
gitSummary
fileSummary
fileHashSummary
diagnosticsSummary
testsSummary
panelPaths
```

示例：

```json
{
  "version": "panel-snapshot.v1",
  "snapshotId": "panel-snapshot-001",
  "createdAt": 1780360000,
  "projectRoot": "/path/to/project",
  "manifestSummary": {
    "files": 1200,
    "symbols": 4300,
    "relations": 9800
  },
  "worktree": {
    "gitBranch": "main",
    "headSha": "abc123",
    "dirty": true
  },
  "gitSummary": {
    "modified": 2,
    "untracked": 1,
    "deleted": 0
  },
  "diagnosticsSummary": {
    "errors": 0,
    "warnings": 3
  },
  "testsSummary": {
    "testFiles": 42,
    "frameworks": ["cargo", "vitest"]
  }
}
```

---

## 5.7 `file-tree.json` 完整目标

路径：

```text
.agentflow/panel/file-tree.json
```

每个文件至少包含：

```text
path
kind
extension
size
modifiedAt
language
mimeType
isBinary
isLarge
isSymlink
ignored
gitStatus
diagnosticSummary
```

---

## 5.8 `languages.json` 完整目标

路径：

```text
.agentflow/panel/languages.json
```

必须包含：

```text
version = panel-languages.v1
languages[]
```

每个 language 包含：

```text
language
fileCount
entryFiles
configFiles
packageFiles
testFrameworkHints
mobilePlatformHints
```

---

## 5.9 `symbols.json` 和 `relations.json`

保留现有能力，但要确认：

```text
version 字段存在或 wrapper 存在
路径稳定
内容不是 JSONL
文件生成在 .agentflow/panel/
```

如果当前是 raw array，也可以接受，但必须在 manifest 中正确记录。

---

## 5.10 Search indexes

必须生成：

```text
.agentflow/panel/search/file-index.json
.agentflow/panel/search/symbol-index.json
.agentflow/panel/search/content-index.json
```

其中：

```text
file-index 包含文件基本搜索字段
symbol-index 包含符号搜索字段
content-index 包含 chunk/snippet 搜索字段
```

---

## 5.11 Context Pack

路径：

```text
.agentflow/panel/context-packs/<context-pack-id>.json
```

必须使用：

```text
version = panel-context-pack.v1
```

不得再出现：

```text
graph-context-pack
```

---

## 5.12 Panel Preflight

`panel_preflight(projectRoot)` 必须检查：

```text
panel path ready
manifest exists
file-tree exists
symbols exists
git status readable
diagnostics readable
tests index exists
watcher status ready/degraded
```

返回：

```text
ready / degraded / failed
reasons
```

不得使用：

```text
Graph 构建失败
Graph 已就绪
Graph 可用
```

---

## 5.13 Panel watcher

要求：

```text
default OS native watcher
fallback fingerprint watcher
fallback 状态为 degraded
watcher detail 写入 PanelStatus
ignore .agentflow/
ignore node_modules / target / dist / build
debounce
no self-refresh loop
```

所有文案使用：

```text
Panel watcher
```

不再使用：

```text
Graph watcher
```

---

# 6. 写入边界

允许写：

```text
.agentflow/panel/**
```

不允许写：

```text
.agentflow/output/graph/**
.agentflow/graph/**
.agentflow/inspect/**
.agentflow/panel/output/**
用户源码
.agentflow/spec/**
.agentflow/goal-tree/**
.agentflow/execute/**
.agentflow/output/evidence/**
.agentflow/output/audit/**
.agentflow/output/release/**
```

---

# 7. 不做什么

```text
不实现 OpenSpec Authoring
不写 SPEC change
不写 Approved SPEC
不生成 Goal Tree fact
不启动 AgentRun
不写用户源码
不执行项目测试命令
不执行项目构建命令
不创建 PR
不创建远程 Issue
不调用模型
不实现 TDD 执行
不实现 Release 执行
不实现 Audit 执行
不做多人协作
不做 CRDT
不做 Review Changes UI
不做 checkpoint restore
```

---

# 8. 开发切片

## Slice 1：Agent roles finalization

```text
Spec / Build / Release / Audit 四角色
更新 Agentflow.md / docs / browser preview
```

验收：

```text
active templates 中不再出现 Intake Agent / Spec Planning Agent 顶层角色
```

---

## Slice 2：Graph decompat cleanup

```text
crates/graph -> crates/panel
删除 Graph API alias
删除 Graph Tauri commands
删除 Graph TS aliases
删除旧路径 compat
```

验收：

```text
active code 不再使用 Graph 命名
```

---

## Slice 3：Panel data schema completion

```text
manifest
git
diagnostics
tests
snapshots
languages
file-tree
```

验收：

```text
每个 Panel 文件都有完整结构
```

---

## Slice 4：Panel preflight / watcher naming

```text
panel_preflight 完整检查
Panel watcher 文案
```

---

## Slice 5：Docs / verification

```text
requirements
README
GOAL
ROADMAP
verification
```

---

# 9. 总验收标准

```text
- [ ] 新增 docs/requirements/008-4-1-project-panel-finalization-and-graph-removal-v1.md。
- [ ] Agent roles 收敛为 Spec / Build / Release / Audit 四个。
- [ ] Agentflow.md 中不再把 Intake Agent / Spec Planning Agent 作为顶层角色。
- [ ] crates/graph 删除。
- [ ] crates/panel 存在。
- [ ] package 为 agentflow-panel。
- [ ] Cargo workspace 不引用 crates/graph。
- [ ] Tauri dependency 不使用 agentflow-graph alias。
- [ ] Rust public API 无 Graph* alias。
- [ ] Rust public API 无 graph_* / *_graph 命名。
- [ ] Tauri commands 无 graph command。
- [ ] Frontend invoke 无 graph command。
- [ ] TypeScript 无 Graph* primary / alias types。
- [ ] workspace-manifest 无 legacyGraphOutput。
- [ ] workspace-manifest 无 legacyGraphCanonical。
- [ ] active UI 文案无 Graph。
- [ ] active docs 无 Graph 当前概念。
- [ ] 新写入只进 .agentflow/panel/**。
- [ ] 不创建 .agentflow/output/graph。
- [ ] 不创建 .agentflow/graph。
- [ ] 不创建 .agentflow/inspect。
- [ ] 不创建 .agentflow/panel/output。
- [ ] panel/manifest.json 使用完整 Panel schema。
- [ ] panel/manifest.json 不包含 graphDb / graph_db。
- [ ] panel/git.json 完整包含 isGitRepo / branch / headSha / dirty / modifiedFiles / untrackedFiles / deletedFiles / ignoredFiles / agentflowProtected。
- [ ] panel/diagnostics.json 是结构化对象，不是空数组。
- [ ] panel/tests.json 包含 testFiles / frameworks / commandCandidates / sourceToLikelyTests / testToLikelySources / hints。
- [ ] panel/snapshots/*.json 包含 worktree / gitSummary / diagnosticsSummary / testsSummary。
- [ ] panel/file-tree.json 包含 gitStatus / diagnosticSummary。
- [ ] panel/languages.json 包含 packageFiles / configFiles / testFrameworkHints。
- [ ] Panel Context Pack 使用 panel-context-pack.v1。
- [ ] Panel Preflight 不使用 Graph 文案。
- [ ] Panel watcher 不使用 Graph 文案。
- [ ] Panel 不写用户源码。
- [ ] Panel 不执行项目测试。
- [ ] Panel 不调用模型。
- [ ] Panel 不写 SPEC / Goal Tree / AgentRun。
- [ ] cargo fmt --check 通过。
- [ ] cargo test -p agentflow-panel 通过。
- [ ] cargo test -p agentflow-desktop 通过。
- [ ] cargo test 通过。
- [ ] npm --prefix apps/desktop run build 通过。
- [ ] git diff --check 通过。
```

---

# 10. 强制检查命令

```bash
cargo fmt --check
cargo test -p agentflow-panel
cargo test -p agentflow-desktop
cargo test
npm --prefix apps/desktop run build
git diff --check
```

Graph 清理检查：

```bash
rg -n "Graph|graph" crates apps Cargo.toml Cargo.lock README.md GOAL.md ROADMAP.md docs/README.md docs/requirements verification.md
```

允许例外：

```text
docs/archive/**
历史旧需求文件
008.4.1 本需求文档中用于说明 Graph removal 的段落
```

路径检查：

```bash
rg -n "\.agentflow/output/graph|\.agentflow/graph|\.agentflow/inspect|\.agentflow/panel/output" crates apps docs README.md GOAL.md ROADMAP.md verification.md
```

允许例外：

```text
008.4.1 本需求文档中用于说明禁止路径的段落
docs/archive/**
```

---

# 11. PR 说明要求

PR 描述必须说明：

```text
1. Agent roles 如何从 5 个收敛为 4 个。
2. Spec Agent 覆盖了哪些职责。
3. Graph compatibility 删除了哪些内容。
4. crates/graph 是否删除。
5. 旧 Graph Tauri commands 是否删除。
6. 旧 Graph TS aliases 是否删除。
7. 旧 Graph 路径是否不再读取 / 写入。
8. Panel 数据完整度补了哪些文件。
9. git.json / diagnostics.json / tests.json / snapshots 补了什么。
10. 本次没有写 SPEC / Goal Tree / AgentRun。
11. 本次没有写用户源码 / 执行项目测试 / 调用模型。
12. 验证命令和结果。
```

---

# 12. Codex 执行指令

```md
请执行 008.4.1 - Project Panel Finalization and Graph Removal V1。

目标：
完成 Project Panel 的最终收口。把 Agent 顶层角色从 5 个收敛为 4 个：Spec / Build / Release / Audit。彻底删除 Graph 兼容层和旧 Graph 路径，不再保留 Graph API / Graph Tauri commands / Graph TS aliases / legacyGraphOutput / legacyGraphCanonical。补齐 Panel 的 git.json、diagnostics.json、tests.json、snapshots 和 manifest，使 Project Panel V1 达到“目标完成”状态。

必须遵守：
1. 顶层 Agent 角色只保留 Spec / Build / Release / Audit。
2. Spec Agent 合并原 Intake + Spec Planning 职责。
3. Graph 不再作为兼容名称。
4. 删除 crates/graph，改为 crates/panel。
5. 不保留 agentflow-graph dependency alias。
6. 不保留 Graph* Rust type alias。
7. 不保留 graph Tauri commands。
8. 不保留 Graph* TS type alias。
9. 不保留 .agentflow/output/graph 兼容读取。
10. 不保留 .agentflow/graph 兼容读取。
11. 不创建 .agentflow/inspect。
12. 不创建 .agentflow/panel/output。
13. 新写入只允许 .agentflow/panel/**。
14. Panel 不写 SPEC。
15. Panel 不写 Goal Tree。
16. Panel 不启动 AgentRun。
17. Panel 不写用户源码。
18. Panel 不执行项目测试。
19. Panel 不调用模型。
20. 本次必须把 Panel 数据文件补齐到目标完成，不接受 basic / partial / degraded as completion。

实现范围：
- 新增 docs/requirements/008-4-1-project-panel-finalization-and-graph-removal-v1.md。
- 更新 Agentflow.md 模板，角色收敛为 4 个。
- 更新 active docs / README / GOAL / ROADMAP / verification。
- crates/graph -> crates/panel。
- Rust public API 全部改为 Panel 命名。
- Tauri commands 全部改为 Panel 命名。
- Frontend hooks/types 全部改为 Panel 命名。
- workspace-manifest 删除 legacyGraphOutput / legacyGraphCanonical。
- panel/manifest.json 改成完整 Panel schema。
- panel/git.json 补完整 Git status。
- panel/diagnostics.json 补结构化 diagnostics。
- panel/tests.json 补完整测试现场。
- panel/snapshots/*.json 补完整现场快照。
- panel/file-tree.json 补 gitStatus / diagnosticSummary。
- panel/languages.json 补 packageFiles / configFiles / testFrameworkHints。
- Panel Context Pack 使用 panel-context-pack.v1。
- Panel Preflight / watcher 文案全部去 Graph。

验证命令：
- cargo fmt --check
- cargo test -p agentflow-panel
- cargo test -p agentflow-desktop
- cargo test
- npm --prefix apps/desktop run build
- git diff --check
- rg -n "Graph|graph" crates apps Cargo.toml Cargo.lock README.md GOAL.md ROADMAP.md docs/README.md docs/requirements verification.md
- rg -n "\.agentflow/output/graph|\.agentflow/graph|\.agentflow/inspect|\.agentflow/panel/output" crates apps docs README.md GOAL.md ROADMAP.md verification.md
```

---

# 13. 完成定义

本需求完成后：

```text
Agent roles = Spec / Build / Release / Audit

Panel = 唯一项目现场域

Graph = 完全退出 active code / active docs / active UI / active API

Panel data = manifest / file-tree / languages / symbols / relations / diagnostics / git / tests / search / context-packs / snapshots / panel.db 全部目标完成
```

最终一句话：

> **008.4.1 是 Project Panel 的最终收口：角色收敛、Graph 清零、Panel 数据补全，完成后不再出现基本完成或兼容残留。**
