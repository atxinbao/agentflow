# AgentFlow Version Roadmap Draft V1

日期：2026-06-20
执行者：Codex
状态：Version Roadmap Draft / 非执行需求 / 不授权 Build Agent 执行

## 1. Purpose

本文档用于沉淀 AgentFlow 从 `v0.4.0` 到 `v1.0.0` 的版本推进路线。

目标不是把所有能力一次性塞进一个版本，而是围绕 Agent Project OS 的底层主链路，逐步完成：

```text
Project Ontology
→ Spec Loop
→ Contract
→ Build Loop
→ Agent Action Arbitration
→ Event Store
→ Projection
→ Industry Surface
```

## 2. Boundary

本文档只定义版本目标和开发顺序。

它不代表：

- 已写入正式 `docs/requirements/**`；
- 已写入 `.agentflow/spec/**`；
- 已授权 Build Agent 执行；
- 已进入当前 `v0.3.x` 审计或修复流；
- 已确定每个版本的最终 issue 数量。

每个版本进入开发前，仍必须单独生成：

```text
SPEC Draft Preview
Project Preview
Issues Preview
```

经人类确认后，才允许写入正式 requirements 和 `.agentflow/spec/**`。

## 3. Architecture Baseline

AgentFlow 的 Project OS 架构分三层：

```text
Industry Layer
  行业客户端 + Domain Pack + Surface Pack + Connector Pack

Runtime Entry
  Runtime API / SDK
  command / query / event

Runtime Core
  Project Ontology
  → Spec Loop
  → Contract
  → Build Loop
  → Action Arbitration
  → Event Store
  → Projection
```

主链路必须保持稳定：

```text
Spec Loop
→ Contract
→ Build Loop
→ Agent Action Arbitration
→ Event Store
→ Projection
```

Message Bus、Artifact Store、Cache、Connector、UI、云部署都是支撑设施，不进入主链路中心。

## 4. Target Version Count

建议用 6 个开发版本完成 Project OS 的主要能力：

```text
v0.4.0 → v0.5.0 → v0.6.0 → v0.7.0 → v0.8.0 → v0.9.0
```

然后用：

```text
v1.0.0
```

作为稳定发布标记。

判断依据：

- `v0.4.0` 只建立 Runtime Foundation；
- Spec Loop 和 Build Loop 各自都需要独立产品化；
- Projection Surface / OS Console 需要在底层事实稳定后再做；
- Pack System 必须等 Runtime API、Projection、Command Surface 稳定后再抽象；
- Message Bus 和云部署不能提前进入主链路，否则会把问题复杂化。

## 5. Version Roadmap

| Version | Target | Core Development Tasks | Out Of Scope |
| --- | --- | --- | --- |
| `v0.4.0` | Definition-driven Runtime Foundation | Ontology Registry、Action Contract、Role Policy、Object State Machine、Action Arbitration、Event Store integration、Projection read models、Runtime Command API | 行业客户端、Figma/UI 产品化、Message Bus、云端部署、完整多 Agent 调度平台 |
| `v0.5.0` | Spec Loop Productization | Requirement Classifier、Context Resolver、Boundary Checker、Route Decider、Spec Builder、Confirmation Gate、Spec-to-Action Proposal | Build Loop 并发、行业 Pack、云端 Runtime |
| `v0.6.0` | Build Loop and Controlled Multi-agent Execution | Build Agent Action Proposal、对象锁、依赖队列、Evidence Gate、状态迁移、cancel / supersede、基础多 Agent 并发 | 行业客户端市场、完整云调度、Message Bus 中心化 |
| `v0.7.0` | Projection Surface and OS Console | Project Home、Spec Workbench、Task Workbench、Event Timeline、Audit / Delivery 只读视图、Command Surface | 多行业产品壳、Pack 市场、云端多租户 |
| `v0.8.0` | Pack System and First Industry Shell | Domain Pack、Surface Pack、Connector Pack 标准；Software Dev Pack；UI Design Pack 作为第二现场试点 | 大规模行业生态、云端部署平台、复杂商业权限 |
| `v0.9.0` | Deployment Shape and Runtime Governance | 本地 / 云端 Runtime 边界、event replay、ontology migration、simulation / evaluation、跨进程调度；按真实需求判断是否引入 Message Bus | v1.0 稳定承诺、长期兼容策略、行业市场 |
| `v1.0.0` | Stable Agent Project OS Baseline | 稳定 Runtime Core、稳定 API / SDK、稳定 Pack 结构、稳定 Software Dev Surface、可复跑迁移和回放 | 新大能力扩张 |

## 6. Version Details

### 6.1 v0.4.0 - Definition-driven Runtime Foundation

目标：

```text
让 AgentFlow 先拥有统一的项目世界定义层和动作写入边界。
```

核心任务：

- 建立 Project Ontology Registry；
- 定义 Action Contract；
- 定义 Agent Role Policy；
- 定义 Object State Machine；
- 建立基础 Action Arbitration；
- 接入 Event Store；
- 建立 Projection Read Models；
- 建立 Runtime Command API；
- 清理旧 Workflow / Capability / Projection 术语冲突；
- 完成 Runtime Foundation closeout。

完成判定：

- Runtime 可以读取 ontology；
- Action Proposal 可以被 contract / role / state 校验；
- Arbitration 可以输出 accept / reject / queue / requireHumanDecision / cancel / supersede；
- 事件可以进入 Event Store；
- Projection 可以从事件和定义层重建只读状态；
- 旧文档不再把 Projection 当事实源。

### 6.2 v0.5.0 - Spec Loop Productization

目标：

```text
让 AgentFlow 能稳定理解人类输入，并把需求转成可确认的 Project / Spec / Issue / Action Proposal。
```

核心任务：

- Requirement Intake Normalizer；
- Requirement Classifier；
- Context Resolver；
- Boundary Checker；
- Route Decider；
- Preview Generator；
- Confirmation Gate；
- Spec Builder；
- Spec-to-Action Proposal；
- Spec Loop projection。

完成判定：

- 用户输入可以被分类为 question / research / feature / bug / audit / design-only / executable issue 等类型；
- 系统可以识别需求歧义、冲突、边界和执行风险；
- 系统可以生成 SPEC Draft Preview / Project Preview / Issues Preview；
- 未确认前不写正式事实源；
- 确认后可以生成符合 Runtime Foundation 的 Action Proposal。

### 6.3 v0.6.0 - Build Loop and Controlled Multi-agent Execution

目标：

```text
让 Build Loop 不再只是单线程任务执行，而是受 Arbitration 保护的多 Agent 执行系统。
```

核心任务：

- Build Agent Action Proposal；
- Issue / Object lock；
- Dependency queue；
- Evidence Gate；
- State transition enforcement；
- cancel / supersede / retry / resume；
- 基础多 Agent 并发；
- Build Loop event model；
- Build Loop projection；
- Done writeback boundary。

完成判定：

- Build Agent 不能绕过 Action Contract；
- 多 Agent 写入必须经过 Arbitration；
- 冲突动作必须进入 rejected / queued / requireHumanDecision；
- evidence 不足时不能完成状态迁移；
- Build Loop 完成不自动触发 Audit；
- 每次状态变化都可以从 Event Store 回放。

### 6.4 v0.7.0 - Projection Surface and OS Console

目标：

```text
让软件开发场景拥有可用的 Project OS Console。
```

核心任务：

- Project Home；
- Spec Workbench；
- Task Workbench；
- Event Timeline；
- Runtime Status；
- Audit read-only view；
- Delivery read-only view；
- Command Surface；
- Projection query API；
- Desktop UI 只读事实改造。

完成判定：

- UI 只读 Projection，不直接修改事实；
- 用户操作通过 Command Surface 回流 Runtime API；
- Project / Spec / Issue / Run / Audit / Delivery 状态能被统一展示；
- 软件开发项目可以从需求、拆解、执行、证据、审计、交付完成闭环；
- 到本版本，AgentFlow 应该成为软件开发场景可用的 Project OS。

### 6.5 v0.8.0 - Pack System and First Industry Shell

目标：

```text
把底层 Runtime 和行业现场分离，让行业只是 Runtime 上的客户端壳和定义包。
```

核心任务：

- Domain Pack 标准；
- Surface Pack 标准；
- Connector Pack 标准；
- Pack manifest；
- Pack validation；
- Software Dev Pack；
- UI Design Pack；
- Pack-aware Projection Surface；
- Pack-aware Command Surface。

完成判定：

- Runtime Core 不硬编码行业现场；
- Software Dev 和 UI Design 可以使用不同 Surface；
- 两个行业场景都通过同一 Runtime API / Projection / Action Proposal 进入系统；
- Domain Pack 定义对象和动作；
- Surface Pack 定义呈现和交互；
- Connector Pack 定义外部工具和文件系统连接。

### 6.6 v0.9.0 - Deployment Shape and Runtime Governance

目标：

```text
让 AgentFlow 从单机工具变成可本地部署、也可云端部署的 Runtime。
```

核心任务：

- Local Runtime boundary；
- Cloud Runtime boundary；
- Runtime API hardening；
- Event replay；
- Ontology versioning / migration；
- Simulation / evaluation；
- Runtime governance；
- Cross-process scheduling；
- Message Bus decision gate；
- Deployment evidence and rollback model。

完成判定：

- 同一 Runtime Core 可以本地部署，也可以云端部署；
- Event Store 可以支持 replay；
- Ontology 升级有 migration 路径；
- Action 执行前可以 simulation；
- Message Bus 是否引入由真实跨进程调度需求决定；
- 行业客户端通过 API / SDK 接入，而不是绑定到单一桌面应用。

### 6.7 v1.0.0 - Stable Agent Project OS Baseline

目标：

```text
冻结 Agent Project OS 的稳定底座。
```

核心任务：

- Runtime Core API freeze；
- Pack schema freeze；
- Event schema freeze；
- Projection schema freeze；
- migration baseline；
- documentation baseline；
- release verification baseline；
- Software Dev Surface baseline。

完成判定：

- 新行业可以通过 Pack 接入，不需要修改 Runtime Core；
- 软件开发场景可以完整闭环；
- UI Design 场景可以作为第二现场证明架构没有被软件开发硬编码；
- Runtime 可以通过本地和云端两种部署形态运行；
- 关键事实可以 replay；
- v1.0 后新增能力不能破坏主链路。

## 7. Dependency Order

```text
v0.4.0 Runtime Foundation
  ↓
v0.5.0 Spec Loop Productization
  ↓
v0.6.0 Build Loop + Multi-agent Control
  ↓
v0.7.0 Projection Surface + OS Console
  ↓
v0.8.0 Pack System + Industry Shell
  ↓
v0.9.0 Deployment + Governance
  ↓
v1.0.0 Stable Baseline
```

不能倒置的依赖：

- 没有 `v0.4.0`，`v0.5.0` 的 Spec Loop 没有标准对象、动作和状态语言；
- 没有 `v0.5.0`，`v0.6.0` 的 Build Loop 不知道从哪里接收合法任务；
- 没有 `v0.6.0`，`v0.7.0` 的 OS Console 只能展示静态状态，不能安全发起命令；
- 没有 `v0.7.0`，`v0.8.0` 的行业壳没有稳定 Projection 和 Command Surface；
- 没有 `v0.8.0`，`v0.9.0` 的云端和 Message Bus 会过早绑定软件开发单一场景。

## 8. Completion Standard

到 `v0.7.0`：

```text
AgentFlow 应该成为软件开发场景可用的 Project OS。
```

到 `v0.9.0`：

```text
AgentFlow 应该具备跨行业 Agent Project Runtime 的底层能力。
```

到 `v1.0.0`：

```text
AgentFlow 应该冻结一套稳定的 Agent Project OS baseline。
```

## 9. Next Step

当前最先进入正式开发的仍是：

```text
v0.4.0 Definition-driven Runtime Foundation
```

下一步应该把 `v0.4.0` 的 10 个技术设计转成正式：

```text
SPEC Draft Preview
Project Preview
Issues Preview
```

等待确认后，再写入：

```text
docs/requirements/**
.agentflow/spec/projects/**
.agentflow/spec/issues/**
```
