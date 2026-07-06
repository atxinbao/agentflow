# Commercial Boundary Negative Fixtures v1

创建日期：2026-07-07  
执行者：Codex

## Purpose

本文定义 `v1.2.3` 商业边界负向夹具，确保 Product surface 在未来接入 payment、public Product 或外部 entitlement 前，不会把商业状态误当成 Core Runtime authority。

这些夹具只验证边界：

```text
Commercial Product Read Model
-> Product surface availability
-> Runtime command proposal policy
```

它们不实现 payment、checkout、account、public marketplace 或外部 entitlement service。

## Version

```text
agentflow-commercial-boundary-negative-fixtures.v1
```

## Fixture Matrix

| Fixture | Flow | Input condition | Expected decision | Runtime command proposal |
| --- | --- | --- | --- | --- |
| expired-entitlement | paid-report-flow | entitlement expired | rejected | blocked-before-runtime |
| disabled-entitlement | paid-report-flow | entitlement disabled | rejected | blocked-before-runtime |
| deferred-entitlement | paid-report-flow | entitlement deferred | deferred | blocked-before-runtime |
| missing-entitlement | paid-report-flow | entitlement missing | invalid | blocked-before-runtime |
| unknown-product | paid-report-flow | productId missing or unknown | invalid | blocked-before-runtime |
| missing-flow-definition | paid-report-flow | flow definition missing | invalid | blocked-before-runtime |
| wrong-flow-type | managed-project-flow | flow type is paid-report-flow | invalid | blocked-before-runtime |
| payment-not-configured | paid-report-flow | payment configuration missing | deferred | blocked-before-runtime |
| paid-report-authority-in-managed-project | managed-project-flow | reportInputRef/reportDefinitionId/reportDeliveryPromise used as authority | invalid | blocked-before-runtime |
| desktop-deferred-rendering | paid-report-flow | entitlement deferred | deferred visible, not ready | no executable command |
| desktop-invalid-rendering | paid-report-flow | paid feature disabled | invalid/rejected visible | no executable command |

## Decision Semantics

每个失败模式必须落到一个明确 decision：

```text
rejected
deferred
invalid
```

不得出现：

- failed commercial preflight 继续进入 Core Runtime admission；
- disabled paid feature 渲染成 executable；
- deferred entitlement 渲染成 ready；
- Managed Project Flow 使用 Paid Report-only authority 字段；
- Desktop 直接写 `.agentflow/spec/**`、`.agentflow/events/**` 或 `.agentflow/tasks/**`。

## Paid-only Flow Rule

`paid-report-flow` 是 paid-only flow。只要 Product-layer preflight 失败：

```text
canSubmitRuntimeCommandProposal = false
runtimeCommandPolicy = blocked-before-runtime
runtimeAdmissionAttempted = false
```

## Managed Project Rule

`managed-project-flow` 只能使用项目权威字段：

```text
projectWorkspaceRef
goalRef
taskGraphRef
deliveryHistoryRef
```

它不能使用 Paid Report-only 字段作为 authority：

```text
reportInputRef
orderIntentId
reportDefinitionId
reportDeliveryPromise
```

## Desktop Rendering Rule

Desktop 只能展示 Product read model 派生状态：

- flow type；
- entitlement status；
- paid feature status；
- unavailable reason；
- next action；
- command policy。

不可用商业动作只允许展示为：

```text
不可执行 / 已暂缓 / 无效
```

不能渲染成可点击执行命令。

## Release Proof Requirement

Release gate 必须生成：

```text
runtime/v123-commercial-boundary-negative-fixtures.json
```

并证明：

- 全部负向夹具已执行；
- 每个夹具包含 input、expected output、actual output；
- failed commercial preflight 不会进入 Runtime admission；
- Managed Project Flow 拒绝 Paid Report-only authority 字段；
- Desktop invalid/deferred 状态不渲染成 executable command；
- 没有新增 payment provider、checkout、account settings 或 public launch messaging。
