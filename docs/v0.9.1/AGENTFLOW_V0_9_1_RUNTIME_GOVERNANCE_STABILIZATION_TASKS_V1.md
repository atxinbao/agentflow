# AgentFlow v0.9.1 Runtime Governance Stabilization Tasks V1

日期：2026-06-25
执行者：Codex

## Goal

`v0.9.1` 聚焦 `v0.9.0` 发布审计后的稳定化修复。

主线是：

```text
Release Source Agent Entry
-> Runtime Governance Admission
-> Deployment Evidence Semantics
-> Pack Migration Semantic Split
-> Project Pack Registry Fixture
-> Negative Semantic Fixtures
-> Release Certification
```

本版本要回答的是：

```text
v0.9.0 的治理、部署证据、migration 和 Pack registry 是否已经真正接入 Runtime 闭环？
```

## Audit Findings

`v0.9.0` 可以保留为功能发布，但不能直接作为 `v1.0` clean stable 基线。

发布审计发现：

- Runtime Governance 目前是独立 CLI report，没有接入 `execute_command_via_arbitration` 的主写路径；
- Deployment Evidence 主要检查文件存在和 sha256，没有验证 release facts / remote proof / tag / commit / URL / rollback target 的语义一致性；
- Pack migration apply / rollback 只生成 receipt，却把 `writesAuthority` 标记为 `true`，容易混淆 receipt 和真实 authority mutation；
- Pack release gate 主要读取 `crates/pack/fixtures/packs/**`，没有证明项目级 `.agentflow/packs/**` path 可以工作；
- release source archive 不包含 tracked `AGENTS.md` 和 `.agentflow/define/agent/**`，但本地 Agent entry 要求读取这些入口。

## Product Principle

`v0.9.1` 不继续扩大架构。

正确修复方向是：

```text
把 report 接入 gate。
把 receipt 和真实 mutation 分开。
把 fixture proof 和 project path proof 分开。
把 source archive 和 Agent entry 对齐。
把 release evidence 从存在性检查升级成语义检查。
```

## Issues

| Issue | Title | Priority | Dependency | Status |
| --- | --- | --- | --- | --- |
| `V091-001` | Release Source Agent Entry Alignment | P0 | none | done |
| `V091-002` | Runtime Governance Admission Integration | P0 | V091-001 | done |
| `V091-003` | Deployment Evidence Semantic Certification | P0 | V091-001 | done |
| `V091-004` | Pack Migration Apply/Rollback Semantic Split | P0 | V091-003 | planned |
| `V091-005` | Project Pack Registry Release Fixture | P1 | V091-001 | planned |
| `V091-006` | Negative Semantic Release Fixtures | P0 | V091-002, V091-003, V091-004, V091-005 | planned |
| `V091-007` | v0.9.1 Release Certification | P0 | V091-001, V091-002, V091-003, V091-004, V091-005, V091-006 | planned |

## V091-001 Release Source Agent Entry Alignment

### Scope

让发布源码包含可读的 Agent entry，或把 `AGENTS.md` 改成指向 tracked docs。

不要把所有 runtime facts 纳入仓库。需要纳入或映射的是 Agent 操作入口和稳定手册。

必须处理：

- source archive 中的 Agent entry 自洽；
- `AGENTS.md` 是否应该退出 `.gitignore`；
- `.agentflow/define/agent/**` 中哪些内容属于 stable manual，哪些属于 local runtime state；
- release gate 能检查 source archive 中的入口可读性；
- 文档说明 runtime facts 仍然不进入发布源码。

### Acceptance

- 从 release source checkout 可以找到 Agent entry；
- Agent entry 指向的手册路径存在，或明确指向 tracked docs 等价入口；
- `.agentflow/runs/**`、`.agentflow/tmp/**`、本地数据库和 runtime artifacts 仍然不进入发布源码；
- release gate 覆盖 Agent entry source alignment。

### Closeout

- 根目录 `AGENTS.md` 已退出 `.gitignore`，作为 release source 中的稳定 Agent entry。
- `AGENTS.md` 指向 tracked docs，而不是要求 release source 携带本地 `.agentflow/define/agent/**`。
- `docs/architecture/040-release-source-agent-entry-v1.md` 记录 source entry 和 runtime-only 边界。
- release gate 新增 `source.agent-entry` 阶段，并输出 `runtime/source-agent-entry.json`。

### Non-goals

- 不把本地执行事实、run、task evidence 全量提交到仓库；
- 不绕过 AgentFlow 现有 role boundary。

## V091-002 Runtime Governance Admission Integration

### Scope

把 Governance 接进 Runtime command path。

`rejected` / `deferred` 不能进入 proposal、Arbitration 或 accepted event。

必须处理：

- `execute_command_via_arbitration` 在写 proposal 前执行 governance admission；
- governance report 成为 Runtime command decision 的一部分；
- `rejected` 输出 rejected decision fact，不写 proposal fact；
- `deferred` 输出 deferred decision fact，不写 proposal fact；
- `allowed` 才能进入 proposal / Arbitration；
- disabled capability 和 failed provider smoke 能阻断 Runtime admission；
- audit sidecar bound-to-main-chain 必须被拒绝。

### Acceptance

- Governance `rejected` / `deferred` path 不产生 Runtime proposal fact；
- Governance `rejected` / `deferred` path 不 append accepted action event；
- Runtime response 能说明 governance decision、stage 和 reason；
- release gate 增加 governance admission happy / reject / defer fixtures；
- `governance-policy evaluate` 保留为诊断命令，但不再是唯一治理入口。

### Closeout

- `execute_command_via_arbitration` 已在写入 proposal fact 前执行 Runtime Governance admission。
- Governance `rejected` / `deferred` 路径只写 command fact 和 decision fact，不写 proposal fact，不写 accepted action fact。
- `RuntimeCommandResponse` 和 `RuntimeDecisionFact` 已携带 `governanceAdmission` 报告，包含 decision、stage trace 和 reason。
- 新增 `runtime-command execute` CLI，用于 release gate 走真实 Runtime command path，而不是只跑 `governance-policy evaluate` 诊断命令。
- release gate 新增 `governance-admission` 阶段，并输出 `runtime/governance-admission.json`，覆盖 allowed / deferred / rejected 三类 fixture。

### Non-goals

- 不实现多租户权限系统；
- 不把 Governance 变成独立 authority store。

## V091-003 Deployment Evidence Semantic Certification

### Scope

把 Deployment Evidence 从文件存在性检查升级为语义一致性检查。

必须处理：

- 解析 release facts；
- 解析 remote release proof；
- 检查 release tag 一致；
- 检查 release commit 一致；
- 检查 remote release URL / provider / release id；
- 检查 artifact manifest path 和 sha256；
- 检查 rollback target tag / commit；
- 检查 pack fingerprint、event replay report、projection rebuild proof、migration receipt 的版本和状态。

### Acceptance

- tag / commit / URL / release id 不一致时 deployment evidence 必须失败；
- rollback target 不一致时 deployment evidence 必须失败；
- 缺 artifact manifest 或 sha256 不再只算存在性通过；
- release gate 覆盖 semantic success 和 semantic failure fixtures；
- `cloudDeployment.status = ready` 必须基于 semantic validation，而不是只基于 `remote-release-proof` 文件存在。

### Non-goals

- 不绑定特定云厂商；
- 不要求真实云部署，只要求 release proof 语义可验证。

### Closeout

- `DeploymentEvidenceReport` 增加 `semanticChecks` / `semanticFailures`，语义失败不再被隐藏在文件存在性检查之后；
- release facts / remote release proof / artifact manifest / rollback target / pack registry / event replay / projection rebuild / migration / rollback receipt 全部进入语义校验；
- `cloudDeployment.status = ready` 只有在远端证明和 artifact manifest 语义一致时成立；
- release gate 新增 `runtime/deployment-evidence-semantic-failure.json`，覆盖 tag mismatch 和 artifact manifest sha 缺失负例；
- `certification.json` 新增 `v091-deployment-evidence-semantics` checklist。

## V091-004 Pack Migration Apply/Rollback Semantic Split

### Scope

区分 receipt-only 和 authority-applied。

如果没有真实迁移，不要标 `writesAuthority = true`。

必须处理：

- `migration-preview` 保持只读；
- `migration-apply --confirmed` 先生成 apply receipt，但不能伪装成真实 authority mutation；
- 如果实现真实 migration apply，必须写清楚 mutation target、event、receipt 和 rollback target；
- `migration-rollback` 同样区分 rollback receipt 和真实 rollback mutation；
- release gate 必须验证 receipt-only 不被当成 authority-applied。

### Acceptance

- receipt-only artifact 使用 `writesAuthority = false`；
- authority-applied artifact 只有在真实 mutation 发生后才允许 `writesAuthority = true`；
- applied receipt 和 rollback receipt 都包含 semantic target；
- fake applied receipt 不能通过 release gate；
- migration 后 replay / projection rebuild 必须验证真实变更或明确说明 no-op。

### Non-goals

- 不做复杂跨版本数据迁移框架；
- 不隐式修改 Pack schema 或项目状态。

## V091-005 Project Pack Registry Release Fixture

### Scope

Release gate 要创建并读取项目级 `.agentflow/packs/**` fixture，证明 project Pack path 可工作。

必须处理：

- 在 release gate workspace 下创建 `.agentflow/packs/software-dev/pack.json`；
- 创建 `.agentflow/packs/ui-design/pack.json`；
- Registry read path 必须走 project root；
- fixture-backed crate path 和 project path 分开证明；
- 空 `.agentflow/packs` 不能被误报为 Pack ready。

### Acceptance

- `pack registry` 在项目级 `.agentflow/packs/**` fixture 下返回 Software Dev / UI Design；
- release gate 证明 registry source 是 `project-files`；
- 缺少 project Pack 时失败或明确 deferred，不能以 `entries=[]` + `fallback=false` 通过；
- Pack readiness 不再只依赖 `load_pack_fixture_registry()`。

### Non-goals

- 不要求把真实用户项目 Pack 提交到仓库；
- 不实现 Pack marketplace。

## V091-006 Negative Semantic Release Fixtures

### Scope

增加负向语义夹具，防止 release gate 只证明 happy path。

必须覆盖：

- wrong release tag；
- wrong release commit；
- wrong remote release URL；
- missing artifact manifest sha256；
- disabled capability still executing；
- governance rejected command still producing proposal；
- fake migration receipt；
- empty project Pack registry reported as ready。

### Acceptance

- 每个 negative fixture 都在正确 stage 失败；
- failure report 包含 stage、reason、evidence path；
- negative fixture 不写 authority；
- release certification 会列出 negative fixture coverage；
- 任何 negative fixture 失败都会阻断 `v0.9.1` release certification。

### Non-goals

- 不把 negative fixture 当成独立 Audit Agent 流程；
- 不扩大到无关历史版本。

## V091-007 v0.9.1 Release Certification

### Scope

证明 `v0.9.1` 修复后，`v0.9.x` 才能作为进入 `v1.0` planning 的稳定地基。

必须输出：

- release source Agent entry proof；
- Runtime Governance admission proof；
- deployment evidence semantic proof；
- Pack migration semantic split proof；
- project Pack registry proof；
- negative semantic fixture report；
- remaining risk / deferred list；
- v1.0 planning readiness decision。

### Acceptance

- `v0.9.1` release gate 覆盖 V091-001 到 V091-006；
- certification 明确 `v1PlanningReadiness = ready | blocked`；
- 如果 Governance 仍未接入 Runtime command path，必须 blocked；
- 如果 deployment evidence 仍只检查文件存在，必须 blocked；
- 如果 migration receipt 仍伪装成 authority mutation，必须 blocked；
- 如果 source archive 仍没有自洽 Agent entry，必须 blocked。

### Non-goals

- 不替代独立 Audit Agent 流程；
- 不跳过 `v0.9.1` 直接进入 `v1.0.0`；
- 不把 release certification 当成人工审计报告。

## Execution Order

建议执行顺序：

```text
V091-001
-> V091-002
-> V091-003
-> V091-004
-> V091-005
-> V091-006
-> V091-007
```

`V091-007` 必须最后执行。

## v1.0 Planning Gate

`v1.0.0` 建议定义为：

```text
Project OS Stable Core
```

但只有 `v0.9.1` 满足以下条件后，才能进入 `v1.0.0` planning：

- Runtime API 有 Governance 硬闸门；
- Pack / Projection / Evidence / Acceptance / Audit sidecar 的 authority 边界可验证；
- Deployment evidence 能证明语义一致性；
- Pack migration 不混淆 receipt 和真实 mutation；
- release source archive 的 Agent entry 自洽；
- release gate 有完整 negative semantic fixtures。
