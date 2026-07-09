# AgentFlow v1.2.8 Paid Report Run and Delivery Artifact Closure

更新日期：2026-07-09
执行者：Codex

## Release Baseline

`v1.2.8` 是 Paid Report Run and Delivery Artifact Closure release baseline。

这一版建立在 `v1.2.7` Project-scoped Paid Report Runtime Handoff Closure 之上，把 Paid Report 从 admitted handoff / run contract 推进到完整可审计交付链：

```text
release provenance / tag policy repair
-> project-unique product instance identity
-> input snapshot / order intent contract
-> run execution receipt
-> report artifact schema
-> generation evidence pack
-> delivery decision gate
-> delivery package projection / download contract
-> feedback / repair request loop
-> v1.2.8 release certification
```

## Scope

`v1.2.8` 收口以下内容：

1. 修复 release proof 漂移，认证实际 `v1.2.8` tag / release facts。
2. 明确 lightweight / annotated tag policy，并记录 tag kind。
3. 让 Paid Report `productInstanceId` 绑定 project/workspace identity。
4. 定义 input snapshot 与 order intent，且 order intent 不等同 payment charge。
5. 定义 run execution receipt，记录 started / completed / blocked 事实。
6. 定义 report artifact schema 与 project-scoped storage boundary。
7. 定义 generation evidence pack，绑定 input、run、artifact 和 generation receipt。
8. 定义 delivery decision gate，支持 accepted / needs-fix / rejected / deferred / blocked。
9. 定义 delivery package projection 与 download/display contract。
10. 定义 feedback / repair request loop，不直接修改已交付 artifact。

## Certified Boundary

`v1.2.8` 认证的是 generic Paid Report Flow 的 runtime delivery closure，不是商业上线。

这一版确认：

- Software Dev 仍是 Managed Project Flow Reference App；
- Paid Report 仍是 generic flow，不是具体行业 SKU；
- Core Runtime 只认识 Product Instance / Run / Artifact / Evidence / Decision / Delivery / Feedback；
- Concrete report domain data 只能来自 Product / Pack；
- projection 不能写 report、evidence、decision 或 delivery authority；
- delivery-ready 必须依赖 accepted decision；
- feedback / repair 只能创建受控 follow-up route，不能修改已交付 artifact。

## Non-goals

`v1.2.8` 不包含：

- payment provider integration；
- checkout / billing implementation；
- customer account system；
- concrete paid-report industry SKU；
- model/provider-specific final report generation；
- public commercial launch；
- cloud multi-tenant launch；
- Product console redesign。

## Primary Proof Index

| Proof | Path / URL | Purpose |
| --- | --- | --- |
| Release provenance and tag policy | `runtime/v128-release-provenance-tag-policy-repair.json` | proves actual tag facts and stale fixture rejection |
| Project-unique product instance | `runtime/v128-project-unique-product-instance-identity.json` | proves project/workspace scoped identity |
| Input snapshot and order intent | `runtime/v128-paid-report-input-snapshot-order-intent-contract.json` | proves input/order intent boundary |
| Run execution receipt | `runtime/v128-paid-report-run-execution-receipt.json` | proves blocked and successful run receipt cases |
| Report artifact schema | `runtime/v128-report-artifact-schema-storage-boundary.json` | proves artifact schema and storage boundary |
| Generation evidence | `runtime/v128-report-generation-evidence-capture.json` | proves evidence pack completeness and missing-evidence state |
| Decision gate | `runtime/v128-decision-gate-report-delivery.json` | proves accepted / non-accepted decision semantics |
| Delivery package | `runtime/v128-delivery-package-projection-download-contract.json` | proves read-only delivery package projection |
| Feedback loop | `runtime/v128-feedback-repair-request-loop.json` | proves feedback / repair route without artifact mutation |
| v1.2.8 release certification | `runtime/v128-release-certification.json` | final release certification for v1.2.8 |

## Release Gate

`scripts/verify_release_gate.sh` must run the v1.2.8 proof chain after v1.2.7:

```text
run_v128_paid_report_run_delivery_artifact_closure_gate
run_v128_release_certification_gate
```

## GitHub Traceability

This release closes GitHub issues `#967` through `#976`.
