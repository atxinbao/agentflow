# AgentFlow v1.0.0 Project OS Stable Core Tasks V1

日期：2026-06-25
执行者：Codex

## Goal

`v1.0.0` 的目标不是继续扩功能，而是冻结 AgentFlow 的 Project OS 稳定核心。

主线是：

```text
Stable Contract Baseline
-> Runtime API / SDK Freeze
-> Filesystem Contract Freeze
-> Pack Contract Freeze
-> Projection Contract Freeze
-> Evidence / Acceptance Contract Freeze
-> Executor Adapter Contract Freeze
-> Replay / Migration / Upgrade Certification
-> Software Dev Pack Stable Baseline
-> v1.0.0 Release Certification
```

本版本要回答的是：

```text
AgentFlow 的底层 Project OS 合约是否已经稳定到可以支撑后续行业壳接入？
```

## Entry Gate

`v1.0.0` 有硬前置条件。

必须先完成 `v0.9.1` release certification，并得到：

```text
v1PlanningReadiness = ready
```

如果 `v0.9.1` 仍存在以下任一问题，`v1.0.0` 不允许进入执行：

- Governance 仍只是独立 report，没有进入 Runtime command admission；
- Deployment Evidence 仍只检查文件存在或 sha256；
- Pack migration receipt 仍伪装成 authority mutation；
- project `.agentflow/packs/**` path 仍没有 release gate proof；
- release source archive 仍没有自洽 Agent entry；
- negative semantic fixtures 仍不能阻断错误 happy path。

## Product Principle

`v1.0.0` 是稳定承诺，不是功能竞赛。

正确方向是：

```text
冻结核心合约。
明确兼容边界。
证明可复跑。
保留 Audit sidecar。
让 executor 被 AgentFlow 约束，而不是反过来拥有项目真相。
```

## Issues

| Issue | Title | Priority | Dependency | Status |
| --- | --- | --- | --- | --- |
| `V100-001` | Stable Contract Baseline | P0 | v0.9.1 certification ready | done |
| `V100-002` | Runtime API / SDK Freeze | P0 | V100-001 | done |
| `V100-003` | AgentFlow Filesystem Contract Freeze | P0 | V100-001 | done |
| `V100-004` | Pack Contract Freeze | P0 | V100-001, V100-003 | done |
| `V100-005` | Projection / Read Model Stable Contract | P0 | V100-002, V100-004 | done |
| `V100-006` | Evidence + Acceptance Stable Contract | P0 | V100-002, V100-005 | done |
| `V100-007` | Executor Adapter Stable Contract | P0 | V100-002, V100-006 | planned |
| `V100-008` | Replay / Migration / Upgrade Certification | P0 | V100-003, V100-004, V100-005, V100-006 | planned |
| `V100-009` | Software Dev Pack Stable Baseline | P1 | V100-004, V100-005, V100-006, V100-007 | planned |
| `V100-010` | v1.0.0 Release Certification | P0 | V100-001, V100-002, V100-003, V100-004, V100-005, V100-006, V100-007, V100-008, V100-009 | planned |

## V100-001 Stable Contract Baseline

### Scope

固化 v1.0 的稳定边界。

必须定义哪些对象在 v1 之后承诺兼容，哪些仍然是 internal implementation detail。

必须处理：

- stable public contract 清单；
- internal runtime implementation 清单；
- compatibility promise；
- breaking-change rule；
- deprecation rule；
- version field rule；
- release certification rule；
- v1 后新增能力不能破坏主链路。

### Acceptance

- 有一份 v1 stable contract baseline 文档；
- 文档明确 stable / internal / experimental 三类边界；
- 所有后续 V100 issue 都引用这份 baseline；
- release gate 能检查 stable contract version metadata；
- 如果缺 baseline，V100-002 到 V100-010 不允许进入 Done。

### Non-goals

- 不承诺所有历史草案兼容；
- 不把内部函数、临时 CLI 输出、debug fixture 全部变成 stable API。

### Closeout

- Stable Contract Baseline 已落到 [../architecture/041-v100-stable-contract-baseline-v1.md](../architecture/041-v100-stable-contract-baseline-v1.md)；
- baseline 明确 `stableContractVersion = agentflow-stable-contract-baseline.v1` 和 `stableContractStatus = active`；
- baseline 将 v1 contract 分为 stable public contracts、internal implementation details、experimental contracts 三类；
- release gate 新增 `stable.contract-baseline` stage，缺少 baseline 或 metadata 不一致时阻断；
- 后续 `V100-002` 到 `V100-010` 必须引用该 baseline，不能绕过 stable / internal / experimental 边界。

## V100-002 Runtime API / SDK Freeze

### Scope

冻结 command / query / event API。

必须引用 [../architecture/041-v100-stable-contract-baseline-v1.md](../architecture/041-v100-stable-contract-baseline-v1.md) 中的 Stable Public Contracts 与 Version Field Rule。

必须处理：

- command input contract；
- query input contract；
- event output contract；
- decision output contract；
- error model；
- version field；
- governance admission decision；
- accepted / rejected / deferred / failed 状态语义；
- Runtime API 与 CLI command 的关系；
- SDK 使用者能依赖的最小稳定面。

### Acceptance

- Runtime API / SDK contract 有稳定 schema；
- command path 不能绕过 Governance admission；
- rejected / deferred 不写 proposal 或 accepted event；
- error response 有稳定 code、stage、reason、evidence path；
- SDK 示例覆盖 command、query、event 三条路径；
- release gate 覆盖 API compatibility fixture。

### Non-goals

- 不冻结每个内部 Rust 函数签名；
- 不做多语言 SDK 全量实现；
- 不绑定特定云 API 网关。

### Closeout

- Runtime API / SDK Freeze 已落到 [../architecture/042-v100-runtime-api-sdk-freeze-v1.md](../architecture/042-v100-runtime-api-sdk-freeze-v1.md)；
- 文档明确 `runtimeApiSdkContractVersion = agentflow-runtime-api-sdk-freeze.v1` 和 `runtimeApiSdkContractStatus = active`；
- command、query、event、decision、error、governance admission、CLI / SDK relationship 和 minimal SDK surface 已冻结；
- release gate 新增 `runtime-api-sdk-compatibility` stage，生成 `runtime/runtime-api-sdk-compatibility.json`；
- compatibility fixture 会检查 command / query / event 三条路径、SDK readonly guard、rejected / deferred 不写 proposal / accepted event、错误模型和 SDK 示例覆盖；
- 下游 `V100-005`、`V100-006`、`V100-007`、`V100-010` 必须引用该 freeze 文档。

## V100-003 AgentFlow Filesystem Contract Freeze

### Scope

固化 `.agentflow/` 文件系统协议。

必须引用 [../architecture/041-v100-stable-contract-baseline-v1.md](../architecture/041-v100-stable-contract-baseline-v1.md) 中的 Stable Public Contracts、Deprecation Rule 和 retired path 规则。

必须处理：

- project facts；
- packs；
- spec projects；
- spec issues；
- tasks；
- runs；
- events；
- evidence；
- reports；
- local-only tmp；
- ignored runtime artifacts；
- source archive 必须包含的 Agent entry；
- 哪些路径是 authority，哪些路径是 projection，哪些路径是 local cache。

### Acceptance

- `.agentflow/` stable path contract 完整列出；
- 每个路径都有 owner、read/write rule、authority level、version rule；
- release source archive 与 local runtime state 的边界清楚；
- 禁止 retired path 被重新写入；
- release gate 覆盖 filesystem contract fixture。

### Non-goals

- 不把所有本地运行事实提交到 git；
- 不恢复 retired `.agentflow/input/**`、`.agentflow/output/**`、`.agentflow/goal-tree/**`。

### Closeout

- Filesystem Contract Freeze 已落到 [../architecture/043-v100-agentflow-filesystem-contract-freeze-v1.md](../architecture/043-v100-agentflow-filesystem-contract-freeze-v1.md)；
- 文档明确 `filesystemContractVersion = agentflow-filesystem-contract-freeze.v1` 和 `filesystemContractStatus = active`；
- `.agentflow/project/**`、`.agentflow/spec/**`、`.agentflow/runtime/**`、`.agentflow/packs/**`、`.agentflow/tasks/**`、`.agentflow/events/**`、`.agentflow/projections/**`、`.agentflow/indexes/**`、`.agentflow/release/**`、`.agentflow/audit/**`、`.agentflow/tmp/**` 的 owner、读写规则、authority level 和 version rule 已冻结；
- retired path 明确包括 `.agentflow/input/**`、`.agentflow/execute/**`、`.agentflow/output/**`、`.agentflow/goal-tree/**`、`.agentflow/define/goals/**`、`.agentflow/define/milestones/**`、`.agentflow/define/issues/**`；
- release gate 新增 `filesystem-contract` stage，生成 `runtime/filesystem-contract.json`；
- release gate 会检查 filesystem freeze metadata、required stable paths、authority classes、retired path 重新写入和 source archive / local runtime state 边界；
- 下游 `V100-004`、`V100-005`、`V100-006`、`V100-008`、`V100-010` 必须引用该 freeze 文档。

## V100-004 Pack Contract Freeze

### Scope

冻结 Domain Pack / Surface Pack / Connector Pack 的 schema、version、capability、migration 规则。

必须引用 [../architecture/041-v100-stable-contract-baseline-v1.md](../architecture/041-v100-stable-contract-baseline-v1.md) 中的 Pack contract、Compatibility Promise 和 Breaking Change Rule。
必须引用 [../architecture/043-v100-agentflow-filesystem-contract-freeze-v1.md](../architecture/043-v100-agentflow-filesystem-contract-freeze-v1.md) 中的 `.agentflow/packs/**` definition 边界。

必须处理：

- Pack manifest；
- Domain object / link / action definitions；
- Surface read model / view model definitions；
- Connector capability definitions；
- capability status；
- provider smoke binding；
- Pack fingerprint；
- Pack migration metadata；
- compatibility and migration rule；
- invalid / deferred Pack 状态。

### Acceptance

- Pack schema 有 stable version；
- project `.agentflow/packs/**` 是可验证 path；
- invalid Pack command 不能进入 Runtime proposal；
- disabled capability 不能被 command resolver 当成 available；
- Pack migration 不混淆 receipt-only 和 authority-applied；
- release gate 覆盖 Software Dev Pack 和 UI Design Pack fixtures。

### Non-goals

- 不做 Pack marketplace；
- 不承诺所有未来行业 Pack 已完成；
- 不把 UI Design Pack 提升为 v1 默认稳定行业壳。

### Closeout

- Pack Contract Freeze 已落到 [../architecture/044-v100-pack-contract-freeze-v1.md](../architecture/044-v100-pack-contract-freeze-v1.md)；
- 文档明确 `packContractVersion = agentflow-pack-contract-freeze.v1` 和 `packContractStatus = active`；
- Pack manifest、Domain Pack、Surface Pack、Connector Pack、capability status、Runtime entry、migration、built-in baseline、compatibility promise 和 breaking change rule 已冻结；
- `software-dev` 被定义为 v1 默认稳定行业壳，状态为 `completed`；
- `ui-design` 被定义为 baseline 行业壳，状态为 `baseline`，不提升为 v1 默认稳定主链；
- release gate 新增 `pack-contract-compatibility` stage，生成 `runtime/pack-contract-compatibility.json`；
- release gate 会检查 file-backed Pack registry、Pack validation / simulation / projection / API plane、negative fixtures、disabled capability、invalid command submit、unexpected fallback 和 migration receipt-only 边界；
- 下游 `V100-005`、`V100-006`、`V100-007`、`V100-008`、`V100-009`、`V100-010` 必须引用该 freeze 文档。

## V100-005 Projection / Read Model Stable Contract

### Scope

固化 Projection API、Read Model、View Model。

必须引用 [../architecture/041-v100-stable-contract-baseline-v1.md](../architecture/041-v100-stable-contract-baseline-v1.md) 中的 Projection / Read Model stable contract 和 projection 只读边界，并引用 [../architecture/042-v100-runtime-api-sdk-freeze-v1.md](../architecture/042-v100-runtime-api-sdk-freeze-v1.md) 中的 Query API 只读边界。
必须引用 [../architecture/043-v100-agentflow-filesystem-contract-freeze-v1.md](../architecture/043-v100-agentflow-filesystem-contract-freeze-v1.md) 中的 `.agentflow/projections/**` 和 `.agentflow/indexes/**` 只读边界。
必须引用 [../architecture/044-v100-pack-contract-freeze-v1.md](../architecture/044-v100-pack-contract-freeze-v1.md) 中的 Pack Surface / Domain readiness，但不能让 Pack 拥有 projection authority。

原则：

```text
行业客户端只读 Projection。
Projection 不拥有 authority。
UI 不直接读 Event Store 写路径。
```

必须处理：

- Projection API；
- read model schema；
- view model schema；
- projection rebuild rule；
- stale / invalid / deferred 状态；
- Pack-specific projection loading；
- evidence graph read model；
- audit sidecar read model；
- delivery read model。

### Acceptance

- Projection schema 有 stable version；
- Projection 可以从 Event Store 重建；
- Projection missing Pack definition 时显示 invalid / deferred，不静默回退 Software Dev；
- Industry Surface 只能消费 Projection / Read Model；
- release gate 覆盖 projection rebuild compatibility fixture。

### Closeout

- [../architecture/045-v100-projection-readmodel-contract-freeze-v1.md](../architecture/045-v100-projection-readmodel-contract-freeze-v1.md) 冻结 Projection API、Read Model、View Model、Rebuild、Freshness、Pack-specific projection、Evidence / Audit / Delivery sidecar read model 和 Industry Surface 只读边界；
- release gate 增加 `projection-readmodel-contract` 阶段，并生成 `runtime/projection-readmodel-contract.json`；
- release gate 会检查 projection replay happy / failure path、Query API readonly、Pack projection invalid / deferred、project / task / spec read model version、sidecar read model API 和 industry surface 只读规则；
- 下游 `V100-006`、`V100-007`、`V100-008`、`V100-009`、`V100-010` 必须引用该 freeze 文档。

### Non-goals

- 不做全新 UI 大改版；
- 不把 Projection 变成写入 authority；
- 不要求每个行业壳都完成。

## V100-006 Evidence + Acceptance Stable Contract

### Scope

把验证、证据、验收、完成写入做成稳定闭环。

必须引用 [../architecture/041-v100-stable-contract-baseline-v1.md](../architecture/041-v100-stable-contract-baseline-v1.md) 中的 Evidence / Acceptance contract、Completion Commit 边界和 Audit sidecar 边界，并引用 [../architecture/042-v100-runtime-api-sdk-freeze-v1.md](../architecture/042-v100-runtime-api-sdk-freeze-v1.md) 中的 Decision Output Contract 与 Error Model。
必须引用 [../architecture/043-v100-agentflow-filesystem-contract-freeze-v1.md](../architecture/043-v100-agentflow-filesystem-contract-freeze-v1.md) 中的 `.agentflow/tasks/<issue-id>/evidence/**`、`.agentflow/release/**` 和 public record boundary。
必须引用 [../architecture/044-v100-pack-contract-freeze-v1.md](../architecture/044-v100-pack-contract-freeze-v1.md) 中的 Pack evidence policy 边界，但 Done 仍由 Acceptance Gate 决定。
必须引用 [../architecture/045-v100-projection-readmodel-contract-freeze-v1.md](../architecture/045-v100-projection-readmodel-contract-freeze-v1.md) 中的 Evidence Graph / Audit Sidecar / Delivery Read Model，只能把验收事实投影给 UI，不能让 Projection 写完成 authority。

主链：

```text
Confirmed Work
-> Admission
-> Execution
-> Verification
-> Evidence Pack
-> Acceptance Gate
-> Completion Commit
-> Done
```

必须处理：

- verification result；
- evidence pack；
- acceptance criteria；
- acceptance decision；
- failure reasons；
- completion commit；
- event append；
- issue / run status writeback；
- delivery record；
- Audit trigger evaluation 只作为 sidecar 判断，不阻断默认 Done。

### Acceptance

- Acceptance Gate 汇总 Verification Gate、Evidence Gate、Contract Gate、State Gate；
- Done 只能由 Acceptance Decision passed 后进入；
- Completion Commit 是唯一完成写入边界；
- failed acceptance 有稳定 reason 和 evidence path；
- Audit 不进入主业务链；
- release gate 覆盖 pass / fail / missing evidence / state blocked fixtures。

### Non-goals

- 不把 Audit Agent 变成默认阻断流程；
- 不把人工审查当成唯一验收依据；
- 不把 CI 当成唯一验证 authority。

### Closeout

- [../architecture/046-v100-evidence-acceptance-contract-freeze-v1.md](../architecture/046-v100-evidence-acceptance-contract-freeze-v1.md) 冻结 Evidence Pack、Acceptance Gate、Completion Commit、failure reason、status writeback、delivery record 和 Audit sidecar 边界；
- 文档明确 `evidenceAcceptanceContractVersion = agentflow-evidence-acceptance-contract.v1` 和 `evidenceAcceptanceContractStatus = active`；
- Acceptance Gate 明确汇总 Verification Gate、Evidence Gate、Contract Gate、State Gate；
- Completion Commit 明确为唯一完成写入边界，Done 只能从 passed Acceptance Decision 和 Completion Commit 派生；
- Audit sidecar 明确不属于默认 Done 主链，audit failed 可以阻断 project release readiness，但不回滚 task Done；
- release gate 新增 `evidence-acceptance-contract` stage，生成 `runtime/evidence-acceptance-contract.json`；
- release gate 会检查 Evidence Pack、Acceptance event、Completion Commit event、closeout proof、task projection Done、delivery read model、Audit sidecar 非 Done authority 和 pass / fail / missing evidence / state blocked fixtures；
- 下游 `V100-007`、`V100-008`、`V100-009`、`V100-010` 必须引用该 freeze 文档。

## V100-007 Executor Adapter Stable Contract

### Scope

固化 Codex / Claude Code 等执行器适配合同。

必须引用 [../architecture/041-v100-stable-contract-baseline-v1.md](../architecture/041-v100-stable-contract-baseline-v1.md) 中的 Executor Adapter contract 和 executor 不拥有 project truth 的规则，并引用 [../architecture/042-v100-runtime-api-sdk-freeze-v1.md](../architecture/042-v100-runtime-api-sdk-freeze-v1.md) 中的 Command API 与 Governance Admission Rule。
必须引用 [../architecture/044-v100-pack-contract-freeze-v1.md](../architecture/044-v100-pack-contract-freeze-v1.md) 中的 Connector Pack capability boundary。
必须引用 [../architecture/045-v100-projection-readmodel-contract-freeze-v1.md](../architecture/045-v100-projection-readmodel-contract-freeze-v1.md) 中的 Work loop session view 和 Runtime health view，只能把 executor session 暴露为 Projection read model。
必须引用 [../architecture/046-v100-evidence-acceptance-contract-freeze-v1.md](../architecture/046-v100-evidence-acceptance-contract-freeze-v1.md) 中的 Evidence Pack、Acceptance Gate 和 Completion Commit 边界。

AgentFlow 管：

- 当前 issue；
- role；
- allowed surface；
- non-goals；
- acceptance criteria；
- expected outputs；
- evidence policy；
- completion writeback。

Executor 管：

- model call；
- tool call；
- shell / file edit；
- local session；
- context window；
- provider behavior。

必须处理：

- work handoff schema；
- allowed path / denied path；
- expected outputs；
- evidence return；
- diff boundary check；
- session isolation guidance；
- executor result normalization；
- executor runtime 不能成为 project truth。

### Acceptance

- Executor adapter contract 有 stable schema；
- Codex / Claude Code handoff 都能映射到同一 AgentFlow task contract；
- executor 越界修改会被 post-run validation 拒绝推进状态；
- executor session / memory 不被当成 AgentFlow authority；
- release gate 覆盖 accepted / rejected executor result fixtures。

### Non-goals

- 不重写 Codex / Claude Code 内部 runtime；
- 不保证所有第三方 executor 行为完全一致；
- 不把 executor 的 chat history 当成项目事实。

### Closeout

- [../architecture/047-v100-executor-adapter-contract-freeze-v1.md](../architecture/047-v100-executor-adapter-contract-freeze-v1.md) 冻结 Executor Adapter、work handoff、allowed / denied path、expected outputs、Evidence return、diff boundary、session isolation、provider mapping 和 executor result 归一化合同；
- 文档明确 `executorAdapterContractVersion = agentflow-executor-adapter-contract.v1` 和 `executorAdapterContractStatus = active`；
- AgentFlow / executor 权威边界已固定：AgentFlow 管任务合同、证据、验收和完成写回，executor 只负责实际执行；
- Codex / Claude Code handoff 必须映射到同一 AgentFlow task contract；
- executor session、chat history、memory 和 provider-local 状态不得成为 project truth；
- release gate 新增 `executor-adapter-contract` stage，生成 `runtime/executor-adapter-contract.json`；
- release gate 覆盖 accepted、rejected、deferred executor result fixtures，并验证越界 diff 不写 Evidence / Done；
- 下游 `V100-008`、`V100-009`、`V100-010` 必须引用该 freeze 文档。

## V100-008 Replay / Migration / Upgrade Certification

### Scope

证明 event replay、projection rebuild、Pack migration、旧版本升级路径可复跑。

必须引用 [../architecture/041-v100-stable-contract-baseline-v1.md](../architecture/041-v100-stable-contract-baseline-v1.md) 中的 Breaking Change Rule、Deprecation Rule 和 Release Certification Rule。
必须引用 [../architecture/043-v100-agentflow-filesystem-contract-freeze-v1.md](../architecture/043-v100-agentflow-filesystem-contract-freeze-v1.md) 中的 retired path 规则，证明迁移和 replay 不会恢复旧 `.agentflow/input/**`、`.agentflow/execute/**`、`.agentflow/output/**` 或 `.agentflow/goal-tree/**`。
必须引用 [../architecture/044-v100-pack-contract-freeze-v1.md](../architecture/044-v100-pack-contract-freeze-v1.md) 中的 Pack migration receipt-only 和 rollback boundary。
必须引用 [../architecture/045-v100-projection-readmodel-contract-freeze-v1.md](../architecture/045-v100-projection-readmodel-contract-freeze-v1.md) 中的 Projection rebuild rule、freshness state 和 structured failure 规则。
必须引用 [../architecture/046-v100-evidence-acceptance-contract-freeze-v1.md](../architecture/046-v100-evidence-acceptance-contract-freeze-v1.md) 中的 Evidence Pack、Acceptance Gate、Completion Commit 和 failure reason 规则。
必须引用 [../architecture/047-v100-executor-adapter-contract-freeze-v1.md](../architecture/047-v100-executor-adapter-contract-freeze-v1.md) 中的 executor session isolation、result normalization 和 rejected executor result 不写 authority 规则。

必须处理：

- event replay；
- projection rebuild；
- Pack migration apply / rollback；
- filesystem contract migration；
- upgrade guide；
- rollback guide；
- semantic fixture；
- negative fixture；
- deterministic report。

### Acceptance

- 至少覆盖 v0.9.x 到 v1.0.0 的 upgrade path；
- replay 后 Projection 与 expected read model 一致；
- migration receipt 和 authority-applied 状态可区分；
- rollback target 语义可验证；
- negative upgrade fixture 能在正确 stage 失败；
- certification report 可复跑。

### Non-goals

- 不承诺所有早期实验版本自动升级；
- 不做复杂数据库迁移平台；
- 不隐藏破坏性变更。

### Closeout

- [../architecture/048-v100-replay-migration-upgrade-certification-v1.md](../architecture/048-v100-replay-migration-upgrade-certification-v1.md) 冻结 event replay、projection rebuild、Pack migration、retired path、negative fixture 和 deterministic upgrade certification 合同；
- 文档明确 `replayMigrationUpgradeCertificationVersion = agentflow-replay-migration-upgrade-certification.v1` 和 `replayMigrationUpgradeCertificationStatus = active`；
- upgrade path 明确覆盖 `v0.9.x runtime facts -> v1.0 filesystem contract check -> event replay -> projection rebuild -> Pack migration receipt check -> negative fixture check -> upgrade certification report`；
- retired path `.agentflow/input/**`、`.agentflow/execute/**`、`.agentflow/output/**`、`.agentflow/goal-tree/**` 不得被 migration、replay 或 fallback 恢复为 authority；
- release gate 新增 `replay-migration-upgrade-certification` stage，生成 `runtime/replay-migration-upgrade-certification.json`；
- release gate 会检查 event replay happy / failure、projection rebuild、Pack migration preview / apply / cancel / rollback、fake authority receipt、retired path negative fixture 和 deterministic report；
- 下游 `V100-009`、`V100-010` 必须引用该 certification 文档。

## V100-009 Software Dev Pack Stable Baseline

### Scope

把 Software Dev Pack 作为 v1.0 默认稳定行业壳。

必须引用 [../architecture/041-v100-stable-contract-baseline-v1.md](../architecture/041-v100-stable-contract-baseline-v1.md) 中的 stable / internal / experimental 边界，不能把 experimental Pack 能力伪装成 stable。
必须引用 [../architecture/044-v100-pack-contract-freeze-v1.md](../architecture/044-v100-pack-contract-freeze-v1.md) 中的 Software Dev Pack stable baseline。
必须引用 [../architecture/045-v100-projection-readmodel-contract-freeze-v1.md](../architecture/045-v100-projection-readmodel-contract-freeze-v1.md) 中的 Pack-specific projection loading、Task Workbench view、Delivery read model 和 Audit sidecar read model。
必须引用 [../architecture/046-v100-evidence-acceptance-contract-freeze-v1.md](../architecture/046-v100-evidence-acceptance-contract-freeze-v1.md) 中的 Evidence / Acceptance / Completion / Delivery / Audit sidecar 边界。
必须引用 [../architecture/047-v100-executor-adapter-contract-freeze-v1.md](../architecture/047-v100-executor-adapter-contract-freeze-v1.md) 中的软件开发 executor handoff、Git / GitHub / Codex / Claude Code provider mapping 和 diff boundary 规则。
必须引用 [../architecture/048-v100-replay-migration-upgrade-certification-v1.md](../architecture/048-v100-replay-migration-upgrade-certification-v1.md) 中的 upgrade path、retired path negative fixture 和 migration / replay certification 规则。

必须证明软件开发现场可闭环：

```text
Requirement
-> Spec
-> Issue
-> Run
-> Evidence
-> Acceptance
-> Delivery
-> Optional Audit sidecar
```

必须处理：

- Software Dev Domain Pack；
- Software Dev Surface Pack；
- Software Dev Connector Pack；
- Git / GitHub / executor connector baseline；
- Project Home / Task Workbench / Delivery / Audit sidecar read model；
- Release flow；
- Audit 仍然独立。

### Acceptance

- Software Dev Pack 有 stable manifest；
- Software Dev read models 不直接读 authority write path；
- GitHub issue 仍只是临时协作镜像，不成为 AgentFlow authority；
- Delivery 和 Audit sidecar 的边界可验证；
- 至少一条 Software Dev fixture 从 intake 到 Done 可复跑。

### Non-goals

- 不把 UI Design Pack 作为 v1 stable 要求；
- 不做所有行业壳；
- 不把 Audit 并回主业务链。

## V100-010 v1.0.0 Release Certification

### Scope

最终发布认证。

必须引用 [../architecture/041-v100-stable-contract-baseline-v1.md](../architecture/041-v100-stable-contract-baseline-v1.md)，并把 `stableContractVersion` / `stableContractStatus` 作为 release certification 的硬门禁。
必须引用 [../architecture/043-v100-agentflow-filesystem-contract-freeze-v1.md](../architecture/043-v100-agentflow-filesystem-contract-freeze-v1.md)，并把 `filesystemContractVersion` / `filesystemContractStatus` / retired path 检查作为 release certification 的硬门禁。
必须引用 [../architecture/044-v100-pack-contract-freeze-v1.md](../architecture/044-v100-pack-contract-freeze-v1.md)，并把 `packContractVersion` / `packContractStatus` / Pack compatibility 检查作为 release certification 的硬门禁。
必须引用 [../architecture/045-v100-projection-readmodel-contract-freeze-v1.md](../architecture/045-v100-projection-readmodel-contract-freeze-v1.md)，并把 `projectionContractVersion` / `projectionContractStatus` / Projection rebuild compatibility 检查作为 release certification 的硬门禁。
必须引用 [../architecture/046-v100-evidence-acceptance-contract-freeze-v1.md](../architecture/046-v100-evidence-acceptance-contract-freeze-v1.md)，并把 `evidenceAcceptanceContractVersion` / `evidenceAcceptanceContractStatus` / Acceptance Gate compatibility 检查作为 release certification 的硬门禁。
必须引用 [../architecture/047-v100-executor-adapter-contract-freeze-v1.md](../architecture/047-v100-executor-adapter-contract-freeze-v1.md)，并把 `executorAdapterContractVersion` / `executorAdapterContractStatus` / Executor result compatibility 检查作为 release certification 的硬门禁。
必须引用 [../architecture/048-v100-replay-migration-upgrade-certification-v1.md](../architecture/048-v100-replay-migration-upgrade-certification-v1.md)，并把 `replayMigrationUpgradeCertificationVersion` / `replayMigrationUpgradeCertificationStatus` / upgrade compatibility 检查作为 release certification 的硬门禁。

必须输出：

- stable contract baseline proof；
- Runtime API / SDK compatibility proof；
- filesystem contract proof；
- Pack contract proof；
- Projection / Read Model proof；
- Evidence / Acceptance proof；
- Executor Adapter proof；
- replay / migration / upgrade proof；
- Software Dev Pack stable proof；
- negative fixture coverage；
- remaining risk / deferred list；
- v1 support boundary。
- Runtime API / SDK compatibility fixture。

### Acceptance

- V100-001 到 V100-009 都有 release gate coverage；
- certification 明确 `v1StableCore = ready | blocked`；
- 如果 Governance admission 不在主链，必须 blocked；
- 如果 Projection 仍能绕过 authority 边界，必须 blocked；
- 如果 Acceptance 不能决定 Done，必须 blocked；
- 如果 Audit 被放回主业务链，必须 blocked；
- 如果 executor runtime 被当成 project truth，必须 blocked；
- 如果 v1 compatibility boundary 不清楚，必须 blocked。

### Non-goals

- 不替代独立 Audit Agent 流程；
- 不承诺长期商业 SLA；
- 不承诺所有 future Pack 兼容；
- 不把 v1.0.0 当成行业市场发布。

## Execution Order

建议执行顺序：

```text
V100-001
-> V100-002
-> V100-003
-> V100-004
-> V100-005
-> V100-006
-> V100-007
-> V100-008
-> V100-009
-> V100-010
```

`V100-010` 必须最后执行。

## Stable Core Completion Target

`v1.0.0` 完成后，AgentFlow 底层能力应达到：

```text
Core Project OS capability: about 80%
```

这个 80% 表示：

- 软件开发 AgentFlow 可以稳定运行；
- 第二、第三个行业壳可以基于 Pack contract 接入试点；
- Runtime / Pack / Projection / Evidence / Acceptance / Executor 的边界稳定；
- 后续能力扩展不能破坏 v1 stable core。

剩余能力进入 v1.x：

- 多行业规模化；
- 云端 Runtime 产品化；
- 跨进程调度和 Message Bus 决策；
- OS Console 深化；
- 第三方 Pack / Skill / Connector 生态；
- 长期兼容与 deprecation 流程。
