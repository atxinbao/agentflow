# AgentFlow v1.2.6 Project-scoped Commercial Product Instance Hardening

更新日期：2026-07-07
执行者：Codex

## Release Baseline

`v1.2.6` 是 Project-scoped Commercial Product Instance Hardening release baseline。

这一版建立在 `v1.2.5` Published Release Certification and Registry-backed Commercial Runtime 之上，把商业产品输入从 source-level registry 继续推进到 project-scoped product instance：

```text
top-level certification kind
-> production / fixture registry separation
-> project-scoped commercial registry resolver
-> read model status semantics
-> registry-only commercial golden path
-> Desktop project-scoped commercial read model
-> Paid Report product instance contract
-> Paid Report preflight to Runtime proposal handoff
-> negative fixture isolation gate
-> v1.2.6 release certification
```

## Scope

`v1.2.6` 收口以下内容：

1. 在最终 release certification 顶层暴露 `certificationKind`。
2. 将生产 commercial registry 与 negative fixture registry 分离。
3. Runtime 支持从当前项目根解析 `products/commercial-runtime/**`。
4. 商业 read model 状态收敛为 `ready` / `partial` / `deferred` / `invalid` / `unavailable`。
5. registry-only golden path 不再依赖默认代码 fixture。
6. Desktop commercial surface 使用 projectRoot 读取项目级 Runtime read model。
7. Paid Report 有项目级 product instance contract。
8. Paid Report preflight 只生成 Runtime proposal handoff，不直接启动 run。
9. release gate 证明 negative fixtures 不能泄漏到生产 product surface。
10. 提供 v1.2.6 release certification。

## Certified Boundary

`v1.2.6` 认证的是 project-scoped commercial product instance，不是商业上线。

这一版确认：

- production registry 只包含可展示的产品实例；
- negative fixture registry 只供 Runtime tests / release gate 使用；
- 项目没有 commercial registry 时，Desktop / Runtime 必须显示 non-ready，不允许用源树 registry 伪造 ready；
- `partial` read model 允许 ready 产品可用，同时保留 deferred / blocked 产品的逐项状态；
- Paid Report instance 必须包含 required inputs、report definition、evidence policy、decision policy 和 delivery promise；
- allowed preflight 只能生成 Runtime proposal handoff，仍需要 Core Runtime admission；
- blocked / deferred / invalid preflight 不能创建 proposal；
- release certification 顶层必须能直接判断 candidate / published。

## Non-goals

`v1.2.6` 不包含：

- payment provider integration；
- checkout / billing implementation；
- customer account system；
- cloud multi-tenant launch；
- public commercial launch；
- paid report actual generation；
- new commercial SKU launch；
- Product console redesign。

## Primary Proof Index

| Proof | Path / URL | Purpose |
| --- | --- | --- |
| Certification kind fixture | `runtime/v126-certification-kind-negative-fixture.json` | proves top-level certificationKind is required |
| Registry separation | `runtime/v126-production-fixture-separation.json` | separates production registry from negative fixtures |
| Project resolver | `runtime/v126-project-commercial-registry-resolver.json` | proves projectRoot commercial registry resolution |
| Status semantics | `runtime/v126-commercial-read-model-status-semantics.json` | records aggregate and per-entry read model status semantics |
| Registry-only golden path | `runtime/v126-registry-only-commercial-golden-path.json` | primary proof uses registry inputs end to end |
| Desktop project read model | `runtime/v126-desktop-project-commercial-read-model.json` | Desktop consumes project-scoped Runtime read model |
| Paid Report instance | `runtime/v126-paid-report-product-instance-contract.json` | defines project-scoped Paid Report product instance |
| Preflight handoff | `runtime/v126-paid-report-preflight-runtime-proposal-handoff.json` | allowed preflight creates proposal handoff only |
| Negative fixture isolation | `runtime/v126-commercial-negative-fixture-isolation-gate.json` | fixture-only product ids cannot leak into production surface |
| v1.2.6 release certification | `runtime/v126-release-certification.json` | final release certification for v1.2.6 |

## Release Gate

`scripts/verify_release_gate.sh` must run the v1.2.6 proof chain after v1.2.5:

```text
run_v126_commercial_project_scope_proofs_gate
run_v126_release_certification_gate
```

## GitHub Traceability

This release closes GitHub issues `#945` through `#954`.
