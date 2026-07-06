# AgentFlow v1.2.3 Release Closeout Proof Hardening and Commercial Surface Traceability

更新日期：2026-07-07
执行者：Codex

## Release Baseline

`v1.2.3` 是 Release Closeout Proof Hardening and Commercial Surface Traceability release baseline。

这一版建立在 `v1.2.2` Release Proof Hardening and Commercial Boundary Preflight 之上，继续收紧发布证明链，并把商业边界从文档合同推进到可追踪的 read model、preflight 和 Desktop 只读 surface：

```text
live GitHub milestone closeout proof
-> release closeout self-assertion negative fixture
-> v1.2.2 closeout repair proof
-> commercial proof version negative fixture
-> commercial product read model contract
-> paid report flow preflight contract
-> managed project flow commercial boundary
-> Desktop commercial boundary surface
-> commercial boundary negative fixtures
-> v1.2.3 release certification
```

## Scope

`v1.2.3` 收口以下内容：

1. live GitHub milestone closeout certification。
2. release closeout proof cannot self-assert remote state。
3. v1.2.2 milestone closeout repair。
4. V122 commercial proof artifact traceability alignment。
5. commercial product read model contract。
6. paid report flow preflight contract。
7. managed project flow commercial boundary。
8. Desktop commercial boundary surface。
9. commercial boundary negative fixtures。
10. v1.2.3 release certification。

## Certified Boundary

`v1.2.3` 认证的是发布 closeout 证明和商业边界可追踪性，不是商业发布。

这一版确认：

- release closeout proof must use live provider evidence when GitHub state is required；
- self-asserted remote closeout facts are rejected by a negative fixture；
- the repaired `v1.2.2` milestone closeout is retained as a v1.2.3 proof；
- commercial proof artifacts must be release-scoped and cannot reuse wrong-version primary proofs；
- commercial Product read model is projection-only and cannot write Runtime authority；
- paid report flow must stop before Runtime when entitlement / paid feature checks fail；
- managed project flow remains Core Runtime-governed and cannot gain paid report authority；
- Desktop commercial surface only renders read model facts and does not submit commands；
- negative fixtures cover unavailable, invalid and wrong-authority commercial states。

## Non-goals

`v1.2.3` 不包含：

- payment provider integration；
- checkout / billing implementation；
- cloud multi-tenant launch；
- public commercial launch；
- customer account system；
- organization admin；
- new industry Product；
- rewriting `v1.2.2` release history。

## Primary Proof Index

| Proof | Path / URL | Purpose |
| --- | --- | --- |
| Live GitHub milestone closeout | `runtime/live-github-milestone-closeout.json` | live provider-backed release closeout state |
| Closeout self-assertion negative fixture | `runtime/release-closeout-proof-negative-fixture.json` | rejects self-asserted remote closeout facts |
| v1.2.2 milestone closeout repair | `runtime/v122-milestone-closeout-repair.json` | records repaired live v1.2.2 milestone state |
| V122 commercial proof artifact traceability alignment | `runtime/v122-commercial-proof-version-negative-fixture.json` | rejects wrong-version commercial primary proofs |
| Commercial product read model | `runtime/v123-commercial-product-read-model-contract.json` | projection-only commercial read model |
| Paid report flow preflight | `runtime/v123-paid-report-flow-preflight-contract.json` | paid report flow preflight and rejection reasons |
| Managed project flow boundary | `runtime/v123-managed-project-flow-commercial-boundary.json` | managed project flow keeps Core Runtime authority |
| Desktop commercial surface | `runtime/v123-desktop-commercial-boundary-surface.json` | Desktop read-only commercial boundary surface |
| Commercial negative fixtures | `runtime/v123-commercial-boundary-negative-fixtures.json` | negative fixture matrix for commercial boundary states |
| v1.2.3 release certification | `runtime/v123-release-certification.json` | v1.2.3 final certification |

## GitHub Traceability

Task traceability is recorded in:

- [AGENTFLOW_V1_2_3_RELEASE_CLOSEOUT_COMMERCIAL_TRACEABILITY_TASKS_V1.md](AGENTFLOW_V1_2_3_RELEASE_CLOSEOUT_COMMERCIAL_TRACEABILITY_TASKS_V1.md)

## Release Certification

The release gate for `v1.2.3` must certify:

- all V123 issues `#903` through `#912` are closed before release certification completes；
- workspace, Desktop package, lockfile and Tauri version metadata match `1.2.3`；
- `CHANGELOG.md`, `docs/delivery/README.md` and this release baseline point at `v1.2.3`；
- live GitHub milestone closeout proof and self-assertion negative fixture pass；
- v1.2.2 closeout repair and commercial proof version negative fixture remain part of the v1.2.3 chain；
- commercial product read model, paid report preflight, managed project boundary, Desktop surface and negative fixture proofs pass；
- the small certification artifact includes top-level release metadata, issue traceability and primary proof index。

## Next Version

The next release can build on this proof chain to continue Product console continuity and commercial preflight work. Payment provider integration, cloud launch and new industry Product work remain out of scope until Runtime authority and commercial admission are separately certified.
