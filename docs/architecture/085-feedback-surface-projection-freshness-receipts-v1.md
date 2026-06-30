# Feedback Surface Projection Freshness Receipts

日期：2026-06-30  
执行者：Codex

## Purpose

`projection` must tell humans and apps whether a surface is current, stale,
blocked, accepted, or ready to start a confirmed Spec evolution flow.

This document defines the read-only Feedback Surface and Projection Freshness
Receipt contract for v1.0.8.

## Boundary

- Projection is read-only.
- Projection does not write `.agentflow/spec/**`.
- Projection does not write runtime facts, evidence, decisions, or audit facts.
- Feedback Surface is a route hint, not Feedback Loop automation.
- Any route into Spec evolution must require preview confirmation and
  materialization before authority changes.

## Freshness Receipt

Every projection read model must carry a freshness receipt:

```text
version
receiptId
projectionRef
sourceRefs
sourceDigest
rebuildReceiptRef
status
staleReason
generatedAt
writesAuthority
```

Rules:

- `version` is `projection-freshness-receipt.v1`.
- `sourceDigest` is a stable projection source fingerprint.
- `rebuildReceiptRef` points to `.agentflow/projections/replay-report.json`.
- `status` mirrors the projection freshness state.
- `staleReason` is present when the projection is stale, missing, or incomplete.
- `writesAuthority` is always `false`.

## Feedback Surface Route

Every projection surface read model must carry a feedback route:

```text
status
route
reason
sourceSurfaceKey
targetAuthority
proposalKind
requiresConfirmation
confirmationBoundary
writesAuthority
```

Rules:

- Current surfaces use `status=accepted`.
- Stale or incomplete surfaces use `status=ready-for-spec-evolution`.
- Missing source facts use `status=blocked`.
- Spec evolution routes must use `proposalKind=spec-evolution-preview`.
- Spec evolution routes must set `requiresConfirmation=true`.
- Spec evolution routes must set
  `confirmationBoundary=preview-confirmation-materialization-required`.
- `targetAuthority` is `.agentflow/spec/**`.
- `writesAuthority` is always `false`.

## Release Gate Evidence

The release gate must prove:

- `ProjectionFreshnessReceipt` exists in the projection query surface.
- `ProjectionFeedbackRoute` exists in the projection query surface.
- task workbench stale projections route to Spec evolution preview.
- feedback routes require confirmation before materialization.
- projection freshness receipts include source refs, source digest, rebuild
  receipt ref, status, and stale reason.
- projection and feedback routes do not write authority.

