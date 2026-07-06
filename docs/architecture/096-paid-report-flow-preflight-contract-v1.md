# Paid Report Flow Preflight Contract v1

创建日期：2026-07-06  
执行者：Codex

## Purpose

本文定义 Paid Report Flow 在 Product surface 提交 Runtime command proposal 之前的预检合同。

Paid Report Flow 是一次性报告交付形态。Preflight 只回答：

```text
这个付费报告请求是否可以生成 Runtime command proposal。
```

它不等于 payment capture，不等于真实报告生成，也不等于 Core Runtime admission。

## Version

```text
agentflow-paid-report-flow-preflight.v1
```

## Authority Boundary

合法路径：

```text
Product surface report request
-> Commercial Product Read Model
-> Paid Report Flow Preflight
-> Runtime command proposal
-> Core Runtime admission
-> Evidence
-> Decision
-> Report Delivery
-> Feedback Projection
```

Paid Report Flow Preflight 不能：

- 直接写 `.agentflow/spec/**`；
- 直接写 `.agentflow/events/**`；
- 直接写 `.agentflow/tasks/**`；
- 直接启动 Runtime run；
- 直接写 Evidence；
- 直接写 Decision；
- 直接写 Report Delivery；
- 绕过 Core Runtime admission。

允许的 preflight 结果只能生成 Runtime command proposal。真正是否启动 run，仍由 Core Runtime admission 决定。

## Non-goals

本文不实现：

- real payment capture；
- customer account system；
- public report marketplace；
- domain-specific report Product；
- report generation engine。

## Preflight Input Contract

最小输入字段：

```json
{
  "version": "agentflow-paid-report-flow-preflight.v1",
  "productId": "software-dev",
  "flowType": "paid-report-flow",
  "requestId": "report-request-001",
  "reportInputRef": "product/report-inputs/report-request-001.json",
  "productAccessState": "available | rejected | deferred | invalid",
  "entitlementState": "active | trial | expired | disabled | deferred | unknown | missing",
  "paidFeatureState": "paid | deferred | unavailable | unknown",
  "orderIntentId": "order-intent-001",
  "reportDefinitionId": "report-definition-001",
  "controlledRunRequest": {
    "commandType": "generate-paid-report",
    "runtimeAdmissionRequired": true
  },
  "evidenceRequirement": {
    "required": true,
    "shape": "report-generation-evidence"
  },
  "decisionRequirement": {
    "required": true,
    "shape": "report-delivery-decision"
  },
  "reportDeliveryPromise": "report",
  "feedbackEntry": {
    "enabled": true,
    "route": "report-feedback"
  }
}
```

## Preflight Output Contract

最小输出字段：

```json
{
  "version": "agentflow-paid-report-flow-preflight.v1",
  "requestId": "report-request-001",
  "flowType": "paid-report-flow",
  "preflightOutcome": "allowed | rejected | deferred | invalid",
  "unavailableReason": "none | missing-input | entitlement-disabled | flow-deferred | report-template-missing | runtime-admission-rejected",
  "runtimeCommandPolicy": "propose-to-runtime | blocked-before-runtime | rejected-by-runtime-admission",
  "runtimeAdmissionRequired": true,
  "canSubmitRuntimeCommandProposal": true,
  "canStartRunDirectly": false,
  "evidenceRequirement": "report-generation-evidence",
  "decisionRequirement": "report-delivery-decision",
  "reportDeliveryPromise": "report",
  "feedbackEntry": "report-feedback"
}
```

## Required Checks

Preflight 必须检查：

| Check | Required behavior |
| --- | --- |
| input requirements | 缺少报告输入时输出 `invalid` |
| product access check | Product read model 不可用时输出 `rejected` / `deferred` / `invalid` |
| order intent | 必须存在 order intent，但它不等于 payment capture |
| controlled run request | 必须只生成 Runtime command proposal |
| evidence requirement | 必须声明报告生成证据 |
| decision requirement | 必须声明报告交付 decision |
| report delivery promise | 必须是 `report` |
| feedback entry | 必须有报告反馈入口 |

## Outcome Rules

| Case | Expected preflight outcome | Runtime policy |
| --- | --- | --- |
| complete paid report request | allowed | propose-to-runtime |
| missing input | invalid | blocked-before-runtime |
| entitlement disabled | rejected | blocked-before-runtime |
| flow deferred | deferred | blocked-before-runtime |
| report template missing | invalid | blocked-before-runtime |
| Runtime admission rejected | rejected | rejected-by-runtime-admission |

即使 outcome 是 `allowed`，也只允许提交 Runtime command proposal，不能直接启动 run。

## Negative Fixtures

Release gate 必须覆盖以下负例：

| Fixture | Expected |
| --- | --- |
| missing input | `invalid`，不能提交 Runtime command proposal |
| disabled entitlement | `rejected`，不能启动 run |
| flow deferred | `deferred`，不能启动 run |
| missing report definition | `invalid`，不能启动 run |
| Runtime admission rejected | `rejected`，不能生成 delivery |

## Release Proof Requirement

Release gate 必须生成：

```text
runtime/v123-paid-report-flow-preflight-contract.json
```

并证明：

- `agentflow-paid-report-flow-preflight.v1` 已定义；
- input requirements、product access check、order intent、controlled run request、evidence requirement、decision requirement、report delivery promise、feedback entry 都有合同字段；
- preflight outcome 覆盖 `allowed`、`rejected`、`deferred`、`invalid`；
- disabled entitlement 和 missing report definition 都不能启动 run；
- allowed preflight 仍必须经过 Core Runtime admission；
- 本 release 不实现 payment、account、public marketplace 或 report generation engine。
