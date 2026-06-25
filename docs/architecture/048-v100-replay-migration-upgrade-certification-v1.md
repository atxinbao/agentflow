# V100 Replay / Migration / Upgrade Certification V1

创建日期：2026-06-25  
执行者：Codex

replayMigrationUpgradeCertificationVersion: agentflow-replay-migration-upgrade-certification.v1  
replayMigrationUpgradeCertificationStatus: active  
stableContractBaseline: agentflow-stable-contract-baseline.v1  
filesystemContractVersion: agentflow-filesystem-contract-freeze.v1  
packContractVersion: agentflow-pack-contract-freeze.v1  
projectionContractVersion: agentflow-projection-readmodel-contract.v1  
evidenceAcceptanceContractVersion: agentflow-evidence-acceptance-contract.v1  
executorAdapterContractVersion: agentflow-executor-adapter-contract.v1  
authority: docs/architecture/048-v100-replay-migration-upgrade-certification-v1.md

## Purpose

本文件冻结 v1.0.0 的 Replay / Migration / Upgrade certification 合同。

核心判断：

```text
Replay proves read models can be rebuilt.
Migration proves receipts are controlled and reversible.
Upgrade proves v0.9.x facts can enter v1.0.0 without reviving retired paths.
Negative fixtures prove wrong paths fail at the correct stage.
```

## References

- [041-v100-stable-contract-baseline-v1.md](041-v100-stable-contract-baseline-v1.md)
- [043-v100-agentflow-filesystem-contract-freeze-v1.md](043-v100-agentflow-filesystem-contract-freeze-v1.md)
- [044-v100-pack-contract-freeze-v1.md](044-v100-pack-contract-freeze-v1.md)
- [045-v100-projection-readmodel-contract-freeze-v1.md](045-v100-projection-readmodel-contract-freeze-v1.md)
- [046-v100-evidence-acceptance-contract-freeze-v1.md](046-v100-evidence-acceptance-contract-freeze-v1.md)
- [047-v100-executor-adapter-contract-freeze-v1.md](047-v100-executor-adapter-contract-freeze-v1.md)
- [033-event-replay-projection-rebuild-v1.md](033-event-replay-projection-rebuild-v1.md)
- [034-pack-migration-execution-model-v1.md](034-pack-migration-execution-model-v1.md)

## Certification Boundary

本认证只证明 v1.0 stable core 的升级闭环，不承诺所有实验版本自动升级。

认证必须覆盖：

- event replay；
- projection rebuild；
- Pack migration preview；
- Pack migration apply；
- Pack migration cancel；
- Pack migration rollback；
- filesystem retired path enforcement；
- semantic negative fixtures；
- deterministic report。

## Upgrade Path Contract

v1.0.0 最小 upgrade path：

```text
v0.9.x runtime facts
-> v1.0 filesystem contract check
-> event replay
-> projection rebuild
-> Pack migration receipt check
-> negative fixture check
-> upgrade certification report
```

升级不能写入或恢复 retired path：

```text
.agentflow/input/**
.agentflow/execute/**
.agentflow/output/**
.agentflow/goal-tree/**
```

如果 retired path 被写入、恢复或被当作 fallback authority，认证必须失败。

## Replay Certification

Replay certification 必须读取：

```text
runtime/event-replay-projection-report.json
runtime/event-replay-projection-failure-report.json
runtime/projection-readmodel-contract.json
```

Happy path 必须满足：

- `status = passed`；
- `eventCount > 0`；
- `taskCount > 0`；
- `rebuiltPaths` 非空；
- `writesAuthority = false`；
- `projectionAuthority = false`。

Failure path 必须满足：

- `status = failed`；
- `failures` 非空；
- failure reason 可读；
- `writesAuthority = false`；
- `projectionAuthority = false`。

## Migration Certification

Migration certification 必须读取：

```text
pack-migration-preview.json
pack-migration-unconfirmed-apply.json
pack-migration-applied-receipt.json
pack-migration-cancel-receipt.json
pack-migration-rollback-receipt.json
pack-migration-fake-authority-receipt.json
runtime/pack-migration-replay-report.json
```

稳定规则：

- preview 是只读预览；
- unconfirmed apply 必须失败；
- applied receipt 必须绑定 preview 和 explicit confirmation；
- cancel receipt 不等于 applied receipt；
- rollback receipt 不等于 applied receipt；
- applied / rollback 都是 receipt-only；
- fake authority receipt 必须被 negative fixture 拒绝；
- migration 后 replay / projection rebuild 必须可运行或输出结构化失败。

## Filesystem Migration Rule

filesystem migration 不能隐藏破坏性变更。

稳定规则：

- retired path 不能被 fallback 读取为 authority；
- retired path 不能被 replay 重新生成；
- retired path 不能被 migration apply 重新写入；
- compatibility 只能通过 explicit upgrade report 表达；
- release gate 必须输出 retired path 检查结果。

## Deterministic Report

release gate 必须生成：

```text
runtime/replay-migration-upgrade-certification.json
```

稳定字段：

```text
version
status
docPath
replayMigrationUpgradeCertificationVersion
upgradeSourceVersion
upgradeTargetVersion
eventReplayStatus
eventReplayFailureStatus
projectionRebuildStatus
packMigrationPreviewStatus
packMigrationApplyStatus
packMigrationCancelStatus
packMigrationRollbackStatus
retiredPathRevived
negativeUpgradeFixturePassed
deterministicReport
checkedAt
```

该 report 必须能复跑，不能依赖人工解释。

## Negative Upgrade Fixture

release gate 必须覆盖至少一个 negative upgrade fixture。

最小负例：

```text
fixture: retired-path-revival
input: .agentflow/input/issues/legacy.json
expectedStatus: failed
failedStage: filesystem-retired-path-check
writesAuthority: false
```

如果负例没有失败，认证必须失败。

## Rollback Guide

Rollback guide 必须说明：

- rollback target；
- rollback receipt；
- failed deployment / migration report；
- replay / projection proof；
- authority mutation 是否发生。

v1.0.0 当前 rollback 是 provider-agnostic proof，不调用云厂商 rollback API。

## V100 Binding

`V100-008` 完成后：

- release gate 必须生成 `runtime/replay-migration-upgrade-certification.json`；
- certification checklist 必须包含 `v100-replay-migration-upgrade-certification`；
- replay / migration / upgrade proof 必须进入 release certification；
- 下游 `V100-009`、`V100-010` 必须引用本认证。

## Non-goals

- 不承诺所有早期实验版本自动升级；
- 不做复杂数据库迁移平台；
- 不隐藏破坏性变更；
- 不把 migration preview 当成 apply；
- 不把 rollback receipt 当成真实 authority mutation。
