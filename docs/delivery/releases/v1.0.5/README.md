# AgentFlow v1.0.5 Core Runtime Kernel

更新日期：2026-06-28
执行者：Codex

## Status

`v1.0.5` 是 `v1.0.4` Core Ontology Kernel 之后的 Core Runtime Kernel planning baseline。

本版本不启动 `v1.1` 产品功能，不把 Software Dev Reference App 的行业词汇写入 Core Runtime authority。它的目标是把 `v1.0.4` 定义出来的 Object、Link、Action、State、Skill、Evidence、Decision 和 Registry 合同接入 Runtime command、admission、Action Proposal、Arbitration、executor closeout 和 task / run state writeback。

## Core Runtime Boundary

Core Runtime Kernel 只定义行业无关的执行管线：

```text
Runtime Command
-> Runtime Admission
-> Action Proposal
-> Arbitration
-> Executor Adapter Closeout
-> Completion Commit / State Writeback
-> Release Certification
```

Software Dev 词汇只能作为 Reference App mapping 进入 Runtime：

```text
Requirement / Spec / Issue / Run / PR / Release
= Software Dev Reference App mapping
!= Core Runtime command authority
```

## Scope

`v1.0.5` 收口以下内容：

1. Core Runtime Kernel Contract。
2. Core contract + App Pack command mapping。
3. Runtime Admission using Core Skill Registry。
4. Core Action Proposal materialization。
5. Arbitration using Core Action / State Semantics。
6. File-backed Ontology Registry runtime loader。
7. Executor Adapter Closeout integration。
8. Task Run State Writeback authority。
9. Negative runtime fixtures and Software Dev Reference App mapping。
10. v1.0.5 release certification artifact。

## Closeout Artifacts

Release gate 必须生成：

- `runtime/core-runtime-kernel.json`
- `runtime/core-runtime-admission.json`
- `runtime/core-runtime-arbitration.json`
- `runtime/core-runtime-negative-fixtures.json`
- `runtime/v105-release-certification.json`

## Public Records

- [AGENTFLOW_V1_0_5_CORE_RUNTIME_KERNEL_TASKS_V1.md](AGENTFLOW_V1_0_5_CORE_RUNTIME_KERNEL_TASKS_V1.md)
- [../../../project/roadmap.md](../../../project/roadmap.md)
- [../../../architecture/054-core-ontology-kernel-contract-v1.md](../../../architecture/054-core-ontology-kernel-contract-v1.md)
- [../../../architecture/055-core-object-link-schema-v1.md](../../../architecture/055-core-object-link-schema-v1.md)
- [../../../architecture/056-core-action-state-semantics-v1.md](../../../architecture/056-core-action-state-semantics-v1.md)
- [../../../architecture/057-core-skill-registry-action-authorization-v1.md](../../../architecture/057-core-skill-registry-action-authorization-v1.md)
- [../../../architecture/058-core-evidence-decision-reference-model-v1.md](../../../architecture/058-core-evidence-decision-reference-model-v1.md)
- [../../../architecture/059-core-file-backed-ontology-registry-projection-v1.md](../../../architecture/059-core-file-backed-ontology-registry-projection-v1.md)

## Non-goals

- 不启动 `v1.1` 产品功能；
- 不实现 Software Dev Product UI；
- 不引入 Message Bus；
- 不把 GitHub issue 当成 AgentFlow authority；
- 不把 provider CLI session 当成项目事实源；
- 不把 Audit 移入主业务链；
- 不认证 Evidence Kernel、Decision Kernel 或 Projection Kernel 完整性。
