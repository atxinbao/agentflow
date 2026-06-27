# AgentFlow v1.0.3 Core 4-D Spec Intake

更新日期：2026-06-28
执行者：Codex

## Status

`v1.0.3` 是 `v1.0.2` 发布后的 Core Spec Kernel baseline。

本版本不启动 `v1.1` 产品功能，不改变 `v1.0.2` 的 release audit
修复事实。它把 Core 4-D Spec Intake 作为后续 Spec Bundle Workspace
的入口合同落成，并由 release gate 认证。

## Scope

`v1.0.3` 收口以下内容：

1. Core 4-D 阶段合同：Deconstruct、Diagnose、Develop、Deliver。
2. Core Spec Bundle slice：Intent、Domain、Goal、Plan、Task、Decision、Output、Feedback。
3. 路由策略：clarify、research、define、plan、task、decide、deliver、evolve。
4. 物料化边界：Draft、Preview、Confirmed、Materialized。
5. 跨行业映射 fixture：Software Dev、UI Design、Video Production。
6. Release gate 生成 `runtime/v103-release-fix-certification.json` 和
   `runtime/core-4d-spec-intake.json`。

## Closeout Artifacts

Release gate 必须生成：

- `runtime/v103-release-fix-certification.json`
- `runtime/core-4d-spec-intake.json`
- `runtime/core-4d-spec-intake-rust-test.log`

## Public Records

- [../../../requirements/v0.18.0-core-4d-spec-intake/spec-bundle.md](../../../requirements/v0.18.0-core-4d-spec-intake/spec-bundle.md)
- [../../../requirements/v0.18.0-core-4d-spec-intake/delivery.md](../../../requirements/v0.18.0-core-4d-spec-intake/delivery.md)
- [../../../architecture/053-core-4d-spec-intake-kernel-v1.md](../../../architecture/053-core-4d-spec-intake-kernel-v1.md)

## Non-goals

- 不启动 `v1.1` 功能；
- 不实现行业壳；
- 不引入新的 Message Bus；
- 不改变 `v1.0.2` 历史 release fact；
- 不把 GitHub issue 当成 AgentFlow authority。

