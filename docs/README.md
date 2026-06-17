# Docs Index

更新日期：2026-06-17
执行者：Codex

## 当前文档入口

| 路径 | 作用 |
| --- | --- |
| `product/` | AgentFlow Project Operating Model 产品设计基线 |
| `foundation/` | 下一代 Project Operating Model 的基础能力需求 |
| `requirements/` | 新需求文档入口，后续开发只从这里开始 |
| `archive/2026-05-agentflow-legacy/` | 旧需求、旧规划、旧规格和旧验证摘要归档 |

## 默认阅读顺序

1. [../README.md](../README.md)
2. [../GOAL.md](../GOAL.md)
3. [../ROADMAP.md](../ROADMAP.md)
4. [product/README.md](product/README.md)
5. [foundation/README.md](foundation/README.md)
6. [requirements/README.md](requirements/README.md)
7. [requirements/next-requirements.md](requirements/next-requirements.md)
8. [requirements/009-input-model-v1.md](requirements/009-input-model-v1.md)
9. [requirements/010-execute-patch-checkpoint-v1.md](requirements/010-execute-patch-checkpoint-v1.md)

## 规则

- `archive/` 下文档不作为后续开发依据。
- `product/` 下文档作为产品模型和项目设计基线，不直接等同于实现任务。
- `foundation/` 下文档作为下一代基础能力需求，不混入当前版本迭代队列。
- 新功能、新页面、新数据模型和新验收标准必须写入 `requirements/`。
- 从 `product/` 派生实现时，必须先转成 `requirements/` 下的开发需求切片。
- 从 `foundation/` 进入当前版本开发时，必须再明确拆成当前版本可执行需求。
- 未进入 `requirements/` 的旧文档内容不能自动转化为 issue 或实现任务。
