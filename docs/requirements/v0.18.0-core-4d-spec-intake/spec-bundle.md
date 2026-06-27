# v0.18.0 Core 4-D Spec Intake Spec Bundle

更新日期：2026-06-28
执行者：Codex
状态：confirmed Spec Bundle

## Purpose

本 Spec Bundle 定义 Core Spec Intake 的 4-D 合同：

```text
Deconstruct -> Diagnose -> Develop -> Deliver
```

它回答：

```text
Core 如何把原始人类请求变成可确认、可物化、可映射到行业 App 的 Spec Bundle？
```

## Source Issues

本 Bundle 覆盖 GitHub planning mirror：

| Issue | 目标 |
| --- | --- |
| `#618` | Core 4-D Spec Intake Contract |
| `#619` | Intent Packet + Deconstruct Schema |
| `#620` | Diagnose Gap Model + Core Route Policy |
| `#621` | Clarify Interaction Contract |
| `#622` | Research Evidence Contract |
| `#623` | Spec Bundle Slice Contract |
| `#624` | Industry Mapping Contract |
| `#625` | Materialization Boundary |
| `#626` | Cross-industry Reference Fixtures |
| `#627` | v0.18.0 Release Certification |

GitHub issue 只是 planning mirror。本文和对应架构合同才是人类可读 confirmed Spec。

## Core Boundary

Core 可以使用的通用概念：

```text
Intent
Domain
Goal
Plan
Task
Decision
Output
Feedback
Route
Spec Bundle
Artifact
Evidence
```

Core 不能把 Software Dev 概念写成必需内核概念：

```text
bug
feature
PR
release
patch
test log
repository
GitHub issue
```

Software Dev、UI Design、Video Production 只能作为 reference mapping fixture。

## 4-D Stage Contract

| Stage | 输入 | 输出 | 状态边界 | 不允许 |
| --- | --- | --- | --- | --- |
| Deconstruct | raw human request, source references | intent packet | Draft | 直接物化、直接执行 |
| Diagnose | intent packet, context artifacts | gap model, route policy | Draft | 把行业对象当 Core authority |
| Develop | route policy, draft spec bundle | preview artifact, confirmation request | Preview | 未确认写入 authority |
| Deliver | confirmed preview, confirmation record | confirmed spec bundle, materialized runtime contract | Materialized | 跳过 confirmation |

## Intent Packet

Deconstruct 阶段必须把原始输入拆成：

- intent；
- domain；
- goal；
- constraints；
- context；
- expected output；
- source references。

Intent Packet 是 Draft，不是 authority。

## Diagnose Gap Model

Diagnose 阶段识别两类缺口：

- human decision gap：需要人确认、取舍、补充；
- fact gap：需要查证事实、来源、证据。

Core Route Policy 必须至少支持：

```text
clarify
research
define
plan
task
decide
deliver
evolve
```

## Clarify Interaction Contract

Clarify 只处理 human decision gap：

- 问题必须绑定具体缺口；
- 回答只能更新 Draft / Preview；
- 不能直接写 Materialized authority；
- 回答后必须重新经过 route policy。

## Research Evidence Contract

Research 只处理 fact gap：

- research question；
- source；
- confirmed fact；
- assumption；
- risk；
- Spec impact；
- recommended route。

Research Evidence 可以支撑后续 Preview，但不能替代 Confirmation。

## Spec Bundle Slices

Core Spec Bundle 固定 8 个 slice：

```text
Intent
Domain
Goal
Plan
Task
Decision
Output
Feedback
```

这些 slice 是行业无关的 Core authority。行业 App 可以映射它们，但不能改变 Core slice 定义。

## Materialization Boundary

状态边界：

```text
Draft
-> Preview
-> Confirmed
-> Materialized
```

规则：

- Draft 可以重写；
- Preview 可以给人确认；
- Confirmed 代表人类确认；
- Materialized 才能进入 `.agentflow/spec/**` 机器合同；
- 未确认 Preview 不能进入 authority。

## Industry Mapping

Reference mappings：

| Industry | Core slice | 映射说明 |
| --- | --- | --- |
| Software Dev | Task / Decision / Output | 映射到软件开发任务、验证证据、完成判定 |
| UI Design | Output / Decision | 映射到页面、组件、视觉预览和设计确认 |
| Video Production | Plan / Output | 映射到分镜、镜头计划、渲染预览和剪辑决策 |

这些 mapping 只证明 Core 可以跨行业使用，不是 Core authority。

## Runtime Contract

本 Bundle 对应的代码合同位于：

```text
crates/spec/src/core_intake.rs
```

Release gate 必须证明：

- 4-D 阶段顺序完整；
- 8 个 Core slices 完整；
- 8 条 routes 完整；
- Draft / Preview / Confirmed / Materialized 边界完整；
- Software Dev / UI Design / Video Production reference fixtures 存在；
- Core authority 不依赖 Software Dev-only terms。

## Acceptance

完成标准：

- `agentflow-spec` 暴露 Core 4-D Intake 合同；
- 单元测试证明合同有效且拒绝 Core authority 中的行业硬编码；
- `scripts/verify_release_gate.sh` 生成 `runtime/core-4d-spec-intake.json`；
- release gate 能证明 #618 到 #627 的覆盖；
- GitHub issues #618 到 #627 可由本 Bundle 对应 PR 关闭。
