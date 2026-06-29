# AgentFlow v1.0.7 Decision Kernel

更新日期：2026-06-29
执行者：Codex

## Status

`v1.0.7` 是 `v1.0.6` Core Evidence Kernel 之后的 Core Decision Kernel release baseline。

本版本把 Evidence Kernel 的 proof / completeness / projection 结果转成可解释、可阻断、可交付的 Decision Kernel：

```text
v1.0.6 Evidence Kernel
-> Release Provenance Tag Policy
-> Decision Inputs
-> Decision Outcomes
-> Evidence-to-Decision Gate
-> Completion Commit Authority
-> Delivery Readiness
-> Decision Projection Read Model
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
- [../../../architecture/072-core-decision-outcome-transition-semantics-v1.md](../../../architecture/072-core-decision-outcome-transition-semantics-v1.md)
- [../../../architecture/073-core-decision-failure-reason-remediation-v1.md](../../../architecture/073-core-decision-failure-reason-remediation-v1.md)
- [../../../architecture/074-core-evidence-to-decision-gate-v1.md](../../../architecture/074-core-evidence-to-decision-gate-v1.md)
- [../../../architecture/075-core-completion-commit-authority-v1.md](../../../architecture/075-core-completion-commit-authority-v1.md)
- [../../../architecture/076-core-delivery-readiness-audit-trigger-v1.md](../../../architecture/076-core-delivery-readiness-audit-trigger-v1.md)
- [../../../architecture/077-core-decision-projection-read-model-v1.md](../../../architecture/077-core-decision-projection-read-model-v1.md)
- [../v1.0.6/README.md](../v1.0.6/README.md)

## Release Gate Artifacts

`v1.0.7` release gate must produce:

```text
runtime/v107-release-provenance-handoff.json
runtime/core-decision-model-contract.json
runtime/core-decision-input-binding.json
runtime/core-decision-outcome-transitions.json
runtime/core-decision-failure-reason-remediation.json
runtime/core-evidence-to-decision-gate.json
runtime/core-completion-commit-authority.json
runtime/core-delivery-readiness-audit-trigger.json
runtime/core-decision-projection-read-model.json
runtime/v107-release-certification.json
```

## Known Boundaries

- `v1.0.7` certifies Decision Projection read model only; full Projection Kernel rebuild remains `v1.0.8` scope.
- Audit remains an optional sidecar evaluation and is not part of the default Done chain.
- Software Dev remains Reference App evidence / decision mapping, not Core authority.

## Non-goals

- 不实现 Software Dev Product UI；
- 不把 GitHub issue 当成 AgentFlow authority；
- 不把 provider CLI session 当成项目事实源；
- 不把 Audit 移入默认主业务链；
- 不认证 Projection Kernel 完整性。

## Next Version

`v1.0.8` should certify Projection Kernel rebuild / replay over stable Core facts without weakening Decision authority.
