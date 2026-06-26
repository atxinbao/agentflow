# Pack Validation and Migration Preview V1

创建日期：2026-06-23  
执行者：Codex

## 1. 目标

Pack 进入 Runtime 前必须先通过验证。

这份文档定义：

- Pack validation artifact；
- Pack version compatibility；
- Pack API Plane mapping；
- Pack migration preview；
- preview receipt；
- applied receipt boundary。

核心规则：

```text
Pack 可以被读取、验证、预览迁移。
Pack validation / migration preview 默认不写 Runtime authority。
invalid Pack 不能被 Runtime 标记为 active。
applied migration 必须有明确 human confirmation 和 applied receipt。
```

## 2. Pack Validation 输入

Pack validation 读取四类定义：

```text
.agentflow/packs/<pack-id>/pack.json
.agentflow/packs/<pack-id>/domain/**
.agentflow/packs/<pack-id>/surface/**
.agentflow/packs/<pack-id>/connectors/**
```

同时读取 Runtime API Plane manifest 中的可用入口：

```text
runtime commands
projection queries
connector actions
provider actions
audit actions
release actions
pack actions
```

## 3. Validation Artifact

Validation 输出结构：

```text
PackValidationArtifact
  version
  packId
  active
  writesAuthority
  manifest report
  domain report
  surface report
  connector report
  versionCompatibility
  apiPlaneMapping
  missingReadModels
  missingCommandMappings
  issues
```

路径约定：

```text
.agentflow/packs/<pack-id>/validation/validation.json
```

注意：

- `writesAuthority` 必须是 `false`。
- `active` 只表示这个 Pack 可以被 Runtime 读取使用。
- `active` 不代表 Pack 写入了 spec / event / task authority。

## 4. Validation 检查项

Validation 至少检查：

| 检查项 | 规则 |
| --- | --- |
| manifest schema | version、packId、路径、runtimeCompatibility 必须有效 |
| domain references | domain packId 必须和 manifest packId 一致 |
| surface mappings | surface packId 必须和 manifest packId 一致 |
| connector capability requirements | connector 必须声明 capability，外部写动作必须走 Runtime Command Surface |
| API Plane mapping | surface / connector 声明的 command 必须在 API Plane 中存在 |
| version compatibility | Pack runtimeCompatibility 必须覆盖当前 runtime version |
| missing read models | surface 依赖的 projection 必须在 manifest projection entries 中声明 |
| missing command mappings | surface / connector 命令必须能映射到 API Plane entry |

任何检查失败：

```text
active = false
issues[] 记录失败原因
```

## 5. Migration Preview

Migration preview 是只读计划，不是迁移执行。

输出结构：

```text
PackMigrationPreview
  version
  previewId
  packId
  fromVersion
  toVersion
  writesAuthority = false
  affectedObjects
  affectedProjections
  requiredHumanConfirmation = true
  previewReceipt
  appliedReceiptBoundary
```

Preview 规则：

- 默认不写 authority；
- 必须列出 affected objects；
- 必须列出 affected projections；
- 必须生成 preview receipt；
- 必须声明 applied receipt boundary。

## 6. Applied Receipt Boundary

Applied migration 不是 preview 的自然结果。

它必须满足：

```text
confirmation.previewId == preview.previewId
confirmation.confirmed == true
```

只有满足这两个条件，才能生成：

```text
PackMigrationAppliedReceipt
  applied = true
  writesAuthority = true
```

如果没有明确确认：

```text
applied = false
writesAuthority = false
```

## 7. Runtime API 边界

Runtime API 只暴露读取入口：

```text
get_pack_registry
get_pack_validation_artifact
```

它不负责：

- 自动 apply migration；
- 写 `.agentflow/spec/**`；
- 写 `.agentflow/events/**`；
- 写 `.agentflow/tasks/**`；
- 启动 provider。

## 8. Release Gate 读取规则

Release gate 可以读取 Pack validation artifact。

读取后只判断：

```text
active == true
writesAuthority == false
issues is empty
```

如果 Pack validation artifact 不存在或无效，Release gate 可以把 Pack readiness 视为未满足。

## 9. 非目标

本阶段不做：

- 自动 migration apply；
- Cloud migration；
- invalid Pack runtime activation；
- UI Pack 编辑器；
- provider 调用；
- `.agentflow/**` authority 写入。

## 10. 实现位置

```text
crates/pack/src/validation.rs
crates/runtime-api/src/pack.rs
crates/runtime-api/src/api_plane.rs
crates/schema-registry/src/lib.rs
```

