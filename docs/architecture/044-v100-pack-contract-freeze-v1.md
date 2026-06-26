# AgentFlow v1.0 Pack Contract Freeze V1

日期：2026-06-25
执行者：Codex

```yaml
packContractVersion: agentflow-pack-contract-freeze.v1
packContractStatus: active
stableContractBaseline: agentflow-stable-contract-baseline.v1
filesystemContractVersion: agentflow-filesystem-contract-freeze.v1
authority: docs/architecture/044-v100-pack-contract-freeze-v1.md
```

## Purpose

本文档冻结 AgentFlow `v1.0.0` 的 Pack 合同。

Pack 的职责是描述行业现场、领域对象、页面入口、外部能力和命令映射。

Pack 不是 Runtime authority。

```text
Pack 定义能力。
Runtime 裁决动作。
Event Store 记录事实。
Projection 提供读模型。
```

本合同引用：

- [041-v100-stable-contract-baseline-v1.md](041-v100-stable-contract-baseline-v1.md)
- [043-v100-agentflow-filesystem-contract-freeze-v1.md](043-v100-agentflow-filesystem-contract-freeze-v1.md)
- [019-pack-filesystem-contract-v1.md](../project/history/2026-06-current-baseline-history/architecture/019-pack-filesystem-contract-v1.md)
- [020-domain-pack-contract-v1.md](../project/history/2026-06-current-baseline-history/architecture/020-domain-pack-contract-v1.md)
- [021-surface-pack-contract-v1.md](../project/history/2026-06-current-baseline-history/architecture/021-surface-pack-contract-v1.md)
- [022-connector-pack-contract-v1.md](../project/history/2026-06-current-baseline-history/architecture/022-connector-pack-contract-v1.md)
- [023-pack-validation-migration-preview-v1.md](../project/history/2026-06-current-baseline-history/architecture/023-pack-validation-migration-preview-v1.md)
- [029-pack-release-gate-readiness-v1.md](../project/history/2026-06-current-baseline-history/architecture/029-pack-release-gate-readiness-v1.md)

## Stable Pack Surfaces

v1 稳定 Pack surface 包括：

| Surface | Path / Artifact | Stable Rule |
| --- | --- | --- |
| Pack root | `.agentflow/packs/<pack-id>/` | Pack 必须以 stable `pack-id` 为目录名 |
| Manifest | `.agentflow/packs/<pack-id>/pack.json` | 必须声明 pack id、pack version、kind、runtime compatibility、dependencies、capability requirements |
| Domain Pack | `.agentflow/packs/<pack-id>/domain/**` | 只能描述 object、link、state、action、acceptance、evidence、migration semantics |
| Surface Pack | `.agentflow/packs/<pack-id>/surface/**` | 只能描述 page、workbench、view model、command entry、read model dependency、sidecar surface |
| Connector Pack | `.agentflow/packs/<pack-id>/connectors/**` | 只能描述 provider、supported action、required capability、health source、smoke policy、disabled reason |
| Validation artifact | `pack-validation-report.json` | release gate artifact，不能写 authority |
| Projection readiness | `pack-projection-readiness.json` | release gate artifact，不能写 authority |
| API plane manifest | `pack-api-plane-manifest.json` | release gate artifact，必须映射 Runtime API / Command Surface |
| Negative fixtures | `pack-negative-fixtures.json` | 必须覆盖 invalid / disabled / fallback / invalid command 场景 |
| Migration preview | `pack-migration-preview.json` | 只读 preview，不写 authority |
| Migration receipts | `pack-migration-*.json` | applied / cancel / rollback 都是 receipt，不伪装成 authority mutation |

## Manifest Contract

Pack manifest 是 Pack 的入口清单。

稳定字段：

- `packId`
- `packVersion`
- `packType`
- `domainPath`
- `surfacePath`
- `connectorPath`
- `runtimeCompatibility`
- `capabilityRequirements`
- `dependencies`
- `migrationPolicy`

规则：

- `packId` 不能为空；
- `packVersion` 必须可机器读取；
- path 必须是相对路径；
- `runtimeCompatibility` 必须覆盖当前 Runtime 版本；
- `migrationPolicy` 默认必须是 preview-first，不允许静默 authority mutation。

## Domain Contract

Domain Pack 稳定表达：

- object types；
- link types；
- state machines；
- action semantics；
- acceptance semantics；
- evidence policy；
- audit trigger hints；
- migration compatibility。

Domain Pack 禁止：

- 写 `.agentflow/spec/**`；
- 写 `.agentflow/events/**`；
- 写 `.agentflow/tasks/**`；
- 写 `.agentflow/audit/**`；
- 把 audit sidecar 放回主业务链。

Action semantics 必须引用：

- `contractRef`
- `arbitrationRef`
- `simulationRef`

## Surface Contract

Surface Pack 稳定表达：

- page registry；
- workbench registry；
- view model mapping；
- command entry mapping；
- read model dependencies；
- navigation rules；
- empty / loading / error state；
- sidecar surfaces。

Surface Pack 只能：

```text
Read Projection
Send Runtime Command / Action Proposal
```

Surface Pack 禁止：

- 直接写 `.agentflow/spec/**`；
- 直接写 `.agentflow/events/**`；
- 直接写 `.agentflow/projections/**`；
- 直接写 `.agentflow/tasks/**`；
- 直接把 UI 操作升级成 authority。

## Connector Contract

Connector Pack 稳定表达：

- connector id；
- provider type；
- supported actions；
- required capability；
- health source；
- smoke policy；
- evidence output；
- disabled reason；
- command boundary。

Connector 输出只能作为：

```text
context
evidence
external-fact
```

外部写动作必须转成 Runtime Command / Action Proposal。

Connector Pack 禁止：

- 直接调用 provider 写 AgentFlow authority；
- 直接关闭任务；
- 直接完成 release；
- 把外部 session 当成项目 truth。

## Capability Status Rule

Pack command 能不能用，只能由 Capability Registry / Provider Smoke / Connector Boundary 共同决定。

状态规则：

- capability available -> command 可发起 proposal；
- capability disabled -> command 不可用，必须提供 disabled reason；
- provider smoke failed -> 相关 provider command 不可用；
- unknown connector -> command 不可用；
- disabled capability 不能被 command resolver 当成 available。

## Runtime Entry Rule

Pack 进入 Runtime 的唯一合法路径：

```text
Pack definition
-> Pack Registry
-> Runtime API / Command Surface
-> Action Proposal
-> Arbitration / Governance
-> Event Store
-> Projection
```

非法路径：

```text
Pack file writes .agentflow/spec/**
Pack file writes .agentflow/events/**
Pack file writes .agentflow/tasks/**
Pack connector calls provider directly
Pack UI writes authority directly
Pack loader silently mutates project state
```

## Migration Rule

Pack migration 必须 preview-first。

稳定语义：

- preview 默认 `writesAuthority = false`；
- preview 必须列出 affected objects；
- preview 必须列出 affected projections；
- apply 必须有 explicit human confirmation；
- applied receipt 只能记录 receipt-only apply，不能伪装成 authority mutation；
- rollback receipt 只能记录 receipt-only rollback；
- fake authority mutation 必须被 release gate negative fixture 阻断。

## Built-in Pack Baseline

v1 稳定默认行业壳：

```text
software-dev
```

状态：

```text
software-dev = completed
```

v1 baseline 行业壳：

```text
ui-design
```

状态：

```text
ui-design = baseline
```

`ui-design` 是 baseline，不是 v1 默认稳定主链。

## Compatibility Promise

v1 之后，Pack contract 承诺：

- 不删除 manifest 必填字段；
- 不重命名 stable Pack status；
- 不让 Pack 绕过 Runtime API；
- 不让 connector output 拥有 project truth；
- 不把 Audit sidecar 变成 Software Dev 主链 blocker；
- 不把 Pack migration receipt 伪装成 authority mutation；
- 不恢复 fallback Pack registry。

## Breaking Change Rule

破坏 Pack stable contract 必须：

1. 提供 replacement contract；
2. 提供 migration preview；
3. 提供 explicit confirmation boundary；
4. 提供 rollback receipt；
5. 增加 release gate negative fixture；
6. 不允许在 patch release 中直接破坏 stable Pack。

## Release Gate Fixture

release gate 必须生成：

```text
runtime/pack-contract-compatibility.json
```

该 fixture 至少证明：

- 本文档存在；
- `packContractVersion = agentflow-pack-contract-freeze.v1`；
- `packContractStatus = active`；
- `stableContractBaseline = agentflow-stable-contract-baseline.v1`；
- `filesystemContractVersion = agentflow-filesystem-contract-freeze.v1`；
- project `.agentflow/packs/**` 是 file-backed registry；
- `software-dev` 和 `ui-design` 都来自 project files；
- Pack validation passed；
- Pack simulation passed；
- Pack projection readiness passed；
- Pack API plane manifest passed；
- disabled capability negative fixture passed；
- invalid command submit negative fixture passed；
- unexpected fallback negative fixture passed；
- Pack migration apply / rollback 为 receipt-only，不伪装成 authority mutation。

## V100 Binding

本合同绑定后续 v1 任务：

| Issue | Required usage |
| --- | --- |
| V100-005 | Projection / Read Model 必须读取 Pack surface / domain readiness，但不能让 Pack 拥有 projection authority |
| V100-006 | Evidence / Acceptance 必须保留 Pack evidence policy，但 Done 仍由 Acceptance Gate 决定 |
| V100-007 | Executor Adapter 必须通过 Connector Pack capability boundary 进入 provider |
| V100-008 | Replay / Migration / Upgrade 必须证明 Pack migration receipt-only 和 rollback boundary |
| V100-009 | Software Dev Pack Stable Baseline 必须引用本合同 |
| V100-010 | Release certification 必须包含 Pack contract compatibility fixture |

## Non-goals

- 不做 Pack marketplace；
- 不承诺所有未来行业 Pack 已完成；
- 不把 UI Design Pack 提升为 v1 默认稳定行业壳；
- 不直接实现完整 GitHub / GitLab / Figma provider；
- 不把 Pack 文件变成 Runtime authority；
- 不恢复 fallback registry；
- 不把 Audit sidecar 放回 Software Dev 主链。
