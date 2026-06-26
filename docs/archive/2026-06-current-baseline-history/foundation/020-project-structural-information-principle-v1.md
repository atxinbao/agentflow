# Project Structural Information Principle V1

日期：2026-06-23
执行者：Codex

## Purpose

AgentFlow 不应该把所有上下文都当成同等价值的信息。

本原则用于判断一段输入、日志、证据、Pack 定义、模拟输出或 release artifact 是否值得进入 AgentFlow 的项目事实链。

核心判断：

```text
只有能降低未来 Agent 判断成本的信息，才应该被提升为项目结构信息。
```

换句话说：

```text
Raw context
-> structured project information
-> reusable agent knowledge
-> verifiable evidence
-> acceptance-ready project facts
```

## Reference

本原则参考：

- Marc Finzi, Shikai Qiu, Yiding Jiang, Pavel Izmailov, J. Zico Kolter, Andrew Gordon Wilson, “From Entropy to Epiplexity: Rethinking Information for Computationally Bounded Intelligence”, arXiv:2601.03220, https://arxiv.org/abs/2601.03220

该论文把随机、不可预测的信息和可学习、可泛化的结构信息分开讨论。AgentFlow 不实现论文中的数学 estimator，也不引入正式 epiplexity 计算；这里只取工程原则：项目事实应优先保存有结构、可验证、可复用的信息，而不是保存更多原始噪音。

## Principle

AgentFlow 中的信息晋升必须经过结构判断。

一段信息只有满足以下轻量维度，才可以进入事实源、证据、Pack 定义或 release artifact：

| Dimension | Meaning | Fails When |
| --- | --- | --- |
| structured | 有明确对象、字段、关系或阶段 | 只是大段聊天、无边界日志、随机输出 |
| bounded | 有范围、大小、生命周期和责任边界 | 无限 transcript dump、全量环境快照 |
| reusable | 后续 Agent 可以复用它减少判断成本 | 只能服务一次性解释，不能被后续流程引用 |
| verifiable | 可以通过命令、文件、hash、状态或审计检查 | 只有口头结论，没有可检查证据 |
| traceable | 可以追溯来源、转换、消费者和影响面 | 不知道从哪里来，也不知道服务哪个阶段 |
| role-readable | 对指定 Agent 角色可读、可执行、可判断 | 只有人类能猜懂，Agent 角色边界不清 |

失败的信息不应被提升为 authority。它可以留在原始诊断、临时日志或人工讨论中，但不能伪装成项目事实。

## Scope

### Spec Builder

Spec Builder 负责把人类输入转成可确认的需求结构。

必须保留：

- 明确目标；
- 明确非目标；
- 约束；
- 验收标准；
- 依赖；
- 冲突；
- 需要人类确认的缺口。

必须降级或丢弃：

- 重复描述；
- 与目标无关的聊天噪音；
- 无法绑定到项目对象的泛泛背景；
- 无边界 transcript dump；
- Agent 自己生成但没有确认来源的推断。

Spec Builder 的输出必须帮助后续 Goal / Plan / Issue / Work Loop 降低判断成本。

### Evidence Pack

证据不是越多越好。

Evidence Pack 应按以下问题判断证据是否可以晋升：

- 它证明哪个合同字段？
- 它对应哪个命令、文件、状态或 artifact？
- 它是否可复跑、可定位或可审计？
- 它是否能被 Acceptance Gate 使用？
- 它是否只是噪音更大的日志？

大体积日志如果不能映射到合同、验收或失败原因，只能作为诊断附件，不应成为 acceptance-ready evidence。

### Pack System

Pack 是行业结构压缩，不是配置倾倒。

Domain Pack 应定义：

- objects；
- links；
- actions；
- states；
- acceptance semantics；
- evidence policy。

Surface Pack 应定义：

- 页面；
- view model；
- command entry；
- status presentation；
- sidecar read surface。

Connector Pack 应定义：

- external capability；
- provider boundary；
- smoke requirement；
- command mapping；
- rejected / disabled reason。

Connector 输出不能直接成为 authority。Pack 只能让项目世界更可读、可验证、可执行。

### Simulation / Dry-run

Simulation 输出可以有价值，但必须保持非权威。

Simulation report 必须标记：

- generated；
- replayable；
- bounded；
- non-authoritative；
- affected objects；
- expected events；
- missing evidence；
- conflict / risk。

Simulation 不能替代执行结果，不能替代 evidence，不能替代 human confirmation。

### Acceptance Gate

Done 不是命令成功。

Done 需要结构闭环：

```text
verification passed
-> evidence complete
-> contract satisfied
-> state transition allowed
-> traceability connected
```

Acceptance Gate 应拒绝以下情况：

- 只有命令输出，没有合同映射；
- 只有大段日志，没有 evidence index；
- 只有 PR/MR 事实，没有 Done writeback；
- 只有 simulation，没有实际执行证据；
- 只有 Pack readiness 结论，没有 registry / validation / projection / capability trace。

### Runtime Governance

Runtime governance 不只是权限和执行规则。

它还必须判断信息质量：

- 这个 action proposal 引用的信息是否结构化？
- 这个 evidence 是否可追溯？
- 这个 connector 输出是否越过 authority boundary？
- 这个 Pack command 是否有完整 command / capability / projection 映射？
- 这个 release artifact 是否能证明项目状态，而不是只重复口头结论？

## Promotion Checklist

任何信息晋升到事实源、证据、Pack 定义或 release artifact 前，至少要回答：

| Check | Question | Required Result |
| --- | --- | --- |
| Structure | 它是否有明确对象、字段、关系或阶段？ | yes |
| Boundary | 它是否有范围、生命周期和责任边界？ | yes |
| Reuse | 后续 Agent 是否能复用它减少判断成本？ | yes |
| Verification | 它是否能被命令、文件、hash、状态或审计检查？ | yes |
| Trace | 它是否能追溯来源、转换和消费者？ | yes |
| Role | 指定 Agent 角色是否能读懂并执行下一步？ | yes |

如果任一项为 `no`，该信息只能保持为 raw context、diagnostic output 或 temporary note，不能晋升为 authority。

## Explicit Boundaries

本原则不授权：

- 实现正式 epiplexity estimator；
- 增加 ML training、dataset scoring 或模型评估基础设施；
- 把 AgentFlow 改成通用数据治理工具；
- 因为上下文存在就存更多原始内容；
- 绕过 Pack validation；
- 绕过 Acceptance Gate；
- 绕过 Runtime authority boundary；
- 手写 `.agentflow/**` task facts。

## Version Impact

本原则直接影响：

- `v0.8.1` Pack Registry / Resolver / Projection / Capability / Release Gate negative fixtures；
- `v0.9.0` Runtime Governance Policy；
- 后续 Evidence Pack 和 Acceptance Gate refinements。

它不是单独的 runtime feature，而是后续版本判断“什么信息值得进入项目事实链”的基础规则。
