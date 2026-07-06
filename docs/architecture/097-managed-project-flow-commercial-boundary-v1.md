# Managed Project Flow Commercial Boundary v1

创建日期：2026-07-06  
执行者：Codex

## Purpose

本文定义 Managed Project Flow 的商业边界，特别是 Software Dev 这类长周期 Product surface 如何复用 Core Runtime，而不成为 Core Runtime authority。

Managed Project Flow 回答：

```text
这个产品是否能以项目形态展示、提交项目级 Runtime command proposal、读取任务和交付历史。
```

## Version

```text
agentflow-managed-project-flow-commercial-boundary.v1
```

## Reference Product

Software Dev 是 Managed Project Flow Reference Product。

它的商业 flow 必须是：

```text
managed-project-flow
```

它不能被解释成：

```text
paid-report-flow
```

Software Dev 可以展示项目、角色、任务、证据、决策、交付历史和反馈入口，但不能直接写 Core Runtime authority。

## Authority Boundary

合法路径：

```text
Product surface project request
-> Commercial Product Read Model
-> Managed Project Flow boundary
-> Runtime command proposal
-> Core Runtime admission
-> Tasks
-> Evidence
-> Decision
-> Delivery History
-> Feedback Projection
```

Managed Project Flow 不能：

- 直接写 `.agentflow/spec/**`；
- 直接写 `.agentflow/events/**`；
- 直接写 `.agentflow/tasks/**`；
- 直接写 accepted Runtime action；
- 直接改 Core Runtime decision；
- 直接把 entitlement state 当作 Core Runtime decision；
- 使用 Paid Report-only 字段作为 Managed Project authority。

## Commercial Semantics

Managed Project Flow 必须覆盖：

| Concept | Product surface meaning | Core mapping |
| --- | --- | --- |
| project workspace | 项目工作区和可见范围 | Runtime workspace / projection |
| roles | Human Owner、Spec Agent、Work Agent / Build Agent、Delivery Agent、Audit Agent | role / permission read model |
| tasks | 项目任务链和状态流 | Runtime command proposals / task events |
| evidence | 每个任务的验证证据 | Evidence |
| decisions | 任务和项目完成判断 | Decision |
| delivery history | 项目交付历史和 Done 写回 | Delivery / Completion projection |
| feedback | 返工、补充需求或后续项目入口 | Feedback projection |

## Entitlement Boundary

License / entitlement 只能影响 Product surface 是否展示、是否允许提出 Runtime command proposal。

它不能改变 Core Runtime 的事实判断。

| Entitlement state | Product surface behavior | Runtime authority |
| --- | --- | --- |
| active | 可展示并可提出项目 Runtime command proposal | 仍需 Core Runtime admission |
| trial | 可展示并可提出受限项目 Runtime command proposal | 仍需 Core Runtime admission |
| disabled | Product surface rejected | 不能提交 Runtime command proposal |
| deferred | Product surface deferred | 不能提交 Runtime command proposal |
| missing | Product surface invalid | 不能提交 Runtime command proposal |

## Boundary Input Contract

最小输入字段：

```json
{
  "version": "agentflow-managed-project-flow-commercial-boundary.v1",
  "productId": "software-dev",
  "flowType": "managed-project-flow",
  "projectWorkspaceRef": "product/workspaces/software-dev-project.json",
  "goalRef": "docs/requirements/software-dev-goal.md",
  "roles": ["human-owner", "spec-agent", "work-agent", "delivery-agent", "audit-agent"],
  "taskGraphRef": ".agentflow/spec/projects/software-dev/tasks.json",
  "evidencePolicy": "per-task-evidence-required",
  "decisionPolicy": "task-and-project-decision-required",
  "deliveryHistoryRef": "projection/delivery-history/software-dev.json",
  "feedbackRoute": "project-feedback",
  "entitlementState": "active | trial | disabled | deferred | missing"
}
```

## Boundary Output Contract

最小输出字段：

```json
{
  "version": "agentflow-managed-project-flow-commercial-boundary.v1",
  "productId": "software-dev",
  "flowType": "managed-project-flow",
  "referenceProduct": true,
  "availability": "available | rejected | deferred | invalid",
  "unavailableReason": "none | disabled-entitlement | deferred-entitlement | missing-entitlement | paid-report-field-not-managed-project-authority | wrong-flow-contract",
  "runtimeCommandPolicy": "propose-to-runtime | blocked-before-runtime",
  "runtimeAdmissionRequired": true,
  "coreRuntimeDecisionAuthority": false,
  "projectWorkspaceRequired": true,
  "deliveryShape": "project-delivery-history"
}
```

## Paid Report Field Rejection

Managed Project Flow 不能使用 Paid Report-only 字段作为 authority。

必须拒绝：

```text
reportInputRef
orderIntentId
reportDefinitionId
reportDeliveryPromise: report
```

这些字段可以作为普通附加上下文存在，但不能替代：

```text
projectWorkspaceRef
goalRef
taskGraphRef
deliveryHistoryRef
```

## Non-goals

本文不实现：

- new Software Dev implementation work；
- payment provider；
- new industry app；
- cloud project collaboration；
- account administration。

## Release Proof Requirement

Release gate 必须生成：

```text
runtime/v123-managed-project-flow-commercial-boundary.json
```

并证明：

- `agentflow-managed-project-flow-commercial-boundary.v1` 已定义；
- Software Dev 映射到 `managed-project-flow`，不是 `paid-report-flow`；
- project workspace、roles、tasks、evidence、decisions、delivery history、feedback 都有合同语义；
- license / entitlement 只影响 Product surface access，不改变 Core Runtime decision；
- Paid Report-only 字段不能作为 Managed Project authority；
- Core Runtime 仍保持 industry-neutral。
