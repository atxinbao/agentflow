# AgentFlow v1.1.6 Executor Adapter Real Execution Closure

更新日期：2026-07-03
执行者：Codex

## Release Baseline

`v1.1.6` 是 Executor Adapter Real Execution Closure release baseline。

这一版把外部执行器返回结果纳入受控 Issue Loop：

```text
Spec Issue
-> Executor handoff package
-> executor result
-> diff boundary check
-> evidence capture
-> issue / run writeback
-> projection
```

## Scope

`v1.1.6` 收口以下内容：

1. 下一版发布权威与 roadmap 对齐。
2. Core route next-action 语义收紧。
3. Product Spec Intake Desktop invocation bridge。
4. Executor Adapter handoff package。
5. Allowed surface / diff boundary check。
6. Executor evidence capture。
7. Executor result to issue / run status writeback。
8. Failure / timeout / cancel / retry lifecycle receipts。
9. Software Dev real executor golden path。
10. v1.1.6 release certification。

## Release Gate Artifacts

`v1.1.6` release gate must produce:

- `runtime/v116-next-release-authority-alignment.json`
- `runtime/v116-core-route-next-action-semantics.json`
- `runtime/v116-product-spec-intake-desktop-invocation-bridge.json`
- `runtime/v116-executor-adapter-handoff-package.json`
- `runtime/v116-allowed-surface-diff-boundary-check.json`
- `runtime/v116-executor-evidence-capture.json`
- `runtime/v116-executor-result-issue-run-writeback.json`
- `runtime/v116-failure-timeout-cancel-retry-semantics.json`
- `runtime/v116-software-dev-real-executor-golden-path.json`
- `runtime/v116-release-certification.json`

## GitHub Traceability

Task traceability is recorded in:

- [AGENTFLOW_V1_1_6_EXECUTOR_ADAPTER_REAL_EXECUTION_TASKS_V1.md](AGENTFLOW_V1_1_6_EXECUTOR_ADAPTER_REAL_EXECUTION_TASKS_V1.md)

## Authority Rules

- `.agentflow/spec/issues/**` remains the executable issue authority.
- `.agentflow/tasks/<issue-id>/runs/<run-id>/launch/**` stores executor handoff packages.
- `.agentflow/tasks/<issue-id>/runs/<run-id>/evidence/**` stores executor evidence and boundary reports.
- Executor session output is not authority until evidence capture and diff boundary checks pass.
- executor session is not authority; `.agentflow/spec/issues/**` and accepted task evidence remain authority.
- Failed, timed out, cancelled and retried runs must leave lifecycle receipts.

## Non-goals

- This release does not add full provider launch supervision.
- This release does not redesign Desktop task pages.
- This release does not make GitHub issues task authority.
- This release does not bypass Spec Issue authority.

## Next Version

`v1.1.7` can improve Evidence / Decision / Delivery surface readability after Executor Adapter real execution closure is certified.
