# AgentFlow v1.0.1 Release Hardening and Operational Certification

日期：2026-06-26
执行者：Codex
状态：Release hardening closeout baseline

## Purpose

`v1.0.1` 是 `v1.0.0` 发布后的稳定补丁版本。

一句话：

```text
不扩功能，先把 v1 stable core 的发布证据、可复现性和运行可见性补硬。
```

`v1.0.0` 已经可以作为 Project OS Stable Core 成立。`v1.0.1` 不改变这个结论，也不回滚核心合同。它只处理发布审计后发现的硬化项：

- source Agent entry 仍有 v0.9.1 残留引用；
- release tag / release event 需要成为最终认证证据的一部分；
- release provenance 需要从“可追溯”升级为“有结构化证明”；
- clean-room `cargo test --workspace` 不应依赖人工 `cargo clean -p agentflow-pack`；
- public delivery audit sidecar 的失败语义需要更清楚；
- provider smoke 可以继续 optional，但必须有标准 proof / skip artifact；
- Message Bus no-go 需要写成正式 ADR，而不是口头延期；
- Software Dev Pack 需要从 stable contract 进入可使用样板；
- Runtime governance telemetry 需要收敛到可信项目事实源，而不是 request input。

## Reading Order

1. [AGENTFLOW_V1_0_1_RELEASE_HARDENING_TASKS_V1.md](AGENTFLOW_V1_0_1_RELEASE_HARDENING_TASKS_V1.md)
2. [../../../architecture/041-v100-stable-contract-baseline-v1.md](../../../architecture/041-v100-stable-contract-baseline-v1.md)
3. [../../../project/history/2026-06-current-baseline-history/versions/v1.0.0/README.md](../../../project/history/2026-06-current-baseline-history/versions/v1.0.0/README.md)
4. [../../../architecture/040-release-source-agent-entry-v1.md](../../../architecture/040-release-source-agent-entry-v1.md)
5. [../../../architecture/041-v100-stable-contract-baseline-v1.md](../../../architecture/041-v100-stable-contract-baseline-v1.md)
6. [../../../architecture/042-v100-runtime-api-sdk-freeze-v1.md](../../../architecture/042-v100-runtime-api-sdk-freeze-v1.md)
7. [../../../architecture/046-v100-evidence-acceptance-contract-freeze-v1.md](../../../architecture/046-v100-evidence-acceptance-contract-freeze-v1.md)
8. [../../../architecture/047-v100-executor-adapter-contract-freeze-v1.md](../../../architecture/047-v100-executor-adapter-contract-freeze-v1.md)
9. [../../../architecture/049-v100-software-dev-pack-stable-baseline-v1.md](../../../architecture/049-v100-software-dev-pack-stable-baseline-v1.md)
10. [../../../architecture/050-v100-release-certification-v1.md](../../../architecture/050-v100-release-certification-v1.md)

## Scope

`v1.0.1` 只做 release hardening 和 operational certification：

- 对齐 v1 source Agent entry；
- 强化 tag / release event certification；
- 增加 release provenance manifest；
- 修复 clean-room test reproducibility；
- 明确 public delivery audit sidecar policy；
- 标准化 provider smoke optional proof；
- 补 Message Bus no-go ADR；
- 补 Software Dev Pack usage baseline；
- 收敛 Runtime governance telemetry source；
- 输出 `v1.0.1` release certification。

## Closeout Proof

`v1.0.1` 的 closeout 由 release gate 生成以下 runtime artifacts：

- `runtime/release-provenance.json`
- `runtime/clean-room-test-proof.json`
- `runtime/audit-sidecar-policy.json`
- `runtime/provider-smoke-proof.json`
- `runtime/software-dev-pack-usage-baseline.json`
- `runtime/trusted-governance-telemetry.json`
- `runtime/v101-release-certification.json`

这些 artifact 是发布认证证据，不是新的产品能力。

## Non-goals

`v1.0.1` 不包含：

- 新行业 Pack；
- Pack marketplace；
- 默认中心化 Message Bus；
- 多租户云平台；
- 全新 UI 大改版；
- 把 Audit 放回主业务链；
- 把 provider smoke 改成所有发布的硬阻断；
- 把 GitHub issues 变成 AgentFlow authority；
- 改变 `v1.0.0` 已冻结的 stable contract；
- 新的 Runtime API 破坏性变更。

## Release Boundary

`v1.0.1` 发布必须回答：

```text
v1.0.0 的 stable core 是否可以在干净环境中复现、证明、解释和使用？
```

最低证明链：

```text
Source Agent Entry
-> Tag / Release Certification
-> Provenance Manifest
-> Clean-room Test Reproducibility
-> Audit Sidecar Policy
-> Provider Smoke Optional Proof
-> Trusted Governance Telemetry
-> Software Dev Pack Usage Baseline
-> v1.0.1 Release Certification
```

如果 `v1.0.1` 只是新增功能，没有修复发布证据和可复现性，它就不应该发布。
