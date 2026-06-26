# V100 Software Dev Pack Stable Baseline V1

创建日期：2026-06-25
执行者：Codex

## Metadata

```yaml
documentId: agentflow-v100-software-dev-pack-stable-baseline-v1
softwareDevPackStableBaselineVersion: agentflow-software-dev-pack-stable-baseline.v1
softwareDevPackStableBaselineStatus: active
authority: docs/core/architecture/049-v100-software-dev-pack-stable-baseline-v1.md
stableContractBaseline: agentflow-stable-contract-baseline.v1
packContractVersion: agentflow-pack-contract-freeze.v1
projectionContractVersion: agentflow-projection-readmodel-contract.v1
evidenceAcceptanceContractVersion: agentflow-evidence-acceptance-contract.v1
executorAdapterContractVersion: agentflow-executor-adapter-contract.v1
replayMigrationUpgradeCertificationVersion: agentflow-replay-migration-upgrade-certification.v1
dependsOn:
  - docs/core/architecture/041-v100-stable-contract-baseline-v1.md
  - docs/core/architecture/044-v100-pack-contract-freeze-v1.md
  - docs/core/architecture/045-v100-projection-readmodel-contract-freeze-v1.md
  - docs/core/architecture/046-v100-evidence-acceptance-contract-freeze-v1.md
  - docs/core/architecture/047-v100-executor-adapter-contract-freeze-v1.md
  - docs/core/architecture/048-v100-replay-migration-upgrade-certification-v1.md
runtimeArtifact: runtime/software-dev-pack-stable-baseline.json
```

## 1. Certification Goal

`Software Dev Pack` 是 v1.0 默认稳定行业壳。

它必须证明软件开发现场能通过稳定 Pack 表达为：

```text
Requirement
-> Spec
-> Issue
-> Run
-> Evidence
-> Acceptance
-> Delivery
-> Release
```

Audit 仍是独立 sidecar：

```text
Delivery / Done
-> Optional Audit Request
-> Audit Report
-> Finding
-> Follow-up Proposal
```

这份文档冻结的是 v1.0 级别的稳定性证明，不是新增行业壳能力。

## 2. Stable Pack Boundary

Software Dev Pack 只能声明：

- Domain object；
- Surface page；
- Connector capability；
- Read model requirement；
- Command entry；
- Evidence policy；
- Acceptance boundary；
- Delivery boundary；
- Optional Audit sidecar。

Software Dev Pack 不能：

- 写 `.agentflow/spec/**`；
- 写 `.agentflow/events/**`；
- 写 `.agentflow/tasks/**`；
- 直接执行 provider；
- 直接创建 PR / MR；
- 把 GitHub issue 当成 AgentFlow authority；
- 把 Audit 合并进主链；
- 把 finding 直接变成主链 blocker。

## 3. Stable Manifest Requirement

release gate 必须证明：

- Pack registry 来自 project files；
- `software-dev` pack entry 存在；
- registry entry `fallback = false`；
- manifest path 存在；
- Pack validation report status 为 `passed`；
- Pack simulation report status 为 `passed`；
- Pack projection readiness status 为 `passed`；
- Software Dev readiness status 为 `completed`；
- UI Design readiness 只能是 `baseline`，不能升级为 v1 stable 要求。

## 4. Read Model Requirement

Software Dev Pack 的 read model 只能读 projection / runtime read surface，不能直接读 authority write path。

必须覆盖：

- Project Home；
- Task Workbench；
- Acceptance；
- Delivery；
- Event Timeline；
- Evidence Graph；
- Audit Surface；
- Finding Review；
- Follow-up Proposal。

Task / Delivery / Audit 的关系必须保持：

- Task 主链承载 Requirement -> Done；
- Delivery 是主链交付记录；
- Audit 是 sidecar 验收，不自动阻断主链。

## 5. Connector Baseline

v1.0 stable connector baseline：

```text
Git
GitHub
Codex
Claude
Browser Preview
```

connector 只能提供：

- capability；
- launch request；
- session evidence；
- diff evidence；
- PR / MR metadata；
- local validation evidence。

connector 不得写 authority。

## 6. Runtime Fixture Requirement

release gate 必须证明至少一条 Software Dev fixture 可复跑：

```text
Raw request
-> Requirement
-> Spec
-> Issue
-> Run
-> Evidence
-> Acceptance
-> Delivery
-> Done
```

证明材料至少包含：

- runtime task loop stage；
- session snapshot；
- validation command evidence；
- acceptance gate evidence；
- delivery summary；
- closeout / completion proof。

## 7. Audit Sidecar Requirement

Audit sidecar 必须可展示、可请求、可记录，但默认不阻断 Software Dev Pack 主链。

release gate 必须证明：

- readiness artifact 包含 `OptionalAuditRequest`；
- readiness artifact 包含 `AuditReport`；
- readiness artifact 包含 `Finding`；
- readiness artifact 包含 `FollowUpProposal`；
- `Finding` 只能生成 follow-up proposal；
- Audit sidecar 不会改变 main chain completion。

## 8. V100 Binding

`V100-009` 通过时，release gate 必须生成：

```text
runtime/software-dev-pack-stable-baseline.json
```

该 artifact 必须包含：

- `softwareDevPackStableBaselineVersion`；
- `softwareDevPackStableBaselineStatus`；
- Pack manifest / registry proof；
- read model boundary proof；
- connector baseline proof；
- runtime fixture proof；
- delivery boundary proof；
- audit sidecar proof；
- downstream contract references。

`V100-010` release certification 必须把该 artifact 作为硬门禁。

## 9. Non-goals

- 不把 UI Design Pack 提升为 v1 stable 要求；
- 不新增行业 Pack；
- 不改变 Software Dev Pack 主链；
- 不把 Audit 并回主业务链；
- 不新增 provider 执行能力；
- 不新增 remote authority。
