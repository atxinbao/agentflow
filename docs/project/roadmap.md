# Project Roadmap

更新日期：2026-07-02
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

`v1.1.3` 完成后，Product Command Surface 应从 dry-run 进入受控 submit。它仍不是商业发布终点，而是 Product Surface 能安全驱动 Runtime 的起点。

产品化目标分两段：

```text
v1.1.9 = 可内测产品
v1.2.3 / v1.2.4 = 可公开发布产品
v1.2.5 / v1.2.6 / v1.2.7 = Commercial Runtime 公开发布后的项目级交接收口
```

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
| `v1.1.1` | Product Contract Data-driven Hardening | Product Surface / Pack bridge |
| `v1.1.2` | Product Execution Proof and Command Surface Hardening | Runtime / Projection / Desktop proof |
| `v1.1.3` | Product Command Submission and State Semantics | Runtime submit / Command state |
| `v1.1.4` | Project Creation and Product Workspace | Project Loop / Workspace |
| `v1.1.5` | Spec Intake to Goal / Roadmap / Task Productization | Spec Loop / Project Loop |
| `v1.1.6` | Executor Adapter Real Execution Closure | Executor adapter / Issue Loop |
| `v1.1.7` | Evidence / Decision / Delivery User Readability | Evidence / Decision / Delivery |
| `v1.1.8` | Recovery / Resume / Failure Handling | Runtime recovery / Issue Loop |
| `v1.1.9` | Software Dev Reference App Beta Certification | Beta certification |
| `v1.2.0` | Product Onboarding and First-run Experience | Product onboarding |
| `v1.2.1` | Team Workflow and Project Sharing | Collaboration |
| `v1.2.2` | Commercial Boundary / License / Usage Model | Commercial boundary |
| `v1.2.3` | Public Release Candidate Certification | Public release readiness |
| `v1.2.4` | Public Product Release Closeout | Public product release |
| `v1.2.5` | Published Release Certification and Registry-backed Commercial Runtime | Release certification / Commercial registry |
| `v1.2.6` | Project-scoped Commercial Product Instance Hardening | Product instance / Project registry |
| `v1.2.7` | Paid Report Runtime Handoff Closure | Paid Report handoff / Runtime admission |

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

### v1.1.1 - Product Contract Data-driven Hardening

`v1.1.1` 继续 `v1.1.0` 的 Product Surface hardening，但收口点更窄：

```text
Product source command mapping
-> Product-to-Pack bridge
-> Runtime resolver
-> Projection read model
-> runtime/projection proof artifacts
```

本版确认：

- Product source 显式声明 commandId、runtimeCommand、actionContractRef、targetObjectType、pageId、skillRef、connectorId、requiredCapability、evidencePolicyRef 和 acceptancePolicyRef；
- Product-to-Pack bridge 不再通过 Software Dev 命令名硬编码推断 action contract 或对象类型；
- Runtime resolver 不再通过 Software Dev 命令名硬编码推断 page、skill、connector 或 capability；
- Projection 不再通过 Software Dev 专用 helper 推断 workbench / connector / evidence 语义；
- `products/_fixtures/synthetic-review/**` 作为第二个 Product 测试夹具证明桥接链路是通用的；
- release gate 产出 v1.1.1 runtime / projection proof artifacts。

UI command route installation、多 Product console、Product installer 和行业壳市场化仍属于后续版本。

### v1.1.2 - Product Execution Proof and Command Surface Hardening

`v1.1.2` 继续 `v1.1.1` 的 Product Contract Data-driven hardening，但把证明从静态合同推进到真实 Runtime / Projection / Desktop command surface：

```text
products/**
-> Runtime API validate / dry-run
-> Projection read model
-> Desktop Product Command Surface
-> release-gate proof artifacts
```

本版确认：

- Runtime proof harness 必须调用 `validate_pack_command` 和 `dry_run_pack_command`，不能手写 positive / negative JSON；
- Projection proof harness 必须调用 Product read model API；
- `products/synthetic-review/**` 作为直接 Product registry entry，证明第二个 Product 能被标准路径发现；
- Desktop 从 Runtime API 加载 Product command route read model，并在按钮点击时先执行 dry-run；
- 多 Product console 可以展示 valid / invalid / deferred 状态，不回落到 Software Dev；
- recursive pollution scanner 覆盖 Product bridge 相关 crates；
- release gate 产出 v1.1.2 execution / projection / desktop / multi-product proof artifacts。

Product command authority submission、Product installer、marketplace 和完整行业壳 UI 仍属于后续版本。

### v1.1.3 - Product Command Submission and State Semantics

`v1.1.3` 把 Product Command Surface 从 dry-run 证明推进到受控 submit：

```text
Product Command dry-run
-> explicit command state
-> human confirmation
-> Runtime submit
-> arbitration / decision
-> evidence handoff
```

本版确认：

- Product command state 明确区分 valid / invalid / deferred / unavailable / rejected；
- capability unavailable 不能再被 UI 简化成 invalid；
- Product command submit 必须经过 Runtime API、governance、arbitration 和 evidence handoff；
- Desktop 只能走 confirm-then-submit，不能绕过 dry-run 或 Runtime admission；
- release gate 必须包含真实 submit positive / negative proof；
- Product bridge pollution scanner 升级成语义扫描，不只依赖窄 token 列表。

本版完成后，AgentFlow 具备安全触发 Product Runtime command 的基础能力，但还不是可销售产品。

### v1.1.4 - Project Creation and Product Workspace

`v1.1.4` 让用户可以从产品入口创建一个真实 AgentFlow project workspace。

本版目标：

- 新建项目时生成标准 `docs/project/**` 和 `.agentflow/**` 运行空间；
- Product source 决定项目类型和默认 surface；
- Project Loop 能读取 goal、roadmap、active product 和 workspace 状态；
- Software Dev Reference App 可以作为默认 Product workspace 启动；
- workspace 初始化必须有 receipt、projection 和失败恢复。

本版不做复杂团队协作，不做商业收费。

### v1.1.5 - Spec Intake to Goal / Roadmap / Task Productization

`v1.1.5` 把用户输入产品化成 Spec-driven 入口。

本版目标：

- Intent intake 进入 Spec Bundle；
- Spec Bundle 派生 goal / roadmap / tasks；
- clarify / research / define / plan / task / decide / deliver / evolve 路由可被 Product 使用；
- 用户确认前不写 authority；
- 确认后 materialize 到 Runtime 可读合同；
- Software Dev Reference App 能从需求生成可执行任务集合。

本版重点是输入闭环，不是强化执行器。

### v1.1.6 - Executor Adapter Real Execution Closure

`v1.1.6` 把 Codex / Claude Code 等执行器真正纳入受控 Issue Loop。

本版目标：

- executor adapter contract；
- handoff、allowed surface、expected outputs、evidence policy；
- 执行前准入，执行后 diff / evidence / status 校验；
- provider failure、timeout、cancel、retry 的最小闭环；
- 不把 executor session 当作 AgentFlow authority。

本版完成后，Software Dev Reference App 应能跑真实 Agent 执行链，而不是只证明 command surface。

### v1.1.7 - Evidence / Decision / Delivery User Readability

`v1.1.7` 把底层事实变成用户看得懂的交付状态。

本版目标：

- evidence graph 对用户可读；
- Decision reason 可解释；
- Delivery package 包含结果、证据、限制和下一步；
- rejected / needs-fix / deferred 有明确修复路径；
- Audit 仍保持 sidecar，不回到主业务链。

本版完成后，用户能判断“为什么 Done 或为什么不能 Done”。

### v1.1.8 - Recovery / Resume / Failure Handling

`v1.1.8` 处理真实使用中最常见的断点。

本版目标：

- run resume；
- failed command recovery；
- stale projection rebuild；
- interrupted executor session closeout；
- duplicate command / idempotency handling；
- workspace health check；
- release gate 覆盖恢复场景负向夹具。

没有恢复能力，产品不能进入真实团队内测。

### v1.1.9 - Software Dev Reference App Beta Certification

`v1.1.9` 是第一个可内测产品基线。

本版目标：

```text
create project
-> intake spec
-> derive tasks
-> run executor
-> collect evidence
-> decision
-> delivery
-> feedback
```

本版必须证明：

- Software Dev Reference App 不是 Core pollution；
- 一个真实小型开发任务可以完成闭环；
- 失败、重试、交付和反馈都有可读 projection；
- release artifact 可以支撑快速审计。

`v1.1.9` 可以进入受控内测，但不等于公开商业发布。

### v1.2.0 - Product Onboarding and First-run Experience

`v1.2.0` 开始从内测产品走向真实用户。

本版目标：

- first-run project setup；
- Product selection；
- workspace readiness check；
- provider / connector readiness；
- sample project / guided run；
- 用户不需要理解 `.agentflow/**` 也能开始使用。

本版不应扩新行业，仍围绕 Software Dev Reference App。

### v1.2.1 - Team Workflow and Project Sharing

`v1.2.1` 支撑团队场景。

本版目标：

- project sharing boundary；
- role / permission / handoff view；
- team-readable delivery and decision history；
- 多人反馈进入 Feedback Loop；
- 不把权限逻辑塞进单个行业 Product。

本版可以先做本地或轻量协作，不急着做完整云多租户。

### v1.2.2 - Commercial Boundary / License / Usage Model

`v1.2.2` 定义商业发布前的使用边界。

本版目标：

- license / entitlement / usage boundary；
- paid feature boundary；
- paid report flow 与 managed project flow 的产品边界；
- telemetry / support evidence 的最小合同；
- delivery / refund / customer feedback 的文档化规则。

本版不是先做支付系统，而是先冻结商业边界。

### v1.2.3 - Public Release Candidate Certification

`v1.2.3` 是公开发布候选。

本版目标：

- clean-room install / run / update；
- full beta scenario certification；
- docs / onboarding / troubleshooting；
- release artifact 自包含；
- known risks and support boundary；
- public release go / no-go decision。

如果 `v1.2.3` 发现阻断问题，进入 `v1.2.4` 修复后发布。

### v1.2.4 - Public Product Release Closeout

`v1.2.4` 是公开发布收口版本，仅在 `v1.2.3` 有阻断修复时需要。

本版目标：

- 修复 RC 阻断问题；
- 关闭 public release checklist；
- 发布稳定安装包、文档和 release notes；
- 确认公开用户入口、支持边界和后续 roadmap。

如果 `v1.2.3` 已满足公开发布标准，`v1.2.4` 可以跳过或作为首个 patch release 预留。

### v1.2.5 - Published Release Certification and Registry-backed Commercial Runtime

`v1.2.5` 在公开发布后修正 release certification 与商业 Runtime source：

- release certification 区分 candidate / published；
- production commercial registry 成为 Product source；
- Commercial Product read model 从 registry/config 生成；
- negative fixtures 不能污染 production product surface；
- release-event artifact 必须带 tag、release URL、workflow run 和 source commit。

本版不做真实支付、checkout、customer account 或具体商业 SKU。

### v1.2.6 - Project-scoped Commercial Product Instance Hardening

`v1.2.6` 把 commercial registry 从 source-level 推进到 project-scoped：

- 当前项目根解析 `products/commercial-runtime/**`；
- 缺少 project commercial registry 时显示 non-ready；
- aggregate status 与 per-entry availability 分离；
- Paid Report product instance 定义 input、report definition、evidence、decision 和 delivery promise；
- allowed preflight 只能生成 Runtime proposal handoff，不能直接 run。

### v1.2.7 - Paid Report Runtime Handoff Closure

`v1.2.7` 继续 `v1.2.6`，关闭 project-scoped Paid Report Runtime handoff：

- Software Dev 保持 Managed Project Flow Reference App；
- Paid Report 只作为 generic backend handoff，不引入具体 report SKU；
- Desktop Paid Report preflight 必须使用 active project root；
- Runtime proposal 进入 admission receipt；
- run contract 必须绑定 admission receipt、input refs、report definition、evidence policy 和 decision policy；
- delivery projection 只读取 evidence / decision 状态，不写 authority；
- release certification 证明 handoff、admission、run contract 和 delivery projection 的完整链路。

本版仍不做支付、checkout、账号、真实 report generation 或公开商业上线。

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
