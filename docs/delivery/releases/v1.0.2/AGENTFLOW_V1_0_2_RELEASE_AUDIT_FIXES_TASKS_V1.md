# AgentFlow v1.0.2 Release Audit Fixes Tasks V1

更新日期：2026-06-26
执行者：Codex

## Goal

`v1.0.2` 修复 `v1.0.1` 发布审计发现的 release hardening 缺口，
并把当前产品目标明确收口为 Software Dev Spec-Driven Workflow。

## Authority Boundary

- GitHub issues 是 planning mirror；
- `docs/project/**` 和 `docs/architecture/**` 是长期 source authority；
- `.agentflow/spec/**` 是未来执行合同 authority；
- release gate artifacts 是 release certification evidence。

## Task Order

### V102-001 Runtime Trusted Governance Telemetry Enforcement

状态：done

修复点：

- Runtime command admission 不再从 request input 读取 provider-ready telemetry；
- request input 里的 `governanceProviderStatuses` / `governanceProviderSmokeArtifacts`
  只能作为诊断输入，不能成为 capability authority；
- capability readiness 必须来自项目 runtime registry 或 release/runtime artifact。

对应 GitHub issue：#607

### V102-002 Release Provenance Lightweight Tag Semantics

状态：done

修复点：

- lightweight tag 写出 `tagObjectKind = commit`；
- annotated tag 写出真实 `annotatedTagObjectId`；
- release provenance 禁止写入字面量失败 revspec，例如 `v1.0.1^{tag}`；
- tag commit、source commit、release URL 必须语义对齐。

对应 GitHub issue：#608

### V102-003 Release Certification Negative Fixtures for V101 Claims

状态：done

修复点：

- 增加 forged governance telemetry negative fixture；
- 增加 malformed provenance / wrong tag / wrong commit / wrong URL 覆盖；
- certification artifact 明确列出 negative fixture coverage。

对应 GitHub issue：#609

### V102-004 Product Goal Baseline Integration

状态：done

修复点：

- 文档明确当前产品目标：

```text
AgentFlow = Spec-Driven Software Dev Workflow
```

- Spec 是 workflow authority；
- Agent 是 executor；
- Software Dev 是当前唯一 active commercial product target；
- GitHub issue 不成为 AgentFlow authority。

对应 GitHub issue：#610

### V102-005 v1.0.2 Release Certification

状态：done

修复点：

- release gate 生成 `runtime/v102-negative-fixtures.json`；
- release gate 生成 `runtime/v102-release-certification.json`；
- certification 输出 `v102ReleaseCertificationStatus = passed`；
- V102-001 到 V102-004 覆盖被显式记录。

对应 GitHub issue：#611

## Release Gate Artifacts

```text
runtime/trusted-governance-telemetry.json
runtime/release-provenance.json
runtime/v102-negative-fixtures.json
runtime/v102-release-certification.json
```

## Completion Standard

`v1.0.2` 只有在以下条件同时满足时才可发布：

- `cargo fmt --all --check` 通过；
- `cargo test --workspace` 通过；
- `npm --prefix apps/desktop run build` 通过；
- `git diff --check` 通过；
- `scripts/verify_release_gate.sh` 输出 v102 certification passed；
- GitHub release-gate 在 PR、main、tag、release 四类事件均通过。
