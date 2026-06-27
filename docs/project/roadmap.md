# Project Roadmap

更新日期：2026-06-26
执行者：Codex

## Purpose

本文定义 AgentFlow 从当前产品目标走到可用商业产品的版本路线图。

`goal.md` 回答：

```text
AgentFlow 要成为什么？
```

`roadmap.md` 回答：

```text
为了达到这个目标，需要分几个阶段完成？
每个阶段解决什么能力？
什么时候可以拆成 confirmed Spec 和 tasks？
```

本文不是执行任务，不直接授权写源码，也不直接生成 `.agentflow/spec/**`。

## Current Goal

当前目标已经固定为：

```text
AgentFlow = Spec-Driven Software Dev Workflow
```

一句话：

```text
Agent 只是执行器，Spec 才是方向盘。
```

AgentFlow 当前商业产品只聚焦 Software Dev。底层 AI OS Project 能力保留通用性，但后续版本必须优先服务软件开发主链。

## Planning Chain

AgentFlow 的项目规划链路是：

```text
docs/project/goal.md
-> docs/project/roadmap.md
-> docs/requirements/<version-or-slice>.md
-> .agentflow/spec/projects/**
-> .agentflow/spec/issues/**
-> .agentflow/tasks/**
```

每层职责不同：

| Layer | Responsibility |
| --- | --- |
| `goal.md` | 固定长期产品目标 |
| `roadmap.md` | 把目标拆成版本阶段和能力顺序 |
| `docs/requirements/**` | 保存 confirmed Spec Bundle |
| `.agentflow/spec/projects/**` | 保存 Runtime 可读的项目执行合同 |
| `.agentflow/spec/issues/**` | 保存 Runtime 可读的任务执行合同 |
| `.agentflow/tasks/**` | 保存单个任务的运行、证据和状态事实 |

## Core Loop Model

AgentFlow 后续版本围绕四个 loop 推进：

| Loop | Owns | Does not own |
| --- | --- | --- |
| Project Loop | 目标、路线图、版本范围、任务集合、发布结果 | 不直接写代码 |
| Spec Loop | 把人类意图变成 confirmed Spec Bundle | 不直接执行任务 |
| Issue Loop | 单个任务执行、证据、验收、状态回写 | 不擅自改变版本目标 |
| Feedback Loop | 把交付反馈带回 Spec / Roadmap | 不绕过确认 |

Loop 之间不能互相越权：

- Issue Loop 发现目标不对，只能发起 feedback，不能直接改 roadmap。
- Feedback Loop 只能提出 update proposal，不能直接绕过 confirmation。
- Spec Loop 只能生成 confirmed Spec，不能直接执行。
- Project Loop 只能调度任务集合，不能跳过 Issue Loop 标记任务完成。

这些 loop 的细则后续需要分别沉淀成合同文档。本文只定义版本推进顺序。

## Roadmap Summary

`v1.0.2` 完成后，后续版本不直接进入 `v1.1.0`。先用 `v1.0.3` 到 `v1.0.9` 把 Software Dev 的 Spec-Driven Workflow 闭环补完整。

| Version | Goal | Primary loop / kernel |
| --- | --- | --- |
| `v1.0.2` | Release hardening closeout | Release / Governance |
| `v1.0.3` | Spec Kernel / Spec Bundle Workspace | Spec Loop |
| `v1.0.4` | Software Dev Ontology Kernel | Project Loop / Ontology |
| `v1.0.5` | Runtime Execution Loop | Issue Loop / Runtime |
| `v1.0.6` | Evidence Kernel | Issue Loop / Evidence |
| `v1.0.7` | Decision Kernel | Issue Loop / Decision |
| `v1.0.8` | Projection / Product Surface | Projection |
| `v1.0.9` | End-to-end Software Dev Workflow Certification | Project Loop / Delivery |
| `v1.1.0` | Software Dev Product Beta | Product release |

## Version Intent

### v1.0.2 - Release Hardening Closeout

关闭 `v1.0.1` 审计后留下的 hardening gap：

- trusted governance telemetry；
- lightweight tag provenance；
- negative release fixtures；
- product goal baseline；
- v1.0.2 release certification。

`v1.0.2` 是进入产品闭环开发前的稳定地基。

### v1.0.3 - Spec Kernel / Spec Bundle Workspace

把人类意图变成可确认、可执行的 Spec Bundle。

核心问题：

```text
用户输入如何成为 AgentFlow 的需求总合同？
```

本版应该定义：

- Spec Bundle contract；
- intent intake；
- route policy；
- PRD / Architecture Plan / Task Slice 的关系；
- issue derivation rule；
- confirmation / materialization boundary；
- Spec Bundle projection。

`v1.0.3` 后续再单独拆 tasks。本文不提前拆 `V103-*`。

### v1.0.4 - Software Dev Ontology Kernel

把 Spec Bundle 映射到 Software Dev 项目世界。

核心对象：

```text
Requirement
Goal / PRD
Spec
Architecture Plan
Issue
Run
Evidence
Decision
PR
Release
Feedback
```

本版目标是让 AgentFlow 清楚知道项目里有什么对象、对象之间如何连接、生命周期如何推进。

### v1.0.5 - Runtime Execution Loop

把 confirmed Spec Issue 变成受控执行。

本版目标：

- Work Handoff；
- Runtime Admission；
- Action Proposal；
- allowed / forbidden surface；
- retry / cancel / failed / blocked；
- executor adapter closeout；
- Issue Loop 状态写回。

### v1.0.6 - Evidence Kernel

把执行结果变成可追踪证据。

本版目标：

- Evidence Pack schema；
- diff / test log / build log / screenshot / PR / release note；
- evidence trace to Spec / Issue / Run；
- missing evidence handling；
- evidence completeness policy。

### v1.0.7 - Decision Kernel

把 Evidence 和 Spec 合同变成 Done 判定。

本版目标：

- acceptance decision model；
- failure reason；
- needs-fix / rejected / deferred；
- completion commit；
- delivery readiness；
- Audit sidecar trigger evaluation。

Audit 仍然是独立 sidecar，不回到主业务链。

### v1.0.8 - Projection / Product Surface

把 Spec、任务、证据、Decision 和 Delivery 展示给人类。

本版目标：

- Project Home；
- Spec Workbench；
- Task Workbench；
- Evidence Graph；
- Decision Gate view；
- Delivery Surface；
- Feedback Surface。

Projection 只读，不是 authority。

### v1.0.9 - End-to-end Software Dev Workflow Certification

证明 Software Dev 主链完整可跑：

```text
Intent
-> Spec Bundle
-> Plan / Issues
-> Agent Execution
-> Evidence
-> Decision
-> Delivery
-> Feedback
-> Spec Evolution
```

本版目标是 release certification，不扩新行业，不做 marketplace，不引入默认 Message Bus。

### v1.1.0 - Software Dev Product Beta

`v1.1.0` 不是底层补丁版。

只有当 `v1.0.3` 到 `v1.0.9` 完成后，`v1.1.0` 才能定义为：

```text
Software Dev Product Beta
```

目标是让一个团队可以真实使用 AgentFlow 跑完整软件开发工作流。

## Task Derivation Rule

任何版本任务必须从 roadmap 进入 confirmed Spec Bundle，再进入 `.agentflow/spec/**`。

规则：

```text
roadmap version intent
-> docs/requirements/<version>.md
-> SPEC project
-> SPEC issues
-> Issue Loop execution
```

GitHub issues 可以作为临时 planning mirror，但不能成为 AgentFlow authority。

## Non-goals

当前路线图不做：

- 新行业壳；
- Pack marketplace；
- 多租户云平台；
- 默认中心化 Message Bus；
- 把 Audit 放回主业务链；
- 把 GitHub issues 作为 authority；
- 把 executor session 当成项目事实源；
- 在 `roadmap.md` 中提前拆 `v1.0.3` tasks。

## Update Rule

`roadmap.md` 可以随着项目推进更新，但必须满足：

- 更新必须服务 `Spec-Driven Software Dev Workflow` 目标；
- 更新不能绕过 confirmed Spec；
- 版本范围变化必须说明原因；
- 已进入 `.agentflow/spec/**` 的执行合同不能被 roadmap 静默改写；
- Feedback 进入 roadmap 前必须经过确认。
