# 002.1 - Graph V1 Completion

创建日期：2026-06-01  
执行者：Codex

## 用户目标

Graph V1 已经完成主链路，但仍有一些能力处于“基础完成 / 基本完成 / 部分完成 / 延后 / 未完成”状态。

本补充需求的目标是：

```text
把 Graph V1 中所有非“完成”的能力，补齐到“完成”状态。
```

大白话：

> Graph V1 现在已经能建本地代码地图，也能搜索和生成 Context Pack。  
> 但它还不够“稳”，也不够“深”。  
> 这一步专门补齐：自动刷新、AgentRun 前检查、Tree-sitter 深度解析、移动端深度语义、关系和推荐质量、Git / PR 保护闭环。

完成后，Graph 应该从“可用第一版”升级为“稳定可依赖的 Agent 工作现场服务”。

---

## 背景

当前 Graph V1 已经落到：

```text
docs/requirements/002-graph-v1.md
```

当前已完成的主链路包括：

- `crates/graph` crate
- Graph DB
- 文件索引
- 轻量符号索引
- 基础关系
- 文件 / 符号 / chunk 搜索
- Context Pack
- 弱版影响提示
- 弱版测试推荐
- Tauri commands
- 前端状态显示
- 浏览器预览 mock

但还有以下状态需要收口：

| 模块 | 当前状态 | 本需求目标状态 |
| --- | --- | --- |
| 符号索引 | 基础完成 | 完成 |
| 基础关系 | 基础完成 | 完成 |
| 弱版影响分析 | 基础完成 | 完成 |
| 测试推荐 | 基础完成 | 完成 |
| Git / PR 保护 | 基本完成 | 完成 |
| 文件 watcher 自动刷新 | 未完成 | 完成 |
| Graph Preflight 独立模块 | 部分完成 | 完成 |
| Tree-sitter 深度解析 | 延后 | 完成 |
| 移动端深度语义 | 基础完成 | 完成 |

---

## 范围

本需求只补齐 Graph V1，不开启新模块。

范围包括 9 个收口项：

1. Graph Watcher 自动刷新
2. Graph Preflight 独立模块
3. Tree-sitter parser registry
4. L1 语言深度符号索引
5. L2 / L3 结构化索引补齐
6. 移动端深度语义识别
7. 基础关系增强
8. 影响分析和测试推荐增强
9. Git / PR 保护闭环

---

## 非目标

本补充需求不做以下事情：

- 不启动 Agent。
- 不执行测试命令。
- 不执行项目构建命令。
- 不调用模型。
- 不修改业务源码。
- 不创建 Goal / Milestone / Issue。
- 不接 OpenSpec / Superpowers / gstack。
- 不创建远程 PR、GitHub issue 或 Linear issue。
- 不做可视化图谱大屏。
- 不做代码问答系统。
- 不做 MCP Server。
- 不把 Graph 产物提交进 Git。

---

# 1. Graph Watcher 自动刷新

## 当前状态

未完成。

当前 Graph 可以在打开项目或调用 `prepare_project_graph` 时构建 / 刷新，但没有独立的文件 watcher。

## 目标状态

完成。

Graph 应该在 Desktop 运行期间自动监听项目文件变化，并在变化后自动刷新索引。

## 行为

Graph Watcher 应该：

1. 在本地 Project 打开后启动。
2. 监听 Project 根目录下文件新增、修改、删除。
3. 跳过无关目录：
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
4. 对连续文件变化做 debounce，默认 1-2 秒。
5. 合并短时间内的多次变化。
6. 自动刷新 Graph。
7. 刷新期间 Graph 状态显示为 `indexing`。
8. 刷新完成后 Graph 状态回到 `ready`。
9. 刷新失败时 Graph 状态变为 `failed` 或 `degraded`，并记录 `lastError`。

## 数据输出

Watcher 不新增单独事实源，只更新：

```text
.agentflow/output/graph/graph.db
.agentflow/output/graph/meta.json
.agentflow/output/graph/exports/*
```

## 验收标准

- [ ] 修改普通源码文件后，Graph 会在后台自动刷新。
- [ ] 新增文件后，`files` 表能看到新文件。
- [ ] 删除文件后，`files` 表不再保留该文件。
- [ ] 修改 `.agentflow/output/graph/**` 不会触发无限刷新。
- [ ] 修改 `target/`、`node_modules/` 等忽略目录不会触发刷新。
- [ ] 多次快速保存只触发一次合并刷新。
- [ ] 刷新失败不会导致 Desktop 崩溃。

---

# 2. Graph Preflight 独立模块

## 当前状态

部分完成。

当前 `prepare_project_graph` 可以承担一部分 preflight 作用，但还没有独立的 Graph Preflight 模块。

## 目标状态

完成。

Graph 需要提供独立的 `GraphPreflight`，专门用于 AgentRun 启动前检查 Graph 是否可用。

## 行为

Graph Preflight 输入：

```text
projectRoot
targetType
targetId
title
objective
acceptanceCriteria
```

Graph Preflight 输出：

```text
status
ready
reason
graphStatus
contextPackPath
recommendedFiles
recommendedSymbols
recommendedTests
impactHints
testHints
```

## 状态规则

```text
missing  -> 自动构建 Graph，然后继续
stale    -> 自动 catch-up，然后继续
indexing -> 等待完成或返回 pending
ready    -> 生成 Context Pack
failed   -> 返回 degraded，不阻塞 UI，但阻止自动 AgentRun
```

## 需要新增的 Rust API

建议新增：

```rust
pub fn preflight_graph_for_target(...) -> Result<GraphPreflightSnapshot>
```

建议新增 Tauri command：

```text
graph_preflight
```

## 验收标准

- [ ] Graph missing 时，preflight 会触发 Graph 构建。
- [ ] Graph stale 时，preflight 会触发 catch-up。
- [ ] Graph ready 时，preflight 会生成 Context Pack。
- [ ] Graph failed 时，preflight 返回明确失败原因。
- [ ] preflight 不启动 Agent。
- [ ] preflight 不执行测试。
- [ ] preflight 不调用模型。

---

# 3. Tree-sitter 深度解析

## 当前状态

延后。

当前 Graph V1 使用轻量 line-based extractor，已经能提取主流符号，但不是 Tree-sitter 深度解析。

## 目标状态

完成。

Graph 需要建立 Tree-sitter parser registry，并优先使用 Tree-sitter 解析 L1 语言。

## L1 语言

Tree-sitter 优先支持：

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

## 行为

每个语言 parser 应该输出统一模型：

```text
symbols
imports
contains relations
parent relations
chunks
```

如果某个语言 parser 不可用：

```text
降级到 lightweight extractor
Graph status 可以保持 ready，但需要在 meta 中记录 degraded reason
```

## 完成定义

Tree-sitter 深度解析不要求 IDE 级语义分析，但必须比 line extractor 更稳定：

- 能跨多行识别符号。
- 能识别嵌套符号。
- 能记录 `parent_symbol_id`。
- 能提取函数 / 类 / 接口 / struct 的起止行。
- 能提取 import/use/include/package/module。

## 验收标准

- [ ] L1 语言都接入 parser registry。
- [ ] Rust 能提取 `fn`、`struct`、`enum`、`trait`、`impl`、`mod`、`use`。
- [ ] TypeScript / JavaScript 能提取 function、class、interface、type、export、import、React component。
- [ ] Python 能提取 def、async def、class、import、from import、decorator。
- [ ] Java 能提取 package、import、class、interface、enum、record、method。
- [ ] Kotlin 能提取 class、object、data class、interface、function、import。
- [ ] Swift 能提取 class、struct、enum、protocol、extension、function、import。
- [ ] Go 能提取 package、import、func、method receiver、struct、interface。
- [ ] C / C++ 能提取 include、function、class、struct、namespace。
- [ ] C# 能提取 namespace、using、class、interface、method。
- [ ] Dart 能提取 import、class、mixin、extension、function、Widget。
- [ ] parser 失败时会降级，不会导致整仓索引失败。

---

# 4. 符号索引补齐

## 当前状态

基础完成。

当前轻量符号索引可以提取常见符号，但缺少稳定的 parent、range、language-specific symbol kind。

## 目标状态

完成。

符号索引需要成为 Agent 可依赖的结构化代码地图。

## 行为

`symbols` 表中的符号需要尽量补齐：

```text
id
file_id
language
name
kind
signature
start_line
end_line
parent_symbol_id
visibility
path
```

符号类型需要支持：

```text
module
package
namespace
class
struct
enum
interface
trait
protocol
function
method
constructor
component
constant
variable
type_alias
test
route
config_key
markdown_heading
```

## 验收标准

- [ ] 多行 class / struct / function 能记录正确起止行。
- [ ] 方法能挂到 class / impl / object / struct 父节点下。
- [ ] test 符号可以被识别。
- [ ] component / Widget / SwiftUI View / Compose function 可以被识别。
- [ ] config key 和 markdown heading 继续保留。
- [ ] 搜索符号时能返回 `line`、`signature`、`symbolKind`。

---

# 5. 基础关系补齐

## 当前状态

基础完成。

当前已有：

- contains
- imports
- test_of
- configures
- same_directory

但需求中还有 `mentions`、`same_module`、`parent_of`、`uses`、`extends`、`implements` 等关系需要补齐。

## 目标状态

完成。

Graph 需要提供足够 Agent 使用的基础关系图。

## 必须支持的关系

```text
contains
imports
uses
extends
implements
parent_of
test_of
configures
mentions
same_module
same_directory
```

## 行为

关系不要求 100% 精确，但必须可解释，每条关系都要有：

```text
confidence
source
```

例如：

```text
source = tree-sitter-import
source = filename-heuristic
source = config-classifier
source = symbol-mention
source = module-path-heuristic
```

## 验收标准

- [ ] file -> symbol 有 `contains`。
- [ ] symbol -> symbol 有 `parent_of`。
- [ ] import/use/include/package 能生成 `imports`。
- [ ] extends / implements 能生成对应关系。
- [ ] 测试文件能和源码建立 `test_of`。
- [ ] 配置文件能生成 `configures`。
- [ ] 同模块文件能生成 `same_module`。
- [ ] 文本命中符号名能生成 `mentions`。
- [ ] 所有关系都有 confidence 和 source。

---

# 6. 弱版影响分析补齐

## 当前状态

基础完成。

当前 Context Pack 已能基于关系给出 impact hints，但仍比较弱。

## 目标状态

完成。

Graph 需要提供独立的影响分析能力，供 Agent 在执行前 / 执行后判断影响范围。

## 输入

```text
projectRoot
changedFiles?
targetFiles?
targetSymbols?
query?
```

## 输出

```text
possiblyAffectedFiles
possiblyAffectedSymbols
possiblyAffectedTests
reasons
confidence
```

## 依据

影响分析基于：

- imports
- contains
- parent_of
- test_of
- configures
- mentions
- same_module
- same_directory

## 建议新增 Rust API

```rust
pub fn analyze_graph_impact(...) -> Result<GraphImpactSnapshot>
```

建议新增 Tauri command：

```text
analyze_graph_impact
```

## 验收标准

- [ ] 输入 changed file 后，能返回可能受影响文件。
- [ ] 输入 symbol 后，能返回相关文件和测试文件。
- [ ] 影响结果包含 reason。
- [ ] 影响结果包含 confidence。
- [ ] 不执行测试。
- [ ] 不调用模型。

---

# 7. 测试推荐补齐

## 当前状态

基础完成。

当前已经有 Rust/npm/focused test hint，但推荐较粗。

## 目标状态

完成。

Graph 应该根据项目类型、语言、测试文件、配置文件和变更范围，给出更可靠的测试推荐。

## 推荐依据

```text
语言
manifest/config 文件
测试目录
测试文件命名
changed files
Context Pack recommended files
impact analysis affected files
test_of 关系
```

## 推荐格式

```text
commandHint
reason
confidence
scope
```

`scope` 可选：

```text
focused
package
module
full
unknown
```

## 语言测试推荐规则

至少支持：

```text
Rust       -> cargo test, cargo test <keyword>
Node/TS    -> npm test / pnpm test / yarn test / vitest / jest hint
Python     -> pytest / python -m pytest
Go         -> go test ./...
Java       -> mvn test / gradle test
Kotlin     -> ./gradlew test / connectedAndroidTest hint
Swift/iOS  -> xcodebuild test hint
Dart       -> flutter test / dart test
C#         -> dotnet test
PHP        -> vendor/bin/phpunit
Ruby       -> bundle exec rspec
```

注意：Graph 只推荐，不执行。

## 验收标准

- [ ] Rust 项目推荐 `cargo test`。
- [ ] package.json 有 test script 时推荐对应 npm/pnpm/yarn 命令。
- [ ] Python 项目推荐 pytest。
- [ ] Go 项目推荐 `go test ./...`。
- [ ] Java / Gradle 项目推荐 Maven / Gradle 测试命令。
- [ ] Android 项目能提示 unit test / instrumented test 区别。
- [ ] iOS 项目能提示 xcodebuild test。
- [ ] Flutter 项目推荐 `flutter test`。
- [ ] 推荐结果包含 reason、confidence、scope。

---

# 8. 移动端深度语义补齐

## 当前状态

基础完成。

当前已能识别移动端相关语言和配置文件，但还没有深度移动端语义。

## 目标状态

完成。

Graph 需要能识别 iOS、Android、Flutter 项目的关键入口、平台配置和常见组件。

## Android

需要识别：

```text
Gradle module
AndroidManifest.xml
applicationId
Activity
Service
BroadcastReceiver
ContentProvider
permission
Compose @Composable
ViewModel
Repository
JNI / NDK 文件
res/layout
res/values
unit test
instrumented test
```

## iOS / Apple

需要识别：

```text
.xcodeproj
.xcworkspace
Package.swift
Podfile
Info.plist
Bundle Identifier
App entry
Scene entry
SwiftUI View
UIViewController
XCTest
storyboard
xib
entitlements
Objective-C interface / implementation
```

## Flutter

需要识别：

```text
pubspec.yaml
lib/main.dart
Widget
StatefulWidget
StatelessWidget
route
platform directories: android/ ios/
test/
flutter_test
assets
```

## 输出

移动端识别结果可以先写入 Graph manifest 扩展字段，或写入 `relations` / `symbols`：

```text
platform = android | ios | flutter
entry_points
mobile_components
mobile_configs
mobile_tests
```

## 验收标准

- [ ] Android 项目能识别 Manifest 和 Gradle module。
- [ ] Android 项目能识别 Activity / Service / permission。
- [ ] Kotlin Compose 项目能识别 `@Composable`。
- [ ] iOS 项目能识别 Info.plist / Package.swift / Podfile。
- [ ] iOS SwiftUI 项目能识别 View。
- [ ] iOS UIKit 项目能识别 UIViewController。
- [ ] Flutter 项目能识别 pubspec.yaml、lib/main.dart、Widget、test。
- [ ] Context Pack 会优先推荐移动端入口和相关配置。

---

# 9. Git / PR 保护闭环

## 当前状态

基本完成。

当前 `.agentflow/output/graph/` 已进入 ignore，但仍需要让 Graph 自身具备保护检查，避免输出进入 Git / PR。

## 目标状态

完成。

Graph 需要提供本地保护检查，确保 Graph 产物不会被当作项目交付物。

## 行为

Graph Protection 应该检查：

1. `.agentflow/output/graph/` 是否只作为本地输出目录存在。
2. `.git/info/exclude` 是否包含 `.agentflow/` 或 `.agentflow/output/graph/`。
3. Graph 不主动写 `.gitignore`。
4. Graph 不写根目录文档。
5. Graph 不写源码。

## 建议新增 Rust API

```rust
pub fn check_graph_git_protection(project_root: impl AsRef<Path>) -> Result<GraphProtectionSnapshot>
```

建议新增 Tauri command：

```text
check_graph_git_protection
```

## 验收标准

- [ ] Git 项目中 `.git/info/exclude` 能保护 `.agentflow/`。
- [ ] Graph 不写 `.gitignore`。
- [ ] Graph 不写源码文件。
- [ ] Graph 所有输出都在 `.agentflow/output/graph/`。
- [ ] 如果保护缺失，Graph 状态显示 warning / degraded。

---

## 页面 / 功能

Graph Completion 不新增复杂页面。

只增强现有状态通道：

```text
工作现场 / Graph
```

需要显示：

- Graph status
- file count
- symbol count
- relation count
- languages
- watcher status
- preflight status
- protection status
- last error

不新增：

- 图谱可视化页面
- 用户手动刷新按钮
- 搜索大屏
- 调用链视图

---

## 数据来源

- 本地 Project 文件系统
- `.agentflow/output/graph/graph.db`
- `.agentflow/output/graph/meta.json`
- `.agentflow/output/graph/context-packs/`
- Git HEAD 文件
- `.git/info/exclude`

---

## 交互边界

允许：

- 读取项目文件。
- 读取 Git HEAD。
- 读取 `.git/info/exclude`。
- 写 `.agentflow/output/graph/**`。
- 后台监听文件变化。
- 后台刷新 Graph 索引。

不允许：

- 修改业务源码。
- 执行项目命令。
- 运行测试。
- 调用模型。
- 访问外部网络。
- 创建 PR / Issue / Linear。
- 写 `.codex/`。
- 写 `graphify-out/`。
- 把 Graph 产物提交进 Git。

---

## 验收标准总表

- [ ] 文件 watcher 自动刷新完成。
- [ ] debounce 机制完成。
- [ ] Graph Preflight 独立模块完成。
- [ ] Graph Preflight 能生成 Context Pack。
- [ ] Tree-sitter parser registry 完成。
- [ ] L1 语言深度符号索引完成。
- [ ] L2 / L3 结构化索引补齐完成。
- [ ] 符号索引支持 parent / range / visibility。
- [ ] 基础关系覆盖完整关系类型。
- [ ] 影响分析能返回 affected files / symbols / tests。
- [ ] 测试推荐覆盖 Rust / Node / Python / Go / Java / Android / iOS / Flutter / C# / PHP / Ruby。
- [ ] 移动端深度语义完成。
- [ ] Git / PR 保护检查完成。
- [ ] Graph 状态通道能显示 watcher / preflight / protection 状态。
- [ ] 所有新增能力不执行命令、不调用模型、不修改源码。
- [ ] 所有 Graph 输出只写 `.agentflow/output/graph/**`。

---

## 验证命令

- `cargo fmt --check`
- `cargo test -p agentflow-graph`
- `cargo test`
- `npm --prefix apps/desktop run build`
- `git diff --check`

---

## 建议开发切片

### Slice 1：Graph Watcher

目标：

- 增加 watcher 模块。
- 增加 debounce。
- 文件变化后自动刷新。

验收：

- 修改 fixture 文件后，Graph 自动更新。

### Slice 2：Graph Preflight

目标：

- 增加 GraphPreflightSnapshot。
- 增加 preflight API 和 Tauri command。
- missing / stale / failed / ready 状态处理完整。

验收：

- preflight 能自动构建或 catch-up，并生成 Context Pack。

### Slice 3：Tree-sitter Registry

目标：

- 增加 parser registry。
- L1 语言优先走 Tree-sitter。
- parser 失败时 fallback。

验收：

- L1 fixture 都能提取核心符号。

### Slice 4：Symbol + Relation Completion

目标：

- 补 parent_symbol_id。
- 补 start_line / end_line。
- 补 parent_of / mentions / same_module / extends / implements。

验收：

- 多语言 fixture 中关系完整可查。

### Slice 5：Impact + Test Recommendation Completion

目标：

- 增加独立 impact API。
- 增强测试推荐规则。

验收：

- changed files 能生成 affected files 和 test hints。

### Slice 6：Mobile Semantics

目标：

- Android / iOS / Flutter 深度语义补齐。

验收：

- 移动端 fixture 能识别入口、配置、组件、测试。

### Slice 7：Protection + UI Status

目标：

- 增加 Graph protection check。
- 状态通道显示 watcher / preflight / protection。

验收：

- Graph 状态能明确显示运行态是否完整。

---

## 完成定义

本补充需求完成后，Graph V1 的状态表应全部变为：

| 模块 | 目标状态 |
| --- | --- |
| 符号索引 | 完成 |
| 基础关系 | 完成 |
| 弱版影响分析 | 完成 |
| 测试推荐 | 完成 |
| Git / PR 保护 | 完成 |
| 文件 watcher 自动刷新 | 完成 |
| Graph Preflight 独立模块 | 完成 |
| Tree-sitter 深度解析 | 完成 |
| 移动端深度语义 | 完成 |

最终一句话：

> Graph V1 Completion 不是新增产品方向，而是把 Graph 从“基础可用”补到“稳定可依赖”，让它可以作为后续 Goal Tree 和 AgentRun 的现场资源底座。
