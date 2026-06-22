# AgentFlow v0.8.0 Pack System Tasks V1

日期：2026-06-23
执行者：Codex

## Goal

`v0.8.0` 聚焦 Pack System 和第一批行业壳。

主线是：

```text
Pack Filesystem Contract
-> Pack Manifest
-> Domain Pack
-> Surface Pack
-> Connector Pack
-> Pack Validation
-> Pack Simulation
-> Pack-aware Projection
-> Pack-aware Command Surface
-> Industry Shell Baselines
-> Release Gate Certification
```

这不是 Cloud Runtime 版本，也不是继续扩张 Console 页面的版本。

本版本要回答的是：

```text
AgentFlow 如何用同一套 Runtime Core 支撑不同项目现场？
```

## Product Principle

AgentFlow 的行业化不是把所有行业塞进一个软件界面。

正确结构是：

```text
底层 Runtime Core 复用
行业客户端独立设计
行业能力通过 Pack 接入
```

Pack 的职责：

| Pack | Responsibility |
| --- | --- |
| Domain Pack | 定义行业对象、关系、动作、状态、验收语义 |
| Surface Pack | 定义行业页面、工作台、视图模型、命令入口 |
| Connector Pack | 定义外部工具、文件系统、provider、MCP 和 capability mapping |

## Main Chain

`v0.8.0` 的 Runtime 主链仍然不能被 Pack 改写：

```text
Spec Loop
-> Contract
-> Build / Work Loop
-> Arbitration
-> Event Store
-> Projection
```

Pack 只能在标准化边界上提供定义：

```text
行业输入
-> Pack 定义
-> Runtime API / Command Surface
-> Runtime Core
-> Projection Surface
-> 行业客户端输出
```

## Issues

| Issue | Title | Priority | Dependency | Status |
| --- | --- | --- | --- | --- |
| `V080-001` | Foundation Carryover Corrections | P0 | none | planned |
| `V080-002` | Pack System Filesystem Contract | P0 | V080-001 | planned |
| `V080-003` | Pack Manifest Schema and Registry | P0 | V080-002 | planned |
| `V080-004` | Domain Pack Contract | P0 | V080-003 | planned |
| `V080-005` | Surface Pack Contract | P0 | V080-003 | planned |
| `V080-006` | Connector Pack Contract and Capability Mapping | P0 | V080-003 | planned |
| `V080-007` | Pack Validation, Versioning, and Migration Preview | P0 | V080-004, V080-005, V080-006 | planned |
| `V080-008` | Pack Simulation and Dry-run Integration | P0 | V080-007 | planned |
| `V080-009` | Pack-aware Projection Read Models | P0 | V080-007 | planned |
| `V080-010` | Pack-aware Command Surface and Runtime API Plane | P0 | V080-008, V080-009 | planned |
| `V080-011` | Software Dev Pack Baseline | P1 | V080-010 | planned |
| `V080-012` | UI Design Pack Baseline | P1 | V080-010 | planned |
| `V080-013` | Pack Release Gate and Readiness Certification | P1 | V080-011, V080-012 | planned |

## V080-001 Foundation Carryover Corrections

### Scope

收口 `v0.7.2` 审计中发现的底层问题，避免 Pack System 建在含糊的 foundation 上。

必须处理：

- Message Bus envelope 的唯一 ID、时间戳和 trace 信息；
- Capability Registry 对 provider smoke 状态的消费路径；
- Migration Receipt 语义，区分 preview receipt 和 applied receipt；
- Simulation 的 Completion Commit 链路表达；
- release evidence 中 GitHub source archive 与 AgentFlow fact source 的边界说明；
- v0.7.2 release 文档中仍然含糊的 completed / baseline / deferred 表述。

### Acceptance

- 相关 foundation 文档、schema 或测试能证明上述边界已收口；
- Pack System 文档不再依赖未定义的 v0.7.2 行为；
- 不引入 Pack 功能实现。

## V080-002 Pack System Filesystem Contract

### Scope

定义 Pack 在项目文件系统中的位置、职责和边界。

建议目标形态：

```text
.agentflow/
  packs/
    software-dev/
      pack.json
      domain/
      surface/
      connectors/
    ui-design/
      pack.json
      domain/
      surface/
      connectors/
```

本 issue 只定义 contract，不直接迁移现有 authority。

### Acceptance

- 明确 Pack 文件不能替代 `.agentflow/spec/**`、`.agentflow/events/**`、`.agentflow/tasks/**`；
- 明确 Pack 是定义层，不是执行结果；
- 明确 Pack 进入 Runtime 的唯一入口是 Runtime API / Command Surface；
- 文档说明 pack path、manifest path、domain path、surface path、connector path。

## V080-003 Pack Manifest Schema and Registry

### Scope

建立 Pack manifest schema 和 registry。

`pack.json` 至少需要表达：

- pack id；
- name；
- type；
- version；
- runtime compatibility；
- domain path；
- surface path；
- connector path；
- required capabilities；
- owned object types；
- exposed commands；
- projection entries；
- migration policy；
- validation status。

### Acceptance

- Pack Registry 可以列出本地 Pack；
- manifest schema 可以校验必填字段；
- Runtime 可以读取 Pack metadata，但不能因为读取 Pack 就写 authority；
- API Plane manifest 能看到 Pack 相关 entry。

## V080-004 Domain Pack Contract

### Scope

定义行业对象世界。

Domain Pack 必须能表达：

- object types；
- link types；
- state machines；
- action semantics；
- acceptance semantics；
- evidence policy；
- audit trigger hints；
- migration compatibility。

Software Dev Pack 的初始 domain：

```text
Requirement / Spec / Issue / Run / PR / Release / Evidence / Finding
```

UI Design Pack 的初始 domain：

```text
Product Brief / PRD / Direction / Wireframe / HiFi / Design System / Page / Handoff / Evidence
```

### Acceptance

- Domain Pack 不能直接写事件；
- Domain Pack 输出的是可执行定义；
- Domain Pack action semantics 可以被 Contract / Arbitration / Simulation 读取；
- Software Dev 和 UI Design 的 domain 差异能被 schema 表达。

## V080-005 Surface Pack Contract

### Scope

定义行业客户端呈现和交互表面。

Surface Pack 必须能表达：

- page registry；
- workbench registry；
- view model mapping；
- command entry mapping；
- read model dependencies；
- navigation rules；
- empty / loading / error state；
- sidecar surfaces。

Software Dev Pack 的主 Surface：

```text
Project Home / Spec Workbench / Task Workbench / Acceptance / Delivery / Event Timeline / Evidence Graph
```

Software Dev Pack 的 sidecar Surface：

```text
Audit Surface / Finding Review / Follow-up Proposal
```

UI Design Pack 的主 Surface：

```text
Design Home / Brief Intake / Direction Board / Wireframe Board / HiFi Review / Design System / Handoff Surface
```

### Acceptance

- Surface Pack 只读 Projection 或发 Command；
- Surface Pack 不能直接写 authority；
- Software Dev Audit Surface 必须标记为 sidecar，不得进入主业务链路；
- UI Design Surface 能独立表达设计现场，不复用 Task Workbench 伪装成设计流程。

## V080-006 Connector Pack Contract and Capability Mapping

### Scope

定义 Pack 如何声明外部工具和 provider 能力。

Connector Pack 必须能表达：

- connector id；
- provider type；
- supported actions；
- required capability；
- health source；
- smoke policy；
- evidence output；
- disabled reason；
- command boundary。

Software Dev Pack 初始 Connector：

```text
GitHub / Git / Codex / Claude / Browser Preview
```

UI Design Pack 初始 Connector：

```text
Figma / image assets / frontend repo / design export / browser preview
```

### Acceptance

- Connector Pack 输出不能直接成为 authority；
- 外部写动作必须转换为 Runtime command；
- capability registry 能读取 Pack capability requirement；
- provider smoke status 能影响 Pack command availability。

## V080-007 Pack Validation, Versioning, and Migration Preview

### Scope

建立 Pack validation 和 Pack migration preview。

Validation 至少检查：

- manifest schema；
- domain references；
- surface mappings；
- connector capability requirements；
- API Plane mapping；
- version compatibility；
- missing read models；
- missing command mappings。

Migration preview 至少输出：

- from version；
- to version；
- affected objects；
- affected projections；
- required human confirmation；
- preview receipt；
- applied receipt boundary。

### Acceptance

- invalid Pack 不能被 Runtime 标记为 active；
- migration preview 默认不写 authority；
- applied migration 必须有明确确认和 applied receipt；
- release gate 能读取 Pack validation artifact。

## V080-008 Pack Simulation and Dry-run Integration

### Scope

让 Pack command 在真实执行前能 dry-run。

Simulation 至少输出：

- command；
- target object；
- expected events；
- affected projections；
- required capabilities；
- required evidence；
- rejection reasons；
- conflict preview；
- acceptance impact。

### Acceptance

- dry-run 不写 authority；
- dry-run 不触发 provider；
- dry-run 可以识别缺失 connector、缺失 surface mapping、缺失 read model；
- Simulation 能覆盖 Software Dev 和 UI Design 至少各一条 command。

## V080-009 Pack-aware Projection Read Models

### Scope

让 Projection 能按 Pack 暴露行业对象和行业视图。

Projection 至少支持：

- pack list；
- active pack；
- pack validation status；
- domain object index；
- surface page index；
- connector capability index；
- industry workbench read model。

### Acceptance

- Projection 仍然只读；
- Projection 不以 Pack 文件为 authority；
- Projection 可以解释 Software Dev 和 UI Design 的不同对象；
- Desktop / CLI 能看到 Pack readiness 状态。

## V080-010 Pack-aware Command Surface and Runtime API Plane

### Scope

让 Command Surface 和 Runtime API Plane 支持 Pack command。

至少支持：

- list pack commands；
- validate pack command；
- dry-run pack command；
- submit pack action proposal；
- query pack capability status；
- query pack surface route。

### Acceptance

- Pack command 不能绕过 Action Contract；
- Pack command 不能绕过 Arbitration；
- API Plane manifest 标记 Pack command 的 authority / readonly / command / internal；
- invalid Pack command 必须给出可读 rejection reason。

## V080-011 Software Dev Pack Baseline

### Scope

把当前软件开发现场作为第一个正式行业壳。

Software Dev Pack 主链：

```text
Requirement
-> Spec
-> Issue
-> Run
-> Acceptance
-> Delivery
-> Release
```

Audit 独立 sidecar：

```text
Delivery / Done
-> Optional Audit Request
-> Audit Report
-> Finding
-> Follow-up Proposal
```

### Acceptance

- Software Dev Pack 能表达现有软件开发 Project OS 现场；
- Audit 不阻塞 Requirement -> Release 主链；
- Finding 只能生成 follow-up proposal，不能直接改 Done；
- Pack readiness artifact 能证明 Software Dev Pack 可加载、可验证、可投影。

## V080-012 UI Design Pack Baseline

### Scope

建立第二个试点行业壳，证明 AgentFlow 不只服务代码执行。

UI Design Pack 主链：

```text
Product Brief
-> Direction
-> Wireframe
-> HiFi
-> Design System
-> Handoff
```

它可以连接 Figma / image assets / frontend repo / browser preview，但本版本不承诺完整生产级 Figma adapter。

### Acceptance

- UI Design Pack 有独立 domain、surface、connector 定义；
- UI Design Pack 不复用 Software Dev Issue / Run 作为唯一业务对象；
- Handoff 能输出 evidence policy；
- Pack simulation 至少覆盖一个 design command。

## V080-013 Pack Release Gate and Readiness Certification

### Scope

把 Pack System 纳入 release gate。

Release gate 必须输出：

- pack-registry.json；
- pack-validation-report.json；
- pack-simulation-report.json；
- pack-projection-readiness.json；
- pack-api-plane-manifest entry；
- software-dev-pack-readiness.json；
- ui-design-pack-readiness.json。

### Acceptance

- release gate 失败时不能发布 Pack System ready 结论；
- readiness report 明确 completed / baseline / deferred / carryover；
- Software Dev Pack 和 UI Design Pack 都有证据；
- 文档说明 Audit sidecar 不属于 Software Dev Pack 主链阻塞条件。

## Boundary

这些 issue 是 `v0.8.0` 的开发任务基线。

不得在本批次中实现：

- Cloud Runtime；
- remote Agent fleet；
- Pack marketplace；
- 自动远程审计；
- Message Bus 中心化；
- 多行业商业权限；
- 完整生产级外部 connector 套件。

## First Executable Issue

第一条可执行 issue 应该是：

```text
V080-001 Foundation Carryover Corrections
```

原因：

```text
Pack System 不能建立在含糊的 Runtime Foundation 上。
```

`V080-001` 完成后再进入 `V080-002`，定义 Pack filesystem contract。
