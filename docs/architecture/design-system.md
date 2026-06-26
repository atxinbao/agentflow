# AgentFlow App Design System

Updated: 2026-06-01
Executor: Codex

## Design Baseline

AgentFlow 的整体设计以当前完成的 `Project Local Files` 页面为基准。

这是一个本地优先的开发者工作台，不是营销页、项目管理 SaaS 或聊天界面。视觉目标是：安静、紧凑、稳定、可长时间阅读，优先支持本地事实源浏览、任务合同审查和证据核对。

## Figma SVG Style Frontend Foundation V1

Updated: 2026-06-06
Executor: Codex

本轮把 Figma Agentflow v3 的 SVG 视觉语言抽象成前端基础层，不把整页 SVG 嵌入前端。当前实现只重组样式和组件，不新增业务能力，不改数据模型，不新增 Tauri command，不写 `.agentflow/`。

### Token Source

统一 token 入口：

```text
apps/desktop/src/styles/tokens.css
apps/desktop/src/styles/components.css
apps/desktop/src/design/tokens.css
apps/desktop/src/design/index.css
```

`apps/desktop/src/design/tokens.css` 现在只作为兼容入口导入 `styles/tokens.css`。后续新增页面优先使用 `--af-*`，旧 `v16-*` 页面变量只做兼容映射。

核心 token：

| Token | Usage |
| --- | --- |
| `--af-bg` | app / workspace 背景 |
| `--af-surface` | 顶栏、侧栏、主面板背景 |
| `--af-surface-muted` | 列表行、嵌套面板、弱背景 |
| `--af-border` | 常规边框和结构线 |
| `--af-border-strong` | 强边框和选中结构线 |
| `--af-text` | 主文本 |
| `--af-text-muted` | 辅助文本 |
| `--af-accent` | 主动作、选中态、焦点 |
| `--af-success` | ready / done |
| `--af-warning` | warning / review |
| `--af-danger` | blocked / failed / high risk |
| `--af-radius` | 基础圆角，当前为 `8px` |
| `--af-header-height` | 顶部栏高度 |
| `--af-sidebar-width` | 左侧栏宽度 |
| `--af-statusbar-height` | 底部状态栏高度 |

Light 模式以白色和浅灰结构线为主。Dark 模式以深色工作台为主。两种模式都保持直角工作台结构、低噪音边框和紧凑信息密度。

### Foundation Components

新增基础组件入口：

```text
apps/desktop/src/components/ui/index.tsx
```

已抽象组件：

- `AppFrame`
- `WindowChrome`
- `TopBar`
- `Sidebar`
- `PageHeader`
- `Panel`
- `Section`
- `ListPanel`
- `ListRow`
- `StatusBadge`
- `RiskBadge`
- `ActionButton`
- `ReadOnlyBadge`
- `StatusBar`

已有 `EmptyState` 继续作为基础组件使用，不重复新增同名实现。

### Applied Pages

已试点页面：

- 登录页：使用 `WindowChrome`、`ListRow`、`ActionButton`。
- 首次引导页：使用 `WindowChrome`、`ActionButton`。
- 工作台页：使用 `AppFrame`、`TopBar`、`Sidebar`、`StatusBar`、`PageHeader`、`Panel`、`StatusBadge`、`ReadOnlyBadge`。
- 任务页：任务看板使用 `ListPanel` / `ListRow`，任务详情使用 `StatusBadge` / `RiskBadge` / `Section` / `ActionButton`。

未覆盖页面：

- 文件页仍由 `features/project-files/ProjectFiles.css` 维护。
- 交付页和审计页只接入了部分 Badge / Button，布局仍保留 `v16-*` 兼容层。
- Companion 区域已改用 `Panel`，但后续还需要按真实窄窗口体验继续拆分。
- 高级详情页和 Design System Preview 仍保留旧预览内容，后续应补齐 Foundation 组件矩阵。

### Implementation Rules

- 不把整页 SVG 作为背景或 UI。
- 不新增装饰性大卡片。
- 不新增复杂动画。
- 页面级布局优先使用清晰结构线、固定高度、固定栏宽和一致间距。
- 后续页面迁移先接入基础组件，再删除局部一次性 CSS。
- Tauri 真实数据读取和浏览器 mock 数据必须继续走现有 hooks，不在组件层做数据分支。

## Product Model

默认 Workspace 是系统隐含容器，不进入用户模板。

```text
Workspace
  Project
    Milestone
    Issue
    View
```

- Project：顶层业务容器。
- Milestone：Project 内的阶段容器，以时间线 / Gantt-lite 方式呈现。
- Issue：唯一可执行单元，承载 Agent 执行合同。
- View：Project 内的保存视图 / 过滤入口。

Desktop 客户端默认只读：不执行命令，不写入工作区，不创建远程 PR，不调用模型。执行、验证、审查由授权 Agent / 后端 / CLI / CI 负责。

## App Shell

全局布局必须复用同一个 shell。

- Sidebar：固定左栏，宽度 `292px`。
- Workspace：右侧工作区，填满剩余空间。
- Topbar：工作区顶部固定高度 `72px`。
- Content：Topbar 下方内容区，独立滚动。
- Statusbar：工作区底部固定高度 `34px`，系统级状态栏，不属于单个页面内容。
- 页面切换只替换 Content，不重写 Sidebar、Topbar 和 Statusbar。

```text
┌──────────── Sidebar 292 ────────────┬────────────── Workspace ──────────────┐
│ Brand / Projects / Footer           │ Topbar 72                             │
│                                     ├───────────────────────────────────────┤
│                                     │ Page Content                          │
│                                     ├───────────────────────────────────────┤
│                                     │ Statusbar 34                          │
└─────────────────────────────────────┴───────────────────────────────────────┘
```

### Agent Status Channel

底部状态栏是 app shell 的一部分，用于承载 Agent 作业相关的系统级状态。后续涉及 Agent 作业、准备、索引、执行现场、验证现场等状态，都应接入同一个 `features/status-channel` 模块，而不是在页面内容里单独实现状态卡。

当前接入项：

- `工作空间`：由 `Project Workspace Manager V0.2` 提供，表示本地项目文件和工作区资源是否已准备。
- `工作现场`：由 `002 - Graph V1` 提供，表示代码地图 / 工作现场索引是否已准备。

状态栏可见内容一次只显示一个“通道事件 + 状态”，不并列展示多个通道，也不直接平铺模块指标。

```text
● 工作空间 · 已就绪
● 工作现场 · 已就绪
```

上面的两行是不同状态事件的展示格式示例；真实状态栏同一时刻只显示其中一个。选择规则为：异常优先，其次警告、处理中、已就绪、空闲；同一状态级别下按通道优先级选择当前最相关的 Agent 作业状态。

资源数、选中文件、文件数、符号数、关系数、语言列表、错误详情等进入状态项详情提示，不作为底部栏常驻文本。

- Height: `34px`
- Background: `#17181c`
- Border top: `1px solid #343842`
- Padding: `0 16px`
- Typography: `12px / 16px`
- Item label examples: `工作空间`, `工作现场`
- Status label examples: `已就绪`, `准备中`, `未就绪`, `异常`
- Metrics: 每个状态 item 可保留详情数据，例：资源、选中文件、文件、符号、关系、语言；默认不在状态栏平铺展示。
- Ready indicator: `#60d394`
- Working indicator: `#82aaff`
- Warning indicator: `#f6c177`
- Failed indicator: `#ff7b86`
- Error detail: 进入状态项详情提示。

## Color Tokens

当前代码使用以下 app 级 token。新增页面优先复用这些颜色，不新增临时主题。

| Token | Value | Usage |
| --- | --- | --- |
| `--af-sidebar-bg` | `#1f1f22` | Sidebar base |
| `--af-workspace-bg` | `#1b1b1f` | Workspace base |
| `--af-topbar-bg` | `#1f1f22` | Topbar base |
| `--af-panel-bg` | `#202126` | Reader / panels |
| `--af-panel-inner-bg` | `#1b1c20` | Nested quiet surfaces |
| `--af-code-bg` | `#17181c` | Code blocks |
| `--af-hover-bg` | `#292a2f` | Hover states |
| `--af-border` | `#343842` | Strong borders |
| `--af-border-soft` | `#2b2f38` | Internal dividers |
| `--af-button-bg` | `#24252a` | Icon buttons |

Core text colors:

- Primary text: `#f4f7fb`
- Secondary text: `#c9d4e4`
- Muted text: `#8c98aa`
- Sidebar text: `#b7beca`
- Active project text: `#a9c3ff`
- Active project background: `#2f4b7a`
- Active project left rail: `#8db3ff`

File type colors:

- Folder: `#72b8ff`
- Hidden file / hidden folder: `#b68cff`
- Markdown / document: `#7dd3fc`
- Code / config: `#f6c177`
- JSON / data: `#86efac`
- Plain file: `#aab6c6`

## Typography

Use system sans fonts for UI and system monospace for code.

```css
font-family: Inter, PingFang SC, Hiragino Sans GB, Microsoft YaHei, system-ui, sans-serif;
font-family: SFMono-Regular, JetBrains Mono, Menlo, Consolas, monospace;
```

Letter spacing stays `0`. Do not use viewport-scaled font sizes.

| Element | Size / Line | Weight |
| --- | --- | --- |
| Brand text | `24px / 30px` | 800 |
| Sidebar section title | `18px / 24px` | 500 |
| Sidebar project | `17px / 24px` | 500 |
| Sidebar child item | `16px / 22px` | 500 |
| Topbar title | `15px / 21px` | 700 |
| Topbar path | `12px / 16px` | 400 |
| Reader file title | `17px / 22px` | 800 |
| Reader file type label | `11px / 14px` | 600 |
| Reader metadata | `11px / 15px` | 700 |
| Body text | `14px / 24px` | 400 |
| Markdown H1 | `26px / 34px` | 800 |
| Markdown H2 | `20px / 28px` | 800 |
| Markdown H3/H4 | `16px / 24px` | 800 |
| File list row | `13px / 20px` | 400 |
| Code reader | `13px / 22px` | 600 |

## Sidebar

Sidebar 是全 app 的主导航，所有页面必须保持一致。

- Width: `292px`
- Background: `#1f1f22`
- Horizontal padding: `16px`
- Brand area: `72px` high, content vertically centered.
- Brand mark: `40px x 40px`, radius `8px`, background `#a9c3ff`, folder color `#07366d`.
- Brand text: one line only; no subtitle in the Project shell.
- Project section title `所有项目` 左右与项目卡片边缘对齐。
- Add Project 使用文件夹加号图标，不使用普通 `+` 文本按钮。
- Add Project opens the native folder picker in the real Tauri client.
- In browser preview, Add Project opens a compact inline path panel because the system folder picker is unavailable.
- Inline panel title: `添加项目`.
- Inline panel helper: `桌面客户端会打开系统文件夹选择器；浏览器预览可输入路径模拟。`
- Inline panel controls: one local path input, `添加`, `取消`.
- After a folder/path is selected, the Sidebar shows the selected folder basename as the Project name.
- Add Project 必须按标准化本地路径校验是否已存在；如果该文件夹已加载，不新增重复 Project，直接切换并展开已有 Project。
- Added Projects persist only as Desktop UI state in `localStorage` key `agentflow.localProjectFolders.v1`.
- Add Project does not write `.agentflow/`, does not write the workspace, and does not create remote objects.
- Expanded Project 使用 active blue background and left rail.
- Project expand/collapse is controlled per Project. Multiple Projects may remain expanded at the same time.
- Project children are visually nested but left edge remains aligned with the project group.
- Footer holds `Settings` and `Docs`; keep it pinned to the bottom.

Sidebar must support multiple Projects. Never design the app as single-project-only.

## Topbar

Topbar 只承载当前 Project 的 title/path 和右侧小型命令。

- Height: `72px`
- Background: `#1f1f22`
- Left padding: `22px`
- Right padding: `10px`
- Title: `15px`
- Path: `12px`
- Refresh button: `30px x 30px`
- Refresh button right edge aligns with the right file panel outer frame.
- On the Project page, refresh only reloads the current Project file snapshot and current selected file content.
- On non-Project pages, refresh may reload the broader local workbench snapshot.

Topbar 不显示 Project Runtime、progress cards、tabs 或营销式标题区。

## Project Page

Project 页面定稿为本地文件阅读器。

Project 页面不再展示：

- Goal 模板
- Milestone 模板
- Architecture / Environment / Agent tabs
- Project Runtime progress cards
- Issue / View 的详情内容

Project 页面只负责：

- 展示当前 Project 本地路径。
- 展示右侧本地文件树。
- 点击文件后在主体阅读器展示内容。
- 点击文件夹后在主体阅读器展示目录概览。
- 保持只读，不执行/不写入。

### Project Module Boundary

Project 本地文件阅读器已经独立为 feature module：

```text
apps/desktop/src/features/project-files/
  index.ts
  ProjectLocalFilesPage.tsx
  ProjectFileReader.tsx
  ProjectFileBrowser.tsx
  ProjectFiles.css
  useProjectFiles.ts
  projectFileUtils.ts
  projectFileTypes.ts
apps/desktop/src-tauri/src/project_files.rs
```

规则：

- `App.tsx` 只通过 `features/project-files/index.ts` 使用 Project 文件模块；它负责 app shell、Project 选择、页面路由和非 Project 页面数据加载，不继续承载 Project reader 组件细节。
- `useProjectFiles.ts` 负责 Project 文件加载、文件选择、刷新、浏览器预览 mock 和真实 Tauri 失败空态。
- `ProjectFiles.css` 负责 Project 文件阅读器和右侧文件树样式；全局 `styles.css` 只保留 app shell / shared layout 规则。
- `apps/desktop/src-tauri/src/project_files.rs` 负责真实 Tauri 客户端的本地文件读取、metadata、preview、路径逃逸拒绝和系统文件夹选择；`main.rs` 只注册 command。
- Project reader 自己维护文件树展开、目录概览和阅读器渲染。
- 旧的 Project Goal / Milestone / Architecture / Environment / Agent tab 模板不属于 Project 文件阅读器，不能继续混在 Project 页面实现里。
- 浏览器 / Vite preview 使用显式 mock 项目和 mock 文件树支撑 UI 测试；mock 只允许在无 Tauri runtime 时启用。
- 真实 Tauri 客户端不允许 mock fallback；真实读取失败时展示错误或空状态。
- 后续 Milestone / Issue / View 成熟后，按同样方式拆成独立 feature module。

### Project Layout

- Content layout: `minmax(0, 1fr) 336px`
- Column gap: `18px`
- Main reader frame margin: `10px 0 10px 10px`
- Right file browser margin: `10px 10px 10px 0`
- Panels radius: `8px`
- Panels border: `#343842`
- Panels background: `#202126`
- Agent 作业状态统一进入底部 Agent Status Channel，不放在 Project content column。

## File Reader

The reader is the core Project page component. It must feel like a polished local document/code reader.

### Reader Header

- Left: file type icon, type label, file name.
- Right: metadata.
- Metadata rows:
  - `文件大小`: display as `1 KB`, `1.2 MB`, etc.
  - `创建日期`: display file creation time.
- Reader header metadata stays on the right; it must not compress the file name into unreadable fragments.
- File name truncation is allowed only after preserving a useful prefix.
- File icon tile: `36px x 36px`, radius `8px`, background `#282a30`, border `#343842`.

### Markdown Documents

Markdown uses a document reading style.

- Render with `react-markdown` and `remark-gfm`.
- No surrounding content box.
- Use a single top divider: `border-top: 1px solid #2b2f38`.
- Body starts with `22px` top padding after the divider.
- Text max readable width: about `860px`.
- Headings use clear hierarchy.
- Paragraphs use `14px / 24px`.
- Fenced code blocks inside markdown use the code-block style.

### Code / Config Files

Code and config files use a lightweight code frame.

- Render with `Shiki` using the `github-dark-default` theme when the language is recognized.
- Fall back to plain `<pre><code>` if syntax highlighting fails.
- Background: `#17181c`
- Border: `#2b2f38`
- Radius: `8px`
- Padding: `14px 16px`
- Font: monospace `13px / 22px`
- Frame height follows content; do not stretch into a large empty box.

### Directory Overview

Directory overview is simple and readable.

- Show selected folder name.
- Show relative path.
- Show immediate children.
- Do not introduce dashboards, cards, summaries, or action controls.

### Unsupported / Empty / Error

- Browser/Vite preview may use explicit mock local files for UI verification.
- Real Tauri client must not fall back to mock data.
- In real Tauri, failed file loading shows an explicit empty/error state.
- Large text files show a truncated preview with an explicit note and a virtualized line window.
- Binary files show metadata and a hex preview when bytes are available.
- Every visible file must render something: specialized reader, text fallback, binary fallback, or explicit metadata fallback.

### File Renderer Registry

Project file content is routed through `FileRendererRegistry.tsx`.

```text
FileBrowser click
-> Tauri metadata + content / preview
-> FileRendererRegistry
-> specialized reader or fallback reader
```

Supported reader classes:

- `MarkdownReader`: `.md`, `.mdx`, README-style docs.
- `CodeReader`: Rust, TypeScript, JavaScript, CSS, TOML, YAML, shell, config files.
- `JsonReader`: JSON pretty print + code highlighting.
- `PlainTextReader`: ordinary text.
- `LargeTextReader`: large/truncated text preview with a virtualized line window; it renders only visible lines plus overscan, with line numbers and explicit preview status.
- `TableReader`: CSV/TSV lightweight table preview; XLSX uses SheetJS to preview the first worksheet.
- `PdfReader`: PDF uses PDF.js to preview the first page when local preview data is available; otherwise metadata fallback.
- `ImageReader`: native image preview when data URL is available.
- `MediaReader`: native audio/video preview when data URL is available.
- `DocxReader`: DOCX uses mammoth.js to render a read-only HTML preview when local preview data is available; otherwise metadata fallback.
- `BinaryFallbackReader`: metadata + hex preview.
- `UnsupportedFallbackReader`: explicit non-blank fallback.

### Reader Capability Matrix

当前 Project file reader 已接入以下格式。后续新增格式时，必须同时更新 `FileRendererRegistry.tsx`、Tauri `project_files.rs` 的 MIME / language mapping、相关验证记录和本节清单。

| Category | Extensions / files | Reader behavior |
| --- | --- | --- |
| Markdown documents | `.md`, `.markdown`, `.mdx`, README-style docs | `react-markdown` + `remark-gfm`; no inner content box; document-style reading layout. |
| DOCX documents | `.docx` | `mammoth.js` converts local preview data into read-only HTML; missing or oversized preview uses metadata fallback. |
| PDF documents | `.pdf` | `PDF.js` renders page 1 when local preview data is available; missing or oversized preview uses metadata fallback. |
| JSON | `.json`, `.jsonc`, `package.json`, `tsconfig.json` | Pretty print JSON, then render through syntax-highlighted code reader when possible. |
| TOML | `.toml`, `Cargo.toml`, `Tauri.toml` | Syntax-highlighted code reader. |
| YAML | `.yaml`, `.yml` | Syntax-highlighted code reader. |
| Rust | `.rs` | Syntax-highlighted code reader. |
| TypeScript | `.ts`, `.tsx` | Syntax-highlighted code reader. |
| JavaScript | `.js`, `.jsx` | Syntax-highlighted code reader. |
| Web source | `.css`, `.html` | Syntax-highlighted code reader. |
| Shell | `.sh`, `.bash`, `.zsh` | Syntax-highlighted code reader. |
| Config text | `.gitignore`, `.env`, `.env.example` | Config/code reader with fallback to plain text. |
| Plain UTF-8 text | `.txt` and other readable UTF-8 files | Plain text reader; large/truncated files route to large-text reader. |
| CSV / TSV | `.csv`, `.tsv` | Lightweight table preview from delimited text. |
| Excel | `.xlsx` | SheetJS previews the first worksheet, capped to the first 80 rows. |
| Images | `.png`, `.jpg`, `.jpeg`, `.gif`, `.webp`, `.svg` | Native image preview from local data URL when available. |
| Audio | `.mp3`, `.wav`, `.ogg` | Native audio controls from local data URL when available. |
| Video | `.mp4`, `.webm` | Native video controls from local data URL when available. |
| Directories | Any folder | Directory overview with immediate children. |
| Large text | Text over preview threshold or explicitly truncated text | Virtualized line window; renders visible lines plus overscan only. |
| Unknown binary | Any binary file without a specialized reader | Metadata + hex preview; never blank. |
| Unsupported / missing preview | Any recognized file without enough preview data | Explicit metadata / fallback state; never executes commands and never writes files. |

Non-goals for the Project page:

- Do not become an IDE.
- Do not execute files.
- Do not edit files.
- Do not build Office-grade editing or conversion workflows in the reader shell.

## File Browser

The right file browser behaves like a developer file tree.

- Shows hidden files and normal files.
- Shows source directories and build directories.
- Does not hide `.git`, `.agentflow`, `target`, or dotfiles.
- Header starts directly with table labels: `名称` / `修改日期`.
- No separate `项目文件` title block.
- Row height: `32px`.
- Nested row height: `29px`.
- Directory rows show a chevron.
- Expanded directory rotates chevron down.
- Children render inline under the parent with subtle vertical guide lines.
- Selecting a directory loads directory overview in the reader.
- Selecting a file loads file content in the reader.
- Browser/Vite preview without Tauri file access shows explicit mock project files for UI verification.

## Read-only Note

Right file browser footer copy is fixed:

```text
只读展示
不执行/不写入，点击右侧任意文件或文件夹后，主体区域加载对应内容或目录概览。
```

Do not split this into multiple explanatory blocks.

## Milestone Page

Milestone is a stage container, not a template editor.

Use Project page shell, Sidebar, Topbar, background and panel rules.

Milestone page should present a compact Gantt-lite / timeline:

- Left: milestone name and short purpose.
- Middle: timeline / dependency / order.
- Right: stage state, evidence state, issue counts.
- No large card stacks.
- No execution contract content.
- Milestone closeout should be automated; do not add a mandatory human-closeout UI.

## Issue Page

Issue is the only executable unit.

Issue page may show:

- Issue list.
- Selected issue contract.
- Goal.
- Scope.
- Non-goals.
- Dependencies.
- Acceptance criteria.
- Validation commands.
- Evidence requirements.
- Boundary / stop conditions.

Issue page must not:

- Execute commands directly from Desktop.
- Hide validation failures.
- Auto-promote next issue.
- Mix unrelated project/milestone fields into the issue contract.

## View Page

View is for saved project filters and local read-only perspectives.

Use the same shell and panel style:

- Left or top: saved view list.
- Main: selected view rules and matching items.
- No model invocation.
- No remote sync.
- No hidden write behavior.

## Interaction States

- Hover: `#292a2f`
- Active row: `#1f3653`
- Active project: `#2f4b7a`
- Active project left rail: `#8db3ff`
- Button hover: `#2c2d33`, border `#4b5870`
- Focus rings must be visible but not oversized.
- Do not use animation that changes layout height after hover.

## Spacing And Radius

- Global radius: `8px` or less.
- Project/file rows: `6px`.
- Shell gaps: `18px`.
- Main content padding: `20px 24px 24px`.
- Reader padding: `22px 24px 28px`.
- File table body padding: `6px`.

Avoid nested cards. Use full panels for major areas and inline dividers for internal separation.

## Scrollbars

- Independent scroll areas:
  - Sidebar
  - Workspace content
  - Reader
  - File browser tree
- Scrollbar width: `10px`
- Thumb color: `#344258`
- Thumb radius: `999px`

## App-wide Do / Do Not

Do:

- Keep pages dense and work-focused.
- Reuse Sidebar / Topbar / panel tokens.
- Prefer file-reader clarity over decorative UI.
- Keep all Desktop surfaces read-only unless execution is explicitly delegated outside Desktop.
- Keep hidden files visible in developer file views.
- Keep Project / Milestone / Issue / View responsibilities separate.

Do not:

- Add marketing hero sections.
- Add decorative gradients, orbs, or bokeh backgrounds.
- Add large rounded cards.
- Add Project status/progress widgets to the Project file reader page.
- Put Milestone or Issue contract content inside Project reader.
- Add execution buttons, terminal buttons, model buttons, or remote PR actions to Desktop.
- Use mock data in real Tauri client as a silent fallback.
- Reuse browser preview mock data inside a real Tauri window.

## Implementation Handoff

When style changes are made:

1. Update this file.
2. Keep `apps/desktop/src/styles.css` tokens aligned.
3. Verify the Project page in browser preview.
4. Run:

```bash
npm --prefix apps/desktop run build
cargo test
git diff --check
```

For docs-only updates, `git diff --check` is the minimum validation.
