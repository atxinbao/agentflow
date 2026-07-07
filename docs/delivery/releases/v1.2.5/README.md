# AgentFlow v1.2.5 Published Release Certification and Registry-backed Commercial Runtime

更新日期：2026-07-07
执行者：Codex

## Release Baseline

`v1.2.5` 是 Published Release Certification and Registry-backed Commercial Runtime release baseline。

这一版建立在 `v1.2.4` Commercial Runtime Read Model and Closeout Distinction 之上，把发布证明继续拆成 candidate / published 两层，并把商业产品读模型从默认代码输入推进到产品注册表和 entitlement source：

```text
release publication state
-> candidate / published certification split
-> waiver contract consistency
-> product registry-backed commercial read model
-> entitlement source fixture
-> paid report product definition fixture
-> Desktop runtime-only commercial surface guard
-> registry-backed commercial golden path
-> release-event artifact certification
-> v1.2.5 release certification
```

## Scope

`v1.2.5` 收口以下内容：

1. 建立 release publication state artifact，区分 candidate、tagged、released 和 published。
2. 区分 candidate certification 与 published certification，避免 PR / 本地门禁伪装成已发布。
3. 收紧 waiver contract：存在 waiver 时必须包含 reason、observed provider state、timestamp 和 source commit。
4. 将 Commercial Product read model 绑定到 `products/commercial-runtime/**` 注册表和 entitlement source。
5. 增加本地 entitlement / license fixture，覆盖 active、trial、deferred、missing、invalid 等状态。
6. 将 Paid Report 定义为一等产品 fixture，明确 required inputs、evidence requirements 和 decision requirements。
7. 认证 Desktop Commercial Surface 只能消费 Runtime/Tauri read model，Browser Preview fallback 必须显式标记。
8. 认证 registry-backed commercial golden path。
9. 认证 release-event artifact 必须带 tag、GitHub Release URL、workflow run、source commit、artifact manifest 和 milestone facts。
10. 提供 v1.2.5 release certification。

## Certified Boundary

`v1.2.5` 认证的是发布事实分层和商业 Runtime 输入源，不是商业上线。

这一版确认：

- candidate certification 只能证明源码和 release gate 候选状态；
- published certification 必须证明 tag、GitHub Release、release-event workflow 和 closed milestone；
- open milestone with zero open issues 仍不能被认证为 published；
- waiver 不能只有一句 reason，必须带观察到的 provider 状态、时间和 source commit；
- Commercial Product read model 的 production source 是 `products/commercial-runtime/**`；
- default in-code inputs 只能作为 fallback / tests / Browser Preview 使用；
- entitlement source 是 Runtime read model 的输入之一；
- Paid Report product definition 必须包含 required inputs、evidence requirements 和 decision requirements；
- Desktop production surface 不写 authority，只读 Runtime/Tauri read model；
- release-event artifact certification 必须自包含发布证明。

## Non-goals

`v1.2.5` 不包含：

- payment provider integration；
- checkout / billing implementation；
- customer account system；
- cloud multi-tenant launch；
- public commercial launch；
- paid report actual generation；
- new commercial SKU launch；
- new Product console feature。

## Primary Proof Index

| Proof | Path / URL | Purpose |
| --- | --- | --- |
| Release publication state | `runtime/v125-release-publication-state.json` | separates candidate / tagged / released / published states |
| Candidate / published split | `runtime/v125-candidate-published-certification-split.json` | proves candidate cannot satisfy published certification |
| Waiver contract consistency | `runtime/v125-waiver-contract-consistency.json` | validates waiver absent / complete / invalid cases |
| Product registry commercial read model | `runtime/v125-product-registry-commercial-read-model.json` | Registry-backed commercial Runtime read model |
| Entitlement source fixture | `runtime/v125-entitlement-source-fixture.json` | local entitlement source coverage |
| Paid Report product definition | `runtime/v125-paid-report-product-definition.json` | paid report required inputs / evidence / decision fixture |
| Desktop runtime-only guard | `runtime/v125-desktop-runtime-only-commercial-surface.json` | Desktop production surface Runtime-only guard |
| Commercial golden path registry | `runtime/v125-commercial-golden-path-registry.json` | registry-backed commercial golden path |
| Release-event artifact certification | `runtime/v125-release-event-artifact-certification.json` | published release-event artifact proof |
| v1.2.5 release certification | `runtime/v125-release-certification.json` | final release certification for v1.2.5 |

## Release Gate

`scripts/verify_release_gate.sh` must run the v1.2.5 proof chain after v1.2.4:

```text
run_v125_release_publication_state_gate
run_v125_candidate_published_certification_split_gate
run_v125_waiver_contract_consistency_gate
run_v125_commercial_registry_runtime_proofs_gate
run_v125_release_event_artifact_certification_gate
run_v125_release_certification_gate
```

## GitHub Traceability

This release closes GitHub issues `#934` through `#943`.
