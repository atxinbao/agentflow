# AgentFlow v1.0.3 Core 4-D Spec Intake Tasks V1

更新日期：2026-06-28
执行者：Codex

## Goal

`v1.0.3` 将 Core 4-D Spec Intake 落成 Core Spec Kernel 的可验证入口。

这不是产品功能扩张，而是把后续 Spec Bundle Workspace 需要的阶段合同、
slice 合同、路由合同和跨行业 fixture 固化到 release gate。

## Authority Boundary

- GitHub issues 是 planning mirror；
- `docs/requirements/v0.18.0-core-4d-spec-intake/**` 是 confirmed Spec Bundle；
- `docs/architecture/053-core-4d-spec-intake-kernel-v1.md` 是长期架构合同；
- `crates/spec/src/core_intake.rs` 是 Rust 合同实现；
- release gate artifacts 是 release certification evidence。

## Task Order

### V018-001 Core 4-D Stage Contract

状态：done

修复点：

- 定义 Deconstruct、Diagnose、Develop、Deliver 四阶段合同；
- 阶段合同不绑定单一行业；
- 阶段输出必须能进入 Spec Bundle slice。

对应 GitHub issue：#618

### V018-002 Intent Packet

状态：done

修复点：

- 定义 raw human request 到 intent packet 的结构；
- 明确缺失信息、证据、边界和后续路由。

对应 GitHub issue：#619

### V018-003 Gap Route Policy

状态：done

修复点：

- 定义 clarify、research、define、plan、task、decide、deliver、evolve 路由；
- 路由由 gap 类型和当前 artifact 决定，不由行业名决定。

对应 GitHub issue：#620

### V018-004 Clarify Interaction

状态：done

修复点：

- 将 human decision gap 固化为 bounded question；
- 明确需要人类确认的缺口不能被 Agent 伪造完成。

对应 GitHub issue：#621

### V018-005 Research Evidence

状态：done

修复点：

- 将 fact gap 物料化为 research evidence；
- evidence 只能支持判断，不能直接升级为 authority。

对应 GitHub issue：#622

### V018-006 Spec Bundle Slices

状态：done

修复点：

- 定义 Intent、Domain、Goal、Plan、Task、Decision、Output、Feedback slice；
- 每个 slice 都必须可追溯到 Core 4-D 阶段。

对应 GitHub issue：#623

### V018-007 Industry Mapping

状态：done

修复点：

- 定义 Software Dev、UI Design、Video Production 三个参考映射；
- Core 合同不能写死 feature、bug、PR、repository 等单一行业词。

对应 GitHub issue：#624

### V018-008 Materialization Boundary

状态：done

修复点：

- 定义 Draft、Preview、Confirmed、Materialized 边界；
- Preview 不能成为 authority，Confirmed 后才能进入执行合同。

对应 GitHub issue：#625

### V018-009 Cross-industry Fixtures

状态：done

修复点：

- 增加跨行业 fixture；
- 验证 Core 4-D 合同不被 Software Dev 专用概念污染。

对应 GitHub issue：#626

### V018-010 Release Certification

状态：done

修复点：

- release gate 生成 `runtime/core-4d-spec-intake.json`；
- `cargo test -p agentflow-spec core_4d` 作为本版本核心证明；
- `v103-release-fix-certification` 保留 `v1.0.2` release audit 修复证明。

对应 GitHub issue：#627

## Release Gate Artifacts

```text
runtime/v103-release-fix-certification.json
runtime/core-4d-spec-intake.json
runtime/core-4d-spec-intake-rust-test.log
```

## Completion Standard

`v1.0.3` 只有在以下条件同时满足时才可发布：

- `cargo fmt --all --check` 通过；
- `cargo test -p agentflow-spec` 通过；
- `npm --prefix apps/desktop run build` 通过；
- `git diff --check` 通过；
- `scripts/verify_release_gate.sh` 输出 v103 和 Core 4-D certification passed；
- GitHub release-gate 在 PR、main、tag、release 四类事件均通过。

