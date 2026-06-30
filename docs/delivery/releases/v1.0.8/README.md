# AgentFlow v1.0.8 Projection Kernel

更新日期：2026-06-30
执行者：Codex

## Status

`v1.0.8` 是 `v1.0.7` Core Decision Kernel 之后的 Core Projection Kernel release baseline。

本版本把 Core facts 投影成稳定、只读、可重放、可刷新、可反馈的 Projection Kernel：

```text
Spec / Event / Evidence / Decision facts
-> Projection Kernel Contract
-> Replay / Rebuild Receipt
-> Read Model Schema
-> View Model Contract
-> Pack Mapping Boundary
-> Invalid App Definition Handling
-> Feedback / Freshness Receipts
```

## Scope

`v1.0.8` 收口以下内容：

1. Core Projection Kernel Contract。
2. Event Replay and Projection Rebuild。
3. Core Read Model Schema。
4. View Model Contract for Industry Apps。
5. Pack-specific Projection Mapping Boundary。
6. Invalid / Missing App Definition Handling。
7. Feedback Surface Projection and Projection Freshness Receipts。
8. v1.0.8 Release Certification。

## Public Records

- [AGENTFLOW_V1_0_8_PROJECTION_KERNEL_TASKS_V1.md](AGENTFLOW_V1_0_8_PROJECTION_KERNEL_TASKS_V1.md)
- [../../../architecture/079-core-projection-kernel-contract-v1.md](../../../architecture/079-core-projection-kernel-contract-v1.md)
- [../../../architecture/080-event-replay-projection-rebuild-v1.md](../../../architecture/080-event-replay-projection-rebuild-v1.md)
- [../../../architecture/081-core-read-model-schema-v1.md](../../../architecture/081-core-read-model-schema-v1.md)
- [../../../architecture/082-view-model-contract-for-industry-apps-v1.md](../../../architecture/082-view-model-contract-for-industry-apps-v1.md)
- [../../../architecture/083-pack-specific-projection-mapping-boundary-v1.md](../../../architecture/083-pack-specific-projection-mapping-boundary-v1.md)
- [../../../architecture/084-invalid-missing-app-definition-handling-v1.md](../../../architecture/084-invalid-missing-app-definition-handling-v1.md)
- [../../../architecture/085-feedback-surface-projection-freshness-receipts-v1.md](../../../architecture/085-feedback-surface-projection-freshness-receipts-v1.md)
- [../v1.0.7/README.md](../v1.0.7/README.md)

## Release Gate Artifacts

`v1.0.8` release gate must produce:

```text
runtime/core-projection-kernel-contract.json
runtime/event-replay-projection-report.json
runtime/event-replay-projection-failure-report.json
runtime/core-read-model-schema.json
runtime/core-view-model-contract.json
runtime/projection-feedback-freshness-receipts.json
runtime/projection-feedback-freshness-rust-test.log
runtime/core-decision-projection-read-model.json
runtime/v108-release-certification.json
pack-projection-readiness.json
```

## Known Boundaries

- Projection remains read-only and cannot write Spec, Runtime, Evidence, Decision, Completion, Delivery or Audit authority.
- GitHub issues remain planning mirrors, not AgentFlow task authority.
- Provider / CLI sessions remain execution records, not project truth.
- Audit remains an optional sidecar flow, not the default business chain.
- Software Dev remains the first Reference App and does not become Core authority.

## Non-goals

- 不认证 Software Dev commercial app completion；
- 不启动 `v1.0.9` implementation；
- 不引入 default Message Bus；
- 不把 Projection 当成 authority；
- 不把 GitHub / provider / CLI session 当成 project truth。

## Known Risks

- Software Dev Reference App certification remains outside this Core Projection Kernel release.
- Future app console work must keep command surfaces bound to projection read models, not authority files.
- Feedback routes can propose Spec evolution, but confirmation and materialization remain separate authority gates.

## Next Version

`v1.0.9` should certify the Software Dev Reference App over the stable Core Projection Kernel without weakening Core industry-neutral boundaries.
