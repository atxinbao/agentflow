# Saved Query v0 Boundary

创建日期：2026-05-22
执行者：Codex

## 定位

Saved Query v0 是 Local Search Reader v0 之后的边界定义阶段。它只定义 saved query / saved search 的产品范围、文件格式候选、授权门、用户确认点、验证方式和 evidence 要求。

当前阶段不实现保存查询，不创建 `.agentflow/queries`，不新增 CLI 命令，不新增 Desktop 搜索 UI，也不保存搜索结果。

## 与现有 SavedView 的关系

AgentFlow 已有 `SavedView`：

- 路径：`.agentflow/views/*.json`
- 用途：保存 issue / run 的过滤视图。
- 读取方式：通过 SQLite 可重建索引展示 issue / run 列表。
- 边界：不保存全文搜索 query，不保存搜索结果。

Saved Query 是后续 Local Search 能力：

- 目标：保存 literal search query 和展示偏好。
- 输入：Local Search Reader v0 已授权的本地文本事实源。
- 输出：运行时派生 `LocalSearchSnapshot`，默认不落盘结果。
- 边界：不能替代 IssueContract，也不能绕过 WIP=1。

因此 v0 不复用 `.agentflow/views/*.json` 来保存全文搜索 query，避免把 issue/run filter 与本地搜索 query 混在同一个事实类型中。

## 当前阶段不允许

| 禁止项 | 说明 |
| --- | --- |
| 创建 `.agentflow/queries/` | 文件格式还未进入实现阶段 |
| 写 saved query JSON | 本阶段只定义 schema 候选，不生成实例文件 |
| 保存搜索结果 | 结果必须由 reader 每次从事实源重新派生 |
| 实现 `agentflow search save` | CLI 需要单独 IssueContract |
| 实现 `agentflow query run` | saved query runner 需要单独 IssueContract |
| 新增 Desktop 搜索 UI | Desktop Search 必须先定义只读视图边界 |
| 引入 FTS / Tantivy / DuckDB / embedding | 索引和语义搜索均后置 |
| 调用模型或上传远程 | Local Search / Saved Query 默认本地、确定性、离线 |
| 绕过 IssueContract 写 `.agentflow/` | 任何事实源写入必须由独立 issue 授权 |

## 后续文件路径候选

后续若实现 saved query，首选路径为：

```text
.agentflow/queries/{query-id}.json
```

路径规则：

| 规则 | 要求 |
| --- | --- |
| 根目录 | 只能在当前项目 `.agentflow/queries/` 下 |
| 文件名 | `query-id` 使用小写 slug，允许 `a-z`、`0-9`、`-` |
| 后缀 | 必须是 `.json` |
| 路径安全 | 禁止 `..`、绝对路径、隐藏文件名和目录穿越 |
| 事实源属性 | query 文件是用户授权写入的事实，不是搜索结果缓存 |

本阶段不创建该目录，也不创建任何样例 query 文件。

## Schema 候选

后续 `SavedQueryDefinition` 最小候选字段如下：

```json
{
  "version": "0.0.1",
  "id": "local-search-boundary",
  "name": "Local Search Boundary",
  "description": "Find Local Search boundary references.",
  "query": {
    "text": "Local Search",
    "mode": "literal",
    "caseSensitive": false
  },
  "scope": {
    "sourceTypes": ["file"],
    "entityKinds": ["issue", "run", "evidence", "review", "project-update"],
    "paths": [".agentflow/issues/*.md", ".agentflow/evidence/*.md"],
    "includeDerivedSnapshots": false
  },
  "display": {
    "fields": ["path", "line", "snippet", "entityKind", "entityId"],
    "maxResults": 50,
    "sort": "score-desc-then-path"
  },
  "resultPersistence": "none",
  "createdAt": "2026-05-22T00:00:00Z",
  "updatedAt": "2026-05-22T00:00:00Z"
}
```

Schema 约束：

- `query.mode` v0 只能是 `literal`。
- `query.text` 必须是非空字符串。
- `caseSensitive` 默认 `false`，必须与 Local Search Reader 的匹配策略一致。
- `scope.paths` 只能落在 Local Search Reader 授权路径内。
- `display.fields` 只能引用 `LocalSearchResult` 已存在字段。
- `display.maxResults` 必须有上限，避免无界输出。
- `resultPersistence` v0 只能是 `none`。

## 用户确认门

后续任何 saved query 写入都必须满足：

| 门 | 要求 |
| --- | --- |
| IssueContract | 明确授权写 `.agentflow/queries/*.json` |
| 用户确认 | 首次创建目录、创建新 query、覆盖 query、删除 query 都需要确认点 |
| 路径确认 | CLI 输出将写入的相对路径 |
| 内容确认 | CLI 输出 query text、scope、display、resultPersistence |
| 只读运行 | `query run/show` 默认只读，不修改事实源 |
| WIP=1 | 不允许 saved query 写入绕过 active issue 限制 |

## 验证方式

后续实现必须至少验证：

| 验证 | 要求 |
| --- | --- |
| JSON schema parse | 合法 query 能解析，非法 mode / 空 query / 非法路径失败 |
| Round-trip | 写入后读取的 schema 与输入一致 |
| Path boundary | 不能写 `.agentflow/queries/../*`、绝对路径或未授权目录 |
| No result persistence | 运行 saved query 不写结果文件、不创建 cache |
| Reader parity | saved query run 与直接 `agentflow search "<query>"` 的结果一致 |
| Desktop boundary | Desktop 若展示 saved query，只能读，不执行写入 |

## Evidence 要求

后续实现 saved query 时，evidence 至少包含：

- saved query schema 样例。
- 创建 / 读取 / 运行的命令输出。
- 路径边界测试输出。
- 不保存结果、不创建 cache 的证明。
- 与 `agentflow search` 直接结果的 parity 证明。
- 用户确认点说明。

## 后续候选小切片

| 顺序 | 小切片 | 授权边界 |
| --- | --- | --- |
| 1 | Desktop Search Read-only View v0 边界定义 | 只定义 Desktop 搜索入口和只读展示边界 |
| 2 | Desktop Search Read-only View v0 实现 | 只调用 Local Search Reader，不写 query、不执行命令 |
| 3 | Saved Query Writer v0 边界定义 | 定义 `.agentflow/queries/*.json` 写入和用户确认 |
| 4 | Saved Query Writer v0 实现 | 在明确授权后创建 query 文件，不保存搜索结果 |
| 5 | Saved Query Reader v0 边界定义 | 只读读取 saved query definitions，不运行搜索 |

## 验收

1. 本文档定义 saved query 与 SavedView 的差异。
2. `.agentflow/queries/*.json` 文件格式候选明确，但本阶段不创建目录或文件。
3. 用户确认门、路径边界、结果不落盘规则明确。
4. 后续实现必须保留 IssueContract 和 WIP=1。
5. Saved Query Writer boundary 完成后，`agentflow goal next` 推荐 `Saved Query Writer v0 实现`。
