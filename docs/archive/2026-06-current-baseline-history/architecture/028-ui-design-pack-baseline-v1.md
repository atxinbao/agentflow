# UI Design Pack Baseline V1

创建日期：2026-06-23
执行者：Codex

## 目标

UI Design Pack 是 AgentFlow 的第二个正式行业壳。

它证明 Pack System 不只服务代码执行，也能表达设计工作流：

```text
Product Brief
-> Direction
-> Wireframe
-> HiFi
-> Design System
-> Handoff
```

UI Design Pack 可以连接 Figma、image assets、frontend repo 和 browser preview，但 V1 不承诺完整生产级 Figma adapter。

## Pack 边界

UI Design Pack 只能定义：

- 设计领域对象；
- 设计对象关系；
- 设计状态机；
- 设计 action semantics；
- handoff acceptance semantics；
- handoff evidence policy；
- 设计 surface pages；
- 设计 connector capabilities；
- simulation dry-run 覆盖。

UI Design Pack 不能：

- 复用 Software Dev 的 `Issue / Run` 作为唯一业务对象；
- 把设计现场伪装成代码任务执行；
- 直接写 `.agentflow/spec/**`；
- 直接写 `.agentflow/events/**`；
- 直接写 `.agentflow/tasks/**`；
- 直接调用 Figma 写入；
- 承诺完整前端设计客户端。

## Domain Baseline

UI Design Domain 至少包含：

```text
ProductBrief
Direction
Wireframe
HiFi
DesignSystem
Page
Handoff
Evidence
```

主链关系：

```text
ProductBrief -> Direction
Direction -> Wireframe
Wireframe -> HiFi
HiFi -> DesignSystem
DesignSystem -> Handoff
Handoff -> Evidence
```

Handoff 的 evidence policy 必须要求：

```text
visual-preview
design-system-ref
handoff
```

## Surface Baseline

主 Surface：

```text
Design Home
Brief Intake
Direction Board
Wireframe Board
HiFi Review
Design System
Handoff Surface
```

命令入口必须挂到对应页面：

| Surface | Command |
| --- | --- |
| Brief Intake | `design.brief.capture` |
| Direction Board | `design.direction.select` |
| Wireframe Board | `design.wireframe.generate` |
| HiFi Review | `design.hifi.review` |
| Design System | `design.system.bind` |
| Handoff Surface | `design.handoff.accept` |

## Connector Baseline

UI Design Pack 初始 connector：

```text
Figma
Image Assets
Frontend Repo
Design Export
Browser Preview
```

这些 connector 只能提供 capability 和 evidence output，不得写 Runtime authority。

## Simulation Baseline

Pack simulation 必须至少覆盖一个 design command。

V1 使用 `design.wireframe.generate` 证明：

- simulation 是 read-only；
- 不写 authority；
- 不写 event store；
- 不启动 provider；
- 能输出 expected event preview；
- 能输出 affected projection preview。

## 实现位置

- `crates/pack/src/domain.rs`
- `crates/pack/src/surface.rs`
- `crates/pack/src/connector.rs`
- `crates/simulation/src/lib.rs`
- `crates/projection/src/query.rs`

## 验收

- UI Design Pack 有独立 domain、surface、connector 定义；
- UI Design Pack 不复用 Software Dev `Issue / Run` 作为唯一业务对象；
- Handoff 输出明确 evidence policy；
- Pack simulation 覆盖 `design.wireframe.generate`；
- Projection 能看到 UI Design Pack 的独立 read model。
