# AgentFlow v1.0.6 Core Evidence Kernel Tasks V1

更新日期：2026-06-29
执行者：Codex

## Goal

`v1.0.6` 将 Core Runtime 的执行结果收束成可追踪、可验证、可投影的 Core Evidence Kernel。

这不是 Software Dev 产品功能扩张，而是把 evidence schema、source registry、capture receipt、trace link、completeness policy、missing evidence、external proof、Reference App mapping 和 projection read model 统一到 Core contract。

## Authority Boundary

- GitHub issues 是 planning mirror；
- `docs/delivery/releases/v1.0.6/**` 是本版本 release public record；
- `docs/architecture/060` 到 `docs/architecture/068` 是上游 Core Evidence 合同；
- `crates/ontology/src/**` 是 Core Evidence Rust 合同实现；
- `crates/projection/src/**` 只能暴露 read model；
- Software Dev Reference App mapping is not Core Evidence authority；
- provider session、GitHub issue、PR、test log 和 release note 都不是 Core Evidence authority。

## Core Evidence Flow

```text
Evidence Pack Schema
-> Source Type Registry
-> Capture Receipt
-> Authority Trace Link
-> Completeness Policy
-> Missing Evidence Handling
-> External Proof Provenance
-> Reference App Mapping
-> Projection Read Model
-> Release Certification
```

Evidence Kernel 必须能解释“证据是什么、来自哪里、证明什么、缺了什么、为什么缺失可以或不可以接受”。Projection 只能读取和聚合这些事实，不能写回 authority。

## Task Order

### V106-001 Core Evidence Pack Schema

状态：done

目标：

- 定义行业无关的 Evidence Pack；
- 明确 evidence item、artifact、receipt、trace、source、provenance 和 completeness fields；
- Software Dev diff / log / PR 只能作为 mapping；
- 对应 GitHub issue：#672。

依赖：#660。

### V106-002 Core Evidence Source Type Registry

状态：done

目标：

- 定义 source type registry；
- 覆盖 local command output、artifact file、external provider proof、human attestation、release record 等来源；
- 保证未知来源不能被当成 verified evidence；
- 对应 GitHub issue：#673。

依赖：#672。

### V106-003 Core Evidence Capture Receipts

状态：done

目标：

- 定义 capture receipt；
- 记录 capturedAt、sourceType、artifactRef、hash、capture actor、tool identity 和 immutable receipt id；
- 防止后补证据伪装成原始执行时证据；
- 对应 GitHub issue：#674。

依赖：#672、#673。

### V106-004 Core Evidence Authority Trace Links

状态：done

目标：

- 定义 evidence 到 Spec、Task、Run、Action、Decision 和 Completion Commit 的 trace link；
- trace link 必须能解释证据证明哪个 authority object；
- 不能让 projection 或 release note 成为 trace source；
- 对应 GitHub issue：#675。

依赖：#672、#674。

### V106-005 Core Evidence Completeness Policy

状态：done

目标：

- 定义 completeness policy；
- 能判断 required evidence、optional evidence、missing evidence、deferred evidence 和 invalid evidence；
- 输出稳定 failure reason；
- 对应 GitHub issue：#676。

依赖：#673、#674、#675。

### V106-006 Core Missing Evidence Handling

状态：done

目标：

- 定义 missing evidence handling；
- 缺失证据必须有 stable reason、blocking flag 和 next required action；
- blocking missing evidence 不能被 release certification 忽略；
- 对应 GitHub issue：#677。

依赖：#676。

### V106-007 Core External Proof Provenance

状态：done

目标：

- 定义 external proof provenance；
- 记录 provider、external id、URL、commit / tag / run id、retrievedAt 和 verification policy；
- fake / wrong / missing external proof 必须进入 negative fixture；
- 对应 GitHub issue：#678。

依赖：#673、#674、#675。

### V106-008 Software Dev Reference Evidence Mapping

状态：done

目标：

- 用 Software Dev Reference App 证明 Core Evidence Kernel 可用；
- 将 diff、test log、build log、browser proof、PR / release record 映射到 Core Evidence；
- 明确 Software Dev mapping 不是 Core authority；
- 对应 GitHub issue：#679。

依赖：#672、#673、#674、#675、#676、#677、#678。

### V106-009 Evidence Projection Read Model

状态：done

目标：

- 定义 Evidence Kernel read model；
- 让 Console / Desktop / Reference App 能读取 evidence completeness、missing reason、trace summary 和 projection metadata；
- read model 必须 `authority=false`；
- 对应 GitHub issue：#680。

依赖：#675、#676、#677、#679。

### V106-010 Release Certification

状态：done

目标：

- 增加 v1.0.6 release-gate certification artifact；
- 认证 schema、source registry、capture receipt、traceability、completeness policy、missing evidence、external proof、Software Dev mapping 和 projection evidence；
- 记录 fake / missing / wrong evidence negative fixture certification；
- 对应 GitHub issue：#681。

依赖：#672、#673、#674、#675、#676、#677、#678、#679、#680。

## Dependency Graph

```text
#672
  -> #673
    -> #674
      -> #675
        -> #676
          -> #677
        -> #678
          -> #679
            -> #680

#681 depends on #672-#680.
```

## Release Gate Artifacts

```text
runtime/core-evidence-pack-schema.json
runtime/core-evidence-source-type-registry.json
runtime/core-evidence-capture-receipts.json
runtime/core-evidence-authority-trace-links.json
runtime/core-evidence-completeness-policy.json
runtime/core-missing-evidence-handling.json
runtime/core-external-proof-provenance.json
runtime/software-dev-reference-evidence-mapping.json
runtime/evidence-projection-read-model.json
runtime/v106-release-certification.json
```

## Certification Expectations

`v1.0.6` release gate 必须证明：

- Evidence Pack Schema 是 Core authority；
- source type registry 阻断 unknown / unsupported source；
- capture receipt 记录 immutable evidence capture；
- trace link 指向 authority object，不指向 projection；
- completeness policy 能区分 required / optional / missing / deferred / invalid；
- missing evidence 有 stable reason 和 blocking flag；
- external proof provenance 能识别 fake / missing / wrong proof；
- Software Dev evidence 只能通过 Reference App mapping 进入；
- Evidence Projection Read Model 只读，不能写回 authority；
- negative fixtures 覆盖 fake evidence、missing evidence、wrong external proof 和 projection authority violation。

## Remaining Risks

以下内容不阻断 `v1.0.6`，但必须带到后续版本：

- `v1.0.7`：Decision Kernel 负责把 Spec + Evidence 转成 completion decision；
- `v1.0.8`：Projection Kernel 负责多视图投影和 rebuild guarantee；
- 后续 Software Dev Reference App closeout：完整 UI / PR / release 操作体验。

## Completion Standard

`v1.0.6` 只有在以下条件同时满足时才可发布：

- `cargo fmt --all --check` 通过；
- `cargo test --workspace` 通过；
- `npm --prefix apps/desktop run build` 通过；
- `git diff --check` 通过；
- `scripts/verify_release_gate.sh` 输出 Core Evidence Kernel 和 v106 certification passed；
- GitHub release-gate 在 PR、main、tag、release 四类事件均通过；
- `v1.0.6` release notes 明确说明 Software Dev 是 reference evidence mapping，不是 Core Evidence authority。
