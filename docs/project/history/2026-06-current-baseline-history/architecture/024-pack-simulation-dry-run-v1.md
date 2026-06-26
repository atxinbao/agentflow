# Pack Simulation Dry-run V1

创建日期：2026-06-23  
执行者：Codex

## 1. 目标

Pack command 在真实执行前必须能 dry-run。

Dry-run 用来回答：

```text
如果这个 Pack command 被执行，会影响什么？
它需要哪些 capability / evidence？
它会触发哪些事件？
它会影响哪些 projection？
它是否缺 surface / connector / read model 映射？
```

## 2. 不变式

Pack simulation 必须保持只读：

```text
writesAuthority = false
writesEventStore = false
executesProvider = false
```

它不负责：

- 写 `.agentflow/spec/**`；
- 写 `.agentflow/events/**`；
- 写 `.agentflow/tasks/**`；
- 重建 projection；
- 启动 Codex / Claude / Figma / Browser provider；
- 把 simulation 结果当成完成事实。

## 3. 输入

Pack command simulation 输入：

```text
PackCommandSimulationRequest
  simulationId
  command
  targetObjectType
  targetObjectId
  actorRole
  PackValidationArtifact
  PackDomainDefinition
  PackSurfaceDefinition
  PackConnectorDefinition
  createdAt
```

其中 `PackValidationArtifact` 必须来自 Pack validation 阶段。

## 4. 输出

输出仍复用统一 `SimulationReport`，kind 为：

```text
pack-command
```

必须包含：

| 字段 | 内容 |
| --- | --- |
| command | 当前 dry-run 的 Pack command |
| target object | command 目标对象 |
| expected events | command 可能产生的事件链 |
| affected projections | 受影响的页面 / read model |
| required capabilities | connector action 需要的 capability |
| required evidence | domain action 需要的 evidence |
| rejected reasons | 缺失或非法项 |
| conflict preview | 预览 scope key，不抢锁 |
| acceptance impact | would-pass / would-reject |

## 5. 缺口识别

Pack simulation 至少识别：

| 缺口 | 触发条件 |
| --- | --- |
| PackValidationFailed | validation artifact 不 active |
| SurfaceMappingMissing | surface 没有 command mapping |
| ConnectorActionMissing | surface 和 connector 都无法识别该 command |
| ReadModelMissing | validation artifact 记录 missing read models |

这些缺口只写入 simulation report，不改 Runtime authority。

## 6. Software Dev / UI Design 覆盖

内置 Pack 必须至少覆盖：

```text
software-dev: work.issue.start
ui-design: design.generate-wireframe
```

这两条 command 都必须 dry-run 成功，并且不得触发 provider。

## 7. 实现位置

```text
crates/simulation/src/lib.rs
```

核心入口：

```text
simulate_pack_command
```

## 8. 非目标

本阶段不做：

- 真实 provider launch；
- 真实 command execution；
- Event Store append；
- Projection rebuild；
- Pack-aware UI 展示；
- Pack migration apply。

