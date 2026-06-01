# 002.2 - Graph V1 OS Native Watcher Closeout

创建日期：2026-06-01
执行者：Codex

## 用户目标

Graph V1 Completion 已经补齐 watcher、preflight、Tree-sitter、影响分析、测试推荐和保护检查。但当前 watcher 属于轮询 fingerprint 方案，不够“OS 原生级别”。

本需求目标是：

```text
把 Graph Watcher 从轮询式 watcher 升级为 OS 原生文件系统事件 watcher。
```

大白话：

> Graph 不应该每隔一段时间扫一遍项目来猜文件有没有变。
> 它应该像成熟开发工具一样接系统原生文件事件。
> macOS 用 FSEvents，Windows 用 ReadDirectoryChangesW，Linux 用 inotify。
> 只有在原生 watcher 不可用的特殊环境里，才允许降级到 PollWatcher / fingerprint fallback。

完成后，Graph watcher 的默认完成态必须是：

```text
OS native watcher by default
poll / fingerprint only as degraded fallback
```

---

## 背景

当前 Graph V1 Completion 已完成主链路：

- Graph Watcher
- Graph Preflight
- Tree-sitter parser registry
- L1 语言符号索引
- L2 / L3 结构化索引
- 移动端语义识别
- Impact analysis
- Test recommendation
- Git protection

但 watcher 边界仍需提升：

```text
当前 watcher = polling fingerprint watcher
目标 watcher = OS native event watcher
```

当前轮询方案可以作为兜底，但不能作为 Graph Watcher 的最终完成态。

---

## 范围

本需求只做 Graph Watcher 的 OS 原生化，以及相关稳定性验收。

范围包括：

1. 使用 OS native watcher 作为默认实现。
2. 明确 macOS / Windows / Linux 平台行为。
3. 保留 PollWatcher / fingerprint fallback，但只能作为 degraded 模式。
4. 保持 debounce。
5. 保持忽略目录过滤。
6. 保持 Graph status 可观测。
7. 补跨平台 watcher fixture / 单元测试 / 集成测试。
8. 继续收尾 L1 / L2-L3 / mobile fixture matrix，保证 Graph watcher 与解析能力一起稳定。

---

## 非目标

本需求不做：

- 不做 IDE。
- 不做 LSP。
- 不做完整调用图。
- 不做类型推导。
- 不执行测试。
- 不启动 Agent。
- 不调用模型。
- 不修改源码。
- 不创建远程 PR / GitHub issue / Linear issue。
- 不做图谱可视化。

---

# 1. OS Native Watcher

## 当前状态

当前 watcher 是：

```text
loop
  sleep
  scan project fingerprint
  if changed:
    debounce
    reindex
```

这个方案可以用，但它不是最终完成态。

## 目标状态

Graph Watcher 默认必须使用 OS native file event watcher。

推荐 Rust 实现：

```text
notify::recommended_watcher
```

平台目标：

```text
macOS   -> FSEvents
Windows -> ReadDirectoryChangesW
Linux   -> inotify
```

## 依赖建议

在 workspace 中增加：

```toml
notify = "8"
```

可以选择增加：

```toml
notify-debouncer-full = "0.5"
```

如果先不引入 `notify-debouncer-full`，也可以继续保留当前手写 debounce，但事件来源必须是 OS native watcher。

---

# 2. Watcher 行为

Graph Watcher 应该：

1. 在 Project 打开后启动。
2. 对 Project root 做 recursive watch。
3. 使用 OS native backend 接收文件事件。
4. 过滤无关目录：
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
5. 支持事件类型：
   - create
   - modify
   - remove
   - rename
6. 多事件合并 debounce，默认 1-2 秒。
7. 事件批处理后触发 Graph refresh。
8. Graph 输出仍只写：

```text
.agentflow/output/graph/**
```

9. Graph 自己写 `.agentflow/output/graph/**` 不能反触发无限刷新。
10. watcher 出错时状态进入 degraded，不导致 Desktop 崩溃。

---

# 3. Fallback 策略

OS native watcher 是默认完成态。

允许 fallback，但必须显式标记为 degraded。

允许 fallback 的情况：

```text
native watcher 创建失败
网络文件系统不发事件
Docker / VM / WSL 等特殊环境事件不可用
Linux inotify watch limit 不足
macOS FSEvents 权限限制
用户显式启用 fallback
```

fallback 模式：

```text
PollWatcher
或者当前 fingerprint watcher
```

但 fallback 必须满足：

```text
watcherStatus = fallback
Graph status = degraded 或 ready_with_degraded_reasons
meta.degradedReasons 包含 fallback 原因
状态栏可见 fallback 信息
```

---

# 4. Graph 状态模型更新

当前 `GraphStatusSnapshot` 已有：

```text
watcherStatus
preflightStatus
protectionStatus
degradedReasons
```

需要增强 watcher 状态：

```text
watcherStatus:
  missing
  starting
  native
  debouncing
  indexing
  fallback
  failed
```

建议新增 watcher backend 字段：

```text
watcherBackend:
  fsevents
  read_directory_changes_w
  inotify
  kqueue
  poll
  fingerprint
  unknown
```

建议新增 watcher detail：

```text
watcherDetail:
  platform
  recursive
  ignoredPathCount
  lastEventAt
  lastEventKind
  lastError
```

---

# 5. Rust API 设计

保留：

```rust
pub fn ensure_graph_watcher(project_root: impl AsRef<Path>) -> Result<GraphWatcherSnapshot>
```

`GraphWatcherSnapshot` 增强为：

```rust
pub struct GraphWatcherSnapshot {
    pub version: String,
    pub project_root: String,
    pub status: String,
    pub backend: String,
    pub recursive: bool,
    pub debounce_ms: u64,
    pub ignored_path_count: usize,
    pub last_event_at: Option<u64>,
    pub last_event_kind: Option<String>,
    pub last_error: Option<String>,
}
```

建议内部拆分：

```text
watcher/native.rs
watcher/debounce.rs
watcher/filter.rs
watcher/fallback.rs
```

如果暂时保持单文件，也要按函数边界拆清楚：

```text
start_native_watcher
start_fallback_watcher
filter_notify_event
schedule_debounced_refresh
record_watcher_state
```

---

# 6. Event Filter

Graph Watcher 必须过滤自身输出和运行态目录。

需要统一 filter：

```rust
fn should_ignore_graph_event(path: &Path) -> bool
```

必须过滤：

```text
.agentflow/
.git/
node_modules/
target/
dist/
build/
coverage/
.cache/
vendor/
.idea/
.vscode/
.DS_Store
```

注意：

```text
.agentflow/output/graph/** 必须过滤，否则 Graph refresh 写 meta/db/export 会再次触发 watcher，形成循环。
```

---

# 7. Debounce 规则

OS native watcher 会发很多事件，必须 debounce。

规则：

```text
native event received
  -> filter ignored paths
  -> record event
  -> debounce 1-2 seconds
  -> if no new event
  -> refresh graph
```

要求：

- 多个 create / modify / remove 事件合并为一次 refresh。
- rename 事件按 remove + create 或 rename hint 处理。
- 如果事件批次里只有 ignored paths，不刷新。
- 如果 Graph 当前正在 indexing，新事件进入 pending，不并发跑两个 index。

---

# 8. 并发控制

Graph Watcher 不能并发启动多个相同 Project watcher。

规则：

```text
same canonical project root:
  只允许一个 watcher
```

如果重复调用：

```text
ensure_graph_watcher(root)
  -> 返回已有 watcher snapshot
```

刷新也不能并发：

```text
if indexing:
  mark pending_refresh = true
  current indexing done 后再刷新一次
```

---

# 9. L1 / L2-L3 / Mobile 稳定性收尾

本需求仍保留稳定性收尾目标：

```text
L1 语言深度符号索引       -> fixture matrix 完成
L2 / L3 结构化索引        -> fixture matrix 完成
移动端深度语义            -> fixture matrix 完成
```

但这些不再扩功能，只补稳定性测试。

## L1 fixture matrix

每个 L1 语言至少一份 fixture：

```text
TypeScript / JavaScript
Python
Java
Kotlin
Swift
Go
Rust
C / C++
C#
Dart
```

验收：

```text
- 提取核心 symbol
- 提取 import/use/include/package
- 有 contains relation
- 有 parent_of relation
- 有 start_line / end_line
- 有 chunk
```

## L2 / L3 fixture matrix

覆盖：

```text
PHP
Ruby
SQL
Shell
PowerShell
HTML
CSS
Markdown
JSON
YAML
TOML
XML
plist
Gradle
Dockerfile
package.json
Cargo.toml
pyproject.toml
go.mod
pubspec.yaml
Package.swift
Podfile
```

验收：

```text
- language 正确
- kind 正确
- chunk 存在
- config_key / markdown_heading 正常
- search 能查到关键字段
```

## Mobile fixture matrix

覆盖：

```text
Android
iOS
Flutter
```

验收：

```text
- platform 正确
- entry_points 正确
- mobile_configs 正确
- mobile_components 正确
- mobile_tests 正确
- test recommendation 正确
```

---

# 10. 验收标准

## OS Native Watcher 验收

- [ ] Graph Watcher 默认使用 OS native watcher，而不是 fingerprint polling。
- [ ] macOS 使用 FSEvents 或 notify recommended backend。
- [ ] Windows 使用 ReadDirectoryChangesW 或 notify recommended backend。
- [ ] Linux 使用 inotify 或 notify recommended backend。
- [ ] Project root 使用 recursive watch。
- [ ] 文件 create 会触发 Graph refresh。
- [ ] 文件 modify 会触发 Graph refresh。
- [ ] 文件 remove 会触发 Graph refresh。
- [ ] 文件 rename 会触发 Graph refresh。
- [ ] `.agentflow/output/graph/**` 不会触发刷新循环。
- [ ] `.git/`、`target/`、`node_modules/` 等被忽略。
- [ ] 多个事件会 debounce 成一次刷新。
- [ ] indexing 期间新事件会排队，不并发刷新。
- [ ] native watcher 不可用时 fallback，并标记 degraded。
- [ ] watcher status 显示 backend。
- [ ] Desktop 不因为 watcher 错误崩溃。

## 稳定性验收

- [ ] L1 fixture matrix 完成。
- [ ] L2 / L3 fixture matrix 完成。
- [ ] Mobile fixture matrix 完成。
- [ ] parser fallback 有测试。
- [ ] protection degraded 有测试。
- [ ] preflight failed / degraded 有测试。

---

## 验证命令

- `cargo fmt --check`
- `cargo test -p agentflow-graph`
- `cargo test`
- `npm --prefix apps/desktop run build`
- `git diff --check`

建议新增跨平台本地 smoke：

```text
cargo test -p agentflow-graph watcher_native_event_refreshes_graph
```

CI 如果暂时不能覆盖 macOS / Windows / Linux 全平台，至少要求：

```text
单元测试覆盖 filter / debounce / fallback
本机 smoke 覆盖当前 OS native watcher
文档记录未覆盖平台
```

---

## 建议开发切片

### Slice 1：引入 notify native watcher

目标：

- 增加 `notify` 依赖。
- 用 `recommended_watcher` 替换当前 fingerprint 主路径。
- 保留 fingerprint 作为 fallback。

验收：

- 修改源码文件触发 native event。
- watcherBackend 不再是 fingerprint。

### Slice 2：Event filter + debounce

目标：

- 统一 ignored path filter。
- 保留 debounce。
- 防止 `.agentflow/output/graph/**` 自触发循环。

验收：

- Graph 写 meta/db/export 不触发无限刷新。

### Slice 3：并发控制

目标：

- 同一个 Project 只有一个 watcher。
- indexing 期间事件进入 pending。

验收：

- 快速多次文件变化只触发合并刷新。

### Slice 4：Fallback + degraded 状态

目标：

- native watcher 创建失败时 fallback。
- status / meta / degradedReasons 记录原因。

验收：

- 模拟 native watcher 失败时进入 fallback。

### Slice 5：Fixture matrix closeout

目标：

- 补 L1 / L2-L3 / Mobile fixture 矩阵。

验收：

- Graph 三项基本完成全部变成完成。

---

## 完成定义

本需求完成后，Graph watcher 状态应变为：

```text
watcher = OS native event watcher by default
fallback = only degraded backup
```

并且：

| 能力 | 目标状态 |
| --- | --- |
| OS native watcher | 完成 |
| Debounce | 完成 |
| Event filter | 完成 |
| No self-refresh loop | 完成 |
| Watcher backend status | 完成 |
| Fallback degraded | 完成 |
| L1 fixture matrix | 完成 |
| L2 / L3 fixture matrix | 完成 |
| Mobile fixture matrix | 完成 |

最终一句话：

> Graph V1 OS Native Watcher Closeout 把 Graph 从“能自动刷新”升级为“按系统原生文件事件自动刷新”，并把 L1 / L2-L3 / 移动端语义的稳定性测试矩阵补齐，作为进入 Project File Reader 和 Goal Tree 的可靠底座。

---

# 附录 A：003.1 - Project File Reader V1 Polish

创建日期：2026-06-02
执行者：Codex

## 用户目标

Project File Reader V1 已完成主体能力，但还有 3 个修复项需要收口：

1. 大文本 range API 已有，但前端还没有真正使用。
2. Graph 推荐文件入口已有，但联动还不够完整。
3. 推荐文件不存在时，UI 没有正确标记 missing 状态。

大白话：

> 文件阅读器已经能用，但这 3 个地方还差最后一层体验闭环。
> 本次只做修复，不做新大功能，不把 Project File Reader 做成 IDE。

## 总范围

本附录只补 Project File Reader V1 的收尾修复：

- LargeTextReader 接入真实 range 加载。
- Graph 推荐文件模型补充 source / reason / status。
- 推荐文件不存在时标记 missing，并避免触发读取错误。

## 总非目标

本附录不做：

- 不做全文搜索。
- 不做编辑。
- 不做保存。
- 不做 diff。
- 不做 tail -f。
- 不做 IDE 级大文件查看器。
- 不做 Graph 可视化。
- 不自动创建 Context Pack。
- 不修改 Graph DB。
- 不自动刷新 Graph。
- 不调用模型。
- 不执行命令。
- 不写 `.agentflow/` runtime。

---

## 修复项 1：LargeTextReader 接入真实 range 加载

### 当前状态

后端已经有：

```text
load_project_file_text_range
```

它可以按行读取大文本文件，并返回：

```text
startLine
endLine
totalLines
content
truncated
```

但前端 LargeTextReader 目前仍主要对已经加载到前端的 `content` 做虚拟滚动，没有真正按需调用 `load_project_file_text_range`。

### 目标状态

大文本文件应该真正支持：

- 按行范围加载。
- 滚动或点击下一段时再请求后端。
- 不一次性把大文件内容塞进前端。

### 开发范围

后端已有 command，本次不新增后端 command，只确认参数可用：

```text
relativePath
startLine
lineCount
projectRoot
```

前端需要改：

- `apps/desktop/src/features/project-files/FileRendererRegistry.tsx`
- `apps/desktop/src/features/project-files/useProjectFiles.ts`
- `apps/desktop/src/types.ts`

建议新增 hook 或函数：

```ts
loadProjectFileTextRange(relativePath, startLine, lineCount)
```

### 行为要求

当文件是大文本或 `truncated = true` 时：

1. 先显示当前 512KB preview。
2. 显示“上一段 / 下一段 / 跳到行号”入口。
3. 用户请求下一段时，调用 `load_project_file_text_range`。
4. 后端返回指定行范围。
5. 前端只追加或替换当前可见 range。
6. 不把整个大文件加载进 DOM。
7. UI 显示当前行范围，例如 `1001-1240 / 共 50000 行`。
8. 加载 range 失败时显示明确错误，不空白。

第一版不要求复杂无限滚动，先做按钮式范围加载即可。

### 验收标准

- [ ] 大文本文件默认显示首段预览。
- [ ] 大文本 reader 能调用 `load_project_file_text_range`。
- [ ] 点击“下一段”能加载后续行。
- [ ] 点击“上一段”能回到前一段。
- [ ] 输入行号后能加载目标行附近内容。
- [ ] UI 显示当前行范围和总行数。
- [ ] 加载 range 失败时显示明确错误。
- [ ] 不把完整大文件写入 `localStorage`。
- [ ] 不执行命令，不修改文件。

---

## 修复项 2：Graph 推荐文件联动补完整

### 当前状态

当前 App 已经从 Graph 里拿推荐文件来源：

```text
latestContextPack.recommendedFiles
latestContextPack.recommendedTests
manifest.importantFiles
```

并传给 Project File Reader。`ProjectLocalFilesPage` 也会把这些路径转成推荐行，右侧文件浏览器已有“推荐文件 / 来自代码地图”的入口。

但目前联动仍偏弱：

1. 主要被动使用 `latestContextPack`。
2. 没有 `latestContextPack` 时主要依赖 `manifest.importantFiles`。
3. 推荐文件和当前选中的 Issue / Goal 没有形成明确绑定。
4. 推荐文件缺失态没有完整处理。

### 目标状态

Project File Reader 应该稳定展示 Graph 推荐文件，并且推荐来源可解释。

Graph 推荐文件不是装饰入口。它应该明确告诉用户或 Agent：

- 这些文件为什么推荐。
- 推荐来自哪里。
- 当前是否可以打开。

### 开发范围

需要改：

- `apps/desktop/src/App.tsx`
- `apps/desktop/src/features/project-files/ProjectLocalFilesPage.tsx`
- `apps/desktop/src/features/project-files/ProjectFileBrowser.tsx`
- `apps/desktop/src/features/project-files/useProjectGraph.ts`
- `apps/desktop/src/types.ts`

### 推荐来源优先级

推荐文件来源分 3 层：

1. `GraphContextPack.recommendedFiles`
2. `GraphContextPack.recommendedTests`
3. `GraphManifest.importantFiles`

### 推荐项模型

每个推荐项需要包含：

```ts
export type ProjectRecommendedFile = {
  path: string;
  name: string;
  source: "context-pack-file" | "context-pack-test" | "manifest-important";
  reason: string;
  status: "available" | "missing" | "unloaded";
};
```

### UI 行为

推荐文件区域显示：

```text
推荐文件
来自代码地图
```

每个推荐 chip 显示：

- 文件名
- 来源
- 状态

示例：

```text
lease.rs        Context
lease_test.rs   Test
Cargo.toml      Important
```

### 验收标准

- [ ] 有 Context Pack 时，优先显示 `recommendedFiles`。
- [ ] 有 `recommendedTests` 时，也能显示为测试推荐文件。
- [ ] 没有 Context Pack 时，显示 `manifest.importantFiles`。
- [ ] 推荐文件能显示来源。
- [ ] 推荐文件能显示推荐原因。
- [ ] 推荐文件点击后能打开。
- [ ] Graph `missing / indexing / failed` 时不阻塞文件阅读器。
- [ ] 推荐区域只读，不执行命令。

---

## 修复项 3：推荐文件不存在态 missing 处理

### 当前状态

`ProjectFileBrowser` 已经有 missing 的 UI 逻辑：

```text
row.missing -> 显示 “已不存在”
```

但 `ProjectLocalFilesPage` 在构建推荐文件时，如果推荐路径找不到真实 entry，会创建 fallback row，却没有设置：

```ts
missing: true
```

所以 UI 可能把不存在的推荐文件显示成普通文件，点击后才进入加载错误。

### 目标状态

推荐文件如果不存在，应在 UI 上直接显示：

```text
已不存在
```

并且点击时不再发起文件读取。

### 开发范围

需要改：

- `apps/desktop/src/features/project-files/ProjectLocalFilesPage.tsx`
- `apps/desktop/src/features/project-files/ProjectFileBrowser.tsx`
- `apps/desktop/src/features/project-files/projectFileTypes.ts`

### 行为要求

当推荐文件路径找不到对应 entry：

1. recommended row 设置 `missing: true`。
2. UI 显示“已不存在”。
3. 点击 missing 推荐项时，不调用 `onSelectFile`。
4. 可以显示轻提示：`推荐文件已不存在，可能是 Graph 还未刷新。`
5. 不报 raw error。

建议 fallback row：

```ts
return {
  name,
  relativePath: normalizedPath,
  kind: "file",
  createdAt: null,
  modifiedAt: null,
  sizeBytes: null,
  childCount: null,
  isSymlink: false,
  extension: getProjectFileExtensionFromName(name),
  children: [],
  depth: 0,
  missing: true,
};
```

选择推荐项时需要短路：

```ts
if (row.missing) {
  return;
}
```

### 验收标准

- [ ] 推荐文件不存在时显示“已不存在”。
- [ ] missing 推荐项点击后不会调用 `load_project_file_content`。
- [ ] missing 推荐项不会导致 reader 进入 raw error。
- [ ] missing 推荐项仍显示原始路径作为 title。
- [ ] 如果 Graph 刷新后文件恢复存在，重新加载后可正常打开。

---

## 总验收标准

这 3 个修复项完成后，Project File Reader V1 从“主体完成”收口为“完整完成”：

- [ ] 大文本 reader 真正接入 `load_project_file_text_range`。
- [ ] 大文本 reader 支持上一段 / 下一段 / 跳到行号。
- [ ] Graph 推荐文件来源可解释。
- [ ] Graph `recommendedFiles / recommendedTests / importantFiles` 都能进入推荐区。
- [ ] 推荐文件不存在时显示 missing。
- [ ] missing 推荐文件点击不触发读取。
- [ ] 所有修复保持只读。
- [ ] 不执行命令。
- [ ] 不调用模型。
- [ ] 不写源码。
- [ ] 不写 `.agentflow/` runtime。

## 验证命令

- `cargo fmt --check`
- `cargo test`
- `npm --prefix apps/desktop run build`
- `git diff --check`

如果补前端单测，建议增加：

- `LargeTextReader range loading test`
- `recommended missing file row test`
- `recommended file source label test`

## 建议开发切片

### Slice 1：LargeText Range 前端接入

目标：

- `useProjectFiles` 增加 `loadProjectFileTextRange`。
- `LargeTextReader` 支持按行加载。
- UI 提供上一段 / 下一段 / 跳转行号。

验收：

- 大文本不只依赖首段 content。

### Slice 2：Recommended File Model

目标：

- 把 `recommendedFilePaths` 升级成 `recommendedFiles`。
- 每个推荐项带 `source / reason / status`。
- 推荐区显示来源。

验收：

- `context-pack-file / context-pack-test / manifest-important` 都能显示。

### Slice 3：Missing Recommended File

目标：

- 推荐路径找不到 entry 时标记 missing。
- 点击 missing 不发起文件读取。
- UI 显示“已不存在”。

验收：

- 不出现 raw file loading error。
