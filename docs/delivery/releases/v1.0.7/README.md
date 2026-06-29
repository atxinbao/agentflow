# AgentFlow v1.0.7 Decision Kernel

更新日期：2026-06-29
执行者：Codex

## Status

`v1.0.7` 是 `v1.0.6` Core Evidence Kernel 之后的 Decision Kernel release line。

第一步不是直接实现 Decision outcome，而是先修复 release provenance / tag policy 和 Evidence Kernel handoff：

```text
v1.0.6 Evidence Kernel
-> Release Provenance Tag Policy
-> Decision Kernel inputs
```

## Scope

`v1.0.7` 收口以下内容：

1. Release Provenance Tag Policy and Evidence Handoff。
2. Core Decision Model Contract。
3. Decision Input Binding。
4. Decision Outcomes and State Transition Semantics。
5. Failure Reason and Remediation Contract。
6. Evidence-to-Decision Gate。
7. Completion Commit Authority Boundary。
8. Delivery Readiness and Optional Audit Trigger Evaluation。
9. Decision Projection Read Model and Negative Fixtures。
10. v1.0.7 Release Certification。

## Public Records

- [AGENTFLOW_V1_0_7_DECISION_KERNEL_TASKS_V1.md](AGENTFLOW_V1_0_7_DECISION_KERNEL_TASKS_V1.md)
- [../../../architecture/058-core-evidence-decision-reference-model-v1.md](../../../architecture/058-core-evidence-decision-reference-model-v1.md)
- [../../../architecture/069-release-provenance-tag-policy-v1.md](../../../architecture/069-release-provenance-tag-policy-v1.md)
- [../../../architecture/070-core-decision-model-contract-v1.md](../../../architecture/070-core-decision-model-contract-v1.md)
- [../../../architecture/071-core-decision-input-binding-v1.md](../../../architecture/071-core-decision-input-binding-v1.md)
- [../v1.0.6/README.md](../v1.0.6/README.md)

## Non-goals

- 不实现 Software Dev Product UI；
- 不把 GitHub issue 当成 AgentFlow authority；
- 不把 provider CLI session 当成项目事实源；
- 不把 Audit 移入默认主业务链；
- 不认证 Projection Kernel 完整性。
