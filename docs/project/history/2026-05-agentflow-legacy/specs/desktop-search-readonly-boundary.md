# Desktop Search Read-only View v0 Boundary

创建日期：2026-05-22
执行者：Codex

## 定位

Desktop Search Read-only View v0 是 Saved Query v0 边界定义之后的 Desktop Workbench 搜索入口。当前阶段已经实现只读 UI，但仍只定义并执行本地只读展示能力，不扩展写入、保存或执行能力。

目标是让后续 Desktop Workbench 可以只读展示 Local Search Reader 的结果，同时继续遵守本地优先、IssueContract、WIP=1 和 `.agentflow/` 事实源边界。

`Desktop Search Read-only View v0 实现` 已在该边界内完成：Desktop Workbench 通过 Tauri `load_search_snapshot` 调用 `read_local_search_snapshot`，只读展示搜索结果，不写 query、不保存结果、不执行命令。

## 允许能力

| 能力 | v0 边界 |
| --- | --- |
| 搜索入口 | 已提供 query 输入框 |
| 查询执行 | 只调用 Local Search Reader 的只读能力 |
| 结果展示 | 已展示 `LocalSearchResult` 字段和 source trace |
| 状态展示 | 已覆盖 empty / loading / error 状态 |
| 只读提示 | 已显示 read-only badge |
| 推荐命令 | 可以展示 recommended command 文本，但不能执行 |

## 当前阶段不允许

| 禁止项 | 说明 |
| --- | --- |
| 扩展 Desktop 搜索写入能力 | 当前只读实现不允许任何写入或执行入口 |
| 写 `.agentflow/search` | 搜索缓存、索引和结果文件未授权 |
| 写 `.agentflow/queries` | saved query writer 仍未授权 |
| 保存搜索结果 | 结果必须由 reader 从事实源重新派生 |
| 执行 run / verify / review | Desktop Search 不能成为执行入口 |
| 创建 issue | issue contract 仍必须由 CLI / goal loop 授权 |
| 调用模型 | 语义搜索和模型解释必须另建 IssueContract |
| 上传远程 | 本地项目事实和搜索结果不能上传 |
| 触发远程 PR / Linear issue | Desktop Search v0 不接远程协作 |

## 架构边界

Desktop Search Read-only View v0 实现只允许采用以下链路：

```text
React Search View
-> Tauri command
-> agentflow-core read_local_search_snapshot(start, query)
-> LocalSearchSnapshot
-> read-only result render
```

架构约束：

- Tauri command 只能读取当前项目根目录。
- Tauri command 不能创建 `.agentflow/search`、`.agentflow/queries` 或其他 cache。
- React 层不能直接写文件。
- React 层不能 shell 执行 `agentflow search`。
- 搜索结果必须来自 `LocalSearchSnapshot`，不能复用 SQLite index。
- Desktop 只展示当前搜索结果，不保存历史结果。

## 后续 UI 契约

当前只读实现必须至少提供以下 UI 元素：

| UI 元素 | 要求 |
| --- | --- |
| Query 输入框 | 输入 literal text query；空 query 显示 empty 状态，不自动写文件 |
| Result list | 展示 path、line、snippet、entityKind、entityId、score |
| Source trace | 每条结果必须可追溯到 `.agentflow/` 相对路径或 derived source |
| Empty 状态 | 无 query 或无结果时展示只读提示，不创建 saved query |
| Loading 状态 | 只表示本地读取中，不暗示远程请求 |
| Error 状态 | 展示本地错误文本，不上传诊断信息 |
| Read-only badge | 明确提示不写事实源、不执行命令 |
| Recommended command | 只展示文本，不提供执行按钮 |

## 输入和输出边界

| 类型 | 边界 |
| --- | --- |
| Query 输入 | 非空 literal string；不支持 regex、boolean grammar、embedding |
| Source scope | 继承 `docs/specs/local-search-boundary.md` 的允许路径和排除路径 |
| Result fields | 继承 `LocalSearchResult` 字段 |
| Max results | 后续实现必须设置明确上限，避免 UI 无界渲染 |
| Sorting | 默认沿用 Local Search Reader 的 deterministic order |
| Persistence | 不保存 query、不保存结果、不写 cache |

## 与 Saved Query 的关系

Desktop Search Read-only View v0 只展示即时搜索结果，不保存 query。

后续若需要保存 query：

- 必须走 `Saved Query Writer v0` 独立 IssueContract。
- 必须遵守 `docs/specs/saved-query-writer-boundary.md` 的 schema、路径和确认门。
- 必须写入 `.agentflow/queries/*.json` 前获得用户确认。
- Desktop 不能绕过 saved query writer 的路径、schema 和确认门。
- 保存 query 后仍默认不保存搜索结果。

## 验证方式

边界定义阶段至少验证：

| 验证 | 要求 |
| --- | --- |
| Docs anchors | README、ROADMAP、MVP Spec、Local Pro、Local Search、Saved Query 都引用 Desktop Search boundary |
| UI read-only implementation | Desktop 搜索 view / tab / command 已实现，但只能读取 Local Search Reader |
| No forbidden directories | `.agentflow/search` 和 `.agentflow/queries` 不存在 |
| Reader sample | `agentflow search "Desktop Search"` 能返回可追溯文档结果 |
| Build safety | 既有 Desktop build 仍通过 |
| Goal Loop | Writer boundary 完成后推荐 `Saved Query Writer v0 实现` |

## Evidence 要求

本阶段 evidence 至少包含：

- boundary 文档路径。
- `agentflow search "Desktop Search"` 输出摘要。
- `.agentflow/search` / `.agentflow/queries` 未创建证明。
- Desktop build 仍通过证明。
- Goal Loop 下一候选证明。
- 明确记录当前已实现 Desktop 搜索 UI，但没有写入、保存、执行或远程能力。

## 后续候选小切片

| 顺序 | 小切片 | 授权边界 |
| --- | --- | --- |
| 1 | Saved Query Writer v0 边界定义 | 定义 `.agentflow/queries/*.json` 写入和用户确认 |
| 2 | Saved Query Writer v0 实现 | 在明确授权后创建 query 文件，不保存搜索结果 |
| 3 | Saved Query Reader v0 边界定义 | 只读读取 saved query definitions |

## 验收

1. Desktop Search 只读 UI 已在本边界内实现。
2. UI 覆盖 query 输入、result list、source trace、empty / loading / error、read-only badge 和 recommended command 展示。
3. 不写 `.agentflow/search` 或 `.agentflow/queries`。
4. 不破坏 Desktop Workbench 只读边界。
5. `agentflow goal next` 在 Writer boundary 完成后推荐 `Saved Query Writer v0 实现`。
