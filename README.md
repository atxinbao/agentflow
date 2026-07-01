# AgentFlow

更新日期：2026-07-01
执行者：Codex

## Project Goal

AgentFlow 的总目标是：

```text
AgentFlow = Spec-Driven AI OS Project
```

一句话：

```text
Agent 只是执行器，Spec 才是方向盘。
```

AgentFlow 不是单一行业工具，不是 Agent Runner，也不是 Spec 文档生成器。AgentFlow 要做的是让用户或团队用 Spec 驱动项目对象、动作、证据、验收、交付、反馈和投影，最终形成可控、可追踪、可验收、可交付的 AI 项目操作系统。

商业产品目标：

```text
AgentFlow 不先卖 Agent。
AgentFlow 卖的是可交付结果。
```

## System Formula

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

每个 Industry Product Surface 由内置 Domain Pack、Surface Pack 和 Connector Pack 支撑。

Core OS Runtime 提供通用项目运行能力。Industry Product Surface 定义具体行业的输入、对象、页面或报告、工具和交付方式。

最直接的商业入口是 Paid Report Flow：

```text
用户输入信息
-> 支付
-> AgentFlow 内部完成一次受控 Run
-> 交付可验收报告
```

Software Dev 是第一个官方 Reference App，用来证明 Core OS Runtime 可以跑完整闭环；它不是 Core OS 的内核目标，也不能把 `Issue / PR / Release / Patch` 等行业词汇写死进 Core。

这些 Pack 是 AgentFlow App 的内置能力，不是用户项目里的可见文档目录。用户项目只记录当前启用的 Pack 引用、confirmed Spec Bundle、执行合同、运行事实、证据和投影。

源码边界：

```text
crates/**     = Core OS Runtime
products/**   = Industry Product Surface / Reference App source
apps/**       = user-facing clients
docs/**       = human-readable records
.agentflow/** = Runtime fact source
```

Software Dev 可以内置发布，但正式行业壳源码应属于 `products/software-dev/**`。`crates/pack/fixtures/**` 只作为测试夹具。

## Canonical Entries

| 入口 | 作用 |
| --- | --- |
| [docs/project/goal.md](docs/project/goal.md) | 当前项目总目标 |
| [docs/project/roadmap.md](docs/project/roadmap.md) | 从目标到版本阶段的路线图 |
| [docs/architecture/021-ai-os-project-core-capabilities-v1.md](docs/architecture/021-ai-os-project-core-capabilities-v1.md) | AI OS Project 底层通用能力 |
| [docs/architecture/builtin-pack-registry.md](docs/architecture/builtin-pack-registry.md) | App 内置 Pack Registry 边界 |
| [docs/architecture/086-industry-product-source-boundary-v1.md](docs/architecture/086-industry-product-source-boundary-v1.md) | Core / Product / App / Runtime fact source 源码边界 |
| [docs/README.md](docs/README.md) | 当前文档地图 |
| [docs/delivery/releases/v1.0.8/README.md](docs/delivery/releases/v1.0.8/README.md) | 当前发布基线 |
| [CHANGELOG.md](CHANGELOG.md) | 当前 changelog 指针 |

## Current Boundary

- `docs/` 面向人类团队、第三方集成方和 Spec Builder。
- `.agentflow/` 面向 Agent、Runtime、Projection、Decision Gate 和 Audit Agent。
- `crates/**` 是 Core OS Runtime 源码。
- `products/**` 是行业壳 / Reference App 源码。
- `apps/**` 是用户界面源码。
- `docs/project/**` 定义产品目标和产品边界。
- `docs/project/roadmap.md` 定义版本路线图，不直接授权实现。
- `docs/architecture/**` 定义底层能力，不直接授权实现。
- 内置 Pack 由 AgentFlow App 管理，不写入 `docs/project/**`。
- `docs/requirements/**` 只保存后续 confirmed Spec Bundle。
- `.agentflow/spec/**` 才是执行合同事实源。
- Audit 是独立 sidecar，不回到主业务链。

历史文档已归档到 [docs/project/history/2026-06-current-baseline-history/](docs/project/history/2026-06-current-baseline-history/)。
