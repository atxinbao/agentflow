# AgentFlow v1.3.0 Commercial Backend Stable Closure

更新日期：2026-07-10
执行者：Codex

## Release Intent

`v1.3.0` 是 Commercial Backend Stable Closure release line。

这一版不引入具体行业 SKU，也不做 public commercial launch。它只把 `v1.2.9`
已经交付的 generic Paid Report commercial backend contracts 收成稳定扩展基线：

```text
v1.2.9 release audit and certification semantics
-> commercial backend stable contract
-> paid report flow state machine
-> commercial authority boundary freeze
-> product SKU extension contract
-> provider / generator adapter boundary
-> payment provider adapter boundary
-> customer delivery backend contract
-> commercial end-to-end golden scenario
-> v1.3.0 release certification
```

## Baseline From v1.2.9

`v1.2.9` 已发布并认证完成：

| Fact | Value |
| --- | --- |
| Release | [AgentFlow v1.2.9](https://github.com/atxinbao/agentflow/releases/tag/v1.2.9) |
| Published at | `2026-07-09T17:05:59Z` |
| Source commit | `5b868ba9a8f37d0ad513d5999a7aa025354f668c` |
| Tag object id | `d03431719b3bb7eca1d141b152f3d4a385279a01` |
| Tag object kind | `tag` |
| Peeled commit sha | `5b868ba9a8f37d0ad513d5999a7aa025354f668c` |
| Release-event gate run | `29035832598` |
| Tag push gate run | `29035185062` |
| Main push gate run | `29035118100` |

The first v1.3.0 task repairs the remaining wording and payload semantics around
this baseline. The release facts authority is live GitHub release provenance;
synthetic `project-release-gate-e2e` sidecar facts can only be negative fixture
or local E2E evidence.

## Scope

`v1.3.0` closes:

1. v1.2.9 release audit and certification semantics repair.
2. Commercial backend stable contract.
3. Paid Report Flow state machine.
4. Commercial authority boundary freeze.
5. Product SKU extension contract.
6. Provider / Generator adapter boundary.
7. Payment provider adapter boundary.
8. Customer delivery backend contract.
9. Commercial end-to-end golden scenario.
10. v1.3.0 release certification.

## Non-goals

`v1.3.0` does not include:

- concrete Bazi / legal / contract / feng shui / study abroad / diligence /
  naming SKU;
- model-provider-specific final report generation;
- public commercial launch;
- cloud multi-tenant launch;
- full customer account system;
- production payment checkout / charge / refund execution;
- Product console redesign beyond read-model hooks required by this release.

## Primary Proof Index

| Proof | Path / URL | Purpose |
| --- | --- | --- |
| v1.2.9 release audit facts | `proofs/v130-001-v129-release-audit-facts.json` | records live v1.2.9 release, tag, source commit, and release-gate facts |
| v1.2.9 certification semantics repair | `runtime/v129-release-certification.json` | final certification now exposes live release authority, tag object kind, tag object id, and peeled commit sha |
| v1.3.0 commercial backend stable contract | `runtime/v130-commercial-backend-stable-contract.json` | machine-readable Product / Order / Entitlement / Run / Artifact / Evidence / Decision / Delivery / Feedback schema inventory |
| v1.3.0 paid report flow state machine | `runtime/v130-paid-report-flow-state-machine.json` | machine-readable Paid Report lifecycle states, positive transitions, and invalid transition failure fixtures |
| v1.3.0 commercial authority boundary | `runtime/v130-commercial-authority-boundary.json` | machine-readable authority writer map and negative fixtures for projection / customer view / download view / sidecar authority writes |
| v1.3.0 product SKU extension contract | `runtime/v130-product-sku-extension-contract.json` | machine-readable Product / Pack / SKU extension contract with synthetic SKU and missing-SKU negative fixture |

## GitHub Traceability

This release line starts with GitHub issue `#993` and ends with `#1002`.
