# 053 - Core 4-D Spec Intake Kernel V1

创建日期：2026-06-28
执行者：Codex
状态：stable architecture contract

## Purpose

本文定义 Core Spec Kernel 的 intake 合同。

Core intake 使用 4-D 方法：

```text
Deconstruct -> Diagnose -> Develop -> Deliver
```

它的职责是把原始人类请求变成可确认、可物化、可映射到行业 App 的 Spec Bundle。

## Authority

本合同的 confirmed Spec Bundle 位于：

```text
docs/requirements/v0.18.0-core-4d-spec-intake/spec-bundle.md
```

机器可验证合同位于：

```text
crates/spec/src/core_intake.rs
```

## Core Boundary

Core 可以使用的概念：

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

Core 不允许把这些 Software Dev 概念写成必需内核概念：

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

Software Dev 可以作为 Reference App mapping 出现，但不能成为 Core authority。

## 4-D Contract

| Phase | Owns | Output boundary |
| --- | --- | --- |
| Deconstruct | raw input to intent packet | Draft |
| Diagnose | gap model and route policy | Draft |
| Develop | draft bundle to preview and confirmation request | Preview |
| Deliver | confirmed preview to materialized runtime contract | Materialized |

## Core Routes

Core route policy 固定 8 条通用路线：

| Route | Trigger |
| --- | --- |
| clarify | human decision gap |
| research | fact gap |
| define | goal gap |
| plan | sequencing gap |
| task | actionable work gap |
| decide | acceptance gap |
| deliver | output gap |
| evolve | feedback gap |

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

行业 App 只能把这些 slice 映射到自己的对象和动作，不能改变 Core slice 定义。

## Materialization Boundary

Core authority 必须遵守：

```text
Draft -> Preview -> Confirmed -> Materialized
```

Rules:

- Draft 可以重写；
- Preview 可展示给人类确认；
- Confirmed 必须绑定具体 preview 和 confirmation record；
- Materialized 才能进入 `.agentflow/spec/**`；
- 未确认 preview 不得写入 authority。

## Reference Mappings

本合同要求至少保留三类 reference mapping fixture：

| Reference App | Purpose |
| --- | --- |
| Software Dev | 证明软件开发任务只是行业映射，不是 Core |
| UI Design | 证明设计输出和确认也能映射到 Core |
| Video Production | 证明非软件生产流程也能映射到 Core |

## Release Gate

`scripts/verify_release_gate.sh` 必须生成：

```text
runtime/core-4d-spec-intake.json
```

该 artifact 至少证明：

- 4-D 阶段顺序完整；
- 8 个 routes 完整；
- 8 个 slices 完整；
- Draft / Preview / Confirmed / Materialized 边界完整；
- 三类 reference mapping fixture 存在；
- Core authority 不依赖 Software Dev-only terms。

## Non-goals

- 不实现行业 Pack 的完整 runtime；
- 不把 GitHub issue 当 AgentFlow authority；
- 不在本合同里启动 Work Loop；
- 不写 `.agentflow/**` runtime facts；
- 不让 Audit 回到主业务链。
