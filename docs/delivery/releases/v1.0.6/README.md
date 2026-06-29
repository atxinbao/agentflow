# AgentFlow v1.0.6 Core Evidence Kernel

更新日期：2026-06-29
执行者：Codex

## Status

`v1.0.6` 是 `v1.0.5` Core Runtime Kernel 之后的 Core Evidence Kernel release baseline。

本版本不启动 Decision Kernel，不把 Software Dev Reference App 的行业词汇写入 Core Evidence authority。它把执行结果、外部证明、缺失证据和投影读模型收束成行业无关的 Evidence Kernel。

Release notes 必须明确：Software Dev 的 diff、test log、build log、PR link 和 release record 只是 Reference App evidence mapping，不是 Core Evidence authority。

## Core Evidence Boundary

Core Evidence Kernel 只定义行业无关的证据链：

```text
Evidence Pack Schema
-> Evidence Source Type Registry
-> Capture Receipt
-> Authority Trace Link
-> Completeness Policy
-> Missing Evidence Handling
-> External Proof Provenance
-> Projection Read Model
-> Release Certification
```

Software Dev 证据只能作为 Reference App mapping 进入 Evidence Kernel：

```text
diff / test log / build log / PR link / release note
= Software Dev Reference App evidence mapping
!= Core Evidence authority
```

## Scope

`v1.0.6` 收口以下内容：

1. Core Evidence Pack Schema。
2. Core Evidence Source Type Registry。
3. Core Evidence Capture Receipts。
4. Core Evidence Authority Trace Links。
5. Core Evidence Completeness Policy。
6. Core Missing Evidence Handling。
7. Core External Proof Provenance。
8. Software Dev Reference Evidence Mapping。
9. Evidence Projection Read Model。
10. v1.0.6 release certification artifact。

## Closeout Artifacts

Release gate 必须生成：

- `runtime/core-evidence-pack-schema.json`
- `runtime/core-evidence-source-type-registry.json`
- `runtime/core-evidence-capture-receipts.json`
- `runtime/core-evidence-authority-trace-links.json`
- `runtime/core-evidence-completeness-policy.json`
- `runtime/core-missing-evidence-handling.json`
- `runtime/core-external-proof-provenance.json`
- `runtime/software-dev-reference-evidence-mapping.json`
- `runtime/evidence-projection-read-model.json`
- `runtime/v106-release-certification.json`

## Public Records

- [AGENTFLOW_V1_0_6_CORE_EVIDENCE_KERNEL_TASKS_V1.md](AGENTFLOW_V1_0_6_CORE_EVIDENCE_KERNEL_TASKS_V1.md)
- [../../../project/roadmap.md](../../../project/roadmap.md)
- [../../../architecture/058-core-evidence-decision-reference-model-v1.md](../../../architecture/058-core-evidence-decision-reference-model-v1.md)
- [../../../architecture/060-core-evidence-pack-schema-v1.md](../../../architecture/060-core-evidence-pack-schema-v1.md)
- [../../../architecture/061-core-evidence-source-type-registry-v1.md](../../../architecture/061-core-evidence-source-type-registry-v1.md)
- [../../../architecture/062-core-evidence-capture-receipts-v1.md](../../../architecture/062-core-evidence-capture-receipts-v1.md)
- [../../../architecture/063-core-evidence-authority-trace-links-v1.md](../../../architecture/063-core-evidence-authority-trace-links-v1.md)
- [../../../architecture/064-core-evidence-completeness-policy-v1.md](../../../architecture/064-core-evidence-completeness-policy-v1.md)
- [../../../architecture/065-core-missing-evidence-handling-v1.md](../../../architecture/065-core-missing-evidence-handling-v1.md)
- [../../../architecture/066-core-external-proof-provenance-v1.md](../../../architecture/066-core-external-proof-provenance-v1.md)
- [../../../architecture/067-software-dev-reference-evidence-mapping-v1.md](../../../architecture/067-software-dev-reference-evidence-mapping-v1.md)
- [../../../architecture/068-evidence-projection-read-model-v1.md](../../../architecture/068-evidence-projection-read-model-v1.md)

## Non-goals

- 不启动 Decision Kernel；
- 不实现 Software Dev Product UI；
- 不把 GitHub issue 当成 AgentFlow authority；
- 不把 provider CLI session 当成项目事实源；
- 不把 Audit 移入主业务链；
- 不认证 full Software Dev Reference App；
- 不认证 Decision Kernel 或 Projection Kernel 完整性。
