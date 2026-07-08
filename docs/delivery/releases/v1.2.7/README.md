# AgentFlow v1.2.7 Paid Report Runtime Handoff Closure

更新日期：2026-07-08
执行者：Codex

## Release Baseline

`v1.2.7` 是 Project-scoped Paid Report Runtime Handoff Closure release baseline。

这一版建立在 `v1.2.6` Project-scoped Commercial Product Instance Hardening 之上，把 Paid Report 从“可生成 proposal handoff”继续推进到完整 Runtime handoff 闭环：

```text
release planning alignment
-> product flow source boundary
-> project-scoped Paid Report instance resolver
-> project-scoped Paid Report preflight / handoff API
-> Desktop projectRoot bridge
-> golden path source semantics
-> Runtime proposal admission receipt
-> Paid Report run contract boundary
-> evidence / decision / delivery projection contract
-> v1.2.7 release certification
```

## Scope

`v1.2.7` 收口以下内容：

1. 将下一版目标明确为 Project-scoped Paid Report Runtime Handoff Closure。
2. 明确 Software Dev 仍是 Managed Project Flow Reference App。
3. 明确 Paid Report 是 generic backend handoff，不是具体商业 SKU。
4. Runtime 从 project root 解析 Paid Report product instance。
5. Paid Report preflight 生成 Runtime proposal handoff，不直接启动 run。
6. Desktop preflight 桥接必须使用 active project root。
7. Golden path 证明 source semantics 不回退到 source-level fallback。
8. Runtime proposal 进入 admission receipt。
9. Paid Report run contract 只接受 generic Paid Report Flow。
10. Evidence / Decision / Delivery 只作为 projection contract，不写 authority。

## Certified Boundary

`v1.2.7` 认证的是项目级 Paid Report Runtime handoff，不是商业上线。

这一版确认：

- `products/commercial-runtime/**` 仍是 project-scoped commercial source；
- Core Runtime 只认识 generic Product Instance / Runtime Proposal / Evidence / Decision / Delivery；
- 具体 report SKU 不能成为 Core authority；
- allowed preflight 只能生成 Runtime proposal handoff；
- handoff 必须先通过 Runtime admission receipt；
- run contract 必须绑定 admission receipt、input refs、report definition、evidence policy 和 decision policy；
- delivery projection 只能读 evidence / decision 状态，不能写 authority；
- Desktop 命令不能忽略 projectRoot。

## Non-goals

`v1.2.7` 不包含：

- payment provider integration；
- checkout / billing implementation；
- customer account system；
- cloud multi-tenant launch；
- public commercial launch；
- actual paid report generation；
- concrete paid report SKU launch；
- Product console redesign。

## Primary Proof Index

| Proof | Path / URL | Purpose |
| --- | --- | --- |
| Planning alignment | `runtime/v127-next-release-planning-alignment.json` | proves v1.2.7 scope and previous baseline |
| Source boundary | `runtime/v127-product-flow-source-boundary.json` | proves Software Dev / Paid Report / Core source boundaries |
| Project instance resolver | `runtime/v127-project-paid-report-instance-resolver.json` | proves projectRoot resolves Paid Report instance |
| Preflight handoff API | `runtime/v127-project-paid-report-preflight-handoff-api.json` | proves preflight creates handoff only |
| Desktop projectRoot bridge | `runtime/v127-desktop-paid-report-project-root-bridge.json` | proves Desktop bridge uses projectRoot |
| Golden source semantics | `runtime/v127-golden-path-source-semantics.json` | proves project-scoped source semantics |
| Admission receipt | `runtime/v127-runtime-proposal-admission-receipt.json` | proves proposal must enter Runtime admission |
| Run contract boundary | `runtime/v127-paid-report-run-contract-boundary.json` | proves generic Paid Report run contract |
| Delivery projection | `runtime/v127-paid-report-evidence-decision-delivery-projection-contract.json` | proves evidence / decision / delivery projection contract |
| v1.2.7 release certification | `runtime/v127-release-certification.json` | final release certification for v1.2.7 |

## Release Gate

`scripts/verify_release_gate.sh` must run the v1.2.7 proof chain after v1.2.6:

```text
run_v127_paid_report_handoff_closure_gate
run_v127_release_certification_gate
```

## GitHub Traceability

This release closes GitHub issues `#956` through `#965`.
