# Local Search v0 Boundary

创建日期：2026-05-22
执行者：Codex

## 定位

Local Search v0 是 Local Metrics Snapshot v0 之后的本地搜索边界定义。边界阶段只定义产品范围、架构边界、授权门和验证方式，不实现搜索引擎、不创建索引、不新增 Desktop 搜索 UI、不写 saved query 文件。

`Local Search Reader v0 只读实现` 已在该边界内完成：它只读扫描授权 `.agentflow/` 文本事实，输出 CLI 搜索结果，不建索引、不写 saved query、不新增 Desktop 搜索 UI。

## 候选能力

| 能力 | v0 边界 | 当前状态 |
| --- | --- | --- |
| `.agentflow/` JSON / Markdown 搜索 | 只读扫描本地事实源文本 | implemented / reader-only |
| Issue / run / evidence / review / update 搜索 | 按实体类型返回命中结果 | implemented / reader-only |
| DesktopWorkbenchSnapshot / LocalMetricsSnapshot 派生文本搜索 | 后续把内存快照转换为只读搜索文档 | deferred |
| saved query / saved search | 必须后置到独立 IssueContract，并先定义文件格式 | deferred |
| Desktop Workbench 搜索入口 | 必须后置到独立 IssueContract，只读展示优先 | deferred |

## 当前阶段不允许

| 禁止项 | 说明 |
| --- | --- |
| 实现搜索引擎 | Reader v0 只做 literal scan，不实现索引型搜索引擎 |
| 引入 Tantivy / SQLite FTS / DuckDB FTS | 搜索引擎和 FTS 选型必须后置 |
| 写 `.agentflow/search` 或 `.agentflow/queries` | saved query 和搜索索引文件格式未授权 |
| 新增 Desktop 搜索 UI | Desktop Workbench 继续保持现有只读页面，不增加搜索入口 |
| 调用模型做语义搜索 | 语义搜索涉及数据、成本和授权边界，必须另建 issue |
| 上传代码或索引到远程 | Local Search 必须本地优先，不创建远程索引 |
| 绕过 IssueContract 写 `.agentflow/` | 所有事实源写入仍必须由独立 issue contract 授权 |

## 可搜索路径

Local Search Reader v0 只允许从以下路径读取，不写入：

| 路径 | 内容类型 | 实体类型 |
| --- | --- | --- |
| `.agentflow/goal.md` | Markdown | goal |
| `.agentflow/goal.json` | JSON | goal |
| `.agentflow/project-definition.json` | JSON | project-definition |
| `.agentflow/scope-state.json` | JSON | scope-state |
| `.agentflow/context.json` | JSON | context |
| `.agentflow/context.md` | Markdown | context |
| `.agentflow/environment.md` | Markdown | environment |
| `.agentflow/architecture.md` | Markdown | architecture |
| `.agentflow/roadmap.md` | Markdown | roadmap |
| `.agentflow/index.json` | JSON | index-summary |
| `.agentflow/settings.json` | JSON | settings |
| `.agentflow/bootstrap/*.md` | Markdown | bootstrap |
| `.agentflow/issues/*.json` | JSON | issue |
| `.agentflow/issues/*.md` | Markdown | issue |
| `.agentflow/runs/*/run.json` | JSON | run |
| `.agentflow/runs/*/transcript.md` | Markdown | run-transcript |
| `.agentflow/runs/*/commands.jsonl` | JSONL | run-command |
| `.agentflow/runs/*/diff-summary.md` | Markdown | run-diff |
| `.agentflow/evidence/*.md` | Markdown | evidence |
| `.agentflow/reviews/*.md` | Markdown | review |
| `.agentflow/updates/*.md` | Markdown | project-update |
| `.agentflow/views/*.json` | JSON | saved-view |
| DesktopWorkbenchSnapshot | in-memory derived text | desktop-snapshot |
| LocalMetricsSnapshot | in-memory derived text | metrics-snapshot |

## 必须排除路径

| 路径 | 原因 |
| --- | --- |
| `.git/` | 不是 AgentFlow 事实源 |
| `target/` | 构建产物 |
| `node_modules/` | 依赖目录 |
| `apps/desktop/dist/` | 前端构建产物 |
| `.agentflow/tmp/` | 临时文件，不作为事实源 |
| `.agentflow/index.sqlite` / `.agentflow/index.sqlite-*` | 可重建 SQLite 索引，不直接搜索 |
| `.agentflow/search/` | 当前阶段禁止创建，后续即使存在也必须单独授权 |
| `.agentflow/queries/` | 当前阶段禁止创建，saved query 文件格式后置 |
| `.env` / `.env.*` | 环境配置不进入搜索结果 |
| 二进制文件 | v0 只处理 JSON、JSONL 和 Markdown 文本 |

## 搜索结果字段

Local Search Reader v0 的结果对象必须保持可追溯，最小字段如下：

| 字段 | 说明 |
| --- | --- |
| `sourceType` | `file` 或 `derived` |
| `entityKind` | issue / run / evidence / review / update / snapshot 等实体类型 |
| `entityId` | 可解析实体 id，例如 `ISSUE-0012`、`RUN-0010`；无法解析时为空 |
| `path` | 文件命中时的相对路径，派生文本命中时为空 |
| `title` | 文件标题、issue title 或派生对象标题 |
| `field` | 命中的 JSON 字段或 Markdown 段落名 |
| `line` | 文件命中的 1-based 行号；派生文本可为空 |
| `snippet` | 命中文本片段 |
| `score` | v0 可以是 deterministic 排序分数，不能依赖模型 |

## Query 表达格式

Local Search Reader v0 只允许 literal text query：

- 输入为一个非空字符串。
- 大小写匹配策略必须固定并可测试。
- 不支持正则表达式。
- 不支持 boolean grammar。
- 不支持向量、embedding 或语义扩展。
- 不支持远程搜索。

任何高级 query grammar 都必须在 saved query 或 search engine issue 中另行定义。

## Saved Query 后置规则

saved query / saved search 不在 Reader 阶段写文件，也不在 Local Search Reader v0 默认写文件。Saved Query v0 的边界已在 `docs/specs/saved-query-boundary.md` 定义；Writer 写入边界已在 `docs/specs/saved-query-writer-boundary.md` 定义，当前仍不创建 `.agentflow/queries`。

后续若实现 saved query writer，必须按 Writer 边界明确：

1. 文件路径是否为 `.agentflow/queries/*.json` 或复用 `.agentflow/views/*.json`。
2. JSON schema。
3. 是否保存 query 字符串、过滤条件、排序方式和展示字段。
4. 是否保存结果；默认不保存结果。
5. 写入前是否需要用户确认。
6. 如何验证 round-trip、schema、路径边界和 no-result-persistence。

## 索引边界

Local Search Reader v0 直接扫描允许路径，不创建 index。

任何索引必须满足：

- 是可删除、可重建的派生缓存。
- 不成为事实源。
- 不改变 issue / run / evidence / review / update。
- 必须有 rebuild parity 验证。
- 必须由独立 IssueContract 授权。

## Desktop 边界

Desktop Search Read-only View v0 已实现，且只能：

- 调用已授权的只读 search reader。
- 展示 query 输入、结果列表和 source trace。
- 显示 recommended command 时仍只展示文本。
- 不执行 run / verify / review。
- 不创建 issue。
- 不写 `.agentflow/search` 或 `.agentflow/queries`。
- 不调用模型。

Desktop 搜索入口的完整边界见 `docs/specs/desktop-search-readonly-boundary.md`。Saved Query Writer 的写入边界见 `docs/specs/saved-query-writer-boundary.md`。

## 后续候选小切片

| 顺序 | 小切片 | 授权边界 |
| --- | --- | --- |
| 1 | Local Search Reader v0 只读实现 | 只读扫描允许路径，输出本地搜索结果，不建索引、不写文件 |
| 2 | Saved Query v0 边界定义 | 只定义 query 文件格式、确认点和验证方式，不创建 query 文件 |
| 3 | Desktop Search Read-only View v0 边界定义 | 只定义 Desktop 搜索入口和只读展示边界 |
| 4 | Desktop Search Read-only View v0 实现 | 只读展示搜索结果，不执行命令、不写事实源 |
| 5 | Saved Query Writer v0 边界定义 | 只定义 `.agentflow/queries/*.json` 写入合同和确认门 |
| 6 | Saved Query Writer v0 实现 | 在确认门下写 query definition，不保存搜索结果 |

## 验收

1. 本文档定义边界，Reader v0 只在该边界内实现 literal scan。
2. 可搜索路径和排除路径明确。
3. saved query 明确后置，不在本阶段写文件。
4. Desktop Workbench 搜索 UI 已保持只读。
5. Saved Query Writer boundary 完成后，`agentflow goal next` 推荐 `Saved Query Writer v0 实现`。
