# 008.4 - Project Panel V1

创建日期：2026-06-04  
执行者：Codex  
状态：待开发  
版本：final-draft

---

## 用户目标

当前 AgentFlow 已经完成：

```text
008 - Agent Working Manual Bootstrap V1
008.1 - Agent Working Manual Health Polish
008.2 - Requirement Intake Filter Skill V1
008.3 - AgentFlow Workflow Directory Blueprint V1
```

现在要继续完成第二优先级能力：

```text
Panel = 项目工作现场
```

当前旧模块叫：

```text
Graph
```

但经过对 Warp / Zed 的项目现场和 Agent 工作流重新梳理后，我们决定把：

```text
Graph
```

升级并改名为：

```text
Panel
```

大白话：

> Graph 听起来只是“代码图谱”或“代码索引”。  
> 但 AgentFlow 真正需要的是 Agent 的“项目工作现场”。  
> 它不仅要知道有哪些文件和符号，还要知道文件树、语言、依赖关系、Git 状态、诊断、测试、搜索索引、上下文包和现场快照。  
> 所以这层应该叫 Panel，代表 Agent 进入项目后看到的完整工作现场。

---

## 一句话定义

> **Project Panel V1 是 AgentFlow 的项目工作现场层。它替代 Graph 作为 canonical 概念，负责维护当前项目的文件现场、语言现场、符号现场、关系现场、诊断现场、Git 现场、测试现场、搜索索引、Context Pack、Panel Snapshot 和 watcher 状态，为后续 Intake / SPEC / Goal Tree / TDD / Build / Release / Audit 提供统一上下文。**

---

# 1. 当前流程中的位置

AgentFlow 的工作流优先级现在是：

```text
1. define/
   = 环境准备 / Agent 工作手册 / 规则

2. panel/
   = 现场准备 / 项目工作现场

3. spec/
   = 真实 SPEC 产物

4. goal-tree/
   = 真实目标树事实源

5. execute/
   = 未来执行过程

6. output/
   = 证据 / 审计 / 发布 / 日志 / 备份

7. state/
   = 健康 / 锁 / 会话 / 索引状态
```

因此 Panel 是第二优先级：

```text
先准备规则
再准备现场
再进入 SPEC / Goal Tree / AgentRun
```

也就是说：

```text
Define Ready
  ↓
Panel Ready / Degraded
  ↓
Requirement Intake
  ↓
SPEC Draft Preview
  ↓
Approved SPEC
  ↓
Goal Tree
  ↓
Future AgentRun
```

---

# 2. 为什么 Graph 要升级成 Panel

## 2.1 Graph 名称太窄

Graph 更像：

```text
符号图
调用关系
代码索引
```

但我们需要的是：

```text
项目现场
文件现场
语言现场
Git 现场
诊断现场
测试现场
搜索现场
Agent 上下文现场
```

所以名称升级为：

```text
Panel
```

中文可以叫：

```text
项目现场
工作现场
```

---

## 2.2 Panel 先服务 SPEC，而不是直接服务写代码

Panel 不是 Build Agent 的直接代码执行工具。

Panel 首先服务：

```text
Intake Agent
Spec Planning Agent
```

它帮助 Agent 回答：

```text
这个需求可能涉及哪些模块？
哪些文件需要看？
项目现在是什么技术栈？
有哪些测试？
有没有当前诊断问题？
Git 工作区是否干净？
哪些文件适合进入 Context Pack？
```

Panel 不是让 Agent 直接开始写代码。

---

## 2.3 Panel 是现场，不是需求源

Panel 只提供现场证据。

它不决定：

```text
用户要做什么
需求是否批准
Goal Tree 怎么拆
Agent 是否能执行
PR 是否能发
审计是否通过
```

这些分别属于：

```text
spec/
goal-tree/
execute/
release/
output/audit/
```

---

# 3. 命名调整

## 3.1 新 canonical name

从 008.4 开始：

```text
Panel = canonical product/domain name
Graph = legacy / internal compatibility name
```

新的用户可见中文：

```text
项目现场
工作现场
```

---

## 3.2 新 canonical path

新的 canonical path：

```text
.agentflow/panel/
```

Panel 产物必须直接放在：

```text
.agentflow/panel/**
```

不要再放到：

```text
.agentflow/output/graph/**
.agentflow/graph/**
.agentflow/inspect/**
```

---

## 3.3 禁止 `panel/output/` 包裹整个 Panel

错误结构：

```text
.agentflow/panel/output/manifest.json
.agentflow/panel/output/file-tree.json
.agentflow/panel/output/symbols.json
```

正确结构：

```text
.agentflow/panel/manifest.json
.agentflow/panel/file-tree.json
.agentflow/panel/symbols.json
```

也就是说：

> **Panel 是一个顶层工作现场域；它的产物直接放在 `.agentflow/panel/` 下。**

如果未来 Panel 自己确实需要导出临时报表，可以再新增：

```text
.agentflow/panel/output/
```

但它只能和 `search/`、`context-packs/` 同级，不能包住整个 Panel。

V1 不需要：

```text
.agentflow/panel/output/
```

---

# 4. 目录结构

## 4.1 `.agentflow/` 总结构

008.4 后目标结构：

```text
.agentflow/
├── workspace.yaml
├── config.yaml
├── workspace-manifest.json
│
├── define/
│   ├── agent/
│   ├── spec/
│   ├── tdd/
│   ├── release/
│   └── audit/
│
├── panel/
│   ├── manifest.json
│   ├── file-tree.json
│   ├── languages.json
│   ├── symbols.json
│   ├── relations.json
│   ├── diagnostics.json
│   ├── git.json
│   ├── tests.json
│   ├── search/
│   │   ├── file-index.json
│   │   ├── symbol-index.json
│   │   └── content-index.json
│   ├── context-packs/
│   │   └── <context-pack-id>.json
│   ├── snapshots/
│   │   └── <snapshot-id>.json
│   └── index/
│       └── panel.db
│
├── spec/
│   ├── changes/
│   ├── approvals/
│   ├── drafts/
│   └── index.json
│
├── goal-tree/
│   ├── goal-tree.json
│   ├── goals/
│   ├── milestones/
│   ├── issues/
│   └── materialization/
│
├── execute/
│   ├── runs/
│   ├── leases/
│   └── commands/
│
├── output/
│   ├── evidence/
│   ├── audit/
│   ├── release/
│   ├── backup/
│   ├── logs/
│   ├── cache/
│   └── tmp/
│
└── state/
    ├── health/
    ├── locks/
    ├── sessions/
    └── indexes/
```

---

## 4.2 `.agentflow/panel/` 结构

Panel V1 canonical structure：

```text
.agentflow/panel/
├── manifest.json
├── file-tree.json
├── languages.json
├── symbols.json
├── relations.json
├── diagnostics.json
├── git.json
├── tests.json
├── search/
│   ├── file-index.json
│   ├── symbol-index.json
│   └── content-index.json
├── context-packs/
│   └── <context-pack-id>.json
├── snapshots/
│   └── <snapshot-id>.json
└── index/
    └── panel.db
```

---

## 4.3 全局 `.agentflow/output/` 的职责

全局 output 只放跨阶段运行产物：

```text
.agentflow/output/
├── evidence/
├── audit/
├── release/
├── backup/
├── logs/
├── cache/
└── tmp/
```

它不放 Panel 现场产物。

因此：

```text
.agentflow/output/graph/
```

从 008.4 开始只作为 legacy compatibility path。

---

# 5. Panel 文件职责

## 5.1 `manifest.json`

Panel 总入口。

记录：

```text
panel version
projectRoot
lastIndexedAt
status
backend
watcher status
file count
language count
symbol count
relation count
diagnostic count
test count
git summary
active snapshot id
degraded reasons
legacy compatibility paths
```

示例：

```json
{
  "version": "panel-manifest.v1",
  "projectRoot": "/path/to/project",
  "status": "ready",
  "backend": "native",
  "lastIndexedAt": 1780360000,
  "activeSnapshotId": "panel-snapshot-001",
  "summary": {
    "files": 1200,
    "languages": 8,
    "symbols": 4300,
    "relations": 9800,
    "diagnostics": 12,
    "tests": 240
  },
  "worktree": {
    "root": "/path/to/project",
    "gitBranch": "main",
    "headSha": "abc123",
    "dirty": true
  },
  "degradedReasons": []
}
```

---

## 5.2 `file-tree.json`

文件现场。

记录：

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

文件树不是简单目录树，它是 Agent 看项目现场的第一层地图。

---

## 5.3 `languages.json`

语言现场。

记录：

```text
language
file count
entry files
config files
package files
test framework hints
mobile platform hints
```

用于：

```text
Spec Planning Agent 判断涉及技术栈
Build Agent 未来推导测试入口
Audit Agent 判断是否跨语言影响
```

---

## 5.4 `symbols.json`

符号现场。

记录：

```text
function
struct
class
enum
interface
trait
component
module
route
handler
test
```

示例：

```json
{
  "symbolId": "sym-001",
  "name": "prepare_agent_working_manual",
  "kind": "function",
  "language": "rust",
  "file": "crates/agent-manual/src/manager.rs",
  "startLine": 10,
  "endLine": 24,
  "parent": null
}
```

---

## 5.5 `relations.json`

关系现场。

记录：

```text
imports
contains
parent_of
calls
references
configures
tests
depends_on
```

V1 不要求完整调用图，但必须有稳定 relation schema。

---

## 5.6 `diagnostics.json`

诊断现场。

记录：

```text
scanner errors
parser fallback
unsupported language
file too large
binary skipped
symlink skipped
index degraded reason
future LSP diagnostics placeholder
```

V1 不接 LSP。

未来再接：

```text
LSP diagnostics
type checker diagnostics
test diagnostics
```

---

## 5.7 `git.json`

Git 现场。

记录：

```text
isGitRepo
branch
headSha
dirty
modifiedFiles
untrackedFiles
deletedFiles
ignoredFiles
agentflowProtected
```

原因：

```text
Agent 后续执行前必须知道工作区是不是 dirty
Release Agent 必须知道变更范围
Audit Agent 必须知道有没有未授权文件改动
```

---

## 5.8 `tests.json`

测试现场。

记录：

```text
test files
test frameworks
test command candidates
source file -> likely tests
test file -> source under test
integration test hints
mobile test hints
```

Panel 不运行测试，但要告诉未来 TDD：

```text
测试在哪里
哪些测试可能相关
```

---

## 5.9 `search/`

搜索索引。

```text
search/file-index.json
search/symbol-index.json
search/content-index.json
```

用途：

```text
Intake Agent 找项目背景
Spec Planning Agent 找相关模块
Build Agent 找影响范围
Audit Agent 查证据
```

---

## 5.10 `context-packs/`

Panel Context Pack。

```text
context-packs/<context-pack-id>.json
```

Context Pack 用于给 SPEC / Goal Tree / Issue 提供上下文。

示例：

```json
{
  "version": "panel-context-pack.v1",
  "contextPackId": "ctx-001",
  "projectRoot": "/path/to/project",
  "source": {
    "type": "intake-result",
    "id": "intake-001"
  },
  "panelSnapshotId": "panel-snapshot-001",
  "recommendedFiles": [],
  "recommendedSymbols": [],
  "recommendedTests": [],
  "impactHints": [],
  "warnings": []
}
```

---

## 5.11 `snapshots/`

Panel 快照。

```text
snapshots/<snapshot-id>.json
```

用途：

```text
记录某一刻的项目现场
SPEC 写作时引用的是哪个现场
Goal Tree materialization 时引用的是哪个现场
AgentRun 时引用的是哪个现场
Audit 时复查的是哪个现场
```

未来每个 SPEC / Goal Tree / AgentRun 都应该能追溯：

```text
基于哪个 Panel Snapshot
```

---

## 5.12 `index/panel.db`

结构化索引数据库。

```text
index/panel.db
```

如果当前旧 Graph 使用：

```text
.agentflow/output/graph/index.db
```

008.4 要迁移到：

```text
.agentflow/panel/index/panel.db
```

并保留旧路径只读兼容。

---

# 6. Panel 的边界

## 6.1 Panel 做什么

Panel 负责回答：

```text
这个项目现在长什么样？
有哪些文件？
有哪些语言？
有哪些符号？
文件之间怎么关联？
有哪些诊断 / 错误 / fallback？
当前 Git 状态是什么？
哪些测试可能相关？
哪些文件适合放进 Context Pack？
某个 SPEC / Issue 应该看哪些上下文？
```

---

## 6.2 Panel 不做什么

Panel 不负责：

```text
不定义需求
不写 SPEC
不生成 Goal Tree
不执行 AgentRun
不写代码
不运行测试
不创建 PR
不做发布
不生成审计报告
```

这些分别属于：

```text
spec/
goal-tree/
execute/
release/
output/audit/
```

---

# 7. Panel 数据源

Panel V1 需要一次性收口以下数据源：

```text
file system scan
language detection
symbol scan
relation scan
git status
diagnostics
test detection
search index
context pack generation
snapshot generation
OS native watcher
```

注意：

```text
Panel 可以读取项目文件
Panel 可以读取 Git 状态
Panel 可以写 .agentflow/panel/**
Panel 不写用户源码
Panel 不执行测试命令
Panel 不调用模型
```

---

# 8. Panel 状态模型

新增：

```text
PanelStatusSnapshot
```

字段建议：

```ts
type PanelStatusSnapshot = {
  version: "panel-status.v1";
  projectRoot: string;
  status: "missing" | "indexing" | "ready" | "degraded" | "failed" | "blocked";
  ready: boolean;
  activeSnapshotId?: string | null;
  manifestPath: string;
  lastIndexedAt?: number | null;
  watcher: {
    status: "idle" | "watching" | "fallback" | "failed";
    backend: "native" | "fingerprint" | "none";
    detail?: unknown;
  };
  summary: {
    files: number;
    languages: number;
    symbols: number;
    relations: number;
    diagnostics: number;
    tests: number;
  };
  worktree: {
    root: string;
    gitBranch?: string | null;
    headSha?: string | null;
    dirty: boolean;
  };
  degradedReasons: string[];
  warnings: string[];
  errors: string[];
};
```

---

# 9. Panel Manifest

新增：

```text
PanelManifestSnapshot
```

它应该包含对所有 canonical files 的路径引用：

```json
{
  "version": "panel-manifest.v1",
  "projectRoot": "/path/to/project",
  "activeSnapshotId": "panel-snapshot-001",
  "paths": {
    "manifest": ".agentflow/panel/manifest.json",
    "fileTree": ".agentflow/panel/file-tree.json",
    "languages": ".agentflow/panel/languages.json",
    "symbols": ".agentflow/panel/symbols.json",
    "relations": ".agentflow/panel/relations.json",
    "diagnostics": ".agentflow/panel/diagnostics.json",
    "git": ".agentflow/panel/git.json",
    "tests": ".agentflow/panel/tests.json",
    "fileIndex": ".agentflow/panel/search/file-index.json",
    "symbolIndex": ".agentflow/panel/search/symbol-index.json",
    "contentIndex": ".agentflow/panel/search/content-index.json",
    "contextPacks": ".agentflow/panel/context-packs",
    "snapshots": ".agentflow/panel/snapshots",
    "database": ".agentflow/panel/index/panel.db"
  },
  "compat": {
    "legacyGraphOutput": ".agentflow/output/graph",
    "legacyGraphCanonical": ".agentflow/graph"
  }
}
```

---

# 10. workspace-manifest 更新

`workspace-manifest.json` 必须将 Panel 作为第二优先级：

```json
{
  "pipeline": [
    "define",
    "panel",
    "spec",
    "goal-tree",
    "execute",
    "release",
    "audit"
  ],
  "gates": {
    "define": {
      "requiredBefore": ["panel", "spec", "goal-tree", "execute"]
    },
    "panel": {
      "requiredBefore": ["spec", "goal-tree", "execute"]
    }
  },
  "paths": {
    "define": ".agentflow/define",
    "panel": ".agentflow/panel",
    "panelManifest": ".agentflow/panel/manifest.json",
    "panelFileTree": ".agentflow/panel/file-tree.json",
    "panelLanguages": ".agentflow/panel/languages.json",
    "panelSymbols": ".agentflow/panel/symbols.json",
    "panelRelations": ".agentflow/panel/relations.json",
    "panelDiagnostics": ".agentflow/panel/diagnostics.json",
    "panelGit": ".agentflow/panel/git.json",
    "panelTests": ".agentflow/panel/tests.json",
    "panelSearch": ".agentflow/panel/search",
    "panelContextPacks": ".agentflow/panel/context-packs",
    "panelSnapshots": ".agentflow/panel/snapshots",
    "panelIndex": ".agentflow/panel/index",
    "spec": ".agentflow/spec",
    "goalTree": ".agentflow/goal-tree",
    "execute": ".agentflow/execute",
    "output": ".agentflow/output",
    "state": ".agentflow/state"
  },
  "compat": {
    "legacyGraphOutput": ".agentflow/output/graph",
    "legacyGraphCanonical": ".agentflow/graph"
  }
}
```

不要写：

```json
"panelOutput": ".agentflow/panel/output"
```

因为 V1 不使用 `panel/output/`。

---

# 11. 兼容策略

## 11.1 旧 Graph 输出路径

旧路径：

```text
.agentflow/output/graph/
```

008.4 要迁移 / 兼容到：

```text
.agentflow/panel/
```

策略：

```text
新写入只写 .agentflow/panel/**
旧路径只读兼容
如果旧路径存在，Panel prepare 可以读取旧 manifest / index 做迁移
迁移后不删除旧路径
workspace-manifest.json 记录 compat.legacyGraphOutput
```

---

## 11.2 008.3 可能创建过的 `.agentflow/graph/`

如果存在：

```text
.agentflow/graph/
```

也需要兼容迁移到：

```text
.agentflow/panel/
```

策略：

```text
copy compatible artifacts
record warning
do not delete old .agentflow/graph/
workspace-manifest.json 记录 compat.legacyGraphCanonical
```

---

## 11.3 旧 Graph 命名

如果当前代码中还存在：

```text
GraphStatus
GraphManifest
GraphContextPack
prepare_project_graph
```

008.4 要做命名收口。

目标：

```text
PanelStatus
PanelManifest
PanelContextPack
prepare_project_panel
```

可以保留 deprecated alias，但新代码和 UI 必须使用 Panel。

---

# 12. Rust 模块设计

## 12.1 推荐方案：重命名 crate

当前如果已有：

```text
crates/graph
```

008.4 建议重命名为：

```text
crates/panel
```

package：

```text
agentflow-panel
```

理由：

```text
Panel 是新 canonical domain
这块后续变动少，应该一次性收口
```

如果为了降低 Git diff 风险，也可以先保留目录名 `crates/graph`，但 package 和 public API 改为 `agentflow-panel`。

本需求偏向完整收口：

```text
crates/graph -> crates/panel
```

---

## 12.2 建议结构

```text
crates/panel/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── manager.rs
    ├── model.rs
    ├── storage.rs
    ├── scanner.rs
    ├── watcher/
    ├── file_tree.rs
    ├── languages.rs
    ├── symbols.rs
    ├── relations.rs
    ├── diagnostics.rs
    ├── git.rs
    ├── tests.rs
    ├── search.rs
    ├── context_pack.rs
    ├── snapshots.rs
    └── migration.rs
```

---

# 13. Tauri commands

旧 commands：

```text
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

新 commands：

```text
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

兼容策略：

```text
旧 graph command 可以暂时作为 alias 调新 panel command
新 UI 只能用 panel command
```

---

# 14. Desktop UI

## 14.1 UI 命名

所有用户可见文案从：

```text
Graph
代码地图
```

统一为：

```text
Panel
项目现场
工作现场
```

中文建议统一：

```text
项目现场
```

状态通道建议：

```text
工作现场
```

---

## 14.2 Project 页面展示

Panel 状态可以显示：

```text
Panel status
files
languages
symbols
diagnostics
tests
git dirty
watcher backend
active snapshot
```

---

## 14.3 Project File Reader 联动

Project File Reader 的推荐文件来源改成：

```text
Panel Context Pack
```

不再叫：

```text
Graph Context Pack
```

---

# 15. Panel Context Pack

Context Pack 不再叫 Graph Context Pack，而叫：

```text
Panel Context Pack
```

它可以来源于：

```text
requirement intake result
SPEC draft
Approved SPEC
Goal Tree Issue
AgentRun preflight
Audit request
```

字段：

```json
{
  "version": "panel-context-pack.v1",
  "contextPackId": "ctx-001",
  "projectRoot": "/path/to/project",
  "source": {
    "type": "spec-draft",
    "id": "spec-draft-001"
  },
  "panelSnapshotId": "panel-snapshot-001",
  "recommendedFiles": [],
  "recommendedSymbols": [],
  "recommendedTests": [],
  "impactHints": [],
  "warnings": []
}
```

---

# 16. Panel Snapshot

每次完整 index 后生成：

```text
.agentflow/panel/snapshots/<snapshot-id>.json
```

snapshot 记录：

```text
manifest summary
worktree state
git state
file hashes / versions
diagnostics summary
test summary
timestamp
```

用途：

```text
SPEC 可引用 Panel Snapshot
Goal Tree materialization 可引用 Panel Snapshot
未来 AgentRun / Audit 可引用 Panel Snapshot
```

---

# 17. Panel Git Status

V1 必须实现：

```text
isGitRepo
branch
headSha
dirty
modifiedFiles
untrackedFiles
deletedFiles
agentflowProtected
```

不要执行危险命令，只允许只读 Git 查询：

```text
git status --porcelain
git rev-parse --abbrev-ref HEAD
git rev-parse HEAD
git check-ignore
```

如果没有 Git：

```text
isGitRepo = false
warnings += "Project is not a Git repository"
```

不自动：

```text
git init
```

---

# 18. Panel Diagnostics

V1 必须实现：

```text
scanner diagnostics
parser diagnostics
unsupported file diagnostics
large file diagnostics
symlink diagnostics
watcher degraded diagnostics
```

不接 LSP。

未来再接：

```text
LSP diagnostics
type checker diagnostics
test diagnostics
```

---

# 19. Panel Tests Detection

V1 必须实现测试发现：

```text
test file patterns
test directories
package manager test scripts
language-specific test hints
source -> likely tests relation
```

示例：

```text
Rust:
  *_test.rs
  tests/
  cargo test

TypeScript:
  *.test.ts
  *.spec.ts
  package.json scripts.test

Python:
  test_*.py
  *_test.py
  pytest / unittest

Go:
  *_test.go
  go test ./...

Java/Kotlin:
  src/test/
  gradle test

Swift:
  Tests/
  swift test

Dart/Flutter:
  test/
  flutter test
```

Panel 不运行测试，只发现测试。

---

# 20. Panel Search

V1 搜索能力：

```text
file search
symbol search
content snippet search
test search
```

输入：

```ts
search_project_panel(projectRoot, query, filters)
```

filters：

```text
kind=file|symbol|test|content
language
pathPrefix
limit
```

---

# 21. Panel Preflight

新增：

```text
panel_preflight(projectRoot)
```

检查：

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

Panel preflight 不检查：

```text
SPEC
Goal Tree
AgentRun
```

---

# 22. Watcher

沿用 PR #6 的 OS native watcher 思路，但统一命名为 Panel watcher。

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

---

# 23. 迁移策略

## 23.1 从旧 Graph 输出迁移

如果存在：

```text
.agentflow/output/graph/
```

则：

```text
创建 .agentflow/panel/
尝试复制 / 转换旧 manifest / context-packs / index 到 panel
如果失败，不阻塞，记录 warning
旧路径保留，不删除
```

---

## 23.2 从 `.agentflow/graph/` 迁移

如果 008.3 曾经创建过：

```text
.agentflow/graph/
```

也需要迁移到：

```text
.agentflow/panel/
```

策略：

```text
copy compatible artifacts
record warning
do not delete old .agentflow/graph/
```

---

# 24. 008.4 不做什么

不做：

```text
不写 SPEC
不写 Goal Tree
不启动 AgentRun
不写用户源码
不执行测试
不执行构建
不创建 PR
不调用模型
不做 LSP diagnostics
不做 CRDT
不做多人协作
不实现 Review Changes UI
不实现 checkpoint restore
```

---

# 25. 写入边界

允许写：

```text
.agentflow/panel/**
```

允许读取兼容路径：

```text
.agentflow/output/graph/**
.agentflow/graph/**
```

但不再新写：

```text
.agentflow/output/graph/**
.agentflow/graph/**
.agentflow/inspect/**
.agentflow/panel/output/**
```

禁止写：

```text
用户源码
.agentflow/spec/**
.agentflow/goal-tree/**
.agentflow/execute/**
.agentflow/output/evidence/**
.agentflow/output/audit/**
.agentflow/output/release/**
```

---

# 26. 验收标准

```text
- [ ] 新增 docs/requirements/008-4-project-panel-v1.md。
- [ ] Graph canonical name 改为 Panel。
- [ ] `.agentflow/panel/` 是新的 canonical path。
- [ ] Panel 产物直接写入 `.agentflow/panel/**`。
- [ ] 不创建 `.agentflow/panel/output/`。
- [ ] 不创建 `.agentflow/inspect/`。
- [ ] workspace-manifest.json 中 paths.panel = `.agentflow/panel`。
- [ ] workspace-manifest.json 中包含 paths.panelManifest。
- [ ] workspace-manifest.json 中包含 paths.panelFileTree。
- [ ] workspace-manifest.json 中包含 paths.panelLanguages。
- [ ] workspace-manifest.json 中包含 paths.panelSymbols。
- [ ] workspace-manifest.json 中包含 paths.panelRelations。
- [ ] workspace-manifest.json 中包含 paths.panelDiagnostics。
- [ ] workspace-manifest.json 中包含 paths.panelGit。
- [ ] workspace-manifest.json 中包含 paths.panelTests。
- [ ] workspace-manifest.json 中包含 paths.panelSearch。
- [ ] workspace-manifest.json 中包含 paths.panelContextPacks。
- [ ] workspace-manifest.json 中包含 paths.panelSnapshots。
- [ ] workspace-manifest.json 中包含 paths.panelIndex。
- [ ] workspace-manifest.json 中 compat.legacyGraphOutput = `.agentflow/output/graph`。
- [ ] workspace-manifest.json 中 compat.legacyGraphCanonical = `.agentflow/graph`。
- [ ] 若存在 `.agentflow/output/graph/`，prepare 会迁移 / 兼容读取。
- [ ] 若存在 `.agentflow/graph/`，prepare 会迁移 / 兼容读取。
- [ ] 新增 / 迁移 `crates/panel` 或 package `agentflow-panel`。
- [ ] 新 public API 使用 Panel 命名。
- [ ] 旧 Graph API 仅作为 deprecated alias。
- [ ] Tauri 新 commands 使用 Panel 命名。
- [ ] Desktop UI 使用 “Panel / 项目现场 / 工作现场”。
- [ ] `manifest.json` 生成在 `.agentflow/panel/manifest.json`。
- [ ] `file-tree.json` 生成在 `.agentflow/panel/file-tree.json`。
- [ ] `languages.json` 生成在 `.agentflow/panel/languages.json`。
- [ ] `symbols.json` 生成在 `.agentflow/panel/symbols.json`。
- [ ] `relations.json` 生成在 `.agentflow/panel/relations.json`。
- [ ] `diagnostics.json` 生成在 `.agentflow/panel/diagnostics.json`。
- [ ] `git.json` 生成在 `.agentflow/panel/git.json`。
- [ ] `tests.json` 生成在 `.agentflow/panel/tests.json`。
- [ ] `search/file-index.json` 生成。
- [ ] `search/symbol-index.json` 生成。
- [ ] `search/content-index.json` 生成。
- [ ] `context-packs/` 可生成 Panel Context Pack。
- [ ] `snapshots/` 可生成 Panel Snapshot。
- [ ] `index/panel.db` 或等效 index 存在。
- [ ] Panel watcher 默认 OS native。
- [ ] Panel watcher fallback degraded。
- [ ] Panel 不写用户源码。
- [ ] Panel 不执行测试。
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

# 27. 验证命令

```bash
cargo fmt --check
cargo test -p agentflow-panel
cargo test -p agentflow-desktop
cargo test
npm --prefix apps/desktop run build
git diff --check
```

如果保留兼容 graph crate alias：

```bash
cargo test -p agentflow-graph
```

---

# 28. PR 说明要求

PR 描述必须说明：

```text
1. 为什么 Graph 改名为 Panel。
2. Panel 的职责是什么。
3. Panel 不负责什么。
4. 新 canonical path 是 `.agentflow/panel/`。
5. 为什么不使用 `.agentflow/panel/output/`。
6. 如何兼容 `.agentflow/output/graph/`。
7. 如何兼容 `.agentflow/graph/`。
8. 是否迁移旧数据：迁移 / 兼容读取，但不删除旧路径。
9. 哪些 Tauri commands 改名。
10. 哪些旧 Graph API 保留 alias。
11. Panel 文件有哪些。
12. 是否写用户源码：必须说明没有。
13. 是否执行测试命令：必须说明没有。
14. 是否写 SPEC / Goal Tree / AgentRun：必须说明没有。
15. 验证命令和结果。
```

---

# 29. Codex 执行指令

```md
请执行 008.4 - Project Panel V1。

目标：
将 AgentFlow 的 Graph 升级并改名为 Project Panel。Panel 是 Agent 的项目工作现场层，不只是代码图谱。它必须一次性收口文件现场、语言现场、符号现场、关系现场、诊断现场、Git 现场、测试现场、搜索索引、Context Pack、Snapshot 和 watcher 状态。新的 canonical path 是 `.agentflow/panel/`，Panel 产物必须直接写入 `.agentflow/panel/**`。

必须遵守：
1. Graph canonical name 改为 Panel。
2. `.agentflow/panel/` 是新的 canonical path。
3. Panel 产物直接写在 `.agentflow/panel/**` 下。
4. 不创建 `.agentflow/panel/output/`。
5. 不创建 `.agentflow/inspect/`。
6. 兼容旧 `.agentflow/output/graph/`，但新写入走 `.agentflow/panel/`。
7. 如果存在 `.agentflow/graph/`，也兼容迁移到 `.agentflow/panel/`。
8. Panel 是项目工作现场，不是需求源。
9. Panel 不写 SPEC。
10. Panel 不写 Goal Tree。
11. Panel 不启动 AgentRun。
12. Panel 不写用户源码。
13. Panel 不执行测试。
14. Panel 不调用模型。
15. Panel 不创建 PR。
16. 旧 Graph API 只能作为 deprecated alias。
17. 新 UI / 新文档 / 新 API 使用 Panel 命名。

实现范围：
- 新增 docs/requirements/008-4-project-panel-v1.md。
- 将 graph domain rename / wrap 为 panel domain。
- 新增 `.agentflow/panel/` layout。
- 生成 panel manifest、file-tree、languages、symbols、relations、diagnostics、git、tests。
- 生成 search/file-index、search/symbol-index、search/content-index。
- 支持 panel context-packs。
- 支持 panel snapshots。
- 支持 panel/index/panel.db 或等效 index。
- 支持 panel watcher status。
- 支持 OS native watcher + fallback degraded。
- 支持 old graph output migration / compatibility。
- Tauri commands 增加 Panel 命名。
- Desktop UI 改为 Panel / 项目现场 / 工作现场。
- Project File Reader 推荐来源改为 Panel Context Pack。
- Workspace manifest paths.graph 改为 paths.panel。
- Workspace manifest 不增加 panelOutput。
- verification 更新。

验证命令：
- cargo fmt --check
- cargo test -p agentflow-panel
- cargo test -p agentflow-desktop
- cargo test
- npm --prefix apps/desktop run build
- git diff --check
```

---

# 30. 完成定义

完成后，AgentFlow 的现场层应该变成：

```text
Panel
= 项目工作现场

.agentflow/panel/
= 文件 + 语言 + 符号 + 关系 + Git + 诊断 + 测试 + 搜索 + Context Pack + Snapshot
```

Graph 不再是产品主概念，只作为 legacy / internal compatibility。

最终一句话：

> **008.4 把 Graph 升级成 Panel：Panel 是 Agent 的完整项目工作现场，不只是代码索引。Panel 的所有产物直接放在 `.agentflow/panel/` 下；它服务 Intake、SPEC、Goal Tree、TDD、Build、Release、Audit，但不承载需求、目标树、执行结果和审计报告。**
