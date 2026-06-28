# AgentFlow v1.0.4 Core Ontology Kernel Tasks V1

更新日期：2026-06-28
执行者：Codex

## Goal

`v1.0.4` 将 Core Ontology Kernel 落成可验证的 Core OS Runtime 定义层。

这不是产品功能扩张，而是把 Object、Link、Action、State、Skill、Evidence、Decision、Registry 和 Projection 的通用合同固化到 release gate。

## Authority Boundary

- GitHub issues 是 planning mirror；
- `docs/architecture/054` 到 `docs/architecture/059` 是长期架构合同；
- `crates/ontology/src/**` 是 Rust 合同实现；
- release gate artifacts 是 release certification evidence；
- Reference App mapping is not Core authority。

## Task Order

### V104-001 Core 4-D Regression Baseline

状态：done

修复点：

- 保留 Core 4-D Spec Intake 的 positive / negative baseline；
- 确保 v1.0.4 不回退 v1.0.3 的 Spec Kernel 入口。

对应 GitHub issue：#630

### V104-002 Core Intake Pollution Guard Hardening

状态：done

修复点：

- 加强 Core intake 行业词污染防线；
- 确保 Core 不接收 Software Dev 专用词作为权威字段。

对应 GitHub issue：#631

### V104-003 Core 4-D Positive and Negative Certification

状态：done

修复点：

- release gate 同时输出 positive 和 negative 认证；
- 证明 Core 4-D 能接受跨行业输入并拒绝行业词污染。

对应 GitHub issue：#632

### V104-004 Core Ontology Kernel Contract

状态：done

修复点：

- 定义 Object、Link、Action、State、Skill、Evidence、Decision、Projection 等 Core element；
- 明确 Reference App mapping 不是 Core authority。

对应 GitHub issue：#633

### V104-005 Object and Link Schema

状态：done

修复点：

- 定义 11 个 Core object；
- 定义 12 个 Core link；
- release gate 输出 `runtime/core-object-link-schema.json`。

对应 GitHub issue：#634

### V104-006 Action and State Semantics

状态：done

修复点：

- 定义 12 个 Core action；
- 定义 10 个 Core state；
- 定义 12 个 Core transition；
- release gate 输出 `runtime/core-action-state-semantics.json`。

对应 GitHub issue：#635

### V104-007 Skill Registry and Action Authorization

状态：done

修复点：

- 定义 6 个 Core skill；
- 将 skill、allowed action、tool scope、connector scope 和 expected output 绑定；
- release gate 输出 `runtime/core-skill-registry.json`。

对应 GitHub issue：#636

### V104-008 Evidence and Decision Reference Model

状态：done

修复点：

- 定义 5 个 evidence reference；
- 定义 3 个 decision reference；
- 定义 10 个 decision outcome；
- release gate 输出 `runtime/core-evidence-decision-reference-model.json`。

对应 GitHub issue：#637

### V104-009 File-backed Ontology Registry and Projection

状态：done

修复点：

- 定义 5 个 file-backed registry source；
- 定义 5 个 read-only projection entry；
- release gate 输出 `runtime/core-file-backed-ontology-registry.json`。

对应 GitHub issue：#638

### V104-010 Release Certification

状态：done

修复点：

- 将 workspace 和 desktop 版本切到 `1.0.4`；
- 将当前 release baseline 切到 `docs/delivery/releases/v1.0.4/README.md`；
- release gate 输出 `runtime/v104-release-certification.json`；
- GitHub release-gate 在 PR、main、tag、release 四类事件均应通过。

对应 GitHub issue：#639

## Release Gate Artifacts

```text
runtime/core-ontology-kernel.json
runtime/core-object-link-schema.json
runtime/core-action-state-semantics.json
runtime/core-skill-registry.json
runtime/core-evidence-decision-reference-model.json
runtime/core-file-backed-ontology-registry.json
runtime/v104-release-certification.json
```

## Completion Standard

`v1.0.4` 只有在以下条件同时满足时才可发布：

- `cargo fmt --all --check` 通过；
- `cargo test --workspace` 通过；
- `npm --prefix apps/desktop run build` 通过；
- `git diff --check` 通过；
- `scripts/verify_release_gate.sh` 输出 Core Ontology Kernel 和 v104 certification passed；
- GitHub release-gate 在 PR、main、tag、release 四类事件均通过。
