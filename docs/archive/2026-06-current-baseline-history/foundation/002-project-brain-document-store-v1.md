# 002 - Project Brain Document Store V1

创建日期：2026-06-17
执行者：Codex

## Source Foundation

本文从 [001-goal-agent-project-brain-v1.md](001-goal-agent-project-brain-v1.md) 派生。

## Purpose

本文定义 Project Brain 文档的本地存储边界。

Project Brain Document Store 是项目大脑的人类可读事实层。它保存 Goal / Plan / Decision，不保存执行状态，不替代 `.agentflow/` 里的结构化执行事实。

## Canonical Location

建议每个 Project 使用独立目录：

```text
docs/projects/<project-id>/
  GOAL.md
  PLAN.md
  DECISIONS.md
```

可选扩展：

```text
docs/projects/<project-id>/
  PROJECT_HEALTH.md
  DELIVERY.md
  EVIDENCE.md
```

## Project ID Rule

`<project-id>` 必须稳定，不能从标题动态推断。

推荐格式：

```text
project-<slug>
```

示例：

```text
docs/projects/project-agentflow-v1/GOAL.md
docs/projects/project-ecommerce-app/GOAL.md
docs/projects/project-login-button-repair/GOAL.md
```

## Document Ownership

| 文档 | Owner | 用途 |
| --- | --- | --- |
| GOAL.md | Goal Agent + Human | 项目方向 |
| PLAN.md | Goal Agent + Spec Agent + Human | 项目路径 |
| DECISIONS.md | Goal Agent + Human | 确认记录 |
| PROJECT_HEALTH.md | Goal Agent | 项目健康观察 |
| DELIVERY.md | Delivery Agent + Human | 交付摘要 |
| EVIDENCE.md | Audit / Delivery | 证据索引 |

## Store Rules

- GOAL.md / PLAN.md / DECISIONS.md 是 Project Brain 的最小集合。
- 文档缺失时，Project Brain status 不能是 ready。
- 文档可以被预览生成，但用户确认前不能写入。
- 文档写入必须留下 DECISIONS.md 记录。
- Project Brain 文档不承载 Issue 执行状态。
- Project Brain 文档不承载 runtime lease、run、checkpoint、audit execution state。

## Read Model

建议读取后形成：

```text
ProjectBrainDocumentSet
```

字段：

```text
projectId
rootPath
goalPath
planPath
decisionsPath
goalExists
planExists
decisionsExists
goalUpdatedAt
planUpdatedAt
decisionsUpdatedAt
missingDocuments
readonly
```

## Write Boundary

V1 不自动写入。

后续如果实现写入，必须经过：

```text
Draft Preview
-> Human Confirm
-> Write GOAL.md / PLAN.md / DECISIONS.md
-> Record Decision
```

不允许：

- 从 raw requirement 直接写文档。
- 从 Goal Draft 直接生成 Issue。
- 写入 `.agentflow/` runtime data。
- 覆盖已有文档且不保留 Decision 记录。

## Preview Boundary

Preview 是草稿，不是事实源。

Preview 可以包含：

- GOAL.md Draft
- PLAN.md Draft
- DECISIONS.md Draft Entry

Preview 不允许：

- 不作为 Project ready 依据。
- 不作为 Work Loop 输入。
- 不生成 Issue。
- 不写 runtime 状态。

## Acceptance Criteria

- [ ] Project Brain 文档路径稳定。
- [ ] GOAL.md / PLAN.md / DECISIONS.md 最小集合明确。
- [ ] 文档 owner 明确。
- [ ] 读取模型明确。
- [ ] Preview 与 Write 边界明确。
- [ ] 不写 `.agentflow/` runtime data。
- [ ] 不进入 Work Loop。
- [ ] 不生成 Issue。
