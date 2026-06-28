# AgentFlow v1.0.5 Core Runtime Kernel Tasks V1

更新日期：2026-06-28
执行者：Codex

## Goal

`v1.0.5` 将 Core Ontology Kernel 接入 Runtime 执行链路。

这不是 Software Dev 产品功能扩张，而是把 Runtime command、admission、Action Proposal、Arbitration、executor closeout 和 state writeback 从 hardcoded Software Dev 行为迁移到 Core contract + App Pack mapping。

## Authority Boundary

- GitHub issues 是 planning mirror；
- `docs/delivery/releases/v1.0.5/**` 是本版本 release public record；
- `docs/architecture/054` 到 `docs/architecture/059` 是上游 Core Ontology 合同；
- `crates/ontology/src/**` 是 Core Ontology Rust 合同实现；
- Runtime contracts 必须消费 Core authority；
- Software Dev Reference App mapping is not Core Runtime authority；
- provider session、GitHub issue、PR、test log 和 release note 都不是 Core Runtime command authority。

## Core Runtime Flow

```text
Core Runtime Command
-> Runtime Admission
-> Action Proposal
-> Arbitration
-> Executor Adapter Closeout
-> Completion Commit / State Writeback
-> Projection Refresh
-> Release Certification
```

Runtime 只能在 admission 和 arbitration 通过后推进执行。Rejected、deferred、blocked、failed、cancelled 和 completed 路径都必须保留 machine-readable reason。

## Task Order

### V105-001 Core Runtime Kernel Contract

状态：done

目标：

- 定义 Core Runtime Kernel 的目的、边界和任务链；
- 明确 Runtime command、admission、Action Proposal、Arbitration、executor closeout、state writeback 和 certification 边界；
- 明确 Software Dev 词汇只能通过 Reference App mapping 进入 Runtime；
- 对应 GitHub issue：#651。

依赖：无。

### V105-002 Replace Hardcoded Command Mapping with Core + App Pack Mapping

状态：done

目标：

- 将 `submitRequirement`、`createIssue`、`markIssueDone`、`requestAudit` 等硬编码命令迁移为 Core action resolver；
- 定义 generic command fields；
- Software Dev command aliases 只通过 Reference App mapping 解析；
- 对应 GitHub issue：#652。

依赖：#651。

### V105-003 Runtime Admission Uses Core Skill Registry

状态：done

目标：

- Runtime Admission 使用 Core Skill Registry 做硬门禁；
- 检查 role、skill、allowed action、allowed tool scope、connector scope、required evidence、target object、allowed surface 和 forbidden surface；
- rejected / deferred command 不得创建 proposal、accepted action 或 state transition；
- 对应 GitHub issue：#653。

依赖：#652。

### V105-004 Core Action Proposal Materialization

状态：done

目标：

- 从 accepted Core Runtime command 物化 Action Proposal；
- proposal 必须绑定 Core Object / Link / Action / State / Skill / Evidence 合同；
- app-specific aliases 必须通过 App Pack mapping 解析；
- 对应 GitHub issue：#654。

依赖：#652、#653。

### V105-005 Arbitration Uses Core Action State Semantics

状态：done

目标：

- Arbitration 只消费 admitted Action Proposal；
- 检查 role、skill、Core action、Core state transition、evidence expectation、allowed surface 和 object lock / queue policy；
- 返回 accepted / rejected / deferred 和稳定 reason；
- 对应 GitHub issue：#655。

依赖：#653、#654。

### V105-006 File-backed Ontology Registry Runtime Loader

状态：done

目标：

- 将 file-backed ontology registry 从 release-gate proof 升级为 Runtime 可消费的只读 loader / projection；
- Runtime command、admission 和 proposal 可以读取已验证 Core definitions；
- pollution guard diagnostics 必须指出 field、original text、normalized term 和 mapping boundary；
- 对应 GitHub issue：#656。

依赖：#651、#654。

### V105-007 Executor Adapter Closeout Integration

状态：done

目标：

- 定义 executor handoff 和 closeout 合同；
- 将 executor output 规范化为 Core Evidence / Artifact / Decision references；
- closeout 必须在 state writeback 前通过验证；
- 对应 GitHub issue：#657。

依赖：#655、#656。

### V105-008 Task Run State Writeback Authority

状态：done

目标：

- 定义 Completion Commit 写入顺序；
- accepted action、event append、run / task status write、projection refresh、delivery-ready record 必须有明确边界；
- projection 不能成为 authority；
- 对应 GitHub issue：#658。

依赖：#655、#657。

### V105-009 Negative Runtime Fixtures and Software Dev Reference Mapping

状态：done

目标：

- 用 Software Dev Reference App 证明 Core Runtime 可用但不被行业词污染；
- 增加 unlisted action、forbidden scope、missing evidence、forged provider telemetry、Software Dev term leaking into Core authority、projection trying to write authority、missing mapping 和 wrong mapping 等 negative fixtures；
- 对应 GitHub issue：#659。

依赖：#652、#653、#654、#655、#658。

### V105-010 Release Certification

状态：done

目标：

- 增加 v1.0.5 release-gate certification artifacts；
- 认证 command、admission、proposal、arbitration、file-backed registry loader、executor closeout、state writeback 和 Software Dev reference mapping；
- 记录 PR、main、tag、release event 的 gate evidence；
- 对应 GitHub issue：#660。

依赖：#651、#652、#653、#654、#655、#656、#657、#658、#659。

## Dependency Graph

```text
#651
  -> #652
    -> #653
      -> #654
        -> #655
          -> #657
            -> #658
      -> #656
        -> #657
    -> #659

#660 depends on #651-#659.
```

## Release Gate Artifacts

```text
runtime/core-runtime-kernel.json
runtime/core-runtime-admission.json
runtime/core-runtime-arbitration.json
runtime/core-runtime-negative-fixtures.json
runtime/v105-release-certification.json
```

## Certification Expectations

`v1.0.5` release gate 必须证明：

- Core Runtime command 可以使用 generic Core command fields；
- Software Dev aliases 只能通过 explicit Reference App mapping 使用；
- Runtime Admission 使用 Core Skill Registry 并阻断 unauthorized command；
- Action Proposal 不包含 Software Dev-only authority terms；
- Arbitration 使用 Core Action / State Semantics；
- file-backed ontology registry loader 被 Runtime 读取，并保留 pollution guard diagnostics；
- executor closeout 缺 evidence、越界 diff 或 invalid result 时不能进入 completion；
- state writeback 只在 accepted closeout 后发生；
- projection refresh 是派生行为，不是 authority；
- negative fixtures 覆盖 Core pollution、missing mapping、forged telemetry 和 projection writeback violation。

## Completion Standard

`v1.0.5` 只有在以下条件同时满足时才可发布：

- `cargo fmt --all --check` 通过；
- `cargo test --workspace` 通过；
- `npm --prefix apps/desktop run build` 通过；
- `git diff --check` 通过；
- `scripts/verify_release_gate.sh` 输出 Core Runtime Kernel 和 v105 certification passed；
- GitHub release-gate 在 PR、main、tag、release 四类事件均通过；
- `v1.0.5` release notes 明确说明 Software Dev 是 reference certification，不是 Core authority。
