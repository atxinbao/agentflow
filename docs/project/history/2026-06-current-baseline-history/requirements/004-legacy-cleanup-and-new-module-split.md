# 004 - Legacy Cleanup and New Module Split

创建日期：2026-06-01  
执行者：Codex

## 用户目标

AgentFlow 当前已经完成了三个新的底座能力：

```text
001 Project Workspace Manager
002 Graph / Graph Watcher
003 Project File Reader
```

但是仓库里仍然保留了大量旧需求体系下的代码，包括旧的：

```text
Goal Protocol
Product Feature
Team / Project / Milestone / Issue
GoalLoop
Eligibility / Lease
Run / Verify / Review
Project Closure / Code Audit / Docs Refresh
Evidence / Saved View / SQLite Index
```

这些旧代码属于 2026-05 归档前的旧流程。新的 AgentFlow 产品工作流还没有重新定义 Goal / Milestone / Issue / AgentRun，所以这些旧代码不能继续作为新产品主干。

本需求的目标是：

```text
旧需求代码隔离，新需求代码拆分，为下一阶段 Goal Tree 做准备。
```

大白话：

> 现在不是继续堆功能，而是先把旧流程代码从主干里降权、隔离出来。  
> 同时把已经确认的新模块：Project Workspace、Graph、Project File Reader 拆清楚。  
> 这样下一阶段 Goal Tree 才不会长在旧需求代码上。

---

## 背景

当前 Rust workspace 只有四块：

```text
crates/agentflow-core
crates/agentflow-cli
crates/graph
apps/desktop/src-tauri
```

其中：

```text
crates/agentflow-core/src/lib.rs
```

仍然是一个超大核心文件，包含大量旧状态模型和旧流程函数。

当前 CLI 也仍然暴露旧流程命令：

```text
goal
feature
team
milestone
issue
run
verify
review
index
view
update
metrics
eligibility
lease
project closure
project code-audit
project docs-refresh
project-seed
issue-link
review-assistant
state
```

这些命令和代码多数来自旧需求体系。

当前 `docs/requirements/README.md` 已经明确：

```text
后续开发的唯一入口是 docs/requirements/
旧 Workflow Control、旧 Product Feature、旧 Project Closure / Audit / Docs Refresh、
旧 GoalLoop / Eligibility / Lease / Evidence 自动推进都不继承，
除非新需求重新明确。
```

因此，这次清理要把旧代码变成：

```text
legacy compatibility layer
```

而不是继续放在新产品主干里。

---

## 一句话定义

> **004 Legacy Cleanup and New Module Split 是一轮代码结构清理，不新增产品功能。它负责把旧需求代码隔离到 legacy，把新需求代码拆成清晰模块，为后续 Goal Tree V1 做准备。**

---

## 范围

本需求包含 6 个清理范围：

```text
1. agentflow-core legacy quarantine
2. agentflow-cli legacy isolation
3. Tauri Desktop command/module split
4. Project Files backend split
5. Graph watcher split
6. Project File Reader frontend + types split
```

---

## 非目标

本需求不做以下事情：

```text
不新增 Goal / Milestone / Issue 新流程
不定义新的 AgentRun
不启动 Agent
不调用模型
不执行项目命令
不修改用户项目源码
不改变 Tauri command 对外名称
不改变 Desktop 只读边界
不删除旧代码，除非能证明无引用且测试通过
不把旧 CLI 直接改名到 legacy 子命令，除非另有确认
不新增 Project File Reader 功能
不新增 Graph 功能
不做 UI 大改版
```

---

# 1. 清理原则

## 1.1 旧代码先隔离，不直接硬删

旧代码可能还被 Desktop snapshot、CLI、测试或已有 read model 使用，所以第一轮不直接大删。

处理方式：

```text
旧代码进入 legacy/
旧 read-only 兼容能力进入 active/transitional read model
通用工具进入 shared/
```

## 1.2 新需求代码必须独立成模块

当前已经明确的新需求是：

```text
Project Workspace Manager
Graph
Project File Reader
```

这些要形成清晰的模块边界，不能继续散落在单个大文件里。

## 1.3 不改变外部行为

这轮是清理，不是功能改版。

必须保持：

```text
Tauri command 名称不变
前端主要行为不变
Desktop 只读边界不变
Graph 对外 API 不变
Project File Reader 对外行为不变
CLI 旧命令暂时不变
```

---

# 2. agentflow-core legacy quarantine

## 当前问题

当前：

```text
crates/agentflow-core/src/lib.rs
```

包含大量旧模型和旧函数，既有旧工作流，也有当前 Desktop 仍然需要的 read-only snapshot。

这会导致：

```text
新功能容易误依赖旧流程
代码边界不清晰
下一阶段 Goal Tree 容易长在旧模型上
```

## 目标结构

将 `crates/agentflow-core/src/lib.rs` 拆成：

```text
crates/agentflow-core/src/
├── lib.rs
│
├── active/
│   ├── mod.rs
│   ├── desktop_snapshot.rs
│   ├── local_metrics.rs
│   ├── local_project_model.rs
│   ├── local_search.rs
│   └── boundary.rs
│
├── legacy/
│   ├── mod.rs
│   ├── goal_protocol.rs
│   ├── product_feature.rs
│   ├── team_project_milestone_issue.rs
│   ├── workflow_control.rs
│   ├── run_verify_review.rs
│   ├── eligibility_lease.rs
│   ├── project_closure.rs
│   ├── project_audit_docs_refresh.rs
│   ├── evidence.rs
│   ├── saved_view.rs
│   └── sqlite_index.rs
│
└── shared/
    ├── mod.rs
    ├── fs_paths.rs
    ├── json_io.rs
    ├── markdown.rs
    ├── ids.rs
    └── time.rs
```

---

## 2.1 `active/`

`active/` 只放当前 Desktop 仍然需要的 read-only transitional capability。

包括：

```text
read_desktop_workbench_snapshot
read_local_metrics_snapshot
read_local_project_model_snapshot
read_project_milestone_issue_view_model_snapshot
read_local_search_snapshot
WorkbenchBoundary
```

这些是为了当前 Desktop 不崩，不代表新流程已经确认。

模块注释：

```rust
//! Active transitional read models.
//!
//! These APIs exist so the current Desktop can keep rendering read-only
//! snapshots while the new AgentFlow workflow is being defined.
//!
//! New write flows must not be added here without a new requirement.
```

---

## 2.2 `legacy/`

`legacy/` 放旧需求体系代码。

包括：

```text
Goal Protocol
Product Feature
Team / Project / Milestone / Issue 旧模型
IssueContract
AgentRun
GoalLoop
Run / Verify / Review
Eligibility / Lease
Project Closure
Project Code Audit
Project Docs Refresh
Evidence / Review / Update
Saved View
旧 SQLite index
```

每个 legacy 模块顶部必须加注释：

```rust
//! Legacy compatibility module.
//!
//! This module belongs to the archived 2026-05 workflow/product-feature system.
//! It is kept only for compatibility and migration.
//! New AgentFlow requirements must not depend on this module unless a new
//! requirement explicitly re-authorizes it.
```

---

## 2.3 `shared/`

`shared/` 只放无业务立场的基础工具。

可以包括：

```text
路径处理
JSON 读写
Markdown 写入
ID 生成
时间函数
通用文件工具
```

不允许放：

```text
Goal
Issue
AgentRun
Lease
Evidence
Project Closure
Product Feature
```

---

## 2.4 `lib.rs`

`lib.rs` 应该变薄，只负责模块导出。

示例：

```rust
pub mod active;
pub mod legacy;
pub mod shared;

pub use active::{
    read_desktop_workbench_snapshot,
    read_local_metrics_snapshot,
    read_local_project_model_snapshot,
    read_project_milestone_issue_view_model_snapshot,
    read_local_search_snapshot,
    WorkbenchBoundary,
};
```

如果 CLI 旧命令暂时还依赖 legacy，可以先保留必要的 `pub use legacy::*`，但必须加注释：

```rust
// Temporary compatibility export for legacy CLI.
// Do not use from new requirements.
```

---

## 验收标准

```text
- [ ] agentflow-core/src/lib.rs 不再是巨型业务文件。
- [ ] active/ 只包含当前 Desktop read-only snapshot/read-model。
- [ ] legacy/ 包含旧 Workflow / Product Feature / GoalLoop / Run / Verify / Review / Closure 等旧流程。
- [ ] shared/ 只包含通用工具。
- [ ] 所有 legacy 模块顶部都有 legacy compatibility 注释。
- [ ] 新需求代码不直接依赖 legacy，除非通过明确 compatibility wrapper。
- [ ] cargo test -p agentflow-core 通过。
```

---

# 3. agentflow-cli legacy isolation

## 当前问题

当前 CLI 暴露大量旧流程命令：

```text
goal
feature
team
milestone
issue
run
verify
review
eligibility
lease
project closure
project code-audit
project docs-refresh
```

这些命令属于旧需求体系，但仍在 `main.rs` 主干中。

## 目标结构

将 CLI 拆成：

```text
crates/agentflow-cli/src/
├── main.rs
├── args.rs
├── active.rs
├── legacy.rs
└── print.rs
```

---

## 3.1 `main.rs`

只做入口：

```rust
fn main() -> Result<()> {
    agentflow_cli::run()
}
```

或者：

```rust
fn main() -> Result<()> {
    let cli = Cli::parse();
    dispatch(cli)
}
```

但 `main.rs` 不再承载大量 match 业务逻辑。

---

## 3.2 `args.rs`

放 clap 参数定义：

```text
Cli
Command
GoalCommand
FeatureCommand
TeamCommand
MilestoneCommand
IssueCommand
IndexCommand
ProjectCommand
StateCommand
```

如果暂时保留旧命令名，也都放在这里。

---

## 3.3 `legacy.rs`

放旧命令实现：

```text
goal bootstrap / check / next
feature create / status / next
team create
project create / closure / code-audit / docs-refresh
milestone create
issue create
plan
run --dry-run
verify
review
index rebuild
view save/show
update summary
metrics
eligibility
lease
project-seed
issue-link
review-assistant
state check
```

模块顶部加注释：

```rust
//! Legacy CLI commands from archived 2026-05 workflow.
//!
//! These commands are kept for compatibility only.
//! New AgentFlow flows must not be added here.
```

---

## 3.4 命令名策略

本轮采用保守策略：

```text
旧 CLI 命令名暂时不改
实现移动到 legacy.rs
help / module 注释标记 legacy
```

不做：

```text
不把旧命令强制改成 agentflow legacy xxx
不删除旧命令
不新增新 CLI 产品功能
```

原因：

```text
避免一次性破坏已有测试和已有调用。
等 Goal Tree 新 CLI 定义后，再决定旧命令隐藏、删除或迁移到 legacy 子命令。
```

---

## 验收标准

```text
- [ ] crates/agentflow-cli/src/main.rs 明显变薄。
- [ ] 旧命令实现进入 legacy.rs。
- [ ] clap args 与 dispatch 逻辑拆开。
- [ ] 输出 helper 进入 print.rs。
- [ ] CLI 旧命令暂时仍可编译。
- [ ] 不新增新 CLI 功能。
- [ ] cargo test -p agentflow-cli 通过。
```

---

# 4. Tauri Desktop command/module split

## 当前问题

当前 Tauri Desktop 里同时存在：

```text
legacy core snapshot commands
graph commands
project file commands
project workspace commands
```

其中部分 command 来自 `agentflow_core` 旧 read model。

## 目标结构

拆成：

```text
apps/desktop/src-tauri/src/
├── main.rs
│
├── commands/
│   ├── mod.rs
│   ├── legacy_core.rs
│   ├── graph.rs
│   ├── project_files.rs
│   └── project_workspace.rs
│
├── project_files/
│   ├── mod.rs
│   ├── commands.rs
│   ├── model.rs
│   ├── path_guard.rs
│   ├── directory.rs
│   ├── content.rs
│   ├── search.rs
│   ├── range.rs
│   ├── mime.rs
│   └── tests.rs
│
└── project_workspace/
    ├── mod.rs
    ├── commands.rs
    ├── model.rs
    ├── prepare.rs
    ├── dedupe.rs
    ├── git.rs
    ├── ignore.rs
    └── remove.rs
```

---

## 4.1 `commands/legacy_core.rs`

放当前仍调用 `agentflow_core` 的旧 read-only command：

```text
load_workbench_snapshot
load_metrics_snapshot
load_project_model_snapshot
load_project_milestone_issue_view_model_snapshot
load_search_snapshot
```

模块顶部加注释：

```rust
//! Transitional legacy read-model commands.
//!
//! These commands wrap agentflow-core legacy/transitional snapshots so the
//! current Desktop can keep rendering while the new workflow is being defined.
//!
//! Do not add new write flows here.
```

---

## 4.2 `commands/graph.rs`

放 Graph command wrapper：

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

---

## 4.3 `commands/project_files.rs`

放 Project File Reader command wrapper：

```text
load_project_files_snapshot
load_project_file_content
load_project_directory_page
search_project_files
load_project_file_text_range
choose_existing_project_folder
```

---

## 4.4 `commands/project_workspace.rs`

放 Project Workspace command wrapper：

```text
prepare_local_project_workspace
```

未来 Add / Remove / Deduplicate Project command 也放这里。

---

## 验收标准

```text
- [ ] Tauri command 名称全部不变。
- [ ] legacy core read-model command 已移动到 commands/legacy_core.rs。
- [ ] graph command wrapper 与 project files / workspace command wrapper 分离。
- [ ] main.rs 只负责注册 command，不承载业务逻辑。
- [ ] npm --prefix apps/desktop run build 通过。
- [ ] cargo test 通过。
```

---

# 5. Project Files backend split

## 当前问题

当前 Project Files 后端已经包含：

```text
文件快照
文件内容读取
目录分页
文件搜索
大文本 range
路径逃逸防护
symlink
MIME / language
binary fallback
测试
```

这些逻辑不应该继续放在一个大文件里。

## 目标结构

```text
apps/desktop/src-tauri/src/project_files/
├── mod.rs
├── commands.rs
├── model.rs
├── path_guard.rs
├── directory.rs
├── content.rs
├── search.rs
├── range.rs
├── mime.rs
└── tests.rs
```

---

## 5.1 `commands.rs`

只放 Tauri command：

```text
load_project_files_snapshot
load_project_file_content
load_project_directory_page
search_project_files
load_project_file_text_range
choose_existing_project_folder
```

---

## 5.2 `model.rs`

放 DTO：

```text
ProjectFilesSnapshot
ProjectFileEntry
ProjectFileChild
ProjectFileContent
ProjectDirectoryPage
ProjectFileSearchSnapshot
ProjectFileSearchResult
ProjectFileTextRange
```

---

## 5.3 `path_guard.rs`

放：

```text
resolve_agentflow_project_root
sanitize_project_relative_path
project_file_node
symlink root 内 / root 外检查
relative_project_path
```

---

## 5.4 `directory.rs`

放：

```text
read_project_file_entries
project_file_entry_from_path
read_project_file_children
read_project_file_child_entries
project_directory_child_count
load directory page helper
Source View / All View filter
```

---

## 5.5 `content.rs`

放：

```text
read_project_file_content
binary fallback
dataUrl fallback
unsupported reason
large text preview
```

---

## 5.6 `search.rs`

放：

```text
search_project_file_entries
parse_project_file_search_query
search_project_file_entries_in_directory
```

---

## 5.7 `range.rs`

放：

```text
read_project_file_text_range
BufReader line range loading
```

---

## 5.8 `mime.rs`

放：

```text
file_language
file_mime_type
preview_data_url
hex_preview
binary_unsupported_reason
```

---

## 验收标准

```text
- [ ] 所有 Project Files command 名称不变。
- [ ] DTO 字段不变。
- [ ] 路径逃逸测试仍通过。
- [ ] symlink root 内 / root 外测试仍通过。
- [ ] directory page 测试仍通过。
- [ ] search 测试仍通过。
- [ ] text range 测试仍通过。
- [ ] binary fallback 测试仍通过。
- [ ] cargo test 通过。
```

---

# 6. Project Workspace backend split

## 当前问题

Project Workspace 是新需求底座，但目前还没有形成足够清晰的内部结构。

## 目标结构

```text
apps/desktop/src-tauri/src/project_workspace/
├── mod.rs
├── commands.rs
├── model.rs
├── prepare.rs
├── dedupe.rs
├── git.rs
├── ignore.rs
└── remove.rs
```

---

## 模块职责

```text
commands.rs
= Tauri command wrapper

model.rs
= workspace config / result DTO

prepare.rs
= 创建 .agentflow 目录结构

dedupe.rs
= canonical path / git root / workspace id 去重

git.rs
= git root / git metadata / git dir

ignore.rs
= 写入 .git/info/exclude，保护 .agentflow/

remove.rs
= 移除项目记录，不删除源码
```

本轮如果某些能力还没实现，不要补新功能，只先保留空模块或 TODO 注释。

---

## 验收标准

```text
- [ ] prepare_local_project_workspace 行为不变。
- [ ] .agentflow 创建结构不变。
- [ ] .git/info/exclude 保护行为不变。
- [ ] 不新增 Add/Remove UI 功能。
- [ ] cargo test 通过。
```

---

# 7. Graph watcher split

## 当前问题

Graph watcher 已经从 fingerprint polling 升级为 OS native watcher，并包含 fallback、event filter、debounce、state registry、backend/detail 状态。

这一块已经是新需求代码，不是 legacy，但内部应拆清楚。

## 目标结构

```text
crates/graph/src/watcher/
├── mod.rs
├── native.rs
├── fallback.rs
├── filter.rs
├── state.rs
├── debounce.rs
└── tests.rs
```

---

## 模块职责

```text
mod.rs
= 对外 API：ensure_graph_watcher / watcher_status / watcher_backend / watcher_detail

native.rs
= notify::recommended_watcher / RecursiveMode::Recursive / OS native event handling

fallback.rs
= fingerprint fallback watcher

filter.rs
= should_ignore_graph_event / ignored entries

state.rs
= WatcherState / registry / snapshot / backend detail

debounce.rs
= event debounce / pending refresh / no concurrent indexing

tests.rs
= native smoke / fallback / filter / debounce tests
```

---

## 保持不变

```text
ensure_graph_watcher API 不变
watcher_status 不变
watcher_backend 不变
watcher_detail 不变
native watcher default 不变
fingerprint fallback 不变
ignored path filter 不变
degraded fallback 行为不变
```

---

## 验收标准

```text
- [ ] crates/graph/src/watcher.rs 不再是大文件。
- [ ] watcher 外部 API 不变。
- [ ] native watcher 行为不变。
- [ ] fallback 行为不变。
- [ ] ignored path filter 行为不变。
- [ ] cargo test -p agentflow-graph 通过。
```

---

# 8. Project File Reader frontend split

## 当前问题

Project File Reader 前端已经完成 V1，但模块内部变重：

```text
ProjectLocalFilesPage.tsx
ProjectFileBrowser.tsx
FileRendererRegistry.tsx
useProjectFiles.ts
projectFileTypes.ts
projectFileUtils.ts
```

承担了太多逻辑。

## 目标结构

```text
apps/desktop/src/features/project-files/
├── index.ts
├── ProjectLocalFilesPage.tsx
├── ProjectFiles.css
│
├── browser/
│   ├── ProjectFileBrowser.tsx
│   ├── projectFileBrowserRows.ts
│   └── recommendedFiles.ts
│
├── reader/
│   ├── ProjectFileReader.tsx
│   ├── FileRendererRegistry.tsx
│   └── renderers/
│       ├── MarkdownReader.tsx
│       ├── CodeReader.tsx
│       ├── LargeTextReader.tsx
│       ├── TableReader.tsx
│       ├── PdfReader.tsx
│       ├── MediaReader.tsx
│       ├── DocxReader.tsx
│       └── FallbackReaders.tsx
│
├── hooks/
│   ├── useProjectFiles.ts
│   ├── useProjectFileSearch.ts
│   ├── useProjectDirectoryPages.ts
│   ├── useProjectFileTextRange.ts
│   └── useProjectFileReaderState.ts
│
└── model/
    ├── projectFileTypes.ts
    ├── projectFileUtils.ts
    └── projectRecommendedFiles.ts
```

---

## 拆分原则

```text
ProjectLocalFilesPage.tsx
= 页面组合，不写复杂业务逻辑

ProjectFileBrowser.tsx
= 文件树 / 搜索框 / 视图切换 / 推荐区展示

recommendedFiles.ts
= 从 GraphContextPack / manifest 构造 ProjectRecommendedFile

LargeTextReader.tsx
= 大文本 range UI 和调用逻辑

useProjectFiles.ts
= 保留总入口，但拆出 search / directory pages / text range / persistence 子 hook

projectFileReaderState.ts
= localStorage 读写
```

---

## 验收标准

```text
- [ ] UI 行为不变。
- [ ] ProjectLocalFilesPage.tsx 明显变薄。
- [ ] recommended file 构造逻辑独立。
- [ ] LargeTextReader 独立。
- [ ] localStorage 读写独立。
- [ ] useProjectFiles 不再承担所有子逻辑。
- [ ] npm --prefix apps/desktop run build 通过。
```

---

# 9. Frontend types split

## 当前问题

`apps/desktop/src/types.ts` 已经承载太多类型：

```text
Workbench
Project Model
Project Files
Graph
Status
Goal / Issue view model
```

## 目标结构

新增：

```text
apps/desktop/src/types/
├── index.ts
├── projectFiles.ts
├── graph.ts
├── workbench.ts
├── projectModel.ts
└── status.ts
```

保留：

```text
apps/desktop/src/types.ts
```

内容改成：

```ts
export * from "./types";
```

这样不破坏现有 import：

```ts
import type { ProjectFileContent } from "../../types";
```

---

## 类型归属

```text
projectFiles.ts
= ProjectFilesSnapshot / ProjectFileContent / ProjectDirectoryPage / ProjectRecommendedFile

graph.ts
= GraphStatusSnapshot / GraphManifestSnapshot / GraphContextPack / GraphTestHint

workbench.ts
= DesktopWorkbenchSnapshot / Metrics / Boundary

projectModel.ts
= LocalProjectModelSnapshot / V1Project / V1Milestone / V1Issue

status.ts
= AgentStatusChannelItem / Status Tone / UI status types
```

---

## 验收标准

```text
- [ ] types.ts 不再持续膨胀。
- [ ] 不破坏现有 import。
- [ ] Project File 类型进入 projectFiles.ts。
- [ ] Graph 类型进入 graph.ts。
- [ ] Workbench / ProjectModel / Status 分离。
- [ ] npm --prefix apps/desktop run build 通过。
```

---

# 10. 文档更新

本轮需要新增或更新文档：

```text
docs/requirements/004-legacy-cleanup-and-new-module-split.md
docs/architecture/legacy-code-map.md
docs/architecture/current-module-boundaries.md
verification.md
```

---

## 10.1 `docs/architecture/legacy-code-map.md`

记录旧代码分布：

```text
旧命令
旧 core functions
旧数据文件路径
旧 Desktop read-model compatibility
哪些仍被引用
哪些只是 legacy
哪些未来可删除
```

---

## 10.2 `docs/architecture/current-module-boundaries.md`

记录清理后的边界：

```text
Project Workspace Manager
Graph
Project File Reader
Legacy Core Read Model
Legacy CLI
```

---

## 10.3 `verification.md`

记录：

```text
执行者
目标
移动了哪些文件
是否有行为变化
验证命令
结果
```

---

# 11. 建议开发切片

## Slice 1：Legacy Inventory + Requirement

目标：

```text
新增 004 需求文档
新增 legacy-code-map
标注 legacy 范围
```

验收：

```text
docs/requirements/README.md 更新
next-requirements.md 更新
不改行为
```

---

## Slice 2：agentflow-core legacy quarantine

目标：

```text
拆 agentflow-core/src/lib.rs
active / legacy / shared
```

验收：

```text
cargo test -p agentflow-core
cargo test
```

---

## Slice 3：CLI legacy isolation

目标：

```text
拆 agentflow-cli main
legacy command implementation
args / print helper
```

验收：

```text
cargo test -p agentflow-cli
cargo test
```

---

## Slice 4：Tauri backend split

目标：

```text
commands/
project_files/
project_workspace/
```

验收：

```text
cargo test
npm --prefix apps/desktop run build
```

---

## Slice 5：Graph watcher split

目标：

```text
watcher/native
watcher/fallback
watcher/filter
watcher/state
watcher/debounce
```

验收：

```text
cargo test -p agentflow-graph
cargo test
```

---

## Slice 6：Project File Reader frontend split

目标：

```text
browser/
reader/
hooks/
model/
```

验收：

```text
npm --prefix apps/desktop run build
```

---

## Slice 7：types split + docs

目标：

```text
types/
architecture docs
verification
```

验收：

```text
npm --prefix apps/desktop run build
git diff --check
```

---

# 12. 总验收标准

```text
- [ ] 旧需求代码全部进入 legacy 或 transitional active read-model。
- [ ] 新需求代码不直接依赖 legacy workflow。
- [ ] agentflow-core/src/lib.rs 不再是巨型业务文件。
- [ ] CLI 旧命令实现进入 legacy.rs。
- [ ] Tauri Desktop 对 agentflow_core 的调用被标记为 legacy read-model compatibility。
- [ ] Project Files 后端拆成独立模块。
- [ ] Project Workspace 后端拆成独立模块。
- [ ] Graph Watcher 拆成 native/fallback/filter/state/debounce。
- [ ] Project File Reader 前端拆成 browser/reader/hooks/model。
- [ ] types.ts 拆成领域类型并保留 barrel export。
- [ ] 所有 Tauri command 名称保持不变。
- [ ] Desktop 只读边界保持不变。
- [ ] CLI 旧命令暂时保持可编译。
- [ ] 不新增产品功能。
- [ ] 不改变用户行为。
- [ ] cargo fmt --check 通过。
- [ ] cargo test -p agentflow-core 通过。
- [ ] cargo test -p agentflow-cli 通过。
- [ ] cargo test -p agentflow-graph 通过。
- [ ] cargo test 通过。
- [ ] npm --prefix apps/desktop run build 通过。
- [ ] git diff --check 通过。
```

---

# 13. 验证命令

必须执行：

```bash
cargo fmt --check
cargo test -p agentflow-core
cargo test -p agentflow-cli
cargo test -p agentflow-graph
cargo test
npm --prefix apps/desktop run build
git diff --check
```

---

# 14. 交付说明要求

PR 说明必须包含：

```text
1. 哪些代码只是移动 / 拆分，没有行为变化。
2. 哪些模块被标记为 legacy。
3. 当前仍被 Desktop 兼容调用的 legacy read-model 有哪些。
4. 新需求代码拆分后的模块边界。
5. 是否有任何行为变化。
6. 如果有行为变化，必须明确说明原因。
7. 验证命令和结果。
```

---

# 15. Codex 执行指令

```md
请执行 004 - Legacy Cleanup and New Module Split。

目标：
旧需求代码隔离，新需求代码拆分，为下一阶段 Goal Tree 做准备。

重点：
- 旧 Workflow / Product Feature / GoalLoop / Run / Verify / Review / Closure 代码进入 legacy。
- 当前 Desktop 仍需读取的 snapshot/read-model 进入 active transitional read model。
- Project Workspace / Graph / Project File Reader 作为新需求底座拆成清晰模块。
- 不新增产品功能，不改变用户行为。

必须遵守：
1. 不新增 Goal / Milestone / Issue 新流程。
2. 不启动 Agent。
3. 不调用模型。
4. 不执行项目命令。
5. 不修改用户项目源码。
6. 不改变 Tauri command 对外名称。
7. 不改变 Desktop 只读边界。
8. 不删除旧代码，除非能证明没有任何引用且测试通过。
9. 旧代码先隔离到 legacy，不直接硬删。
10. CLI 旧命令暂时保持可编译。

清理范围：
1. agentflow-core legacy quarantine。
2. agentflow-cli legacy isolation。
3. Tauri command/module split。
4. Project Files backend split。
5. Project Workspace backend split。
6. Graph watcher split。
7. Project File Reader frontend split。
8. Frontend types split。
9. 架构文档和 verification 更新。

验证命令：
- cargo fmt --check
- cargo test -p agentflow-core
- cargo test -p agentflow-cli
- cargo test -p agentflow-graph
- cargo test
- npm --prefix apps/desktop run build
- git diff --check

交付时必须说明：
- 哪些只是移动 / 拆分，没有行为变化。
- 哪些是 legacy。
- 哪些是 active transitional read model。
- 新模块边界是什么。
- 是否有行为变化。
```

---

# 16. 完成定义

本需求完成后，AgentFlow 代码结构应该达到：

```text
旧流程代码被隔离
新需求代码边界清晰
Desktop 仍能运行
CLI 仍能编译
Graph 仍能工作
Project File Reader 仍保持只读
下一阶段 Goal Tree 不再需要踩在旧流程代码上
```

最终一句话：

> **004 不是功能开发，而是主干清理。完成后，AgentFlow 才真正具备进入 Goal Tree V1 的代码基础。**
