# Project Roadmap

更新日期：2026-07-01
执行者：Codex

## Purpose

本文定义 AgentFlow 从当前目标走到可用产品的版本路线图。

`goal.md` 回答：

```text
AgentFlow 要成为什么？
```

`roadmap.md` 回答：

```text
为了达到这个目标，需要分几个阶段完成？
每个阶段解决什么 Core OS 能力？
什么时候用 Software Dev Reference App 做认证？
```

本文不是执行任务，不直接授权写源码，也不直接生成 `.agentflow/spec/**`。

## Current Goal

当前目标已经修正为：

```text
AgentFlow = Spec-Driven AI OS Project
```

一句话：

```text
Agent 只是执行器，Spec 才是方向盘。
Core OS 提供底层能力，行业是运行在 Core 上的 Product Surface。
```

AgentFlow 后续版本必须先把 Core OS Runtime 的通用能力稳住。Software Dev 是第一个官方 Reference App，用来证明 Core 可运行，不得被写死进 Core。

商业产品目标是卖可交付结果，不是先卖 Agent。行业层可以是 Paid Report Flow、Managed Project Flow 或 Assistant / Ops Flow。

## System Formula

AgentFlow 的系统公式是：

```text
AgentFlow AI OS Project
= Core OS Runtime
+ Industry Product Surface
```

其中：

```text
Core OS Runtime
= Spec Kernel
+ Ontology Kernel
+ Runtime Kernel
+ Evidence Kernel
+ Decision Kernel
+ Projection Kernel
```

```text
Industry Product Surface
= Paid Report Flow
or Managed Project Flow
or Assistant / Ops Flow
```

每个 Industry Product Surface 由内置 Domain Pack、Surface Pack 和 Connector Pack 支撑：

```text
Industry Product Surface
= Domain Pack
+ Surface Pack
+ Connector Pack
```

源码目录边界必须保持：

```text
crates/**     = Core OS Runtime
products/**   = Industry Product Surface / Reference App source
apps/**       = user-facing clients
docs/**       = human-readable records
.agentflow/** = Runtime fact source
```

详细边界以 [../architecture/086-industry-product-source-boundary-v1.md](../architecture/086-industry-product-source-boundary-v1.md) 为准。

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

`v1.0.2` 完成后，后续版本不直接进入 `v1.1.0`。

`v1.0.3` 到 `v1.0.8` 先完成 Core OS Runtime 的 6 个 Kernel。

`v1.0.9` 再用 Software Dev Reference App 做端到端认证，并冻结第一个行业壳的源码边界。

`v1.1.0` 先做 Product Surface Hardening，把 `products/software-dev/**` 接入 Product Registry、Runtime command route 和 Projection source。

| Version | Goal | Primary loop / kernel |
| --- | --- | --- |
| `v1.0.2` | Release hardening closeout | Release / Governance |
| `v1.0.3` | Core Spec Kernel / Spec Bundle Workspace | Spec Loop |
| `v1.0.4` | Core Ontology Kernel | Ontology / Pack boundary |
| `v1.0.5` | Core Runtime Kernel | Runtime / Issue Loop |
| `v1.0.6` | Core Evidence Kernel | Evidence |
| `v1.0.7` | Core Decision Kernel | Decision |
| `v1.0.8` | Core Projection Kernel | Projection |
| `v1.0.9` | Software Dev Reference App / Industry Shell Certification | Project Loop / Delivery |
| `v1.1.0` | Product Surface Hardening | Product Surface / Runtime / Projection |

## Version Intent

### v1.0.2 - Release Hardening Closeout

关闭 `v1.0.1` 审计后留下的 hardening gap：

- trusted governance telemetry；
- lightweight tag provenance；
- negative release fixtures；
- product goal baseline；
- v1.0.2 release certification。

`v1.0.2` 是进入 Core Kernel 收敛前的稳定地基。

### v1.0.3 - Core Spec Kernel / Spec Bundle Workspace

把人类意图变成可确认、可物化、可派生任务的 Spec Bundle。

核心问题：

```text
用户输入如何成为 AgentFlow 的需求总合同？
```

本版应该定义：

- Core Spec Bundle contract；
- intent intake；
- route policy；
- slice contract；
- confirmation / materialization boundary；
- app-specific slice mapping boundary；
- Spec Bundle projection。

本版必须避免把 Software Dev 的 `Issue / PR / Release / Patch` 写死进 Spec Kernel。Software Dev 只能作为 fixture 或 Reference App mapping。

`v1.0.3` 后续再单独拆 tasks。本文不提前拆 `V103-*`。

### v1.0.4 - Core Ontology Kernel

把 Spec Bundle 映射到行业无关的项目世界。

Core 对象应优先收敛为：

```text
Object
Link
Action
State
Skill
Artifact
Evidence
Decision
Projection
```

Software Dev 对象应通过 Domain Pack 映射：

```text
Requirement
Spec
Issue
Run
PR
Release
Feedback
```

本版目标是让 Core 知道项目世界如何定义，但不把某个行业的对象固定成唯一内核。

### v1.0.5 - Core Runtime Kernel

把 confirmed Spec / Task 变成受控执行。

本版目标：

- Runtime command；
- Runtime admission；
- Action Proposal；
- Arbitration；
- allowed / forbidden surface；
- retry / cancel / failed / blocked；
- executor adapter closeout；
- task / run 状态写回。

Runtime command 解析必须逐步从 hardcoded command mapping 迁移到 Core contract + App Pack mapping。

### v1.0.6 - Core Evidence Kernel

把执行结果变成可追踪证据。

本版目标：

- Evidence Pack schema；
- artifact / log / screenshot / external proof / provenance；
- evidence trace to Spec / Task / Run；
- missing evidence handling；
- evidence completeness policy。

Software Dev 的 diff / test log / build log / PR link 只是 Evidence 的一种行业映射。

### v1.0.7 - Core Decision Kernel

把 Evidence 和 Spec 合同变成完成判定。

本版目标：

- decision model；
- accepted / rejected / deferred / blocked；
- failure reason；
- needs-fix；
- completion commit；
- delivery readiness；
- audit sidecar trigger evaluation。

Audit 仍然是独立 sidecar，不回到主业务链。

### v1.0.8 - Core Projection Kernel

把 Spec、任务、证据、Decision 和 Delivery 投影给人类和系统读取。

本版目标：

- projection API；
- read model；
- view model；
- pack-specific projection mapping；
- invalid / missing app definition handling；
- feedback surface projection。

Projection 只读，不是 authority。

Industry Surface 只能消费 Projection，不能直接读取或修改 Core authority facts。

### v1.0.9 - Software Dev Reference App Certification

证明 Software Dev Reference App 可以在 Core OS Runtime 上跑完整闭环：

```text
Intent
-> Spec Bundle
-> Software Dev Domain Mapping
-> Issues / Runs
-> Agent Execution
-> Evidence
-> Decision
-> Delivery
-> Feedback
-> Spec Evolution
```

本版目标是 certification，不扩新行业，不做 marketplace，不引入默认 Message Bus。

本版也必须证明第一个行业壳不污染 Core：

```text
Core OS Runtime
+ products/software-dev
= Software Dev Reference App
```

`v1.0.9` 应引入或认证：

- `products/software-dev/**` 作为 first-party Reference App source；
- Domain Pack / Surface Pack / Connector Pack 通过 Pack contract 接入 Core；
- `crates/pack/fixtures/**` 只保留为测试夹具；
- `crates/**` 不把 `Issue / PR / Release / Patch / GitHub / Codex / Claude` 写成 Core authority；
- release certification 能追溯 Product source -> Runtime facts -> Evidence -> Decision -> Delivery。

### v1.1.0 - Product Surface Hardening

`v1.1.0` 不是 Software Dev commercial beta。

只有当 `v1.0.3` 到 `v1.0.9` 完成后，`v1.1.0` 才能把 Product Surface 接到 Runtime / Projection 消费链：

```text
products/software-dev/**
-> Product Registry
-> Product to Pack Contract
-> Runtime API command routes
-> Projection read models
```

目标是让 Product source 成为 Runtime / Projection 的真实输入，而不是继续依赖 fixture mirror 或内置 fallback。

本版应该回答：

- `products/software-dev/product.toml` 和阶段入口是否可被稳定读取；
- Product command 如何映射到现有 Pack / Runtime action contract；
- Runtime API 是否优先从 Product source 解析命令；
- Projection 是否从 Product source 生成 read model；
- 缺少 Product / Pack source 时是否进入 invalid / deferred，而不是注入内置 fallback；
- release gate 是否能证明 Product Surface 没有污染 Core authority。

商业 Product Beta、安装器、完整行业壳 UI 和外部市场化交付仍属于后续版本。

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

GitHub issues 可以作为外部同步视图，但不能成为 AgentFlow authority。
