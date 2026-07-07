# AgentFlow v1.2.4 Commercial Runtime Read Model and Closeout Distinction

更新日期：2026-07-07
执行者：Codex

## Release Baseline

`v1.2.4` 是 Commercial Runtime Read Model and Closeout Distinction release baseline。

这一版建立在 `v1.2.3` Release Closeout Proof Hardening and Commercial Surface Traceability 之上，把商业边界从文档和 Desktop 静态派生继续推进到 Runtime API、Projection Query、Preflight API、Runtime fixture 和 golden path：

```text
live closeout split fields
-> final closeout certification
-> commercial product Runtime read model
-> commercial projection query
-> paid report Runtime preflight
-> managed project runtime fixture
-> commercial negative runtime fixtures
-> commercial surface golden path
-> v1.2.4 release certification
```

## Scope

`v1.2.4` 收口以下内容：

1. 区分 `hasNoOpenIssues` 和 `isMilestoneClosed`，禁止把“无 open issue”当作 milestone closed。
2. final release certification 拒绝未带合规 waiver 的 deferred live closeout。
3. 提供 Commercial Product Runtime read model API。
4. 提供 Commercial Product projection query surface。
5. Desktop Commercial Surface 只消费 Runtime read model，Browser Preview fallback 必须显式标记。
6. 提供 Paid Report preflight Runtime API。
7. 提供 Managed Project commercial boundary runtime fixture。
8. 将 commercial negative fixtures 纳入 Runtime tests / artifacts。
9. 提供 Commercial Surface golden path proof。
10. 提供 v1.2.4 release certification。

## Certified Boundary

`v1.2.4` 认证的是商业 runtime read model、projection query、preflight 和 closeout 语义，不是商业发布。

这一版确认：

- `hasNoOpenIssues` 和 `isMilestoneClosed` 是两个独立事实；
- open milestone with zero open issues 不能被认证为 closed milestone；
- final release certification 不能接受无 waiver 的 deferred live closeout；
- waiver 必须包含原因、观测到的 provider 状态、时间和 source commit；
- Commercial Product read model 由 Runtime API 输出，Desktop 不再从 command surface 合成；
- projection query 只读，不能写 authority facts；
- Paid Report 只能提交 Runtime command proposal，不能直接启动 run；
- Managed Project 保持 Core Runtime 工作流，不继承 Paid Report authority 字段；
- negative fixtures 覆盖 disabled / expired / deferred / missing / wrong flow 等商业失败状态；
- Desktop Browser Preview fallback 必须标记为 preview，不伪装成 Runtime fact。

## Non-goals

`v1.2.4` 不包含：

- payment provider integration；
- checkout / billing implementation；
- customer account system；
- cloud multi-tenant launch；
- public commercial launch；
- new industry Product；
- paid report actual generation；
- managed project paid packaging。

## Primary Proof Index

| Proof | Path / URL | Purpose |
| --- | --- | --- |
| Live closeout distinction | `runtime/v124-live-closeout-distinction.json` | proves split no-open-issues vs closed milestone semantics |
| Final closeout certification | `runtime/v124-final-closeout-certification.json` | rejects deferred closeout in published certification without waiver |
| Commercial Product Runtime read model | `runtime/v124-commercial-product-read-model-runtime-api.json` | Runtime API read model for paid report and managed project commercial state |
| Commercial projection query | `runtime/v124-commercial-product-projection-query.json` | projection query surface for the commercial read model |
| Paid Report preflight Runtime API | `runtime/v124-paid-report-preflight-runtime-api.json` | paid report preflight decisions before Runtime command admission |
| Managed Project commercial runtime fixture | `runtime/v124-managed-project-commercial-runtime-fixture.json` | managed project commercial boundary fixture |
| Commercial negative runtime fixtures | `runtime/v124-commercial-negative-runtime-fixtures.json` | negative runtime fixtures for unavailable / invalid commercial states |
| Commercial golden path | `runtime/v124-commercial-golden-path.json` | read model -> projection -> Desktop -> preflight golden path |
| v1.2.4 release certification | `runtime/v124-release-certification.json` | final release certification for v1.2.4 |

## Release Gate

`scripts/verify_release_gate.sh` must run the v1.2.4 proof chain after the v1.2.3 certification chain:

```text
run_v124_live_closeout_distinction_gate
run_v124_final_closeout_certification_gate
run_v124_commercial_runtime_proofs_gate
run_v124_release_certification_gate
```

## GitHub Traceability

This release closes GitHub issues `#923` through `#932`.

