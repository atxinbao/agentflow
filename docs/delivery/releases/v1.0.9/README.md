# AgentFlow v1.0.9 Software Dev Reference App Boundary Certification

更新日期：2026-07-01
执行者：Codex

## Status

`v1.0.9` 是 `v1.0.8` Core Projection Kernel 之后的 Software Dev Reference App boundary certification release baseline。

本版本确认：

```text
crates/**                 = Core OS Runtime
products/software-dev/**  = first-party Software Dev Reference App source
crates/pack/fixtures/**   = fixture mirror only
apps/**                   = UI consumption only
docs/**                   = human-readable release and architecture records
.agentflow/**             = runtime fact surface for user projects
```

## Scope

`v1.0.9` 收口以下内容：

1. Release task / GitHub issue traceability gate。
2. Quick-audit Pack projection primary proof inclusion。
3. `products/software-dev/**` Reference App contract。
4. Software Dev Spec Bundle to task flow。
5. Software Dev connector handoff and Runtime command baseline。
6. Software Dev Evidence / Decision / Delivery closed loop。
7. Software Dev Projection Workbench read models。
8. Core / Product Pack-backed mapping boundary cleanup。
9. End-to-end Software Dev Reference App golden scenario and negative fixtures。
10. v1.0.9 Release Certification。

## Public Records

- [AGENTFLOW_V1_0_9_SOFTWARE_DEV_REFERENCE_APP_TASKS_V1.md](AGENTFLOW_V1_0_9_SOFTWARE_DEV_REFERENCE_APP_TASKS_V1.md)
- [../../../architecture/086-industry-product-source-boundary-v1.md](../../../architecture/086-industry-product-source-boundary-v1.md)
- [../v1.0.8/README.md](../v1.0.8/README.md)
- [../../../../products/software-dev/product.toml](../../../../products/software-dev/product.toml)

## Release Gate Artifacts

`v1.0.9` release gate must produce:

```text
runtime/v109-task-issue-traceability.json
runtime/v109-software-dev-product-contract.json
runtime/v109-spec-task-flow.json
runtime/v109-connector-handoff.json
runtime/v109-evidence-decision-delivery.json
runtime/v109-workbench-read-models.json
runtime/v109-mapping-boundary.json
runtime/v109-golden-scenario.json
runtime/v109-release-certification.json
pack-projection-readiness.json
quick-audit-manifest.json
```

## Known Boundaries

- GitHub issues remain planning mirrors, not AgentFlow authority.
- Product source does not write Runtime authority.
- Projection remains read-only and cannot write Spec, Runtime, Evidence, Decision, Completion, Delivery or Audit authority.
- Provider / CLI sessions remain execution records, not project truth.
- Audit remains an optional sidecar flow, not the default business chain.
- Software Dev is a first-party Reference App, not Core authority.

## Non-goals

- 不认证 Software Dev commercial product beta；
- 不启动 `v1.1.0` implementation；
- 不引入 default Message Bus；
- 不把 GitHub / provider / CLI session 当成 project truth；
- 不把 Software Dev concepts 写成 Core universal authority。

## Known Risks

- Product Surface hardening remains a follow-up release line.
- `products/software-dev/**` is now the source boundary, but future versions still need richer product installation and UI route handling.
- Quick-audit package is intentionally small; full release gate artifacts remain separate.

## Next Version

`v1.1.0` should harden product surface installation, console command routes and product pack consumption without weakening Core industry-neutral boundaries.
