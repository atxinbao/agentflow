# 012 - Schema Version Migration Registry V1

创建日期：2026-06-23
执行者：Codex

## Purpose

本文定义 AgentFlow 的最小 schema version 和 migration registry 边界。

它回答四个问题：

1. 系统当前有哪些事实 schema version；
2. 如何判断一个事实文件是 current / legacy / missing version / unknown；
3. migration preview 应该输出什么；
4. migration apply 为什么必须显式确认。

## Module Boundary

实现模块：

```text
crates/schema-registry
```

负责：

- 列出当前核心事实 schema version；
- 检测 observed version 是否过期；
- 生成 migration preview；
- 定义 explicit apply confirmation；
- 证明 preview 默认不写 authority。

不负责：

- 扫描整个工作区；
- 自动修改 `.agentflow/**` authority；
- 做数据库 migration framework；
- 迁移 Pack schema；
- 绕过 Runtime API / Action Arbitration。

## Registry Entries

Registry 第一版覆盖核心事实层：

| Layer | Example schema |
| --- | --- |
| Spec | `spec.issue`, `spec.project`, `spec.requirement-preview` |
| Event Store | `event.task`, `event.runtime-envelope` |
| Task Artifact | `task.run`, `task.evidence` |
| Projection | `projection.task`, `projection.project` |
| Audit | `audit.request`, `audit.findings` |
| Release | `release.delivery-summary` |
| Runtime API | `runtime.command-api` |
| State | `state.manifest` |
| Workflow | `workflow.definition` |

## Detection Status

检测结果固定为四类：

```text
current
legacy
missing-version
unknown-schema
```

含义：

- `current`：observed version 与 registry current version 一致；
- `legacy`：schema 已注册，但 observed version 与 current version 不一致；
- `missing-version`：schema 已注册，但事实缺少 version；
- `unknown-schema`：schema id 不在 registry 中。

## Migration Preview

Migration preview 是只读计划。

它必须包含：

- `previewId`
- `mode = preview`
- `writesAuthority = false`
- detections
- proposed actions

Preview 不能修改 authority。

Preview 还必须生成 preview receipt：

```text
receiptKind = preview
writesAuthority = false
proposedActionCount
legacyCount
missingVersionCount
unknownSchemaCount
```

preview receipt 只证明“系统看到了什么、建议什么”，不代表已经迁移。

## Explicit Apply Boundary

Migration apply 必须传入：

- `previewId`
- `confirmed = true`
- `actor`
- `reason`

如果没有显式确认，apply 必须失败。

第一版 apply 只返回 applied receipt，不写 `.agentflow/**` authority。真正写 authority 的迁移必须由后续 issue 单独授权。

Applied receipt 必须和 preview receipt 区分：

```text
receiptKind = applied
applied = true
authorityWrites = []
deferredActions = preview.proposedActions
```

## Acceptance

本边界成立时，应满足：

- 能列出当前 schema version；
- 能检测 legacy / missing / unknown；
- 能生成 migration preview；
- preview 默认不写 authority；
- apply 没有显式确认会失败；
- preview receipt 和 applied receipt 语义分离；
- applied receipt 不包含自动 authority writes。
