# Saved Query Writer v0 Boundary

创建日期：2026-05-22
执行者：Codex

## 定位

Saved Query Writer v0 是 Saved Query v0 边界和 Desktop Search Read-only View v0 之后的写入能力边界。当前阶段只定义 Writer 的产品、架构、授权、验证和 evidence 要求，不实现 writer，不创建 `.agentflow/queries`，不写 query 文件。

目标是把后续“保存搜索查询”从只读搜索中拆出来，让它成为一个明确受控的 `.agentflow/` 事实源写入能力，而不是 Desktop Search 或 Local Search Reader 的附带行为。

## 允许定义的能力

| 能力 | v0 边界 |
| --- | --- |
| Query definition schema | 定义 `SavedQueryDefinition` 的字段和约束 |
| 文件路径合同 | 定义 `.agentflow/queries/{query-id}.json` 的路径规则 |
| 用户确认门 | 定义创建目录、创建文件、覆盖和删除前必须确认 |
| Writer 命令边界 | 后续可新增 writer CLI，但本阶段不实现 |
| 验证矩阵 | 定义 schema、路径、round-trip、no-result-persistence 验证 |
| Evidence 要求 | 定义 writer 实现必须留下的证据 |

## 当前阶段不允许

| 禁止项 | 说明 |
| --- | --- |
| 创建 `.agentflow/queries` | 目录创建属于 Writer 实现阶段 |
| 写 saved query JSON | 本阶段不生成样例或真实 query 文件 |
| 实现 CLI writer | `agentflow query save` / `agentflow query write` 均后置 |
| 实现 Desktop writer UI | Desktop 仍只能只读搜索和展示 |
| 保存搜索结果 | Saved query 只能保存 query definition，不能保存 result snapshot |
| 创建搜索索引或 cache | FTS、DuckDB、Tantivy、`.agentflow/search` 均未授权 |
| 调用模型或上传远程 | v0 必须本地、确定性、离线 |
| 绕过 IssueContract | 任何写 `.agentflow/` 都必须由独立 issue 授权 |

## 文件路径合同

后续 Writer 实现唯一允许的默认写入路径：

```text
.agentflow/queries/{query-id}.json
```

路径约束：

| 规则 | 要求 |
| --- | --- |
| 根目录 | 必须是当前项目根目录下的 `.agentflow/queries/` |
| 文件名 | `query-id` 只能使用 `a-z`、`0-9`、`-` |
| 后缀 | 必须是 `.json` |
| 禁止路径 | 禁止绝对路径、`..`、空文件名、隐藏文件名、嵌套目录 |
| 事实源属性 | query 文件是用户确认后写入的事实，不是搜索缓存 |
| 可删除性 | 删除 query 不应影响 issue、run、evidence、review 或搜索 reader |

当前阶段必须保持：

```bash
test ! -d .agentflow/queries
```

## SavedQueryDefinition 合同

后续 Writer 只能写入 query definition，不写 result snapshot：

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
    "fields": ["path", "line", "snippet", "entityKind", "entityId", "score"],
    "maxResults": 50,
    "sort": "score-desc-then-path"
  },
  "resultPersistence": "none",
  "createdAt": "2026-05-22T00:00:00Z",
  "updatedAt": "2026-05-22T00:00:00Z"
}
```

字段约束：

| 字段 | 约束 |
| --- | --- |
| `version` | v0 固定为 `0.0.1` |
| `id` | 与文件名一致，使用 slug |
| `name` | 非空，供 Desktop 和 CLI 展示 |
| `query.text` | 非空 literal string |
| `query.mode` | v0 只能是 `literal` |
| `query.caseSensitive` | 默认 `false` |
| `scope.paths` | 必须是 Local Search Reader 授权路径的子集 |
| `display.fields` | 只能引用 `LocalSearchResult` 已存在字段 |
| `display.maxResults` | 必须有上限 |
| `resultPersistence` | v0 只能是 `none` |

## 用户确认门

后续 Writer 实现必须在以下动作前设置确认点：

| 动作 | 确认内容 |
| --- | --- |
| 首次创建 `.agentflow/queries` | 目录路径、事实源属性、不会保存结果 |
| 创建新 query | query id、name、query text、scope、display、写入路径 |
| 覆盖已有 query | 旧文件路径、新内容摘要、覆盖原因 |
| 删除 query | 文件路径、删除后不可由 writer 自动恢复 |
| 修改 query scope | 新增/移除的 searchable paths 和 entity kinds |

确认门要求：

- CLI 必须在写入前展示目标相对路径。
- CLI 必须在写入前展示 `resultPersistence: none`。
- Desktop v0 不允许直接执行 Writer 写入。
- Writer 不能因为用户在 Search 视图输入 query 就自动保存。
- Writer 必须尊重 WIP=1；存在 active issue 时不能绕过当前 issue 合同。

## 架构边界

后续实现只能采用以下链路：

```text
IssueContract
-> CLI writer command
-> validate SavedQueryDefinition
-> confirm user intent
-> write .agentflow/queries/{query-id}.json
-> read-back round-trip
-> evidence / review
```

Reader 和 Desktop 的边界：

- Local Search Reader 可以读取 saved query definition 的后续 runner 另建 issue。
- Desktop Search Read-only View 只能展示即时结果，不保存 query。
- Desktop 若后续展示 saved query 列表，默认只读；写入必须另建 Desktop Interaction Gate。
- `.agentflow/search` 仍然禁止，除非未来搜索 index/cache issue 明确授权。

## 验证矩阵

后续 Writer 实现必须至少验证：

| 验证 | 要求 |
| --- | --- |
| Schema parse | 合法 JSON 通过，非法 mode、空 query、非法 maxResults 失败 |
| Path boundary | 拒绝绝对路径、`..`、嵌套目录、隐藏文件和非 `.json` 后缀 |
| Confirmation gate | 写入前有确认点，未确认时不创建目录或文件 |
| Round-trip | 写入后读取结果与输入 definition 一致 |
| No result persistence | 不写搜索结果、不创建 `.agentflow/search`、不创建 cache |
| Reader parity | 后续 runner 与 `agentflow search "<query>"` 结果一致 |
| WIP=1 | active issue 存在时不能启动无关 writer 写入 |

## Evidence 要求

后续 Writer 实现 evidence 至少包含：

- saved query schema 样例。
- 目标路径确认输出。
- 用户确认点说明。
- 写入 / 读取 round-trip 输出。
- 非法路径和非法 schema 失败输出。
- `test ! -d .agentflow/search` 证明。
- no-result-persistence 证明。

本阶段 evidence 只证明边界定义完成，并继续证明 `.agentflow/queries` 不存在。

## 后续候选小切片

| 顺序 | 小切片 | 授权边界 |
| --- | --- | --- |
| 1 | Saved Query Writer v0 实现 | 在明确 IssueContract 和用户确认点下创建 `.agentflow/queries/*.json` |
| 2 | Saved Query Reader v0 边界定义 | 只读读取 saved query definitions，不运行搜索 |
| 3 | Saved Query Runner v0 边界定义 | 使用 saved query 派生搜索结果，但不保存结果 |

## 验收

1. Writer 边界文档存在，并被 README、ROADMAP、MVP Spec、Local Pro、Local Search、Saved Query、Desktop Search 文档引用。
2. `.agentflow/queries` 和 `.agentflow/search` 不存在。
3. Writer schema、路径、确认门、验证矩阵和 evidence 要求明确。
4. 当前阶段不实现 writer、不新增 CLI 命令、不新增 Desktop 写入 UI。
5. `agentflow goal next` 在本 issue 完成后推荐 `Saved Query Writer v0 实现`。
