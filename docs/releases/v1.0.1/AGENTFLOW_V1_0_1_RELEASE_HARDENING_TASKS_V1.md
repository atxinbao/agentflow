# AgentFlow v1.0.1 Release Hardening Tasks V1

日期：2026-06-26
执行者：Codex

## Goal

`v1.0.1` 聚焦 `v1.0.0` 发布审计后的稳定补丁。

主线是：

```text
Source Agent Entry
-> Release Tag Certification
-> Provenance Manifest
-> Clean-room Test Reproducibility
-> Audit Sidecar Policy
-> Provider Smoke Proof
-> Message Bus ADR
-> Software Dev Pack Usage
-> Trusted Governance Telemetry
-> v1.0.1 Release Certification
```

本版本要回答的是：

```text
v1 stable core 能不能被干净复现、被明确证明、被真实使用，并且不扩大 v1.0.0 的稳定合同？
```

## Audit Findings

`v1.0.0` 可以作为 Project OS Stable Core 发布成立，不建议回滚。

发布后审计发现的主要硬化点是：

- `AGENTS.md` 和 source-agent-entry 相关文档仍残留 `v0.9.1` 稳定入口引用；
- release gate 已覆盖 PR、tag、release event，但 release tag event 需要成为最终权威认证链的一部分；
- release 证据可追溯，但缺少结构化 provenance manifest；
- `cargo test --workspace` 曾在 release-gate 后命中 stale `agentflow-pack` fixture pointer，需要清理后复跑；
- Audit sidecar 保持非阻断是正确设计，但 public delivery audit failed 的发布语义需要 policy 化；
- provider smoke 仍可 optional，但 optional 不能等于无证据；
- Cross-process Message Bus 仍是 no-go / deferred，需要 ADR 固化重新评估条件；
- Software Dev Pack 已是 stable Pack，但还需要真实使用样板；
- Runtime command governance 的 provider smoke / capability telemetry 需要从可信项目事实源读取，而不是让 request input 成为事实来源。

## Product Principle

`v1.0.1` 不扩大产品面。

正确方向是：

```text
修发布证据。
修可复现性。
修运行可见性。
修 authority 边界表达。
保留 v1 stable core，不重开大架构。
```

## Issues

| Issue | Title | Priority | Dependency | Status |
| --- | --- | --- | --- | --- |
| `V101-001` | Source Agent Entry v1 Alignment | P0 | v1.0.0 release audit | done |
| `V101-002` | Release Tag Event Certification Gate | P0 | V101-001 | done |
| `V101-003` | Release Provenance Manifest | P0 | V101-002 | done |
| `V101-004` | Clean-room Cargo Test Reproducibility | P0 | none | done |
| `V101-005` | Public Delivery Audit Sidecar Policy | P1 | V101-002, V101-003 | done |
| `V101-006` | Provider Smoke Optional Proof Artifact | P1 | V101-002 | done |
| `V101-007` | Message Bus No-go ADR | P1 | none | done |
| `V101-008` | Software Dev Pack Usage Baseline | P1 | V101-003, V101-005, V101-006 | done |
| `V101-009` | Trusted Governance Telemetry Source | P0 | V101-006 | done |
| `V101-010` | v1.0.1 Release Certification | P0 | V101-001, V101-002, V101-003, V101-004, V101-005, V101-006, V101-007, V101-008, V101-009 | done |

## V101-001 Source Agent Entry v1 Alignment

### Scope

把 release source 中的 Agent entry 完整对齐到当前 v1 release baseline。

必须处理：

- 根目录 `AGENTS.md` 不再指向 `docs/v0.9.1/README.md` 作为当前稳定入口；
- `docs/core/architecture/040-release-source-agent-entry-v1.md` 更新 current stabilization 文档口径；
- release gate 的 source-agent-entry 检查覆盖当前 tracked docs、`docs/releases/v1.0.1/README.md` 和 v1 release certification 文档；
- 保持 local runtime facts 不进入 release source archive；
- 证明 source archive 能给 Agent 一个自洽入口。

### Acceptance

- release source checkout 中 `AGENTS.md` 指向 v1 当前稳定入口；
- source-agent-entry gate 不再只以 `v0.9.1` 为 current stabilization 文档；
- release gate 输出 source entry evidence；
- 不把 `.agentflow/tasks/**`、`.agentflow/events/**`、`.agentflow/tmp/**` 等本地运行事实提交到源码。

### Non-goals

- 不恢复 retired AgentFlow fact paths；
- 不把 runtime local state 变成 git tracked source。

## V101-002 Release Tag Event Certification Gate

### Scope

让正式 tag / release event 成为 v1 补丁发布的最终认证链。

必须处理：

- release gate 能区分 PR gate、main push gate、tag gate、release event gate；
- `REQUIRE_PUBLISHED_RELEASE_FACTS=1` 或等价策略能要求 release facts 存在；
- certification artifact 绑定 release tag commit，而不是只依赖 PR merge context；
- tag / release event 缺少正式发布事实时不能给出 clean release certification；
- GitHub release URL、tag name、commit sha、run id 进入 release evidence。

### Acceptance

- tag / release event gate 产出独立 certification evidence；
- release tag commit 与 source commit 不一致时 gate 失败；
- release facts 缺失时 strict release certification 失败；
- release certification 能说明它认证的是 PR、main、tag 还是 release event。

### Non-goals

- 不把 GitHub Release 变成 AgentFlow authority；
- 不依赖人工复制 release 页面作为唯一证据。

## V101-003 Release Provenance Manifest

### Scope

新增结构化 release provenance manifest。

必须处理：

- release version；
- tag name；
- tag object id；
- source commit sha；
- GitHub release URL；
- release-gate run ids；
- artifact manifest path；
- certification artifact path；
- release note reference；
- tag signature status 或 unsigned reason；
- reproducibility command summary。

### Acceptance

- release gate 生成 provenance manifest；
- manifest 能被 `jq` 或等价结构化检查读取；
- tag、commit、release URL、artifact hash 不一致时 provenance check 失败；
- 如果 tag 未签名，manifest 必须显式记录，而不是默默通过。

### Non-goals

- 不强制 v1.0.1 必须使用 GPG signed tag；
- 不把 provenance manifest 当作 Event Store authority。

## V101-004 Clean-room Cargo Test Reproducibility

### Scope

修复 release-gate 后 stale `agentflow-pack` fixture pointer 导致的测试复现问题。

必须处理：

- `cargo test --workspace` 在 clean-room checkout 中一次性通过；
- release gate 不能向后续 cargo test 泄漏临时 workspace fixture path；
- `agentflow-pack` fixture registry test 不依赖陈旧 target 缓存；
- 如必须清理，清理逻辑应自动化并写入验证说明；
- release documentation 不应要求人工记住 `cargo clean -p agentflow-pack`。

### Acceptance

- 从干净 checkout 执行 `cargo test --workspace` 通过；
- 先跑 release gate 再跑 `cargo test --workspace` 也通过；
- 不需要人工执行 `cargo clean -p agentflow-pack` 才能复现；
- release certification 记录 clean-room test proof。

### Non-goals

- 不缩小测试覆盖来绕过失败；
- 不把失败测试标记 ignore 作为修复。

## V101-005 Public Delivery Audit Sidecar Policy

### Scope

保留 Audit sidecar 独立性，同时明确 public delivery audit failed 的发布语义。

必须处理：

- Audit sidecar failed 默认不阻断主 release gate 的原则继续保留；
- formal release / tag 场景下，public delivery audit failed 至少产出 warning with required acknowledgement；
- strict release mode 可以选择把 public delivery audit failed 变成 blocker；
- certification artifact 要区分 `releaseGateStatus` 与 `auditSidecarStatus`；
- release notes / public delivery 缺口应有可追踪 remediation hint。

### Acceptance

- release certification 不再让 audit sidecar failed 与 release gate failed 混淆；
- strict mode 下 public delivery audit failed 可被配置为阻断；
- non-strict mode 下 audit sidecar failed 必须有 acknowledgement evidence；
- Audit 仍不回到主业务链，不自动创建 audit request。

### Non-goals

- 不把 Audit 变回默认主链 gate；
- 不要求所有补丁发布必须完成人工审计。

## V101-006 Provider Smoke Optional Proof Artifact

### Scope

把 provider smoke 从“可跳过”升级为“可跳过但必须可解释”。

必须处理：

- provider smoke optional proof artifact；
- skip reason；
- provider name；
- expected capability；
- last known provider status；
- next verification path；
- manual / nightly smoke 入口；
- release certification 中的 deferred item 说明。

### Acceptance

- provider smoke skipped 时生成结构化 skip artifact；
- provider smoke failed 时 capability availability 不应显示 ready；
- release certification 显示 provider smoke 是 passed、failed、skipped 还是 not configured；
- provider smoke optional 不能让 Runtime governance 误认为 provider 已 ready。

### Non-goals

- 不把 live provider smoke 变成所有本地发布的硬阻断；
- 不强制开发者必须配置 Codex / Claude Code provider credentials。

## V101-007 Message Bus No-go ADR

### Scope

把 Cross-process Message Bus 的 no-go / deferred 决策写成 ADR。

必须处理：

- 当前为什么不默认启用 Message Bus；
- 哪些场景 local runtime / Event Store lock 已足够；
- 哪些场景必须重新评估 cross-process scheduling；
- no-go 决策的有效期；
- v1.1 / v1.2 是否进入实验的判断条件；
- Message Bus 不能成为 authority 的原则。

### Acceptance

- 新增 ADR 文档；
- release certification 能引用 ADR；
- deferred item 不再只是裸字符串；
- 后续实现 Message Bus 前必须先满足 ADR 中的 go criteria。

### Non-goals

- 不在 `v1.0.1` 实现中心化 Message Bus；
- 不引入跨进程调度作为默认 runtime path。

## V101-008 Software Dev Pack Usage Baseline

### Scope

把 Software Dev Pack 从 stable contract 补成可使用样板。

必须处理：

- 示例项目结构；
- command path 示例；
- Evidence Pack 示例；
- Acceptance Decision 示例；
- Replay / Migration 示例；
- 失败路径示例；
- Audit sidecar 示例；
- executor handoff 示例；
- public delivery 示例。

### Acceptance

- 文档能让用户理解 Software Dev Pack 如何驱动一次真实开发任务；
- 示例不把 GitHub issues 当 authority；
- 示例不把 Audit 放回主链；
- 示例能映射到 Runtime API、Evidence、Acceptance 和 Projection；
- release certification 能引用 usage baseline。

### Non-goals

- 不做新的行业 Pack；
- 不把 Software Dev Pack 变成唯一行业模型。

## V101-009 Trusted Governance Telemetry Source

### Scope

收敛 Runtime command governance 的 provider smoke / capability telemetry 来源。

必须处理：

- raw Runtime command 不应把 request input 中的 provider smoke / capability telemetry 当 authority；
- Runtime admission 应读取项目级 trusted capability registry 或 provider smoke artifact；
- request input 可以引用 evidence path，但不能直接伪造 provider ready 状态；
- Pack command 和 raw runtime command 的 telemetry source 规则保持一致；
- governance decision fact 要说明 telemetry source。

### Acceptance

- provider smoke failed / skipped 时 Runtime admission 不能被 request input 伪造成 allowed；
- trusted registry 缺失时应输出 deferred 或明确 failed reason；
- release gate 增加伪 telemetry input 的 negative fixture；
- governance admission trace 包含 telemetry source path 或 source kind。

### Non-goals

- 不做多租户权限系统；
- 不把 provider runtime session 变成 AgentFlow authority。

## V101-010 v1.0.1 Release Certification

### Scope

证明 `v1.0.1` 补丁版修复完成。

必须处理：

- V101 issue coverage；
- release tag event certification；
- release provenance manifest；
- clean-room test proof；
- audit sidecar policy evidence；
- provider smoke optional proof；
- Message Bus ADR reference；
- Software Dev Pack usage baseline reference；
- trusted governance telemetry negative fixture；
- remaining risks；
- support boundary。

### Acceptance

- release gate 输出 `v101ReleaseCertificationStatus = passed` 或等价字段；
- 所有 V101 coverage 项为 passed；
- remaining risks 不包含阻断项；
- v1 stable core 仍保持 ready；
- v1.0.1 release notes 能说明它是 hardening patch，不是功能扩张。

### Non-goals

- 不承诺 v1.1 功能；
- 不把 deferred items 伪装成已完成。
