# 002 - Graph V1

创建日期：2026-06-01
执行者：Codex

## 用户目标

用户把本地 Project 添加到 AgentFlow 后，系统可以在后台自动整理该项目的代码现场，生成本地 Graph。

Graph V1 不是给用户手动刷图看的可视化产品，而是给后续 Agent 使用的项目现场地图服务。它让 Agent 在执行 Goal / Milestone / Issue 之前，可以先拿到项目里的文件、语言、符号、基础关系、推荐上下文、弱版影响提示和测试推荐。

一句话定义：

```text
Graph V1 = AgentFlow 的本地代码现场地图服务。
```

## 背景

当前第一个新需求是 [001 - Project Workspace Manager V0.2](001-add-local-project.md)，它负责把本地项目接入 AgentFlow，创建或复用 `.agentflow/` 本地运行目录，并保护 `.agentflow/` 不进入 Git / PR。

001 明确不理解代码项目本身，不检测技术栈、不分析代码库、不启动 Agent。

Graph V1 是 001 之后的第二个自然需求：

```text
001 Project Workspace Manager
= 把本地项目接进 AgentFlow

002 Graph V1
= 把本地项目的代码现场整理成 Agent 可用的地图
```

## 范围

Graph V1 做 6 件事：

- 文件索引
- 符号索引
- 基础依赖关系
- 文件 / 符号 / 片段搜索
- Context Pack 上下文包
- 弱版影响分析 + 测试推荐

### 文件索引

Graph 扫描 Project 根目录并记录项目文件。

需要识别：

- 源码文件
- 测试文件
- 文档文件
- 配置文件
- 构建文件
- 移动端工程文件
- 脚本文件
- 资源文件
- 二进制文件
- 生成文件

需要跳过：

- `.git/`
- `.agentflow/`
- `node_modules/`
- `target/`
- `dist/`
- `build/`
- `coverage/`
- `.cache/`
- `vendor/`
- `.idea/`
- `.vscode/`
- `.DS_Store`

文件记录至少包括：

- `path`
- `name`
- `extension`
- `language`
- `kind`
- `size_bytes`
- `line_count`
- `modified_at`
- `content_hash`
- `is_source`
- `is_test`
- `is_doc`
- `is_config`
- `is_generated`
- `is_ignored`

### 符号索引

Graph 从源码和结构化文档中提取主流符号。

符号类型包括：

- `class`
- `struct`
- `enum`
- `interface`
- `trait`
- `protocol`
- `function`
- `method`
- `constructor`
- `component`
- `module`
- `package`
- `namespace`
- `test`
- `route`
- `markdown_heading`
- `config_key`

符号记录至少包括：

- `name`
- `kind`
- `language`
- `file`
- `start_line`
- `end_line`
- `signature`
- `parent_symbol`
- `visibility`

### 基础依赖关系

Graph V1 不做完整调用链，不做 IDE 级类型推导。第一版只做基础关系：

- `file contains symbol`
- `symbol contains symbol`
- `file imports module`
- `symbol parent_of symbol`
- `test file test_of source file`
- `config file configures project/module`
- `file mentions symbol/query`
- `same directory relation`
- `same module relation`

目标是先知道“谁在哪、谁属于谁、谁引用了谁、哪个测试可能对应哪个源码”。

### 搜索能力

Graph 支持 Agent 查询：

- 文件
- 符号
- 代码片段
- 配置项
- Markdown 标题

搜索结果必须包含：

- `kind`
- `path`
- `title`
- `language`
- `symbolKind`
- `line`
- `snippet`
- `score`

### Context Pack

Context Pack 是 Graph V1 最重要的产物。

它的职责是：Agent 每次执行任务前，Graph 帮它准备一包“应该先看的东西”。

输入可以是：

- Goal
- Milestone
- Issue
- freeform query

输出包括：

- 推荐文件
- 推荐符号
- 推荐测试
- 相关配置
- 影响提示
- 测试提示
- 推荐原因
- 置信度

路径：

```text
.agentflow/output/graph/context-packs/<target-id>.json
```

### 弱版影响分析 + 测试推荐

Graph V1 提供启发式影响分析，不追求 100% 精确。

输入：

- 目标 query
- changed files
- recommended files

输出：

- 可能受影响的文件
- 可能受影响的符号
- 可能相关测试
- 建议优先跑的测试命令提示

依据：

- import/use 关系
- 同目录关系
- 同模块关系
- 测试文件匹配
- 同名文件匹配
- 符号名匹配
- 配置文件关系

## 非目标

- 不调用模型。
- 不启动 Agent。
- 不执行测试。
- 不修改源码。
- 不创建 Goal / Milestone / Issue。
- 不做完整调用链。
- 不做完整类型推导。
- 不做 IDE 级语义分析。
- 不做可视化知识图谱。
- 不做代码问答系统。
- 不做 MCP Server。
- 不提供用户手动刷新按钮。
- 不接 OpenSpec / Superpowers / gstack。
- 不创建远程 PR、GitHub issue 或 Linear issue。

Graph V1 只做：

- 本地索引
- 本地搜索
- 本地上下文包
- 本地影响提示
- 本地测试推荐

## 数据目录

Graph V1 的所有数据放到：

```text
.agentflow/output/graph/
├── graph.db
├── meta.json
├── context-packs/
└── exports/
    ├── manifest.json
    ├── files.jsonl
    ├── symbols.jsonl
    ├── relations.jsonl
    └── chunks.jsonl
```

说明：

- `graph.db` 是主数据库，给系统查询用。
- `meta.json` 记录 Graph 状态、版本、更新时间、Git HEAD、索引状态。
- `context-packs/` 保存给后续 AgentRun 使用的上下文包。
- `exports/` 用于调试和排查，不是主数据源。

规则：

- `.agentflow/output/graph/**` 不进 Git。
- `.agentflow/output/graph/**` 不进 PR。
- Graph V1 不写业务源码。
- Graph V1 不写根目录文档。
- Graph V1 不写 `.gitignore`。

## Rust 模块结构

新增 crate：

```text
crates/graph/
```

package 名：

```toml
[package]
name = "agentflow-graph"
```

建议结构：

```text
crates/
└── graph/
    ├── Cargo.toml
    └── src/
        ├── lib.rs
        ├── manager.rs
        ├── scanner.rs
        ├── watcher.rs
        ├── context_pack.rs
        ├── impact.rs
        ├── test_recommendation.rs
        ├── model/
        ├── db/
        ├── parser/
        └── preflight/
```

## 技术路线

### Rust 原生实现

Graph V1 不 fork、不嵌入、不依赖外部 codegraph CLI。

采用：

- Rust 原生 Graph 模块
- SQLite 本地数据库
- Tree-sitter 优先解析
- 轻量 extractor 兜底

原因：

- AgentFlow 当前核心是 Rust workspace。
- Graph 是内部基础设施。
- Graph 数据要统一进入 `.agentflow/output/graph/`。
- 后续要和 Desktop / AgentRun / Goal Tree 深度整合。

### Tree-sitter 优先

Graph V1 要覆盖主流代码项目，不能只靠正则。

实现策略：

```text
Tree-sitter parser 优先
-> 没有稳定 parser 的语言，用轻量 extractor 兜底
-> 所有提取结果统一写入 graph.db
```

V1 可以先建立 parser registry 和轻量 extractor，后续逐步补 Tree-sitter parser。

## 语言覆盖范围

### L1：深度结构索引

需要提取文件、符号、import/use、测试、基础关系：

- TypeScript / JavaScript
- Python
- Java
- Kotlin
- Swift
- Go
- Rust
- C / C++
- C#
- Dart

### L2：中度结构索引

先做函数 / 类 / 方法 / 脚本结构，不追求完整语义：

- Objective-C
- PHP
- Ruby
- SQL
- Shell / Bash
- PowerShell
- HTML
- CSS / SCSS / Sass

### L3：配置 / 文档结构索引

识别标题、依赖、脚本、入口、平台配置：

- Markdown
- JSON
- YAML
- TOML
- XML
- plist
- Gradle
- Maven
- `Cargo.toml`
- `package.json`
- `pyproject.toml`
- `go.mod`
- `pubspec.yaml`
- `Podfile`
- `Package.swift`
- `Dockerfile`
- `docker-compose.yaml`
- GitHub Actions workflow
- `AndroidManifest.xml`
- `Info.plist`

## 移动端项目支持

### Android

识别：

- `.kt`
- `.kts`
- `.java`
- `.cpp`
- `.c`
- `.h`
- `build.gradle`
- `build.gradle.kts`
- `settings.gradle`
- `AndroidManifest.xml`
- `res/**/*.xml`
- `proguard-rules.pro`

提取：

- Kotlin class / object / data class / interface / function
- Activity / Fragment / ViewModel / Repository
- Compose `@Composable`
- Java class / method
- JNI / NDK C++ 文件
- Manifest activity / service / permission
- Gradle module / dependency
- 测试文件

### iOS / Apple

识别：

- `.swift`
- `.h`
- `.m`
- `.mm`
- `.xcodeproj`
- `.xcworkspace`
- `Package.swift`
- `Podfile`
- `Info.plist`
- `*.storyboard`
- `*.xib`
- `*.entitlements`

提取：

- Swift class / struct / enum / protocol / extension
- SwiftUI View
- UIViewController
- App / Scene 入口
- Objective-C interface / implementation / category / protocol
- import
- 测试文件
- plist 配置

### Flutter

识别：

- `.dart`
- `pubspec.yaml`
- `analysis_options.yaml`
- `lib/`
- `test/`
- `android/`
- `ios/`

提取：

- class
- mixin
- extension
- function
- Widget
- StatefulWidget
- StatelessWidget
- `main()`
- route
- test
- pubspec dependencies

## Graph DB

Graph V1 使用 SQLite：

```text
.agentflow/output/graph/graph.db
```

### files

记录项目文件。

字段：

- `id`
- `path`
- `name`
- `extension`
- `language`
- `kind`
- `size_bytes`
- `line_count`
- `modified_at`
- `content_hash`
- `is_source`
- `is_test`
- `is_doc`
- `is_config`
- `is_generated`
- `is_ignored`

`kind` 可选：

- `source`
- `test`
- `doc`
- `config`
- `asset`
- `binary`
- `generated`
- `unknown`

### symbols

记录代码符号。

字段：

- `id`
- `file_id`
- `language`
- `name`
- `kind`
- `signature`
- `start_line`
- `end_line`
- `parent_symbol_id`
- `visibility`

### relations

记录基础关系。

字段：

- `id`
- `from_type`
- `from_id`
- `to_type`
- `to_id`
- `relation_kind`
- `confidence`
- `source`

`relation_kind` 可选：

- `contains`
- `imports`
- `uses`
- `extends`
- `implements`
- `parent_of`
- `test_of`
- `configures`
- `mentions`
- `same_module`
- `same_directory`

### chunks

记录可搜索代码片段。

字段：

- `id`
- `file_id`
- `symbol_id`
- `start_line`
- `end_line`
- `text`
- `token_estimate`
- `content_hash`

### context_packs

记录为 Goal / Milestone / Issue / freeform query 生成的上下文包。

字段：

- `id`
- `target_type`
- `target_id`
- `query`
- `created_at`
- `graph_revision`
- `recommended_files_json`
- `recommended_symbols_json`
- `recommended_tests_json`
- `impact_hints_json`
- `reason`
- `confidence`

### index_runs

记录每次后台索引任务。

字段：

- `id`
- `started_at`
- `finished_at`
- `status`
- `project_root`
- `git_head`
- `files_scanned`
- `files_indexed`
- `symbols_indexed`
- `relations_indexed`
- `error`

## meta.json

路径：

```text
.agentflow/output/graph/meta.json
```

示例：

```json
{
  "version": "graph.v1",
  "status": "ready",
  "projectRoot": "/Users/example/AgentFlow",
  "graphDb": ".agentflow/output/graph/graph.db",
  "updatedAt": 1780290000,
  "gitHead": "abc123",
  "fileCount": 1200,
  "symbolCount": 8400,
  "relationCount": 4300,
  "lastIndexRunId": "graph-run-001",
  "languages": ["rust", "typescript", "markdown", "toml"]
}
```

`status` 可选：

- `missing`
- `indexing`
- `ready`
- `stale`
- `failed`
- `degraded`

## 自动触发规则

Graph V1 不提供用户手动刷新按钮。它是后台基础设施，自动维护。

### Add / Open Project 后自动触发

```text
Project Workspace ready
-> Graph Manager 检查 .agentflow/output/graph/meta.json
-> missing: 后台索引
-> stale: 后台刷新
-> ready: 直接复用
-> failed: 标记失败但不阻塞打开项目
```

规则：

- 打开项目不能被 Graph 卡死。
- Graph 在后台跑。
- UI 只显示状态。

### 文件变化自动刷新

Desktop 运行期间，Graph Watcher 监听文件变化：

```text
文件新增 / 修改 / 删除
-> debounce 1-2 秒
-> 合并变化
-> 后台增量刷新
```

### Git HEAD 变化自动刷新

如果是 Git 项目：

```text
当前 HEAD != meta.json.gitHead
-> 标记 stale
-> 后台刷新
```

### AgentRun 前强制 preflight

未来 AgentRun 启动前必须跑 Graph Preflight：

- `missing` -> 自动索引
- `stale` -> 自动 catch-up
- `indexing` -> 等待或返回 pending
- `failed` -> AgentRun 降级，但必须记录原因
- `ready` -> 生成 Context Pack

当前仓库还没有授权 AgentRun 新流程；Graph V1 只提供 preflight 能力，不启动 Agent。

## Tauri 命令

新增：

- `prepare_project_graph`
- `load_project_graph_status`
- `load_project_graph_manifest`
- `search_project_graph`
- `build_graph_context_pack`
- `load_graph_context_pack`

### prepare_project_graph

输入：

```json
{
  "projectRoot": "/path/to/project"
}
```

行为：

- 确保 `.agentflow/output/graph/` 存在。
- 如果 `graph.db` 不存在，后台构建。
- 如果 stale，后台刷新。
- 返回当前 Graph 状态。

### load_project_graph_status

输出：

```json
{
  "version": "graph-status.v1",
  "projectRoot": "/path/to/project",
  "status": "ready",
  "fileCount": 1200,
  "symbolCount": 8400,
  "relationCount": 4300,
  "updatedAt": 1780290000,
  "lastError": null
}
```

### load_project_graph_manifest

输出：

```json
{
  "version": "graph-manifest.v1",
  "projectRoot": "/path/to/project",
  "languages": ["rust", "typescript", "markdown"],
  "topLevelDirs": ["apps", "crates", "docs"],
  "importantFiles": ["Cargo.toml", "README.md"],
  "sourceFiles": 120,
  "testFiles": 20,
  "docFiles": 12,
  "configFiles": 8
}
```

### search_project_graph

输入：

```json
{
  "projectRoot": "/path/to/project",
  "query": "lease",
  "limit": 20
}
```

输出：

```json
{
  "version": "graph-search.v1",
  "query": "lease",
  "results": []
}
```

### build_graph_context_pack

输入：

```json
{
  "projectRoot": "/path/to/project",
  "targetType": "issue",
  "targetId": "issue-001",
  "title": "Reject duplicate active lease",
  "objective": "拒绝重复 active lease",
  "acceptanceCriteria": []
}
```

输出：

```json
{
  "version": "graph-context-pack.v1",
  "targetType": "issue",
  "targetId": "issue-001",
  "query": "Reject duplicate active lease"
}
```

## 前端 UI 范围

Graph V1 不是用户手动浏览工具，所以 UI 要轻。

当前 Desktop 已经有本地文件阅读器 `ProjectLocalFilesPage`，Graph V1 不替代它。

关系：

```text
Project Files = 用户看文件
Graph = Agent 用项目地图
```

UI 只需要显示轻状态：

- Graph: indexing
- Graph: ready
- Graph: stale, updating
- Graph: failed

可以显示：

- 文件数
- 符号数
- 语言
- 更新时间

不做：

- 图谱大屏
- 复杂搜索页面
- 调用链可视化
- 手动刷新按钮

## 浏览器预览模式

浏览器预览环境没有真实文件系统。

Graph V1 在浏览器预览模式下：

- 不写真实 `.agentflow/output/graph/`
- 不构建真实 `graph.db`
- 返回 mock `GraphStatusSnapshot`
- 返回 mock `GraphManifest`
- 返回 mock `GraphSearchResult`

目的只是验证 UI 不崩。

## 交互边界

允许：

- 读取项目文件。
- 读取 Git HEAD。
- 写 `.agentflow/output/graph/**`。
- 写 Graph meta / db / context packs / exports。

不允许：

- 修改源码。
- 执行项目命令。
- 运行测试。
- 调用模型。
- 访问外部网络。
- 创建 PR / Issue / Linear。
- 把 Graph 产物提交进 Git。

## 验收标准

- [ ] 添加并打开本地 Project 后，系统会自动确保 `.agentflow/output/graph/` 存在。
- [ ] Graph V1 会在后台自动生成 `graph.db` 和 `meta.json`。
- [ ] 打开项目不会被 Graph 构建卡死。
- [ ] Graph 状态可以显示 `missing` / `indexing` / `ready` / `stale` / `failed` / `degraded`。
- [ ] Graph 不需要用户手动刷新。
- [ ] Graph 会跳过 `.git/`、`.agentflow/`、`node_modules/`、`target/`、`dist/`、`build/` 等目录。
- [ ] Graph 能索引文件列表、语言、文件类型、测试文件、配置文件。
- [ ] Graph 能提取 L1 语言的主要符号。
- [ ] Graph 能识别 Android / iOS / Flutter 项目结构。
- [ ] Graph 能写入基础关系：`contains`、`imports`、`test_of`、`configures`、`mentions`。
- [ ] Graph 支持文件 / 符号 / 片段搜索。
- [ ] Graph 能根据 freeform query 生成 Context Pack。
- [ ] Graph 能根据目标标题、目标描述、验收标准推荐相关文件和符号。
- [ ] Graph 能输出弱版影响提示。
- [ ] Graph 能输出弱版测试推荐。
- [ ] Graph 所有产物只写入 `.agentflow/output/graph/`。
- [ ] Graph 不修改业务源码。
- [ ] Graph 不执行命令。
- [ ] Graph 不调用模型。
- [ ] Graph 产物不会进入 Git / PR。
- [ ] 浏览器预览模式下可以返回 mock graph 状态，不写真实文件。

## 验证命令

- `npm --prefix apps/desktop run build`
- `cargo test`
- `cargo test -p agentflow-graph`
- `git diff --check`

## 建议开发切片

### Slice 1：Graph crate scaffold

目标：

- 新增 `crates/graph`
- 接入 workspace
- 创建 graph model 基础结构
- 创建 graph output path helper

验收：

- `cargo test -p agentflow-graph` 通过
- workspace 能识别 `agentflow-graph`

### Slice 2：Graph DB

目标：

- 创建 `graph.db`
- 实现 migrations
- 建立 `files` / `symbols` / `relations` / `chunks` / `context_packs` / `index_runs` 表

验收：

- 能创建空 `graph.db`
- 能写入 `index_runs`
- 能重复迁移不报错

### Slice 3：File Scanner

目标：

- 扫描项目文件
- 跳过忽略目录
- 识别语言和文件类型
- 写入 `files` 表
- 生成 manifest export

验收：

- 能扫描 Rust / TS / Markdown 测试 fixture
- 能跳过 `.agentflow` 和 `node_modules`

### Slice 4：Parser Registry + Symbol Extractor

目标：

- 支持 L1/L2/L3 文件的符号提取
- Tree-sitter 优先，轻量 extractor 兜底

验收：

- Rust / TS / Python / Java / Kotlin / Swift / Go / C++ / C# / Dart 至少能提取核心符号
- Markdown / JSON / YAML / TOML / XML 能提取结构化信息

### Slice 5：Relations

目标：

- 写入 `contains` / `imports` / `test_of` / `configures` / `mentions` 等关系

验收：

- 测试文件能和源码建立弱关联
- 配置文件能识别 project/module 关系

### Slice 6：Graph Search

目标：

- 支持文件搜索
- 支持符号搜索
- 支持 chunk 搜索

验收：

- `query=lease` 能返回相关文件、符号、片段
- 结果带 `score` 和 `snippet`

### Slice 7：Context Pack + Impact/Test Hints

目标：

- 根据 target 生成 Context Pack
- 输出推荐文件、符号、测试、影响提示、测试提示

验收：

- 输入 title/objective/acceptance 后能生成 `context-packs/<target-id>.json`

### Slice 8：Desktop / Tauri 集成

目标：

- 新增 Tauri commands
- 打开 Project 后后台 prepare graph
- UI 显示 Graph 状态
- 浏览器预览返回 mock

验收：

- Desktop build 通过
- 打开本地项目后 Graph 状态从 indexing 到 ready
- 文件阅读器不被替代、不被阻塞
