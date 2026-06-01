# 003 - Project File Reader V1 Completion

创建日期：2026-06-01
执行者：Codex

## 用户目标

当前 Project 文件阅读器已经可以打开本地项目、展示文件树、点击文件并渲染内容，但整体仍是轻量版。

本需求的目标是：

```text
把 Project File Reader 从“可用的本地文件预览器”补齐成“稳定可依赖的项目文件阅读器”。
```

大白话：

> Project Workspace Manager 负责把项目接进来。
> Graph 负责整理代码现场。
> Project File Reader 负责让用户和 AgentFlow 清楚看到项目里的真实文件和内容。
> 这一版不做 IDE，不编辑文件，不执行命令，只把“读项目文件”这件事做稳。

---

## 背景

当前代码里，项目文件阅读器已经独立为 feature module：

```text
apps/desktop/src/features/project-files/
  index.ts
  ProjectLocalFilesPage.tsx
  ProjectFileReader.tsx
  ProjectFileBrowser.tsx
  FileRendererRegistry.tsx
  ProjectFiles.css
  useProjectFiles.ts
  projectFileUtils.ts
  projectFileTypes.ts

apps/desktop/src-tauri/src/project_files.rs
```

当前实现已经具备：

- 本地项目文件快照读取。
- 文件内容读取。
- 系统文件夹选择。
- 路径逃逸防护。
- 目录概览。
- Markdown / Code / JSON / Large Text / Table / PDF / Image / Media / DOCX / Binary fallback 等多格式 reader。
- 浏览器预览 mock。
- 真实 Tauri 客户端只读边界。

但仍有以下轻量化问题：

- 文件树只做基础加载，没有完整的虚拟化 / 分页 / 快速定位。
- 目录 children 有数量上限，但没有“继续加载”机制。
- 文件搜索、快速打开、路径过滤还没有。
- 语言识别和代码高亮覆盖不够完整。
- 大文件 / 二进制 / Office / PDF 仍是预览级能力。
- 文件加载没有独立的 request id / abort / out-of-order 防护。
- 最近打开文件、展开状态、选中文件没有按 Project 持久化。
- 没有和 Graph 的推荐文件形成联动。
- symlink、权限错误、超大目录、异常文件系统情况需要更明确的策略。

---

## 范围

本需求补齐 10 个能力：

1. 文件树加载稳定化
2. 目录分页 / 虚拟化
3. 文件搜索 / 快速打开
4. Project 文件视图模式
5. 文件内容加载状态管理
6. Renderer 覆盖增强
7. 大文件 / 二进制预览策略增强
8. Graph 推荐文件联动
9. 安全边界和异常处理增强
10. 阅读器状态持久化

---

## 非目标

本需求不做以下事情：

- 不编辑文件。
- 不保存文件。
- 不执行命令。
- 不运行测试。
- 不调用模型。
- 不创建 Goal / Milestone / Issue。
- 不创建远程 PR、GitHub issue 或 Linear issue。
- 不做完整 IDE。
- 不做 LSP。
- 不做调试器。
- 不做终端。
- 不做 Office 级文档编辑。
- 不做完整图谱可视化。
- 不替代 Graph。

---

# 1. 当前版本能力梳理

## 1.1 Tauri 文件读取后端

当前后端提供 3 个 Tauri command：

```text
load_project_files_snapshot
load_project_file_content
choose_existing_project_folder
```

当前能力：

- 可以选择本地文件夹。
- 可以读取项目根目录文件列表。
- 可以读取文件内容。
- 可以读取目录 children。
- 可以根据扩展名判断 language / mime。
- 可以限制文本预览最大 512KB。
- 可以对二进制文件生成十六进制 fallback。
- 可以为小型图片 / PDF / DOCX / XLSX / 音视频生成 data URL。
- 可以拒绝 `../`、绝对路径、路径逃逸。

当前问题：

- 目录 children 固定最多 80 个，没有分页。
- 文件树不是完整递归树，只是 top-level + 当前展开目录。
- 语言识别偏 Web / Rust / 文档，不覆盖 Graph 的主流语言矩阵。
- 没有单独的 symlink 策略。
- 没有文件过滤 / 搜索 API。
- 没有“只读业务文件视图”和“全部文件视图”的区分。
- 大文件只能预览前 512KB，没有 range 加载。
- data URL 对二进制文件有大小限制，超大 PDF / 图片 / Office 文件只能 fallback。

---

## 1.2 前端 Hook：useProjectFiles

当前能力：

- 根据 selectedProjectRoot 加载文件快照。
- 根据 selectedPath 加载文件内容。
- 浏览器预览中使用 mock 数据。
- 真实 Tauri 失败时展示真实错误，不使用 mock fallback。
- 选择文件后更新 content / selectedPath。

当前问题：

- 没有 request id / abort controller，快速切换文件时可能出现旧请求覆盖新请求。
- 文件内容加载状态只有全局 `source`，没有 per-file loading。
- 没有最近打开文件。
- 没有按 Project 保存展开目录 / 选中文件。
- 刷新后可能丢失上下文。
- 错误态较粗，没有区分权限、路径失效、文件被删除、文件过大、编码不支持等原因。

---

## 1.3 文件树：ProjectFileBrowser

当前能力：

- 右侧文件列表展示文件和目录。
- 目录可以展开 / 收起。
- 目录行显示 chevron。
- 点击文件或目录后，主体区域加载文件内容或目录概览。
- 显示隐藏文件、普通文件、源码目录、构建目录。
- 不隐藏 `.git`、`.agentflow`、`target`、dotfiles。

当前问题：

- 没有搜索。
- 没有筛选。
- 没有排序切换。
- 没有虚拟滚动。
- 没有超大目录分页。
- children 只有 name / path / kind，缺少完整 metadata。
- 没有 breadcrumb。
- 没有键盘导航。
- 没有“业务源码视图 / 全部文件视图 / 最近文件视图”。

---

## 1.4 内容阅读器：ProjectFileReader + FileRendererRegistry

当前支持：

- Markdown：`react-markdown` + `remark-gfm`
- Code：`Shiki`
- JSON：pretty print
- Large Text：虚拟化行窗口
- Plain Text
- CSV / TSV
- XLSX：SheetJS 预览首个 worksheet 前 80 行
- PDF：PDF.js 预览第一页
- Image
- Audio / Video
- DOCX：mammoth.js 转 HTML
- Binary fallback
- Unsupported fallback

当前问题：

- Code 高亮语言覆盖较少。
- PDF 只预览第一页。
- XLSX 只预览第一个 sheet。
- DOCX 只是 HTML 预览，没有目录 / 页级概念。
- CSV 解析是简单 split，不处理引号、转义、换行单元格。
- 不支持 SQL / Python / Go / Java / Kotlin / Swift / Dart / C# / C++ / PHP / Ruby 等主流语言高亮映射。
- 不支持 plist / Gradle / AndroidManifest / storyboard / xib 的结构化阅读。
- 没有文件内搜索。
- 没有复制路径 / 复制内容片段。
- 没有跳转到行号。

---

# 2. 目标状态

本需求完成后，Project File Reader 应达到：

```text
稳定打开项目
稳定加载文件树
稳定加载文件内容
能处理大项目
能处理大目录
能处理多种文件格式
能快速搜索和打开文件
能和 Graph 推荐上下文联动
保持只读、不执行、不写源码
```

一句话：

> Project File Reader V1 Completion = 稳定、快速、可搜索、可过滤、可恢复状态的只读项目文件阅读器。

---

# 3. 文件树加载稳定化

## 目标

文件树需要从轻量列表升级为稳定的项目文件导航。

## 行为

文件树需要支持：

- 根目录加载。
- 子目录懒加载。
- 大目录分页。
- 超大项目不一次性读取所有文件。
- 跳过不可读目录时不崩溃。
- 文件删除 / 移动后显示可理解错误。
- 保留当前选中文件。
- 刷新后尽量恢复展开状态。

## 建议新增后端模型

```rust
ProjectDirectoryPage {
    version: String,
    project_root: String,
    directory_path: String,
    entries: Vec<ProjectFileEntry>,
    next_cursor: Option<String>,
    total_hint: Option<usize>,
    truncated: bool,
}
```

## 建议新增 Tauri command

```text
load_project_directory_page
```

输入：

```json
{
  "projectRoot": "/path/to/project",
  "directoryPath": "src",
  "cursor": null,
  "limit": 200
}
```

## 验收标准

- [ ] 打开有上千文件的项目不会卡死。
- [ ] 打开有大量 children 的目录不会一次性渲染全部。
- [ ] 超过 limit 时返回 `nextCursor`。
- [ ] 无权限目录显示明确错误或空态，不导致页面崩溃。
- [ ] 文件树继续保持只读。

---

# 4. 文件树虚拟化 / 分页

## 目标

文件树要能承载大型项目。

## 行为

前端文件树需要支持：

- 虚拟滚动或分段渲染。
- 大目录分页加载。
- 展开目录时只加载当前目录 children。
- 同一目录 children 可追加加载。
- 展开 / 收起状态稳定。

## 非目标

- 不做 IDE 级文件树。
- 不做拖拽。
- 不做重命名。
- 不做创建 / 删除文件。

## 验收标准

- [ ] 5000 个文件的 fixture 项目可以打开。
- [ ] 1000 个 children 的目录可以分页显示。
- [ ] 展开 / 收起不卡顿。
- [ ] 滚动文件树不会明显掉帧。
- [ ] 不写任何项目文件。

---

# 5. 文件搜索 / 快速打开

## 目标

用户和后续 AgentFlow UI 能快速定位文件。

## 行为

支持：

- 按文件名搜索。
- 按路径搜索。
- 按扩展名过滤。
- 支持最近打开文件。
- 搜索结果点击后打开对应文件。
- 搜索只读，不写索引。

## 与 Graph 的关系

文件阅读器搜索可以先基于当前文件快照 / 后端文件扫描实现。

如果 Graph 已 ready，可以优先调用 Graph 搜索结果，但不能强依赖 Graph。

```text
Graph ready:
  文件搜索可融合 Graph 搜索

Graph missing / indexing / failed:
  文件搜索仍可用，只使用文件树数据
```

## 建议新增 Tauri command

```text
search_project_files
```

输入：

```json
{
  "projectRoot": "/path/to/project",
  "query": "lease",
  "limit": 50,
  "kind": "all"
}
```

输出：

```json
{
  "version": "project-file-search.v1",
  "query": "lease",
  "results": [
    {
      "path": "crates/core/src/lease.rs",
      "name": "lease.rs",
      "kind": "file",
      "extension": "rs",
      "score": 0.92
    }
  ]
}
```

## 验收标准

- [ ] 可以搜索文件名。
- [ ] 可以搜索路径片段。
- [ ] 可以按扩展名过滤。
- [ ] 搜索结果点击后打开文件。
- [ ] Graph 不可用时搜索仍可用。
- [ ] 搜索不调用模型。

---

# 6. Project 文件视图模式

## 目标

当前文件树展示所有文件，包括 `.git`、`.agentflow`、`target`、dotfiles。下一版需要区分“普通项目阅读”和“完整本地查看”。

## 视图模式

新增三种视图：

```text
Source View
  只显示源码、文档、配置、测试、重要资源

All Files View
  显示所有文件，包括 .git、.agentflow、target、dotfiles

Recent View
  显示最近打开文件
```

默认模式建议：

```text
Source View
```

原因：

> 用户打开项目主要是看正常代码和文档。
> `.git`、`target`、`.agentflow`、`node_modules` 这类文件不是不能看，但不应该默认淹没正常项目文件。

## 注意

这会调整现有设计中“显示 hidden / build dirs”的策略，因此要在需求文档中重新授权。

## 验收标准

- [ ] 默认显示 Source View。
- [ ] 可以切换 All Files View。
- [ ] All Files View 能看到 `.git`、`.agentflow`、`target` 等。
- [ ] Source View 默认隐藏 `.git`、`.agentflow`、`node_modules`、`target`、`dist`、`build`。
- [ ] Recent View 显示最近打开文件。
- [ ] 模式切换只影响 UI 展示，不删除、不写文件。

---

# 7. 文件内容加载状态管理

## 目标

快速点击多个文件时，不应出现旧文件内容覆盖新选择的问题。

## 行为

`useProjectFiles` 需要增强：

- 每次文件内容请求生成 request id。
- 只允许最后一次请求更新 content。
- 快速切换文件时旧请求结果丢弃。
- 每个 selectedPath 有 loading 状态。
- 文件被删除时显示“文件不存在或已移动”。
- 权限不足时显示“无权限读取”。
- 二进制 / 编码不支持时显示 fallback。

## 验收标准

- [ ] 快速点击多个文件，最终显示最后点击的文件。
- [ ] 慢请求不会覆盖新请求。
- [ ] 文件加载中有明确 loading。
- [ ] 文件不存在有明确错误。
- [ ] 权限错误有明确错误。
- [ ] 不使用 browser mock fallback 处理真实 Tauri 错误。

---

# 8. Renderer 覆盖增强

## 目标

阅读器需要覆盖 Graph 支持的主流语言和项目类型。

## 需要增强的代码高亮语言

至少补齐：

```text
Python
Go
Java
Kotlin
Swift
Dart
C
C++
C#
PHP
Ruby
SQL
PowerShell
Objective-C
Gradle
plist
XML
Dockerfile
```

## 结构化阅读增强

建议新增轻量结构化 reader：

```text
XmlReader
PlistReader
GradleReader
SqlReader
ManifestReader
PubspecReader
PackageJsonReader
```

它们仍然只读，不做编辑。

## 验收标准

- [ ] `.py` 使用代码阅读器。
- [ ] `.go` 使用代码阅读器。
- [ ] `.java` / `.kt` 使用代码阅读器。
- [ ] `.swift` / `.m` / `.mm` 使用代码阅读器。
- [ ] `.dart` 使用代码阅读器。
- [ ] `.sql` 使用代码阅读器或 SQL reader。
- [ ] `AndroidManifest.xml` 显示为 Android manifest 配置。
- [ ] `Info.plist` 显示为 plist 配置。
- [ ] `pubspec.yaml` 显示为 Flutter 配置。
- [ ] 未识别语言仍有 plain text fallback。

---

# 9. 大文件 / 二进制预览策略增强

## 目标

大文件和二进制文件不能让 UI 卡顿，也不能空白。

## 行为

后端需要明确文件预览策略：

```text
text <= 512KB:
  直接文本预览

text > 512KB:
  返回头部 preview + truncated=true
  后续可支持 range page

binary <= preview limit:
  可生成 dataUrl 或 hex

binary > preview limit:
  metadata + hex preview + no dataUrl
```

下一版增强：

- 增加 range preview API。
- 支持读取某个文本行范围。
- 支持 PDF 页码切换的基础 API。
- 支持 XLSX sheet 列表和选 sheet。
- 支持图片缩略图尺寸限制。
- 对 data URL 大小做明确限制，防止前端内存暴涨。

## 建议新增 Tauri command

```text
load_project_file_text_range
```

输入：

```json
{
  "projectRoot": "/path/to/project",
  "relativePath": "logs/huge.log",
  "startLine": 1000,
  "lineCount": 200
}
```

## 验收标准

- [ ] 大文本不会一次性塞进 DOM。
- [ ] 大文本可以按行范围加载。
- [ ] 超大 PDF 不生成巨大 data URL。
- [ ] 超大图片不造成页面卡死。
- [ ] XLSX 至少能展示 sheet 名称和首个 sheet。
- [ ] 所有 fallback 都有明确说明，不空白。

---

# 10. Graph 推荐文件联动

## 目标

Graph 已经能生成 Context Pack。Project File Reader 应能展示或打开 Graph 推荐文件，但不变成 Graph UI。

## 行为

在 Project 文件阅读器中增加一个轻量入口：

```text
推荐文件
```

来源：

```text
GraphContextPack.recommendedFiles
GraphSearch results
```

能力：

- 点击推荐文件后打开。
- 如果文件不存在，显示“推荐文件已不存在”。
- Graph 未 ready 时隐藏或显示“代码地图建立中”。

## 非目标

- 不做图谱视图。
- 不做复杂搜索页面。
- 不做 Agent 任务执行。
- 不做代码问答。

## 验收标准

- [ ] Graph ready 且有 Context Pack 时显示推荐文件。
- [ ] 点击推荐文件能打开对应文件。
- [ ] Graph missing / indexing 时不阻塞文件阅读器。
- [ ] 推荐文件只读展示。
- [ ] 不调用模型。

---

# 11. 安全边界和异常处理增强

## 目标

本地文件读取必须稳，不应因为异常路径、symlink、权限、编码、文件删除导致不可控行为。

## 行为

后端需要明确策略：

### 路径逃逸

继续保持：

```text
拒绝绝对路径
拒绝 ..
拒绝 RootDir / Prefix
canonical target 必须在 canonical root 内
```

### Symlink

新增策略：

```text
symlink 指向 root 内:
  可以读取，并标记 isSymlink=true

symlink 指向 root 外:
  默认拒绝读取，并返回 explicit reason
```

### 权限

```text
无法读取目录:
  返回目录空态 + reason

无法读取文件:
  返回 explicit error
```

### 编码

```text
非 UTF-8:
  binary fallback 或 encoding fallback
```

## 验收标准

- [ ] symlink 指向项目外部时不能读取。
- [ ] symlink 指向项目内部时可以读取并标记。
- [ ] 权限错误有明确错误消息。
- [ ] 文件读取过程中被删除时有明确错误。
- [ ] 不出现未捕获异常导致页面崩溃。

---

# 12. 阅读器状态持久化

## 目标

每个 Project 的文件阅读状态应可恢复。

## 本地状态

保存到 localStorage：

```text
agentflow.projectFileReaderState.v1
```

保存内容：

```text
projectRoot
selectedPath
expandedPaths
viewMode
recentFiles
lastOpenedAt
```

不保存：

```text
文件内容
二进制内容
dataUrl
源码片段
```

## 验收标准

- [ ] 切换 Project 后能恢复该 Project 的 selectedPath。
- [ ] 能恢复 expandedPaths。
- [ ] 能记录最近打开文件。
- [ ] 不把文件内容写入 localStorage。
- [ ] 浏览器预览和真实 Tauri 都能使用该 UI 状态。

---

# 13. 页面 / 功能

Project 页面保持当前结构：

```text
左侧 App Sidebar
顶部 Topbar
主体 Project File Reader
右侧 Project File Browser
底部 Agent Status Channel
```

增强点：

- 文件树增加搜索 / 视图模式。
- 文件树支持分页 / 虚拟化。
- Reader 增加更完整加载状态。
- Reader 支持更多语言和配置文件。
- Reader 支持 Graph 推荐文件入口。
- Status Channel 可显示文件阅读器状态。

不新增：

- 多 Tab IDE。
- 编辑器。
- 终端。
- 调试器。
- 保存按钮。
- 命令按钮。

---

## 数据来源

- Tauri command: `load_project_files_snapshot`
- Tauri command: `load_project_file_content`
- 新增 Tauri command: `load_project_directory_page`
- 新增 Tauri command: `search_project_files`
- 新增 Tauri command: `load_project_file_text_range`
- Graph Context Pack: `.agentflow/output/graph/context-packs/*.json`
- Client local state: `agentflow.projectFileReaderState.v1`

---

## 交互边界

允许：

- 读取项目文件。
- 读取目录。
- 读取 metadata。
- 读取 Graph Context Pack。
- 写 UI localStorage 状态。

不允许：

- 修改源码。
- 删除文件。
- 新建文件。
- 重命名文件。
- 执行命令。
- 调用模型。
- 上传文件。
- 创建远程对象。
- 把文件内容写入 localStorage。

---

## 验收标准总表

- [ ] 文件树支持大目录分页。
- [ ] 文件树支持虚拟化或分段渲染。
- [ ] 文件搜索 / 快速打开完成。
- [ ] Source View / All Files View / Recent View 完成。
- [ ] 快速切换文件不会出现旧请求覆盖新内容。
- [ ] per-file loading / error 状态完成。
- [ ] 主流语言代码高亮映射补齐。
- [ ] Android / iOS / Flutter 配置文件能被正确识别和展示。
- [ ] 大文本支持 range preview。
- [ ] 二进制和超大文件不会导致页面卡死。
- [ ] Graph 推荐文件可以被打开。
- [ ] symlink 策略明确且有测试。
- [ ] 权限 / 文件删除 / 编码异常都有明确 fallback。
- [ ] Project 级 selectedPath / expandedPaths / recentFiles 可恢复。
- [ ] 不执行命令、不写源码、不调用模型。
- [ ] 浏览器预览 mock 只在无 Tauri runtime 时启用。

---

## 验证命令

- `cargo fmt --check`
- `cargo test`
- `npm --prefix apps/desktop run build`
- `git diff --check`

建议新增针对 Tauri 后端的测试：

- 大目录分页测试
- 路径逃逸测试
- symlink root 内 / root 外测试
- 大文本 range 测试
- 二进制 fallback 测试
- 搜索文件测试

---

## 建议开发切片

### Slice 1：Directory Page API

目标：

- 新增 `load_project_directory_page`。
- 支持 limit / cursor。
- 后端测试大目录分页。

### Slice 2：File Tree UI Virtualization

目标：

- 前端文件树支持虚拟化或分段渲染。
- 支持加载更多 children。

### Slice 3：File Search / Quick Open

目标：

- 新增 `search_project_files`。
- 前端增加文件搜索输入。
- 搜索结果点击打开。

### Slice 4：View Modes

目标：

- Source View / All Files View / Recent View。
- localStorage 持久化 viewMode。

### Slice 5：Content Loading Robustness

目标：

- request id / stale response guard。
- per-file loading / error。
- 文件删除、权限错误、编码错误细分。

### Slice 6：Renderer Coverage

目标：

- 补主流语言高亮映射。
- 补移动端配置文件 reader。
- 补 SQL / plist / Gradle / Manifest reader。

### Slice 7：Large File Strategy

目标：

- 新增 text range API。
- 强化 data URL 大小限制。
- 大文本分页预览。

### Slice 8：Graph Recommended Files

目标：

- 读取 Context Pack。
- 显示推荐文件入口。
- 点击打开推荐文件。

### Slice 9：Security + Symlink

目标：

- 明确 symlink 策略。
- 权限和路径异常测试补齐。

### Slice 10：Reader State Persistence

目标：

- 保存 selectedPath / expandedPaths / recentFiles / viewMode。
- 不保存文件内容。

---

## 完成定义

本需求完成后，Project File Reader 的目标状态应为：

| 能力 | 目标状态 |
| --- | --- |
| 本地项目文件读取 | 完成 |
| 目录懒加载 | 完成 |
| 大目录分页 | 完成 |
| 文件树虚拟化 | 完成 |
| 文件搜索 / 快速打开 | 完成 |
| 多视图模式 | 完成 |
| 文件内容加载稳定性 | 完成 |
| 主流 renderer 覆盖 | 完成 |
| 大文件 / 二进制策略 | 完成 |
| Graph 推荐文件联动 | 完成 |
| 安全边界 / symlink | 完成 |
| 阅读状态持久化 | 完成 |

最终一句话：

> Project File Reader V1 Completion 不是把 AgentFlow 做成 IDE，而是把“只读项目文件阅读”做成稳定底座，让后续 Graph、Goal Tree 和 AgentRun 都能依赖它。
