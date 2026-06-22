# AgentFlow v0.7.2 Runtime Foundation Hardening Tasks V1

日期：2026-06-23
执行者：Codex

## Goal

`v0.7.2` 聚焦 Runtime Foundation hardening。

它不继续堆 Console 页面，也不扩张到 Pack / Cloud / 行业壳。

主线是：

```text
Audit Sidecar
-> Version / Migration
-> Simulation
-> Message Bus
-> Capability Registry
-> Provider Smoke
-> Connector Boundary
-> Plane Manifest
-> Readiness Report
-> Release Gate Coverage
```

## Issues

| Issue | Title | Priority | Dependency | Status |
| --- | --- | --- | --- | --- |
| `V072-001` | Independent Audit Loop Extraction | P0 | none | done |
| `V072-002` | Version & Migration Registry Baseline | P0 | none | done |
| `V072-003` | Simulation Rules & Dry-run Runtime | P0 | V072-002 | done |
| `V072-004` | Local Message Bus Contract | P0 | V072-001 | done |
| `V072-005` | Worker / Tool Capability Registry | P0 | V072-004 | planned |
| `V072-006` | Provider Smoke Gate Minimal | P0 | V072-005 | planned |
| `V072-007` | Connector / MCP Boundary Baseline | P1 | V072-005 | planned |
| `V072-008` | Runtime / Projection / Command API Plane Manifest | P1 | V072-004 | planned |
| `V072-009` | Foundation Readiness Report | P1 | V072-001 ~ V072-008 | planned |
| `V072-010` | Release Gate Foundation Coverage | P0 | V072-009 | planned |

## V072-001 Acceptance Anchor

`V072-001` 固定以下规则：

- Work Loop Done 不依赖 Audit；
- Delivery Package 不依赖 Audit；
- Completion Commit 不等待 Audit；
- Audit Surface 只读；
- Audit Finding 只能生成 Follow-up Proposal；
- `no-audit` 是合法 Done 状态。

## Boundary

这些 issue 是 Runtime Foundation hardening 任务。

不得在本批次中实现：

- Pack System；
- Cloud Runtime；
- remote Agent fleet；
- 行业产品行为；
- 大规模 provider production E2E；
- 自动远程审计。
