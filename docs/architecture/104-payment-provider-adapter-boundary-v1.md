# Payment Provider Adapter Boundary V1

更新日期：2026-07-10
执行者：Codex

## Purpose

本文定义 `v1.3.0` 的 Payment Provider Adapter boundary。

它回答一个问题：

```text
未来 Stripe / WeChat / Alipay 或其他支付 provider
如何把支付授权结果喂给 AgentFlow，
同时不让 Core Runtime 拥有 checkout / charge / refund 执行。
```

## Authority Boundary

Core Runtime owns:

- normalized payment authorization result；
- entitlement authorization reference；
- payment / refund status strings；
- provider evidence refs；
- entitlement effect；
- failure reason。

Payment Provider Adapter owns:

- provider checkout session creation；
- charge execution；
- refund execution；
- provider credentials；
- provider webhook normalization；
- provider-specific payment intent state。

Core Runtime must not:

- create checkout sessions；
- execute charges；
- execute refunds；
- store provider credentials；
- treat provider checkout implementation as Core authority；
- mutate delivered artifacts during refund handling。

## Machine-readable Contract

Release gate proof:

```text
runtime/v130-payment-provider-adapter-boundary.json
```

Contract version:

```text
agentflow-payment-provider-adapter-boundary.v1
```

Required fields:

```text
providerPaymentIntentRef
checkoutSessionRef
entitlementAuthorizationRef
paymentStatus
refundStatus
sourceRefs
```

## Dry-run Fixtures

Required fixtures:

```text
paid
failed
refunded
revoked
missing
```

The `paid` fixture must prove:

- provider payment intent ref exists；
- checkout session ref exists；
- entitlement authorization ref exists；
- `paymentStatus = paid`；
- `refundStatus = none`；
- Core consumes normalized authorization result；
- Core consumes provider evidence；
- provider checkout implementation is not Core authority。

Negative fixtures must prove:

- payment failure blocks entitlement；
- refund state changes entitlement effect without mutating delivery artifact；
- revoked state blocks entitlement；
- missing provider state does not fabricate provider refs；
- refund execution remains provider-side。

## Stable Statuses

Payment statuses:

```text
paid
failed
missing
```

Refund statuses:

```text
none
refunded
unknown
```

## Release Gate

`scripts/verify_release_gate.sh` must run:

```text
cargo run -p agentflow-runtime-api --example v130_payment_provider_adapter_boundary_proofs -- \
  <runtime-dir>/v130-payment-provider-adapter-boundary.json
```

The gate fails if:

- any required field is missing；
- paid fixture lacks provider payment intent, checkout session or entitlement
  authorization ref；
- Core owns checkout implementation；
- Core owns refund execution；
- failed / refunded / revoked / missing fixtures lack failure reasons；
- missing fixture fabricates provider refs。

## Non-goals

This boundary does not implement:

- real payment provider integration；
- checkout session creation；
- charge execution；
- refund execution；
- public commercial launch；
- customer account system；
- cloud multi-tenant launch。
